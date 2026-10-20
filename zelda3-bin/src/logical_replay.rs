//! Lag-free parity: replay the ROM's logical-frame ledger one main-loop
//! iteration per engine frame and compare WRAM and video after each iteration.
//!
//! The ledger comes from `crates/snes/examples/logical_frame_ledger.rs`, which
//! runs the cartridge on the in-repo source CPU and cuts the route at every NMI
//! that lands in the wait loop. Lag frames vanish here by construction: each
//! engine frame is one completed iteration, fed the joypad word and cartridge
//! RNG outputs that the ROM's iteration consumed.
//!
//! Usage: zelda3 --logical-replay LEDGER_DIR SRAM [--frames N] [--image pre|post]
//!        [--offset K] [--report N] [--ignore lo-hi,...] [--histogram PATH]
//!        [--seed-every N] [--video CACHE_DIR [--video-host-offset D] [--video-misses]]
//!        [--apu-trace lo-hi] [--start K] [--dump-wram K:DIR] [--live-apu-reads]
//!
//! The cartridge's `$2140` reads (song-end polls) are replayed per iteration
//! like the RNG: their values depend on elapsed SPC time, which lag changes.
//! `--live-apu-reads` lets the engine's own SPC answer instead.
//!
//! `--start K` seeds at iteration K (a multiple of the seed interval) and skips
//! everything before it.
//!
//! WRAM is compared against the ROM image taken when the iteration's closing
//! NMI returns (`--image post`, the default) or is accepted (`pre`), skipping
//! [`PORT_REPRESENTATION`] and any `--ignore` ranges.
//!
//! With `--seed-every N`, the engine is re-seeded from the ledger's ROM state
//! (`seeds/iter-<k>.bin`) at every Nth iteration, so each segment measures
//! lag-free parity on its own: a timing-driven sequence (the intro's polygon
//! thread, an APU poll) that moves the engine off the ROM's iteration count
//! then costs one segment, not the rest of the route. An engine still in step
//! at a seed boundary continues instead (a seed into the middle of some
//! sequences cannot rebuild every runtime owner); `--eager-seed` reseeds at
//! every boundary.
//!
//! With `--video`, each engine frame is rendered and compared with the pinned
//! Snes9x frame that first shows the iteration's work: the ledger's
//! `picture_host`, the host scanning out the first frame after the NMI that
//! uploads it (cache frame N is retro_run N). Ledgers without it fall back to
//! the next iteration's display host plus `D` (default -1). A frame that differs is classified by whether it
//! equals the oracle picture one iteration later (`early`: the engine showed
//! the change a frame sooner), one earlier (`late`), or neither (`content`).

use crate::{
    apply_sram_to_game_or_exit,
    gpu_capture::{LiveGpuFrameCapture, NativeWindowOracleRenderer, QueuedGpuVideoDigest},
    load_romless_play_state,
    snes9x_compare::acquire_snes9x_compare_lock_mode,
};
use parity::av::VideoDigest;
use std::{
    collections::VecDeque,
    fs,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process,
};
use zelda3::{RomRandomSample, ZeldaState};

/// One iteration's cartridge RNG outputs as replay samples.
fn rng_samples(iteration: Option<&Iteration>, frame: u32) -> Vec<RomRandomSample> {
    iteration
        .map(|iteration| {
            iteration
                .rng
                .iter()
                .map(|&(value, carry)| RomRandomSample::with_carry(frame, value, carry))
                .collect()
        })
        .unwrap_or_default()
}

const WRAM_LEN: usize = 0x20000;

/// Snes9x powers WRAM on filled with $55 while the engine starts zeroed; a
/// byte the ROM has not yet moved off that fill is not compared.
const POWER_ON_FILL: u8 = 0x55;

/// WRAM the port represents differently from the cartridge without any
/// gameplay consequence: the stack page, direct-page and `$C8-$CD` scratch the
/// ROM's routines share (the translated routines keep those values in locals),
/// the port's own feature flags in unused WRAM, and the dialogue source
/// pointer (the ROM points into cartridge text; the port decodes its own text
/// source), the room-object data pointers (cartridge address vs asset offset),
/// the per-sprite draw position scratch (rewritten for every sprite drawn),
/// and the spotlight HDMA table, which the ROM builds at `$1B00` and the port
/// at `$7F:DBA0` (its visible effect is covered by the video check).
const PORT_REPRESENTATION: &[(usize, usize, &str)] = &[
    (0x0000, 0x000f, "direct-page scratch"),
    (0x0072, 0x0075, "SCRATCH_0/SCRATCH_1 scratch"),
    (0x00b7, 0x00bc, "DUNG_LOAD_PTR / DUNG_LOAD_PTR_OFFS (cartridge pointers)"),
    (0x00c8, 0x00cd, "R16-R18 scratch"),
    (0x0100, 0x01ff, "stack page"),
    (0x0648, 0x0648, "RAM_APUI00 (the port's echo of APU port 0)"),
    (0x0649, 0x064a, "port feature flags (RAM_BUGS_FIXED)"),
    (0x0fa8, 0x0fa9, "DRAW_WORK_POSITION_X/Y (per-sprite draw scratch)"),
    (0x1b00, 0x1ccf, "SAVELOAD_HDMA_TABLE (ROM spotlight table)"),
    (0x1cdd, 0x1cdf, "DIALOGUE_MSG_SRC_OFFS"),
    (0x1dba0, 0x1dd7f, "HDMA_TABLE_DYNAMIC (port spotlight table)"),
];

/// Mode-reused WRAM the port stages differently only while the ROM image is
/// in the given mode: `(lo, hi, applies, reason)`.
const MODE_REPRESENTATION: &[(usize, usize, fn(&[u8]) -> bool, &str)] = &[
    // Overworld scrolling (modules 09 and 0b) stages map16 tiles at $1100 for
    // its VRAM upload; the port builds that upload natively, and the staged
    // bytes linger through outdoor transitions and menus. Indoors the bytes
    // are door state and stay compared.
    (0x1100, 0x1233, |ram| ram[PLAYER_IS_INDOORS] == 0, "UVRAM_DATA_OVERWORLD (overworld upload staging)"),
];

