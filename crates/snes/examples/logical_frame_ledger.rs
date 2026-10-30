//! Record the ROM's logical-frame ledger for lag-free parity checks.
//!
//! Usage: cargo run -p snes --example logical_frame_ledger -- ROM SRAM INPUT HOST_CALLS OUT_DIR
//! Optional `--picture-events HOSTS_FILE` accepts one decimal host per line,
//! renders those hosts (hashes only), and records their display-affecting CPU
//! stores in `picture-events.jsonl.zst`. `at` is the source bus-access timestamp;
//! line/cycle are its physical beam position; retirement is recorded separately.
//! This does not alter execution.
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
//! `ZELDA3_LEDGER_FRAMES=<lo>-<hi>[,...]` draws just those windows and writes
//! `frames/host-<n>.ppm`, plus the PPU registers, OAM and VRAM beside it.
//! `ZELDA3_LEDGER_PROFILE=<lo>-<hi>[,...]` writes inclusive/exclusive main
//! instruction clocks to `routine-profile.csv`. `ZELDA3_LEDGER_PROFILE_ONLY=1`
//! suppresses the ordinary image/iteration streams; use a fresh output dir.
//! The same selection writes object-profile.csv, partitioning $01:893C and
//! its real JSR/JSL children by (subtype << 8 | jump-table index). JMP tails
//! keep their original call frame; interrupts and bus stalls remain excluded.
//! `sprite-profile.csv` partitions state bodies by entry type/state/slot,
//! matching the native profiler. Entry, timers and tail dispatch are excluded.
//! `ZELDA3_LEDGER_BLOCK_PROFILE=1` also writes per-PC instruction receipts
//! for the same complete iterations to instruction-profile.csv. These feed
//! scripts/rom_block_worklist.py; routine totals cannot identify executed blocks.
//! `ZELDA3_LEDGER_TIMING_AUDIT=1` adds `timing-audit.csv` for that selection,
//! separating main, opening/lag NMI, IRQ, polygon and idle instruction time,
//! DMA requests, refresh events, bus accesses and ROM/WRAM read bytes.
//! Interrupt-edge refresh attribution retains an explicit ambiguity count;
//! wall minus CPU/DMA/refresh is a residual, not a new timing price.

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

/// Receipts include both the completed opcode and any interrupt accepted
/// after it. Retire RTI before accounting for that new interrupt: an IRQ
/// accepted on the opening NMI's RTI must not keep that NMI open until the
/// following iteration. NMI resets depth for the polygon stack handoff.
fn handler_returned(opcode: u8, accepted: Option<SourceCpuAcceptedInterrupt>, depth: &mut u32) -> bool {
    let returned = if opcode == RTI {
        *depth = depth.saturating_sub(1);
        *depth == 0
    } else {
        false
    };
    match accepted {
        Some(SourceCpuAcceptedInterrupt::Nmi { .. }) => *depth = 1,
        Some(SourceCpuAcceptedInterrupt::Irq { .. }) => *depth += 1,
        None => {}
    }
    returned
}

#[cfg(test)]
mod tests {
    use super::*;

    fn irq() -> SourceCpuAcceptedInterrupt {
        SourceCpuAcceptedInterrupt::Irq {
            started_at: snes::CpuMasterTimestamp::new(100),
            interrupted_pc: 0x008034,
        }
    }

    #[test]
    fn rti_accepting_irq_closes_opening_before_it_can_go_stale() {
        let mut depth = 1;
        let mut opening = Some(7);
        let mut snapshots = Vec::new();
        // The NMI returns and an IRQ is accepted on the same receipt. Save
        // this image now, rather than leaving opening pending for the next NMI.
        if handler_returned(RTI, Some(irq()), &mut depth) {
            snapshots.push((opening.take().unwrap(), "NMI return image"));
        }
        assert_eq!(depth, 1);
        assert!(handler_returned(RTI, None, &mut depth));
        let next_nmi = SourceCpuAcceptedInterrupt::Nmi {
            started_at: snes::CpuMasterTimestamp::new(200),
            interrupted_pc: 0x008034,
        };
        assert!(!handler_returned(0xea, Some(next_nmi), &mut depth));
        assert!(opening.is_none(), "the stale-opening fallback must not capture the next iteration");
        assert_eq!(snapshots, [(7, "NMI return image")]);
        assert_eq!(depth, 1);
    }

    #[test]
    fn nested_irq_return_does_not_close_nmi() {
        let mut depth = 1;
        assert!(!handler_returned(0xea, Some(irq()), &mut depth));
        assert_eq!(depth, 2);
        assert!(!handler_returned(RTI, None, &mut depth));
        assert!(handler_returned(RTI, None, &mut depth));
        assert_eq!(depth, 0);
    }
}

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

