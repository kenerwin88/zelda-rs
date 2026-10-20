//! Record the ROM's logical-frame ledger for lag-free parity checks.
//!
//! Usage: cargo run -p snes --example logical_frame_ledger -- ROM SRAM INPUT HOST_CALLS OUT_DIR
//!
//! Runs the cartridge from reset on the in-repo source CPU (the owner whose
//! presented OAM matches the pinned Snes9x cache) and cuts the route into main
//! loop iterations instead of host frames. An iteration ends at an NMI accepted
//! while `$12` is clear, i.e. Main finished its work; an NMI with `$12` set is
//! a lag NMI and stays inside the iteration.
//!
//! `OUT_DIR/iterations.jsonl` has one record per iteration: the host that
//! accepted its opening NMI, the joypad word that handler latched, and the
//! timing-derived values the iteration consumed (cartridge RNG outputs and APU
//! port reads). A lag-free engine replays these per iteration rather than per
//! host. `wram-pre-nmi.xor.zst` and `wram-post-nmi.xor.zst` are streams of
//! 128 KiB WRAM images, each XORed with the previous one, taken when the
//! iteration's closing NMI is accepted and when it returns to the wait loop.
//!
//! `ZELDA3_LEDGER_SEED_EVERY=N` also writes `seeds/iter-<k>.bin` at every Nth
//! post-NMI image: WRAM, VRAM, CGRAM and OAM (see `write_seed`), so a lag-free
//! replay can restart at iteration k after a timing-driven sequence (the intro's
//! polygon thread, an APU poll) moved the engine off the ROM's iteration count.
//! It also seeds the first iteration after each timing-defined stretch (the
//! polygon thread running, or the intro or attract module) and lists those
//! iterations in `seeds/resync.txt`, so the replay can realign right there.
//!
//! Each iteration records the host whose scanned-out frame first shows its
//! work (`picture_host`), so it is compared with exactly that Snes9x frame.
//! `ZELDA3_LEDGER_PICTURES=1` also draws every frame with the port's line
//! renderer from the ROM's PPU state, line by line (HDMA included): each
//! iteration then carries that picture's RGB hash and `host-pictures.txt`
//! lists it per host for checking against the cache (it matches Snes9x on
//! most frames; it is a diagnostic, not the authority).
//! `ZELDA3_LEDGER_FRAMES=<lo>-<hi>` (implies pictures) writes
//! `frames/host-<n>.ppm`.

use sha2::{Digest, Sha256};
use snes::{Snes9xColdCpuExecutor, SourceCpuAcceptedInterrupt, SourceCpuBusAccessKind};
use std::{
    env,
    error::Error,
    fs,
    io::{BufWriter, Write},
    path::PathBuf,
};

const WRAM_LEN: usize = 0x20000;

fn in_wait_loop(pc: u32) -> bool {
    pc & 0x7f_0000 == 0 && (0x8034..=0x8037).contains(&(pc & 0xffff))
}

/// `$12`: Main clears it when its iteration is complete; the NMI handler
/// performs the frame's updates only then (otherwise the NMI is a lag NMI).
const NMI_BOOLEAN: usize = 0x12;

const RTI: u8 = 0x40;
const MAIN_MODULE: usize = 0x10;
const NMI_THREAD_ACTIVE: usize = 0x12a;

/// Whether the cartridge is in a sequence that advances per NMI rather than
/// per iteration (see `docs/parity/lagless-logical-parity.md`).
fn timing_defined(ram: &[u8]) -> bool {
    ram[NMI_THREAD_ACTIVE] != 0 || matches!(ram[MAIN_MODULE], 0x00 | 0x14)
}