const PLAYER_IS_INDOORS: usize = 0x1b;

/// `NMI_THREAD_ACTIVE`: the cartridge runs its polygon thread (the intro
/// triforce, the crystal, the ending) on the NMI clock, so it advances with
/// every lag NMI. A lag-free frame runs one NMI per iteration; these
/// sequences match in frames, not iterations, and are timing-defined here.
const NMI_THREAD_ACTIVE: usize = 0x012a;

/// Whether a WRAM image is in a sequence the cartridge advances per NMI: the
/// polygon thread, or the intro or attract module. Matches the ledger's
/// resync rule.
fn timing_defined(ram: &[u8]) -> bool {
    ram[NMI_THREAD_ACTIVE] != 0 || matches!(ram[0x10], 0x00 | 0x14)
}

/// WRAM owned by the polygon thread while it is active.
const POLY_THREAD_RANGES: &[(usize, usize)] = &[(0x1f00, 0x1fff), (0xe800, 0xefff)];

/// Scratch the port leaves differently everywhere except in the main module
/// that gives it a lasting meaning: `(lo, hi, owning module, reason)`.
const SCRATCH_OUTSIDE_MODULE: &[(usize, usize, u8, &str)] = &[
    // $BD is shared by tile detection and graphics decompression; only the
    // menu keeps a value there across frames (MENU_PREV_JOYPAD_H).
    (0x00bd, 0x00be, 0x0e, "HUD_TMP1 / MENU_PREV_JOYPAD_H scratch"),
];

#[derive(serde::Deserialize)]
struct Iteration {
    index: u64,
    open_host: usize,
    #[serde(default)]
    display_host: Option<usize>,
    /// Host whose scanned-out frame first shows this iteration's work.
    #[serde(default)]
    picture_host: Option<usize>,
    joypad: Option<u16>,
    lag_nmis: u32,
    rng: Vec<(u8, bool)>,
    /// `$2140-$2143` reads outside NMI as `(port, value)`.
    #[serde(default)]
    apu_reads: Vec<(u8, u8)>,
    /// `$2140-$2143` writes as `(port, value)`, opening handler first.
    #[serde(default)]
    apu_writes: Vec<(u8, u8)>,
    /// Scanline at which the closing NMI handler returned (absent in older
    /// ledgers).
    #[serde(default)]
    nmi_return_line: Option<u16>,
}

impl Iteration {
    /// The closing NMI handler overran vblank: its register writes landed
    /// during the scan-out of the frame that shows this iteration.
    fn nmi_overran_vblank(&self) -> bool {
        self.nmi_return_line.is_some_and(|line| line < 225)
    }

    fn shown_host(&self) -> usize {
        self.display_host.unwrap_or(self.open_host)
    }
}

struct WramImages {
    decoder: zstd::Decoder<'static, BufReader<fs::File>>,
    image: Vec<u8>,
    delta: Vec<u8>,
}

impl WramImages {
    fn open(path: &Path) -> Self {
        let file = fs::File::open(path).unwrap_or_else(|error| {
            eprintln!("failed to open {}: {error}", path.display());
            process::exit(2);
        });
        Self {
            decoder: zstd::Decoder::new(file).expect("zstd decoder"),
            image: vec![0; WRAM_LEN],
            delta: vec![0; WRAM_LEN],
        }
    }

    fn advance(&mut self) -> bool {
        if self.decoder.read_exact(&mut self.delta).is_err() {
            return false;
        }
        for (byte, delta) in self.image.iter_mut().zip(&self.delta) {
            *byte ^= delta;
        }
        true
    }
}

/// Port transitions to a nonzero value, updating `last` (per port).
fn port_commands(writes: impl IntoIterator<Item = (u8, u8)>, last: &mut [u8; 4]) -> Vec<(u8, u8)> {
    let mut commands = Vec::new();
    for (port, value) in writes {
        let slot = &mut last[usize::from(port & 3)];
        if value != 0 && value != *slot {
            commands.push((port, value));
        }
        *slot = value;
    }
    commands
}

/// Libretro joypad words carry the SNES `$4218/$4219` bits in reverse order.
fn libretro_word(joypad: u16) -> u16 {
    joypad.reverse_bits()
}

fn parse_ranges(text: &str) -> Vec<(usize, usize)> {
    text.split(',')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (lo, hi) = part.split_once('-').unwrap_or((part, part));
            let parse = |value: &str| {
                usize::from_str_radix(value.trim_start_matches("0x"), 16).unwrap_or_else(|error| {
                    eprintln!("invalid --ignore address {value:?}: {error}");
                    process::exit(2);
                })
            };
            (parse(lo), parse(hi))
        })
        .collect()
}

/// Diverging WRAM as merged address ranges (gaps of up to four bytes join).
fn diff_ranges(actual: &[u8], expected: &[u8], compared: &[bool]) -> Vec<(usize, usize)> {
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for (address, (a, e)) in actual.iter().zip(expected).enumerate() {
        if a == e || !compared[address] {
            continue;
        }
        match ranges.last_mut() {
            Some((_, end)) if address <= *end + 4 => *end = address,
            _ => ranges.push((address, address)),
        }
    }
    ranges
}

#[derive(Default)]
struct VideoTally {
    compared: u64,
    matched: u64,
    early: u64,
    late: u64,
    /// Different pictures while the ROM was lagging (its iteration or the
    /// next one took more than one host): HDMA and other raster reads then
    /// see Main's tables mid-rewrite, tearing a lag-free engine never shows.
    content_lag: u64,
    /// Different pictures while the polygon thread runs (timing-defined).
    content_poly: u64,
    /// Different pictures whose ROM frame was torn: the closing NMI handler
    /// overran vblank, so its scroll and register writes apply partway down.
    content_tear: u64,
}

