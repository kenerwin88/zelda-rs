//! CPU-only PCM observation: cold captures and explicitly marked census seeds.
//! ZELDA3_DEBUG_PCM_WINDOWS is a JSON plan: output directory and inclusive
//! zero-based host ranges. Existing output files are never overwritten.
use std::{fs::{self, File}, io::{BufRead, BufReader, BufWriter, Write}, path::PathBuf};
use serde::Deserialize;
use zelda3::debug_env;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    output: PathBuf,
    ranges: Vec<[u32; 2]>,
    /// Optional source logical input sequence, consumed only after all native
    /// presentation holds finish. Never imports source machine state or RNG.
    logical_inputs: Option<PathBuf>,
    /// Cartridge tables used by the exact CPU/HDMA clock, never executed here.
    rom: Option<PathBuf>,
    /// Retain instruction boundaries only inside the selected host ranges.
    #[serde(default)]
    instructions: bool,
    /// Explicit identity for an extended trace core. Initial state and every
    /// PCM callback must still match the cache's pinned original exactly.
    trace_core_sha256: Option<String>,
}

#[derive(Deserialize)]
struct OracleFrame {
    frame: u32,
    input: String,
    audio: parity::digest::AudioDigest,
}

fn create(path: PathBuf) -> Result<BufWriter<File>> {
    Ok(BufWriter::new(File::options().write(true).create_new(true).open(path)?))
}

/// The same raw live-register observer for the original and seeded native lane.
/// A reset is explicit: no note can continue across a fresh SPC driver.
pub(crate) struct NoteCapture {
    output: PathBuf,
    events: zstd::stream::write::Encoder<'static, BufWriter<File>>,
    notes: zstd::stream::write::Encoder<'static, BufWriter<File>>,
    hashes: zstd::stream::write::Encoder<'static, BufWriter<File>>,
    pcm: BufWriter<File>,
    metadata: BufWriter<File>,
    instructions: Option<zstd::stream::write::Encoder<'static, BufWriter<File>>>,
    ranges: Vec<[u32; 2]>,
    registers: [u8; 128],
    pub(crate) samples: u64,
    frames: u64,
    writes: u64,
    edges: u64,
    bytes: u64,
}

impl NoteCapture {
    pub(crate) fn from_env() -> Option<Self> {
        let path = debug_env::var_os("ZELDA3_DEBUG_PCM_WINDOWS")?;
        Some(Self::open(&serde_json::from_slice::<Plan>(&fs::read(path).expect("read capture plan")).expect("parse capture plan"))
            .expect("open note capture"))
    }

    fn open(plan: &Plan) -> Result<Self> {
        if plan.ranges.iter().any(|[lo, hi]| lo > hi) { return Err("invalid PCM range".into()); }
        fs::create_dir_all(&plan.output)?;
        Ok(Self {
            output: plan.output.clone(),
            events: zstd::stream::write::Encoder::new(create(plan.output.join("dsp.jsonl.zst"))?, 3)?,
            notes: zstd::stream::write::Encoder::new(create(plan.output.join("notes.jsonl.zst"))?, 3)?,
            hashes: zstd::stream::write::Encoder::new(create(plan.output.join("hashes.jsonl.zst"))?, 3)?,
            pcm: create(plan.output.join("windows.s16le"))?,
            metadata: create(plan.output.join("frames.jsonl"))?,
            instructions: if plan.instructions { Some(zstd::stream::write::Encoder::new(
                create(plan.output.join("instructions.jsonl.zst"))?, 3)?) } else { None },
            ranges: plan.ranges.clone(), registers: [0; 128], samples: 0,
            frames: 0, writes: 0, edges: 0, bytes: 0,
        })
    }

    pub(crate) fn wants_instructions(&self) -> bool {
        self.instructions.is_some() && self.ranges.iter().any(|[lo, hi]|
            u64::from(*lo) <= self.frames && self.frames <= u64::from(*hi))
    }

    pub(crate) fn record_instructions(&mut self, trace: impl serde::Serialize) -> Result<()> {
        if let Some(writer) = self.instructions.as_mut() {
            serde_json::to_writer(&mut *writer, &serde_json::json!({
                "host": self.frames, "sample": self.samples, "trace": trace
            }))?;
            writer.write_all(b"\n")?;
        }
        Ok(())
    }