/// Whether an NMI opens a new iteration. The handler decides by `$12`, not by
/// where it interrupted: Main can finish (clear `$12`) and be interrupted
/// before it reaches the `$00:8034` wait loop. `ZELDA3_LEDGER_WAIT_LOOP_BOUNDARY`
/// restores the older wait-loop PC test for comparison.
fn opens_iteration(interrupted_pc: u32, nmi_boolean: u8) -> bool {
    if env::var_os("ZELDA3_LEDGER_WAIT_LOOP_BOUNDARY").is_some() {
        in_wait_loop(interrupted_pc)
    } else {
        nmi_boolean == 0
    }
}

/// The WRAM offset a CPU bus address reaches: banks $7E-$7F, or the low 8 KB
/// mirror in the system banks.
fn wram_offset(address: u32) -> Option<u32> {
    match address >> 16 {
        0x7e | 0x7f => Some(address & 0x1_ffff),
        _ if system_bank(address) && address & 0xffff < 0x2000 => Some(address & 0x1fff),
        _ => None,
    }
}

fn system_bank(address: u32) -> bool {
    (address >> 16) & 0x7f < 0x40
}

#[derive(Default, serde::Serialize)]
struct Iteration {
    index: u64,
    /// Host that accepted the NMI opening this iteration.
    open_host: usize,
    /// Host that renders the first active-display scanline after that NMI:
    /// the frame that first shows this iteration's NMI uploads.
    display_host: Option<usize>,
    /// Libretro joypad word installed for `open_host`.
    host_buttons: u16,
    /// `$4218/$4219` as read by the opening NMI handler (`None` if it did not read).
    joypad: Option<u16>,
    /// Lag NMIs accepted while Main was still working.
    lag_nmis: u32,
    irqs: u32,
    /// The RNG routine's `$0FA1` stores with the carry the final `ADC` left.
    rng: Vec<(u8, bool)>,
    /// `$2140-$2143` reads outside NMI: `(port, value)`.
    apu_reads: Vec<(u8, u8)>,
    /// `$2140-$2143` writes: `(port, value)`.
    apu_writes: Vec<(u8, u8)>,
    /// Host that accepted the closing NMI.
    close_host: usize,
    /// Host that scans out the first frame after the closing NMI returned:
    /// the frame showing this iteration's work (cache frame `picture_host`).
    picture_host: Option<usize>,
    /// Scanline at which the closing NMI handler returned. A line inside the
    /// active display (below 225) means the handler overran vblank and its
    /// register writes tore the frame that shows this iteration.
    nmi_return_line: u16,
    /// That frame's RGB SHA-256 from the port's line renderer, when
    /// `ZELDA3_LEDGER_PICTURES=1` (diagnostic; usually equals the cache).
    #[serde(skip_serializing_if = "Option::is_none")]
    picture_sha256: Option<String>,
}

struct XorStream {
    previous: Vec<u8>,
    scratch: Vec<u8>,
    encoder: zstd::Encoder<'static, BufWriter<fs::File>>,
}

impl XorStream {
    fn create(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        let encoder = zstd::Encoder::new(BufWriter::new(fs::File::create(path)?), 6)?;
        Ok(Self {
            previous: vec![0; WRAM_LEN],
            scratch: vec![0; WRAM_LEN],
            encoder,
        })
    }

    fn push(&mut self, wram: &[u8]) -> Result<(), Box<dyn Error>> {
        for ((out, now), before) in self.scratch.iter_mut().zip(wram).zip(&self.previous) {
            *out = now ^ before;
        }
        self.previous.copy_from_slice(wram);
        self.encoder.write_all(&self.scratch)?;
        Ok(())
    }

    fn finish(self) -> Result<(), Box<dyn Error>> {
        self.encoder.finish()?.flush()?;
        Ok(())
    }
}