impl VideoTally {
    fn content(&self) -> u64 {
        self.compared - self.matched - self.early - self.late - self.content_lag - self.content_poly - self.content_tear
    }
}

struct Segment {
    start: u64,
    iterations: u64,
    diverged: u64,
    /// Iteration, ROM module/submodule, and lowest diverging address.
    first_divergence: Option<(u64, u8, u8, usize)>,
    video: VideoTally,
    audio: AudioTally,
    /// Cartridge RNG outputs the engine left unused / draws it made past them.
    rng_unconsumed: u64,
    rng_extra: u64,
    /// Replayed `$2140` reads the engine's polls left unused / polls past them.
    apu_reads_unconsumed: u64,
    apu_reads_extra: u64,
    /// Iteration whose frame panicked; the rest of the segment is skipped.
    panicked: Option<u64>,
    /// ROM main module when the segment's first iteration completed.
    start_module: Option<u8>,
    /// Iterations with the cartridge's polygon thread active.
    poly_iterations: u64,
    /// Polygon-thread iterations whose WRAM differed: the thread paces Main
    /// (sprites wait on its steps), so these are timing-defined.
    poly_diverged: u64,
    /// The engine carried over from the previous segment instead of seeding.
    continued: bool,
    /// Latest iteration with a WRAM, RNG, APU command or video content miss.
    last_problem: Option<u64>,
    /// Frames that ended with translated work scheduled for a later frame.
    deferred_work_frames: u64,
}

/// An engine with no miss in this many iterations before a seed boundary is
/// still in step and continues instead of reseeding.
const RESEED_WINDOW: u64 = 200;

/// One native 32 kHz audio frame.
const AUDIO_SAMPLES: usize = 534;

/// APU commands matched between the engine and the ROM. The SPC driver acts
/// on a port changing to a nonzero value, so on both sides a command is a
/// nonzero port value that differs from that port's previous value: the ROM
/// writes each command once and zero at the next NMI, while the engine's
/// latches keep the value until it is replaced. The ROM sends Main k's
/// commands at the NMI opening iteration k + 1 or at a lag NMI inside k; the
/// engine's frame k pushes them at its own NMI.
#[derive(Default)]
struct AudioTally {
    engine_commands: u64,
    engine_unmatched: u64,
    rom_commands: u64,
    rom_unmatched: u64,
    /// ROM commands the engine never sends from an iteration with lag NMIs:
    /// a lag NMI sent a command that Main replaced later in the same
    /// iteration, which a lag-free frame never transmits.
    rom_unmatched_lag: u64,
    /// Iterations whose port writes are a song-bank upload, not commands.
    rom_upload_iterations: u64,
}

/// More port writes than this in one iteration is a song-bank upload: the
/// NMI and lag NMIs write at most four ports each.
const APU_UPLOAD_WRITES: usize = 256;

impl Segment {
    fn starting(start: u64) -> Self {
        Self {
            start,
            iterations: 0,
            diverged: 0,
            first_divergence: None,
            video: VideoTally::default(),
            audio: AudioTally::default(),
            rng_unconsumed: 0,
            rng_extra: 0,
            apu_reads_unconsumed: 0,
            apu_reads_extra: 0,
            panicked: None,
            start_module: None,
            poly_iterations: 0,
            poly_diverged: 0,
            continued: false,
            last_problem: None,
            deferred_work_frames: 0,
        }
    }

    /// Whether the engine is out of step with the ROM at the end of this
    /// segment: it panicked, or game WRAM, RNG consumption, an APU command or
    /// the picture's content differed within the last `RESEED_WINDOW`
    /// iterations.
    fn diverged_from_rom(&self, boundary: u64) -> bool {
        self.panicked.is_some()
            || self
                .last_problem
                .is_some_and(|iteration| iteration + RESEED_WINDOW >= boundary)
    }

    fn problem_at(&mut self, iteration: u64) {
        self.last_problem = Some(self.last_problem.map_or(iteration, |last| last.max(iteration)));
    }

    fn report(&self, video: bool) {
        let wram = match self.first_divergence {
            None => "wram=clean".to_owned(),
            Some((iteration, module, submodule, address)) => format!(
                "wram_diverged={} first={iteration} module={module:02x}/{submodule:02x} address={address:05x}",
                self.diverged
            ),
        };
        let video = if video {
            format!(
                " video={}/{} early={} late={} content_lag={} content={} content_poly={} content_tear={}",
                self.video.matched,
                self.video.compared,
                self.video.early,
                self.video.late,
                self.video.content_lag,
                self.video.content(),
                self.video.content_poly,
                self.video.content_tear
            )
        } else {
            String::new()
        };
        let audio = format!(
            " apu_commands engine={} unmatched={} rom={} unmatched={} unmatched_lag={} uploads={}",
            self.audio.engine_commands,
            self.audio.engine_unmatched,
            self.audio.rom_commands,
            self.audio.rom_unmatched,
            self.audio.rom_unmatched_lag,
            self.audio.rom_upload_iterations
        );
        let panicked = self
            .panicked
            .map(|iteration| format!(" panicked={iteration}"))
            .unwrap_or_default();
        println!(
            "logical-segment start={} iterations={} {wram}{video}{audio} rng unconsumed={} extra={}{panicked} start_module={:02x} apu_reads unconsumed={} extra={} seeded={} deferred_work={}",
            self.start,
            self.iterations,
            self.rng_unconsumed,
            self.rng_extra,
            self.start_module.unwrap_or(0xff),
            self.apu_reads_unconsumed,
            self.apu_reads_extra,
            !self.continued,
            self.deferred_work_frames
        );
        if self.poly_iterations != 0 {
            println!(
                "logical-segment-poly start={} iterations={} wram_diverged={}",
                self.start, self.poly_iterations, self.poly_diverged
            );
        }
    }
}