    pub(crate) fn record(&mut self, iteration: u64, reset: bool, context: [u16; 3],
        music: serde_json::Value, writes: &[(i64, u8, u8, u8)], audio: &[i16], oracle_match: bool) -> Result<()> {
        if reset { self.registers.fill(0); }
        let host = self.frames;
        for &(sample, phase, address, value) in writes {
            if address >= 128 { continue; }
            self.registers[usize::from(address)] = value;
            if address != 0x4c && address != 0x5c { continue; }
            for voice in 0..8 {
                if value & (1 << voice) == 0 { continue; }
                let b = voice * 16;
                serde_json::to_writer(&mut self.notes, &serde_json::json!({
                    "host": host, "iteration": iteration, "sample": sample, "phase": phase,
                    "kind": if address == 0x4c { "on" } else { "off" }, "voice": voice,
                    "source": self.registers[b + 4],
                    "pitch": u16::from_le_bytes([self.registers[b + 2], self.registers[b + 3] & 63]),
                    "volume": [self.registers[b] as i8, self.registers[b + 1] as i8],
                    "adsr": [self.registers[b + 5], self.registers[b + 6]], "gain": self.registers[b + 7]
                }))?;
                self.notes.write_all(b"\n")?;
                self.edges += 1;
            }
        }
        // Include empty callbacks: resets and route context must not disappear
        // just because the chip wrote no register in that callback.
        serde_json::to_writer(&mut self.events, &serde_json::json!({
            "host": host, "iteration": iteration, "sample": self.samples,
            "reset": reset, "context": context, "music": music, "writes": writes
        }))?;
        self.events.write_all(b"\n")?;
        let mut hash = parity::hash::Sha256Digest::new();
        hash.update_i16_le(audio);
        let hash = hash.finish();
        serde_json::to_writer(&mut self.hashes, &serde_json::json!({
            "host": host, "sample_frames": audio.len() / 2, "sha256": hash, "oracle_match": oracle_match
        }))?;
        self.hashes.write_all(b"\n")?;
        if self.ranges.iter().any(|[lo, hi]| u64::from(*lo) <= host && host <= u64::from(*hi)) {
            if self.bytes + audio.len() as u64 * 2 > 256 * 1024 * 1024 { return Err("PCM cap exceeded".into()); }
            for sample in audio { self.pcm.write_all(&sample.to_le_bytes())?; }
            serde_json::to_writer(&mut self.metadata, &serde_json::json!({
                "host": host, "iteration": iteration, "sample": self.samples,
                "byte_offset": self.bytes, "sample_frames": audio.len() / 2, "sha256": hash,
                "oracle_match": oracle_match, "context": context
            }))?;
            self.metadata.write_all(b"\n")?;
            self.bytes += audio.len() as u64 * 2;
        }
        self.samples += audio.len() as u64 / 2;
        self.frames += 1;
        self.writes += writes.len() as u64;
        Ok(())
    }

    pub(crate) fn finish(mut self, reference: bool, extra: serde_json::Value) -> Result<()> {
        if let Some(writer) = self.instructions.take() { writer.finish()?.flush()?; }
        self.events.finish()?.flush()?;
        self.notes.finish()?.flush()?;
        self.hashes.finish()?.flush()?;
        self.pcm.flush()?;
        self.metadata.flush()?;
        fs::write(self.output.join("complete.json"), serde_json::to_vec_pretty(&serde_json::json!({
            "frames": self.frames, "sample_frames": self.samples, "dsp_writes": self.writes,
            "note_edges": self.edges, "pcm_bytes": self.bytes, "reference_verified": reference,
            "lag_free": !reference, "sample_rate": 32000, "channels": 2, "extra": extra
        }))?)?;
        Ok(())
    }
}

