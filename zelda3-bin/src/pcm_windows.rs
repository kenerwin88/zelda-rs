//! CPU-only cold PCM observation. Never installs receipts or restores state.
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