struct PendingVideo {
    iteration_index: usize,
    oracle_frame: usize,
    queued: QueuedGpuVideoDigest,
}

/// Renders engine frames and checks them against the oracle cache's frame
/// hashes, keeping the GPU readback pipeline full like the cached A/V replay.
struct VideoCheck {
    renderer: NativeWindowOracleRenderer,
    oracle: Vec<Option<VideoDigest>>,
    host_offset: i64,
    pending: VecDeque<PendingVideo>,
    print_misses: bool,
    /// Whether the polygon thread was active at each queued iteration.
    poly_active: std::collections::HashMap<usize, bool>,
    first_mismatch: Option<(u64, usize)>,
    total: VideoTally,
}

impl VideoCheck {
    fn load(cache: &Path, host_offset: i64, print_misses: bool) -> Self {
        #[derive(serde::Deserialize)]
        struct Record {
            video: Option<VideoDigest>,
        }
        let file = fs::File::open(cache.join("oracle-av-hashes.jsonl")).unwrap_or_else(|error| {
            eprintln!("failed to open oracle hashes: {error}");
            process::exit(2);
        });
        let oracle = BufReader::new(file)
            .lines()
            .map(|line| {
                serde_json::from_str::<Record>(&line.expect("oracle line"))
                    .expect("oracle record")
                    .video
            })
            .collect();
        let renderer = NativeWindowOracleRenderer::load_from_env().unwrap_or_else(|error| {
            eprintln!("failed to initialize GPU renderer: {error}");
            process::exit(1);
        });
        Self {
            renderer,
            oracle,
            host_offset,
            pending: VecDeque::new(),
            print_misses,
            poly_active: Default::default(),
            first_mismatch: None,
            total: VideoTally::default(),
        }
    }

    /// Cache frame first showing iteration `index`'s work: the ledger's
    /// picture host (cache frame N is retro_run N), else the older estimate.
    fn oracle_frame(&self, iterations: &[Iteration], index: usize) -> Option<usize> {
        let host = match iterations.get(index)?.picture_host {
            Some(host) => host as i64,
            None => iterations.get(index + 1)?.shown_host() as i64 + self.host_offset,
        };
        let frame = usize::try_from(host).ok()?;
        self.oracle.get(frame)?.as_ref().map(|_| frame)
    }

    fn queue(&mut self, game: &mut ZeldaState, iterations: &[Iteration], index: usize, segment: &mut Segment) {
        let Some(oracle_frame) = self.oracle_frame(iterations, index) else {
            return;
        };
        let capture = LiveGpuFrameCapture::from_game_at_comparison_frame(game, oracle_frame as u32);
        let queued = self.renderer.queue_capture_video_digest(capture).unwrap_or_else(|error| {
            eprintln!("video submission failed: {error}");
            process::exit(1);
        });
        self.pending.push_back(PendingVideo {
            iteration_index: index,
            oracle_frame,
            queued,
        });
        if self.pending.len() == renderer::MODERN_GPU_READBACK_PIPELINE_DEPTH {
            self.finish_one(iterations, segment);
        }
    }

    fn drain(&mut self, iterations: &[Iteration], segment: &mut Segment) {
        while !self.pending.is_empty() {
            self.finish_one(iterations, segment);
        }
    }

    fn finish_one(&mut self, iterations: &[Iteration], segment: &mut Segment) {
        let pending = self.pending.pop_front().expect("pending video frame");
        let actual = self.renderer.finish_game_video_digest(pending.queued).unwrap_or_else(|error| {
            eprintln!("video readback failed: {error}");
            process::exit(1);
        });
        let oracle_at = |index: Option<usize>| {
            index
                .and_then(|index| self.oracle_frame(iterations, index))
                .and_then(|frame| self.oracle[frame].as_ref())
        };
        let index = pending.iteration_index;
        let class = if Some(&actual) == oracle_at(Some(index)) {
            "match"
        } else if Some(&actual) == oracle_at(index.checked_add(1)) {
            "early"
        } else if Some(&actual) == oracle_at(index.checked_sub(1)) {
            "late"
        } else if self.poly_active.remove(&index).unwrap_or(false) {
            "content_poly"
        } else if iterations[index].nmi_overran_vblank() {
            "content_tear"
        } else if [index, index + 1]
            .iter()
            .any(|&i| iterations.get(i).is_some_and(|iteration| iteration.lag_nmis != 0))
        {
            "content_lag"
        } else {
            "content"
        };
        for tally in [&mut self.total, &mut segment.video] {
            tally.compared += 1;
            match class {
                "match" => tally.matched += 1,
                "early" => tally.early += 1,
                "late" => tally.late += 1,
                "content_lag" => tally.content_lag += 1,
                "content_poly" => tally.content_poly += 1,
                "content_tear" => tally.content_tear += 1,
                _ => {}
            }
        }
        if class == "content" {
            segment.problem_at(iterations[index].index);
        }
        if class != "match" {
            let iteration = &iterations[index];
            self.first_mismatch.get_or_insert((iteration.index, pending.oracle_frame));
            if self.print_misses {
                println!(
                    "logical-video-miss iteration={} oracle_frame={} lag_nmis={} class={class} sha256={}",
                    iteration.index, pending.oracle_frame, iteration.lag_nmis, actual.sha256
                );
            }
        }
    }
}