fn original(args: &[String]) -> Result<()> {
    use crate::libretro_core::{LibretroCore, LIBRETRO_AUDIO_ONLY, RETRO_MEMORY_SYSTEM_RAM};
    use std::sync::atomic::Ordering;
    if args.len() != 4 { return Err("usage: --render-original-audio-cpu CACHE CORE ROM FRAMES".into()); }
    let cache = PathBuf::from(&args[0]);
    let plan: Plan = serde_json::from_slice(&fs::read(debug_env::var_os("ZELDA3_DEBUG_PCM_WINDOWS")
        .ok_or("missing capture plan")?)?)?;
    let manifest: serde_json::Value = serde_json::from_slice(&fs::read(cache.join("cache-manifest.json"))?)?;
    let trace_identity = plan.trace_core_sha256.as_ref().map(|hash| serde_json::json!(hash));
    for (path, expected) in [(PathBuf::from(&args[1]), trace_identity.as_ref().unwrap_or(&manifest["cache_identity"]["core_sha256"])),
        (PathBuf::from(&args[2]), &manifest["cache_identity"]["rom_sha256"]),
        (cache.join("initial.srm"), &manifest["artifact_sha256"]["initial.srm"]),
        (cache.join("oracle-av-hashes.jsonl"), &manifest["artifact_sha256"]["oracle-av-hashes.jsonl"])] {
        if parity::hash::sha256_file(&path)? != expected.as_str().ok_or("missing bound hash")? {
            return Err(format!("provenance hash mismatch: {}", path.display()).into());
        }
    }
    let mut capture = NoteCapture::open(&plan)?;
    // Sparse bus receipts share the PCM selection, but retain the core's raw
    // cycle/phase fields so command arrival and gameplay polls can be compared
    // without rounding either boundary to a callback or an output sample.
    let mut ports = create(plan.output.join("ports.jsonl"))?;
    let frames: u32 = args[3].parse()?;
    // Conservative exclusive lock, even though this mode never renders.
    let _lock = crate::oracle_lock::acquire_snes9x_compare_lock_mode(true);
    LIBRETRO_AUDIO_ONLY.store(true, Ordering::Relaxed);
    let sram = fs::read(cache.join("initial.srm"))?;
    let mut core = LibretroCore::load_with_sram(&args[1], &args[2], Some(&sram))?;
    if core.library_name != "Snes9x" || !core.library_version.starts_with("1.63")
        || !(32000.0..32100.0).contains(&core.av_info.timing.sample_rate) {
        return Err(format!("unexpected core identity/timing: {} {} {}", core.library_name,
            core.library_version, core.av_info.timing.sample_rate).into());
    }
    // Serialization affects the pinned DSP. Match the cache's initial
    // serialization exactly once, before any retro_run; never checkpoint.
    let initial = core.serialize_state()?;
    if parity::hash::sha256_bytes(&initial) != manifest["artifact_sha256"]["oracle_initial.state"].as_str().ok_or("initial state hash")? {
        return Err("initial original state differs from cache".into());
    }
    let mut ledger = BufReader::new(File::open(cache.join("oracle-av-hashes.jsonl"))?).lines();
    let started = std::time::Instant::now();
    for host in 0..frames {
        let row: OracleFrame = serde_json::from_str(&ledger.next().ok_or("oracle EOF")??)?;
        if row.frame != host { return Err("oracle host sequence".into()); }
        let input = u16::from_str_radix(row.input.trim_start_matches("0x"), 16)?;
        let audio = core.run_frame_with_input(input).audio;
        let mut hash = parity::hash::Sha256Digest::new();
        hash.update_i16_le(&audio);
        if hash.finish() != row.audio.sha256 || audio.len() as u64 / 2 != row.audio.sample_frames {
            capture.finish(false, serde_json::json!({"failure_host": host, "kind": "oracle_pcm_mismatch"}))?;
            return Err(format!("original PCM mismatch at {host}").into());
        }
        let raw = core.debug_dsp_register_writes().ok_or("core lacks DSP register trace")?;
        if capture.wants_instructions() {
            let instructions = core.debug_smp_instructions().ok_or("core lacks SMP instruction trace")?;
            if instructions.len() >= 8192 { return Err("SMP instruction trace capacity reached".into()); }
            if instructions.iter().any(|instruction| instruction.hardware[15] > 4) {
                return Err("SMP MMIO trace capacity reached".into());
            }
            capture.record_instructions(instructions)?;
        }
        if plan.ranges.iter().any(|[lo, hi]| (*lo..=*hi).contains(&host)) {
            let cpu = core.debug_apu_port_writes_exact()?.ok_or("core lacks APUI trace")?;
            let smp = core.debug_smp_output_port_writes_exact()?.ok_or("core lacks SMP port trace")?;
            serde_json::to_writer(&mut ports, &serde_json::json!({
                "host": host, "sample": capture.samples, "cpu": cpu, "smp": smp
            }))?;
            ports.write_all(b"\n")?;
        }
        // The pinned buffer holds 1024 writes; a full buffer is ambiguous.
        if raw.len() >= 1024 { return Err("DSP trace capacity reached".into()); }
        let writes: Vec<_> = raw.iter().map(|w| (capture.samples as i64 + i64::from(w.output_sample),
            w.dsp_phase as u8, w.register as u8, w.value as u8)).collect();
        let ram = core.memory_bytes(RETRO_MEMORY_SYSTEM_RAM).ok_or("missing WRAM")?;
        capture.record(u64::from(host), false, [ram[0x10] as u16, ram[0x11] as u16,
            u16::from_le_bytes([ram[0xa0], ram[0xa1]])], serde_json::json!({
                "music_control": ram[0x130], "last_music_control": ram[0x133]
            }), &writes, &audio, true)?;
        if host % 10000 == 0 { eprintln!("original host={host} seconds={:.1}", started.elapsed().as_secs_f64()); }
    }
    ports.flush()?;
    capture.finish(true, serde_json::json!({"oracle_eof": ledger.next().is_none(),
        "core_sha256": parity::hash::sha256_file(&PathBuf::from(&args[1]))?,
        "pinned_core_sha256": manifest["cache_identity"]["core_sha256"], "seconds": started.elapsed().as_secs_f64()}))
}