/// Optional source-only timing diagnostics. Interrupt entry is split at its
/// receipt timestamp; idle wait-loop instructions never become main work.
/// Refresh at an interrupt edge is retained with the preceding instruction,
/// so component residuals at those edges have a 40-clock attribution bound.
#[derive(Default)]
struct TimingAudit {
    index: u64,
    owner: &'static str,
    stack: Vec<&'static str>,
    rows: std::collections::BTreeMap<(u64, &'static str), [u64; 15]>,
}

impl TimingAudit {
    fn observe(&mut self, step: &snes::SourceCpuStepReceipt, machine: &snes::Snes,
        next_index: u64, previous_sp: u16, refresh: u64, dma: u64,
        selection: &[(u64, u64)]) {
        if step.origin_pc == 0x008038 && self.stack.is_empty() { self.owner = "main"; }
        let owner = if self.stack.is_empty() && previous_sp >> 8 != 1 { "poly" }
            else if self.owner.is_empty() { "idle" } else { self.owner };
        let interrupt = match step.accepted_interrupt {
            Some(SourceCpuAcceptedInterrupt::Nmi { started_at, .. }) => Some((started_at, true)),
            Some(SourceCpuAcceptedInterrupt::Irq { started_at, .. }) => Some((started_at, false)),
            None => None,
        };
        let end = interrupt.map_or(step.ended_at, |(at, _)| at);
        let selected = |i| selection.iter().any(|&(lo, hi)| (lo..=hi).contains(&i));
        if selected(self.index) {
            let row = self.rows.entry((self.index, owner)).or_default();
            row[0] += end.master_cycles() - step.started_at.master_cycles();
            row[1] += step.transactions.iter().filter(|t| t.started_at < end)
                .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
            row[2] += refresh * 40;
            row[3] += dma;
            row[4] += 1;
            row[12] += u64::from(interrupt.is_some() && refresh != 0);
            row[13] += u64::from(machine.fast_mem);
            for access in step.accesses.iter().filter(|a| a.timestamp < end) {
                row[5] += 1;
                row[6] += u64::from(access.charged_master_cycles);
                let bytes = match access.kind {
                    SourceCpuBusAccessKind::Read { width, .. }
                    | SourceCpuBusAccessKind::Write { width, .. } => u64::from(width),
                    SourceCpuBusAccessKind::ReadLong { .. } => 3,
                    _ => 1,
                };
                if wram_offset(access.address).is_some() || access.address & 0x408000 != 0 {
                    let speed = if access.address & 0x800000 != 0
                        && access.address & 0x408000 != 0 && machine.fast_mem { 6 } else { 8 };
                    row[14] += u64::from(u64::from(access.charged_master_cycles) != bytes * speed);
                }
                if !matches!(access.kind, SourceCpuBusAccessKind::Write { .. }) {
                    let class = if wram_offset(access.address).is_some() { 8 }
                        else if access.address & 0x408000 != 0 { 7 } else { 9 };
                    row[class] += bytes;
                }
                row[10] += u64::from(access.charged_master_cycles == 6);
                row[11] += u64::from(access.charged_master_cycles == 8);
            }
        }
        if step.opcode == RTI { self.owner = self.stack.pop().unwrap_or("idle"); }
        if step.origin_pc == 0x00805d && self.stack.is_empty() { self.owner = "idle"; }
        if let Some((at, nmi)) = interrupt {
            let new_owner = if nmi && opens_iteration(0, machine.ram[NMI_BOOLEAN]) {
                self.index = next_index;
                self.stack.clear();
                self.owner = "idle";
                "entry"
            } else if nmi { "lag" } else { "irq" };
            self.stack.push(self.owner);
            self.owner = new_owner;
            if selected(self.index) {
                let row = self.rows.entry((self.index, new_owner)).or_default();
                row[0] += step.ended_at.master_cycles() - at.master_cycles();
                row[1] += step.transactions.iter().filter(|t| t.started_at >= at)
                    .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
            }
        }
    }

    fn write(&self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        let mut file = BufWriter::new(fs::File::create(path)?);
        writeln!(file, "iteration,owner,wall,cpu,refresh,dma,instructions,accesses,bus_master,rom_read_bytes,wram_read_bytes,other_read_bytes,six_clock_accesses,eight_clock_accesses,refresh_edges,memsel_instructions,rom_wram_speed_mismatches")?;
        for ((index, owner), row) in &self.rows {
            write!(file, "{index},{owner}")?;
            for value in row { write!(file, ",{value}")?; }
            writeln!(file)?;
        }
        file.flush()?;
        Ok(())
    }
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
    encoder: Option<zstd::Encoder<'static, BufWriter<fs::File>>>,
}

impl XorStream {
    fn create(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        let encoder = if env::var_os("ZELDA3_LEDGER_PROFILE_ONLY").is_some() {
            None
        } else {
            Some(zstd::Encoder::new(BufWriter::new(fs::File::create(path)?), 6)?)
        };
        Ok(Self {
            previous: vec![0; WRAM_LEN],
            scratch: vec![0; WRAM_LEN],
            encoder,
        })
    }

    fn push(&mut self, wram: &[u8]) -> Result<(), Box<dyn Error>> {
        let Some(encoder) = self.encoder.as_mut() else { return Ok(()); };
        for ((out, now), before) in self.scratch.iter_mut().zip(wram).zip(&self.previous) {
            *out = now ^ before;
        }
        self.previous.copy_from_slice(wram);
        encoder.write_all(&self.scratch)?;
        Ok(())
    }