/// The ledger's RNG outputs must be exactly the cache's `rom-random.txt`
/// (recorded from the pinned Snes9x core) over the hosts the ledger covers;
/// a ledger that missed the routine's store would silently starve the engine.
fn check_ledger_rng(cache: &Path, iterations: &[Iteration]) {
    let Some(last_host) = iterations.last().map(|iteration| iteration.open_host as u32) else {
        return;
    };
    let text = fs::read_to_string(cache.join("rom-random.txt")).unwrap_or_else(|error| {
        eprintln!("failed to read cache rom-random.txt: {error}");
        process::exit(2);
    });
    let cache_samples: Vec<(u8, bool)> = zelda3::parse_rom_random_script(&text)
        .unwrap_or_else(|error| {
            eprintln!("failed to parse cache rom-random.txt: {error}");
            process::exit(2);
        })
        .into_iter()
        .filter(|sample| sample.execution_frame < last_host)
        .map(|sample| (sample.value, sample.carry))
        .collect();
    let ledger_samples: Vec<(u8, bool)> = iterations
        .iter()
        .flat_map(|iteration| iteration.rng.iter().copied())
        .collect();
    let compared = cache_samples.len().min(ledger_samples.len());
    let first_difference = (0..compared).find(|&i| cache_samples[i] != ledger_samples[i]);
    if first_difference.is_some() || cache_samples.len().abs_diff(ledger_samples.len()) > 8 {
        eprintln!(
            "ledger RNG disagrees with the cache: ledger {} samples, cache {} before host {last_host}, first difference at sample {first_difference:?}",
            ledger_samples.len(),
            cache_samples.len()
        );
        process::exit(2);
    }
    println!("ledger RNG matches the cache: {compared} samples");
}

/// Replace the engine's machine state with a ledger seed (layout written by
/// `logical_frame_ledger::write_seed`) and leave it in lag-free play.
fn seed_game(game: &mut ZeldaState, seed: &[u8], host_frame: u32) {
    let (wram, rest) = seed.split_at(WRAM_LEN);
    let (vram, rest) = rest.split_at(0x10000);
    let (cgram, rest) = rest.split_at(0x200);
    let (oam, rest) = rest.split_at(0x220);
    let (sram, ppu_json) = rest.split_at(game.sram.len().min(rest.len()));
    let cgram: Vec<u16> = cgram
        .chunks_exact(2)
        .map(|word| u16::from_le_bytes([word[0], word[1]]))
        .collect();
    game.seed_memory_from_foreign_image(wram, vram, sram, &cgram, oam, host_frame)
        .unwrap_or_else(|error| {
            eprintln!("failed to seed from ledger: {error}");
            process::exit(1);
        });
    game.adopt_cartridge_hdma_table();
    if !ppu_json.is_empty() {
        // The ROM's PPU registers (BG modes, OBJ base, windows, color math):
        // some are written once at init, so WRAM shadows cannot rebuild them.
        // The engine keeps its own presentation configuration.
        let mut ppu: snes::PpuState = serde_json::from_slice(ppu_json).unwrap_or_else(|error| {
            eprintln!("failed to parse seed PPU state: {error}");
            process::exit(1);
        });
        ppu.render_flags = game.ppu.render_flags;
        ppu.render_pitch = game.ppu.render_pitch;
        ppu.render_buffer = game.ppu.render_buffer.take();
        ppu.extra_left_cur = game.ppu.extra_left_cur;
        ppu.extra_right_cur = game.ppu.extra_right_cur;
        ppu.extra_left_right = game.ppu.extra_left_right;
        ppu.extra_bottom_cur = game.ppu.extra_bottom_cur;
        ppu.mode7_perspective_low = game.ppu.mode7_perspective_low;
        ppu.mode7_perspective_high = game.ppu.mode7_perspective_high;
        game.ppu = ppu;
    }
}