pub(crate) fn run_original(args: &[String]) {
    if let Err(error) = original(args) { eprintln!("original CPU audio: {error}"); std::process::exit(2); }
}

fn render(args: &[String]) -> Result<()> {
    let started = std::time::Instant::now();
    if !(3..=4).contains(&args.len()) || (args.len() == 4 && args[3] != "--lag-free") {
        return Err("usage: --render-audio-cpu ORACLE_HASH_LEDGER INITIAL_SRAM FRAMES [--lag-free]".into());
    }
    let path = debug_env::var_os("ZELDA3_DEBUG_PCM_WINDOWS").ok_or("ZELDA3_DEBUG_PCM_WINDOWS must name a JSON plan")?;
    let plan: Plan = serde_json::from_slice(&fs::read(path)?)?;
    let frames: u32 = args[2].parse()?;
    if frames == 0 || plan.ranges.iter().any(|[lo, hi]| lo > hi || *hi >= frames) {
        return Err("invalid PCM host range or frame count".into());
    }
    fs::create_dir_all(&plan.output)?;
    let mut pcm = create(plan.output.join("windows.s16le"))?;
    let mut metadata = create(plan.output.join("frames.jsonl"))?;
    let mut commands = create(plan.output.join("commands.jsonl"))?;
    let mut events = zstd::stream::write::Encoder::new(create(plan.output.join("dsp.jsonl.zst"))?, 3)?;
    let mut proof = zstd::stream::write::Encoder::new(create(plan.output.join("hashes.jsonl.zst"))?, 3)?;
    let mut notes = zstd::stream::write::Encoder::new(create(plan.output.join("notes.jsonl.zst"))?, 3)?;
    let mut registers = [0u8; 128];
    let lag_free = args.len() == 4;
    if plan.logical_inputs.is_some() && !lag_free {
        return Err("logical inputs require --lag-free".into());
    }
    #[derive(Deserialize)]
    struct Iteration { index: usize, joypad: Option<u16> }
    let logical_inputs = plan.logical_inputs.as_ref().map(|path| -> Result<Vec<u16>> {
        let mut inputs = Vec::new();
        for (index, line) in BufReader::new(File::open(path)?).lines().enumerate() {
            let row: Iteration = serde_json::from_str(&line?)?;
            if row.index != index { return Err("noncontiguous logical inputs".into()); }
            if index != 0 { inputs.push(row.joypad.ok_or("missing logical joypad")?.reverse_bits()); }
        }
        inputs.push(0);
        Ok(inputs)
    }).transpose()?;
    let mut game = if lag_free {
        let mut game = crate::load_romless_play_state();
        game.set_lag_free(true);
        game
    } else {
        crate::load_default_play_state()
    };
    if !lag_free {
        let rom = plan.rom.as_ref().ok_or("reference requires cartridge tables in plan.rom")?;
        game.set_rom(&fs::read(rom)?);
    }
    let sram = fs::read(&args[1])?;
    if sram.len() != game.sram.len() { return Err("initial SRAM size mismatch".into()); }
    game.sram.copy_from_slice(&sram);
    let mut oracle = BufReader::new(File::open(&args[0])?).lines();
    let mut audio = Vec::new();
    let mut byte_offset = 0u64;
    let mut sample_offset = 0u64;
    let mut latch = [0u8; 4];
    let mut iteration = 0usize;
    let mut completed = 0u32;
    let mut note_count = 0u64;
    let mut dsp_write_count = 0u64;
    for host in 0..frames {
        if logical_inputs.as_ref().is_some_and(|inputs| iteration >= inputs.len()) { break; }
        let row: OracleFrame = serde_json::from_str(&oracle.next().ok_or("oracle ledger ended early")??)?;
        if row.frame != host || row.audio.channels != 2 { return Err("noncontiguous or nonstereo oracle ledger".into()); }
        let input = if let Some(inputs) = &logical_inputs {
            *inputs.get(iteration).ok_or("logical input sequence ended early")?
        } else {
            u16::from_str_radix(row.input.strip_prefix("0x").ok_or("invalid input")?, 16)?
        };
        // discard_unused_audio_frames may reset the queue's diagnostic count.
        // Compare across this callback, never across that reset boundary.
        let push = game.zelda_last_pushed_apu_ports().0;
        game.zelda_run_frame(i32::from(input));
        let (next_push, ports) = game.zelda_last_pushed_apu_ports();
        if next_push != push {
            if next_push.wrapping_sub(push) != 1 {
                return Err(format!("multiple command publications at host {host}; snapshot cannot prove trigger alignment").into());
            }
            let ram = game.wram_image();
            for port in 0..4 {
                if ports[port] != 0 && ports[port] != latch[port] {
                    serde_json::to_writer(&mut commands, &serde_json::json!({
                        "host": host, "iteration": iteration, "sample": sample_offset, "port": port, "value": ports[port],
                        "context": [ram[0x10], ram[0x11], u16::from_le_bytes([ram[0xa0], ram[0xa1]])],
                        "message": u16::from_le_bytes([ram[0x1cf0], ram[0x1cf1]])
                    }))?;
                    commands.write_all(b"\n")?;
                }
            }
            latch = ports;
        }
        let samples = if lag_free { 534 } else { usize::try_from(row.audio.sample_frames)? };
        audio.resize(samples * 2, 0);
        let frame = game.zelda_render_audio(&mut audio, samples as i32, 2);
        let writes: Vec<_> = frame.events.iter().filter_map(|event| event.parity_dsp)
            .map(|write| (sample_offset as i64 + i64::from(write.sample_offset), write.timer_cycles, write.addr, write.value))
            .collect();
        dsp_write_count += writes.len() as u64;
        for &(sample, phase, address, value) in &writes {
            if address >= 128 { continue; }
            registers[usize::from(address)] = value;
            if address != 0x4c && address != 0x5c { continue; }
            for voice in 0..8 {
                if value & (1 << voice) == 0 { continue; }
                let base = voice * 16;
                serde_json::to_writer(&mut notes, &serde_json::json!({
                    "host": host, "iteration": iteration, "sample": sample, "phase": phase,
                    "kind": if address == 0x4c { "on" } else { "off" }, "voice": voice,
                    "source": registers[base + 4],
                    "pitch": u16::from_le_bytes([registers[base + 2], registers[base + 3] & 0x3f]),
                    "volume": [registers[base] as i8, registers[base + 1] as i8],
                    "adsr": [registers[base + 5], registers[base + 6]], "gain": registers[base + 7]
                }))?;
                notes.write_all(b"\n")?;
                note_count += 1;
            }
        }
        if !writes.is_empty() {
            let ram = game.wram_image();
            serde_json::to_writer(&mut events, &serde_json::json!({
                "host": host, "iteration": iteration, "sample": sample_offset,
                "context": [ram[0x10] as u16, ram[0x11] as u16, u16::from_le_bytes([ram[0xa0], ram[0xa1]])],
                "music": frame.music, "writes": writes
            }))?;
            events.write_all(b"\n")?;
        }
        game.zelda_discard_unused_audio_frames();
        crate::gpu_capture::capture_gpu_frame_from_game(&mut game);
        let mut hash = parity::hash::Sha256Digest::new();
        hash.update_i16_le(&audio);
        let hash = hash.finish();
        serde_json::to_writer(&mut proof, &serde_json::json!({"host": host, "sample_frames": samples,
            "sha256": hash, "oracle_match": !lag_free && hash == row.audio.sha256}))?;
        proof.write_all(b"\n")?;
        let selected = plan.ranges.iter().any(|[lo, hi]| (*lo..=*hi).contains(&host));
        if selected {
            if byte_offset + (audio.len() * 2) as u64 > 256 * 1024 * 1024 {
                return Err("selected PCM exceeds the 256 MiB per-run limit; use sparse windows".into());
            }
            for sample in &audio { pcm.write_all(&sample.to_le_bytes())?; }
            serde_json::to_writer(&mut metadata, &serde_json::json!({
                "host": host, "sample": sample_offset, "byte_offset": byte_offset,
                "sample_frames": samples, "sha256": hash, "oracle_sha256": row.audio.sha256,
                "oracle_match": !lag_free && hash == row.audio.sha256,
                "iteration": iteration,
                "context": [game.wram_image()[0x10] as u16, game.wram_image()[0x11] as u16,
                    u16::from_le_bytes([game.wram_image()[0xa0], game.wram_image()[0xa1]])]
            }))?;
            metadata.write_all(b"\n")?;
            byte_offset += (audio.len() * 2) as u64;
        }
        if !lag_free && hash != row.audio.sha256 {
            pcm.flush()?; metadata.flush()?; commands.flush()?;
            events.finish()?.flush()?;
            proof.finish()?.flush()?;
            notes.finish()?.flush()?;
            fs::write(plan.output.join("failure.json"), serde_json::to_vec_pretty(&serde_json::json!({
                "host": host, "verified_frames": host, "verified_sample_frames": sample_offset,
                "native_sha256": hash, "oracle_sha256": row.audio.sha256,
                "context": [game.wram_image()[0x10] as u16, game.wram_image()[0x11] as u16,
                    u16::from_le_bytes([game.wram_image()[0xa0], game.wram_image()[0xa1]])],
                "spc_clock": game.zelda_spc_driver_clock_debug_summary(),
                "seconds": started.elapsed().as_secs_f64()
            }))?)?;
            return Err(format!("reference PCM mismatch at host {host}: native={hash} oracle={}", row.audio.sha256).into());
        }
        sample_offset += samples as u64;
        completed = host + 1;
        if !game.native_presentation_pending() { iteration += 1; }
        if host % 1000 == 0 { eprintln!("PCM host={host} bytes={byte_offset} reference_verified={}", !lag_free); }
    }
    pcm.flush()?; metadata.flush()?; commands.flush()?;
    events.finish()?.flush()?;
    proof.finish()?.flush()?;
    notes.finish()?.flush()?;
    fs::write(plan.output.join("complete.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "frames": completed, "lag_free": lag_free, "sample_rate": 32000, "channels": 2,
        "requested_frames": frames, "oracle_eof": oracle.next().is_none(),
        "logical_inputs_consumed": logical_inputs.as_ref().map(|inputs| iteration >= inputs.len()),
        "dsp_writes": dsp_write_count, "note_edges": note_count,
        "sample_frames": sample_offset, "pcm_bytes": byte_offset, "reference_verified": !lag_free,
        "seconds": started.elapsed().as_secs_f64(), "iterations": iteration,
        "logical_inputs": plan.logical_inputs
    }))?)?;
    Ok(())
}