    fn finish(self) -> Result<(), Box<dyn Error>> {
        if let Some(encoder) = self.encoder { encoder.finish()?.flush()?; }
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
    let first = args.next().ok_or("missing ROM path")?;
    // Static snapshot diagnostic, deliberately separate from the source's
    // line-by-line capture: no CPU or HDMA is advanced by this mode.
    if first == "--ppu-snapshot" {
        let mut ppu: snes::PpuState = serde_json::from_slice(&fs::read(args.next().ok_or("missing PPU JSON")?)?)?;
        let output = args.next().ok_or("missing output PPM")?;
        if args.next().is_some() { return Err("too many snapshot arguments".into()); }
        let margin = usize::from(ppu.extra_left_right);
        let width = 256 + margin * 2;
        ppu.begin_drawing(&mut vec![0; width * 224 * 4], width * 4, ppu.render_flags);
        for line in 0..=224 { ppu.run_line(line); }
        let mut rgb = Vec::with_capacity(256 * 224 * 3);
        for row in ppu.render_buffer.as_ref().ok_or("missing rendered snapshot")?.chunks_exact(width * 4) {
            for pixel in row[margin * 4..(margin + 256) * 4].chunks_exact(4) {
                rgb.extend_from_slice(&[pixel[2], pixel[1], pixel[0]]);
            }
        }
        println!("{:x}", Sha256::digest(&rgb));
        let mut ppm = b"P6\n256 224\n255\n".to_vec();
        ppm.extend(rgb);
        fs::write(output, ppm)?;
        return Ok(());
    }
    let rom = fs::read(first)?;
    let sram = fs::read(args.next().ok_or("missing SRAM path")?)?;
    let input = fs::read_to_string(args.next().ok_or("missing input path")?)?;
    let host_calls: usize = args.next().ok_or("missing host call count")?.parse()?;
    let out_dir = PathBuf::from(args.next().ok_or("missing output directory")?);
    let mut picture_hosts = std::collections::BTreeSet::new();
    if let Some(option) = args.next() {
        if option != "--picture-events" { return Err("unknown ledger option".into()); }
        for line in fs::read_to_string(args.next().ok_or("missing picture hosts file")?)?.lines() {
            picture_hosts.insert(line.parse::<usize>()?);
        }
    }
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
    let mut picture_events = if picture_hosts.is_empty() { None } else {
        Some(zstd::stream::write::Encoder::new(
            BufWriter::new(fs::File::create(out_dir.join("picture-events.jsonl.zst"))?), 3)?)
    };
    let mut records = BufWriter::new(fs::File::create(if env::var_os("ZELDA3_LEDGER_PROFILE_ONLY").is_some() {
        PathBuf::from("/dev/null")
    } else { out_dir.join("iterations.jsonl") })?);
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
    let frame_hosts: Vec<(usize, usize)> = env::var("ZELDA3_LEDGER_FRAMES")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(|range| {
                    let (lo, hi) = range.split_once('-').unwrap_or((range, range));
                    Ok::<_, Box<dyn Error>>((lo.parse::<usize>()?, hi.parse::<usize>()?))
                })
                .collect::<Result<_, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    // `ZELDA3_LEDGER_WATCH=<wram offset hex>[,...]` prints every CPU write to
    // those WRAM bytes with the iteration, host and writing PC.
    let watch: Vec<u32> = env::var("ZELDA3_LEDGER_WATCH")
        .map(|value| value.split(',').filter_map(|a| u32::from_str_radix(a.trim_start_matches("0x"), 16).ok()).collect())
        .unwrap_or_default();
    // `ZELDA3_LEDGER_PPU_WRITES=<lo>-<hi>` prints every CPU write to the PPU
    // registers, the DMA/HDMA enables and channel registers in those hosts, with the beam
    // position: the probe for a picture that differs on a few lines only.
    // `ZELDA3_LEDGER_THREAD_TRACE=<lo>-<hi>` prints every accepted NMI and IRQ
    // in those hosts with the beam line and the interrupted PC: which of the
    // main loop and the polygon thread each interrupt preempted.
    // CPU-only instruction coverage. Inclusive rows retain every live JSR/JSL
    // caller; exclusive rows belong to the innermost call target. Tail jumps
    // keep the caller's identity (they do not push a return address).
    let mut profile_iterations: Vec<(u64, u64)> = env::var("ZELDA3_LEDGER_PROFILE")
        .ok().map(|value| value.split(',').map(|range| {
            let (lo, hi) = range.split_once('-').unwrap_or((range, range));
            let (lo, hi) = (lo.parse::<u64>()?, hi.parse::<u64>()?);
            if lo > hi { return Err("reversed profile window".into()); }
            Ok::<_, Box<dyn Error>>((lo, hi))
        }).collect::<Result<_, _>>()).transpose()?.unwrap_or_default();
    profile_iterations.sort_unstable();
    let mut merged: Vec<(u64, u64)> = Vec::new();
    for (lo, hi) in profile_iterations {
        if let Some(last) = merged.last_mut().filter(|last| lo <= last.1.saturating_add(1)) {
            last.1 = last.1.max(hi);
        } else {
            merged.push((lo, hi));
        }
    }
    let profile_iterations = merged;
    let mut timing_audit = env::var_os("ZELDA3_LEDGER_TIMING_AUDIT").map(|_| TimingAudit::default());
    let mut profile_output = (!profile_iterations.is_empty()).then(|| {
        let mut writer = BufWriter::new(fs::File::create(out_dir.join("routine-profile.csv"))?);
        writeln!(writer, "iteration,routine,calls,inclusive,exclusive")?;
        Ok::<_, std::io::Error>(writer)
    }).transpose()?;
    // Object JMP tails retain the $01:893C call frame.
    let mut object_identity = None;
    let mut object_work = std::collections::BTreeMap::<u32, [u64; 3]>::new();
    let mut object_profile = std::collections::BTreeMap::<(u64, u16, u32), [u64; 3]>::new();
    let mut profile_stack: Vec<(u32, u16)> = Vec::new();
    // Each real $06:84E2 call owns its selection, including nested calls.
    // Capture state before timers; capture type when the selected tail begins.
    let mut sprite_frames = std::collections::BTreeMap::<usize, (u8, u8, u8, bool)>::new();
    let mut sprite_profile = std::collections::BTreeMap::<(u64, u8, u8, u8, u32), [u64; 3]>::new();
    let sprite_tails: Vec<u32> = (0..12).map(|state| {
        let offset = 6 * 0x8000 + 0x4f4 + state * 2;
        0x06_0000 | u32::from(u16::from_le_bytes([rom[offset], rom[offset + 1]]))
    }).collect();
    let mut profile_depth = 0usize;
    let mut profile_main_active = false;
    let mut profile: std::collections::BTreeMap<(u64, u32), [u64; 3]> = Default::default();
    let block_profile_enabled = env::var_os("ZELDA3_LEDGER_BLOCK_PROFILE").is_some();
    let mut instruction_profile: std::collections::BTreeMap<(u64, u32, u32, u8), [u64; 3]> = Default::default();
    // Follow main instructions only; interrupts must not invent block entries
    // in the interrupted stream. Include actual computed-jump destinations.
    let mut profile_block_entry = true;
    let mut profile_complete = std::collections::BTreeSet::new();
    let mut previous_sp = 0u16;
    let mut previous_widths = 0x30u8;
    // Selected main steps only. Wall minus CPU transactions includes the
    // stalls deliberately excluded from routine-profile.csv.
    let mut structural: std::collections::BTreeMap<u64, [u64; 6]> = Default::default();
    let mut dma_sizes = [0u16; 8];
    // `ZELDA3_LEDGER_NMI_PROFILE=<lo>-<hi>` (hosts) does the same for the
    // NMI handler, from its acceptance to its RTI, with each frame's wall
    // time beside its bus cycles (DMA runs inside the storing instruction):
    // `ledger-nmi-profile host=<h> routine=<pc> master=<cycles> wall=<cycles>`.
    let nmi_profile_hosts = env::var("ZELDA3_LEDGER_NMI_PROFILE").ok().and_then(|value| {
        let (lo, hi) = value.split_once('-')?;
        Some((lo.parse::<u64>().ok()?, hi.parse::<u64>().ok()?))
    });
    let mut nmi_profile_stack: Vec<(u32, u16)> = Vec::new();
    let mut nmi_profile_depth = 0usize;
    let mut nmi_profile: std::collections::BTreeMap<(u64, u32), (u64, u64)> = Default::default();
    // `ZELDA3_LEDGER_STEP_TRACE=<lo>-<hi>` (hosts): every instruction's
    // origin PC and start/end master cycle (`ledger-step`).
    let step_trace_hosts = env::var("ZELDA3_LEDGER_STEP_TRACE").ok().and_then(|value| {
        let (lo, hi) = value.split_once('-')?;
        Some((lo.parse::<u64>().ok()?, hi.parse::<u64>().ok()?))
    });
    let thread_trace_hosts = env::var("ZELDA3_LEDGER_THREAD_TRACE")
        .ok()
        .and_then(|value| {
            let (lo, hi) = value.split_once('-')?;
            Some((lo.parse::<usize>().ok()?, hi.parse::<usize>().ok()?))
        });
    let ppu_write_hosts = env::var("ZELDA3_LEDGER_PPU_WRITES")
        .ok()
        .map(|value| {
            let (lo, hi) = value.split_once('-').unwrap_or((&value, &value));
            Ok::<_, Box<dyn Error>>((lo.parse::<usize>()?, hi.parse::<usize>()?))
        })
        .transpose()?;
    let mut cpu = Snes9xColdCpuExecutor::from_lorom_reset_with_sram(&rom, Some(&sram))?;
    let render_every = env::var_os("ZELDA3_LEDGER_PICTURES").is_some();
    let render_pictures = !frame_hosts.is_empty() || !picture_hosts.is_empty() || render_every;
    // `ZELDA3_LEDGER_FRAMES` alone draws only from a few hosts before each
    // window to its end: the line renderer needs no earlier pictures, and
    // drawing every frame from reset makes a deep window take hours.
    let rendering_at = |host: usize| {
        render_every || frame_hosts.iter().any(|&(lo, hi)| (lo.saturating_sub(3)..=hi + 1).contains(&host))
            || picture_hosts.range(host.saturating_sub(1)..=host.saturating_add(3)).next().is_some()
    };
    let mut rendering = rendering_at(0);
    cpu.enable_frame_capture(rendering);
    if !frame_hosts.is_empty() {
        fs::create_dir_all(out_dir.join("frames"))?;
    }
    let mut host_pictures = render_pictures
        .then(|| fs::File::create(out_dir.join("host-pictures.txt")).map(BufWriter::new))
        .transpose()?;
    // Finished iterations wait here for the frame that shows their work.
    let mut awaiting_picture: std::collections::VecDeque<Iteration> = Default::default();
    let mut host_picture: Option<String> = None;
    drop(cpu.take_dsp_samples());
    let mut bus_observer = cpu.machine().timeline().synchronous_event_observer();
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
        if rendering != rendering_at(host) {
            rendering = !rendering;
            cpu.enable_frame_capture(rendering);
        }
        cpu.set_libretro_joypad_words(host_buttons, 0);
        let observe_picture_host = picture_hosts.contains(&host);
        let mut picture_wram = observe_picture_host.then(|| cpu.machine().snes().ram.to_vec());
        let mut picture_inidisp = u16::from(cpu.machine().snes().ppu.brightness)
            | if cpu.machine().snes().ppu.forced_blank { 0x80 } else { 0 };
        if observe_picture_host {
            let setup = serde_json::json!({
                "kind": "host_setup", "host": host,
                "at": cpu.machine().timeline().timestamp().master_cycles(),
                "field": cpu.machine().timeline().field_index(),
                "dma": cpu.machine().snes().dma,
                "thread_active": cpu.machine().snes().ram[NMI_THREAD_ACTIVE],
                "module": &cpu.machine().snes().ram[MAIN_MODULE..MAIN_MODULE + 2],
            });
            let output = picture_events.as_mut().expect("picture event stream");
            serde_json::to_writer(&mut *output, &setup)?;
            writeln!(output)?;
        }
        let mut failure: Option<Box<dyn Error>> = None;
        let result = cpu.run_until_main_loop_return_with_quiescent_step_hook(
            |step, exec| -> Result<(), snes::SourceCpuError> {
                if failure.is_some() {
                    return Ok(());
                }
                // Bus stores precede the interrupt accepted at retirement.
                // Observe before handler_returned and the opening-NMI early return.
                if observe_picture_host && let Some(iteration) = opening.as_ref().or(current.as_ref()) {
                    let in_open_handler = opening.is_some();
                    for access in &step.accesses {
                        if let SourceCpuBusAccessKind::Write { value, width } = access.kind
                        {
                            let register = access.address & 0xffff;
                            let display_register = system_bank(access.address)
                                && ((0x2100..=0x2133).contains(&register)
                                    || matches!(register, 0x4200 | 0x420b | 0x420c)
                                    || (0x4300..=0x437f).contains(&register));
                            let table_or_scroll = wram_offset(access.address).is_some_and(|offset|
                                (0x1b00..0x1d00).contains(&offset)
                                    || (0xe0..0xea).contains(&offset)
                                    || (0x630..0x634).contains(&offset));
                            if display_register || table_or_scroll {
                                let previous = if let Some(offset) = wram_offset(access.address) {
                                    let ram = picture_wram.as_mut().expect("picture WRAM observer");
                                    let offset = offset as usize;
                                    let mut previous = 0u16;
                                    for byte in 0..usize::from(width) {
                                        previous |= u16::from(ram[offset + byte]) << (byte * 8);
                                        ram[offset + byte] = (value >> (byte * 8)) as u8;
                                    }
                                    Some(previous)
                                } else if register == 0x2100 {
                                    Some(std::mem::replace(&mut picture_inidisp, value))
                                } else { None };
                                let raster = exec.machine().timeline().raster_position();
                                let access_clock = snes::CpuMasterTimeline::new(
                                    access.timestamp.master_cycles(), snes::CpuBusWorkload::default(),
                                    exec.machine().timeline().field_timing());
                                let at_raster = access_clock.raster_position();
                                let row = serde_json::json!({
                                    "kind": "store",
                                    "host": host, "iteration": iteration.index,
                                    "opening_handler": in_open_handler, "lag_handler": in_lag_handler,
                                    "interrupt_depth": interrupt_depth,
                                    "accepted_after_store": match step.accepted_interrupt {
                                        Some(SourceCpuAcceptedInterrupt::Nmi { .. }) => Some("nmi"),
                                        Some(SourceCpuAcceptedInterrupt::Irq { .. }) => Some("irq"),
                                        None => None,
                                    },
                                    "pc": step.origin_pc, "address": access.address,
                                    "value": value, "width": width,
                                    "previous": previous,
                                    "at": access.timestamp.master_cycles(),
                                    "field": access_clock.field_index(),
                                    "line": at_raster.scanline(), "cycle": at_raster.coordinates().1,
                                    "retired_line": raster.scanline(), "retired_cycle": raster.coordinates().1,
                                    "retired_at": exec.machine().timeline().timestamp().master_cycles(),
                                });
                                let output = picture_events.as_mut().expect("picture event stream");
                                if let Err(error) = serde_json::to_writer(&mut *output, &row)
                                    .map_err(std::io::Error::other).and_then(|()| writeln!(output))
                                { failure = Some(error.into()); }
                            }
                        }
                    }
                }
                let mut refresh_events = 0u64;
                exec.machine().timeline().observe_processed_synchronous_events(&mut bus_observer, |event| {
                    if event == snes::CpuSynchronousTimelineEvent::Bus(snes::CpuBusEvent::WramRefresh) {
                        refresh_events += 1;
                    }
                });
                let mut dma_master = 0;
                for access in &step.accesses {
                    if system_bank(access.address)
                        && access.address & 0xffff == 0x420b
                        && let SourceCpuBusAccessKind::Write { value, .. } = access.kind
                        && value != 0
                    {
                        dma_master += 18;
                        for (channel, size) in dma_sizes.iter().enumerate() {
                            if value & (1 << channel) != 0 {
                                dma_master += 8 + 8 * if *size == 0 { 65536 } else { u64::from(*size) };
                            }
                        }
                    }
                }
                if step.accesses.iter().any(|access| system_bank(access.address)
                    && matches!(access.kind, SourceCpuBusAccessKind::Write { .. })
                    && matches!(access.address & 0xffff, 0x420b | 0x4300..=0x437f)) {
                    dma_sizes = std::array::from_fn(|i| exec.machine().snes().dma.channel[i].size);
                }
                if let Some(audit) = timing_audit.as_mut() {
                    audit.observe(step, exec.machine().snes(), next_index, previous_sp,
                        refresh_events, dma_master, &profile_iterations);
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
                    if frame_hosts.iter().any(|&(lo, hi)| (lo..=hi).contains(&host)) {
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
                            "bg": ppu.bg_layer,
                            "mode": ppu.mode,
                            "oam": ppu.oam,
                        });
                        let vram: Vec<u8> = ppu.vram.iter().flat_map(|word| word.to_le_bytes()).collect();
                        if let Err(error) = fs::write(out_dir.join("frames").join(format!("host-{host}.vram")), vram) {
                            failure = Some(error.into());
                        }
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
                if !profile_iterations.is_empty() {
                    let cpu = &exec.machine().snes().cpu;
                    // Receipts are post-instruction AND post-interrupt entry.
                    // Use the preceding SP for ownership and undo interrupt
                    // entry when identifying the just-executed call target.
                    let main_stack = previous_sp >> 8 == 0x01;
                    let iteration = current.as_ref().map(|iteration| iteration.index);
                    let selected = iteration.filter(|index| {
                        let end = profile_iterations.partition_point(|&(lo, _)| lo <= *index);
                        end != 0 && *index <= profile_iterations[end - 1].1
                    });
                    if step.origin_pc == 0x00_8038 {
                        profile_block_entry = true;
                        profile_main_active = true;
                        profile_depth = 0;
                        profile_stack.clear();
                        sprite_frames.clear();
                        if let Some(index) = selected { profile.entry((index, 0x008034)).or_default()[0] += 1; }
                    }
                    if profile_depth == 0 && main_stack && profile_main_active {
                        if let Some(index) = selected {
                            let end = match step.accepted_interrupt {
                                Some(SourceCpuAcceptedInterrupt::Nmi { started_at, .. })
                                | Some(SourceCpuAcceptedInterrupt::Irq { started_at, .. }) => started_at,
                                None => step.ended_at,
                            };
                            // Bus/internal CPU transactions only: interrupt
                            // entry, DMA/HDMA and refresh are not instruction prices.
                            let cycles: u64 = step.transactions.iter()
                                .filter(|transaction| transaction.started_at < end)
                                .map(|transaction| u64::from(transaction.duration_master_cycles)).sum();
                            if block_profile_enabled {
                                let routine = profile_stack.last().map_or(0x008034, |frame| frame.0);
                                let row = instruction_profile.entry((index, routine, step.origin_pc & 0x7fffff, previous_widths)).or_default();
                                row[0] += 1;
                                row[1] += cycles;
                                row[2] += u64::from(profile_block_entry);
                            }
                            if let Some((&depth, selection)) = sprite_frames.last_key_value() {
                                let (mut kind, state, slot, mut active) = *selection;
                                if !active && sprite_tails.get(usize::from(state)).copied()
                                    == Some(step.origin_pc & 0x7fffff) {
                                    kind = exec.machine().snes().ram[0xe20 + usize::from(slot)];
                                    active = true;
                                    sprite_frames.insert(depth, (kind, state, slot, active));
                                    sprite_profile.entry((index, kind, state, slot, 0x06_84e2)).or_default()[0] += 1;
                                }
                                if active {
                                    let routine = profile_stack.last().unwrap().0;
                                    sprite_profile.entry((index, kind, state, slot, routine)).or_default()[2] += cycles;
                                    for (routine, _) in &profile_stack[depth..] {
                                        sprite_profile.entry((index, kind, state, slot, *routine)).or_default()[1] += cycles;
                                    }
                                }
                            }
                            let row = structural.entry(index).or_default();
                            row[0] += end.master_cycles() - step.started_at.master_cycles();
                            row[1] += cycles;
                            row[2] += refresh_events * 40;
                            row[3] += dma_master;
                            row[4] += u64::from(step.accesses.iter().any(|access|
                                system_bank(access.address) && access.address & 0xffff == 0x213d));
                            // An interrupt-entry refresh may be observed by
                            // this step although its clocks follow `end`.
                            row[5] += u64::from(step.accepted_interrupt.is_some() && refresh_events != 0);
                            if let Some(depth) = profile_stack.iter().position(|frame| frame.0 == 0x01893c) {
                                let routine = profile_stack.last().unwrap().0;
                                object_work.entry(routine).or_default()[2] += cycles;
                                for (routine, _) in &profile_stack[depth..] {
                                    object_work.entry(*routine).or_default()[1] += cycles;
                                }
                                // ASL produces the actual jump-table index.
                                match step.origin_pc & 0x7fffff {
                                    0x01896c => object_identity = Some(0x100 | (cpu.a >> 1)),
                                    0x0189a6 => object_identity = Some(0x200 | (cpu.a >> 1)),
                                    0x0189ca => object_identity = Some(0x300 | (cpu.a >> 1)),
                                    _ => {},
                                }
                            }
                            let routine = profile_stack.last().map_or(0x008034, |frame| frame.0);
                            profile.entry((index, routine)).or_default()[2] += cycles;
                            profile.entry((index, 0x008034)).or_default()[1] += cycles;
                            for (routine, _) in &profile_stack {
                                profile.entry((index, *routine)).or_default()[1] += cycles;
                            }
                        }
                        let (next_pc, next_sp) = match step.accepted_interrupt {
                            Some(SourceCpuAcceptedInterrupt::Nmi { interrupted_pc, .. })
                            | Some(SourceCpuAcceptedInterrupt::Irq { interrupted_pc, .. }) =>
                                (interrupted_pc, cpu.sp.wrapping_add(4)), // native-mode PBR/PC/P
                            None => ((u32::from(cpu.k) << 16) | u32::from(cpu.pc), cpu.sp),
                        };
                        if matches!(step.opcode, 0x20 | 0xfc | 0x22) {
                            let target = next_pc & 0x7fffff;
                            if selected.is_some() {
                                if target == 0x01893c {
                                    assert!(object_work.is_empty());
                                    object_identity = None;
                                    object_work.entry(target).or_default()[0] += 1;
                                } else if profile_stack.iter().any(|frame| frame.0 == 0x01893c) {
                                    object_work.entry(target).or_default()[0] += 1;
                                }
                            }
                            profile_stack.push((target, next_sp));
                            if selected.is_some() {
                                if target == 0x06_84e2 {
                                    let slot = cpu.x as u8;
                                    let state = exec.machine().snes().ram[0xdd0 + usize::from(slot)];
                                    sprite_frames.insert(profile_stack.len() - 1, (0, state, slot, false));
                                } else if let Some((&_, &(kind, state, slot, true))) = sprite_frames.last_key_value() {
                                    if let Some(index) = selected {
                                        sprite_profile.entry((index, kind, state, slot, target)).or_default()[0] += 1;
                                    }
                                }
                            }
                            if let Some(index) = selected { profile.entry((index, target)).or_default()[0] += 1; }
                        }
                        while profile_stack.last().is_some_and(|frame| next_sp > frame.1) {
                            sprite_frames.remove(&(profile_stack.len() - 1));
                            let popped = profile_stack.pop().unwrap();
                            if popped.0 == 0x01893c && let Some(index) = selected {
                                let object = object_identity.take().expect("object dispatch identity");
                                for (routine, values) in std::mem::take(&mut object_work) {
                                    let row = object_profile.entry((index, object, routine)).or_default();
                                    for i in 0..3 { row[i] += values[i]; }
                                }
                            }
                        }
                    }
                    if profile_depth == 0 && main_stack && profile_main_active {
                        profile_block_entry = matches!(step.opcode,
                            0x00 | 0x02 | 0x10 | 0x20 | 0x22 | 0x30 | 0x40 | 0x44 |
                            0x4c | 0x50 | 0x54 | 0x5c | 0x60 | 0x6b | 0x6c | 0x70 |
                            0x7c | 0x80 | 0x82 | 0x90 | 0xb0 | 0xd0 | 0xdc | 0xf0 | 0xfc);
                    }
                    if step.origin_pc == 0x00_805d {
                        if let Some(index) = selected { profile_complete.insert(index); }
                        profile_main_active = false;
                    }
                    if step.opcode == RTI { profile_depth = profile_depth.saturating_sub(1); }
                    if step.accepted_interrupt.is_some() { profile_depth += 1; }
                    previous_sp = cpu.sp;
                    previous_widths = cpu.pack_flags() & 0x30;
                }
                if step_trace_hosts.is_some_and(|(lo, hi)| (lo..=hi).contains(&(host as u64))) {
                    let raster = exec.raster_position();
                    eprintln!(
                        "ledger-step host={host} pc={:06x} start={} end={} line={} irq={}",
                        step.origin_pc,
                        step.started_at.master_cycles(),
                        step.ended_at.master_cycles(),
                        raster.scanline(),
                        step.accepted_interrupt.is_some(),
                    );
                }
                if nmi_profile_hosts.is_some_and(|(lo, hi)| (lo..=hi).contains(&(host as u64))) {
                    let cpu = &exec.machine().snes().cpu;
                    if nmi_profile_depth > 0 {
                        let end = match step.accepted_interrupt {
                            Some(SourceCpuAcceptedInterrupt::Nmi { started_at, .. })
                            | Some(SourceCpuAcceptedInterrupt::Irq { started_at, .. }) => started_at,
                            None => step.ended_at,
                        };
                        let cycles: u64 = step
                            .transactions
                            .iter()
                            .filter(|transaction| transaction.started_at < end)
                            .map(|transaction| u64::from(transaction.duration_master_cycles))
                            .sum();
                        let wall = end.master_cycles().saturating_sub(step.started_at.master_cycles());
                        let routine = nmi_profile_stack.last().map_or(0x00_80c9, |frame| frame.0);
                        let entry = nmi_profile.entry((host as u64, routine)).or_default();
                        entry.0 += cycles;
                        entry.1 += wall;
                    }
                    if matches!(step.accepted_interrupt, Some(SourceCpuAcceptedInterrupt::Nmi { .. })) {
                        nmi_profile_depth = 1;
                        nmi_profile_stack.clear();
                    } else if nmi_profile_depth > 0 {
                        if step.accepted_interrupt.is_some() {
                            nmi_profile_depth += 1;
                        } else if step.opcode == RTI {
                            nmi_profile_depth -= 1;
                        } else if nmi_profile_depth == 1 {
                            if matches!(step.opcode, 0x20 | 0xfc | 0x22) {
                                nmi_profile_stack.push(((u32::from(cpu.k) << 16) | u32::from(cpu.pc), cpu.sp));
                            }
                            while nmi_profile_stack.last().is_some_and(|frame| cpu.sp > frame.1) {
                                nmi_profile_stack.pop();
                            }
                        }
                    }
                }
                if thread_trace_hosts.is_some_and(|(lo, hi)| (lo..=hi).contains(&host)) {
                    let accepted = match step.accepted_interrupt {
                        Some(SourceCpuAcceptedInterrupt::Nmi { interrupted_pc, .. }) => Some(("nmi", interrupted_pc)),
                        Some(SourceCpuAcceptedInterrupt::Irq { interrupted_pc, .. }) => Some(("irq", interrupted_pc)),
                        None => None,
                    };
                    // Main's iteration: released at $00:8038 (CLI), done at
                    // $00:805D (STZ $12) before it spins in the wait loop.
                    if matches!(step.origin_pc, 0x00_8038 | 0x00_805d) && step.accepted_interrupt.is_none() {
                        let raster = exec.machine().timeline().raster_position();
                        eprintln!(
                            "ledger-thread host={host} {} line={} dot={} abs={}",
                            if step.origin_pc == 0x00_8038 { "main_start" } else { "main_done" },
                            raster.scanline(),
                            raster.master_cycle(),
                            exec.machine().timeline().timestamp().master_cycles(),
                        );
                    }
                    if step.opcode == RTI && step.accepted_interrupt.is_none() {
                        let raster = exec.machine().timeline().raster_position();
                        eprintln!(
                            "ledger-thread host={host} rti line={} dot={} abs={}",
                            raster.scanline(),
                            raster.master_cycle(),
                            exec.machine().timeline().timestamp().master_cycles(),
                        );
                    }
                    if let Some((kind, pc)) = accepted {
                        let raster = exec.machine().timeline().raster_position();
                        eprintln!(
                            "ledger-thread host={host} {kind} line={} dot={} abs={} pc={pc:06x} main_done={} go={:02x} upload={:02x}",
                            raster.scanline(),
                            raster.master_cycle(),
                            exec.machine().timeline().timestamp().master_cycles(),
                            exec.machine().snes().ram[NMI_BOOLEAN],
                            exec.machine().snes().ram[0x1f00],
                            exec.machine().snes().ram[0x1f0c],
                        );
                    }
                }
                let nmi_accepted = matches!(step.accepted_interrupt, Some(SourceCpuAcceptedInterrupt::Nmi { .. }));
                let stale_opening = nmi_accepted && opening.is_some();
                let returned = handler_returned(step.opcode, step.accepted_interrupt, &mut interrupt_depth);
                let handler_returned = stale_opening || returned;
                if handler_returned {
                    in_lag_handler = false;
                    if opening.is_some() {
                        if stale_opening || (returned && step.accepted_interrupt.is_some()) {
                            eprintln!("ledger-handler-boundary opening={} host={host} pc={:06x} stale={stale_opening} returned={returned} interrupt={:?}",
                                opening.as_ref().unwrap().index, step.origin_pc, step.accepted_interrupt);
                        }
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
                        if step.accepted_interrupt.is_none() {
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
                                    "ledger-watch iteration={} host={host} line={} abs={} pc={:06x} wram={:05x} value={:02x}",
                                    iteration.index,
                                    exec.machine().timeline().raster_position().scanline(),
                                    exec.machine().timeline().timestamp().master_cycles(),
                                    step.origin_pc,
                                    offset + byte,
                                    (value >> (8 * byte)) as u8,
                                );
                            }
                        }
                    }
                    if !system_bank(access.address) {
                        continue;
                    }
                    let register = access.address & 0xffff;
                    if ppu_write_hosts.is_some_and(|(lo, hi)| (lo..=hi).contains(&host))
                        && let SourceCpuBusAccessKind::Write { value, width } = access.kind
                        && ((0x2100..=0x2133).contains(&register)
                            || matches!(register, 0x420b | 0x420c)
                            || (0x4300..=0x437f).contains(&register))
                    {
                        let raster = exec.machine().timeline().raster_position();
                        eprintln!(
                            "ledger-ppu iteration={} host={host} v={} pc={:06x} reg={register:04x} value={value:0w$x}",
                            iteration.index, raster.scanline(), step.origin_pc, w = usize::from(width) * 2,
                        );
                    }
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
    for ((host, routine), (master, wall)) in &nmi_profile {
        eprintln!("ledger-nmi-profile host={host} routine={routine:06x} master={master} wall={wall}");
    }
    if let Some(audit) = timing_audit { audit.write(out_dir.join("timing-audit.csv"))?; }
    if let Some(writer) = profile_output.as_mut() {
        if block_profile_enabled {
            let mut blocks = BufWriter::new(fs::File::create(out_dir.join("instruction-profile.csv"))?);
            writeln!(blocks, "iteration,routine,pc,m8,x8,count,master,entries")?;
            for ((index, routine, pc, widths), [count, master, entries]) in instruction_profile {
                if profile_complete.contains(&index) {
                    writeln!(blocks, "{index},{routine:06x},{pc:06x},{},{},{count},{master},{entries}",
                        u8::from(widths & 0x20 != 0), u8::from(widths & 0x10 != 0))?;
                }
            }
        }
        let mut sprites = BufWriter::new(fs::File::create(out_dir.join("sprite-profile.csv"))?);
        writeln!(sprites, "iteration,type,state,slot,routine,calls,inclusive,exclusive")?;
        for ((index, kind, state, slot, routine), [calls, inclusive, exclusive]) in sprite_profile {
            if profile_complete.contains(&index) {
                writeln!(sprites, "{index},{kind:02x},{state:02x},{slot},{routine:06x},{calls},{inclusive},{exclusive}")?;
            }
        }
        let mut objects = BufWriter::new(fs::File::create(out_dir.join("object-profile.csv"))?);
        writeln!(objects, "iteration,object,routine,calls,inclusive,exclusive")?;
        for ((index, object, routine), [calls, inclusive, exclusive]) in object_profile {
            if profile_complete.contains(&index) {
                writeln!(objects, "{index},{object:03x},{routine:06x},{calls},{inclusive},{exclusive}")?;
            }
        }
        let mut wall = BufWriter::new(fs::File::create(out_dir.join("structural-profile.csv"))?);
        writeln!(wall, "iteration,main_wall,instructions,refresh_master,dma_master,vcounter_reads,interrupt_refresh_edges")?;
        for (index, row) in &structural {
            if profile_complete.contains(index) {
                writeln!(wall, "{index},{},{},{},{},{},{}", row[0], row[1], row[2], row[3], row[4], row[5])?;
            }
        }
        for ((iteration, routine), [calls, inclusive, exclusive]) in &profile {
            if !profile_complete.contains(iteration) { continue; }
            writeln!(writer, "{iteration},{routine:06x},{calls},{inclusive},{exclusive}")?;
        }
        let totals: std::collections::BTreeMap<u64, u64> = profile.iter()
            .filter(|((index, address), _)| *address == 0x008034 && profile_complete.contains(index))
            .map(|((index, _), row)| (*index, row[1])).collect();
        for (index, total) in totals { writeln!(writer, "{index},total,1,{total},{total}")?; }
        writer.flush()?;
    }
    if let Some(events) = picture_events { events.finish()?.flush()?; }
    println!("logical ledger wrote {emitted} complete iterations over {host_calls} host calls");
    Ok(())
}

#[cfg(test)]
mod timing_audit_tests {
    use super::*;
    use snes::{CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline, CpuRasterPosition,
        RomCpuTimingProbe, SourcePpuReadState, CartType};

    #[test]
    fn audit_preserves_probe_bus_bytes_and_slowrom_wram_prices() {
        // LDA long $7e0000, then STA absolute $2100: ROM, WRAM and I/O.
        let mut machine = snes::Snes::new();
        let mut rom = vec![0xea; 0x8000];
        rom[..7].copy_from_slice(&[0xaf, 0, 0, 0x7e, 0x8d, 0, 0x21]);
        machine.cart.load(CartType::LoRom, &rom, 0x2000);
        machine.cpu.pc = 0x8000;
        machine.cpu.sp = 0x1ff;
        machine.cpu.e = false;
        machine.cpu.mf = false;
        let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(20, 100),
            CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
        let mut probe = RomCpuTimingProbe::new(machine, clock, SourcePpuReadState::snes9x_reset()).unwrap();
        let mut audit = TimingAudit { owner: "main", ..Default::default() };
        for _ in 0..2 {
            let step = probe.step().unwrap();
            audit.observe(&step, &snes::Snes::new(), 1, 0x1ff, 0, 0, &[(0, 0)]);
        }
        let row = audit.rows[&(0, "main")];
        assert_eq!(row[4], 2);
        assert_eq!(row[7], 7);
        assert_eq!(row[8], 2);
        assert_eq!(row[14], 0);
        assert_eq!(row[0], row[1]);
    }

    #[test]
    fn audit_splits_real_probe_interrupt_entry_from_main() {
        for latched in [false, true] {
            let mut machine = snes::Snes::new();
            machine.cart.load(CartType::LoRom, &vec![0xea; 0x8000], 0x2000);
            machine.cpu.pc = 0x8000;
            machine.cpu.sp = 0x1ff;
            machine.cpu.e = false;
            machine.ram[NMI_BOOLEAN] = u8::from(latched);
            let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(20, 100),
                CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
            let mut probe = RomCpuTimingProbe::new(machine, clock, SourcePpuReadState::snes9x_reset()).unwrap();
            let mut step = probe.step().unwrap();
            let entry = probe.accept_native_nmi().unwrap();
            let prototype = step.transactions[0];
            step.transactions.extend(entry.transactions.iter().map(|t| snes::SourceCpuTransaction {
                kind: t.kind, duration_master_cycles: t.duration_master_cycles,
                started_at: t.started_at, ended_at: t.ended_at,
                start_wram_refresh_position: t.start_wram_refresh_position,
                end_wram_refresh_position: t.end_wram_refresh_position,
                ..prototype
            }));
            step.accesses.extend(entry.accesses);
            step.ended_at = entry.ended_at;
            step.accepted_interrupt = Some(SourceCpuAcceptedInterrupt::Nmi {
                started_at: entry.started_at, interrupted_pc: entry.interrupted_pc,
            });
            let mut audit = TimingAudit { owner: "main", ..Default::default() };
            audit.observe(&step, probe.snes(), 1, 0x1ff, 0, 0, &[(0, 1)]);
            assert_eq!(audit.rows[&(0, "main")][1], 14);
            let key = if latched { (0, "lag") } else { (1, "entry") };
            assert_eq!(audit.rows[&key][1], 62);
            assert_eq!(audit.rows.values().map(|r| r[0]).sum::<u64>(), 76);
        }
    }

    #[test]
    fn audit_distinguishes_fastrom_fetches_from_eight_clock_wram_data() {
        for (bank, fast, expected) in [(0, true, 48), (0x80, false, 48), (0x80, true, 40)] {
            let mut machine = snes::Snes::new();
            let mut rom = vec![0xea; 0x8000];
            rom[..4].copy_from_slice(&[0xaf, 0, 0, 0x7e]);
            machine.cart.load(CartType::LoRom, &rom, 0x2000);
            machine.fast_mem = fast;
            machine.cpu.pc = 0x8000;
            machine.cpu.k = bank;
            machine.cpu.sp = 0x1ff;
            machine.cpu.e = false;
            machine.cpu.mf = false;
            let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(20, 100),
                CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
            let mut probe = RomCpuTimingProbe::new(machine, clock, SourcePpuReadState::snes9x_reset()).unwrap();
            let step = probe.step().unwrap();
            let mut audit = TimingAudit { owner: "main", ..Default::default() };
            audit.observe(&step, probe.snes(), 1, 0x1ff, 0, 0, &[(0, 0)]);
            let row = audit.rows[&(0, "main")];
            assert_eq!(row[1], expected);
            assert_eq!((row[7], row[8], row[14]), (4, 2, 0));
        }
    }
}