pub(crate) fn run_logical_replay(args: &[String]) {
    let mut positional = Vec::new();
    let mut frames = u64::MAX;
    let mut image_kind = "post".to_owned();
    let mut offset = 0i64;
    let mut report = 8usize;
    let mut ignore: Vec<(usize, usize)> =
        PORT_REPRESENTATION.iter().map(|&(lo, hi, _)| (lo, hi)).collect();
    let mut histogram_path: Option<PathBuf> = None;
    let mut video_cache: Option<PathBuf> = None;
    // Cache frame N is the picture of engine host N + 1.
    let mut video_host_offset = -1i64;
    let mut video_misses = false;
    let mut seed_every: Option<u64> = None;
    let mut apu_trace: Option<(u64, u64)> = None;
    let mut start_at: u64 = 0;
    let mut apu_read_replay = true;
    let mut eager_seed = false;
    let mut dump_wram: Option<(u64, PathBuf)> = None;
    let mut index = 0;
    while index < args.len() {
        let value = |index: usize| {
            args.get(index + 1).cloned().unwrap_or_else(|| {
                eprintln!("{} needs a value", args[index]);
                process::exit(2);
            })
        };
        match args[index].as_str() {
            "--frames" => frames = value(index).parse().expect("--frames"),
            "--image" => image_kind = value(index),
            "--offset" => offset = value(index).parse().expect("--offset"),
            "--report" => report = value(index).parse().expect("--report"),
            "--ignore" => ignore.extend(parse_ranges(&value(index))),
            "--histogram" => histogram_path = Some(PathBuf::from(value(index))),
            "--apu-trace" => {
                let text = value(index);
                let (lo, hi) = text.split_once('-').unwrap_or((&text, &text));
                apu_trace = Some((lo.parse().expect("--apu-trace"), hi.parse().expect("--apu-trace")));
            }
            "--dump-wram" => {
                let text = value(index);
                let (iteration, dir) = text.split_once(':').unwrap_or_else(|| {
                    eprintln!("--dump-wram takes ITERATION:DIR");
                    process::exit(2);
                });
                dump_wram = Some((iteration.parse().expect("--dump-wram iteration"), PathBuf::from(dir)));
            }
            "--eager-seed" => {
                eager_seed = true;
                index += 1;
                continue;
            }
            "--live-apu-reads" => {
                apu_read_replay = false;
                index += 1;
                continue;
            }
            "--start" => start_at = value(index).parse().expect("--start"),
            "--seed-every" => seed_every = Some(value(index).parse().expect("--seed-every")),
            "--video" => video_cache = Some(PathBuf::from(value(index))),
            "--video-host-offset" => {
                video_host_offset = value(index).parse().expect("--video-host-offset")
            }
            "--video-misses" => {
                video_misses = true;
                index += 1;
                continue;
            }
            other => {
                positional.push(other.to_owned());
                index += 1;
                continue;
            }
        }
        index += 2;
    }
    let [ledger, sram_path] = positional.as_slice() else {
        eprintln!("usage: zelda3 --logical-replay LEDGER_DIR SRAM [--frames N] [--image pre|post] [--offset K] [--report N] [--ignore lo-hi,...] [--histogram PATH] [--seed-every N] [--video CACHE_DIR [--video-host-offset D] [--video-misses]]");
        process::exit(2);
    };
    let ledger = PathBuf::from(ledger);
    let records = fs::File::open(ledger.join("iterations.jsonl")).unwrap_or_else(|error| {
        eprintln!("failed to open ledger: {error}");
        process::exit(2);
    });
    // A ledger still being written ends in a partial line: stop there.
    let iterations: Vec<Iteration> = BufReader::new(records)
        .lines()
        .map_while(|line| serde_json::from_str(&line.ok()?).ok())
        .collect();
    if let Some(cache) = video_cache.as_deref() {
        check_ledger_rng(cache, &iterations);
    }
    // The ledger seeds the first iteration after each timing-defined stretch;
    // an engine that drifted there (or is still inside its own stretch)
    // realigns at once instead of at the next periodic seed.
    let resync: std::collections::BTreeSet<u64> = fs::read_to_string(ledger.join("seeds").join("resync.txt"))
        .map(|text| text.lines().filter_map(|line| line.parse().ok()).collect())
        .unwrap_or_default();
    let mut images = WramImages::open(&ledger.join(format!("wram-{image_kind}-nmi.xor.zst")));
    // Image j of either stream belongs to the j-th wait-loop NMI, which opens
    // iteration j; image j+1 is the state once iteration j has finished.
    if !images.advance() {
        eprintln!("ledger has no WRAM images");
        process::exit(2);
    }
    let mut compared = vec![true; WRAM_LEN];
    for &(lo, hi) in &ignore {
        compared[lo..=hi.min(WRAM_LEN - 1)].fill(false);
    }

    let sram = fs::read(sram_path).unwrap_or_else(|error| {
        eprintln!("failed to read SRAM {sram_path}: {error}");
        process::exit(2);
    });
    let mut game = load_romless_play_state();
    game.set_lag_free(true);
    apply_sram_to_game_or_exit(&mut game, Path::new(sram_path), &sram);

    let _compare_lock = video_cache.is_some().then(|| acquire_snes9x_compare_lock_mode(true));
    let mut video = video_cache
        .as_deref()
        .map(|cache| VideoCheck::load(cache, video_host_offset, video_misses));
    // Per address: iterations diverged, first diverging iteration.
    let mut histogram = vec![(0u64, 0u64); WRAM_LEN];
    let mut written: Vec<bool> = images.image.iter().map(|&byte| byte != POWER_ON_FILL).collect();
    let mut segment = Segment::starting(0);
    let mut audio_buffer = vec![0i16; AUDIO_SAMPLES * 2];
    let mut last_apu_push = game.zelda_last_pushed_apu_ports().0;
    let mut engine_ports = game.zelda_last_pushed_apu_ports().1;
    let mut rom_ports = [0u8; 4];
    // ROM commands per iteration, computed in order as iterations are reached.
    let mut rom_commands: Vec<Vec<(u8, u8)>> = Vec::with_capacity(iterations.len());
    let mut previous_engine_commands: Vec<(u8, u8)> = Vec::new();
    let mut audio_reported = 0usize;
    let mut deferred_work_reported = std::collections::BTreeSet::new();
    let mut rng_reported = 0usize;
    let mut reported = 0usize;
    let mut clean = 0u64;
    let mut first_divergence: Option<u64> = None;
    for frame in 0..(iterations.len() as i64 + offset).max(0) as u64 {
        let iteration_index = frame as i64 - offset;
        let iteration = usize::try_from(iteration_index)
            .ok()
            .and_then(|index| iterations.get(index));
        if iteration.is_some_and(|iteration| iteration.index >= frames) {
            break;
        }
        if iteration.is_some_and(|iteration| iteration.index < start_at) {
            if !images.advance() {
                break;
            }
            continue;
        }
        if let (Some(every), Some(iteration)) = (seed_every, iteration) {
            let at_resync = resync.contains(&iteration.index) && iteration.index != start_at;
            if (iteration.index % every == 0 && iteration.index != 0)
                || (iteration.index == start_at && start_at != 0)
                || at_resync
            {
                let path = ledger.join("seeds").join(format!("iter-{}.bin", iteration.index));
                if let Some(video) = video.as_mut() {
                    video.drain(&iterations, &mut segment);
                }
                // Seeds contain drift; a seed into the middle of some
                // sequences cannot rebuild every runtime owner. Unless asked
                // to seed every boundary, an engine still in step continues.
                let continues = !eager_seed
                    && iteration.index != start_at
                    && !segment.diverged_from_rom(iteration.index)
                    && !(at_resync && timing_defined(&game.ram));
                if continues && at_resync && iteration.index % every != 0 {
                    // In step after the stretch: the segment carries on.
                } else if continues {
                    segment.report(video.is_some());
                    segment = Segment::starting(iteration.index);
                    segment.continued = true;
                } else if let Ok(seed) = fs::read(&path) {
                    segment.report(video.is_some());
                    // A fresh engine per segment: nothing the previous
                    // segment left running (a suspended translated call)
                    // carries across the seed.
                    game = load_romless_play_state();
                    game.set_lag_free(true);
                    seed_game(&mut game, &seed, iteration.open_host as u32);
                    if let Ok(dma) = fs::read(path.with_extension("dma.json")) {
                        game.dma = serde_json::from_slice(&dma).unwrap_or_else(|error| {
                            eprintln!("failed to parse seed DMA state: {error}");
                            process::exit(1);
                        });
                    }
                    written.fill(true);
                    segment = Segment::starting(iteration.index);
                    last_apu_push = game.zelda_last_pushed_apu_ports().0;
                    engine_ports = game.zelda_last_pushed_apu_ports().1;
                    previous_engine_commands.clear();
                }
            }
        }
        if segment.panicked.is_some() {
            // The engine panicked earlier in this segment: keep the ROM image
            // stream in step and wait for the next seed.
            if iteration.is_some() && !images.advance() {
                break;
            }
            continue;
        }
        // The engine runs Main and then the NMI that reads the pad, so Main of
        // frame f consumes what the previous frame's NMI read. The ROM's NMI
        // opening iteration k + 1 latches the pad Main k + 1 uses: the engine's
        // frame k must be handed that word.
        let input = usize::try_from(iteration_index + 1)
            .ok()
            .and_then(|index| iterations.get(index))
            .and_then(|iteration| iteration.joypad)
            .map(libretro_word)
            .unwrap_or(0);
        // Each frame draws exactly its iteration's RNG outputs; a draw the ROM
        // did not make, or an output left unused, is a divergence to count.
        let (unconsumed, extra) = game.replace_rom_random_frame(rng_samples(iteration, frame as u32), frame as u32);
        if segment.iterations != 0 || segment.continued {
            segment.rng_unconsumed += unconsumed as u64;
            segment.rng_extra += u64::from(extra);
            if unconsumed != 0 || extra != 0 {
                segment.problem_at(iteration_index.max(1) as u64 - 1);
            }
            if (unconsumed != 0 || extra != 0) && rng_reported < report {
                rng_reported += 1;
                println!(
                    "logical-rng iteration={} unconsumed={unconsumed} extra={extra} module={:02x}/{:02x}",
                    iteration_index - 1,
                    game.ram[0x10],
                    game.ram[0x11]
                );
            }
        }
        if apu_read_replay {
            // A song-bank upload's handshake reads are not song-end polls.
            let reads = iteration
                .filter(|iteration| iteration.apu_writes.len() <= APU_UPLOAD_WRITES)
                .map(|iteration| {
                    iteration
                        .apu_reads
                        .iter()
                        .filter(|(port, _)| *port == 0)
                        .map(|(_, value)| *value)
                        .fold(Vec::new(), |mut values: Vec<u8>, value| {
                            // A busy-wait spins on one value for as long as
                            // the SPC takes: only the value sequence is logic.
                            if values.last() != Some(&value) {
                                values.push(value);
                            }
                            values
                        })
                })
                .unwrap_or_default();
            let (unconsumed, extra) = game.replace_apui00_poll_frame(reads);
            if segment.iterations != 0 || segment.continued {
                segment.apu_reads_unconsumed += unconsumed as u64;
                segment.apu_reads_extra += u64::from(extra);
            }
        }
        let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            game.zelda_run_frame(i32::from(input));
            // The audio side consumes the frame's commands and takes the SPC's
            // acknowledgements, which the next NMI needs to clear a port and
            // send a repeated command, as in a frontend.
            game.zelda_render_audio(&mut audio_buffer, AUDIO_SAMPLES as i32, 2);
        }));
        if ran.is_err() {
            segment.panicked = Some(iteration_index.max(0) as u64);
            if video.is_some() {
                // Drop frames queued from the poisoned engine's segment.
                if let Some(video) = video.as_mut() {
                    video.drain(&iterations, &mut segment);
                }
            }
            if iteration.is_some() && !images.advance() {
                break;
            }
            continue;
        }
        if let Some(work) = game.pending_game_work_debug() {
            segment.deferred_work_frames += 1;
            let label = work.split(['{', '(', ' ']).find(|part| part.len() > 4).unwrap_or(&work).to_owned();
            if deferred_work_reported.insert(label) {
                println!(
                    "logical-deferred-work iteration={} module={:02x}/{:02x} work={work}",
                    iteration_index, game.ram[0x10], game.ram[0x11]
                );
            }
        }
        let (pushes, ports) = game.zelda_last_pushed_apu_ports();
        let last_apu_push_before = last_apu_push;
        let engine_commands = if pushes != last_apu_push {
            port_commands((0..4u8).zip(ports), &mut engine_ports)
        } else {
            Vec::new()
        };
        last_apu_push = pushes;
        let Some(iteration) = iteration else { continue };
        let index = iteration_index as usize;
        if apu_trace.is_some_and(|(lo, hi)| (lo..=hi).contains(&iteration.index)) {
            println!(
                "logical-apu-trace iteration={} engine_push={} engine_ports={ports:02x?} rom_writes={:02x?} rom_012c={:02x?}",
                iteration.index,
                pushes != last_apu_push_before,
                iteration.apu_writes,
                &game.ram[0x12c..0x130],
            );
        }
        while rom_commands.len() <= index + 1 && rom_commands.len() < iterations.len() {
            let next = &iterations[rom_commands.len()];
            rom_commands.push(port_commands(next.apu_writes.iter().copied(), &mut rom_ports));
        }
        // Engine frame k against ROM iterations k and k + 1. The first frame
        // of a seeded segment re-sends the restored song, and the last
        // iteration has no successor to hold its commands.
        let audio_checked = (segment.iterations != 0 || segment.continued) && index + 1 < iterations.len();
        for command in engine_commands.iter().filter(|_| audio_checked) {
            segment.audio.engine_commands += 1;
            let found = [index, index + 1].iter().any(|&i| {
                rom_commands.get(i).is_some_and(|rom| rom.contains(command))
                    || iterations.get(i).is_some_and(|rom| rom.apu_writes.len() > APU_UPLOAD_WRITES)
            });
            if !found {
                segment.audio.engine_unmatched += 1;
                segment.problem_at(iteration.index);
                if audio_reported < report {
                    audio_reported += 1;
                    println!("logical-apu-extra iteration={} port={} value={:#04x}", iteration.index, command.0, command.1);
                }
            }
        }
        // ROM iteration k + 1 against engine frames k and k + 1; the frame
        // after k is not run yet, so check iteration k against frames k - 1, k.
        // A song-bank upload streams data through the ports; it is not a
        // command stream (the engine uploads banks internally).
        let rom_upload = iteration.apu_writes.len() > APU_UPLOAD_WRITES;
        if rom_upload {
            segment.audio.rom_upload_iterations += 1;
        }
        for &command in rom_commands[index].iter().filter(|_| !rom_upload) {
            if segment.iterations == 0 && !segment.continued {
                // A seeded segment has no engine frame k - 1.
                break;
            }
            segment.audio.rom_commands += 1;
            if !(previous_engine_commands.contains(&command) || engine_commands.contains(&command)) {
                if iteration.lag_nmis != 0 {
                    segment.audio.rom_unmatched_lag += 1;
                    continue;
                }
                segment.audio.rom_unmatched += 1;
                segment.problem_at(iteration.index);
                if audio_reported < report {
                    audio_reported += 1;
                    println!("logical-apu-missing iteration={} port={} value={:#04x}", iteration.index, command.0, command.1);
                }
            }
        }
        previous_engine_commands = engine_commands;
        if let Some(video) = video.as_mut() {
            video.queue(&mut game, &iterations, iteration_index as usize, &mut segment);
        }
        if !images.advance() {
            break;
        }
        if let Some((target, dir)) = dump_wram.as_ref().filter(|(target, _)| *target == iteration.index) {
            fs::create_dir_all(dir).expect("create --dump-wram directory");
            fs::write(dir.join(format!("engine-{target}.bin")), &game.ram[..WRAM_LEN]).expect("write engine WRAM");
            fs::write(dir.join(format!("rom-{target}.bin")), &images.image).expect("write ROM WRAM");
        }
        for (written, &byte) in written.iter_mut().zip(&images.image) {
            *written |= byte != POWER_ON_FILL;
        }
        let mut compared_now: Vec<bool> = compared.iter().zip(&written).map(|(c, w)| *c && *w).collect();
        for &(lo, hi, applies, _) in MODE_REPRESENTATION {
            if applies(&images.image) {
                compared_now[lo..=hi].fill(false);
            }
        }
        for &(lo, hi, module, _) in SCRATCH_OUTSIDE_MODULE {
            if images.image[0x10] != module && zelda3::debug_env::var_os("ZELDA3_LOGICAL_COMPARE_ALL_SCRATCH").is_none() {
                compared_now[lo..=hi].fill(false);
            }
        }
        let poly_active = images.image[NMI_THREAD_ACTIVE] != 0 || game.ram[NMI_THREAD_ACTIVE] != 0;
        if poly_active {
            segment.poly_iterations += 1;
            for &(lo, hi) in POLY_THREAD_RANGES {
                compared_now[lo..=hi].fill(false);
            }
        }
        if let Some(video) = video.as_mut() {
            video.poly_active.insert(index, poly_active);
        }
        let ranges = diff_ranges(&game.ram, &images.image, &compared_now);
        for &(lo, hi) in &ranges {
            for address in lo..=hi {
                if game.ram[address] != images.image[address] && compared_now[address] {
                    let slot = &mut histogram[address];
                    if slot.0 == 0 {
                        slot.1 = iteration.index;
                    }
                    slot.0 += 1;
                }
            }
        }
        segment.start_module.get_or_insert(images.image[0x10]);
        segment.iterations += 1;
        if ranges.is_empty() {
            clean += 1;
            continue;
        }
        segment.problem_at(iteration.index);
        if poly_active {
            segment.poly_diverged += 1;
            continue;
        }
        segment.diverged += 1;
        segment.first_divergence.get_or_insert((
            iteration.index,
            images.image[0x10],
            images.image[0x11],
            ranges[0].0,
        ));
        first_divergence.get_or_insert(iteration.index);
        if reported < report {
            reported += 1;
            let bytes: usize = ranges.iter().map(|(lo, hi)| hi - lo + 1).sum();
            let shown: Vec<String> = ranges
                .iter()
                .take(12)
                .map(|&(lo, hi)| {
                    let hex = |bytes: &[u8]| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
                    let end = hi.min(lo + 7);
                    format!(
                        "{lo:05x}-{hi:05x} rust={} rom={}",
                        hex(&game.ram[lo..=end]),
                        hex(&images.image[lo..=end])
                    )
                })
                .collect();
            println!(
                "logical-diff iteration={} engine_frame={frame} host={} lag_nmis={} ranges={} span_bytes={bytes} module={:02x}/{:02x}\n  {}",
                iteration.index,
                iteration.open_host,
                iteration.lag_nmis,
                ranges.len(),
                images.image[0x10],
                images.image[0x11],
                shown.join("\n  "),
            );
        }
    }
    if let Some(video) = video.as_mut() {
        video.drain(&iterations, &mut segment);
    }
    segment.report(video.is_some());
    if let Some(path) = histogram_path {
        let lines: String = histogram
            .iter()
            .enumerate()
            .filter(|(_, (count, _))| *count != 0)
            .map(|(address, (count, first))| format!("{address:05x} {count} {first}\n"))
            .collect();
        fs::write(&path, lines).unwrap_or_else(|error| {
            eprintln!("failed to write {}: {error}", path.display());
            process::exit(2);
        });
    }
    println!("logical replay: {clean} clean iteration(s); first WRAM divergence {first_divergence:?}");
    if let Some(video) = video {
        let total = &video.total;
        println!(
            "logical video: {}/{} match, {} early, {} late, {} content during ROM lag, {} content; first mismatch (iteration, oracle frame) {:?}",
            total.matched,
            total.compared,
            total.early,
            total.late,
            total.content_lag,
            total.content(),
            video.first_mismatch
        );
    }
}