pub(crate) fn run(args: &[String]) {
    if let Err(error) = render(args) {
        eprintln!("CPU PCM: {error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    /// The study's 32 kHz capture must observe the same live chip/mixer work
    /// as the shipped 44.1 kHz callback. Only its final resampling may differ.
    #[test]
    fn native_pcm_matches_the_product_callback_before_resampling() {
        let mut product = crate::load_romless_play_state();
        product.set_lag_free(true);
        product.sram.fill(0);
        let mut native = product.clone();
        let mut a = vec![0i16; 735 * 2];
        let mut b = vec![0i16; 534 * 2];
        for host in 0..1000 {
            product.zelda_run_frame(0);
            native.zelda_run_frame(0);
            product.zelda_render_audio(&mut a, 735, 2);
            native.zelda_render_audio(&mut b, 534, 2);
            let mut position = 0.0f32;
            for sample in 0..735 {
                let index = (position as usize).min(533);
                assert_eq!(&a[sample * 2..sample * 2 + 2], &b[index * 2..index * 2 + 2], "host {host}, sample {sample}");
                position += 534.0 / 735.0;
            }
            product.zelda_discard_unused_audio_frames();
            native.zelda_discard_unused_audio_frames();
            crate::gpu_capture::capture_gpu_frame_from_game(&mut product);
            crate::gpu_capture::capture_gpu_frame_from_game(&mut native);
            assert_eq!(product.wram_image(), native.wram_image());
            assert_eq!(product.zelda_spc_driver_clock_debug_summary(), native.zelda_spc_driver_clock_debug_summary());
        }
    }
}