/// Seed layout: 128 KiB WRAM, 64 KiB VRAM (little-endian words), 512 bytes
/// CGRAM (little-endian words), 544 bytes OAM, cartridge SRAM, then the
/// PPU register state as JSON (`snes::PpuState`, which the engine also uses).
fn write_seed(path: PathBuf, snes: &snes::Snes) -> Result<(), Box<dyn Error>> {
    let mut bytes = Vec::with_capacity(WRAM_LEN + 0x10000 + 0x200 + 0x220);
    bytes.extend_from_slice(&snes.ram[..WRAM_LEN]);
    bytes.extend(snes.ppu.vram.iter().flat_map(|word| word.to_le_bytes()));
    bytes.extend(snes.ppu.cgram.iter().flat_map(|word| word.to_le_bytes()));
    bytes.extend(snes.ppu.oam.iter().flat_map(|word| word.to_le_bytes()));
    bytes.extend_from_slice(&snes.cart.ram);
    let mut ppu = snes.ppu.clone();
    ppu.render_buffer = None;
    bytes.extend(serde_json::to_vec(&ppu)?);
    // DMA/HDMA channel registers sit beside the seed: a sequence's one-time
    // channel setup (the world map's perspective, an iris) is not in WRAM.
    fs::write(path.with_extension("dma.json"), serde_json::to_vec(&snes.dma)?)?;
    fs::write(path, bytes)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let rom = fs::read(args.next().ok_or("missing ROM path")?)?;
    let sram = fs::read(args.next().ok_or("missing SRAM path")?)?;
    let input = fs::read_to_string(args.next().ok_or("missing input path")?)?;
    let host_calls: usize = args.next().ok_or("missing host call count")?.parse()?;
    let out_dir = PathBuf::from(args.next().ok_or("missing output directory")?);
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let mut buttons = vec![0u16; host_calls];
    for line in input
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let (range, value) = line.split_once(' ').ok_or("invalid input line")?;
        let (start, end) = range.split_once("..").unwrap_or((range, range));
        let start: usize = start.parse()?;
        let end: usize = end.parse()?;
        if start < host_calls {
            buttons[start..=end.min(host_calls - 1)]
                .fill(u16::from_str_radix(value.trim_start_matches("0x"), 16)?);
        }
    }
    fs::create_dir_all(&out_dir)?;
    let mut records = BufWriter::new(fs::File::create(out_dir.join("iterations.jsonl"))?);
    let mut pre_nmi = XorStream::create(out_dir.join("wram-pre-nmi.xor.zst"))?;
    let mut post_nmi = XorStream::create(out_dir.join("wram-post-nmi.xor.zst"))?;
    let progress_every = env::var("ZELDA3_SOURCE_PROGRESS_EVERY")
        .ok()
        .map(|value| value.parse::<usize>())
        .transpose()?
        .filter(|&value| value != 0);

    let seed_every = env::var("ZELDA3_LEDGER_SEED_EVERY")
        .ok()
        .map(|value| value.parse::<u64>())
        .transpose()?
        .filter(|&value| value != 0);
    if seed_every.is_some() {
        fs::create_dir_all(out_dir.join("seeds"))?;
    }
    let frame_hosts = env::var("ZELDA3_LEDGER_FRAMES")
        .ok()
        .map(|value| {
            let (lo, hi) = value.split_once('-').unwrap_or((&value, &value));
            Ok::<_, Box<dyn Error>>((lo.parse::<usize>()?, hi.parse::<usize>()?))
        })
        .transpose()?;
    // `ZELDA3_LEDGER_WATCH=<wram offset hex>[,...]` prints every CPU write to
    // those WRAM bytes with the iteration, host and writing PC.
    let watch: Vec<u32> = env::var("ZELDA3_LEDGER_WATCH")
        .map(|value| value.split(',').filter_map(|a| u32::from_str_radix(a.trim_start_matches("0x"), 16).ok()).collect())
        .unwrap_or_default();
    let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset_with_sram(&rom, Some(&sram))?;
    let render_pictures = frame_hosts.is_some() || env::var_os("ZELDA3_LEDGER_PICTURES").is_some();
    cpu.enable_frame_capture(render_pictures);
    if frame_hosts.is_some() {
        fs::create_dir_all(out_dir.join("frames"))?;
    }
    let mut host_pictures = render_pictures
        .then(|| fs::File::create(out_dir.join("host-pictures.txt")).map(BufWriter::new))
        .transpose()?;
    // Finished iterations wait here for the frame that shows their work.
    let mut awaiting_picture: std::collections::VecDeque<Iteration> = Default::default();
    let mut host_picture: Option<String> = None;
    drop(cpu.take_dsp_samples());
    // `current` accumulates the iteration Main is running; `opening` is the
    // iteration whose NMI handler is running. Image j of each stream is taken
    // at the j-th wait-loop NMI: pre-NMI image j ends iteration j-1, post-NMI
    // image j starts iteration j.
    let mut current: Option<Iteration> = None;
    let mut opening: Option<Iteration> = None;
    let mut in_lag_handler = false;
    let mut interrupt_depth = 0u32;
    let mut in_timing_defined = false;
    let mut resync = seed_every
        .map(|_| fs::File::create(out_dir.join("seeds").join("resync.txt")).map(BufWriter::new))
        .transpose()?;
    // Iteration (by index) still waiting for its first displayed scanline.
    let mut awaiting_display: Option<u64> = None;
    let mut display_hosts: std::collections::HashMap<u64, usize> = Default::default();
    let mut next_index = 0u64;
    let mut emitted = 0u64;
    let mut emitted_at_check = 0u64;
    let mut check_host = 0usize;
    let mut stall_reported = false;
    for (host, host_buttons) in buttons.iter().copied().enumerate() {
        if progress_every.is_some_and(|interval| host % interval == 0) {
            eprintln!("ledger host {host} iterations {emitted}");
        }
        if emitted == emitted_at_check {
            if host - check_host >= 3000 && !stall_reported {
                stall_reported = true;
                eprintln!(
                    "ledger warning: no iteration closed since host {check_host} (now {host}); opening={:?} pc={:06x}",
                    opening.as_ref().map(|iteration| iteration.index),
                    {
                        let state = &cpu.machine().snes().cpu;
                        (u32::from(state.k) << 16) | u32::from(state.pc)
                    }
                );
            }
        } else {
            emitted_at_check = emitted;
            check_host = host;
            stall_reported = false;
        }
        cpu.set_libretro_joypad_words(host_buttons, 0);
        let mut failure: Option<Box<dyn Error>> = None;
        let result = cpu.run_until_main_loop_return_with_quiescent_step_hook(
            |step, exec| -> Result<(), snes::SourceCpuError> {
                if failure.is_some() {
                    return Ok(());
                }
                if let Some(frame) = exec.take_captured_frame() {
                    let sha = (!frame.is_empty()).then(|| {
                        let mut digest = Sha256::new();
                        for pixel in frame.chunks_exact(4) {
                            // The port's line renderer stores BGRA.
                            digest.update([pixel[2], pixel[1], pixel[0]]);
                        }
                        format!("{:x}", digest.finalize())
                    });
                    if frame_hosts.is_some_and(|(lo, hi)| (lo..=hi).contains(&host)) {
                        let mut ppm = b"P6\n256 224\n255\n".to_vec();
                        for pixel in frame.chunks_exact(4) {
                            ppm.extend_from_slice(&[pixel[2], pixel[1], pixel[0]]);
                        }
                        if let Err(error) = fs::write(out_dir.join("frames").join(format!("host-{host}.ppm")), ppm) {
                            failure = Some(error.into());
                        }
                        let ppu = &exec.machine().snes().ppu;
                        let state = serde_json::json!({
                            "fixed_color": [ppu.fixed_color_r, ppu.fixed_color_g, ppu.fixed_color_b],
                            "cgram": ppu.cgram,
                        });
                        if let Err(error) = fs::write(
                            out_dir.join("frames").join(format!("host-{host}.json")),
                            state.to_string(),
                        ) {
                            failure = Some(error.into());
                        }
                    }
                    while let Some(mut finished) = awaiting_picture.pop_front() {
                        finished.picture_host = Some(host);
                        finished.picture_sha256 = sha.clone();
                        let line = serde_json::to_string(&finished).expect("serialize iteration");
                        if let Err(error) = writeln!(records, "{line}") {
                            failure = Some(error.into());
                        }
                        emitted += 1;
                    }
                    host_picture = sha;
                }
                if let Some(index) = awaiting_display {
                    if exec.raster_position().scanline() < 225 {
                        display_hosts.insert(index, host);
                        awaiting_display = None;
                    }
                }
                // An interrupt handler ends at its RTI; IRQs can nest inside
                // the NMI handler, so the opening handler has returned when
                // the depth falls back to zero. The polygon thread switches
                // stacks inside the NMI and need not return through it: an
                // opening handler still open at the next NMI closes there.
                let nmi_accepted = matches!(step.accepted_interrupt, Some(SourceCpuAcceptedInterrupt::Nmi { .. }));
                let stale_opening = nmi_accepted && opening.is_some();
                let handler_returned = stale_opening
                    || (step.opcode == RTI && step.accepted_interrupt.is_none() && {
                        interrupt_depth = interrupt_depth.saturating_sub(1);
                        interrupt_depth == 0
                    });
                if nmi_accepted {
                    interrupt_depth = 1;
                } else if step.accepted_interrupt.is_some() {
                    interrupt_depth += 1;
                }
                if handler_returned {
                    in_lag_handler = false;
                    if opening.is_some() {
                        if let Err(error) = post_nmi.push(&exec.machine().snes().ram[..WRAM_LEN]) {
                            failure = Some(error);
                        }
                        let opened = opening.as_ref().map(|iteration| iteration.index);
                        let was_timing_defined = in_timing_defined;
                        in_timing_defined = timing_defined(&exec.machine().snes().ram);
                        let resync_here = seed_every.is_some() && was_timing_defined && !in_timing_defined;
                        if let (true, Some(index), Some(file)) = (resync_here, opened, resync.as_mut()) {
                            if let Err(error) = writeln!(file, "{index}") {
                                failure = Some(error.into());
                            }
                        }
                        if let Some(index) = opened.filter(|index| {
                            resync_here || seed_every.is_some_and(|every| index % every == 0)
                        }) {
                            if let Err(error) = write_seed(
                                out_dir.join("seeds").join(format!("iter-{index}.bin")),
                                exec.machine().snes(),
                            ) {
                                failure = Some(error);
                            }
                        }
                        if let Some(mut finished) = current.take() {
                            finished.display_host = display_hosts.remove(&finished.index);
                            finished.nmi_return_line = exec.machine().timeline().raster_position().scanline();
                            awaiting_picture.push_back(finished);
                        }
                        current = opening.take();
                        if !stale_opening {
                            return Ok(());
                        }
                    }
                }
                match step.accepted_interrupt {
                    Some(SourceCpuAcceptedInterrupt::Nmi { interrupted_pc, .. })
                        if opens_iteration(interrupted_pc, exec.machine().snes().ram[NMI_BOOLEAN]) =>
                    {
                        if let Err(error) = pre_nmi.push(&exec.machine().snes().ram[..WRAM_LEN]) {
                            failure = Some(error);
                        }
                        if let Some(iteration) = current.as_mut() {
                            iteration.close_host = host;
                        }
                        awaiting_display = Some(next_index);
                        opening = Some(Iteration {
                            index: next_index,
                            open_host: host,
                            host_buttons,
                            ..Default::default()
                        });
                        next_index += 1;
                        return Ok(());
                    }
                    Some(SourceCpuAcceptedInterrupt::Nmi { .. }) => {
                        if let Some(iteration) = current.as_mut() {
                            iteration.lag_nmis += 1;
                        }
                        in_lag_handler = true;
                    }
                    Some(SourceCpuAcceptedInterrupt::Irq { .. }) => {
                        if let Some(iteration) = current.as_mut() {
                            iteration.irqs += 1;
                        }
                    }
                    None => {}
                }
                let in_open_handler = opening.is_some();
                let Some(iteration) = opening.as_mut().or(current.as_mut()) else {
                    return Ok(());
                };
                for access in &step.accesses {
                    if !watch.is_empty()
                        && let SourceCpuBusAccessKind::Write { value, width } = access.kind
                        && let Some(offset) = wram_offset(access.address)
                    {
                        for byte in 0..u32::from(width) {
                            if watch.contains(&(offset + byte)) {
                                eprintln!(
                                    "ledger-watch iteration={} host={host} pc={:06x} wram={:05x} value={:02x}",
                                    iteration.index, step.origin_pc, offset + byte, (value >> (8 * byte)) as u8,
                                );
                            }
                        }
                    }
                    if !system_bank(access.address) {
                        continue;
                    }
                    let register = access.address & 0xffff;
                    match access.kind {
                        SourceCpuBusAccessKind::Read { value, width } => {
                            if in_open_handler && register == 0x4218 {
                                let word = if width >= 2 {
                                    value
                                } else {
                                    (iteration.joypad.unwrap_or(0) & 0xff00) | (value & 0xff)
                                };
                                iteration.joypad = Some(word);
                            } else if in_open_handler && register == 0x4219 {
                                iteration.joypad = Some(
                                    (iteration.joypad.unwrap_or(0) & 0x00ff) | ((value & 0xff) << 8),
                                );
                            } else if (0x2140..=0x2143).contains(&register)
                                && !in_open_handler
                                && !in_lag_handler
                            {
                                for byte in 0..u32::from(width) {
                                    let port = register + byte;
                                    if port <= 0x2143 {
                                        iteration
                                            .apu_reads
                                            .push(((port - 0x2140) as u8, (value >> (8 * byte)) as u8));
                                    }
                                }
                            }
                        }
                        SourceCpuBusAccessKind::Write { value, width } => {
                            if (0x2140..=0x2143).contains(&register) {
                                for byte in 0..u32::from(width) {
                                    let port = register + byte;
                                    if port <= 0x2143 {
                                        iteration
                                            .apu_writes
                                            .push(((port - 0x2140) as u8, (value >> (8 * byte)) as u8));
                                    }
                                }
                            }
                            // The RNG routine's `STA $0FA1` starts at $0D:BA7C
                            // (Snes9x traces it after the fetch, as `*:BA7F`).
                            if step.origin_pc & 0x7f_ffff == 0x0d_ba7c && register == 0x0fa1 {
                                let carry = exec.machine().snes().cpu.pack_flags() & 1 != 0;
                                iteration.rng.push((value as u8, carry));
                            }
                        }
                        _ => {}
                    }
                }
                Ok(())
            },
        );
        if let Some(error) = failure {
            return Err(error);
        }
        if let Err(error) = result {
            let state = &cpu.machine().snes().cpu;
            return Err(format!(
                "ledger stopped at host {host} pc={:06x}: {error}",
                (u32::from(state.k) << 16) | u32::from(state.pc)
            )
            .into());
        }
        drop(cpu.take_dsp_samples());
        if let (Some(sha), Some(writer)) = (host_picture.take(), host_pictures.as_mut()) {
            writeln!(writer, "{host} {sha}")?;
        }
    }
    for finished in awaiting_picture {
        writeln!(records, "{}", serde_json::to_string(&finished)?)?;
        emitted += 1;
    }
    records.flush()?;
    if let Some(writer) = host_pictures.as_mut() {
        writer.flush()?;
    }
    pre_nmi.finish()?;
    post_nmi.finish()?;
    if let Some(file) = resync.as_mut() {
        file.flush()?;
    }
    println!("logical ledger wrote {emitted} complete iterations over {host_calls} host calls");
    Ok(())
}
