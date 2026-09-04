//! The Snes9x oracle comparison harness: scripted exact video+audio
//! comparison, replay validation, session receipts, and failure artifacts.

use crate::*;

use std::collections::VecDeque;
use std::env;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Component, Path, PathBuf};
use std::process;
use std::sync::atomic::Ordering;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::audio_trace::AudioFrameStats;
use crate::gpu_capture::NativeWindowOracleRenderer;
use crate::image_output::{write_argb_frame_png, write_rgba_frame_png};
use crate::input_script::InputScript;
use crate::libretro_timeline::{
    format_input_history, AudioComparisonMode, AudioTimingOptions, StreamingAudioComparator,
};
use crate::render_diagnostics::format_render_ppu_summary;
use crate::snes9x_presented_bg_scroll::snes9x_presented_bg_scroll;
use crate::snes9x_presented_bg_tilemaps::{snes9x_presented_bg_tilemaps, PresentedBgTilemapCache};
use crate::snes9x_presented_dialogue_text::snes9x_presented_dialogue_text;
use crate::snes9x_presented_mode7::snes9x_presented_mode7_transform;
use crate::snes9x_presented_window_mask::snes9x_presented_window_mask;
use crate::snes9x_semantic_receipts::{
    Snes9xOracleSemanticTrace, Snes9xOracleSemanticTraceCheckpoint,
};
use serde::{Deserialize, Serialize};
use zelda3::{
    game_output::DspWriteEvent, NmiPpuRegisterOperands, OriginalTimingHostReceipts,
    OriginalTimingSemanticReceipt, PresentedAnimatedBgTiles, PresentedAudio, PresentedCgram,
    PresentedHudTilemap, PresentedOam, PresentedObjTiles, RomRandomSample, ZeldaState, RUN_MAIN,
};

pub(crate) const ORACLE_MUSIC_CONTROL: usize = 0x012c;
pub(crate) const ORACLE_QUEUED_MUSIC_CONTROL: usize = 0x0132;
pub(crate) const ORACLE_LAST_MUSIC_CONTROL: usize = 0x0133;
const COMPARE_ORACLE_USAGE: &str = "<path-to-snes-libretro.dylib> <path-to-rom.sfc> [frames] [--replay-bundle <session-dir> | --input-script <path> --rom-random-script <path> --load-sram <path>] [--allow-mixed-replay-provenance] [--replay-save <path>] [--rom-random-script <path> | --live-oracle-rng] [--resume-paired <dir> | --resume-rust-state <path> --resume-oracle-state <path> [--resume-oracle-sram <path>]] [--save-paired-resume-at <frame> <dir>] [--save-rolling-paired-resume <interval> <dir>] [--native-apu-bootstrap <path>] [--ignore-video] [--ignore-audio] [--compare-from-frame <n>] [--compare-engine-state-from-frame <n> | --ignore-engine-state] [--skip-oracle-frames <n>] [--audio-comparison timing|exact] [--session-dir <path>] [--cold-evidence-invocation-id <id>] [--scan-all] (engine state is compared from --compare-from-frame by default; pass both --ignore-video and --ignore-audio for renderless replay)";

// The cartridge RNG routine stores its return byte at mapped PC $0d:ba7f.
// Other game code also writes $0fa1, so the address alone is not sufficient
// provenance for a replay sample.
const CARTRIDGE_RNG_STORE_PC_LOW16: u64 = 0xba7f;
const LIVE_ORACLE_RNG_TRACE_ARTIFACT: &str = "oracle-rom-random.jsonl";
// Schema 49 adds exact Lanmola draw publication and the zero-hit-timer branch,
// and recognizes the publication-free `$00:F361` spotlight loop prefix.
// Older caches cannot prove these source statement boundaries.
// Schema 50 adds source-ordered partial publication of Intro_ValidateSram's
// final low-WRAM clear. Schema 49 ledgers only proved the loop's completion.
// Schema 51 corrects SpritePrep_Zelda's pinned follower-loader return address.
// Schema 52 removes the obsolete Module0F-entry suppression and preserves the
// entry call's exact Link_MovePosition host-return prefix. Schema 53
// distinguishes the source interval after the low coordinate-byte store from
// the later state where both coordinate stores have committed.
// Schema 58 adds caller-owned partial publication for the master-sword light
// beam's replacement `Sprite_SpawnDynamically` call. Schema 57 caches can
// prove its movement prefix but not a host boundary inside the spawn helper.
// Schema 60 preserves the nested guard initializer's parry-hitbox boundary.
// Schema 61 includes the spotlight loop's branch fallthrough before INC r4.
// Schema 62 preserves LinkOam's initial stores before equipment selection.
// Schema 63 also distinguishes the earlier pose-selected NMI checkpoint.
// Schema 64 retains the first as well as second actual-velocity pass.
// Schema 65 retires LinkOam checkpoints with their resumed source context.
// Schema 66 qualifies the new LinkOam receipt by its stair-drawing branch.
// Schema 67 recognizes Module0F's call after its speed-setting store.
// Schema 68 preserves active guard weapon-coordinate stores and temporary pose.
// Schema 69 also preserves the earlier guard head-flags boundary.
// Schema 70 carries the source body-entry/coordinate/flags and weapon-entry cursors.
// Schema 71 includes host returns before head-character and body-flags stores.
const ORIGINAL_TIMING_HOST_RECEIPT_SCHEMA: u32 = 71;

// Source instructions which sample APUI00 while waiting for an item fanfare
// to end. These adapter-only PCs become backend-neutral sample offsets before
// they cross into ZeldaState.
const SONG_END_POLL_APUI00_READ_PCS: [i32; 2] = [0x08_c400, 0x08_c609];

fn song_end_poll_native_sample_offset(
    program_counter: i32,
    port: i32,
    is_read: bool,
    output_sample: i32,
    audio_samples: usize,
) -> Option<Result<u16, String>> {
    if !is_read || port != 0 || !SONG_END_POLL_APUI00_READ_PCS.contains(&program_counter) {
        return None;
    }
    Some((|| {
        let offset = usize::try_from(output_sample).map_err(|_| {
            format!(
                "Snes9x song-end APUI00 poll at ${program_counter:06x} used negative sample offset {output_sample}",
            )
        })?;
        if offset > audio_samples {
            return Err(format!(
                "Snes9x song-end APUI00 poll at ${program_counter:06x} used sample offset {offset} beyond the {audio_samples}-sample host window",
            ));
        }
        u16::try_from(offset).map_err(|_| {
            format!("Snes9x song-end APUI00 poll sample offset {offset} exceeds the receipt width",)
        })
    })())
}

const PRESENTED_OBJ_CACHE_ABI: i32 = 1;
const PRESENTED_OBJ_CACHE_SLOT_COUNT: usize = 512;
const PRESENTED_OBJ_CACHE_PAGE_TILE_COUNT: usize = 256;
const PRESENTED_OBJ_CACHE_PIXELS_FIELD: i32 = 29;
const PRESENTED_OBJ_CACHE_VALID_FIELD: i32 = 30;
const PRESENTED_OBJ_CACHE_WORD_ADDRESS_FIELD: i32 = 45;
const PRESENTED_OBJ_CACHE_META_FIELD: i32 = 46;

#[derive(Clone, Debug, PartialEq, Eq)]
struct SmpBootstrapInstructionStep {
    absolute_start_cycle: u64,
    absolute_end_cycle: u64,
    origin_pc: i32,
    opcode: i32,
    boundary_opcode_cycle: i32,
    op_step_calls: i32,
    max_continuation_opcode_cycle: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct SmpBootstrapPatternInstruction {
    start_cycle_offset: u64,
    end_cycle_offset: u64,
    origin_pc: i32,
    opcode: i32,
    boundary_opcode_cycle: i32,
    op_step_calls: i32,
    max_continuation_opcode_cycle: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct SmpBootstrapInstructionSpan {
    absolute_start_cycle: u64,
    absolute_end_cycle: u64,
    repeat_count: usize,
    repeat_cycle_stride: u64,
    instructions: Vec<SmpBootstrapPatternInstruction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct SmpBootstrapInstructionSequence {
    encoding: &'static str,
    instruction_count: usize,
    absolute_start_cycle: u64,
    absolute_end_cycle: u64,
    spans: Vec<SmpBootstrapInstructionSpan>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct SmpBootstrapDeltaSequence {
    encoding: &'static str,
    fields: Vec<&'static str>,
    record_count: usize,
    expanded_sha256: String,
    data_base64: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FramedApuPortAccess {
    frame: u32,
    access: crate::libretro_core::LibretroApuPortWrite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FramedCpuTimingTransaction {
    frame: u32,
    transaction: crate::libretro_core::LibretroCpuTimingTransaction,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct FramedSmpInstruction {
    frame: u32,
    instruction: crate::libretro_core::LibretroSmpInstruction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SmpPostHandoffAnchor {
    handoff_cycle: u64,
    final_cpu_access: FramedApuPortAccess,
    final_cpu_timing_transaction: FramedCpuTimingTransaction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FirstNmiApuAnchor {
    access: FramedApuPortAccess,
    completed_timing_transaction: FramedCpuTimingTransaction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Snes9xRetroRunTrace {
    entry: serde_json::Value,
    return_event: serde_json::Value,
    hdma_events: Vec<serde_json::Value>,
    video_events: Vec<serde_json::Value>,
    raw_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OrdinalApuPortAccess {
    cpu_transaction_ordinal: usize,
    access: crate::libretro_core::LibretroApuPortWrite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OrdinalSmpOutputPortWrite {
    cpu_transaction_ordinal: usize,
    write: crate::libretro_core::LibretroSmpOutputPortWrite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
struct CpuTimingGapReceipt {
    previous_transaction_ordinal: usize,
    next_transaction_ordinal: usize,
    previous_end_v_counter: i32,
    previous_end_cpu_cycle: i32,
    next_start_v_counter: i32,
    next_start_cpu_cycle: i32,
    elapsed_master_cycles: i64,
}

const FIRST_NMI_RETURN_RECEIPT_FRAME: u32 = 81;
const FIRST_NMI_RETURN_HOST_FRAME: u32 = 81;
const FIRST_NMI_RETURN_RETRO_RUN: u32 = 81;
const FIRST_NMI_RETURN_START_PC: i32 = 0x008a38;

struct PendingFirstNmiReturnFixture {
    final_path: PathBuf,
    temporary_path: PathBuf,
    writer: BufWriter<fs::File>,
    installed: bool,
}

impl PendingFirstNmiReturnFixture {
    fn create(path: impl AsRef<Path>) -> Result<Self, String> {
        let final_path = path.as_ref().to_path_buf();
        let temporary_path =
            PathBuf::from(format!("{}.tmp-{}", final_path.display(), process::id()));
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)
            .map_err(|error| {
                format!(
                    "failed to create temporary first-NMI return fixture {}: {error}",
                    temporary_path.display()
                )
            })?;
        Ok(Self {
            final_path,
            temporary_path,
            writer: BufWriter::new(file),
            installed: false,
        })
    }

    fn install(&mut self) -> Result<(), String> {
        self.writer.flush().map_err(|error| {
            format!(
                "failed to flush temporary first-NMI return fixture {}: {error}",
                self.temporary_path.display()
            )
        })?;
        self.writer.get_ref().sync_all().map_err(|error| {
            format!(
                "failed to sync temporary first-NMI return fixture {}: {error}",
                self.temporary_path.display()
            )
        })?;
        fs::rename(&self.temporary_path, &self.final_path).map_err(|error| {
            format!(
                "failed to atomically install first-NMI return fixture {}: {error}",
                self.final_path.display()
            )
        })?;
        self.installed = true;
        Ok(())
    }
}

const DMA_LEDGER_FIELDS: [&str; crate::libretro_core::LIBRETRO_DMA_LEDGER_FIELDS] = [
    "kind",
    "outer",
    "owner",
    "global_byte_ordinal",
    "channel_byte_ordinal",
    "reverse",
    "a_bus_address",
    "b_bus_address",
    "value",
    "completed",
    "before_v_counter",
    "before_cpu_cycle",
    "before_next_event",
    "before_which_event",
    "before_h_max",
    "before_wram_refresh_position",
    "before_cpu_model_identity",
    "before_cpu_model_5a22",
    "before_apu_reference_time",
    "before_apu_remainder",
    "before_smp_clock",
    "before_smp_pc",
    "before_smp_opcode",
    "before_smp_opcode_cycle",
    "before_dsp_clock",
    "before_dsp_phase",
    "before_output_sample",
    "before_cpu_pbpc",
    "after_v_counter",
    "after_cpu_cycle",
    "after_next_event",
    "after_which_event",
    "after_h_max",
    "after_wram_refresh_position",
    "after_cpu_model_identity",
    "after_cpu_model_5a22",
    "after_apu_reference_time",
    "after_apu_remainder",
    "after_smp_clock",
    "after_smp_pc",
    "after_smp_opcode",
    "after_smp_opcode_cycle",
    "after_dsp_clock",
    "after_dsp_phase",
    "after_output_sample",
    "after_cpu_pbpc",
    "transfer_mode",
    "a_address_fixed",
    "a_address_decrement",
    "before_transfer_bytes",
    "before_a_address",
    "a_bank",
    "base_b_address",
    "before_vma_address",
    "vma_increment",
    "vma_high",
    "vma_full_graphic_count",
    "before_oam_address",
    "before_cgram_address",
    "before_cgram_flip",
    "before_in_wram_dma_or_hdma",
    "before_hdma_ran_in_dma",
    "before_open_bus",
    "after_transfer_bytes",
    "after_a_address",
    "after_vma_address",
    "after_oam_address",
    "after_cgram_address",
    "after_cgram_flip",
    "after_in_wram_dma_or_hdma",
    "after_hdma_ran_in_dma",
    "after_open_bus",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReplayBundle {
    dir: PathBuf,
    input_script: PathBuf,
    rom_random_script: PathBuf,
    initial_sram: PathBuf,
    frames_completed: u32,
    manifest_sha256: String,
    input_sha256: String,
    rom_random_sha256: String,
    initial_sram_sha256: String,
}

#[derive(Debug, Deserialize)]
struct ReplayBundleManifest {
    schema: u32,
    frames_completed: Option<u32>,
    rom: ReplayBundleRom,
    #[serde(default)]
    rom_random_replay: Option<ReplayBundleArtifact>,
}

#[derive(Debug, Deserialize)]
struct ReplayBundleRom {
    sha256: String,
}

#[derive(Debug, Deserialize)]
struct ReplayBundleArtifact {
    #[serde(default)]
    sha256: Option<String>,
}

fn resolve_replay_bundle(dir: &Path, frames: u32, rom_path: &Path) -> Result<ReplayBundle, String> {
    let manifest_path = dir.join("manifest.json");
    let manifest_bytes = fs::read(&manifest_path)
        .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
    let manifest: ReplayBundleManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("failed to parse {}: {error}", manifest_path.display()))?;
    if manifest.schema != 1 {
        return Err(format!(
            "{} has unsupported replay-bundle schema {}",
            manifest_path.display(),
            manifest.schema
        ));
    }
    let frames_completed = manifest.frames_completed.ok_or_else(|| {
        format!(
            "{} has no frames_completed receipt; the replay bundle is incomplete or still running",
            manifest_path.display()
        )
    })?;
    if frames > frames_completed {
        return Err(format!(
            "replay bundle {} is proven through frame {frames_completed}, but this run requests frame {frames}; choose a bundle with sufficient coverage",
            dir.display()
        ));
    }

    let actual_rom_sha256 = parity::runner::sha256_file(rom_path)
        .map_err(|error| format!("failed to hash ROM {}: {error}", rom_path.display()))?;
    if !manifest.rom.sha256.eq_ignore_ascii_case(&actual_rom_sha256) {
        return Err(format!(
            "replay bundle {} belongs to ROM {}, but {} hashes to {}",
            dir.display(),
            manifest.rom.sha256,
            rom_path.display(),
            actual_rom_sha256
        ));
    }

    let required_file = |name: &str| -> Result<PathBuf, String> {
        let path = dir.join(name);
        if !path.is_file() {
            return Err(format!(
                "replay bundle {} is missing required artifact {name}",
                dir.display()
            ));
        }
        Ok(path)
    };
    let input_script = required_file("input.txt")?;
    let rom_random_script = required_file("rom-random.txt")?;
    let initial_sram = required_file("initial.srm")?;
    let rom_random_sha256 = parity::runner::sha256_file(&rom_random_script).map_err(|error| {
        format!(
            "failed to hash replay bundle artifact {}: {error}",
            rom_random_script.display()
        )
    })?;
    if let Some(expected) = manifest
        .rom_random_replay
        .as_ref()
        .and_then(|artifact| artifact.sha256.as_deref())
    {
        if !expected.eq_ignore_ascii_case(&rom_random_sha256) {
            return Err(format!(
                "replay bundle {} has a modified rom-random.txt: manifest says {expected}, file hashes to {rom_random_sha256}",
                dir.display()
            ));
        }
    }

    Ok(ReplayBundle {
        dir: dir.to_path_buf(),
        input_sha256: parity::runner::sha256_file(&input_script).map_err(|error| {
            format!(
                "failed to hash replay bundle artifact {}: {error}",
                input_script.display()
            )
        })?,
        initial_sram_sha256: parity::runner::sha256_file(&initial_sram).map_err(|error| {
            format!(
                "failed to hash replay bundle artifact {}: {error}",
                initial_sram.display()
            )
        })?,
        manifest_sha256: parity::runner::sha256_file(&manifest_path).map_err(|error| {
            format!(
                "failed to hash replay bundle manifest {}: {error}",
                manifest_path.display()
            )
        })?,
        rom_random_sha256,
        input_script,
        rom_random_script,
        initial_sram,
        frames_completed,
    })
}

fn validate_replay_source_parents(
    sources: &[(&str, Option<&Path>)],
    allow_mixed: bool,
) -> Result<(), String> {
    if allow_mixed {
        return Ok(());
    }
    let mut first = None::<(&str, PathBuf)>;
    for (name, path) in sources {
        let Some(path) = path else {
            continue;
        };
        let canonical = fs::canonicalize(path)
            .map_err(|error| format!("failed to resolve {name} {}: {error}", path.display()))?;
        let parent = canonical.parent().ok_or_else(|| {
            format!(
                "cannot determine replay provenance directory for {name} {}",
                path.display()
            )
        })?;
        if let Some((first_name, first_parent)) = first.as_ref() {
            if parent != first_parent {
                return Err(format!(
                    "mixed replay provenance is unsafe: {first_name} comes from {}, but {name} comes from {}; use --replay-bundle <dir>, or pass --allow-mixed-replay-provenance only for an intentional diagnostic",
                    first_parent.display(),
                    parent.display()
                ));
            }
        } else {
            first = Some((name, parent.to_path_buf()));
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct OracleRngTraceEvent {
    run: u32,
    pc: u64,
    value: u8,
    carry: u8,
}

#[derive(Debug, Deserialize)]
struct OracleTraceEventKind {
    event: String,
}

fn oracle_rng_sample_from_trace_line(
    line: &str,
    expected_trace_run: u32,
    execution_frame: u32,
) -> Result<Option<RomRandomSample>, String> {
    let kind: OracleTraceEventKind = serde_json::from_str(line)
        .map_err(|error| format!("invalid live oracle trace event: {error}"))?;
    if kind.event != "rng-write" {
        return Ok(None);
    }
    let event: OracleRngTraceEvent = serde_json::from_str(line)
        .map_err(|error| format!("invalid live oracle RNG trace event: {error}"))?;
    if event.pc & 0xffff != CARTRIDGE_RNG_STORE_PC_LOW16 {
        return Ok(None);
    }
    if event.run != expected_trace_run {
        return Err(format!(
            "live oracle RNG trace run {} arrived while expecting trace run {expected_trace_run} for Rust execution frame {execution_frame}",
            event.run,
        ));
    }
    if event.carry > 1 {
        return Err(format!(
            "live oracle RNG trace run {expected_trace_run} for execution frame {execution_frame} has invalid carry {}",
            event.carry
        ));
    }
    Ok(Some(RomRandomSample::with_carry(
        execution_frame,
        event.value,
        event.carry != 0,
    )))
}

fn trace_events_with_rom_rng(configured: Option<&str>) -> String {
    let mut events = configured
        .into_iter()
        .flat_map(|events| events.split(','))
        .map(str::trim)
        .filter(|event| !event.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if !events.iter().any(|event| event == "rom-rng") {
        events.push("rom-rng".to_owned());
    }
    events.join(",")
}

struct LiveOracleRngTrace {
    path: PathBuf,
    reader: Option<BufReader<fs::File>>,
    line: String,
}

impl LiveOracleRngTrace {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            reader: None,
            line: String::new(),
        }
    }

    fn samples_for_run(
        &mut self,
        trace_run: u32,
        execution_frame: u32,
    ) -> Result<Vec<RomRandomSample>, String> {
        if self.reader.is_none() {
            match fs::File::open(&self.path) {
                Ok(file) => self.reader = Some(BufReader::new(file)),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return Ok(Vec::new());
                }
                Err(error) => {
                    return Err(format!(
                        "failed to open live oracle RNG trace {}: {error}",
                        self.path.display()
                    ));
                }
            }
        }
        let reader = self.reader.as_mut().expect("reader initialized above");
        let mut samples = Vec::new();
        loop {
            self.line.clear();
            let bytes = reader.read_line(&mut self.line).map_err(|error| {
                format!(
                    "failed to read live oracle RNG trace {}: {error}",
                    self.path.display()
                )
            })?;
            if bytes == 0 {
                break;
            }
            if let Some(sample) =
                oracle_rng_sample_from_trace_line(&self.line, trace_run, execution_frame)?
            {
                samples.push(sample);
            }
        }
        Ok(samples)
    }
}

fn validate_oracle_rng_samples_for_run(
    expected: &[RomRandomSample],
    cursor: &mut usize,
    run: u32,
    actual: &[RomRandomSample],
) -> Result<(), String> {
    if expected
        .get(*cursor)
        .is_some_and(|sample| sample.execution_frame < run)
    {
        return Err(format!(
            "RNG script expected an unobserved cartridge call at frame {} before source run {run}",
            expected[*cursor].execution_frame
        ));
    }
    let start = *cursor;
    while expected
        .get(*cursor)
        .is_some_and(|sample| sample.execution_frame == run)
    {
        *cursor += 1;
    }
    let expected_run = &expected[start..*cursor];
    if expected_run == actual {
        return Ok(());
    }
    Err(format!(
        "RNG script/source mismatch at frame {run}: script={expected_run:?}, source={actual:?}"
    ))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PairedResumeCapture {
    frame: u32,
    dir: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RollingPairedResumeCapture {
    interval: u32,
    root: PathBuf,
}

const ROLLING_PAIRED_RESUME_GENERATIONS_KEPT: usize = 2;

fn parse_paired_resume_capture(frame: &str, dir: &str) -> Result<PairedResumeCapture, String> {
    let frame = frame
        .parse()
        .map_err(|error| format!("invalid paired-resume frame `{frame}`: {error}"))?;
    Ok(PairedResumeCapture {
        frame,
        dir: PathBuf::from(dir),
    })
}

fn parse_rolling_paired_resume_capture(
    interval: &str,
    root: &str,
) -> Result<RollingPairedResumeCapture, String> {
    let interval = interval
        .parse()
        .map_err(|error| format!("invalid rolling paired-resume interval `{interval}`: {error}"))?;
    if interval == 0 {
        return Err("rolling paired-resume interval must be greater than zero".to_string());
    }
    Ok(RollingPairedResumeCapture {
        interval,
        root: PathBuf::from(root),
    })
}

fn rolling_capture_frame_after(frame: u32, interval: u32) -> u32 {
    debug_assert_ne!(interval, 0);
    frame
        .checked_div(interval)
        .expect("rolling paired-resume interval is validated")
        .saturating_add(1)
        .saturating_mul(interval)
}

const fn semantic_trace_authority_available(
    trace_configured: bool,
    generic_trace_api_exported: bool,
) -> bool {
    trace_configured && generic_trace_api_exported
}

const PAIRED_RESUME_SCHEMA: u32 = 2;

#[derive(Debug, Deserialize)]
struct PairedResumeManifest {
    schema: u32,
    boundary: String,
    frame: u32,
    rust_state: PairedResumeArtifact,
    oracle_state: PairedResumeArtifact,
    original_timing_resume_checkpoint: PairedResumeArtifact,
    semantic_trace_checkpoint: PairedResumeArtifact,
    core: PairedResumeProvenance,
    rom: PairedResumeProvenance,
    input_script: Option<PairedResumeProvenance>,
    rom_random_script: Option<PairedResumeProvenance>,
    initial_sram: PairedResumeArtifact,
}

#[derive(Debug, Deserialize)]
struct PairedResumeArtifact {
    artifact: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
struct PairedResumeProvenance {
    sha256: String,
}

#[derive(Debug, Deserialize)]
struct LatestPairedResume {
    schema: u32,
    frame: u32,
    checkpoint: String,
}

fn checkpoint_member(dir: &Path, member: &str) -> Result<PathBuf, String> {
    let mut components = Path::new(member).components();
    let Some(Component::Normal(name)) = components.next() else {
        return Err(format!("invalid paired-resume path member `{member}`"));
    };
    if components.next().is_some() {
        return Err(format!("invalid paired-resume path member `{member}`"));
    }
    Ok(dir.join(name))
}

fn resolve_paired_resume_dir(path: &Path) -> Result<(PathBuf, Option<u32>), String> {
    if path.join("manifest.json").is_file() {
        return Ok((path.to_path_buf(), None));
    }
    let latest_path = path.join("latest.json");
    let latest: LatestPairedResume = serde_json::from_slice(
        &fs::read(&latest_path)
            .map_err(|error| format!("failed to read {}: {error}", latest_path.display()))?,
    )
    .map_err(|error| format!("failed to parse {}: {error}", latest_path.display()))?;
    if latest.schema != PAIRED_RESUME_SCHEMA {
        return Err(format!(
            "{} has unsupported paired-resume schema {}; schema {} is required",
            latest_path.display(),
            latest.schema,
            PAIRED_RESUME_SCHEMA,
        ));
    }
    let checkpoint = checkpoint_member(path, &latest.checkpoint)?;
    Ok((checkpoint, Some(latest.frame)))
}

fn paired_resume_paths(path: &Path) -> Result<(PathBuf, PathBuf, PathBuf, PathBuf), String> {
    let (dir, expected_frame) = resolve_paired_resume_dir(path)?;
    let manifest_path = dir.join("manifest.json");
    let manifest_bytes = fs::read(&manifest_path)
        .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
    let manifest_value: serde_json::Value = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("failed to parse {}: {error}", manifest_path.display()))?;
    let schema = manifest_value
        .get("schema")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| format!("{} has no paired-resume schema", manifest_path.display()))?;
    if schema != u64::from(PAIRED_RESUME_SCHEMA) {
        return Err(format!(
            "{} has unsupported paired-resume schema {schema}; schema {} is required",
            manifest_path.display(),
            PAIRED_RESUME_SCHEMA,
        ));
    }
    let manifest: PairedResumeManifest = serde_json::from_value(manifest_value)
        .map_err(|error| format!("failed to parse {}: {error}", manifest_path.display()))?;
    debug_assert_eq!(manifest.schema, PAIRED_RESUME_SCHEMA);
    if manifest.boundary != "pre-frame" {
        return Err(format!(
            "{} is not a supported pre-frame paired resume",
            manifest_path.display()
        ));
    }
    if let Some(expected_frame) = expected_frame {
        if expected_frame != manifest.frame {
            return Err(format!(
                "{} points to frame {expected_frame}, but its checkpoint records frame {}",
                path.join("latest.json").display(),
                manifest.frame
            ));
        }
    }
    let verify_artifact =
        |label: &str, artifact: &PairedResumeArtifact| -> Result<PathBuf, String> {
            let artifact_path = checkpoint_member(&dir, &artifact.artifact)?;
            if !artifact_path.is_file() {
                return Err(format!(
                    "{} paired-resume {label} artifact is missing",
                    artifact_path.display()
                ));
            }
            let actual_sha256 = parity::evidence::sha256_file(&artifact_path)
                .map_err(|error| format!("failed to hash {}: {error}", artifact_path.display()))?;
            if actual_sha256 != artifact.sha256 {
                return Err(format!(
                    "{} paired-resume {label} hash mismatch: expected {}, got {}",
                    artifact_path.display(),
                    artifact.sha256,
                    actual_sha256,
                ));
            }
            Ok(artifact_path)
        };
    let rust_state = verify_artifact("Rust state", &manifest.rust_state)?;
    let oracle_state = verify_artifact("oracle state", &manifest.oracle_state)?;
    let original_timing_resume = verify_artifact(
        "original-timing checkpoint",
        &manifest.original_timing_resume_checkpoint,
    )?;
    let semantic_trace = verify_artifact(
        "semantic-trace checkpoint",
        &manifest.semantic_trace_checkpoint,
    )?;
    verify_artifact("initial SRAM", &manifest.initial_sram)?;
    let rust_checkpoint = load_play_crash_checkpoint(&rust_state)
        .map_err(|error| format!("failed to validate {}: {error}", rust_state.display()))?;
    if rust_checkpoint.host_frame != manifest.frame {
        return Err(format!(
            "{} records frame {}, but its Rust checkpoint records frame {}",
            manifest_path.display(),
            manifest.frame,
            rust_checkpoint.host_frame,
        ));
    }
    Ok((
        rust_state,
        oracle_state,
        original_timing_resume,
        semantic_trace,
    ))
}

fn validate_paired_resume_provenance(
    path: &Path,
    core: &Path,
    rom: &Path,
    input_script: Option<&Path>,
    rom_random_script: Option<&Path>,
    allow_mixed_replay_provenance: bool,
) -> Result<(), String> {
    let (dir, _) = resolve_paired_resume_dir(path)?;
    let manifest_path = dir.join("manifest.json");
    let manifest: PairedResumeManifest = serde_json::from_slice(
        &fs::read(&manifest_path)
            .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?,
    )
    .map_err(|error| format!("failed to parse {}: {error}", manifest_path.display()))?;
    if manifest.schema != PAIRED_RESUME_SCHEMA {
        return Err(format!(
            "{} has unsupported paired-resume schema {}; schema {} is required",
            manifest_path.display(),
            manifest.schema,
            PAIRED_RESUME_SCHEMA,
        ));
    }

    let verify_required =
        |label: &str, selected: &Path, expected: &PairedResumeProvenance| -> Result<(), String> {
            let actual = parity::evidence::sha256_file(selected).map_err(|error| {
                format!(
                    "failed to hash selected {label} {}: {error}",
                    selected.display()
                )
            })?;
            if actual != expected.sha256 && !allow_mixed_replay_provenance {
                return Err(format!(
                    "paired-resume {label} provenance mismatch: manifest={}, selected={actual}",
                    expected.sha256,
                ));
            }
            Ok(())
        };
    let verify_optional = |label: &str,
                           selected: Option<&Path>,
                           expected: Option<&PairedResumeProvenance>|
     -> Result<(), String> {
        match (selected, expected) {
            (None, None) => Ok(()),
            (Some(selected), Some(expected)) => verify_required(label, selected, expected),
            (Some(_), None) => Err(format!(
                "paired-resume {label} provenance is absent, but a source was selected"
            )),
            (None, Some(_)) => Err(format!(
                "paired-resume {label} provenance requires the recorded source"
            )),
        }
    };

    verify_required("core", core, &manifest.core)?;
    verify_required("ROM", rom, &manifest.rom)?;
    verify_optional("input script", input_script, manifest.input_script.as_ref())?;
    verify_optional(
        "ROM-random script",
        rom_random_script,
        manifest.rom_random_script.as_ref(),
    )?;
    Ok(())
}

fn validate_paired_resume_sram_selection(
    resume_paired: bool,
    load_sram: bool,
) -> Result<(), String> {
    if resume_paired && load_sram {
        return Err(
            "--resume-paired is self-contained and cannot be combined with --load-sram; the paired initial.srm is provenance, not a progressed-state replacement"
                .to_string(),
        );
    }
    Ok(())
}

fn restore_original_timing_resume_checkpoint(
    game: &mut ZeldaState,
    path: &Path,
) -> Result<(), String> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let checkpoint: zelda3::OriginalTimingResumeCheckpoint = serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to decode {}: {error}", path.display()))?;
    game.restore_original_timing_resume_checkpoint(checkpoint)
        .map_err(|error| format!("failed to restore {}: {error}", path.display()))
}

/// Display-domain receipt captured from the Snes9x PPU and the immutable Rust
/// scanout snapshot. This is deliberately upstream of RGBA comparison: it
/// tells us whether a failure is a ROM/state-publication issue or a renderer
/// issue before any pixel-level investigation begins.
#[derive(Debug, Serialize)]
pub(crate) struct DisplayOracleReceipt {
    frame: u32,
    stage: &'static str,
    oracle: DisplayPpuProbe,
    rust: DisplayPpuProbe,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    rust_candidates: Vec<DisplayPublicationCandidateProbe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rust_context: Option<DisplayPublicationContextProbe>,
}

#[derive(Debug, Serialize)]
struct DisplayPublicationCandidateProbe {
    name: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    cgram: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cgram_difference: Option<ValueDomainDiff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presented_oam: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presented_oam_difference: Option<ValueDomainDiff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presented_obj_tile_cache: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presented_obj_tile_cache_valid_difference: Option<ValueDomainDiff>,
}

#[derive(Debug, Serialize)]
struct DisplayPublicationContextProbe {
    render_host_frame: u32,
    publication_host_frame: u32,
    entry_frame: [u8; 4],
    following_frame: [u8; 4],
    dungeon_room: u8,
    staircase_index: u8,
    palette_filter_countdown: u16,
    nmi_update_latch: u8,
    oam_scanout_source: String,
    retain_captured_oam: bool,
    link_obj_scanout_generation: String,
    link_obj_source_generation: String,
    captured_to_host_oam_mismatches: usize,
    oam_dma_completed_after_active_scanout: bool,
}

#[derive(Debug, Serialize)]
struct DisplayPpuProbe {
    capture_source: &'static str,
    comparison_frame: Option<u32>,
    host_frame: Option<u32>,
    mode: i32,
    brightness: i32,
    scanout_brightness_override: Option<u8>,
    forced_blank: bool,
    brightness_white: i32,
    cgram: Vec<i32>,
    fixed_color: [i32; 3],
    display_control: [i32; 6],
    bg_scroll: [i32; 8],
    /// OAM visible after the most recent DMA and the generation actually used
    /// for the completed scanout. Snes9x can advance the former before the
    /// host observes the latter, so keep both domains explicit.
    oam: Vec<i32>,
    presented_oam: Vec<i32>,
    mode7: [i32; 8],
    mode7_scanlines: Vec<[i32; 8]>,
    /// Window 1/2 left/right positions actually consumed for each completed
    /// scanline, after HDMA writes.
    window_scanlines: Vec<[i32; 4]>,
    /// Snes9x's renderer-resolved main/sub clip spans for BG1..backdrop.
    ///
    /// Each of the twelve clips contributes Count followed by six
    /// (DrawMode, Left, Right) spans. Rust does not cache this derived form,
    /// so its receipt leaves the field absent.
    presented_clip: Option<Vec<i32>>,
    /// Final Snes9x draw operands for `ZELDA3_SNES9X_TRACE_PIXEL=x,y`.
    presented_pixel: Option<Vec<i32>>,
    /// Snes9x's completed per-scanline OBJ evaluation. Raw OAM can be exact
    /// while the evaluated sprite rows still belong to an earlier PPU
    /// boundary, so preserve the derived list separately.
    presented_obj: Option<PresentedObjEvaluation>,
    /// Decoded 4bpp OBJ tile cache retained from the completed scanout. The
    /// raw VRAM image can advance before libretro returns, while this cache
    /// still contains the pixels Snes9x actually drew.
    presented_obj_tile_cache: Option<Vec<i32>>,
    presented_obj_tile_cache_valid: Option<Vec<i32>>,
}

#[derive(Debug, Serialize)]
struct PresentedObjEvaluation {
    lines: Vec<PresentedObjLine>,
    visible_tiles: Vec<i32>,
    widths: Vec<i32>,
}

#[derive(Debug, Serialize)]
struct PresentedObjLine {
    sprites: Vec<i32>,
    rows: Vec<i32>,
    tiles_remaining: i32,
    range_time_over: i32,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct ValueDomainDiff {
    rust_values: usize,
    oracle_values: usize,
    mismatched_values: usize,
    first_mismatch: Option<usize>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct VramDomainReceipt {
    rust_words: usize,
    oracle_words: usize,
    rust_sha256: String,
    oracle_sha256: String,
    mismatched_words: usize,
    first_mismatch_word: Option<usize>,
    first_rust_word: Option<u16>,
    first_oracle_word: Option<u16>,
    mismatch_ranges: Vec<[usize; 2]>,
    mismatch_ranges_truncated: bool,
    /// `[block_start_word, mismatched_words]` for every non-exact 0x100-word
    /// block. Unlike the capped exact ranges, this always covers all VRAM.
    mismatch_blocks: Vec<[usize; 2]>,
}

fn vram_domain_receipt(
    rust_words: &[u16],
    oracle_bytes: Option<&[u8]>,
) -> Option<VramDomainReceipt> {
    const MAX_MISMATCH_RANGES: usize = 16;
    let oracle_bytes = oracle_bytes?;
    let oracle_words = oracle_bytes
        .chunks_exact(2)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
        .collect::<Vec<_>>();
    let rust_bytes = rust_words
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect::<Vec<_>>();
    let first_mismatch_word = rust_words
        .iter()
        .zip(&oracle_words)
        .position(|(rust, oracle)| rust != oracle)
        .or_else(|| {
            (rust_words.len() != oracle_words.len())
                .then(|| rust_words.len().min(oracle_words.len()))
        });
    let mut mismatched_words = rust_words.len().abs_diff(oracle_words.len());
    let mut mismatch_ranges = Vec::new();
    let mut mismatch_blocks = Vec::new();
    let mut open_range = None;
    let mut mismatch_ranges_truncated = false;
    for (index, (rust, oracle)) in rust_words.iter().zip(&oracle_words).enumerate() {
        if rust != oracle {
            mismatched_words += 1;
            open_range.get_or_insert(index);
        } else if let Some(start) = open_range.take() {
            if mismatch_ranges.len() < MAX_MISMATCH_RANGES {
                mismatch_ranges.push([start, index]);
            } else {
                mismatch_ranges_truncated = true;
            }
        }
    }
    if let Some(start) = open_range {
        if mismatch_ranges.len() < MAX_MISMATCH_RANGES {
            mismatch_ranges.push([start, rust_words.len().min(oracle_words.len())]);
        } else {
            mismatch_ranges_truncated = true;
        }
    }
    if rust_words.len() != oracle_words.len() {
        let start = rust_words.len().min(oracle_words.len());
        let end = rust_words.len().max(oracle_words.len());
        if mismatch_ranges.len() < MAX_MISMATCH_RANGES {
            mismatch_ranges.push([start, end]);
        } else {
            mismatch_ranges_truncated = true;
        }
    }
    for block_start in (0..rust_words.len().max(oracle_words.len())).step_by(0x100) {
        let rust = rust_words.get(block_start..).unwrap_or_default();
        let oracle = oracle_words.get(block_start..).unwrap_or_default();
        let rust = &rust[..rust.len().min(0x100)];
        let oracle = &oracle[..oracle.len().min(0x100)];
        let mismatches = rust.iter().zip(oracle).filter(|(a, b)| a != b).count()
            + rust.len().abs_diff(oracle.len());
        if mismatches != 0 {
            mismatch_blocks.push([block_start, mismatches]);
        }
    }
    Some(VramDomainReceipt {
        rust_words: rust_words.len(),
        oracle_words: oracle_words.len(),
        rust_sha256: parity::evidence::sha256_bytes(&rust_bytes),
        oracle_sha256: parity::evidence::sha256_bytes(oracle_bytes),
        mismatched_words,
        first_mismatch_word,
        first_rust_word: first_mismatch_word.and_then(|index| rust_words.get(index).copied()),
        first_oracle_word: first_mismatch_word.and_then(|index| oracle_words.get(index).copied()),
        mismatch_ranges,
        mismatch_ranges_truncated,
        mismatch_blocks,
    })
}

impl ValueDomainDiff {
    fn is_exact(&self) -> bool {
        self.mismatched_values == 0
    }
}

fn summarize_value_domain<T: PartialEq>(rust: &[T], oracle: &[T]) -> ValueDomainDiff {
    let mismatched_shared = rust
        .iter()
        .zip(oracle)
        .filter(|(rust, oracle)| rust != oracle)
        .count();
    let first_mismatch = rust
        .iter()
        .zip(oracle)
        .position(|(rust, oracle)| rust != oracle)
        .or_else(|| (rust.len() != oracle.len()).then(|| rust.len().min(oracle.len())));
    ValueDomainDiff {
        rust_values: rust.len(),
        oracle_values: oracle.len(),
        mismatched_values: mismatched_shared + rust.len().abs_diff(oracle.len()),
        first_mismatch,
    }
}

fn summarize_presented_obj_cache(
    rust: Option<&[i32]>,
    oracle: Option<&[i32]>,
    oracle_valid: Option<&[i32]>,
) -> Option<ValueDomainDiff> {
    let (rust, oracle, oracle_valid) = (rust?, oracle?, oracle_valid?);
    let comparable_pixels = oracle_valid
        .iter()
        .take(64)
        .filter(|&&valid| valid != 0)
        .count()
        * 64;
    let mut mismatched_values = 0;
    let mut first_mismatch = None;
    for (tile, &valid) in oracle_valid.iter().take(64).enumerate() {
        if valid == 0 {
            continue;
        }
        for pixel in 0..64 {
            let index = tile * 64 + pixel;
            if rust.get(index) != oracle.get(index) {
                mismatched_values += 1;
                first_mismatch.get_or_insert(index);
            }
        }
    }
    Some(ValueDomainDiff {
        rust_values: comparable_pixels,
        oracle_values: comparable_pixels,
        mismatched_values,
        first_mismatch,
    })
}

#[derive(Debug, Serialize)]
struct DisplayOracleDifferences {
    divergent_domains: Vec<&'static str>,
    registers: ValueDomainDiff,
    cgram: ValueDomainDiff,
    live_oam: ValueDomainDiff,
    presented_oam: ValueDomainDiff,
    presented_obj_tile_cache: Option<ValueDomainDiff>,
    window1_scanlines: ValueDomainDiff,
    window2_scanlines: ValueDomainDiff,
    mode7: Option<ValueDomainDiff>,
    mode7_scanlines: Option<ValueDomainDiff>,
}

fn display_register_values(probe: &DisplayPpuProbe) -> Vec<i32> {
    let mut values = vec![probe.mode, probe.brightness, i32::from(probe.forced_blank)];
    values.extend(probe.fixed_color);
    values.extend(probe.display_control);
    values.extend(probe.bg_scroll);
    values
}

fn display_oracle_differences(receipt: &DisplayOracleReceipt) -> DisplayOracleDifferences {
    let rust_registers = display_register_values(&receipt.rust);
    let oracle_registers = display_register_values(&receipt.oracle);
    let rust_scanlines = receipt
        .rust
        .mode7_scanlines
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    let oracle_scanlines = receipt
        .oracle
        .mode7_scanlines
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    let window_scanlines = |probe: &DisplayPpuProbe, window: usize| {
        probe
            .window_scanlines
            .iter()
            .flat_map(|scanline| scanline[window * 2..window * 2 + 2].iter().copied())
            .collect::<Vec<_>>()
    };
    let rust_window1_scanlines = window_scanlines(&receipt.rust, 0);
    let oracle_window1_scanlines = window_scanlines(&receipt.oracle, 0);
    let rust_window2_scanlines = window_scanlines(&receipt.rust, 1);
    let oracle_window2_scanlines = window_scanlines(&receipt.oracle, 1);
    let registers = summarize_value_domain(&rust_registers, &oracle_registers);
    let cgram = summarize_value_domain(&receipt.rust.cgram, &receipt.oracle.cgram);
    let live_oam = summarize_value_domain(&receipt.rust.oam, &receipt.oracle.oam);
    let presented_oam =
        summarize_value_domain(&receipt.rust.presented_oam, &receipt.oracle.presented_oam);
    let presented_obj_tile_cache = summarize_presented_obj_cache(
        receipt.rust.presented_obj_tile_cache.as_deref(),
        receipt.oracle.presented_obj_tile_cache.as_deref(),
        receipt.oracle.presented_obj_tile_cache_valid.as_deref(),
    );
    let window1_scanlines =
        summarize_value_domain(&rust_window1_scanlines, &oracle_window1_scanlines);
    let window2_scanlines =
        summarize_value_domain(&rust_window2_scanlines, &oracle_window2_scanlines);
    let mode7_active = receipt.rust.mode == 7 || receipt.oracle.mode == 7;
    let mode7 =
        mode7_active.then(|| summarize_value_domain(&receipt.rust.mode7, &receipt.oracle.mode7));
    let mode7_scanlines =
        mode7_active.then(|| summarize_value_domain(&rust_scanlines, &oracle_scanlines));
    let mut divergent_domains = [
        ("registers", &registers),
        ("cgram", &cgram),
        ("live_oam", &live_oam),
        ("presented_oam", &presented_oam),
        ("window1_scanlines", &window1_scanlines),
        ("window2_scanlines", &window2_scanlines),
    ]
    .into_iter()
    .filter_map(|(name, difference)| (!difference.is_exact()).then_some(name))
    .collect::<Vec<_>>();
    if mode7
        .as_ref()
        .is_some_and(|difference| !difference.is_exact())
    {
        divergent_domains.push("mode7");
    }
    if mode7_scanlines
        .as_ref()
        .is_some_and(|difference| !difference.is_exact())
    {
        divergent_domains.push("mode7_scanlines");
    }
    if presented_obj_tile_cache
        .as_ref()
        .is_some_and(|difference| !difference.is_exact())
    {
        divergent_domains.push("presented_obj_tile_cache");
    }
    DisplayOracleDifferences {
        divergent_domains,
        registers,
        cgram,
        live_oam,
        presented_oam,
        presented_obj_tile_cache,
        window1_scanlines,
        window2_scanlines,
        mode7,
        mode7_scanlines,
    }
}

fn capture_oracle_ppu_probe(oracle: &LibretroCore) -> Option<DisplayPpuProbe> {
    oracle.debug_ppu_value(0, 0)?;
    let presented_obj_tile_cache_supported = oracle
        .debug_ppu_value(29, 0)
        .is_some_and(|value| value >= 0);
    let presented_obj = PresentedObjEvaluation {
        lines: (0..224)
            .map(|line| {
                let sprites = (0..128)
                    .map(|slot| oracle.debug_ppu_value(21, line * 128 + slot).unwrap_or(-1))
                    .take_while(|sprite| *sprite >= 0)
                    .collect::<Vec<_>>();
                let rows = (0..sprites.len())
                    .map(|slot| {
                        oracle
                            .debug_ppu_value(26, line * 128 + slot as i32)
                            .unwrap_or(-1)
                    })
                    .collect();
                PresentedObjLine {
                    sprites,
                    rows,
                    tiles_remaining: oracle.debug_ppu_value(22, line).unwrap_or(-1),
                    range_time_over: oracle.debug_ppu_value(23, line).unwrap_or(-1),
                }
            })
            .collect(),
        visible_tiles: (0..128)
            .map(|sprite| oracle.debug_ppu_value(24, sprite).unwrap_or(-1))
            .collect(),
        widths: (0..128)
            .map(|sprite| oracle.debug_ppu_value(25, sprite).unwrap_or(-1))
            .collect(),
    };
    Some(DisplayPpuProbe {
        capture_source: "snes9x_last_completed_scanout",
        comparison_frame: None,
        host_frame: None,
        mode: oracle.debug_ppu_value(0, 0)?,
        brightness: oracle.debug_ppu_value(1, 0)?,
        scanout_brightness_override: None,
        forced_blank: oracle.debug_ppu_value(7, 0)? != 0,
        brightness_white: oracle.debug_ppu_value(8, 0)?,
        cgram: (0..256)
            .map(|i| oracle.debug_ppu_value(2, i).unwrap_or(-1))
            .collect(),
        fixed_color: std::array::from_fn(|i| oracle.debug_ppu_value(4, i as i32).unwrap_or(-1)),
        display_control: std::array::from_fn(|i| {
            oracle.debug_ppu_value(16, i as i32).unwrap_or(-1)
        }),
        bg_scroll: std::array::from_fn(|i| oracle.debug_ppu_value(14, i as i32).unwrap_or(-1)),
        oam: (0..544)
            .map(|i| oracle.debug_ppu_value(15, i).unwrap_or(-1))
            .collect(),
        presented_oam: (0..544)
            .map(|i| oracle.debug_ppu_value(20, i).unwrap_or(-1))
            .collect(),
        mode7: std::array::from_fn(|i| oracle.debug_ppu_value(5, i as i32).unwrap_or(-1)),
        mode7_scanlines: (0..224)
            .map(|line| {
                std::array::from_fn(|field| {
                    oracle
                        .debug_scanline_mode7_value(line, field as i32)
                        .unwrap_or(-1)
                })
            })
            .collect(),
        window_scanlines: (0..224)
            .map(|line| {
                std::array::from_fn(|field| {
                    oracle
                        .debug_scanline_mode7_value(line, field as i32 + 8)
                        .unwrap_or(-1)
                })
            })
            .collect(),
        presented_clip: Some(
            (0..228)
                .map(|i| oracle.debug_ppu_value(27, i).unwrap_or(-1))
                .collect(),
        ),
        presented_pixel: Some(
            (0..10)
                .map(|i| oracle.debug_ppu_value(28, i).unwrap_or(-1))
                .collect(),
        ),
        presented_obj: Some(presented_obj),
        // Older trace cores return -1 for unknown probe fields. Treat that as
        // an unavailable capability, not 4096 bytes of cache divergence.
        presented_obj_tile_cache: presented_obj_tile_cache_supported.then(|| {
            (0..64 * 64)
                .map(|index| oracle.debug_ppu_value(29, index).unwrap_or(-1))
                .collect()
        }),
        presented_obj_tile_cache_valid: presented_obj_tile_cache_supported.then(|| {
            (0..64)
                .map(|index| oracle.debug_ppu_value(30, index).unwrap_or(-1))
                .collect()
        }),
    })
}

fn capture_rust_ppu_probe(
    game: &mut ZeldaState,
) -> (
    DisplayPpuProbe,
    Vec<DisplayPublicationCandidateProbe>,
    Option<DisplayPublicationContextProbe>,
) {
    let live_oam = game
        .ppu
        .oam
        .iter()
        .flat_map(|word| word.to_le_bytes().map(i32::from))
        .collect();
    game.with_display_snapshot(move |snapshot| {
        let scanlines = snapshot.ppu_scanline_windows();
        capture_rust_ppu_probe_from_presented(
            snapshot,
            &snapshot.ppu,
            &scanlines,
            live_oam,
            "rust_recomposed_display_snapshot",
            None,
            snapshot.frame_ctr_dbg,
        )
    })
}

fn capture_rendered_rust_ppu_probe(
    game: &ZeldaState,
    rendered: &crate::gpu_capture::LiveGpuFrameCapture,
) -> (
    DisplayPpuProbe,
    Vec<DisplayPublicationCandidateProbe>,
    Option<DisplayPublicationContextProbe>,
) {
    let live_oam = game
        .ppu
        .oam
        .iter()
        .flat_map(|word| word.to_le_bytes().map(i32::from))
        .collect();
    capture_rust_ppu_probe_from_presented(
        game,
        rendered.presented_ppu(),
        rendered.presented_scanlines(),
        live_oam,
        "native_window_render_capture",
        rendered.comparison_frame(),
        rendered.host_frame(),
    )
}

fn capture_rust_ppu_probe_from_presented(
    game: &ZeldaState,
    ppu: &snes::ppu::PpuState,
    scanlines: &renderer::gpu_frame::RawScanlineFrame,
    live_oam: Vec<i32>,
    capture_source: &'static str,
    comparison_frame: Option<u32>,
    host_frame: u32,
) -> (
    DisplayPpuProbe,
    Vec<DisplayPublicationCandidateProbe>,
    Option<DisplayPublicationContextProbe>,
) {
    let probe = DisplayPpuProbe {
        capture_source,
        comparison_frame,
        host_frame: Some(host_frame),
        mode: i32::from(ppu.bg_mode()),
        brightness: i32::from(ppu.brightness),
        scanout_brightness_override: ppu.scanout_brightness_override,
        forced_blank: ppu.forced_blank,
        brightness_white: i32::from(ppu.brightness_mult.get(31).copied().unwrap_or(0) >> 3),
        cgram: ppu.cgram.iter().map(|&value| i32::from(value)).collect(),
        fixed_color: [
            i32::from(ppu.fixed_color_r),
            i32::from(ppu.fixed_color_g),
            i32::from(ppu.fixed_color_b),
        ],
        display_control: [
            i32::from(ppu.screen_enabled[0]),
            i32::from(ppu.screen_enabled[1]),
            i32::from(ppu.screen_windowed[0]),
            i32::from(ppu.screen_windowed[1]),
            i32::from(
                u8::from(ppu.add_subscreen) << 1 | ppu.prevent_math_mode << 4 | ppu.clip_mode << 6,
            ),
            i32::from(
                ppu.math_enabled
                    | u8::from(ppu.half_color) << 6
                    | u8::from(ppu.subtract_color) << 7,
            ),
        ],
        bg_scroll: std::array::from_fn(|i| {
            let layer = &ppu.bg_layer[i / 2];
            i32::from(if i % 2 == 0 {
                layer.h_scroll
            } else {
                layer.v_scroll
            })
        }),
        oam: live_oam,
        presented_oam: ppu
            .oam
            .iter()
            .flat_map(|word| word.to_le_bytes().map(i32::from))
            .collect(),
        mode7: ppu.m7_matrix.map(i32::from),
        mode7_scanlines: scanlines.iter().map(|line| line.7.map(i32::from)).collect(),
        window_scanlines: scanlines
            .iter()
            .map(|line| [line.0, line.1, line.2, line.3].map(i32::from))
            .collect(),
        presented_clip: None,
        presented_pixel: None,
        presented_obj: None,
        presented_obj_tile_cache: Some(
            (0..64u16)
                .flat_map(|tile| {
                    let presented_obj_vram = ppu.obj_vram_latch.as_deref().unwrap_or(&ppu.vram);
                    renderer::modern_extract::decode_snes_4bpp_tile_indices(
                        presented_obj_vram,
                        0x4000,
                        tile,
                    )
                    .map(i32::from)
                })
                .collect(),
        ),
        presented_obj_tile_cache_valid: None,
    };
    let candidates = game
        .zelda_debug_display_publication_candidates()
        .iter()
        .map(|candidate| DisplayPublicationCandidateProbe {
            name: candidate.name,
            cgram: None,
            cgram_difference: None,
            presented_oam: candidate.oam.as_ref().map(|oam| {
                oam.iter()
                    .flat_map(|word| word.to_le_bytes().map(i32::from))
                    .collect()
            }),
            presented_oam_difference: None,
            presented_obj_tile_cache: candidate.obj_vram.as_ref().map(|obj_vram| {
                (0..64u16)
                    .flat_map(|tile| {
                        renderer::modern_extract::decode_snes_4bpp_tile_indices(obj_vram, 0, tile)
                            .map(i32::from)
                    })
                    .collect()
            }),
            presented_obj_tile_cache_valid_difference: None,
        })
        .collect::<Vec<_>>();
    let mut candidates = candidates;
    for cgram_candidate in game.zelda_debug_display_cgram_candidates() {
        let cgram = Some(
            cgram_candidate
                .cgram
                .iter()
                .copied()
                .map(i32::from)
                .collect(),
        );
        if let Some(candidate) = candidates
            .iter_mut()
            .find(|candidate| candidate.name == cgram_candidate.name)
        {
            candidate.cgram = cgram;
        } else {
            candidates.push(DisplayPublicationCandidateProbe {
                name: cgram_candidate.name,
                cgram,
                cgram_difference: None,
                presented_oam: None,
                presented_oam_difference: None,
                presented_obj_tile_cache: None,
                presented_obj_tile_cache_valid_difference: None,
            });
        }
    }
    let context = game
        .zelda_debug_display_publication_context()
        .map(|context| DisplayPublicationContextProbe {
            render_host_frame: context.render_host_frame,
            publication_host_frame: context.publication_host_frame,
            entry_frame: context.entry_frame,
            following_frame: context.following_frame,
            dungeon_room: context.dungeon_room,
            staircase_index: context.staircase_index,
            palette_filter_countdown: context.palette_filter_countdown,
            nmi_update_latch: context.nmi_update_latch,
            oam_scanout_source: context.oam_scanout_source.clone(),
            retain_captured_oam: context.retain_captured_oam,
            link_obj_scanout_generation: context.link_obj_scanout_generation.clone(),
            link_obj_source_generation: context.link_obj_source_generation.clone(),
            captured_to_host_oam_mismatches: context.captured_to_host_oam_mismatches,
            oam_dma_completed_after_active_scanout: context.oam_dma_completed_after_active_scanout,
        });
    (probe, candidates, context)
}

fn write_display_oracle_receipt(
    writer: &mut BufWriter<fs::File>,
    frame: u32,
    stage: &'static str,
    oracle: &LibretroCore,
    game: &mut ZeldaState,
) {
    let Some(oracle_ppu) = capture_oracle_ppu_probe(oracle) else {
        eprintln!("display-oracle capture requires an instrumented Snes9x core");
        process::exit(2);
    };
    let (rust, mut rust_candidates, rust_context) = capture_rust_ppu_probe(game);
    annotate_display_candidate_differences(&oracle_ppu, &mut rust_candidates);
    let receipt = DisplayOracleReceipt {
        frame,
        stage,
        oracle: oracle_ppu,
        rust,
        rust_candidates,
        rust_context,
    };
    serde_json::to_writer(&mut *writer, &receipt).unwrap_or_else(|error| {
        eprintln!("failed to write display-oracle receipt: {error}");
        process::exit(1);
    });
    writeln!(writer).unwrap_or_else(|error| {
        eprintln!("failed to terminate display-oracle receipt: {error}");
        process::exit(1);
    });
    writer.flush().unwrap_or_else(|error| {
        eprintln!("failed to flush display-oracle receipt: {error}");
        process::exit(1);
    });
}

fn annotate_display_candidate_differences(
    oracle: &DisplayPpuProbe,
    candidates: &mut [DisplayPublicationCandidateProbe],
) {
    for candidate in candidates {
        candidate.cgram_difference = candidate
            .cgram
            .as_deref()
            .map(|candidate| summarize_value_domain(candidate, &oracle.cgram));
        candidate.presented_oam_difference = candidate
            .presented_oam
            .as_deref()
            .map(|candidate| summarize_value_domain(candidate, &oracle.presented_oam));
        candidate.presented_obj_tile_cache_valid_difference = summarize_presented_obj_cache(
            candidate.presented_obj_tile_cache.as_deref(),
            oracle.presented_obj_tile_cache.as_deref(),
            oracle.presented_obj_tile_cache_valid.as_deref(),
        );
    }
}

#[derive(Debug, Serialize)]
struct HashedDomainDiff {
    rust_fnv1a32: u32,
    oracle_fnv1a32: u32,
    difference: ValueDomainDiff,
}

#[derive(Debug, Serialize)]
struct ObjStateLedgerReceipt {
    frame: u32,
    /// Semantic WRAM fields modeled by the native engine. Full WRAM remains
    /// below as an observational hash because unmodeled/scratch bytes are not
    /// a valid parity gate.
    modeled_wram_mismatches: Vec<String>,
    wram: HashedDomainDiff,
    raw_obj_vram: HashedDomainDiff,
    live_oam: HashedDomainDiff,
    presented_oam: HashedDomainDiff,
    presented_obj_tile_cache: HashedDomainDiff,
    oracle_valid_obj_tiles: usize,
}

impl ObjStateLedgerReceipt {
    fn presented_cache_is_exact(&self) -> bool {
        self.presented_obj_tile_cache.difference.is_exact()
    }
}

fn fnv1a32(values: impl IntoIterator<Item = u8>) -> u32 {
    values.into_iter().fold(2_166_136_261u32, |hash, value| {
        (hash ^ u32::from(value)).wrapping_mul(16_777_619)
    })
}

fn hashed_byte_domain(rust: &[u8], oracle: &[u8]) -> HashedDomainDiff {
    HashedDomainDiff {
        rust_fnv1a32: fnv1a32(rust.iter().copied()),
        oracle_fnv1a32: fnv1a32(oracle.iter().copied()),
        difference: summarize_value_domain(rust, oracle),
    }
}

fn hashed_i32_domain(rust: &[i32], oracle: &[i32]) -> HashedDomainDiff {
    HashedDomainDiff {
        rust_fnv1a32: fnv1a32(rust.iter().map(|&value| value as u8)),
        oracle_fnv1a32: fnv1a32(oracle.iter().map(|&value| value as u8)),
        difference: summarize_value_domain(rust, oracle),
    }
}

fn capture_obj_state_ledger_receipt(
    frame: u32,
    oracle: &LibretroCore,
    game: &mut ZeldaState,
) -> Option<ObjStateLedgerReceipt> {
    oracle.debug_ppu_value(29, 0)?;
    let oracle_wram = oracle.memory_bytes(RETRO_MEMORY_SYSTEM_RAM)?;
    let oracle_vram = oracle.memory_bytes(RETRO_MEMORY_VIDEO_RAM)?;
    let oracle_obj_vram = oracle_vram.get(0x8000..0x8800)?;
    let rust_obj_vram = game.ppu.vram[0x4000..0x4400]
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect::<Vec<_>>();
    let rust_live_oam = game
        .ppu
        .oam
        .iter()
        .flat_map(|word| word.to_le_bytes().map(i32::from))
        .collect::<Vec<_>>();
    let oracle_live_oam = (0..544)
        .map(|index| oracle.debug_ppu_value(15, index).unwrap_or(-1))
        .collect::<Vec<_>>();
    let (rust_presented_oam, rust_presented_cache) = game.with_display_snapshot(|snapshot| {
        let presented_obj_vram = snapshot
            .ppu
            .obj_vram_latch
            .as_deref()
            .unwrap_or(&snapshot.ppu.vram);
        let cache = (0..64u16)
            .flat_map(|tile| {
                renderer::modern_extract::decode_snes_4bpp_tile_indices(
                    presented_obj_vram,
                    0x4000,
                    tile,
                )
                .map(i32::from)
            })
            .collect::<Vec<_>>();
        let oam = snapshot
            .ppu
            .oam
            .iter()
            .flat_map(|word| word.to_le_bytes().map(i32::from))
            .collect::<Vec<_>>();
        (oam, cache)
    });
    let oracle_presented_oam = (0..544)
        .map(|index| oracle.debug_ppu_value(20, index).unwrap_or(-1))
        .collect::<Vec<_>>();
    let oracle_valid = (0..64)
        .map(|index| oracle.debug_ppu_value(30, index).unwrap_or(-1))
        .collect::<Vec<_>>();
    let oracle_presented_cache = (0..64 * 64)
        .map(|index| oracle.debug_ppu_value(29, index).unwrap_or(-1))
        .collect::<Vec<_>>();
    let mut rust_valid_cache = Vec::new();
    let mut oracle_valid_cache = Vec::new();
    for (tile, &valid) in oracle_valid.iter().enumerate() {
        if valid == 0 {
            continue;
        }
        let range = tile * 64..tile * 64 + 64;
        rust_valid_cache.extend_from_slice(&rust_presented_cache[range.clone()]);
        oracle_valid_cache.extend_from_slice(&oracle_presented_cache[range]);
    }
    let presented_cache_difference = summarize_presented_obj_cache(
        Some(&rust_presented_cache),
        Some(&oracle_presented_cache),
        Some(&oracle_valid),
    )?;
    Some(ObjStateLedgerReceipt {
        frame,
        modeled_wram_mismatches: compact_engine_state_mismatches(&game.ram, oracle_wram),
        wram: hashed_byte_domain(&game.ram, oracle_wram),
        raw_obj_vram: hashed_byte_domain(&rust_obj_vram, oracle_obj_vram),
        live_oam: hashed_i32_domain(&rust_live_oam, &oracle_live_oam),
        presented_oam: hashed_i32_domain(&rust_presented_oam, &oracle_presented_oam),
        presented_obj_tile_cache: HashedDomainDiff {
            rust_fnv1a32: fnv1a32(rust_valid_cache.iter().map(|&value| value as u8)),
            oracle_fnv1a32: fnv1a32(oracle_valid_cache.iter().map(|&value| value as u8)),
            difference: presented_cache_difference,
        },
        oracle_valid_obj_tiles: oracle_valid.iter().filter(|&&valid| valid != 0).count(),
    })
}

/// The ROM-visible publication flags that delimit an emulated frame.
///
/// These are intentionally a narrow semantic contract: unlike a raw WRAM
/// dump, they have stable meaning in both the ROM decompilation crosswalk and
/// the Rust state model.  They are compared before RGBA pixels so a renderer
/// cannot conceal a scheduler or producer error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct BootBoundaryState {
    frame: u32,
    stage: &'static str,
    main_module: u8,
    submodule: u8,
    nmi_latch: u8,
    inidisp: u8,
    bg_vram_load: u8,
    cgram_upload: u8,
    hud_upload: u8,
    nmi_subroutine: u8,
}

impl BootBoundaryState {
    fn from_ram(frame: u32, stage: &'static str, ram: &[u8]) -> Self {
        Self {
            frame,
            stage,
            main_module: ram[0x10],
            submodule: ram[0x11],
            nmi_latch: ram[0x12],
            inidisp: ram[0x13],
            bg_vram_load: ram[0x14],
            cgram_upload: ram[0x15],
            hud_upload: ram[0x16],
            nmi_subroutine: ram[0x17],
        }
    }

    fn first_difference(&self, oracle: &Self) -> Option<(&'static str, u8, u8)> {
        [
            ("main_module", self.main_module, oracle.main_module),
            ("submodule", self.submodule, oracle.submodule),
            ("nmi_latch", self.nmi_latch, oracle.nmi_latch),
            ("inidisp", self.inidisp, oracle.inidisp),
            ("bg_vram_load", self.bg_vram_load, oracle.bg_vram_load),
            ("cgram_upload", self.cgram_upload, oracle.cgram_upload),
            ("hud_upload", self.hud_upload, oracle.hud_upload),
            ("nmi_subroutine", self.nmi_subroutine, oracle.nmi_subroutine),
        ]
        .into_iter()
        .find_map(|(name, rust, oracle)| (rust != oracle).then_some((name, rust, oracle)))
    }
}

pub(crate) fn run_compare_snes9x_oracle(args: &[String]) {
    run_compare_libretro_oracle(args, Some("snes9x"), Some("Snes9x"));
}

fn validate_oracle_av_checkpoint_interval(interval: Option<u32>) -> Result<(), String> {
    if interval.is_some() {
        return Err(
            "oracle A/V checkpoints are unavailable: pinned Snes9x serialization mutates live DSP state; capture from reset without in-run checkpoints"
                .to_string(),
        );
    }
    Ok(())
}

fn validate_cold_evidence_invocation_id(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(
            "cold-evidence invocation ID must be 1..=128 ASCII letters, digits, '-', '_', or '.'"
                .to_string(),
        );
    }
    Ok(())
}

fn cold_evidence_run_nonce(
    session_dir: &Path,
    invocation_id: &str,
    unix_time_ns: u128,
    process_id: u32,
) -> String {
    let session_dir = fs::canonicalize(session_dir).unwrap_or_else(|_| session_dir.to_path_buf());
    parity::evidence::sha256_bytes(
        format!(
            "zelda3-cold-run-v1\0{unix_time_ns}\0{process_id}\0{}\0{invocation_id}",
            session_dir.to_string_lossy()
        )
        .as_bytes(),
    )
}

pub(crate) fn run_capture_snes9x_av(args: &[String]) {
    let (
        Some(core_path),
        Some(rom_path),
        Some(frames),
        Some(input_path),
        Some(rng_path),
        Some(sram_path),
        Some(output_dir),
        Some(expected_core_sha256),
        Some(expected_rom_sha256),
    ) = (
        args.first(),
        args.get(1),
        args.get(2),
        args.get(3),
        args.get(4),
        args.get(5),
        args.get(6),
        args.get(7),
        args.get(8),
    )
    else {
        eprintln!(
            "usage: zelda3 --capture-snes9x-av CORE ROM FRAMES INPUT RNG SRAM OUTPUT EXPECTED_CORE_SHA256 EXPECTED_ROM_SHA256 [--resume-oracle-state STATE --resume-semantic-trace-checkpoint CHECKPOINT --start-frame N] [--checkpoint-interval N]"
        );
        process::exit(2);
    };
    let mut resume_oracle_state = None;
    let mut resume_semantic_trace_checkpoint = None;
    let mut start_frame = 0;
    let mut checkpoint_interval = None;
    let mut argument = 9;
    while argument < args.len() {
        match args[argument].as_str() {
            "--resume-oracle-state" if resume_oracle_state.is_none() => {
                let Some(value) = args.get(argument + 1) else {
                    eprintln!("--resume-oracle-state requires a path");
                    process::exit(2);
                };
                resume_oracle_state = Some(PathBuf::from(value));
                argument += 2;
            }
            "--resume-semantic-trace-checkpoint" if resume_semantic_trace_checkpoint.is_none() => {
                let Some(value) = args.get(argument + 1) else {
                    eprintln!("--resume-semantic-trace-checkpoint requires a path");
                    process::exit(2);
                };
                resume_semantic_trace_checkpoint = Some(PathBuf::from(value));
                argument += 2;
            }
            "--start-frame" if start_frame == 0 => {
                let Some(value) = args.get(argument + 1) else {
                    eprintln!("--start-frame requires a frame");
                    process::exit(2);
                };
                start_frame = value.parse::<u32>().unwrap_or_else(|error| {
                    eprintln!("invalid --start-frame `{value}`: {error}");
                    process::exit(2);
                });
                argument += 2;
            }
            "--checkpoint-interval" if checkpoint_interval.is_none() => {
                let Some(value) = args.get(argument + 1) else {
                    eprintln!("--checkpoint-interval requires a nonzero frame count");
                    process::exit(2);
                };
                let interval = value.parse::<u32>().unwrap_or_else(|error| {
                    eprintln!("invalid --checkpoint-interval `{value}`: {error}");
                    process::exit(2);
                });
                if interval == 0 {
                    eprintln!("--checkpoint-interval requires a nonzero frame count");
                    process::exit(2);
                }
                checkpoint_interval = Some(interval);
                argument += 2;
            }
            option => {
                eprintln!("unknown oracle A/V capture option: {option}");
                process::exit(2);
            }
        }
    }
    validate_oracle_av_checkpoint_interval(checkpoint_interval).unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(2);
    });
    if resume_oracle_state.is_some() != resume_semantic_trace_checkpoint.is_some()
        || (resume_oracle_state.is_some() != (start_frame != 0))
    {
        eprintln!(
            "oracle A/V resume requires --resume-oracle-state, --resume-semantic-trace-checkpoint, and nonzero --start-frame together"
        );
        process::exit(2);
    }
    let frames = frames.parse::<u32>().unwrap_or_else(|error| {
        eprintln!("invalid oracle A/V capture frame count {frames}: {error}");
        process::exit(2);
    });
    if frames == 0 {
        eprintln!("oracle A/V capture frame count must be greater than zero");
        process::exit(2);
    }
    if start_frame >= frames {
        eprintln!("oracle A/V capture start frame {start_frame} must precede final frame {frames}");
        process::exit(2);
    }
    verify_expected_sha256(core_path, "libretro core", Some(expected_core_sha256));
    verify_expected_sha256(rom_path, "ROM", Some(expected_rom_sha256));
    let input_path = Path::new(input_path);
    let rng_path = Path::new(rng_path);
    let sram_path = Path::new(sram_path);
    let output_dir = Path::new(output_dir);
    if output_dir.exists()
        && fs::read_dir(output_dir)
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(true)
    {
        eprintln!(
            "oracle A/V capture output must be absent or empty: {}",
            output_dir.display()
        );
        process::exit(2);
    }
    let input_script = InputScript::from_path(input_path).unwrap_or_else(|error| {
        eprintln!(
            "failed to parse input script {}: {error}",
            input_path.display()
        );
        process::exit(2);
    });
    let input_bytes = fs::read(input_path).unwrap_or_else(|error| {
        eprintln!(
            "failed to read input script {}: {error}",
            input_path.display()
        );
        process::exit(2);
    });
    let rng_bytes = fs::read(rng_path).unwrap_or_else(|error| {
        eprintln!("failed to read RNG script {}: {error}", rng_path.display());
        process::exit(2);
    });
    // Parsing is a provenance check even though the ROM, not this harness,
    // produces random values on the oracle side.
    let rng_text = std::str::from_utf8(&rng_bytes).unwrap_or_else(|error| {
        eprintln!("RNG script {} is not UTF-8: {error}", rng_path.display());
        process::exit(2);
    });
    let rng_samples = zelda3::parse_rom_random_script(rng_text).unwrap_or_else(|error| {
        eprintln!("failed to parse RNG script {}: {error}", rng_path.display());
        process::exit(2);
    });
    let initial_sram = fs::read(sram_path).unwrap_or_else(|error| {
        eprintln!("failed to read SRAM {}: {error}", sram_path.display());
        process::exit(2);
    });
    fs::create_dir_all(output_dir).unwrap_or_else(|error| {
        eprintln!(
            "failed to create oracle A/V capture {}: {error}",
            output_dir.display()
        );
        process::exit(1);
    });
    fs::write(output_dir.join("input.txt"), &input_bytes).unwrap_or_else(|error| {
        eprintln!("failed to persist oracle capture input: {error}");
        process::exit(1);
    });
    fs::write(output_dir.join("rom-random.txt"), &rng_bytes).unwrap_or_else(|error| {
        eprintln!("failed to persist oracle capture RNG: {error}");
        process::exit(1);
    });
    fs::write(output_dir.join("initial.srm"), &initial_sram).unwrap_or_else(|error| {
        eprintln!("failed to persist oracle capture SRAM: {error}");
        process::exit(1);
    });
    let mut semantic_trace =
        Snes9xOracleSemanticTrace::configure(Some(output_dir)).unwrap_or_else(|error| {
            eprintln!("failed to configure Snes9x semantic receipt capture: {error}");
            process::exit(1);
        });
    if let Some(path) = resume_semantic_trace_checkpoint.as_deref() {
        let checkpoint: Snes9xOracleSemanticTraceCheckpoint = fs::read(path)
            .map_err(|error| format!("read {}: {error}", path.display()))
            .and_then(|bytes| {
                serde_json::from_slice(&bytes)
                    .map_err(|error| format!("decode {}: {error}", path.display()))
            })
            .unwrap_or_else(|error| {
                eprintln!("failed to read Snes9x semantic trace checkpoint: {error}");
                process::exit(2);
            });
        semantic_trace
            .restore_checkpoint(checkpoint)
            .unwrap_or_else(|error| {
                eprintln!("failed to restore Snes9x semantic trace checkpoint: {error}");
                process::exit(2);
            });
    }
    let mut oracle_rng_trace = LiveOracleRngTrace::new(semantic_trace.backing_path().to_path_buf());
    let mut rng_cursor = rng_samples.partition_point(|sample| sample.execution_frame < start_frame);
    // ZELDA3_ORACLE_CAPTURE_RECORD_RNG=1: run the pinned oracle past any
    // recorded RNG coverage and write the cartridge RNG it produces as the
    // capture's rom-random.txt (a wire/RNG recorder for frames Rust has not
    // reached yet). The result is development evidence, not a parity cache.
    let record_rng = env::var_os("ZELDA3_ORACLE_CAPTURE_RECORD_RNG").is_some_and(|v| v == "1");
    let mut recorded_rng: Vec<zelda3::RomRandomSample> = Vec::new();
    // Oracle-only capture performs no GPU work; it needs no lock.
    let _compare_lock = acquire_snes9x_compare_lock_mode(false);
    let mut oracle = LibretroCore::load_with_sram(core_path, rom_path, Some(&initial_sram))
        .unwrap_or_else(|error| {
            eprintln!("failed to initialize Snes9x for A/V capture: {error}");
            process::exit(1);
        });
    if let Some(path) = resume_oracle_state.as_deref() {
        let state = fs::read(path).unwrap_or_else(|error| {
            eprintln!(
                "failed to read paired oracle state {}: {error}",
                path.display()
            );
            process::exit(2);
        });
        oracle.unserialize_state(&state).unwrap_or_else(|error| {
            eprintln!(
                "failed to restore paired oracle state {}: {error}",
                path.display()
            );
            process::exit(2);
        });
    }
    validate_required_libretro_core(
        Some(("Snes9x", "1.63")),
        &oracle.library_name,
        &oracle.library_version,
    )
    .unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(2);
    });
    let initial_oracle_state = oracle.serialize_state().unwrap_or_else(|error| {
        eprintln!("failed to serialize initial Snes9x state: {error}");
        process::exit(1);
    });
    fs::write(
        output_dir.join("oracle_initial.state"),
        &initial_oracle_state,
    )
    .unwrap_or_else(|error| {
        eprintln!("failed to write initial Snes9x state: {error}");
        process::exit(1);
    });
    let core_sha256 = parity::evidence::sha256_file(Path::new(core_path)).unwrap();
    let rom_sha256 = parity::evidence::sha256_file(Path::new(rom_path)).unwrap();
    let manifest = serde_json::json!({
        "schema": 1,
        "status": "oracle_capture_running",
        "core": {
            "path": core_path,
            "sha256": core_sha256,
            "library_name": oracle.library_name,
            "library_version": oracle.library_version,
            "libretro_api_version": oracle.api_version,
        },
        "rom": {"path": rom_path, "sha256": rom_sha256},
        "rom_random_replay": {
            "mode": "source_trace_verified_replay_script",
            "artifact": "rom-random.txt",
            "sha256": parity::evidence::sha256_bytes(&rng_bytes),
        },
        "timing": {
            "fps": oracle.av_info.timing.fps,
            "sample_rate": oracle.av_info.timing.sample_rate,
            "frames_requested": frames,
            "start_frame": start_frame,
            "compare_from_frame": start_frame,
            "fixed_oracle_startup_skip_frames": 0,
            "dynamic_alignment": false,
        },
        "comparison_lanes": {"video": true, "audio": true},
        "resume_oracle_state": resume_oracle_state.as_ref().map(|path| serde_json::json!({
            "path": path,
            "sha256": parity::evidence::sha256_file(path).unwrap(),
            "frame": start_frame,
        })),
        "resume_semantic_trace_checkpoint": resume_semantic_trace_checkpoint.as_ref().map(|path| serde_json::json!({
            "path": path,
            "sha256": parity::evidence::sha256_file(path).unwrap(),
            "frame": start_frame,
        })),
        "av_hash_ledger": {
            "schema": 1,
            "evidence_schema": 2,
            "coverage": "every captured oracle frame",
            "video_canonicalization": "visible row-major RGB bytes; alpha and libretro row padding excluded",
            "audio_canonicalization": "interleaved stereo signed 16-bit little-endian samples",
        },
        "original_timing_host_receipts": {
            "schema": ORIGINAL_TIMING_HOST_RECEIPT_SCHEMA,
            "artifact": "original-timing-host-receipts.jsonl.zst",
            "coverage": "one backend-neutral source receipt for every captured host frame",
        },
        "artifacts": [
            "input.txt", "rom-random.txt", "initial.srm", "oracle_initial.state",
            "oracle_last_before.state", "oracle_final.state", "av_hashes.jsonl",
            "original-timing-host-receipts.jsonl.zst",
            "semantic-trace-final.checkpoint.json",
            "result.json"
        ],
    });
    fs::write(
        output_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap_or_else(|error| {
        eprintln!("failed to write oracle A/V capture manifest: {error}");
        process::exit(1);
    });
    let mut writer = BufWriter::new(
        fs::File::create(output_dir.join("av_hashes.jsonl")).unwrap_or_else(|error| {
            eprintln!("failed to create oracle A/V hash ledger: {error}");
            process::exit(1);
        }),
    );
    let receipt_file = fs::File::create(output_dir.join("original-timing-host-receipts.jsonl.zst"))
        .unwrap_or_else(|error| {
            eprintln!("failed to create source timing receipt ledger: {error}");
            process::exit(1);
        });
    let mut receipt_writer =
        zstd::stream::write::Encoder::new(receipt_file, 3).unwrap_or_else(|error| {
            eprintln!("failed to initialize source timing receipt compression: {error}");
            process::exit(1);
        });
    let mut presented_bg_tilemap_cache = PresentedBgTilemapCache::default();
    let mut previous_oracle_video = None;
    let mut oracle_last_before = initial_oracle_state;
    for frame in start_frame..frames {
        if frame.saturating_add(1) == frames {
            oracle
                .serialize_state_into(&mut oracle_last_before)
                .unwrap_or_else(|error| {
                    eprintln!("failed to serialize final pre-frame Snes9x state: {error}");
                    process::exit(1);
                });
        }
        let input = input_script.input_for_frame(frame);
        let capture = oracle.run_frame_with_input(input);
        let oracle_wram = oracle.memory_bytes(RETRO_MEMORY_SYSTEM_RAM);
        let dialogue_message_read_position = oracle_wram
            .and_then(|ram| ram.get(0x1cd9..0x1cdb))
            .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]));
        let spotlight_var4_low_at_return = oracle_wram
            .and_then(|ram| ram.get(crate::snes9x_semantic_receipts::SPOTLIGHT_VAR4_LOW_ADDRESS))
            .copied();
        let spotlight_lower_cursor_at_return = oracle_wram
            .and_then(|ram| {
                ram.get(
                    crate::snes9x_semantic_receipts::SPOTLIGHT_LOWER_CURSOR_ADDRESS
                        ..crate::snes9x_semantic_receipts::SPOTLIGHT_LOWER_CURSOR_ADDRESS + 2,
                )
            })
            .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]));
        let mut semantic = semantic_trace
            .read_after_host_call(
                dialogue_message_read_position,
                spotlight_var4_low_at_return,
                spotlight_lower_cursor_at_return,
            )
            .unwrap_or_else(|error| {
                eprintln!(
                    "failed to read Snes9x semantic receipts at capture frame {frame}: {error}"
                );
                process::exit(1);
            });
        let nmi_acceptance_ppu_register_operands =
            semantic_trace.take_host_nmi_ppu_register_operands();
        let actual_rng = oracle_rng_trace
            .samples_for_run(frame - start_frame, frame)
            .unwrap_or_else(|error| {
                eprintln!("failed to read source RNG receipts at capture frame {frame}: {error}");
                process::exit(1);
            });
        if record_rng {
            // Recorder mode: the oracle IS the RNG authority for frames no
            // recorded script covers yet; persist its samples instead of
            // validating them.
            recorded_rng.extend(actual_rng.iter().copied());
        } else {
            validate_oracle_rng_samples_for_run(&rng_samples, &mut rng_cursor, frame, &actual_rng)
                .unwrap_or_else(|error| {
                    eprintln!("oracle A/V capture rejected stale RNG provenance: {error}");
                    process::exit(2);
                });
        }
        semantic.extend(
            snes9x_oracle_semantic_receipts(&oracle).unwrap_or_else(|error| {
                eprintln!(
                    "failed to decode Snes9x semantic receipts at capture frame {frame}: {error}"
                );
                process::exit(1);
            }),
        );
        let receipts = snes9x_original_timing_host_receipts(
            &oracle,
            &capture,
            previous_oracle_video.as_ref(),
            &mut presented_bg_tilemap_cache,
            frame,
            input,
            semantic,
            nmi_acceptance_ppu_register_operands,
        )
        .unwrap_or_else(|error| {
            eprintln!("failed to decode Snes9x host receipts at capture frame {frame}: {error}");
            process::exit(1);
        });
        serde_json::to_writer(&mut receipt_writer, &receipts).unwrap_or_else(|error| {
            eprintln!("failed to write source timing receipt at frame {frame}: {error}");
            process::exit(1);
        });
        receipt_writer.write_all(b"\n").unwrap_or_else(|error| {
            eprintln!("failed to terminate source timing receipt at frame {frame}: {error}");
            process::exit(1);
        });
        previous_oracle_video = Some(PresentedOracleVideo::from(&capture));
        let video = canonical_oracle_video_digest(&capture).unwrap_or_else(|error| {
            eprintln!("failed to hash oracle video at frame {frame}: {error}");
            process::exit(1);
        });
        let audio = canonical_audio_digest(&capture.audio);
        write_av_hash_record(
            Some(&mut writer),
            frame,
            input,
            capture.audio.len() / 2,
            Some(serde_json::json!({"oracle": video})),
            Some(serde_json::json!({"oracle": audio})),
        );
    }
    writer.flush().unwrap_or_else(|error| {
        eprintln!("failed to flush oracle A/V hash ledger: {error}");
        process::exit(1);
    });
    receipt_writer.finish().unwrap_or_else(|error| {
        eprintln!("failed to finish source timing receipt compression: {error}");
        process::exit(1);
    });
    if record_rng {
        let mut text = String::from(
            "# Cartridge RNG outputs RECORDED by an oracle-only capture (ZELDA3_ORACLE_CAPTURE_RECORD_RNG=1); development evidence only.\n",
        );
        for sample in &recorded_rng {
            text.push_str(&format!(
                "{} {} carry={}\n",
                sample.execution_frame, sample.value, sample.carry as u8
            ));
        }
        fs::write(output_dir.join("rom-random.txt"), text).unwrap_or_else(|error| {
            eprintln!("failed to persist recorded oracle RNG: {error}");
            process::exit(1);
        });
        println!(
            "recorded {} cartridge RNG sample(s) into {}",
            recorded_rng.len(),
            output_dir.join("rom-random.txt").display()
        );
    }
    fs::write(
        output_dir.join("oracle_last_before.state"),
        oracle_last_before,
    )
    .unwrap_or_else(|error| {
        eprintln!("failed to write final pre-frame Snes9x state: {error}");
        process::exit(1);
    });
    let oracle_final = oracle.serialize_state().unwrap_or_else(|error| {
        eprintln!("failed to serialize final Snes9x state: {error}");
        process::exit(1);
    });
    fs::write(output_dir.join("oracle_final.state"), oracle_final).unwrap_or_else(|error| {
        eprintln!("failed to write final Snes9x state: {error}");
        process::exit(1);
    });
    fs::write(
        output_dir.join("semantic-trace-final.checkpoint.json"),
        serde_json::to_vec_pretty(&semantic_trace.checkpoint()).unwrap(),
    )
    .unwrap_or_else(|error| {
        eprintln!("failed to write final Snes9x semantic trace checkpoint: {error}");
        process::exit(1);
    });
    let result = serde_json::json!({
        "status": "oracle_captured",
        "parity_eligible": false,
        "coverage_label": "oracle-only canonical A/V capture; no Rust comparison",
        "frames_completed": frames,
        "video": {"captured": true, "matched": null},
        "audio": {"captured": true, "matched": null},
    });
    fs::write(
        output_dir.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap_or_else(|error| {
        eprintln!("failed to write oracle A/V capture result: {error}");
        process::exit(1);
    });
    semantic_trace
        .remove_backing_file()
        .unwrap_or_else(|error| {
            eprintln!("failed to discard completed raw semantic trace: {error}");
            process::exit(1);
        });
    let mut finalized_manifest = manifest;
    finalized_manifest["status"] = serde_json::json!("oracle_captured");
    finalized_manifest["frames_completed"] = serde_json::json!(frames);
    fs::write(
        output_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&finalized_manifest).unwrap(),
    )
    .unwrap_or_else(|error| {
        eprintln!("failed to finalize oracle A/V capture manifest: {error}");
        process::exit(1);
    });
    println!(
        "captured {} oracle-only Snes9x A/V frame(s) from frame {start_frame}",
        frames - start_frame
    );
}

#[derive(Debug, Deserialize, Serialize)]
struct CachedOracleAvRecord {
    schema: u32,
    frame: u32,
    input: String,
    oracle_audio_sample_frames: Option<usize>,
    video: Option<parity::av::VideoDigest>,
    audio: Option<parity::av::AudioDigest>,
}

struct PendingCachedAvFrame {
    record: CachedOracleAvRecord,
    replay_input: u16,
    sample_frames: usize,
    rust_audio: Option<serde_json::Value>,
    rust_video: Option<crate::gpu_capture::QueuedGpuVideoDigest>,
    paired_boundary: Option<(u32, Box<ZeldaState>)>,
    compare: bool,
}

fn finish_pending_cached_av_frame(
    pending: PendingCachedAvFrame,
    renderer: Option<&mut NativeWindowOracleRenderer>,
    compare_video: bool,
    compare_audio: bool,
    candidate_writer: &mut BufWriter<fs::File>,
    oracle_slice_writer: &mut BufWriter<fs::File>,
    serialization_nanos: &mut u128,
) -> Result<Option<(u32, bool, bool, Option<(u32, Box<ZeldaState>)>)>, String> {
    let rust_video = match pending.rust_video {
        Some(queued) => Some(
            renderer
                .ok_or_else(|| "queued cached video has no renderer owner".to_string())?
                .finish_game_video_digest(queued)?,
        ),
        None => None,
    };
    if !pending.compare {
        return Ok(None);
    }
    let serialization_started = Instant::now();
    let video_matches = !compare_video || rust_video.as_ref() == pending.record.video.as_ref();
    let audio_matches = !compare_audio
        || match (&pending.rust_audio, &pending.record.audio) {
            (Some(rust), Some(oracle)) => {
                rust == &serde_json::to_value(oracle)
                    .expect("serialize cached audio digest for comparison")
            }
            (None, None) => true,
            _ => false,
        };
    serde_json::to_writer(
        &mut *oracle_slice_writer,
        &serde_json::json!({
            "schema": pending.record.schema,
            "frame": pending.record.frame,
            "input": pending.record.input,
            "oracle_audio_sample_frames": pending.record.oracle_audio_sample_frames,
            "video": if compare_video { pending.record.video.as_ref() } else { None },
            "audio": if compare_audio { pending.record.audio.as_ref() } else { None },
        }),
    )
    .map_err(|error| format!("failed to write cached oracle A/V slice: {error}"))?;
    oracle_slice_writer
        .write_all(b"\n")
        .map_err(|error| format!("failed to terminate cached oracle A/V slice row: {error}"))?;
    write_av_hash_record(
        Some(candidate_writer),
        pending.record.frame,
        pending.replay_input,
        pending.sample_frames,
        rust_video.map(|rust| serde_json::json!({"rust": rust})),
        pending
            .rust_audio
            .map(|rust| serde_json::json!({"rust": rust})),
    );
    *serialization_nanos += serialization_started.elapsed().as_nanos();
    Ok(Some((
        pending.record.frame,
        video_matches,
        audio_matches,
        pending.paired_boundary,
    )))
}

fn cached_ledger_input(value: &str) -> Result<u16, String> {
    let digits = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .ok_or_else(|| format!("cached input is not hexadecimal: {value}"))?;
    u16::from_str_radix(digits, 16)
        .map_err(|error| format!("invalid cached input {value}: {error}"))
}

fn write_cached_av_paired_resume_from_sources(
    cache: &Path,
    cache_manifest: &serde_json::Value,
    cache_manifest_bytes: &[u8],
    rom: &Path,
    rom_sha256: &str,
    frame: u32,
    game: &ZeldaState,
    oracle_source: &Path,
    semantic_trace_source: &Path,
    final_dir: &Path,
) -> Result<PathBuf, String> {
    if !game.paired_resume_cpu_boundary_is_quiescent() {
        return Err(
            "cached A/V replay ended inside an unserialized ROM-call continuation".to_string(),
        );
    }
    let initial_sram_source = cache.join("initial.srm");
    let oracle_relative = oracle_source.strip_prefix(cache).map_err(|_| {
        format!(
            "oracle checkpoint {} is outside cache {}",
            oracle_source.display(),
            cache.display()
        )
    })?;
    let semantic_relative = semantic_trace_source.strip_prefix(cache).map_err(|_| {
        format!(
            "semantic checkpoint {} is outside cache {}",
            semantic_trace_source.display(),
            cache.display()
        )
    })?;
    let expected_oracle_sha256 = cache_manifest
        .get("artifact_sha256")
        .and_then(|artifacts| artifacts.get(oracle_relative.to_string_lossy().as_ref()))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            format!(
                "cached A/V manifest has no oracle-state hash for {}",
                oracle_relative.display()
            )
        })?;
    let actual_oracle_sha256 = parity::evidence::sha256_file(&oracle_source)
        .map_err(|error| format!("failed to hash {}: {error}", oracle_source.display()))?;
    if actual_oracle_sha256 != expected_oracle_sha256 {
        return Err(format!(
            "cached final oracle state hash mismatch: expected {expected_oracle_sha256}, got {actual_oracle_sha256}"
        ));
    }
    if !semantic_trace_source.is_file() {
        return Err(format!(
            "cached A/V frontier has no semantic trace checkpoint: {}",
            semantic_trace_source.display()
        ));
    }
    let expected_semantic_trace_sha256 = cache_manifest
        .get("artifact_sha256")
        .and_then(|artifacts| artifacts.get(semantic_relative.to_string_lossy().as_ref()))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            "cached semantic trace checkpoint is absent from the artifact inventory".to_string()
        })?;
    let semantic_trace_sha256 =
        parity::evidence::sha256_file(&semantic_trace_source).map_err(|error| {
            format!(
                "failed to hash {}: {error}",
                semantic_trace_source.display()
            )
        })?;
    if semantic_trace_sha256 != expected_semantic_trace_sha256 {
        return Err(format!(
            "cached semantic trace checkpoint hash mismatch: expected {expected_semantic_trace_sha256}, got {semantic_trace_sha256}"
        ));
    }
    let cache_identity = cache_manifest
        .get("cache_identity")
        .ok_or_else(|| "cached A/V manifest has no cache identity".to_string())?;
    let core_sha256 = cache_identity
        .get("core_sha256")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "cached A/V identity has no core hash".to_string())?;
    let source_artifacts = cache_identity
        .get("source_artifact_sha256")
        .ok_or_else(|| "cached A/V identity has no source artifact hashes".to_string())?;
    let input_sha256 = source_artifacts
        .get("input.txt")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "cached A/V identity has no input-script hash".to_string())?;
    let rom_random_sha256 = source_artifacts
        .get("rom-random.txt")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "cached A/V identity has no ROM-random-script hash".to_string())?;
    let initial_sram_sha256 = source_artifacts
        .get("initial.srm")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "cached A/V identity has no initial-SRAM hash".to_string())?;

    if final_dir.exists() {
        return Err(format!(
            "refusing to replace existing paired frontier {}",
            final_dir.display()
        ));
    }
    fs::create_dir_all(
        final_dir
            .parent()
            .ok_or_else(|| format!("paired frontier has no parent: {}", final_dir.display()))?,
    )
    .map_err(|error| format!("create paired frontier parent: {error}"))?;
    let temporary_dir = final_dir.with_file_name(format!(
        ".{}.tmp-{}",
        final_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("paired"),
        process::id()
    ));
    if temporary_dir.exists() {
        return Err(format!(
            "stale paired-frontier temporary directory exists: {}",
            temporary_dir.display()
        ));
    }

    let result = (|| -> Result<(), String> {
        fs::create_dir(&temporary_dir).map_err(|error| {
            format!(
                "failed to create paired frontier {}: {error}",
                temporary_dir.display()
            )
        })?;
        let original_timing_resume =
            game.capture_original_timing_resume_checkpoint()
                .map_err(|error| {
                    format!("cached Rust frontier is not an exact timing boundary: {error}")
                })?;
        let original_timing_resume_bytes = serde_json::to_vec_pretty(&original_timing_resume)
            .map_err(|error| {
                format!("failed to encode original-timing resume checkpoint: {error}")
            })?;
        let rust_state = PlayCrashCheckpoint {
            magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
            host_frame: frame,
            input: 0,
            run_what: select_run_what(&game.ram),
            game: game.clone(),
        };
        let rust_bytes = bincode::serialize(&rust_state)
            .map_err(|error| format!("failed to serialize cached Rust frontier: {error}"))?;
        fs::write(temporary_dir.join("rust.z3state"), &rust_bytes)
            .map_err(|error| format!("failed to write cached Rust frontier: {error}"))?;
        fs::write(
            temporary_dir.join("original-timing.resume.json"),
            &original_timing_resume_bytes,
        )
        .map_err(|error| format!("failed to write original-timing resume checkpoint: {error}"))?;
        fs::copy(oracle_source, temporary_dir.join("oracle.state"))
            .map_err(|error| format!("failed to copy cached final oracle state: {error}"))?;
        fs::copy(&initial_sram_source, temporary_dir.join("initial.srm"))
            .map_err(|error| format!("failed to copy cached initial SRAM: {error}"))?;
        fs::copy(
            semantic_trace_source,
            temporary_dir.join("semantic-trace.checkpoint.json"),
        )
        .map_err(|error| format!("failed to copy cached semantic trace checkpoint: {error}"))?;
        let manifest = serde_json::json!({
            "schema": PAIRED_RESUME_SCHEMA,
            "boundary": "pre-frame",
            "frame": frame,
            "cpu_boundary": "quiescent",
            "renderer_warmup_required": true,
            "rust_state": {
                "artifact": "rust.z3state",
                "sha256": parity::evidence::sha256_bytes(&rust_bytes),
            },
            "oracle_state": {
                "artifact": "oracle.state",
                "sha256": actual_oracle_sha256,
            },
            "original_timing_resume_checkpoint": {
                "artifact": "original-timing.resume.json",
                "sha256": parity::evidence::sha256_bytes(&original_timing_resume_bytes),
            },
            "semantic_trace_checkpoint": {
                "artifact": "semantic-trace.checkpoint.json",
                "sha256": semantic_trace_sha256,
            },
            "source": {
                "kind": "matched-rust-only-cached-snes9x-av-replay",
                "cache": cache,
                "cache_key": cache_manifest.get("cache_key"),
                "cache_manifest_sha256": parity::evidence::sha256_bytes(cache_manifest_bytes),
            },
            "core": {"sha256": core_sha256},
            "rom": {"path": rom, "sha256": rom_sha256},
            "input_script": {"sha256": input_sha256},
            "rom_random_script": {"sha256": rom_random_sha256},
            "initial_sram": {"artifact": "initial.srm", "sha256": initial_sram_sha256},
        });
        fs::write(
            temporary_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest)
                .map_err(|error| format!("failed to encode paired frontier manifest: {error}"))?,
        )
        .map_err(|error| format!("failed to write paired frontier manifest: {error}"))?;
        fs::rename(&temporary_dir, &final_dir).map_err(|error| {
            format!(
                "failed to install paired frontier {}: {error}",
                final_dir.display()
            )
        })?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&temporary_dir);
    }
    result?;
    Ok(final_dir.to_path_buf())
}

fn write_cached_av_final_paired_resume(
    cache: &Path,
    output: &Path,
    cache_manifest: &serde_json::Value,
    cache_manifest_bytes: &[u8],
    rom: &Path,
    rom_sha256: &str,
    frame: u32,
    game: &ZeldaState,
) -> Result<PathBuf, String> {
    let oracle_source = cache.join("oracle_final.state");
    let semantic_trace_source = cache.join("semantic-trace-final.checkpoint.json");
    let final_dir = output.join("paired-final");
    write_cached_av_paired_resume_from_sources(
        cache,
        cache_manifest,
        cache_manifest_bytes,
        rom,
        rom_sha256,
        frame,
        game,
        &oracle_source,
        &semantic_trace_source,
        &final_dir,
    )
}

fn cached_oracle_checkpoint_sources(
    cache: &Path,
    cache_manifest: &serde_json::Value,
    frame: u32,
) -> Result<(PathBuf, PathBuf), String> {
    let relative_dir = PathBuf::from(format!("oracle-checkpoints/frame-{frame:08}"));
    let checkpoint_dir = cache.join(&relative_dir);
    let manifest_path = checkpoint_dir.join("manifest.json");
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(&manifest_path)
            .map_err(|error| format!("read {}: {error}", manifest_path.display()))?,
    )
    .map_err(|error| format!("decode {}: {error}", manifest_path.display()))?;
    if manifest.get("schema").and_then(serde_json::Value::as_u64) != Some(1)
        || manifest.get("boundary").and_then(serde_json::Value::as_str) != Some("pre-frame")
        || manifest.get("frame").and_then(serde_json::Value::as_u64) != Some(u64::from(frame))
    {
        return Err(format!(
            "unsupported oracle checkpoint boundary: {}",
            manifest_path.display()
        ));
    }
    let resolve = |key: &str, expected_name: &str| -> Result<PathBuf, String> {
        let record = manifest
            .get(key)
            .ok_or_else(|| format!("oracle checkpoint has no {key}"))?;
        let artifact = record
            .get("artifact")
            .and_then(serde_json::Value::as_str)
            .filter(|value| *value == expected_name)
            .ok_or_else(|| format!("oracle checkpoint has unsafe {key} artifact"))?;
        let path = checkpoint_dir.join(artifact);
        let actual = parity::evidence::sha256_file(&path)
            .map_err(|error| format!("hash {}: {error}", path.display()))?;
        let expected = record
            .get("sha256")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("oracle checkpoint has no {key} hash"))?;
        if actual != expected {
            return Err(format!(
                "oracle checkpoint {key} hash mismatch: expected {expected}, got {actual}"
            ));
        }
        let relative = path
            .strip_prefix(cache)
            .expect("checkpoint path was constructed beneath cache")
            .to_string_lossy();
        let inventory = cache_manifest
            .get("artifact_sha256")
            .and_then(|artifacts| artifacts.get(relative.as_ref()))
            .and_then(serde_json::Value::as_str);
        if inventory != Some(actual.as_str()) {
            return Err(format!(
                "oracle checkpoint {key} is not bound by the immutable cache inventory"
            ));
        }
        Ok(path)
    };
    let oracle = resolve("oracle_state", "oracle.state")?;
    let semantic = resolve(
        "semantic_trace_checkpoint",
        "semantic-trace.checkpoint.json",
    )?;
    let cache_identity = cache_manifest
        .get("cache_identity")
        .ok_or_else(|| "cached A/V manifest has no cache identity".to_string())?;
    let provenance = manifest
        .get("provenance")
        .ok_or_else(|| "oracle checkpoint has no source provenance".to_string())?;
    for (checkpoint_key, cache_key) in
        [("core_sha256", "core_sha256"), ("rom_sha256", "rom_sha256")]
    {
        if provenance.get(checkpoint_key) != cache_identity.get(cache_key) {
            return Err(format!(
                "oracle checkpoint {checkpoint_key} does not match cache identity"
            ));
        }
    }
    let cache_sources = cache_identity
        .get("source_artifact_sha256")
        .ok_or_else(|| "cached A/V identity has no source artifacts".to_string())?;
    for (checkpoint_key, cache_name) in [
        ("input_sha256", "input.txt"),
        ("rom_random_sha256", "rom-random.txt"),
        ("initial_sram_sha256", "initial.srm"),
    ] {
        if provenance.get(checkpoint_key) != cache_sources.get(cache_name) {
            return Err(format!(
                "oracle checkpoint {checkpoint_key} does not match cache source identity"
            ));
        }
    }
    Ok((oracle, semantic))
}

fn write_cached_av_periodic_paired_resume(
    cache: &Path,
    output: &Path,
    cache_manifest: &serde_json::Value,
    cache_manifest_bytes: &[u8],
    rom: &Path,
    rom_sha256: &str,
    frame: u32,
    game: &ZeldaState,
) -> Result<PathBuf, String> {
    let (oracle, semantic) = cached_oracle_checkpoint_sources(cache, cache_manifest, frame)?;
    let paired_root = output.join("paired");
    let final_dir = paired_root.join(format!("frame-{frame:08}"));
    let checkpoint = write_cached_av_paired_resume_from_sources(
        cache,
        cache_manifest,
        cache_manifest_bytes,
        rom,
        rom_sha256,
        frame,
        game,
        &oracle,
        &semantic,
        &final_dir,
    )?;
    let latest = serde_json::json!({
        "schema": PAIRED_RESUME_SCHEMA,
        "frame": frame,
        "checkpoint": checkpoint
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("invalid paired checkpoint path: {}", checkpoint.display()))?,
    });
    let latest_path = paired_root.join("latest.json");
    let temporary = paired_root.join(format!(".latest.tmp-{}", process::id()));
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(&latest)
            .map_err(|error| format!("encode paired latest manifest: {error}"))?,
    )
    .map_err(|error| format!("write paired latest manifest: {error}"))?;
    fs::rename(&temporary, &latest_path)
        .map_err(|error| format!("install paired latest manifest: {error}"))?;
    Ok(checkpoint)
}

pub(crate) fn run_replay_cached_snes9x_av(args: &[String]) {
    let (Some(cache), Some(rom), Some(output)) = (args.first(), args.get(1), args.get(2)) else {
        eprintln!("usage: zelda3 --replay-cached-snes9x-av CACHE_DIR ROM_PATH OUTPUT_DIR [--resume-paired DIR] [--compare-from-frame N] [--frames N] [--paired-checkpoint-interval N] [--ignore-video] [--ignore-audio]");
        process::exit(2);
    };
    let mut resume_paired = None;
    let mut requested_compare_from_frame = None;
    let mut requested_frames = None;
    let mut requested_paired_checkpoint_interval = None;
    let mut ignore_video = false;
    let mut ignore_audio = false;
    let mut argument = 3;
    while argument < args.len() {
        match args[argument].as_str() {
            "--resume-paired" if resume_paired.is_none() => {
                let Some(value) = args.get(argument + 1) else {
                    eprintln!("--resume-paired requires a directory");
                    process::exit(2);
                };
                resume_paired = Some(PathBuf::from(value));
                argument += 2;
            }
            "--compare-from-frame" if requested_compare_from_frame.is_none() => {
                let Some(value) = args.get(argument + 1) else {
                    eprintln!("--compare-from-frame requires a frame");
                    process::exit(2);
                };
                requested_compare_from_frame = Some(value.parse::<u32>().unwrap_or_else(|error| {
                    eprintln!("invalid --compare-from-frame `{value}`: {error}");
                    process::exit(2);
                }));
                argument += 2;
            }
            "--frames" if requested_frames.is_none() => {
                let Some(value) = args.get(argument + 1) else {
                    eprintln!("--frames requires a nonzero frame count");
                    process::exit(2);
                };
                let frames = value.parse::<u32>().unwrap_or_else(|error| {
                    eprintln!("invalid --frames `{value}`: {error}");
                    process::exit(2);
                });
                if frames == 0 {
                    eprintln!("--frames requires a nonzero frame count");
                    process::exit(2);
                }
                requested_frames = Some(frames);
                argument += 2;
            }
            "--paired-checkpoint-interval" if requested_paired_checkpoint_interval.is_none() => {
                let Some(value) = args.get(argument + 1) else {
                    eprintln!("--paired-checkpoint-interval requires a nonzero frame count");
                    process::exit(2);
                };
                let interval = value.parse::<u32>().unwrap_or_else(|error| {
                    eprintln!("invalid --paired-checkpoint-interval `{value}`: {error}");
                    process::exit(2);
                });
                if interval == 0 {
                    eprintln!("--paired-checkpoint-interval requires a nonzero frame count");
                    process::exit(2);
                }
                requested_paired_checkpoint_interval = Some(interval);
                argument += 2;
            }
            "--ignore-video" if !ignore_video => {
                ignore_video = true;
                argument += 1;
            }
            "--ignore-audio" if !ignore_audio => {
                ignore_audio = true;
                argument += 1;
            }
            _ => {
                eprintln!("usage: zelda3 --replay-cached-snes9x-av CACHE_DIR ROM_PATH OUTPUT_DIR [--resume-paired DIR] [--compare-from-frame N] [--frames N] [--paired-checkpoint-interval N] [--ignore-video] [--ignore-audio]");
                process::exit(2);
            }
        }
    }
    let cache = Path::new(cache);
    let rom = Path::new(rom);
    let output = Path::new(output);
    parity::evidence::verify_oracle_cache_entry(cache).unwrap_or_else(|error| {
        eprintln!("cached Snes9x evidence verification failed: {error}");
        process::exit(2);
    });
    let manifest_path = cache.join("cache-manifest.json");
    let manifest_bytes = fs::read(&manifest_path).unwrap_or_else(|error| {
        eprintln!("failed to read {}: {error}", manifest_path.display());
        process::exit(2);
    });
    let manifest: serde_json::Value =
        serde_json::from_slice(&manifest_bytes).unwrap_or_else(|error| {
            eprintln!("failed to parse {}: {error}", manifest_path.display());
            process::exit(2);
        });
    let identity = manifest.get("cache_identity").unwrap_or_else(|| {
        eprintln!("{} has no cache identity", manifest_path.display());
        process::exit(2);
    });
    let source_artifacts = identity.get("source_artifact_sha256").unwrap_or_else(|| {
        eprintln!(
            "{} has no source artifact identity",
            manifest_path.display()
        );
        process::exit(2);
    });
    let timing_receipts_schema = identity
        .get("oracle_evidence")
        .and_then(|evidence| evidence.get("timing_host_receipts_schema"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    if timing_receipts_schema != u64::from(ORIGINAL_TIMING_HOST_RECEIPT_SCHEMA) {
        eprintln!(
            "cached Rust-only A/V replay requires timing-host receipt schema {}, got {timing_receipts_schema}",
            ORIGINAL_TIMING_HOST_RECEIPT_SCHEMA,
        );
        process::exit(2);
    }
    let paired_checkpoint_interval = requested_paired_checkpoint_interval.or_else(|| {
        identity
            .get("oracle_evidence")
            .and_then(|evidence| evidence.get("oracle_checkpoint_interval"))
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .filter(|value| *value != 0)
    });
    let cache_start_frame = identity
        .get("start_frame")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as u32;
    let compare_from_frame = identity
        .get("compare_from_frame")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as u32;
    if compare_from_frame != cache_start_frame {
        eprintln!(
            "cached Rust-only A/V replay requires one contiguous cache boundary; got start/compare {cache_start_frame}/{compare_from_frame}"
        );
        process::exit(2);
    }
    let expected_rom_sha256 = identity
        .get("rom_sha256")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| {
            eprintln!("{} cache identity has no ROM hash", manifest_path.display());
            process::exit(2);
        });
    let actual_rom_sha256 = parity::evidence::sha256_file(rom).unwrap_or_else(|error| {
        eprintln!("failed to hash ROM {}: {error}", rom.display());
        process::exit(2);
    });
    if actual_rom_sha256 != expected_rom_sha256 {
        eprintln!(
            "cached Rust-only A/V replay ROM mismatch: expected {expected_rom_sha256}, got {actual_rom_sha256}"
        );
        process::exit(2);
    }
    let lanes = identity.get("comparison_lanes").unwrap_or_else(|| {
        eprintln!(
            "{} cache identity has no comparison lanes",
            manifest_path.display()
        );
        process::exit(2);
    });
    let compare_video = lanes
        .get("video")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
        && !ignore_video;
    let compare_audio = lanes
        .get("audio")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
        && !ignore_audio;
    if !compare_video && !compare_audio {
        eprintln!("cached Rust-only A/V replay requires at least one enabled lane");
        process::exit(2);
    }
    let input_path = cache.join("input.txt");
    let rng_path = cache.join("rom-random.txt");
    let sram_path = cache.join("initial.srm");
    let ledger_path = cache.join("oracle-av-hashes.jsonl");
    let timing_receipts_path = cache.join("original-timing-host-receipts.jsonl.zst");
    if !timing_receipts_path.is_file() {
        eprintln!(
            "cached Rust-only A/V replay requires backend-neutral source receipts: {}",
            timing_receipts_path.display()
        );
        process::exit(2);
    }
    let input_script = InputScript::from_path(&input_path).unwrap_or_else(|error| {
        eprintln!(
            "failed to parse cached input {}: {error}",
            input_path.display()
        );
        process::exit(2);
    });
    let rng_text = fs::read_to_string(&rng_path).unwrap_or_else(|error| {
        eprintln!("failed to read cached RNG {}: {error}", rng_path.display());
        process::exit(2);
    });
    let rng_samples = zelda3::parse_rom_random_script(&rng_text).unwrap_or_else(|error| {
        eprintln!("failed to parse cached RNG {}: {error}", rng_path.display());
        process::exit(2);
    });
    let sram = fs::read(&sram_path).unwrap_or_else(|error| {
        eprintln!(
            "failed to read cached SRAM {}: {error}",
            sram_path.display()
        );
        process::exit(2);
    });
    let rom_bytes = fs::read(rom).unwrap_or_else(|error| {
        eprintln!("failed to read ROM {}: {error}", rom.display());
        process::exit(2);
    });
    let (mut game, start_frame) = if let Some(path) = resume_paired.as_deref() {
        let (rust_state, _oracle_state, original_timing_resume, _semantic_trace) =
            paired_resume_paths(path).unwrap_or_else(|error| {
                eprintln!(
                    "failed to resolve paired resume {}: {error}",
                    path.display()
                );
                process::exit(2);
            });
        let (paired_dir, _) = resolve_paired_resume_dir(path).unwrap_or_else(|error| {
            eprintln!(
                "failed to resolve paired resume {}: {error}",
                path.display()
            );
            process::exit(2);
        });
        let paired_manifest_path = paired_dir.join("manifest.json");
        let paired_manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&paired_manifest_path).unwrap_or_else(|error| {
                eprintln!("failed to read {}: {error}", paired_manifest_path.display());
                process::exit(2);
            }))
            .unwrap_or_else(|error| {
                eprintln!(
                    "failed to parse {}: {error}",
                    paired_manifest_path.display()
                );
                process::exit(2);
            });
        for (name, expected) in [
            (
                "core",
                identity
                    .get("core_sha256")
                    .and_then(serde_json::Value::as_str),
            ),
            (
                "rom",
                identity
                    .get("rom_sha256")
                    .and_then(serde_json::Value::as_str),
            ),
            (
                "input_script",
                source_artifacts
                    .get("input.txt")
                    .and_then(serde_json::Value::as_str),
            ),
            (
                "rom_random_script",
                source_artifacts
                    .get("rom-random.txt")
                    .and_then(serde_json::Value::as_str),
            ),
            (
                "initial_sram",
                source_artifacts
                    .get("initial.srm")
                    .and_then(serde_json::Value::as_str),
            ),
        ] {
            let actual = paired_manifest
                .get(name)
                .and_then(|record| record.get("sha256"))
                .and_then(serde_json::Value::as_str);
            if actual != expected {
                eprintln!(
                    "paired A/V {name} identity {:?} does not match cache {:?}",
                    actual, expected
                );
                process::exit(2);
            }
        }
        let checkpoint = load_play_crash_checkpoint(&rust_state).unwrap_or_else(|error| {
            eprintln!(
                "failed to load Rust resume state {}: {error}",
                rust_state.display()
            );
            process::exit(2);
        });
        let mut game = checkpoint.game;
        game.restore_live_rom_timing_after_checkpoint();
        restore_original_timing_resume_checkpoint(&mut game, &original_timing_resume)
            .unwrap_or_else(|error| {
                eprintln!("{error}");
                process::exit(2);
            });
        (game, checkpoint.host_frame)
    } else {
        (load_default_play_state(), 0)
    };
    let cache_end_frame = identity
        .get("frames_requested")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as u32;
    if start_frame < cache_start_frame || start_frame >= cache_end_frame {
        eprintln!(
            "cached A/V replay boundary {start_frame} is outside cache coverage {cache_start_frame}..{cache_end_frame}"
        );
        process::exit(2);
    }
    let stop_before_frame = requested_frames.map(|frames| {
        start_frame
            .checked_add(frames)
            .unwrap_or_else(|| {
                eprintln!("cached A/V requested frame range overflows u32");
                process::exit(2);
            })
            .min(cache_end_frame)
    });
    game.set_rom(&rom_bytes);
    if start_frame == 0 {
        apply_sram_to_game_or_exit(&mut game, &sram_path, &sram);
    }
    game.install_rom_random_replay(rng_samples, start_frame);
    let compare_from_frame = requested_compare_from_frame.unwrap_or(start_frame);
    if compare_from_frame < start_frame {
        eprintln!(
            "cached A/V comparison frame {compare_from_frame} precedes resumed frame {start_frame}"
        );
        process::exit(2);
    }
    let _compare_lock = acquire_snes9x_compare_lock();
    let mut renderer = compare_video.then(|| {
        NativeWindowOracleRenderer::load_from_env().unwrap_or_else(|error| {
            eprintln!("failed to initialize cached A/V GPU renderer: {error}");
            process::exit(1);
        })
    });
    fs::create_dir_all(output).unwrap_or_else(|error| {
        eprintln!(
            "failed to create cached A/V output {}: {error}",
            output.display()
        );
        process::exit(1);
    });
    let ledger_file = fs::File::open(&ledger_path).unwrap_or_else(|error| {
        eprintln!(
            "failed to open cached A/V ledger {}: {error}",
            ledger_path.display()
        );
        process::exit(2);
    });
    let timing_receipts_file = fs::File::open(&timing_receipts_path).unwrap_or_else(|error| {
        eprintln!(
            "failed to open cached source receipts {}: {error}",
            timing_receipts_path.display()
        );
        process::exit(2);
    });
    let timing_receipts_decoder = zstd::stream::read::Decoder::new(timing_receipts_file)
        .unwrap_or_else(|error| {
            eprintln!(
                "failed to decode cached source receipts {}: {error}",
                timing_receipts_path.display()
            );
            process::exit(2);
        });
    let mut timing_receipt_lines = BufReader::new(timing_receipts_decoder).lines();
    let mut candidate_writer = BufWriter::new(
        fs::File::create(output.join("av_hashes.jsonl")).unwrap_or_else(|error| {
            eprintln!("failed to create Rust A/V candidate ledger: {error}");
            process::exit(1);
        }),
    );
    let mut oracle_slice_writer = BufWriter::new(
        fs::File::create(output.join("oracle-av-hashes-slice.jsonl")).unwrap_or_else(|error| {
            eprintln!("failed to create cached oracle A/V slice: {error}");
            process::exit(1);
        }),
    );
    let mut audio_buffer = Vec::<i16>::new();
    let mut ledger_frames_seen = cache_start_frame;
    let mut frames_completed = start_frame;
    let mut frames_compared = 0_u32;
    let mut matched = true;
    let mut first_rng_drift = None;
    let mut pending_frames = VecDeque::<PendingCachedAvFrame>::new();
    let timing_enabled = env::var_os("ZELDA3_SNES9X_TIMING").is_some();
    let replay_started = Instant::now();
    let mut receipt_nanos = 0_u128;
    let mut engine_nanos = 0_u128;
    let mut audio_nanos = 0_u128;
    let mut serialization_nanos = 0_u128;
    for (line_index, line) in BufReader::new(ledger_file).lines().enumerate() {
        let receipt_started = Instant::now();
        let line = line.unwrap_or_else(|error| {
            eprintln!("failed to read {}: {error}", ledger_path.display());
            process::exit(2);
        });
        let record: CachedOracleAvRecord = serde_json::from_str(&line).unwrap_or_else(|error| {
            eprintln!(
                "invalid cached A/V record {}:{}: {error}",
                ledger_path.display(),
                line_index + 1
            );
            process::exit(2);
        });
        if record.schema != 1 || record.frame != ledger_frames_seen {
            eprintln!(
                "cached A/V ledger must be schema 1 and contiguous from cache frame {cache_start_frame}; line {} has schema {} frame {}, expected {}",
                line_index + 1,
                record.schema,
                record.frame,
                ledger_frames_seen
            );
            process::exit(2);
        }
        ledger_frames_seen = ledger_frames_seen.saturating_add(1);
        let cached_input = cached_ledger_input(&record.input).unwrap_or_else(|error| {
            eprintln!("{error}");
            process::exit(2);
        });
        let replay_input = input_script.input_for_frame(record.frame);
        if cached_input != replay_input {
            eprintln!(
                "cached input provenance mismatch at frame {}: ledger={cached_input:04x} script={replay_input:04x}",
                record.frame
            );
            process::exit(2);
        }
        let timing_receipt_line = timing_receipt_lines
            .next()
            .unwrap_or_else(|| {
                eprintln!(
                    "cached source receipt ledger ended before frame {}",
                    record.frame
                );
                process::exit(2);
            })
            .unwrap_or_else(|error| {
                eprintln!(
                    "failed to read cached source receipt at frame {}: {error}",
                    record.frame
                );
                process::exit(2);
            });
        let timing_receipts: OriginalTimingHostReceipts =
            serde_json::from_str(&timing_receipt_line).unwrap_or_else(|error| {
                eprintln!(
                    "invalid cached source receipt at frame {}: {error}",
                    record.frame
                );
                process::exit(2);
            });
        if !timing_receipts.matches_host_call(u64::from(record.frame), replay_input) {
            eprintln!(
                "cached source receipt provenance mismatch at frame {}: host_call={} canonical_input={:04x} raw_input={replay_input:04x}",
                record.frame,
                timing_receipts.host_call(),
                timing_receipts.input_state()
            );
            process::exit(2);
        }
        if record.frame < start_frame {
            continue;
        }
        if stop_before_frame.is_some_and(|end| record.frame >= end) {
            break;
        }
        if timing_enabled {
            receipt_nanos += receipt_started.elapsed().as_nanos();
        }
        let engine_started = Instant::now();
        game.install_original_timing_host_receipts(timing_receipts)
            .unwrap_or_else(|error| {
                eprintln!(
                    "failed to install cached source receipt at frame {}: {error:?}",
                    record.frame
                );
                process::exit(1);
            });
        game.zelda_run_frame(replay_input as i32);
        if timing_enabled {
            engine_nanos += engine_started.elapsed().as_nanos();
        }
        let rust_video = renderer.as_mut().map(|renderer| {
            renderer
                .queue_game_video_digest(&mut game, record.frame)
                .unwrap_or_else(|error| {
                    eprintln!(
                        "cached A/V video render submission failed at frame {}: {error}",
                        record.frame
                    );
                    process::exit(1);
                })
        });
        let audio_started = Instant::now();
        let sample_frames = record
            .oracle_audio_sample_frames
            .or_else(|| record.audio.as_ref().map(|audio| audio.sample_frames as usize))
            .unwrap_or_else(|| {
                eprintln!(
                    "cached A/V record {} has no oracle audio frame schedule; regenerate the cache with the current harness",
                    record.frame
                );
                process::exit(2);
            });
        audio_buffer.resize(sample_frames.saturating_mul(2), 0);
        game.zelda_render_audio(&mut audio_buffer, sample_frames as i32, 2);
        game.zelda_discard_unused_audio_frames();
        let rust_audio = compare_audio.then(|| canonical_audio_digest(&audio_buffer));
        if timing_enabled {
            audio_nanos += audio_started.elapsed().as_nanos();
        }
        frames_completed = frames_completed.saturating_add(1);
        if first_rng_drift.is_none() {
            if let Err(error) = game.finish_rom_random_replay_through(frames_completed) {
                eprintln!(
                    "cached A/V diagnostic: first ROM random consumption drift at execution frame {}: {error}",
                    record.frame
                );
                first_rng_drift = Some(serde_json::json!({
                    "execution_frame": record.frame,
                    "error": error,
                }));
            }
        }
        let paired_boundary = paired_checkpoint_interval
            .filter(|interval| frames_completed % interval == 0)
            .map(|_| (frames_completed, Box::new(game.clone())));
        pending_frames.push_back(PendingCachedAvFrame {
            compare: record.frame >= compare_from_frame,
            record,
            replay_input,
            sample_frames,
            rust_audio,
            rust_video,
            paired_boundary,
        });
        if pending_frames.len() == renderer::MODERN_GPU_READBACK_PIPELINE_DEPTH {
            let pending = pending_frames
                .pop_front()
                .expect("GPU readback pipeline is non-empty at capacity");
            if let Some((frame, video_matches, audio_matches, paired_boundary)) =
                finish_pending_cached_av_frame(
                    pending,
                    renderer.as_mut(),
                    compare_video,
                    compare_audio,
                    &mut candidate_writer,
                    &mut oracle_slice_writer,
                    &mut serialization_nanos,
                )
                .unwrap_or_else(|error| {
                    eprintln!("cached A/V readback failed: {error}");
                    process::exit(1);
                })
            {
                frames_compared = frames_compared.saturating_add(1);
                if !video_matches || !audio_matches {
                    eprintln!(
                        "cached Snes9x A/V divergence at frame {frame}: video_match={video_matches} audio_match={audio_matches}"
                    );
                    matched = false;
                    // Explicit diagnostic continuation keeps replaying past the
                    // first divergence. Development runs fail fast by default.
                    if std::env::var_os("ZELDA3_CACHED_AV_CONTINUE").is_none() {
                        break;
                    }
                } else if let Some((boundary, checkpoint_game)) = paired_boundary {
                    write_cached_av_periodic_paired_resume(
                        cache,
                        output,
                        &manifest,
                        &manifest_bytes,
                        rom,
                        &actual_rom_sha256,
                        boundary,
                        &checkpoint_game,
                    )
                    .unwrap_or_else(|error| {
                        eprintln!(
                            "failed to write cached A/V paired checkpoint at {boundary}: {error}"
                        );
                        process::exit(1);
                    });
                }
            }
        }
    }
    while matched || std::env::var_os("ZELDA3_CACHED_AV_CONTINUE").is_some() {
        let Some(pending) = pending_frames.pop_front() else {
            break;
        };
        if let Some((frame, video_matches, audio_matches, paired_boundary)) =
            finish_pending_cached_av_frame(
                pending,
                renderer.as_mut(),
                compare_video,
                compare_audio,
                &mut candidate_writer,
                &mut oracle_slice_writer,
                &mut serialization_nanos,
            )
            .unwrap_or_else(|error| {
                eprintln!("cached A/V readback failed: {error}");
                process::exit(1);
            })
        {
            frames_compared = frames_compared.saturating_add(1);
            if !video_matches || !audio_matches {
                eprintln!(
                    "cached Snes9x A/V divergence at frame {frame}: video_match={video_matches} audio_match={audio_matches}"
                );
                matched = false;
                if std::env::var_os("ZELDA3_CACHED_AV_CONTINUE").is_none() {
                    break;
                }
            } else if let Some((boundary, checkpoint_game)) = paired_boundary {
                write_cached_av_periodic_paired_resume(
                    cache,
                    output,
                    &manifest,
                    &manifest_bytes,
                    rom,
                    &actual_rom_sha256,
                    boundary,
                    &checkpoint_game,
                )
                .unwrap_or_else(|error| {
                    eprintln!(
                        "failed to write cached A/V paired checkpoint at {boundary}: {error}"
                    );
                    process::exit(1);
                });
            }
        }
    }
    candidate_writer.flush().unwrap_or_else(|error| {
        eprintln!("failed to flush Rust A/V candidate ledger: {error}");
        process::exit(1);
    });
    oracle_slice_writer.flush().unwrap_or_else(|error| {
        eprintln!("failed to flush cached oracle A/V slice: {error}");
        process::exit(1);
    });
    if timing_enabled {
        let total_nanos = replay_started.elapsed().as_nanos();
        let (
            capture_ms,
            gpu_render_ms,
            gpu_submit_ms,
            gpu_readback_ms,
            video_hash_ms,
            source_extract_ms,
            compositor_submit_ms,
            history_submit_ms,
            surface_present_ms,
        ) = renderer
            .as_ref()
            .map(NativeWindowOracleRenderer::timing_millis)
            .unwrap_or_default();
        let accounted_nanos = receipt_nanos
            + engine_nanos
            + audio_nanos
            + serialization_nanos
            + (capture_ms + gpu_render_ms + gpu_submit_ms + gpu_readback_ms + video_hash_ms)
                * 1_000_000;
        eprintln!(
            "cached_av_timing frames={} total_ms={} receipts_ms={} engine_ms={} video_capture_ms={} render_total_ms={} source_extract_ms={} compositor_submit_ms={} history_submit_ms={} surface_present_ms={} gpu_copy_submit_ms={} gpu_readback_ms={} video_hash_ms={} audio_hash_ms={} serialization_ms={} residual_ms={}",
            frames_completed.saturating_sub(start_frame),
            total_nanos / 1_000_000,
            receipt_nanos / 1_000_000,
            engine_nanos / 1_000_000,
            capture_ms,
            gpu_render_ms,
            source_extract_ms,
            compositor_submit_ms,
            history_submit_ms,
            surface_present_ms,
            gpu_submit_ms,
            gpu_readback_ms,
            video_hash_ms,
            audio_nanos / 1_000_000,
            serialization_nanos / 1_000_000,
            total_nanos.saturating_sub(accounted_nanos) / 1_000_000,
        );
    }
    if frames_compared == 0 {
        eprintln!("comparison frame {compare_from_frame} is not covered by the cached A/V ledger");
        process::exit(2);
    }
    if matched {
        game.finish_rom_random_replay_through(frames_completed)
            .unwrap_or_else(|error| {
                eprintln!("cached A/V ROM random replay did not complete: {error}");
                process::exit(1);
            });
    } else if let Err(error) = game.finish_rom_random_replay_through(frames_completed) {
        eprintln!(
            "cached A/V diagnostic: ROM random consumption had already drifted by the first A/V mismatch: {error}"
        );
    }
    let paired_frontier =
        (matched && compare_video && compare_audio && frames_completed == cache_end_frame).then(
            || {
                write_cached_av_final_paired_resume(
                    cache,
                    output,
                    &manifest,
                    &manifest_bytes,
                    rom,
                    &actual_rom_sha256,
                    frames_completed,
                    &game,
                )
                .unwrap_or_else(|error| {
                    eprintln!("failed to write cached A/V paired frontier: {error}");
                    process::exit(1);
                })
            },
        );
    let candidate_manifest = serde_json::json!({
        "schema": 1,
        "kind": "zelda3-rust-only-cached-snes9x-av-replay",
        "oracle_cache": cache,
        "oracle_cache_key": manifest.get("cache_key"),
        "oracle_cache_manifest_sha256": parity::evidence::sha256_bytes(&manifest_bytes),
        "rom": {"path": rom, "sha256": actual_rom_sha256},
        "start_frame": start_frame,
        "compare_from_frame": compare_from_frame,
        "frames_completed": frames_completed,
        "stop_before_frame": stop_before_frame,
        "paired_checkpoint_interval": paired_checkpoint_interval,
        "frames_compared": frames_compared,
        "resume_paired": resume_paired,
        "comparison_lanes": {"video": compare_video, "audio": compare_audio},
        "matched": matched,
        "first_rng_drift": first_rng_drift,
        "candidate_ledger": "av_hashes.jsonl",
        "paired_frontier": paired_frontier.as_ref().map(|path| path.strip_prefix(output).unwrap_or(path)),
    });
    fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(&candidate_manifest).unwrap(),
    )
    .unwrap_or_else(|error| {
        eprintln!("failed to write cached A/V candidate manifest: {error}");
        process::exit(1);
    });
    if matched {
        println!("Rust-only cached Snes9x A/V replay matched {frames_completed} frame(s)");
        if let Some(path) = paired_frontier {
            println!("paired frontier: {}", path.display());
        }
    } else {
        process::exit(1);
    }
}

pub(crate) fn replay_save_recorded_frames(path: &Path) -> Result<u32, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    if bytes.len() < 8 {
        return Err("replay save is shorter than its 8-byte version/frame header".to_string());
    }
    let version = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    if version != 1 {
        return Err(format!(
            "unsupported replay save version {version}; expected 1"
        ));
    }
    let frames = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    if frames == 0 {
        return Err("replay save contains no frames".to_string());
    }
    Ok(frames)
}

/// Validate that an unmodified Snes9x oracle can finish a recorded route.
///
/// `ZeldaState` is used only to parse the replay container's input commands.
/// It does not execute gameplay, render video, or synthesize audio in this
/// mode; every recorded controller state is fed directly to Snes9x.
pub(crate) fn run_validate_snes9x_replay(args: &[String]) {
    let (core_path, rom_path, replay_path, sram_path) = match (
        args.first(),
        args.get(1),
        args.get(2),
        args.get(3),
    ) {
        (Some(core), Some(rom), Some(replay), Some(sram)) => (
            core.as_str(),
            rom.as_str(),
            Path::new(replay),
            Path::new(sram),
        ),
        _ => {
            eprintln!(
                    "usage: zelda3 --validate-snes9x-replay <snes9x_libretro.dylib> <rom.sfc> <replay.sav> <sram.dat> [--expected-core-sha256 <sha>] [--expected-rom-sha256 <sha>]"
                );
            process::exit(2);
        }
    };
    let mut expected_core_sha256 = None::<String>;
    let mut expected_rom_sha256 = None::<String>;
    let mut i = 4usize;
    while i < args.len() {
        match args[i].as_str() {
            "--expected-core-sha256" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--expected-core-sha256 requires a hash");
                    process::exit(2);
                };
                expected_core_sha256 = Some(value.clone());
                i += 2;
            }
            "--expected-rom-sha256" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--expected-rom-sha256 requires a hash");
                    process::exit(2);
                };
                expected_rom_sha256 = Some(value.clone());
                i += 2;
            }
            flag => {
                eprintln!("unknown --validate-snes9x-replay option: {flag}");
                process::exit(2);
            }
        }
    }

    verify_expected_sha256(core_path, "libretro core", expected_core_sha256.as_deref());
    verify_expected_sha256(rom_path, "ROM", expected_rom_sha256.as_deref());
    let frames = replay_save_recorded_frames(replay_path).unwrap_or_else(|error| {
        eprintln!(
            "failed to read replay save {}: {error}",
            replay_path.display()
        );
        process::exit(2);
    });
    let replay_sha256 = parity::runner::sha256_file(replay_path).unwrap_or_else(|error| {
        eprintln!(
            "failed to hash replay save {}: {error}",
            replay_path.display()
        );
        process::exit(2);
    });
    let sram_sha256 = parity::runner::sha256_file(sram_path).unwrap_or_else(|error| {
        eprintln!("failed to hash SRAM {}: {error}", sram_path.display());
        process::exit(2);
    });
    let sram = read_file_or_exit(sram_path, "SRAM");

    let mut replay_decoder = load_play_state(rom_path);
    replay_decoder
        .replay_save_file(replay_path)
        .unwrap_or_else(|error| {
            eprintln!(
                "failed to load replay save {}: {error}",
                replay_path.display()
            );
            process::exit(2);
        });
    if replay_decoder.state_recorder.total_frames != frames {
        eprintln!(
            "replay header/parser frame count mismatch: header={frames} parser={}",
            replay_decoder.state_recorder.total_frames
        );
        process::exit(2);
    }

    let mut oracle =
        LibretroCore::load_with_sram(core_path, rom_path, Some(&sram)).unwrap_or_else(|error| {
            eprintln!("failed to initialize Snes9x libretro core: {error}");
            process::exit(1);
        });
    validate_required_libretro_core(
        Some(("Snes9x", "1.63")),
        &oracle.library_name,
        &oracle.library_version,
    )
    .unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(2);
    });
    println!(
        "validating Snes9x replay: core={} version={} frames={} replay_sha256={} sram_sha256={}",
        oracle.library_name, oracle.library_version, frames, replay_sha256, sram_sha256,
    );

    LIBRETRO_CAPTURE_ENABLED.store(false, Ordering::Relaxed);
    let mut recorder = std::mem::take(&mut replay_decoder.state_recorder);
    let mut first_credits_frame = None::<u32>;
    let mut final_credits_frame = None::<u32>;
    let mut final_state = [0u8; 3];
    let mut nonzero_input_frames = 0u32;
    let mut input_hash = 0xcbf29ce484222325u64;
    for frame in 0..frames {
        let input = replay_decoder.state_recorder_read_next_replay_state(&mut recorder);
        if input != 0 {
            nonzero_input_frames = nonzero_input_frames.saturating_add(1);
        }
        for byte in input.to_le_bytes() {
            input_hash ^= u64::from(byte);
            input_hash = input_hash.wrapping_mul(0x100000001b3);
        }
        oracle.run_frame_discard_with_input(input);
        let ram = oracle
            .memory_bytes(RETRO_MEMORY_SYSTEM_RAM)
            .unwrap_or_else(|| {
                eprintln!("Snes9x did not expose system RAM after frame {frame}");
                process::exit(1);
            });
        if ram.len() <= 0xb0 {
            eprintln!("Snes9x system RAM is too short: {} bytes", ram.len());
            process::exit(1);
        }
        final_state = [ram[0x10], ram[0x11], ram[0xb0]];
        if final_state[0] == 0x1a {
            first_credits_frame.get_or_insert(frame);
            if final_state[1] == 0x26 {
                final_credits_frame.get_or_insert(frame);
            }
        }
        let completed = frame + 1;
        if completed % 100_000 == 0 || completed == frames {
            println!(
                "Snes9x replay progress {completed}/{frames}: module={:02x}/{:02x}/{:02x}",
                final_state[0], final_state[1], final_state[2]
            );
        }
    }
    LIBRETRO_CAPTURE_ENABLED.store(true, Ordering::Relaxed);
    replay_decoder.state_recorder = recorder;

    if replay_decoder.state_recorder.replay_mode {
        eprintln!("replay input stream was not fully consumed after {frames} frames");
        process::exit(1);
    }
    let Some(first_credits_frame) = first_credits_frame else {
        eprintln!(
            "Snes9x did not reach credits module 1A; final module={:02x}/{:02x}/{:02x}",
            final_state[0], final_state[1], final_state[2]
        );
        process::exit(1);
    };
    let Some(final_credits_frame) = final_credits_frame else {
        eprintln!(
            "Snes9x entered credits at frame {first_credits_frame} but did not reach final credits state 1A/26; final module={:02x}/{:02x}/{:02x}",
            final_state[0], final_state[1], final_state[2]
        );
        process::exit(1);
    };
    println!(
        "Snes9x replay validated: consumed={frames} nonzero_input_frames={nonzero_input_frames} input_fnv64={input_hash:016x} credits_first_frame={first_credits_frame} final_credits_frame={final_credits_frame} final_module={:02x}/{:02x}/{:02x}",
        final_state[0], final_state[1], final_state[2]
    );
}

/// Run a native Snes9x boundary forward with a deterministic input script.
///
/// This is deliberately oracle-only: it makes route-wide CPU/NMI/DMA tracing
/// available even when the translated runtime has not reached that boundary
/// yet. The trace core remains controlled by its `ZELDA3_SNES9X_TRACE_*`
/// environment variables.
pub(crate) fn run_snes9x_script(args: &[String]) {
    let (core_path, rom_path, state_path, input_path, frames) = match (
        args.first(),
        args.get(1),
        args.get(2),
        args.get(3),
        args.get(4),
    ) {
        (Some(core), Some(rom), Some(state), Some(input), Some(frames)) => {
            let frames = frames.parse::<u32>().unwrap_or_else(|error| {
                eprintln!("invalid frame count `{frames}`: {error}");
                process::exit(2);
            });
            (
                core.as_str(),
                rom.as_str(),
                Path::new(state),
                Path::new(input),
                frames,
            )
        }
        _ => {
            eprintln!(
                "usage: zelda3 --run-snes9x-script <snes9x_libretro.dylib> <rom.sfc> <oracle.state> <input.txt> <frames> [--load-sram <path>] [--input-frame-offset <n>] [--save-state <path>] [--dump-wram <path>] [--dump-sram <path>] [--expected-core-sha256 <sha>] [--expected-rom-sha256 <sha>]"
            );
            process::exit(2);
        }
    };

    let mut load_sram = None::<PathBuf>;
    let mut input_frame_offset = 0u32;
    let mut save_state = None::<PathBuf>;
    let mut dump_wram = None::<PathBuf>;
    let mut dump_sram = None::<PathBuf>;
    let mut expected_core_sha256 = None::<String>;
    let mut expected_rom_sha256 = None::<String>;
    let mut i = 5usize;
    while i < args.len() {
        match args[i].as_str() {
            "--load-sram" => {
                load_sram = Some(PathBuf::from(args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--load-sram requires a path");
                    process::exit(2);
                })));
                i += 2;
            }
            "--input-frame-offset" => {
                input_frame_offset = args
                    .get(i + 1)
                    .and_then(|value| value.parse::<u32>().ok())
                    .unwrap_or_else(|| {
                        eprintln!("--input-frame-offset requires an unsigned integer");
                        process::exit(2);
                    });
                i += 2;
            }
            "--save-state" => {
                save_state = Some(PathBuf::from(args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--save-state requires a path");
                    process::exit(2);
                })));
                i += 2;
            }
            "--dump-wram" => {
                dump_wram = Some(PathBuf::from(args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--dump-wram requires a path");
                    process::exit(2);
                })));
                i += 2;
            }
            "--dump-sram" => {
                dump_sram = Some(PathBuf::from(args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--dump-sram requires a path");
                    process::exit(2);
                })));
                i += 2;
            }
            "--expected-core-sha256" => {
                expected_core_sha256 = Some(
                    args.get(i + 1)
                        .unwrap_or_else(|| {
                            eprintln!("--expected-core-sha256 requires a hash");
                            process::exit(2);
                        })
                        .clone(),
                );
                i += 2;
            }
            "--expected-rom-sha256" => {
                expected_rom_sha256 = Some(
                    args.get(i + 1)
                        .unwrap_or_else(|| {
                            eprintln!("--expected-rom-sha256 requires a hash");
                            process::exit(2);
                        })
                        .clone(),
                );
                i += 2;
            }
            flag => {
                eprintln!("unknown --run-snes9x-script option: {flag}");
                process::exit(2);
            }
        }
    }

    verify_expected_sha256(core_path, "libretro core", expected_core_sha256.as_deref());
    verify_expected_sha256(rom_path, "ROM", expected_rom_sha256.as_deref());
    let input_script = InputScript::from_path(input_path).unwrap_or_else(|error| {
        eprintln!(
            "failed to parse input script {}: {error}",
            input_path.display()
        );
        process::exit(2);
    });
    let state = read_file_or_exit(state_path, "Snes9x state");
    let sram = load_sram
        .as_deref()
        .map(|path| read_file_or_exit(path, "SRAM"));

    let _compare_lock = acquire_snes9x_compare_lock();
    let mut oracle = LibretroCore::load_with_sram(core_path, rom_path, sram.as_deref())
        .unwrap_or_else(|error| {
            eprintln!("failed to initialize Snes9x libretro core: {error}");
            process::exit(1);
        });
    validate_required_libretro_core(
        Some(("Snes9x", "1.63")),
        &oracle.library_name,
        &oracle.library_version,
    )
    .unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(2);
    });
    oracle.unserialize_state(&state).unwrap_or_else(|error| {
        eprintln!(
            "failed to restore Snes9x state {}: {error}",
            state_path.display()
        );
        process::exit(2);
    });

    LIBRETRO_CAPTURE_ENABLED.store(false, Ordering::Relaxed);
    for frame in 0..frames {
        let script_frame = input_frame_offset.wrapping_add(frame);
        oracle.run_frame_discard_with_input(input_script.input_for_frame(script_frame));
        let completed = frame + 1;
        if completed % 100_000 == 0 {
            println!("Snes9x script progress {completed}/{frames}");
        }
    }
    LIBRETRO_CAPTURE_ENABLED.store(true, Ordering::Relaxed);

    if let Some(path) = save_state {
        let state = oracle.serialize_state().unwrap_or_else(|error| {
            eprintln!("failed to serialize final Snes9x state: {error}");
            process::exit(1);
        });
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|error| {
                eprintln!("failed to create {}: {error}", parent.display());
                process::exit(1);
            });
        }
        fs::write(&path, state).unwrap_or_else(|error| {
            eprintln!("failed to write {}: {error}", path.display());
            process::exit(1);
        });
    }

    for (path, memory_id, label) in [
        (dump_wram, RETRO_MEMORY_SYSTEM_RAM, "WRAM"),
        (dump_sram, RETRO_MEMORY_SAVE_RAM, "SRAM"),
    ] {
        let Some(path) = path else {
            continue;
        };
        let bytes = oracle.memory_bytes(memory_id).unwrap_or_else(|| {
            eprintln!("failed to read final Snes9x {label}");
            process::exit(1);
        });
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|error| {
                eprintln!("failed to create {}: {error}", parent.display());
                process::exit(1);
            });
        }
        fs::write(&path, bytes).unwrap_or_else(|error| {
            eprintln!("failed to write {}: {error}", path.display());
            process::exit(1);
        });
    }

    let ram = oracle
        .memory_bytes(RETRO_MEMORY_SYSTEM_RAM)
        .unwrap_or_default();
    let byte = |address: usize| ram.get(address).copied().unwrap_or(0);
    println!(
        "Snes9x script completed {frames} frame(s): module={:02x}/{:02x}/{:02x} room={:04x}",
        byte(0x10),
        byte(0x11),
        byte(0xb0),
        u16::from_le_bytes([byte(0xa0), byte(0xa1)])
    );
}

pub(crate) fn run_compare_libretro_oracle(
    args: &[String],
    default_oracle_name: Option<&str>,
    required_library_name: Option<&str>,
) {
    // A fail-closed panic anywhere in the timing machinery should carry the
    // wire window with it: print the most recent installed host receipt
    // vectors after the default panic report.
    let previous_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        previous_panic_hook(info);
        let vectors = zelda3::zelda_rtl::recent_host_receipt_vectors();
        if !vectors.is_empty() {
            eprintln!("--- recent installed host receipt vectors (oldest first) ---");
            for line in vectors {
                eprintln!("[RCPT-RING] {line}");
            }
        }
    }));
    let operation = "--compare-snes9x-oracle";
    let core_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!("usage: zelda3 {operation} {COMPARE_ORACLE_USAGE}");
            process::exit(2);
        }
    };
    let rom_path = match args.get(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: zelda3 {operation} {COMPARE_ORACLE_USAGE}");
            process::exit(2);
        }
    };
    let mut frames = 300u32;
    let mut input_script = InputScript::default();
    let mut input_script_path = None::<PathBuf>;
    let mut rom_random_script = None::<PathBuf>;
    let mut replay_bundle_dir = None::<PathBuf>;
    let mut replay_bundle = None::<ReplayBundle>;
    let mut allow_mixed_replay_provenance = false;
    let mut live_oracle_rng = false;
    let mut replay_save = None::<PathBuf>;
    let mut load_sram = None::<PathBuf>;
    let mut resume_rust_state = None::<PathBuf>;
    let mut resume_oracle_state = None::<PathBuf>;
    let mut resume_original_timing_checkpoint = None::<PathBuf>;
    let mut resume_semantic_trace_checkpoint = None::<PathBuf>;
    let mut resume_oracle_sram = None::<PathBuf>;
    let mut resume_paired = None::<PathBuf>;
    let mut seed_rust_from_oracle = None::<u32>;
    let mut paired_resume_captures = Vec::<PairedResumeCapture>::new();
    let mut rolling_paired_resume = None::<RollingPairedResumeCapture>;
    let mut native_apu_bootstrap = None::<PathBuf>;
    let mut compare_video = true;
    let mut compare_audio = true;
    let mut compare_from_frame = 0u32;
    let mut compare_engine_state_from_frame = None::<u32>;
    let mut ignore_engine_state = false;
    let mut skip_oracle_frames = 0u32;
    let mut auto_align_video = false;
    let mut lead_rust_audio_blocks = 0u32;
    let mut trace_video_pixel: Option<(usize, usize)> = None;
    let mut color_tolerance = 0u8;
    let mut max_mismatched_pixels = 0usize;
    let mut audio_comparison = AudioComparisonMode::Exact;
    let mut audio_window_ms = 1.0f64;
    let mut audio_silence_threshold = 64i16;
    let mut audio_timing_tolerance_ms = 2.0f64;
    let mut audio_envelope_tolerance = 0.05f64;
    let mut session_dir = None::<PathBuf>;
    let mut cold_evidence_invocation_id = None::<String>;
    let mut scan_all = false;
    let mut expected_core_sha256 = None::<String>;
    let mut expected_rom_sha256 = None::<String>;
    let mut oracle_name = default_oracle_name
        .map(str::to_string)
        .unwrap_or_else(|| oracle_name_from_core_path(core_path));
    let mut i = 2usize;
    if let Some(candidate) = args.get(i) {
        if !candidate.starts_with("--") {
            frames = match candidate.parse() {
                Ok(frames) => frames,
                Err(e) => {
                    eprintln!("invalid frame count `{candidate}`: {e}");
                    process::exit(2);
                }
            };
            i += 1;
        }
    }
    while i < args.len() {
        match args[i].as_str() {
            "--replay-bundle" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--replay-bundle requires a session directory");
                    process::exit(2);
                };
                replay_bundle_dir = Some(PathBuf::from(path));
                i += 2;
            }
            "--allow-mixed-replay-provenance" => {
                allow_mixed_replay_provenance = true;
                i += 1;
            }
            "--replay-save" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--replay-save requires a path");
                    process::exit(2);
                };
                replay_save = Some(PathBuf::from(path));
                i += 2;
            }
            "--input-script" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--input-script requires a path");
                    process::exit(2);
                };
                input_script = match InputScript::from_path(Path::new(path)) {
                    Ok(script) => script,
                    Err(e) => {
                        eprintln!("failed to parse input script {}: {e}", path);
                        process::exit(2);
                    }
                };
                input_script_path = Some(PathBuf::from(path));
                i += 2;
            }
            "--rom-random-script" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--rom-random-script requires a path");
                    process::exit(2);
                };
                rom_random_script = Some(PathBuf::from(path));
                i += 2;
            }
            "--live-oracle-rng" => {
                live_oracle_rng = true;
                i += 1;
            }
            "--load-sram" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--load-sram requires a path");
                    process::exit(2);
                };
                load_sram = Some(PathBuf::from(path));
                i += 2;
            }
            "--resume-rust-state" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--resume-rust-state requires a path");
                    process::exit(2);
                };
                resume_rust_state = Some(PathBuf::from(path));
                i += 2;
            }
            "--resume-oracle-state" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--resume-oracle-state requires a path");
                    process::exit(2);
                };
                resume_oracle_state = Some(PathBuf::from(path));
                i += 2;
            }
            "--resume-oracle-sram" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--resume-oracle-sram requires a path");
                    process::exit(2);
                };
                resume_oracle_sram = Some(PathBuf::from(path));
                i += 2;
            }
            "--resume-paired" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!(
                        "--resume-paired requires a checkpoint or rolling-checkpoint directory"
                    );
                    process::exit(2);
                };
                resume_paired = Some(PathBuf::from(path));
                i += 2;
            }
            "--seed-rust-from-oracle-state" => {
                let Some(frame) = args.get(i + 1).and_then(|v| v.parse::<u32>().ok()) else {
                    eprintln!(
                        "--seed-rust-from-oracle-state requires the oracle state's route frame"
                    );
                    process::exit(2);
                };
                seed_rust_from_oracle = Some(frame);
                i += 2;
            }
            "--save-paired-resume-at" => {
                let (Some(frame), Some(dir)) = (args.get(i + 1), args.get(i + 2)) else {
                    eprintln!("--save-paired-resume-at requires a frame and directory");
                    process::exit(2);
                };
                paired_resume_captures.push(
                    parse_paired_resume_capture(frame, dir).unwrap_or_else(|error| {
                        eprintln!("{error}");
                        process::exit(2);
                    }),
                );
                i += 3;
            }
            "--save-rolling-paired-resume" => {
                let (Some(interval), Some(dir)) = (args.get(i + 1), args.get(i + 2)) else {
                    eprintln!("--save-rolling-paired-resume requires an interval and directory");
                    process::exit(2);
                };
                rolling_paired_resume = Some(
                    parse_rolling_paired_resume_capture(interval, dir).unwrap_or_else(|error| {
                        eprintln!("{error}");
                        process::exit(2);
                    }),
                );
                i += 3;
            }
            "--native-apu-bootstrap" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--native-apu-bootstrap requires a .z3apu path");
                    process::exit(2);
                };
                native_apu_bootstrap = Some(PathBuf::from(path));
                i += 2;
            }
            "--ignore-video" => {
                compare_video = false;
                i += 1;
            }
            "--ignore-audio" => {
                compare_audio = false;
                i += 1;
            }
            "--compare-from-frame" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--compare-from-frame requires a frame number");
                    process::exit(2);
                };
                compare_from_frame = match value.parse() {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("invalid --compare-from-frame `{value}`: {e}");
                        process::exit(2);
                    }
                };
                i += 2;
            }
            "--compare-engine-state-from-frame" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--compare-engine-state-from-frame requires a frame number");
                    process::exit(2);
                };
                compare_engine_state_from_frame = Some(value.parse().unwrap_or_else(|e| {
                    eprintln!("invalid --compare-engine-state-from-frame `{value}`: {e}");
                    process::exit(2);
                }));
                i += 2;
            }
            "--ignore-engine-state" => {
                ignore_engine_state = true;
                i += 1;
            }
            "--skip-snes9x-frames" | "--skip-oracle-frames" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("{} requires a count", args[i]);
                    process::exit(2);
                };
                skip_oracle_frames = match value.parse() {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("invalid {} `{value}`: {e}", args[i]);
                        process::exit(2);
                    }
                };
                i += 2;
            }
            "--oracle-name" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--oracle-name requires a name");
                    process::exit(2);
                };
                oracle_name = value.clone();
                i += 2;
            }
            "--color-tolerance" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--color-tolerance requires a value");
                    process::exit(2);
                };
                color_tolerance = match value.parse() {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("invalid --color-tolerance `{value}`: {e}");
                        process::exit(2);
                    }
                };
                i += 2;
            }
            "--max-mismatched-pixels" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--max-mismatched-pixels requires a count");
                    process::exit(2);
                };
                max_mismatched_pixels = match value.parse() {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("invalid --max-mismatched-pixels `{value}`: {e}");
                        process::exit(2);
                    }
                };
                i += 2;
            }
            "--auto-align-video" => {
                auto_align_video = true;
                i += 1;
            }
            "--lead-rust-audio-blocks" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--lead-rust-audio-blocks requires a count");
                    process::exit(2);
                };
                lead_rust_audio_blocks = match value.parse() {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("invalid --lead-rust-audio-blocks `{value}`: {e}");
                        process::exit(2);
                    }
                };
                i += 2;
            }
            "--trace-video-pixel" => {
                let (Some(x), Some(y)) = (args.get(i + 1), args.get(i + 2)) else {
                    eprintln!("--trace-video-pixel requires x and y");
                    process::exit(2);
                };
                let x = match x.parse() {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("invalid --trace-video-pixel x `{x}`: {e}");
                        process::exit(2);
                    }
                };
                let y = match y.parse() {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("invalid --trace-video-pixel y `{y}`: {e}");
                        process::exit(2);
                    }
                };
                trace_video_pixel = Some((x, y));
                i += 3;
            }
            "--audio-comparison" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--audio-comparison requires exact or timing");
                    process::exit(2);
                };
                audio_comparison = AudioComparisonMode::parse(value).unwrap_or_else(|| {
                    eprintln!("invalid --audio-comparison `{value}`; expected exact or timing");
                    process::exit(2);
                });
                i += 2;
            }
            "--audio-window-ms" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--audio-window-ms requires a positive number");
                    process::exit(2);
                };
                audio_window_ms = value.parse().unwrap_or_else(|e| {
                    eprintln!("invalid --audio-window-ms `{value}`: {e}");
                    process::exit(2);
                });
                i += 2;
            }
            "--audio-silence-threshold" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--audio-silence-threshold requires an i16 value");
                    process::exit(2);
                };
                audio_silence_threshold = value.parse().unwrap_or_else(|e| {
                    eprintln!("invalid --audio-silence-threshold `{value}`: {e}");
                    process::exit(2);
                });
                i += 2;
            }
            "--audio-timing-tolerance-ms" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--audio-timing-tolerance-ms requires a non-negative number");
                    process::exit(2);
                };
                audio_timing_tolerance_ms = value.parse().unwrap_or_else(|e| {
                    eprintln!("invalid --audio-timing-tolerance-ms `{value}`: {e}");
                    process::exit(2);
                });
                i += 2;
            }
            "--audio-envelope-tolerance" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--audio-envelope-tolerance requires a number from 0 through 1");
                    process::exit(2);
                };
                audio_envelope_tolerance = value.parse().unwrap_or_else(|e| {
                    eprintln!("invalid --audio-envelope-tolerance `{value}`: {e}");
                    process::exit(2);
                });
                i += 2;
            }
            "--session-dir" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--session-dir requires a path");
                    process::exit(2);
                };
                session_dir = Some(PathBuf::from(value));
                i += 2;
            }
            "--cold-evidence-invocation-id" if cold_evidence_invocation_id.is_none() => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--cold-evidence-invocation-id requires an ID");
                    process::exit(2);
                };
                validate_cold_evidence_invocation_id(value).unwrap_or_else(|error| {
                    eprintln!("invalid --cold-evidence-invocation-id: {error}");
                    process::exit(2);
                });
                cold_evidence_invocation_id = Some(value.clone());
                i += 2;
            }
            "--cold-evidence-invocation-id" => {
                eprintln!("--cold-evidence-invocation-id may be specified only once");
                process::exit(2);
            }
            "--expected-core-sha256" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--expected-core-sha256 requires a hash");
                    process::exit(2);
                };
                expected_core_sha256 = Some(value.clone());
                i += 2;
            }
            "--expected-rom-sha256" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--expected-rom-sha256 requires a hash");
                    process::exit(2);
                };
                expected_rom_sha256 = Some(value.clone());
                i += 2;
            }
            "--scan-all" => {
                scan_all = true;
                i += 1;
            }
            flag => {
                eprintln!("unknown {operation} option: {flag}");
                process::exit(2);
            }
        }
    }
    compare_engine_state_from_frame = resolve_engine_state_compare_start(
        compare_from_frame,
        compare_engine_state_from_frame,
        ignore_engine_state,
    )
    .unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(2);
    });
    validate_paired_resume_sram_selection(resume_paired.is_some(), load_sram.is_some())
        .unwrap_or_else(|error| {
            eprintln!("{error}");
            process::exit(2);
        });
    if let Some(dir) = replay_bundle_dir.as_deref() {
        if input_script_path.is_some() || rom_random_script.is_some() || load_sram.is_some() {
            eprintln!(
                "--replay-bundle selects input.txt, rom-random.txt, and initial.srm atomically; do not combine it with --input-script, --rom-random-script, or --load-sram"
            );
            process::exit(2);
        }
        if replay_save.is_some()
            || live_oracle_rng
            || resume_paired.is_some()
            || resume_rust_state.is_some()
            || resume_oracle_state.is_some()
            || resume_oracle_sram.is_some()
        {
            eprintln!(
                "--replay-bundle is a cold recorded-route source and cannot be combined with replay-save, live-RNG, or resume-state modes"
            );
            process::exit(2);
        }
        let resolved =
            resolve_replay_bundle(dir, frames, Path::new(rom_path)).unwrap_or_else(|error| {
                eprintln!("invalid --replay-bundle: {error}");
                process::exit(2);
            });
        input_script = InputScript::from_path(&resolved.input_script).unwrap_or_else(|error| {
            eprintln!(
                "failed to parse replay-bundle input script {}: {error}",
                resolved.input_script.display()
            );
            process::exit(2);
        });
        input_script_path = Some(resolved.input_script.clone());
        rom_random_script = Some(resolved.rom_random_script.clone());
        load_sram = Some(resolved.initial_sram.clone());
        replay_bundle = Some(resolved);
    }
    if let Err(error) = validate_replay_source_parents(
        &[
            ("--input-script", input_script_path.as_deref()),
            ("--rom-random-script", rom_random_script.as_deref()),
            ("--load-sram", load_sram.as_deref()),
        ],
        allow_mixed_replay_provenance,
    ) {
        eprintln!("{error}");
        process::exit(2);
    }
    if resume_paired.is_some()
        && (resume_rust_state.is_some()
            || resume_oracle_state.is_some()
            || resume_oracle_sram.is_some())
    {
        eprintln!(
            "--resume-paired cannot be combined with explicit Rust, oracle, or oracle SRAM resume paths"
        );
        process::exit(2);
    }
    if let Some(path) = resume_paired.as_deref() {
        let (rust_state, oracle_state, original_timing_resume, semantic_trace) =
            paired_resume_paths(path).unwrap_or_else(|error| {
                eprintln!(
                    "failed to resolve paired resume {}: {error}",
                    path.display()
                );
                process::exit(2);
            });
        validate_paired_resume_provenance(
            path,
            Path::new(core_path),
            Path::new(rom_path),
            input_script_path.as_deref(),
            rom_random_script.as_deref(),
            allow_mixed_replay_provenance,
        )
        .unwrap_or_else(|error| {
            eprintln!(
                "failed to bind paired resume {} to the selected replay sources: {error}",
                path.display()
            );
            process::exit(2);
        });
        resume_rust_state = Some(rust_state);
        resume_oracle_state = Some(oracle_state);
        resume_original_timing_checkpoint = Some(original_timing_resume);
        resume_semantic_trace_checkpoint = Some(semantic_trace);
    }
    if seed_rust_from_oracle.is_some() {
        if resume_oracle_state.is_none() || resume_rust_state.is_some() || resume_paired.is_some() {
            eprintln!(
                "--seed-rust-from-oracle-state requires --resume-oracle-state (plus optional --resume-oracle-sram) and no Rust resume state"
            );
            process::exit(2);
        }
    } else if resume_rust_state.is_some() != resume_oracle_state.is_some() {
        eprintln!(
            "--resume-rust-state and --resume-oracle-state must be provided together so both engines resume at one boundary"
        );
        process::exit(2);
    }
    if resume_oracle_sram.is_some() && resume_oracle_state.is_none() {
        eprintln!("--resume-oracle-sram requires --resume-oracle-state");
        process::exit(2);
    }
    if replay_save.is_some() && resume_rust_state.is_some() {
        eprintln!("--replay-save cannot be combined with paired resume states");
        process::exit(2);
    }
    if live_oracle_rng && rom_random_script.is_some() {
        eprintln!("--live-oracle-rng cannot be combined with --rom-random-script");
        process::exit(2);
    }
    if live_oracle_rng && replay_save.is_some() {
        eprintln!("--live-oracle-rng requires a direct input script, not --replay-save");
        process::exit(2);
    }
    // Engine-state comparison is independent of how cartridge RNG values are
    // supplied.  Live RNG is required by the cold calibration which creates a
    // replay script, but a manifest-bound recorded script is the exact input
    // needed by checkpoint diagnostics after that calibration.  Keeping this
    // lane available for both sources lets a short paired replay reproduce the
    // same Zelda-state divergence without changing promotion authority.
    // ZELDA3_LIVE_RNG_DIAGNOSTIC_RESUME=1 opts a live-RNG run into a paired
    // resume for the fix loop (a ~1400-frame resumed probe instead of a cold
    // replay to the frontier). Such a run is diagnostic only: its session is
    // never parity-eligible and its RNG stream is never a calibration source.
    let live_rng_diagnostic_resume =
        std::env::var_os("ZELDA3_LIVE_RNG_DIAGNOSTIC_RESUME").is_some_and(|v| v == "1");
    if live_oracle_rng && resume_rust_state.is_some() && !live_rng_diagnostic_resume {
        eprintln!(
            "--live-oracle-rng is a cold-oracle authority mode and cannot use paired resume states (set ZELDA3_LIVE_RNG_DIAGNOSTIC_RESUME=1 for a diagnostic-only resumed probe)"
        );
        process::exit(2);
    }
    if live_oracle_rng && session_dir.is_none() {
        eprintln!("--live-oracle-rng requires --session-dir for its oracle trace receipt");
        process::exit(2);
    }
    if native_apu_bootstrap.is_some()
        && (replay_save.is_some() || resume_rust_state.is_some() || load_sram.is_some())
    {
        eprintln!(
            "--native-apu-bootstrap currently requires a clean-start route; resumed game state needs a matching captured APU state"
        );
        process::exit(2);
    }
    if native_apu_bootstrap.is_some() && lead_rust_audio_blocks != 0 {
        eprintln!("--native-apu-bootstrap cannot be combined with --lead-rust-audio-blocks");
        process::exit(2);
    }
    if native_apu_bootstrap.is_some() && skip_oracle_frames != 0 {
        eprintln!(
            "--native-apu-bootstrap requires the same clean-start frame origin as the oracle; do not use --skip-oracle-frames"
        );
        process::exit(2);
    }
    if resume_rust_state.is_some() && (load_sram.is_some() || skip_oracle_frames != 0) {
        eprintln!(
            "paired resume states cannot be combined with --load-sram or --skip-oracle-frames"
        );
        process::exit(2);
    }
    if auto_align_video && compare_audio {
        eprintln!("--auto-align-video is video-only; pass --ignore-audio for this mode");
        process::exit(2);
    }
    if required_library_name == Some("Snes9x") && auto_align_video {
        eprintln!(
            "Snes9x parity never auto-aligns; use one fixed --skip-oracle-frames value instead"
        );
        process::exit(2);
    }
    let debug_smp_bootstrap_path = env::var_os("ZELDA3_DEBUG_SNES9X_SMP_BOOTSTRAP");
    let debug_smp_first_nmi_path = env::var_os("ZELDA3_DEBUG_SNES9X_SMP_FIRST_NMI");
    let debug_first_nmi_dma_setup_path = env::var_os("ZELDA3_DEBUG_SNES9X_FIRST_NMI_DMA_SETUP");
    let debug_first_nmi_dma_path = env::var_os("ZELDA3_DEBUG_SNES9X_FIRST_NMI_DMA");
    let debug_first_nmi_return_path = env::var_os("ZELDA3_DEBUG_SNES9X_FIRST_NMI_RETURN");
    let debug_first_nmi_return_core_trace_path = debug_first_nmi_return_path.as_ref().map(|path| {
        PathBuf::from(format!(
            "{}.core-trace.tmp.jsonl",
            Path::new(path).display()
        ))
    });
    if (debug_smp_first_nmi_path.is_some()
        || debug_first_nmi_dma_setup_path.is_some()
        || debug_first_nmi_dma_path.is_some()
        || debug_first_nmi_return_path.is_some())
        && debug_smp_bootstrap_path.is_none()
    {
        // The pinned core's existing timing buffer is enabled by the original
        // bootstrap flag at retro_run entry. This CLI is still single-threaded
        // and the core has not been loaded yet, so it is safe to activate that
        // maintained buffer without also creating a bootstrap output file.
        unsafe {
            env::set_var(
                "ZELDA3_DEBUG_SNES9X_SMP_BOOTSTRAP",
                "cpu-timing-enabled-by-first-nmi-capture",
            );
        }
    }
    if debug_first_nmi_dma_path.is_some() || debug_first_nmi_return_path.is_some() {
        // The pinned core owns a generic per-retro_run DMA ledger. Route
        // selection remains below in this host-only fixture writer.
        unsafe {
            env::set_var("ZELDA3_DEBUG_SNES9X_DMA_LEDGER", "1");
        }
    }
    if let Some(path) = debug_first_nmi_return_core_trace_path.as_deref() {
        if live_oracle_rng {
            eprintln!(
                "first-NMI retro_run-return fixture capture cannot share the generic trace stream with --live-oracle-rng"
            );
            process::exit(2);
        }
        fs::File::create(path).unwrap_or_else(|error| {
            eprintln!(
                "failed to initialize first-NMI retro_run-return core trace {}: {error}",
                path.display()
            );
            process::exit(1);
        });
        unsafe {
            env::set_var("ZELDA3_SNES9X_TRACE", path);
            env::set_var("ZELDA3_SNES9X_TRACE_EVENTS", "frame,hdma");
        }
    }
    let semantic_trace_available = replay_save.is_none()
        && debug_smp_bootstrap_path.is_none()
        && debug_smp_first_nmi_path.is_none()
        && debug_first_nmi_dma_setup_path.is_none()
        && debug_first_nmi_dma_path.is_none()
        && debug_first_nmi_return_path.is_none();
    if !audio_window_ms.is_finite()
        || audio_window_ms <= 0.0
        || !audio_timing_tolerance_ms.is_finite()
        || audio_timing_tolerance_ms < 0.0
        || !audio_envelope_tolerance.is_finite()
        || !(0.0..=1.0).contains(&audio_envelope_tolerance)
        || audio_silence_threshold < 0
    {
        eprintln!("invalid audio comparison thresholds");
        process::exit(2);
    }
    if let Err(error) = validate_libretro_frame_window(frames, compare_from_frame) {
        eprintln!("{error}");
        process::exit(2);
    }
    paired_resume_captures.sort_by_key(|capture| capture.frame);
    if let Some(capture) = paired_resume_captures
        .iter()
        .find(|capture| capture.frame >= frames)
    {
        eprintln!(
            "paired-resume frame {} must be earlier than final frame count {frames}",
            capture.frame
        );
        process::exit(2);
    }
    // Writing a replayable result must not change the comparison's stopping
    // policy. Frontier probes need a session receipt and must still stop at
    // the first video mismatch; focused windows opt into `--scan-all`
    // explicitly so they can show recovery on the surrounding frames.
    scan_all = scan_all_policy(scan_all, session_dir.is_some());
    verify_expected_sha256(core_path, "libretro core", expected_core_sha256.as_deref());
    verify_expected_sha256(rom_path, "ROM", expected_rom_sha256.as_deref());
    let _compare_lock =
        acquire_snes9x_compare_lock_mode(compare_video || trace_video_pixel.is_some());

    let live_oracle_rng_trace_path = live_oracle_rng.then(|| {
        let dir = session_dir
            .as_deref()
            .expect("live oracle RNG mode validates its session directory");
        fs::create_dir_all(dir).unwrap_or_else(|error| {
            eprintln!(
                "failed to create live oracle RNG session {}: {error}",
                dir.display()
            );
            process::exit(1);
        });
        let path = dir.join(LIVE_ORACLE_RNG_TRACE_ARTIFACT);
        fs::File::create(&path).unwrap_or_else(|error| {
            eprintln!(
                "failed to initialize live oracle RNG trace {}: {error}",
                path.display()
            );
            process::exit(1);
        });
        env::set_var("ZELDA3_SNES9X_TRACE", &path);
        // Only the cartridge routine's final store is needed here. The broader
        // `rng` stream also records beam-counter reads and unrelated $0fa1
        // writes, producing tens of thousands of events for a few hundred
        // replay samples.
        // Live RNG authority and hardware chronology must be observable in the
        // same cold run. Preserve explicitly requested trace domains and add
        // the cartridge RNG store required by `LiveOracleRngTrace`.
        let trace_events =
            trace_events_with_rom_rng(env::var("ZELDA3_SNES9X_TRACE_EVENTS").ok().as_deref());
        env::set_var("ZELDA3_SNES9X_TRACE_EVENTS", trace_events);
        // The trace core exempts only the required `rom-rng` domain from its
        // frame filter. Preserve an explicitly requested diagnostic window so
        // PC/DMA/HDMA traces cannot silently expand to the entire route.
        path
    });
    // Configure semantic receipt decoding only after every host-only trace
    // producer has selected the shared generic trace path. Live RNG and Zelda
    // semantic adapters then consume the same source stream through independent
    // cursors instead of silently binding different files.
    let mut oracle_semantic_trace = semantic_trace_available.then(|| {
        Snes9xOracleSemanticTrace::configure(session_dir.as_deref()).unwrap_or_else(|error| {
            eprintln!("failed to configure pinned-Snes9x semantic receipts: {error}");
            process::exit(1);
        })
    });

    let (mut game, start_frame) = if let Some(path) = resume_rust_state.as_deref() {
        let checkpoint = load_play_crash_checkpoint(path).unwrap_or_else(|error| {
            eprintln!(
                "failed to load Rust resume state {}: {error}",
                path.display()
            );
            process::exit(2);
        });
        let mut game = checkpoint.game;
        game.restore_live_rom_timing_after_checkpoint();
        if let Some(path) = resume_original_timing_checkpoint.as_deref() {
            restore_original_timing_resume_checkpoint(&mut game, path).unwrap_or_else(|error| {
                eprintln!("{error}");
                process::exit(2);
            });
        }
        (game, checkpoint.host_frame)
    } else if let Some(frame) = seed_rust_from_oracle {
        // Seeded from the oracle's memory once the oracle state is loaded
        // below; the translated state starts as a fresh boot until then.
        (load_default_play_state(), frame)
    } else {
        // Start from the same embedded asset pack as plain `cargo run`.
        (load_default_play_state(), 0)
    };
    // A parity oracle is always tied to a concrete ROM. Attach those bytes to
    // the translated state as well so ROM-semantic operations (including the
    // byte-exact CPU<->SPC upload streams) use the same source authority.
    let translated_rom = fs::read(rom_path).unwrap_or_else(|error| {
        eprintln!("failed to read ROM {rom_path}: {error}");
        process::exit(2);
    });
    game.set_rom(&translated_rom);
    if let Some(path) = replay_save.as_deref() {
        game.replay_save_file(path).unwrap_or_else(|error| {
            eprintln!("failed to load replay save {}: {error}", path.display());
            process::exit(2);
        });
    }
    if start_frame >= frames {
        eprintln!("resume frame {start_frame} must be earlier than final frame count {frames}");
        process::exit(2);
    }
    if let Some(capture) = paired_resume_captures
        .iter()
        .find(|capture| capture.frame < start_frame)
    {
        eprintln!(
            "paired-resume frame {} precedes resumed route frame {start_frame}",
            capture.frame
        );
        process::exit(2);
    }
    let effective_compare_from_frame = compare_from_frame.max(start_frame);
    if let Some(path) = load_sram.as_deref() {
        let sram = read_file_or_exit(path, "SRAM");
        apply_sram_to_game_or_exit(&mut game, path, &sram);
    }
    if let Some(path) = rom_random_script.as_deref() {
        let text = fs::read_to_string(path).unwrap_or_else(|error| {
            eprintln!(
                "failed to read ROM random script {}: {error}",
                path.display()
            );
            process::exit(2);
        });
        let samples = zelda3::parse_rom_random_script(&text).unwrap_or_else(|error| {
            eprintln!(
                "failed to parse ROM random script {}: {error}",
                path.display()
            );
            process::exit(2);
        });
        game.install_rom_random_replay(samples, start_frame);
    }
    let width = 256u32;
    let height = 224u32;
    let mut rust_audio = Vec::new();
    let mut discard_audio = Vec::new();
    let mut dsp_writes: Vec<DspWriteEvent> = Vec::new();
    let mut last_sample_frames = 800usize;
    let native_apu_trace_path =
        env::var_os("ZELDA3_DEBUG_NATIVE_APU_DSP_WRITES").map(PathBuf::from);
    let mut native_apu = native_apu_bootstrap.as_ref().map(|path| {
        let mut checkpoint = load_apu_bootstrap_checkpoint(path).unwrap_or_else(|error| {
            eprintln!(
                "failed to load native APU bootstrap {}: {error}",
                path.display()
            );
            process::exit(2);
        });
        checkpoint.apu.dsp.sample_offset = 0;
        checkpoint.apu.dsp.sample_buffer.fill(0);
        checkpoint.apu.dsp_write_history.clear();
        if native_apu_trace_path.is_some() {
            checkpoint.apu.debug_dsp_write_trace = Some(Vec::new());
        }
        println!(
            "native bootstrapped APU diagnostic enabled: {} (SPC pc=${:04x})",
            path.display(),
            checkpoint.apu.spc.pc
        );
        checkpoint.apu
    });
    let initial_sram = game.sram.clone();
    let mut oracle = match LibretroCore::load_with_sram(core_path, rom_path, Some(&initial_sram)) {
        Ok(core) => core,
        Err(e) => {
            eprintln!("failed to initialize libretro core: {e}");
            process::exit(1);
        }
    };
    if !semantic_trace_authority_available(
        oracle_semantic_trace.is_some(),
        oracle.debug_ppu_value.is_some(),
    ) {
        // An empty trace interval is authoritative only when the loaded core
        // actually exports the maintained generic trace API. The stock core
        // used by the checkpointable video preflight creates no trace rows;
        // treating that absence as a genuinely empty Snes9x receipt would
        // activate Live timing and suppress the legacy fallback without any
        // observation from the authority.
        oracle_semantic_trace = None;
    }
    if let Some(path) = resume_semantic_trace_checkpoint.as_deref() {
        let trace = oracle_semantic_trace.as_mut().unwrap_or_else(|| {
            eprintln!(
                "paired resume requires the maintained Snes9x semantic trace API to restore {}",
                path.display()
            );
            process::exit(2);
        });
        let checkpoint: Snes9xOracleSemanticTraceCheckpoint = fs::read(path)
            .map_err(|error| format!("read {}: {error}", path.display()))
            .and_then(|bytes| {
                serde_json::from_slice(&bytes)
                    .map_err(|error| format!("decode {}: {error}", path.display()))
            })
            .unwrap_or_else(|error| {
                eprintln!("failed to read paired Snes9x semantic checkpoint: {error}");
                process::exit(2);
            });
        trace
            .restore_checkpoint(checkpoint)
            .unwrap_or_else(|error| {
                eprintln!(
                    "failed to restore paired Snes9x semantic checkpoint {}: {error}",
                    path.display()
                );
                process::exit(2);
            });
    }
    if let Some(path) = resume_oracle_state.as_deref() {
        let state = read_file_or_exit(path, "libretro resume state");
        oracle.unserialize_state(&state).unwrap_or_else(|error| {
            eprintln!(
                "failed to load oracle resume state {}: {error}",
                path.display()
            );
            process::exit(2);
        });
        if let Some(sram_path) = resume_oracle_sram.as_deref() {
            let sram = read_file_or_exit(sram_path, "oracle resume SRAM");
            oracle
                .replace_memory(RETRO_MEMORY_SAVE_RAM, &sram, "SRAM")
                .unwrap_or_else(|error| {
                    eprintln!(
                        "failed to load oracle resume SRAM {}: {error}",
                        sram_path.display()
                    );
                    process::exit(2);
                });
        }
        if let Some(frame) = seed_rust_from_oracle {
            // Rust is seeded inside the frame loop at the first run boundary
            // the oracle reaches with no pending NMI publication and a
            // completed main-loop iteration (a savestate captured mid-NMI
            // cannot seed a quiescent translated state).
            let Some(trace) = oracle_semantic_trace.as_mut() else {
                eprintln!("--seed-rust-from-oracle-state requires the pinned trace core's semantic receipts");
                process::exit(2);
            };
            trace.begin_seed_warmup();
            println!(
                "oracle-seeded segment: warming up from {oracle_name} state at frame {frame} until a clean run boundary (development evidence, not parity authority)"
            );
        } else {
            println!(
                "resumed Rust and {oracle_name} from paired pre-frame states at frame {start_frame}"
            );
        }
    }
    let required_core = required_library_name.map(|name| {
        if name == "Snes9x" {
            (name, "1.63")
        } else {
            (name, "")
        }
    });
    if let Err(error) = validate_required_libretro_core(
        required_core,
        &oracle.library_name,
        &oracle.library_version,
    ) {
        eprintln!("{error}");
        process::exit(2);
    }
    println!(
        "{oracle_name} oracle core={} version={} api={} geometry={}x{} fps={:.9} sample_rate={:.3}",
        oracle.library_name,
        oracle.library_version,
        oracle.api_version,
        oracle.geometry.base_width,
        oracle.geometry.base_height,
        oracle.av_info.timing.fps,
        oracle.av_info.timing.sample_rate,
    );
    for _ in 0..skip_oracle_frames {
        let _ = oracle.run_frame_with_input(0);
    }
    if skip_oracle_frames != 0 {
        println!("advanced {oracle_name} by {skip_oracle_frames} frame(s) before comparison");
    }
    if lead_rust_audio_blocks != 0 {
        println!("leading rust audio by {lead_rust_audio_blocks} block(s) per compared frame");
    }
    let initial_oracle_state = oracle.serialize_state().unwrap_or_else(|e| {
        eprintln!("Snes9x/libretro serialization is required for replayable parity failures: {e}");
        process::exit(1);
    });
    oracle
        .unserialize_state(&initial_oracle_state)
        .unwrap_or_else(|e| {
            eprintln!("Snes9x/libretro state round-trip failed before comparison: {e}");
            process::exit(1);
        });
    let timing_options = AudioTimingOptions::from_sample_rate(
        oracle.av_info.timing.sample_rate,
        audio_window_ms,
        audio_silence_threshold,
        audio_timing_tolerance_ms,
        audio_envelope_tolerance,
    );
    if audio_comparison == AudioComparisonMode::Timing {
        eprintln!(
            "audio comparison mode `timing` is diagnostic only and cannot produce a full parity pass"
        );
    }
    let mut continuous_audio = StreamingAudioComparator::new(audio_comparison, timing_options);
    let resumed_frame_count = frames.saturating_sub(start_frame) as usize;
    let mut input_history = Vec::<(u32, u16)>::with_capacity(resumed_frame_count);
    let mut audio_frame_ends = Vec::<u64>::with_capacity(resumed_frame_count);
    let mut compared_audio_sample_frames = 0u64;
    let mut wrote_first_audio_mismatch = false;
    let mut completed_frames = start_frame;
    let mut first_engine_state_mismatch: Option<(u32, Vec<String>)> = None;
    let mut rom_random_overdue_reported = false;
    // Enumerator mode: instead of breaking on the first engine-state mismatch,
    // record every divergent frame and keep going, so one cold pass surfaces the
    // whole class of WRAM divergences (matching field sets collapse to ranges in
    // the post-run summary). Diagnostic only; the ordinary ratchet breaks at the
    // first unequal frame.
    //
    // Pair with ZELDA3_DEBUG_ALLOW_UNEXPECTED_ROM_RANDOM=1: past the FIRST divergence
    // that changes Rust's RNG-call count, the live-oracle RNG replay exhausts and
    // would otherwise panic. With the allow flag it continues on zero samples, so
    // divergences BEFORE that point are exact and later ones are degraded (Rust is
    // on a drifted path). Read the summary up to the first sprite/AI fork.
    let engine_state_scan_all = env::var_os("ZELDA3_ENGINE_STATE_SCAN_ALL").is_some();
    // ZELDA3_ENGINE_STATE_DUMP_FRAMES=<lo>-<hi> prints both sides' module and
    // Link-coordinate boundary values for every compared frame in the range,
    // straight from the same RAM reads the mismatch check uses. This is the
    // ambiguity-free readout of per-frame chronology (trace event frame labels
    // drift when the game frame counter is held).
    let engine_state_dump_frames =
        env::var("ZELDA3_ENGINE_STATE_DUMP_FRAMES")
            .ok()
            .and_then(|raw| {
                let (lo, hi) = raw.split_once('-')?;
                Some((
                    lo.trim().parse::<u32>().ok()?,
                    hi.trim().parse::<u32>().ok()?,
                ))
            });
    // ZELDA3_ENGINE_STATE_DUMP_SPRITE=<slot> adds that sprite slot's
    // state/delay_main/ai_state to each dumped line.
    let engine_state_dump_sprite = env::var("ZELDA3_ENGINE_STATE_DUMP_SPRITE")
        .ok()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .filter(|slot| *slot < 16);
    // ZELDA3_ENGINE_STATE_DUMP_BYTES=<hexaddr,hexaddr,...> adds both sides'
    // values of arbitrary WRAM bytes to each dumped frame ([ENGINE-BYTES]).
    let engine_state_dump_bytes: Vec<usize> = env::var("ZELDA3_ENGINE_STATE_DUMP_BYTES")
        .ok()
        .map(|raw| {
            raw.split(',')
                .filter_map(|part| {
                    usize::from_str_radix(part.trim().trim_start_matches("0x"), 16).ok()
                })
                .collect()
        })
        .unwrap_or_default();
    let mut engine_state_divergences: Vec<(u32, Vec<String>)> = Vec::new();
    let mut oracle_before_state = initial_oracle_state.clone();
    let oracle_before_state_frame = start_frame;
    let mut video_mismatch_ranges = Vec::<(u32, u32)>::new();
    let mut first_video_mismatch = None::<String>;
    let mut live_oracle_rng_trace = live_oracle_rng_trace_path.map(LiveOracleRngTrace::new);
    let debug_spc_clock_witness = env::var_os("ZELDA3_DEBUG_SPC_CLOCK_WITNESS").is_some();
    let mut previous_spc_clock_phase = None::<Option<u8>>;
    let initial_input_script = input_script_path.as_deref().map(|path| {
        fs::read(path).unwrap_or_else(|error| {
            eprintln!(
                "failed to read controller stream for replayable session {}: {error}",
                path.display()
            );
            process::exit(1);
        })
    });
    let mut frame_receipts = initialize_libretro_session(
        session_dir.as_deref(),
        core_path,
        rom_path,
        &oracle,
        &game,
        &initial_sram,
        &initial_oracle_state,
        frames,
        start_frame,
        effective_compare_from_frame,
        compare_engine_state_from_frame,
        skip_oracle_frames,
        compare_video,
        compare_audio,
        audio_comparison,
        timing_options,
        replay_save.as_deref(),
        replay_bundle.as_ref(),
        initial_input_script.as_deref().unwrap_or_default(),
        rom_random_script.as_deref(),
        live_oracle_rng,
        scan_all,
        cold_evidence_invocation_id.as_deref(),
    );
    let mut av_hashes = session_dir.as_deref().map(|dir| {
        BufWriter::new(
            fs::File::create(dir.join("av_hashes.jsonl")).unwrap_or_else(|error| {
                eprintln!("failed to create canonical A/V hash ledger: {error}");
                process::exit(1);
            }),
        )
    });
    let mut debug_dsp_globals = if env::var_os("ZELDA3_DEBUG_DSP_GLOBALS").is_some() {
        session_dir.as_deref().map(|dir| {
            BufWriter::new(
                fs::File::create(dir.join("oracle_dsp_globals.jsonl")).unwrap_or_else(|error| {
                    eprintln!("failed to create oracle DSP global trace: {error}");
                    process::exit(1);
                }),
            )
        })
    } else {
        None
    };
    let mut debug_dsp_globals_previous = None::<[i32; 4]>;
    let mut debug_dsp_writes = env::var_os("ZELDA3_DEBUG_SNES9X_DSP_WRITES").map(|path| {
        BufWriter::new(fs::File::create(path).unwrap_or_else(|error| {
            eprintln!("failed to create Snes9x DSP-write trace: {error}");
            process::exit(1);
        }))
    });
    let mut debug_dsp_register_writes =
        env::var_os("ZELDA3_DEBUG_SNES9X_DSP_REGISTER_WRITES").map(|path| {
            BufWriter::new(fs::File::create(path).unwrap_or_else(|error| {
                eprintln!("failed to create Snes9x DSP-register-write trace: {error}");
                process::exit(1);
            }))
        });
    let mut debug_smp_bootstrap = debug_smp_bootstrap_path.map(|path| {
        let mut writer = BufWriter::new(fs::File::create(path).unwrap_or_else(|error| {
            eprintln!("failed to create Snes9x SMP-bootstrap trace: {error}");
            process::exit(1);
        }));
        write_snes9x_smp_bootstrap_header(
            &mut writer,
            core_path,
            rom_path,
            &oracle,
            &initial_oracle_state,
        )
        .unwrap_or_else(|error| {
            eprintln!("failed to write Snes9x SMP-bootstrap header: {error}");
            process::exit(1);
        });
        writer
    });
    let mut debug_smp_bootstrap_complete = false;
    let mut debug_smp_bootstrap_instructions = Vec::new();
    let mut debug_smp_bootstrap_output_writes = Vec::new();
    let mut debug_smp_bootstrap_cpu_accesses = Vec::new();
    let mut debug_smp_bootstrap_cpu_timing_transactions = Vec::new();
    let mut debug_smp_first_nmi = debug_smp_first_nmi_path.map(|path| {
        let mut writer = BufWriter::new(fs::File::create(path).unwrap_or_else(|error| {
            eprintln!("failed to create Snes9x post-handoff first-NMI trace: {error}");
            process::exit(1);
        }));
        write_snes9x_smp_first_nmi_header(&mut writer, core_path, rom_path, &oracle)
            .unwrap_or_else(|error| {
                eprintln!("failed to write Snes9x post-handoff first-NMI header: {error}");
                process::exit(1);
            });
        writer
    });
    let mut debug_smp_first_nmi_complete = false;
    let mut debug_smp_first_nmi_anchor = None::<SmpPostHandoffAnchor>;
    let mut debug_smp_first_nmi_instructions = Vec::new();
    let mut debug_smp_first_nmi_output_writes = Vec::new();
    let mut debug_smp_first_nmi_cpu_accesses = Vec::new();
    let mut debug_smp_first_nmi_cpu_timing_transactions = Vec::new();
    let mut debug_first_nmi_dma_setup = debug_first_nmi_dma_setup_path.map(|path| {
        let mut writer = BufWriter::new(fs::File::create(path).unwrap_or_else(|error| {
            eprintln!("failed to create Snes9x first-NMI DMA-setup trace: {error}");
            process::exit(1);
        }));
        write_snes9x_first_nmi_dma_setup_header(
            &mut writer,
            core_path,
            rom_path,
            &oracle,
            load_sram.as_deref(),
            &initial_sram,
        )
        .unwrap_or_else(|error| {
            eprintln!("failed to write Snes9x first-NMI DMA-setup header: {error}");
            process::exit(1);
        });
        writer
    });
    let mut debug_first_nmi_dma_setup_complete = false;
    let mut debug_first_nmi_dma_setup_anchor = None::<FirstNmiApuAnchor>;
    let mut debug_first_nmi_dma_setup_cpu_accesses = Vec::new();
    let mut debug_first_nmi_dma_setup_cpu_timing_transactions = Vec::new();
    let mut debug_first_nmi_dma = debug_first_nmi_dma_path.map(|path| {
        let mut writer = BufWriter::new(fs::File::create(path).unwrap_or_else(|error| {
            eprintln!("failed to create Snes9x first-NMI DMA trace: {error}");
            process::exit(1);
        }));
        write_snes9x_first_nmi_dma_header(
            &mut writer,
            core_path,
            rom_path,
            &oracle,
            load_sram.as_deref(),
            &initial_sram,
        )
        .unwrap_or_else(|error| {
            eprintln!("failed to write Snes9x first-NMI DMA header: {error}");
            process::exit(1);
        });
        writer
    });
    let mut debug_first_nmi_dma_complete = false;
    let mut debug_first_nmi_return = debug_first_nmi_return_path.map(|path| {
        let mut pending = PendingFirstNmiReturnFixture::create(path).unwrap_or_else(|error| {
            eprintln!("failed to create Snes9x first-NMI return trace: {error}");
            process::exit(1);
        });
        write_snes9x_first_nmi_return_header(
            &mut pending.writer,
            core_path,
            rom_path,
            &oracle,
            load_sram.as_deref(),
            &initial_sram,
        )
        .unwrap_or_else(|error| {
            eprintln!("failed to write Snes9x first-NMI return header: {error}");
            process::exit(1);
        });
        pending
    });
    let mut debug_first_nmi_return_complete = false;
    let mut debug_native_apu_dsp_writes = native_apu_trace_path.map(|path| {
        BufWriter::new(fs::File::create(path).unwrap_or_else(|error| {
            eprintln!("failed to create native APU DSP-write trace: {error}");
            process::exit(1);
        }))
    });
    let capture_all_display_oracle = env::var_os("ZELDA3_CAPTURE_DISPLAY_ORACLE").is_some();
    let display_oracle_after_frames =
        debug_frame_selection_from_env("ZELDA3_CAPTURE_DISPLAY_ORACLE_FRAMES", None);
    let display_oracle_before_frames =
        debug_frame_selection_from_env("ZELDA3_CAPTURE_DISPLAY_ORACLE_BEFORE_FRAMES", None);
    let mut display_oracle_receipts = (capture_all_display_oracle
        || !display_oracle_after_frames.is_empty()
        || !display_oracle_before_frames.is_empty())
    .then(|| {
        let dir = session_dir.as_deref().unwrap_or_else(|| {
            eprintln!(
                "ZELDA3_CAPTURE_DISPLAY_ORACLE[_BEFORE_FRAMES|_FRAMES] requires --session-dir"
            );
            process::exit(2);
        });
        BufWriter::new(
            fs::File::create(dir.join("display_oracle.jsonl")).unwrap_or_else(|error| {
                eprintln!("failed to create display-oracle receipt: {error}");
                process::exit(1);
            }),
        )
    });
    let mut obj_state_ledger = env::var_os("ZELDA3_CAPTURE_OBJ_STATE_LEDGER").map(|_| {
        let dir = session_dir.as_deref().unwrap_or_else(|| {
            eprintln!("ZELDA3_CAPTURE_OBJ_STATE_LEDGER requires --session-dir");
            process::exit(2);
        });
        if oracle.debug_ppu_value(29, 0).is_none() {
            eprintln!("OBJ state ledger requires the instrumented Snes9x core");
            process::exit(2);
        }
        BufWriter::new(
            fs::File::create(dir.join("obj_state_ledger.jsonl")).unwrap_or_else(|error| {
                eprintln!("failed to create OBJ state ledger: {error}");
                process::exit(1);
            }),
        )
    });
    let mut wrote_first_obj_cache_divergence = false;
    let trace_poly_sched = std::env::var_os("TRACE_POLY_SCHED").is_some();
    let trace_shield_dma = std::env::var_os("ZELDA3_DEBUG_SHIELD_DMA").is_some();
    let debug_vram_frames = debug_frame_selection_from_env("ZELDA3_DEBUG_VRAM_FRAMES", None);
    let debug_video_frames = debug_frame_selection_from_env("ZELDA3_DEBUG_VIDEO_FRAMES", None);
    let debug_text_frames = debug_frame_selection_from_env("ZELDA3_DEBUG_TEXT_FRAMES", None);
    let debug_sprite_frames = debug_frame_selection_from_env("ZELDA3_DEBUG_SPRITE_FRAMES", None);
    let debug_wram_frames =
        debug_frame_selection_from_env("ZELDA3_DEBUG_WRAM_FRAMES", Some("ZELDA3_DEBUG_WRAM_FRAME"));
    let debug_dsp_trace_frames = debug_frame_selection_from_env(
        "ZELDA3_DEBUG_DSP_TRACE_FRAMES",
        Some("ZELDA3_DEBUG_DSP_TRACE_FRAME"),
    );
    // Oracle-side publication probe. This deliberately compares the raw PPU
    // VRAM bytes after every emulated frame, independently of RGBA output, so
    // a simulation/DMA skew can be located before a later rendering mismatch
    // obscures its cause.
    let assert_oracle_vram = std::env::var_os("ZELDA3_ASSERT_ORACLE_VRAM").is_some();
    let assert_oracle_vram_range = std::env::var("ZELDA3_ASSERT_ORACLE_VRAM_RANGE")
        .ok()
        .map(|value| parse_debug_byte_range(&value).unwrap_or_else(|| {
            eprintln!(
                "invalid ZELDA3_ASSERT_ORACLE_VRAM_RANGE={value:?}; expected START..END (decimal or 0x-prefixed hexadecimal)"
            );
            process::exit(2);
        }));
    let assert_oracle_boot_contract =
        std::env::var_os("ZELDA3_ASSERT_ORACLE_BOOT_CONTRACT").is_some();
    let mut rust_boot_contract = std::env::var_os("ZELDA3_RUST_BOOT_CONTRACT").map(|path| {
        BufWriter::new(fs::File::create(path).unwrap_or_else(|error| {
            eprintln!("failed to create Rust boot-contract trace: {error}");
            process::exit(1);
        }))
    });
    let mut previous_oracle_vram = None::<Vec<u8>>;
    let mut previous_rust_vram = None::<Vec<u8>>;
    let mut previous_oracle_video = None::<PresentedOracleVideo>;
    let mut presented_bg_tilemap_cache = PresentedBgTilemapCache::default();
    let mut previous_shield_dma_trace = None::<(u8, u8, u16, u16, u8, u8, u16, u16)>;
    let mut previous_uncle_trace = None::<(u8, u8, u8, u8, u8, u8, u8, u8)>;
    // Video parity is intentionally measured through the same native window
    // renderer used by `cargo run`.  Do not replace this with an offscreen or
    // CPU/headless compositor: a successful oracle receipt must prove what
    // the user actually sees in the window.
    let mut native_window_video = (compare_video || trace_video_pixel.is_some()).then(|| {
        NativeWindowOracleRenderer::load_from_env().unwrap_or_else(|error| {
            eprintln!("failed to initialize native-window oracle video renderer: {error}");
            process::exit(1);
        })
    });
    use std::time::Instant;
    let stage_timing = std::env::var_os("ZELDA3_SNES9X_TIMING").is_some();
    // [pre_state, poly, run_frame, video, oracle, audio+compare, receipts]
    let mut stage_ns = [0u128; 7];
    let mut stage_mark = Instant::now();
    let stage = |slot: usize, stage_ns: &mut [u128; 7], mark: &mut Instant| {
        if stage_timing {
            let now = Instant::now();
            stage_ns[slot] += now.duration_since(*mark).as_nanos();
            *mark = now;
        }
    };
    // TEMP DIAGNOSTIC: render with the BG-anim CHR region (VRAM 0x3c00..0x3e00)
    // as it was before this frame's step, to test a one-step animation skew.
    let debug_anim_lag = std::env::var_os("ZELDA3_DEBUG_ANIM_LAG").is_some();
    let mut pre_anim_region: Option<Vec<u16>> = None;
    let mut next_paired_resume_capture = 0usize;
    let mut next_rolling_resume_frame = rolling_paired_resume
        .as_ref()
        .map(|rolling| rolling_capture_frame_after(start_frame, rolling.interval));
    // Oracle-seeded start: Some(boundary) until Rust has been seeded.
    let mut seed_pending = seed_rust_from_oracle;
    let seed_warmup_max = env::var("ZELDA3_SEED_WARMUP_MAX")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(240);
    for frame_index in start_frame..frames {
        let mut stop_after_exact_audio_mismatch = false;
        let mut video_mismatch_this_frame = false;
        while paired_resume_captures
            .get(next_paired_resume_capture)
            .is_some_and(|capture| capture.frame == frame_index)
        {
            if !video_mismatch_ranges.is_empty() || wrote_first_audio_mismatch {
                eprintln!(
                    "refusing to save paired resume at frame {frame_index} after an earlier parity mismatch"
                );
                process::exit(1);
            }
            let capture = &paired_resume_captures[next_paired_resume_capture];
            write_paired_resume_capture(
                capture,
                core_path,
                rom_path,
                input_script_path.as_deref(),
                rom_random_script.as_deref(),
                &initial_sram,
                &game,
                &oracle,
                oracle_semantic_trace.as_ref(),
            )
            .unwrap_or_else(|error| {
                eprintln!(
                    "failed to save paired resume at frame {frame_index} in {}: {error}",
                    capture.dir.display()
                );
                process::exit(1);
            });
            println!(
                "saved paired pre-frame resume at frame {frame_index}: {}",
                capture.dir.display()
            );
            next_paired_resume_capture += 1;
        }
        if next_rolling_resume_frame.is_some_and(|due| frame_index >= due)
            && video_mismatch_ranges.is_empty()
            && !wrote_first_audio_mismatch
            && game.paired_resume_cpu_boundary_is_quiescent()
        {
            let rolling = rolling_paired_resume
                .as_ref()
                .expect("rolling resume schedule requires a configuration");
            match write_rolling_paired_resume_capture(
                rolling,
                frame_index,
                core_path,
                rom_path,
                input_script_path.as_deref(),
                rom_random_script.as_deref(),
                &initial_sram,
                &game,
                &oracle,
                oracle_semantic_trace.as_ref(),
            ) {
                Ok(capture_dir) => {
                    println!(
                        "saved rolling paired pre-frame resume at frame {frame_index}: {}",
                        capture_dir.display()
                    );
                    next_rolling_resume_frame =
                        Some(rolling_capture_frame_after(frame_index, rolling.interval));
                }
                Err(error)
                    if error
                        .to_string()
                        .contains("inside a translated call continuation") =>
                {
                    // Rolling captures are best-effort: a frame inside a
                    // suspended original-timing continuation cannot be
                    // checkpointed; retry on the next frame and keep the
                    // last successful capture.
                    next_rolling_resume_frame = Some(frame_index.wrapping_add(1));
                }
                Err(error) => {
                    eprintln!(
                        "failed to save rolling paired resume at frame {frame_index} in {}: {error}",
                        rolling.root.display()
                    );
                    process::exit(1);
                }
            }
        }
        let requested_input = input_script.input_for_frame(frame_index);
        let compare_this_frame = frame_index >= effective_compare_from_frame;
        if let Some(writer) = display_oracle_receipts
            .as_mut()
            .filter(|_| display_oracle_before_frames.contains(&frame_index))
        {
            write_display_oracle_receipt(writer, frame_index, "before", &oracle, &mut game);
        }
        if stage_timing {
            stage_mark = Instant::now();
        }
        // A full per-frame ZeldaState clone (ROM + asset pack + audio state)
        // dominated the comparison loop; the receipt only needs the pre-frame
        // WRAM and one debug value, and the failure-artifact writer only needs
        // the full state while the poly thread is pending.
        let poly_pending = game.ram[0x1f00] != 0;
        let pre_game = poly_pending.then(|| game.clone());
        let pre_ram = game.ram.to_vec();
        let pre_load_remaining_nmi_slices =
            game.zelda_debug_selected_game_load_remaining_nmi_slices();
        stage(0, &mut stage_ns, &mut stage_mark);
        let rust_poly_cycles: Option<u64> = None;
        let mut early_oracle_capture = None;
        if replay_save.is_none()
            && (oracle_semantic_trace.is_some() || live_oracle_rng_trace.is_some())
        {
            if oracle_preframe_snapshot_required(frame_index, frames, compare_this_frame) {
                oracle
                    .serialize_state_into(&mut oracle_before_state)
                    .unwrap_or_else(|error| {
                        eprintln!(
                            "failed to serialize {oracle_name} before frame {frame_index}: {error}"
                        );
                        process::exit(1);
                    });
            }
            early_oracle_capture = Some(oracle.run_frame_with_input(requested_input));
            if let Some(trace) = oracle_semantic_trace.as_mut() {
                let oracle_wram = oracle.memory_bytes(RETRO_MEMORY_SYSTEM_RAM);
                let dialogue_message_read_position = oracle_wram
                    .and_then(|ram| ram.get(0x1cd9..0x1cdb))
                    .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]));
                let spotlight_var4_low_at_return = oracle_wram
                    .and_then(|ram| {
                        ram.get(crate::snes9x_semantic_receipts::SPOTLIGHT_VAR4_LOW_ADDRESS)
                    })
                    .copied();
                let spotlight_lower_cursor_at_return = oracle_wram
                    .and_then(|ram| {
                        ram.get(
                            crate::snes9x_semantic_receipts::SPOTLIGHT_LOWER_CURSOR_ADDRESS
                                ..crate::snes9x_semantic_receipts::SPOTLIGHT_LOWER_CURSOR_ADDRESS
                                    + 2,
                        )
                    })
                    .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]));
                let mut semantic = trace
                    .read_after_host_call(
                        dialogue_message_read_position,
                        spotlight_var4_low_at_return,
                        spotlight_lower_cursor_at_return,
                    )
                    .unwrap_or_else(|error| {
                    eprintln!(
                        "failed to read pinned-Snes9x semantic receipts at frame {frame_index}: {error}"
                    );
                    process::exit(1);
                    });
                let nmi_acceptance_ppu_register_operands =
                    trace.take_host_nmi_ppu_register_operands();
                semantic.extend(snes9x_oracle_semantic_receipts(&oracle).unwrap_or_else(
                    |error| {
                    eprintln!(
                        "failed to decode pinned-Snes9x semantic receipts at frame {frame_index}: {error}"
                    );
                    process::exit(1);
                    },
                ));
                let receipts = snes9x_original_timing_host_receipts(
                    &oracle,
                    early_oracle_capture
                        .as_ref()
                        .expect("Snes9x host capture precedes semantic receipt decoding"),
                    previous_oracle_video.as_ref(),
                    &mut presented_bg_tilemap_cache,
                    frame_index,
                    requested_input,
                    semantic,
                    nmi_acceptance_ppu_register_operands,
                )
                .unwrap_or_else(|error| {
                    eprintln!(
                        "failed to decode pinned-Snes9x host receipts at frame {frame_index}: {error}"
                    );
                    process::exit(1);
                });
                if let Some(seed_boundary) = seed_pending {
                    // Warm-up run: the oracle advanced alone. Seed Rust at the
                    // first clean boundary; drain this run's RNG samples.
                    let semantic = receipts.semantic();
                    // ZELDA3_DEBUG_WARMUP_RECEIPTS=1 prints every warm-up
                    // host's semantic vector: the oracle-only wire for a
                    // window Rust cannot reach yet (the Triforce-room load
                    // at route hosts 1557656-1557724 was read this way).
                    if env::var_os("ZELDA3_DEBUG_WARMUP_RECEIPTS").is_some() {
                        eprintln!(
                            "[WARMUP-RCPT] host_call={frame_index} pub_pending={} semantic={semantic:?}",
                            trace.nmi_publication_pending()
                        );
                    }
                    let clean = !trace.nmi_publication_pending()
                        && semantic.last()
                            == Some(&OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted)
                        && semantic.iter().any(|receipt| {
                            matches!(
                                receipt,
                                OriginalTimingSemanticReceipt::MainLoopProgress(
                                    zelda3::MainLoopProgress::IterationStarted
                                )
                            )
                        });
                    previous_oracle_video = early_oracle_capture
                        .as_ref()
                        .map(PresentedOracleVideo::from);
                    if let Some(rng) = live_oracle_rng_trace.as_mut() {
                        let _ = rng
                            .samples_for_run(frame_index - start_frame, frame_index)
                            .unwrap_or_else(|error| {
                                eprintln!(
                                    "live oracle RNG authority failed at warm-up frame {frame_index}: {error}"
                                );
                                process::exit(1);
                            });
                    }
                    input_history.push((frame_index, requested_input));
                    completed_frames = frame_index.saturating_add(1);
                    if clean {
                        let seed_frame = frame_index.saturating_add(1);
                        seed_rust_game_from_oracle_memory(&mut game, &oracle, seed_frame)
                            .unwrap_or_else(|error| {
                                eprintln!("failed to seed Rust from the oracle state: {error}");
                                process::exit(2);
                            });
                        trace.end_seed_warmup();
                        seed_pending = None;
                        compare_engine_state_from_frame =
                            compare_engine_state_from_frame.map(|start| start.max(seed_frame));
                        println!(
                            "seeded Rust from the {oracle_name} state's memory at frame {seed_frame} after {} warm-up run(s): {semantic:?}",
                            seed_frame - seed_boundary
                        );
                    } else if frame_index - seed_boundary + 1 >= seed_warmup_max {
                        eprintln!(
                            "oracle-seeded start found no clean run boundary within {seed_warmup_max} frame(s) of {seed_boundary}; last run: {semantic:?}"
                        );
                        process::exit(2);
                    }
                    continue;
                }
                let semantic_debug = format!("{:?}", receipts.semantic());
                game.install_original_timing_host_receipts(receipts)
                .unwrap_or_else(|error| {
                    eprintln!(
                        "failed to install pinned-Snes9x host receipt at frame {frame_index}: {error:?} semantic={semantic_debug}"
                    );
                    process::exit(1);
                });
                previous_oracle_video = early_oracle_capture
                    .as_ref()
                    .map(PresentedOracleVideo::from);
            }
            if let Some(trace) = live_oracle_rng_trace.as_mut() {
                let samples = trace
                    .samples_for_run(frame_index - start_frame, frame_index)
                    .unwrap_or_else(|error| {
                        eprintln!(
                            "live oracle RNG authority failed at frame {frame_index}: {error}"
                        );
                        process::exit(1);
                    });
                game.install_rom_random_replay(samples, frame_index);
            }
        }
        stage(1, &mut stage_ns, &mut stage_mark);
        if replay_save.is_some() {
            game.zelda_run_frame_with_replay_input_override(requested_input as i32, None);
        } else {
            game.zelda_run_frame(requested_input as i32);
        }
        let input = if replay_save.is_some() {
            game.state_recorder.last_inputs
        } else {
            requested_input
        };
        if oracle_semantic_trace.is_some()
            && game.last_consumed_original_timing_host_call() != Some(u64::from(frame_index))
        {
            eprintln!(
                "translated host call {frame_index} did not consume its pinned-Snes9x receipt"
            );
            process::exit(1);
        }
        if compare_engine_state_from_frame.is_some_and(|start| frame_index >= start) {
            let oracle_ram = oracle
                .memory_bytes(RETRO_MEMORY_SYSTEM_RAM)
                .unwrap_or_default();
            if engine_state_dump_frames.is_some_and(|(lo, hi)| (lo..=hi).contains(&frame_index)) {
                let word = |ram: &[u8], address: usize| {
                    u16::from_le_bytes([
                        ram.get(address).copied().unwrap_or_default(),
                        ram.get(address + 1).copied().unwrap_or_default(),
                    ])
                };
                let byte =
                    |ram: &[u8], address: usize| ram.get(address).copied().unwrap_or_default();
                eprintln!(
                    "[ENGINE] f={frame_index} rust main={:02x} sub={:02x} subsub={:02x} fc={:02x} y={:04x} x={:04x} ysub={:02x} yvel={:02x} bg2={:04x},{:04x} | oracle main={:02x} sub={:02x} subsub={:02x} fc={:02x} y={:04x} x={:04x} ysub={:02x} yvel={:02x} bg2={:04x},{:04x}",
                    byte(&game.ram, 0x10),
                    byte(&game.ram, 0x11),
                    byte(&game.ram, 0xb0),
                    byte(&game.ram, 0x1a),
                    word(&game.ram, 0x20),
                    word(&game.ram, 0x22),
                    byte(&game.ram, 0x2a),
                    byte(&game.ram, 0x27),
                    word(&game.ram, 0xe2),
                    word(&game.ram, 0xe8),
                    byte(oracle_ram, 0x10),
                    byte(oracle_ram, 0x11),
                    byte(oracle_ram, 0xb0),
                    byte(oracle_ram, 0x1a),
                    word(oracle_ram, 0x20),
                    word(oracle_ram, 0x22),
                    byte(oracle_ram, 0x2a),
                    byte(oracle_ram, 0x27),
                    word(oracle_ram, 0xe2),
                    word(oracle_ram, 0xe8),
                );
                if let Some(slot) = engine_state_dump_sprite {
                    eprintln!(
                        "[ENGINE-SPR] f={frame_index} slot={slot} rust st={:02x} ty={:02x} delay={:02x} ai={:02x} x={:02x}{:02x} y={:02x}{:02x} | oracle st={:02x} ty={:02x} delay={:02x} ai={:02x} x={:02x}{:02x} y={:02x}{:02x}",
                        byte(&game.ram, 0x0dd0 + slot),
                        byte(&game.ram, 0x0e20 + slot),
                        byte(&game.ram, 0x0df0 + slot),
                        byte(&game.ram, 0x0d80 + slot),
                        byte(&game.ram, 0x0d30 + slot),
                        byte(&game.ram, 0x0d10 + slot),
                        byte(&game.ram, 0x0d20 + slot),
                        byte(&game.ram, 0x0d00 + slot),
                        byte(oracle_ram, 0x0dd0 + slot),
                        byte(oracle_ram, 0x0e20 + slot),
                        byte(oracle_ram, 0x0df0 + slot),
                        byte(oracle_ram, 0x0d80 + slot),
                        byte(oracle_ram, 0x0d30 + slot),
                        byte(oracle_ram, 0x0d10 + slot),
                        byte(oracle_ram, 0x0d20 + slot),
                        byte(oracle_ram, 0x0d00 + slot),
                    );
                }
                if !engine_state_dump_bytes.is_empty() {
                    let cells: Vec<String> = engine_state_dump_bytes
                        .iter()
                        .map(|&address| {
                            format!(
                                "{address:05x}={:02x}|{:02x}",
                                byte(&game.ram, address),
                                byte(oracle_ram, address),
                            )
                        })
                        .collect();
                    eprintln!(
                        "[ENGINE-BYTES] f={frame_index} rust|oracle {}",
                        cells.join(" ")
                    );
                }
            }
            let mismatches = compact_engine_state_mismatches(&game.ram, oracle_ram);
            if !mismatches.is_empty() {
                if engine_state_scan_all {
                    engine_state_divergences.push((frame_index, mismatches));
                } else {
                    eprintln!(
                        "engine-state divergence at frame {frame_index}: {}",
                        mismatches.join(", ")
                    );
                    input_history.push((frame_index, input));
                    completed_frames = frame_index.saturating_add(1);
                    first_engine_state_mismatch = Some((frame_index, mismatches));
                    break;
                }
            }
        }
        if live_oracle_rng {
            if let Err(error) = game.finish_rom_random_replay_through(frame_index.saturating_add(1))
            {
                if std::env::var_os("ZELDA3_DEBUG_ROM_RANDOM_FRAME_DRIFT").is_some() {
                    // Diagnostic mode: report the first overdue frame and keep
                    // replaying so the late consumer names its callsite in a
                    // rom_random_frame_drift line.
                    if !rom_random_overdue_reported {
                        rom_random_overdue_reported = true;
                        eprintln!(
                            "live oracle RNG call-count divergence at frame {frame_index}: {error}"
                        );
                    }
                } else {
                    eprintln!(
                        "live oracle RNG call-count divergence at frame {frame_index}: {error}"
                    );
                    process::exit(1);
                }
            }
        }
        input_history.push((frame_index, input));
        stage(2, &mut stage_ns, &mut stage_mark);
        // The production display route retains hardware-visible history
        // (notably OBJ evaluation) across scanouts. Compose every boundary,
        // but avoid submitting thousands of discarded cold-warmup frames to
        // the GPU. A one-second raster tail primes the native surface for any
        // scanout-retention rule at the comparison boundary.
        let video_requested = trace_video_pixel.is_some() || compare_video;
        let render_video_this_frame =
            should_render_video_frame(frame_index, effective_compare_from_frame, video_requested);
        let rendered_rust_frame = render_video_this_frame.then(|| {
            let restored = if debug_anim_lag {
                pre_anim_region.as_ref().map(|prev| {
                    let cur = game.ppu.vram[0x3c00..0x3e00].to_vec();
                    game.ppu.vram[0x3c00..0x3e00].copy_from_slice(prev);
                    cur
                })
            } else {
                None
            };
            let frame_result = native_window_video
                .as_mut()
                .expect("native window renderer allocated for libretro video comparison")
                .render_game_rgba_with_capture(&mut game, frame_index);
            let rendered = frame_result.unwrap_or_else(|error| {
                eprintln!("native-window oracle video render failed: {error}");
                process::exit(1);
            });
            if let Some(cur) = restored {
                game.ppu.vram[0x3c00..0x3e00].copy_from_slice(&cur);
            }
            rendered
        });
        let rust_video_frame = rendered_rust_frame.as_ref().map(|(frame, _)| frame);
        let rust_rendered_display = rendered_rust_frame.as_ref().map(|(_, display)| display);
        if video_requested && !render_video_this_frame {
            game.advance_display_publication_history();
        }
        if debug_anim_lag {
            pre_anim_region = Some(game.ppu.vram[0x3c00..0x3e00].to_vec());
        }
        stage(3, &mut stage_ns, &mut stage_mark);
        let ports = game.zelda_debug_apu_write_ports();
        if trace_poly_sched {
            eprintln!(
                "poly frame={frame_index} main={:02x} sub={:02x} subsub={:02x} fc={:02x} step={:02x} timer={:02x} iframe={:02x} did={:02x} flag={:02x} defer={} started={} sched={:02x} hold={} phase={} alt={} cfg={:02x} a={:02x} b={:02x} tri0=({:02x}{:02x},{:02x}{:02x};{:02x},{:02x}) tri1=({:02x}{:02x},{:02x}{:02x};{:02x},{:02x}) tri2=({:02x}{:02x},{:02x}{:02x};{:02x},{:02x})",
                game.ram[0x10],
                game.ram[0x11],
                game.ram[0xb0],
                game.ram[0x1a],
                game.ram[0x1e00],
                game.ram[0x1e01],
                game.ram[0x1e0a],
                game.ram[0x1f00],
                game.ram[0x1f0c],
                game.debug_nmi_poly_upload_deferred(),
                game.debug_nmi_poly_upload_started(),
                game.debug_snes9x_poly_scheduler_counter(),
                game.debug_snes9x_hold_intro_step_this_frame(),
                game.debug_snes9x_intro_step_carry_phase_active(),
                game.debug_snes9x_intro_step_hold_alternate(),
                game.ram[0x1f02],
                game.ram[0x1f04],
                game.ram[0x1f05],
                game.ram[0x1e38],
                game.ram[0x1e30],
                game.ram[0x1e50],
                game.ram[0x1e48],
                game.ram[0x1e58],
                game.ram[0x1e60],
                game.ram[0x1e39],
                game.ram[0x1e31],
                game.ram[0x1e51],
                game.ram[0x1e49],
                game.ram[0x1e59],
                game.ram[0x1e61],
                game.ram[0x1e3a],
                game.ram[0x1e32],
                game.ram[0x1e52],
                game.ram[0x1e4a],
                game.ram[0x1e5a],
                game.ram[0x1e62],
            );
        }
        let mut capture = if let Some(capture) = early_oracle_capture {
            debug_assert_eq!(input, requested_input);
            capture
        } else {
            if oracle_preframe_snapshot_required(frame_index, frames, compare_this_frame) {
                oracle
                    .serialize_state_into(&mut oracle_before_state)
                    .unwrap_or_else(|e| {
                        eprintln!(
                            "failed to serialize {oracle_name} before frame {frame_index}: {e}"
                        );
                        process::exit(1);
                    });
            }
            oracle.run_frame_with_input(input)
        };
        if let Some(writer) = display_oracle_receipts.as_mut().filter(|_| {
            capture_all_display_oracle || display_oracle_after_frames.contains(&frame_index)
        }) {
            write_display_oracle_receipt(writer, frame_index, "after", &oracle, &mut game);
        }
        if let Some(writer) = obj_state_ledger.as_mut() {
            let receipt = capture_obj_state_ledger_receipt(frame_index, &oracle, &mut game)
                .unwrap_or_else(|| {
                    eprintln!(
                        "OBJ state ledger could not read instrumented PPU/WRAM/VRAM at frame {frame_index}"
                    );
                    process::exit(2);
                });
            serde_json::to_writer(&mut *writer, &receipt).unwrap_or_else(|error| {
                eprintln!("failed to write OBJ state ledger: {error}");
                process::exit(1);
            });
            writeln!(writer).unwrap_or_else(|error| {
                eprintln!("failed to terminate OBJ state ledger receipt: {error}");
                process::exit(1);
            });
            writer.flush().unwrap_or_else(|error| {
                eprintln!("failed to flush OBJ state ledger receipt: {error}");
                process::exit(1);
            });
            if !wrote_first_obj_cache_divergence && !receipt.presented_cache_is_exact() {
                let dir = session_dir
                    .as_deref()
                    .expect("OBJ state ledger requires a session directory");
                eprintln!(
                    "OBJ state ledger first presented-cache divergence at frame {frame_index}: mismatched_valid_pixels={} first_cache_index={:?}",
                    receipt
                        .presented_obj_tile_cache
                        .difference
                        .mismatched_values,
                    receipt.presented_obj_tile_cache.difference.first_mismatch,
                );
                let mut detail = BufWriter::new(
                    fs::File::create(dir.join("obj_state_first_cache_divergence.jsonl"))
                        .unwrap_or_else(|error| {
                            eprintln!("failed to create first OBJ divergence detail: {error}");
                            process::exit(1);
                        }),
                );
                write_display_oracle_receipt(&mut detail, frame_index, "after", &oracle, &mut game);
                fs::write(dir.join("obj_state_first_rust_wram.bin"), &game.ram).unwrap_or_else(
                    |error| {
                        eprintln!("failed to dump first OBJ-divergence Rust WRAM: {error}");
                        process::exit(1);
                    },
                );
                fs::write(
                    dir.join("obj_state_first_oracle_wram.bin"),
                    oracle
                        .memory_bytes(RETRO_MEMORY_SYSTEM_RAM)
                        .unwrap_or_default(),
                )
                .unwrap_or_else(|error| {
                    eprintln!("failed to dump first OBJ-divergence oracle WRAM: {error}");
                    process::exit(1);
                });
                fs::write(
                    dir.join("obj_state_first_oracle_vram.bin"),
                    oracle
                        .memory_bytes(RETRO_MEMORY_VIDEO_RAM)
                        .unwrap_or_default(),
                )
                .unwrap_or_else(|error| {
                    eprintln!("failed to dump first OBJ-divergence oracle VRAM: {error}");
                    process::exit(1);
                });
                let rust_vram = game
                    .ppu
                    .vram
                    .iter()
                    .flat_map(|word| word.to_le_bytes())
                    .collect::<Vec<_>>();
                fs::write(dir.join("obj_state_first_rust_vram.bin"), rust_vram).unwrap_or_else(
                    |error| {
                        eprintln!("failed to dump first OBJ-divergence Rust VRAM: {error}");
                        process::exit(1);
                    },
                );
                wrote_first_obj_cache_divergence = true;
            }
        }
        // Libretro frame numbering starts at one; keep this artifact aligned
        // with `scripts/snes9x_boot_contract.py` rather than the CLI's
        // zero-based input index.
        let contract_frame = frame_index.saturating_add(1);
        let rust_boundary = BootBoundaryState::from_ram(contract_frame, "after", &game.ram);
        if let Some(trace) = rust_boot_contract.as_mut() {
            serde_json::to_writer(&mut *trace, &rust_boundary).unwrap_or_else(|error| {
                eprintln!("failed to write Rust boot-contract frame: {error}");
                process::exit(1);
            });
            writeln!(trace).unwrap_or_else(|error| {
                eprintln!("failed to terminate Rust boot-contract frame: {error}");
                process::exit(1);
            });
        }
        if assert_oracle_boot_contract {
            let oracle_ram = oracle
                .memory_bytes(RETRO_MEMORY_SYSTEM_RAM)
                .unwrap_or_else(|| {
                    eprintln!("{oracle_name} did not expose WRAM for boot-contract comparison");
                    process::exit(1);
                });
            let oracle_boundary = BootBoundaryState::from_ram(contract_frame, "after", oracle_ram);
            if let Some((field, rust, oracle)) = rust_boundary.first_difference(&oracle_boundary) {
                eprintln!(
                    "oracle_boot_contract_divergence frame={contract_frame} stage=after field={field} rust={rust:02x} oracle={oracle:02x}"
                );
                process::exit(1);
            }
        }
        if assert_oracle_vram {
            let oracle_vram = oracle
                .memory_bytes(RETRO_MEMORY_VIDEO_RAM)
                .unwrap_or_else(|| {
                    eprintln!("{oracle_name} did not expose VRAM for parity probe");
                    process::exit(1);
                });
            let rust_vram = game
                .ppu
                .vram
                .iter()
                .flat_map(|word| word.to_le_bytes())
                .collect::<Vec<_>>();
            if debug_vram_frames.contains(&frame_index) {
                let changes = |previous: Option<&Vec<u8>>, current: &[u8]| {
                    previous
                        .into_iter()
                        .flat_map(|previous| {
                            previous
                                .iter()
                                .zip(current)
                                .enumerate()
                                .filter(|(_, (before, after))| before != after)
                                .map(|(byte, (before, after))| (byte, *before, *after))
                        })
                        .collect::<Vec<_>>()
                };
                let rust_changes = changes(previous_rust_vram.as_ref(), &rust_vram);
                let oracle_changes = changes(previous_oracle_vram.as_ref(), oracle_vram);
                eprintln!(
                    "oracle_vram_writes frame={frame_index} rust_count={} oracle_count={} rust_first={:?} oracle_first={:?}",
                    rust_changes.len(),
                    oracle_changes.len(),
                    &rust_changes[..rust_changes.len().min(24)],
                    &oracle_changes[..oracle_changes.len().min(24)],
                );
            }
            let range_start = assert_oracle_vram_range
                .as_ref()
                .map_or(0, |range| range.start);
            let range_end = assert_oracle_vram_range
                .as_ref()
                .map_or(rust_vram.len().min(oracle_vram.len()), |range| range.end)
                .min(rust_vram.len())
                .min(oracle_vram.len());
            if let Some((byte, (&rust, &oracle))) = rust_vram
                .iter()
                .zip(oracle_vram.iter())
                .enumerate()
                .skip(range_start)
                .take(range_end.saturating_sub(range_start))
                .find(|(_, (rust, oracle))| rust != oracle)
            {
                eprintln!(
                    "oracle_vram_divergence frame={frame_index} byte={byte:04x} word={:04x} rust={rust:02x} oracle={oracle:02x}",
                    byte / 2,
                );
                process::exit(1);
            }
            if rust_vram.len() != oracle_vram.len() {
                eprintln!(
                    "oracle_vram_length_divergence frame={frame_index} rust={} oracle={}",
                    rust_vram.len(),
                    oracle_vram.len(),
                );
                process::exit(1);
            }
            previous_oracle_vram = Some(oracle_vram.to_vec());
            previous_rust_vram = Some(rust_vram);
        }
        stage(4, &mut stage_ns, &mut stage_mark);
        if debug_wram_frames.contains(&frame_index) {
            let Some(dir) = session_dir.as_deref() else {
                eprintln!("ZELDA3_DEBUG_WRAM_FRAMES requires --session-dir");
                process::exit(2);
            };
            let oracle_ram = oracle
                .memory_bytes(RETRO_MEMORY_SYSTEM_RAM)
                .unwrap_or_else(|| {
                    eprintln!("{oracle_name} did not expose system RAM for WRAM capture");
                    process::exit(1);
                });
            fs::write(
                dir.join(format!("rust_wram_frame_{frame_index}.bin")),
                &game.ram,
            )
            .unwrap_or_else(|error| {
                eprintln!("failed to write Rust WRAM capture: {error}");
                process::exit(1);
            });
            fs::write(
                dir.join(format!("oracle_wram_frame_{frame_index}.bin")),
                oracle_ram,
            )
            .unwrap_or_else(|error| {
                eprintln!("failed to write oracle WRAM capture: {error}");
                process::exit(1);
            });
            let live_oam = game
                .ppu
                .oam
                .iter()
                .flat_map(|word| word.to_le_bytes())
                .collect::<Vec<_>>();
            fs::write(
                dir.join(format!("rust_live_oam_frame_{frame_index}.bin")),
                live_oam,
            )
            .unwrap_or_else(|error| {
                eprintln!("failed to write Rust live OAM capture: {error}");
                process::exit(1);
            });
            let displayed_oam = game.with_display_snapshot(|snapshot| {
                snapshot
                    .ppu
                    .oam
                    .iter()
                    .flat_map(|word| word.to_le_bytes())
                    .collect::<Vec<_>>()
            });
            fs::write(
                dir.join(format!("rust_displayed_oam_frame_{frame_index}.bin")),
                displayed_oam,
            )
            .unwrap_or_else(|error| {
                eprintln!("failed to write Rust displayed OAM capture: {error}");
                process::exit(1);
            });
            let displayed_vram = game.with_display_snapshot(|snapshot| {
                snapshot
                    .ppu
                    .vram
                    .iter()
                    .flat_map(|word| word.to_le_bytes())
                    .collect::<Vec<_>>()
            });
            fs::write(
                dir.join(format!("rust_displayed_vram_frame_{frame_index}.bin")),
                displayed_vram,
            )
            .unwrap_or_else(|error| {
                eprintln!("failed to write Rust displayed VRAM capture: {error}");
                process::exit(1);
            });
            let live_vram = game
                .ppu
                .vram
                .iter()
                .flat_map(|word| word.to_le_bytes())
                .collect::<Vec<_>>();
            fs::write(
                dir.join(format!("rust_live_vram_frame_{frame_index}.bin")),
                live_vram,
            )
            .unwrap_or_else(|error| {
                eprintln!("failed to write Rust live VRAM capture: {error}");
                process::exit(1);
            });
            let oracle_state = oracle.serialize_state().unwrap_or_else(|error| {
                eprintln!("failed to serialize oracle debug state: {error}");
                process::exit(1);
            });
            fs::write(
                dir.join(format!("oracle_state_frame_{frame_index}.state")),
                oracle_state,
            )
            .unwrap_or_else(|error| {
                eprintln!("failed to write oracle debug state: {error}");
                process::exit(1);
            });
            let rust_checkpoint = PlayCrashCheckpoint {
                magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
                host_frame: frame_index.saturating_add(1),
                input,
                run_what: RUN_MAIN,
                game: game.clone(),
            };
            fs::write(
                dir.join(format!("rust_state_frame_{frame_index}.z3state")),
                bincode::serialize(&rust_checkpoint).unwrap_or_else(|error| {
                    eprintln!("failed to serialize Rust debug state: {error}");
                    process::exit(1);
                }),
            )
            .unwrap_or_else(|error| {
                eprintln!("failed to write Rust debug state: {error}");
                process::exit(1);
            });
        }
        if debug_sprite_frames.contains(&frame_index) {
            // Follower/tagalong tracking probe: slot-0 sprite state both sides.
            let oracle_ram = oracle.memory_bytes(RETRO_MEMORY_SYSTEM_RAM).unwrap_or(&[]);
            let g = |a: usize| game.ram.get(a).copied().unwrap_or(0);
            let o = |a: usize| oracle_ram.get(a).copied().unwrap_or(0xff);
            // Slot 0's full motion/AI witness. Keep this aligned with the
            // `sprite_slots` receipt below so a visual failure can be walked
            // backward without adding another bespoke trace field.
            let f = |a: usize| (g(a), o(a), if g(a) != o(a) { "*" } else { "" });
            eprintln!(
                "sprite_probe frame={frame_index} xlo={:?} xhi={:?} xsub={:?} xvel={:?} ylo={:?} yhi={:?} ysub={:?} yvel={:?} dir={:?} state={:?} type={:?} ai={:?} wall={:?} subtype={:?} subtype2={:?} delay={:?} aux1={:?} main={:02x}/{:02x} sub={:02x}/{:02x}",
                f(0xd10), f(0xd30), f(0xd70), f(0xd50), f(0xd00), f(0xd20), f(0xd60),
                f(0xd40), f(0xde0), f(0xdd0), f(0xe20), f(0xd80), f(0xe70), f(0xe30),
                f(0xe80), f(0xdf0), f(0xe00),
                g(0x10), o(0x10), g(0x11), o(0x11),
            );
        }
        if debug_text_frames.contains(&frame_index) {
            // Typewriter-cadence probe: end-frame messaging state on both sides.
            // 0x1cd8 module / 0x1cd9 read_pos / 0x1cd4 render state /
            // 0x1cd5 line speed counter / 0x1cd6.
            let oracle_ram = oracle.memory_bytes(RETRO_MEMORY_SYSTEM_RAM).unwrap_or(&[]);
            let fc_oracle = if oracle_ram.len() > 0x1a {
                oracle_ram[0x1a]
            } else {
                0xff
            };
            let mo = |a: usize| {
                if oracle_ram.len() > a {
                    oracle_ram[a]
                } else {
                    0xff
                }
            };
            let mow = |a: usize| {
                if oracle_ram.len() > a + 1 {
                    u16::from_le_bytes([oracle_ram[a], oracle_ram[a + 1]])
                } else {
                    0xffff
                }
            };
            let rp_r = u16::from_le_bytes([game.ram[0x1cd9], game.ram[0x1cda]]);
            eprintln!(
                "text_probe frame={frame_index} fc_r={:02x} fc_o={fc_oracle:02x}{} rst_r={:02x} rst_o={:02x} rp_r={rp_r:04x} rp_o={:04x} coreDis_r={:02x} coreDis_o={:02x}",
                game.ram[0x1a],
                if game.ram[0x1a] != fc_oracle { "  FCd" } else { "" },
                game.ram[0x1cd4], mo(0x1cd4),
                mow(0x1cd9),
                game.ram[0x1ccd], mo(0x1ccd),
            );
        }
        if debug_vram_frames.contains(&frame_index) {
            // Palette-phase probe: end-frame WRAM main palette buffer entry 23 on
            // both sides, plus our displayed (snapshot-composed) CGRAM entry 23.
            let oracle_ram = oracle.memory_bytes(RETRO_MEMORY_SYSTEM_RAM).unwrap_or(&[]);
            let ours = u16::from_le_bytes([game.ram[0xc500 + 46], game.ram[0xc500 + 47]]);
            let theirs = if oracle_ram.len() > 0xc52f {
                u16::from_le_bytes([oracle_ram[0xc500 + 46], oracle_ram[0xc500 + 47]])
            } else {
                0xdead
            };
            let displayed = game.with_display_snapshot(|snapshot| snapshot.ppu.cgram[23]);
            let thread_rust = game.ram[0x12a];
            let thread_oracle = if oracle_ram.len() > 0x12a {
                oracle_ram[0x12a]
            } else {
                0xff
            };
            eprintln!(
                "palette_probe frame={frame_index} buffer23 rust={ours:04x} oracle={theirs:04x} displayed_cgram23={displayed:04x} live_cgram23={:04x} thread12a rust={thread_rust:02x} oracle={thread_oracle:02x}",
                game.ppu.cgram[23]
            );
        }
        if debug_vram_frames.contains(&frame_index) {
            let Some(dir) = session_dir.as_deref() else {
                eprintln!("ZELDA3_DEBUG_VRAM_FRAMES requires --session-dir");
                process::exit(2);
            };
            let rust_vram = game
                .ppu
                .vram
                .iter()
                .flat_map(|word| word.to_le_bytes())
                .collect::<Vec<_>>();
            fs::write(
                dir.join(format!("rust_vram_frame_{frame_index}.bin")),
                &rust_vram,
            )
            .unwrap_or_else(|error| {
                eprintln!("failed to write Rust VRAM capture: {error}");
                process::exit(1);
            });
            // The DISPLAYED generation: the compose snapshot VRAM, which is
            // what the renderer scans out (may differ from the live post-frame
            // VRAM above).
            let snap_vram = game.with_display_snapshot(|display| {
                display
                    .ppu
                    .vram
                    .iter()
                    .flat_map(|word| word.to_le_bytes())
                    .collect::<Vec<_>>()
            });
            fs::write(
                dir.join(format!("rust_snapvram_frame_{frame_index}.bin")),
                &snap_vram,
            )
            .unwrap_or_else(|error| {
                eprintln!("failed to write Rust snapshot VRAM capture: {error}");
                process::exit(1);
            });
            let obj_vram = game.with_display_snapshot(|display| {
                display
                    .ppu
                    .obj_vram_latch
                    .as_deref()
                    .unwrap_or(&display.ppu.vram)
                    .iter()
                    .flat_map(|word| word.to_le_bytes())
                    .collect::<Vec<_>>()
            });
            fs::write(
                dir.join(format!("rust_objvram_frame_{frame_index}.bin")),
                &obj_vram,
            )
            .unwrap_or_else(|error| {
                eprintln!("failed to write Rust OBJ VRAM capture: {error}");
                process::exit(1);
            });
            let oracle_vram = oracle
                .memory_bytes(RETRO_MEMORY_VIDEO_RAM)
                .unwrap_or_else(|| {
                    eprintln!("{oracle_name} did not expose VRAM for capture");
                    process::exit(1);
                });
            fs::write(
                dir.join(format!("oracle_vram_frame_{frame_index}.bin")),
                oracle_vram,
            )
            .unwrap_or_else(|error| {
                eprintln!("failed to write oracle VRAM capture: {error}");
                process::exit(1);
            });
        }
        if std::env::var("ZELDA3_DEBUG_SCANLINES_FRAME")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            == Some(frame_index)
        {
            let displayed_summary = game.with_display_snapshot(|snapshot| {
                crate::render_diagnostics::format_render_ppu_summary(snapshot)
            });
            eprintln!("ppu_summary_displayed {displayed_summary}");
            eprintln!(
                "ppu_summary_live {}",
                crate::render_diagnostics::format_render_ppu_summary(&game)
            );
            let windows = game.ppu_scanline_windows();
            let fixed = game.ppu_scanline_fixed_color();
            for line in 0..224usize {
                let (w1l, w1r, w2l, w2r, tm, _, _, _, blank) = windows[line];
                let (fr, fg, fb) = fixed[line];
                eprintln!(
                    "scanline {line}: w1=({w1l},{w1r}) w2=({w2l},{w2r}) tm={tm:02x} fixed=({fr},{fg},{fb}) blank={blank}"
                );
            }
        }
        if trace_shield_dma {
            let oracle_ram = oracle
                .memory_bytes(RETRO_MEMORY_SYSTEM_RAM)
                .unwrap_or_else(|| {
                    eprintln!("{oracle_name} did not expose system RAM for shield DMA tracing");
                    process::exit(1);
                });
            let rust_index = game.ram[0x0108];
            let oracle_index = oracle_ram[0x0108];
            let rust_source = u16::from_le_bytes([game.ram[0x0ac4], game.ram[0x0ac5]]);
            let oracle_source = u16::from_le_bytes([oracle_ram[0x0ac4], oracle_ram[0x0ac5]]);
            let rust_sword_index = game.ram[0x0107];
            let oracle_sword_index = oracle_ram[0x0107];
            let rust_sword_source = u16::from_le_bytes([game.ram[0x0ac2], game.ram[0x0ac3]]);
            let oracle_sword_source = u16::from_le_bytes([oracle_ram[0x0ac2], oracle_ram[0x0ac3]]);
            let trace = (
                rust_index,
                oracle_index,
                rust_source,
                oracle_source,
                rust_sword_index,
                oracle_sword_index,
                rust_sword_source,
                oracle_sword_source,
            );
            if previous_shield_dma_trace != Some(trace) {
                eprintln!(
                    "shield-dma frame={frame_index} input={input:04x} main={:02x}/{:02x} shield-index={rust_index:02x}/{oracle_index:02x} shield-source={rust_source:04x}/{oracle_source:04x} sword-index={rust_sword_index:02x}/{oracle_sword_index:02x} sword-source={rust_sword_source:04x}/{oracle_sword_source:04x} facing={:02x}/{:02x} pose={:02x}/{:02x} step={:04x}/{:04x} shield={:02x}/{:02x} progress={:02x}/{:02x}",
                    game.ram[0x10],
                    game.ram[0x11],
                    game.ram[0x002f],
                    oracle_ram[0x002f],
                    game.ram[0x0354],
                    oracle_ram[0x0354],
                    u16::from_le_bytes([game.ram[0x0076], game.ram[0x0077]]),
                    u16::from_le_bytes([oracle_ram[0x0076], oracle_ram[0x0077]]),
                    game.ram[0xf35a],
                    oracle_ram[0xf35a],
                    game.ram[0xf3c5],
                    oracle_ram[0xf3c5],
                );
                previous_shield_dma_trace = Some(trace);
            }
            let uncle_trace = (
                game.ram[0x0dd0],
                oracle_ram[0x0dd0],
                game.ram[0x0de0],
                oracle_ram[0x0de0],
                game.ram[0x0dc0],
                oracle_ram[0x0dc0],
                game.ram[0x0d80],
                oracle_ram[0x0d80],
            );
            if previous_uncle_trace != Some(uncle_trace) {
                eprintln!(
                    "uncle-state frame={frame_index} state={:02x}/{:02x} direction={:02x}/{:02x} graphics={:02x}/{:02x} ai={:02x}/{:02x}",
                    uncle_trace.0,
                    uncle_trace.1,
                    uncle_trace.2,
                    uncle_trace.3,
                    uncle_trace.4,
                    uncle_trace.5,
                    uncle_trace.6,
                    uncle_trace.7,
                );
                previous_uncle_trace = Some(uncle_trace);
            }
        }
        if let (Some(writer), Some(writes)) = (debug_dsp_writes.as_mut(), oracle.debug_dsp_writes())
        {
            let music = oracle
                .memory_bytes(RETRO_MEMORY_SYSTEM_RAM)
                .and_then(oracle_music_route_state)
                .unwrap_or_else(|| {
                    eprintln!(
                        "{oracle_name} did not expose enough system RAM for the DSP-write trace"
                    );
                    process::exit(1);
                });
            let dsp_clock = oracle.debug_dsp_frame_clock();
            serde_json::to_writer(
                &mut *writer,
                &serde_json::json!({
                    "frame": frame_index,
                    "audio_sample_frames": capture.audio.len() / 2,
                    "dsp_clock": dsp_clock,
                    "music": [
                        music[0],
                        music[1],
                        music[2],
                    ],
                    "dsp_write_events": writes,
                }),
            )
            .unwrap();
            writer.write_all(b"\n").unwrap();
            writer.flush().unwrap();
        }
        if let (Some(writer), Some(writes)) = (
            debug_dsp_register_writes.as_mut(),
            oracle.debug_dsp_register_writes(),
        ) {
            let apu_port_writes = oracle.debug_apu_port_writes();
            serde_json::to_writer(
                &mut *writer,
                &serde_json::json!({
                    "frame": frame_index,
                    "audio_sample_frames": capture.audio.len() / 2,
                    "dsp_write_events": writes,
                    "apu_port_writes": apu_port_writes,
                }),
            )
            .unwrap();
            writer.write_all(b"\n").unwrap();
            writer.flush().unwrap();
        }
        if !debug_smp_bootstrap_complete {
            if let Some(writer) = debug_smp_bootstrap.as_mut() {
                let output_writes = oracle.debug_smp_output_port_writes().unwrap_or_else(|| {
                    eprintln!("SMP-bootstrap trace requires the current instrumented Snes9x core");
                    process::exit(2);
                });
                let cpu_accesses = oracle.debug_apu_port_writes().unwrap_or_else(|| {
                    eprintln!(
                        "SMP-bootstrap trace requires Snes9x CPU-side APU-port instrumentation"
                    );
                    process::exit(2);
                });
                let cpu_timing_transactions = oracle
                    .debug_cpu_timing_transactions()
                    .unwrap_or_else(|error| {
                        eprintln!("failed to capture Snes9x CPU timing transactions: {error}");
                        process::exit(2);
                    })
                    .unwrap_or_else(|| {
                        eprintln!(
                            "SMP-bootstrap trace requires Snes9x CPU timing transaction instrumentation"
                        );
                        process::exit(2);
                    });
                debug_smp_bootstrap_output_writes.extend(output_writes);
                debug_smp_bootstrap_cpu_accesses.extend(cpu_accesses.into_iter().map(|access| {
                    FramedApuPortAccess {
                        frame: frame_index,
                        access,
                    }
                }));
                debug_smp_bootstrap_cpu_timing_transactions.extend(
                    cpu_timing_transactions.into_iter().map(|transaction| {
                        FramedCpuTimingTransaction {
                            frame: frame_index,
                            transaction,
                        }
                    }),
                );
                append_smp_instruction_frame(
                    &mut debug_smp_bootstrap_instructions,
                    oracle.debug_smp_instructions().unwrap_or_else(|| {
                        eprintln!(
                            "SMP-bootstrap trace requires Snes9x instruction instrumentation"
                        );
                        process::exit(2);
                    }),
                );

                if let Some(handoff_index) =
                    smp_bootstrap_handoff_index(&debug_smp_bootstrap_instructions)
                {
                    let handoff_cycle =
                        debug_smp_bootstrap_instructions[handoff_index + 1].absolute_cycle;
                    debug_smp_bootstrap_output_writes
                        .retain(|write| write.absolute_cycle <= handoff_cycle);
                    let cpu_end = debug_smp_bootstrap_cpu_accesses
                        .iter()
                        .position(|framed| {
                            let access = &framed.access;
                            !access.is_read
                                && access.port == 3
                                && access.value == 0
                                && (access.program_counter & 0xffff) == 0x88ff
                        })
                        .map(|index| index + 1)
                        .unwrap_or_else(|| {
                            eprintln!("SMP reached $0800 without Zelda's final $88fc STZ $2143");
                            process::exit(1);
                        });
                    debug_smp_bootstrap_cpu_accesses.truncate(cpu_end);
                    let final_cpu_access = &debug_smp_bootstrap_cpu_accesses[cpu_end - 1];
                    let cpu_timing_end = debug_smp_bootstrap_cpu_timing_transactions
                        .iter()
                        .position(|framed| {
                            let transaction = &framed.transaction;
                            framed.frame == final_cpu_access.frame
                                && transaction.kind == 2
                                && (transaction.origin_pc & 0xffff) == 0x88fc
                                && transaction.opcode == 0x9c
                                && transaction.start_v_counter
                                    == final_cpu_access.access.v_counter
                                && transaction.start_cpu_cycle
                                    == final_cpu_access.access.cpu_cycle
                        })
                        .map(|index| index + 1)
                        .unwrap_or_else(|| {
                            eprintln!(
                                "SMP reached $0800 without a timing receipt for the final $88fc STZ $2143"
                            );
                            process::exit(1);
                        });
                    debug_smp_bootstrap_cpu_timing_transactions.truncate(cpu_timing_end);
                    let first_cc = debug_smp_bootstrap_output_writes
                        .iter()
                        .find(|event| event.port == 0 && event.value == 0xcc)
                        .unwrap_or_else(|| {
                            eprintln!("SMP handoff trace has no initial CC write");
                            process::exit(1);
                        });
                    let instruction_sequence = compact_smp_bootstrap_instruction_sequence(
                        &debug_smp_bootstrap_instructions,
                        first_cc,
                    )
                    .unwrap_or_else(|error| {
                        eprintln!("failed to compact Snes9x SMP-bootstrap trace: {error}");
                        process::exit(1);
                    });
                    let cpu_milestones = debug_smp_bootstrap_cpu_accesses
                        .iter()
                        .take(4)
                        .chain(
                            debug_smp_bootstrap_cpu_accesses
                                .iter()
                                .skip(4)
                                .filter(|framed| {
                                    let access = &framed.access;
                                    (access.program_counter & 0xffff) == 0x88ef
                                        && access.value == 0xcc
                                })
                                .take(1),
                        )
                        .chain(debug_smp_bootstrap_cpu_accesses.iter().rev().take(8).rev())
                        .map(|framed| {
                            serde_json::json!({
                                "frame": framed.frame,
                                "access": &framed.access,
                            })
                        })
                        .collect::<Vec<_>>();
                    let output_milestones = debug_smp_bootstrap_output_writes
                        .iter()
                        .take(3)
                        .chain(debug_smp_bootstrap_output_writes.iter().rev().take(3).rev())
                        .copied()
                        .collect::<Vec<_>>();
                    serde_json::to_writer(
                        &mut *writer,
                        &serde_json::json!({
                            "kind": "bootstrap-events",
                            "final_frame": frame_index,
                            "first_cc_acknowledged": true,
                            "final_ipl_handoff": {
                                "absolute_cycle": handoff_cycle,
                                "origin_pc": 0xfffb,
                                "opcode": 0x1f,
                                "target_pc": 0x0800,
                            },
                            "cpu_apu_access_milestones": cpu_milestones,
                            "cpu_apu_access_sequence": compact_cpu_apu_accesses(
                                &debug_smp_bootstrap_cpu_accesses,
                            ),
                            "cpu_timing_transaction_kinds": {
                                "0": "fast_pcbase_opcode_fetch_non_draining",
                                "1": "cpuops_add_cycles_draining",
                                "2": "getset_memory_access_after_semantic_draining",
                                "3": "getset_memory_access_x2_after_semantic_draining",
                            },
                            "cpu_timing_transaction_sequence": compact_cpu_timing_transactions(
                                &debug_smp_bootstrap_cpu_timing_transactions,
                            ),
                            "smp_output_port_write_milestones": output_milestones,
                            "smp_output_port_write_sequence": compact_smp_output_port_writes(
                                &debug_smp_bootstrap_output_writes,
                            ),
                            "smp_instruction_boundary_sequence": instruction_sequence,
                        }),
                    )
                    .unwrap();
                    writer.write_all(b"\n").unwrap();
                    writer.flush().unwrap();
                    debug_smp_bootstrap_complete = true;
                }
            }
        }
        if !debug_smp_first_nmi_complete {
            if let Some(writer) = debug_smp_first_nmi.as_mut() {
                let output_writes = oracle.debug_smp_output_port_writes().unwrap_or_else(|| {
                    eprintln!(
                        "post-handoff first-NMI trace requires the current instrumented Snes9x core"
                    );
                    process::exit(2);
                });
                let cpu_accesses = oracle.debug_apu_port_writes().unwrap_or_else(|| {
                    eprintln!(
                        "post-handoff first-NMI trace requires CPU-side APU-port instrumentation"
                    );
                    process::exit(2);
                });
                let cpu_timing_transactions = oracle
                    .debug_cpu_timing_transactions()
                    .unwrap_or_else(|error| {
                        eprintln!("failed to capture Snes9x CPU timing transactions: {error}");
                        process::exit(2);
                    })
                    .unwrap_or_else(|| {
                        eprintln!(
                            "post-handoff first-NMI trace requires CPU timing instrumentation"
                        );
                        process::exit(2);
                    });
                let smp_instructions = oracle.debug_smp_instructions().unwrap_or_else(|| {
                    eprintln!(
                        "post-handoff first-NMI trace requires SMP instruction instrumentation"
                    );
                    process::exit(2);
                });
                debug_smp_first_nmi_output_writes.extend(output_writes);
                debug_smp_first_nmi_cpu_accesses.extend(cpu_accesses.into_iter().map(|access| {
                    FramedApuPortAccess {
                        frame: frame_index,
                        access,
                    }
                }));
                debug_smp_first_nmi_cpu_timing_transactions.extend(
                    cpu_timing_transactions.into_iter().map(|transaction| {
                        FramedCpuTimingTransaction {
                            frame: frame_index,
                            transaction,
                        }
                    }),
                );
                append_framed_smp_instruction_frame(
                    &mut debug_smp_first_nmi_instructions,
                    frame_index,
                    smp_instructions,
                );

                if debug_smp_first_nmi_anchor.is_none() {
                    if let Some(handoff_index) =
                        framed_smp_bootstrap_handoff_index(&debug_smp_first_nmi_instructions)
                    {
                        let handoff_cycle = debug_smp_first_nmi_instructions[handoff_index + 1]
                            .instruction
                            .absolute_cycle;
                        let cpu_end = debug_smp_first_nmi_cpu_accesses
                            .iter()
                            .position(|framed| {
                                let access = &framed.access;
                                !access.is_read
                                    && access.port == 3
                                    && access.value == 0
                                    && (access.program_counter & 0xffff) == 0x88ff
                            })
                            .map(|index| index + 1)
                            .unwrap_or_else(|| {
                                eprintln!(
                                    "SMP reached $0800 without Zelda's final $88fc STZ $2143"
                                );
                                process::exit(1);
                            });
                        let final_cpu_access =
                            debug_smp_first_nmi_cpu_accesses[cpu_end - 1].clone();
                        let cpu_timing_end = debug_smp_first_nmi_cpu_timing_transactions
                            .iter()
                            .position(|framed| {
                                let transaction = &framed.transaction;
                                framed.frame == final_cpu_access.frame
                                    && transaction.kind == 2
                                    && (transaction.origin_pc & 0xffff) == 0x88fc
                                    && transaction.opcode == 0x9c
                                    && transaction.start_v_counter
                                        == final_cpu_access.access.v_counter
                                    && transaction.start_cpu_cycle
                                        == final_cpu_access.access.cpu_cycle
                            })
                            .map(|index| index + 1)
                            .unwrap_or_else(|| {
                                let candidates = debug_smp_first_nmi_cpu_timing_transactions
                                    .iter()
                                    .filter(|framed| {
                                        (framed.transaction.origin_pc & 0xffff) == 0x88fc
                                    })
                                    .map(|framed| {
                                        serde_json::json!({
                                            "frame": framed.frame,
                                            "transaction": framed.transaction,
                                        })
                                    })
                                    .collect::<Vec<_>>();
                                eprintln!(
                                    "SMP reached $0800 without the final $88fc CPU timing anchor; \
                                     final access={:?}; $88fc candidates={}",
                                    final_cpu_access,
                                    serde_json::to_string(&candidates).unwrap()
                                );
                                process::exit(1);
                            });
                        let final_cpu_timing_transaction =
                            debug_smp_first_nmi_cpu_timing_transactions[cpu_timing_end - 1];
                        debug_smp_first_nmi_cpu_accesses.drain(..cpu_end);
                        debug_smp_first_nmi_cpu_timing_transactions.drain(..cpu_timing_end);
                        debug_smp_first_nmi_output_writes
                            .retain(|write| write.absolute_cycle > handoff_cycle);
                        debug_smp_first_nmi_instructions.drain(..=handoff_index);
                        debug_smp_first_nmi_anchor = Some(SmpPostHandoffAnchor {
                            handoff_cycle,
                            final_cpu_access,
                            final_cpu_timing_transaction,
                        });
                    }
                }

                if let Some(anchor) = debug_smp_first_nmi_anchor.as_ref() {
                    if let Some(cpu_access_index) =
                        debug_smp_first_nmi_cpu_accesses.iter().position(|framed| {
                            let access = &framed.access;
                            access.is_read
                                && access.port == 0
                                && (access.program_counter & 0x00ff_ffff) == 0x0080e4
                        })
                    {
                        let first_nmi_access =
                            debug_smp_first_nmi_cpu_accesses[cpu_access_index].clone();
                        let cpu_timing_index = debug_smp_first_nmi_cpu_timing_transactions
                            .iter()
                            .position(|framed| {
                                let transaction = &framed.transaction;
                                framed.frame == first_nmi_access.frame
                                    && transaction.kind == 2
                                    && (transaction.origin_pc & 0x00ff_ffff) == 0x0080e1
                                    && transaction.opcode == 0xad
                                    && transaction.start_v_counter
                                        == first_nmi_access.access.v_counter
                                    && transaction.start_cpu_cycle
                                        == first_nmi_access.access.cpu_cycle
                            })
                            .unwrap_or_else(|| {
                                eprintln!(
                                    "first $8080e1 APU read has no completed kind-2 timing transaction"
                                );
                                process::exit(1);
                            });
                        debug_smp_first_nmi_cpu_accesses.truncate(cpu_access_index + 1);
                        debug_smp_first_nmi_cpu_timing_transactions.truncate(cpu_timing_index + 1);

                        let nmi_enable_index = debug_smp_first_nmi_cpu_timing_transactions
                            .iter()
                            .position(|framed| {
                                framed.transaction.kind == 2
                                    && (framed.transaction.origin_pc & 0x00ff_ffff) == 0x008031
                                    && framed.transaction.opcode == 0x8d
                            })
                            .unwrap_or_else(|| {
                                eprintln!("post-handoff trace never executed $8031 STA $4200");
                                process::exit(1);
                            });
                        let nmi_entry_index = debug_smp_first_nmi_cpu_timing_transactions
                            .iter()
                            .position(|framed| {
                                framed.transaction.kind == 0
                                    && (framed.transaction.origin_pc & 0x00ff_ffff) == 0x0080c9
                                    && framed.transaction.opcode == 0x78
                            })
                            .unwrap_or_else(|| {
                                eprintln!(
                                    "post-handoff trace never entered the first NMI at $80c9"
                                );
                                process::exit(1);
                            });
                        if !(nmi_enable_index < nmi_entry_index
                            && nmi_entry_index < cpu_timing_index)
                        {
                            eprintln!(
                                "post-handoff NMI-enable, entry, and APUI timing receipts are out of order"
                            );
                            process::exit(1);
                        }

                        let first_hmax_index = debug_smp_first_nmi_cpu_timing_transactions
                            .iter()
                            .position(|framed| {
                                framed.transaction.end_v_counter
                                    != framed.transaction.start_v_counter
                            })
                            .unwrap_or_else(|| {
                                eprintln!(
                                    "post-handoff trace has no HMax crossing before first NMI"
                                );
                                process::exit(1);
                            });
                        let (smp_before_index, smp_after_index) =
                            smp_instruction_bracket_for_apu_access(
                                &debug_smp_first_nmi_instructions,
                                &first_nmi_access,
                            )
                            .unwrap_or_else(|error| {
                                eprintln!(
                                    "failed to bracket first-NMI APU synchronization: {error}"
                                );
                                process::exit(1);
                            });
                        let smp_terminal_successor = debug_smp_first_nmi_instructions
                            .get(smp_after_index + 1)
                            .cloned()
                            .unwrap_or_else(|| {
                                eprintln!(
                                    "first-NMI SMP owner boundary has no successor/end-cycle receipt"
                                );
                                process::exit(1);
                            });
                        let smp_stop_cycle = smp_terminal_successor.instruction.absolute_cycle;
                        debug_smp_first_nmi_output_writes
                            .retain(|write| write.absolute_cycle <= smp_stop_cycle);
                        debug_smp_first_nmi_instructions.truncate(smp_after_index + 1);
                        let smp_boundary_digest =
                            framed_smp_instruction_digest(&debug_smp_first_nmi_instructions);
                        let smp_boundary_sequence =
                            compact_framed_smp_instructions(&debug_smp_first_nmi_instructions);

                        serde_json::to_writer(
                            &mut *writer,
                            &serde_json::json!({
                                "kind": "post-handoff-first-nmi",
                                "start_anchor": {
                                    "final_ipl_handoff": {
                                        "absolute_cycle": anchor.handoff_cycle,
                                        "origin_pc": 0xfffb,
                                        "opcode": 0x1f,
                                        "target_pc": 0x0800,
                                    },
                                    "final_cpu_apu_access": {
                                        "frame": anchor.final_cpu_access.frame,
                                        "access": &anchor.final_cpu_access.access,
                                    },
                                    "final_cpu_timing_transaction": {
                                        "frame": anchor.final_cpu_timing_transaction.frame,
                                        "transaction": &anchor.final_cpu_timing_transaction.transaction,
                                    },
                                },
                                "nmi_enable_source": {
                                    "load_immediate_origin_pc": 0x00802f,
                                    "load_immediate_opcode": 0xa9,
                                    "immediate_value": 0x81,
                                    "store_origin_pc": 0x008031,
                                    "store_opcode": 0x8d,
                                    "absolute_address": 0x4200,
                                    "rom_bytes": [0xa9, 0x81, 0x8d, 0x00, 0x42],
                                    "timing_transaction": {
                                        "frame": debug_smp_first_nmi_cpu_timing_transactions[nmi_enable_index].frame,
                                        "transaction": &debug_smp_first_nmi_cpu_timing_transactions[nmi_enable_index].transaction,
                                    },
                                },
                                "first_nmi_entry_transaction": {
                                    "frame": debug_smp_first_nmi_cpu_timing_transactions[nmi_entry_index].frame,
                                    "transaction": &debug_smp_first_nmi_cpu_timing_transactions[nmi_entry_index].transaction,
                                },
                                "first_hmax_crossing_transaction": {
                                    "frame": debug_smp_first_nmi_cpu_timing_transactions[first_hmax_index].frame,
                                    "transaction": &debug_smp_first_nmi_cpu_timing_transactions[first_hmax_index].transaction,
                                },
                                "first_nmi_apui_read": {
                                    "instruction_origin_pc": 0x0080e1,
                                    "instruction_opcode": 0xad,
                                    "absolute_address": 0x2140,
                                    "frame": first_nmi_access.frame,
                                    "access": &first_nmi_access.access,
                                    "completed_timing_transaction": &debug_smp_first_nmi_cpu_timing_transactions[cpu_timing_index].transaction,
                                },
                                "cpu_apu_access_sequence": compact_cpu_apu_accesses(
                                    &debug_smp_first_nmi_cpu_accesses,
                                ),
                                "cpu_timing_transaction_sequence": compact_cpu_timing_transactions(
                                    &debug_smp_first_nmi_cpu_timing_transactions,
                                ),
                                "smp_output_port_write_sequence": compact_smp_output_port_writes(
                                    &debug_smp_first_nmi_output_writes,
                                ),
                                "smp_instruction_boundary_sequence": smp_boundary_sequence,
                                "smp_instruction_boundaries": {
                                    "fields_in_digest": framed_smp_instruction_digest_fields(),
                                    "count": debug_smp_first_nmi_instructions.len(),
                                    "expanded_sha256": smp_boundary_digest,
                                    "absolute_end_cycle": smp_terminal_successor.instruction.absolute_cycle,
                                    "terminal_successor": &smp_terminal_successor,
                                    "before_sync": &debug_smp_first_nmi_instructions[smp_before_index],
                                    "after_sync": &debug_smp_first_nmi_instructions[smp_after_index],
                                },
                                "stop_reason": "first_$8080e1_apui0_read_semantic_and_kind2_timing_complete",
                            }),
                        )
                        .unwrap();
                        writer.write_all(b"\n").unwrap();
                        writer.flush().unwrap();
                        debug_smp_first_nmi_complete = true;
                    }
                }
            }
        }
        if !debug_first_nmi_dma_setup_complete {
            if let Some(writer) = debug_first_nmi_dma_setup.as_mut() {
                let cpu_accesses = oracle.debug_apu_port_writes().unwrap_or_else(|| {
                    eprintln!(
                        "first-NMI DMA-setup trace requires CPU-side APU-port instrumentation"
                    );
                    process::exit(2);
                });
                let cpu_timing_transactions = oracle
                    .debug_cpu_timing_transactions()
                    .unwrap_or_else(|error| {
                        eprintln!("failed to capture Snes9x CPU timing transactions: {error}");
                        process::exit(2);
                    })
                    .unwrap_or_else(|| {
                        eprintln!("first-NMI DMA-setup trace requires CPU timing instrumentation");
                        process::exit(2);
                    });
                debug_first_nmi_dma_setup_cpu_accesses.extend(cpu_accesses.into_iter().map(
                    |access| FramedApuPortAccess {
                        frame: frame_index,
                        access,
                    },
                ));
                debug_first_nmi_dma_setup_cpu_timing_transactions.extend(
                    cpu_timing_transactions.into_iter().map(|transaction| {
                        FramedCpuTimingTransaction {
                            frame: frame_index,
                            transaction,
                        }
                    }),
                );

                if debug_first_nmi_dma_setup_anchor.is_none() {
                    match first_nmi_apui_anchor_indices(
                        &debug_first_nmi_dma_setup_cpu_accesses,
                        &debug_first_nmi_dma_setup_cpu_timing_transactions,
                    ) {
                        Ok(Some((access_index, transaction_index))) => {
                            let access =
                                debug_first_nmi_dma_setup_cpu_accesses[access_index].clone();
                            let completed_timing_transaction =
                                debug_first_nmi_dma_setup_cpu_timing_transactions
                                    [transaction_index];
                            if access.frame != 81
                                || access.access.v_counter != 225
                                || access.access.cpu_cycle != 480
                                || access.access.smp_clock_before != 0
                                || access.access.smp_clock_after != 1
                                || completed_timing_transaction.transaction.start_v_counter != 225
                                || completed_timing_transaction.transaction.start_cpu_cycle != 480
                                || completed_timing_transaction.transaction.end_v_counter != 225
                                || completed_timing_transaction.transaction.end_cpu_cycle != 486
                            {
                                eprintln!(
                                    "first-NMI DMA-setup trace does not continue the committed $8080e1 H480->486 / SMP-clock 0->1 anchor: access={:?}; transaction={:?}",
                                    access, completed_timing_transaction
                                );
                                process::exit(1);
                            }
                            debug_first_nmi_dma_setup_cpu_accesses.drain(..=access_index);
                            debug_first_nmi_dma_setup_cpu_timing_transactions
                                .drain(..=transaction_index);
                            debug_first_nmi_dma_setup_anchor = Some(FirstNmiApuAnchor {
                                access,
                                completed_timing_transaction,
                            });
                        }
                        Ok(None) => {}
                        Err(error) => {
                            eprintln!("invalid first-NMI DMA-setup start anchor: {error}");
                            process::exit(1);
                        }
                    }
                }

                if let Some(anchor) = debug_first_nmi_dma_setup_anchor.as_ref() {
                    match first_nmi_dma_setup_stop_index(
                        &debug_first_nmi_dma_setup_cpu_timing_transactions,
                    ) {
                        Ok(Some(stop_index)) => {
                            let excluded_raw_fetch =
                                debug_first_nmi_dma_setup_cpu_timing_transactions[stop_index];
                            debug_first_nmi_dma_setup_cpu_timing_transactions.truncate(stop_index);
                            let final_transaction = debug_first_nmi_dma_setup_cpu_timing_transactions
                                .last()
                                .copied()
                                .unwrap_or_else(|| {
                                    eprintln!(
                                        "first-NMI DMA-setup trace has no setup transaction after the $8080e1 anchor"
                                    );
                                    process::exit(1);
                                });
                            let mut final_instruction_rows =
                                debug_first_nmi_dma_setup_cpu_timing_transactions
                                    .iter()
                                    .rev()
                                    .take_while(|framed| {
                                        framed.transaction.origin_pc
                                            == final_transaction.transaction.origin_pc
                                    })
                                    .collect::<Vec<_>>();
                            final_instruction_rows.reverse();
                            if (final_transaction.transaction.origin_pc & 0x00ff_ffff) != 0x008a33
                                || final_transaction.transaction.opcode != 0xa9
                                || final_instruction_rows.len() != 2
                                || final_instruction_rows[0].transaction.kind != 0
                                || final_instruction_rows[1].transaction.kind != 1
                            {
                                eprintln!(
                                    "first-NMI DMA-setup trace did not stop after the exact $008a33 LDA #$07 fetch/operand pair: {:?}",
                                    final_instruction_rows
                                );
                                process::exit(1);
                            }
                            let final_instruction_receipts = final_instruction_rows
                                .iter()
                                .map(|framed| {
                                    serde_json::json!({
                                        "frame": framed.frame,
                                        "transaction": &framed.transaction,
                                    })
                                })
                                .collect::<Vec<_>>();
                            if debug_first_nmi_dma_setup_cpu_timing_transactions
                                .iter()
                                .any(|framed| {
                                    (framed.transaction.origin_pc & 0x00ff_ffff) == 0x008a35
                                })
                            {
                                eprintln!(
                                    "first-NMI DMA-setup trace retained a $008a35 transaction"
                                );
                                process::exit(1);
                            }

                            serde_json::to_writer(
                                &mut *writer,
                                &serde_json::json!({
                                    "kind": "first-nmi-dma-setup",
                                    "start_anchor": {
                                        "first_nmi_apui_read": {
                                            "instruction_origin_pc": 0x0080e1,
                                            "instruction_opcode": 0xad,
                                            "absolute_address": 0x2140,
                                            "frame": anchor.access.frame,
                                            "access": &anchor.access.access,
                                            "completed_timing_transaction": &anchor.completed_timing_transaction.transaction,
                                        },
                                    },
                                    "final_completed_setup_instruction": {
                                        "frame": final_transaction.frame,
                                        "origin_pc": 0x008a33,
                                        "opcode": 0xa9,
                                        "rom_bytes": [0xa9, 0x07],
                                        "ordered_timing_transactions": final_instruction_receipts,
                                        "completed_timing_transaction": &final_transaction.transaction,
                                    },
                                    "stop_before_instruction": {
                                        "origin_pc": 0x008a35,
                                        "opcode": 0x8d,
                                        "absolute_address": 0x420b,
                                        "rom_bytes": [0x8d, 0x0b, 0x42],
                                        "excluded_raw_fetch_transaction": {
                                            "frame": excluded_raw_fetch.frame,
                                            "transaction": &excluded_raw_fetch.transaction,
                                        },
                                    },
                                    "cpu_timing_transaction_sequence": compact_cpu_timing_transactions(
                                        &debug_first_nmi_dma_setup_cpu_timing_transactions,
                                    ),
                                    "stop_reason": "before_$008a35_sta_$420b_raw_fetch_transaction",
                                }),
                            )
                            .unwrap();
                            writer.write_all(b"\n").unwrap();
                            writer.flush().unwrap();
                            debug_first_nmi_dma_setup_complete = true;
                        }
                        Ok(None) => {}
                        Err(error) => {
                            eprintln!("invalid first-NMI DMA-setup stop anchor: {error}");
                            process::exit(1);
                        }
                    }
                }
            }
        }
        if !debug_first_nmi_dma_complete {
            if let Some(writer) = debug_first_nmi_dma.as_mut() {
                let cpu_timing_transactions = oracle
                    .debug_cpu_timing_transactions()
                    .unwrap_or_else(|error| {
                        eprintln!("failed to capture first-NMI DMA CPU timing: {error}");
                        process::exit(2);
                    })
                    .unwrap_or_else(|| {
                        eprintln!("first-NMI DMA trace requires CPU timing instrumentation");
                        process::exit(2);
                    });
                let dma_events = oracle
                    .debug_dma_ledger()
                    .unwrap_or_else(|error| {
                        eprintln!("failed to capture first-NMI DMA ledger: {error}");
                        process::exit(2);
                    })
                    .unwrap_or_else(|| {
                        eprintln!("first-NMI DMA trace requires DMA ledger instrumentation");
                        process::exit(2);
                    });
                let transaction_slice = first_nmi_dma_transaction_slice(&cpu_timing_transactions)
                    .unwrap_or_else(|error| {
                        eprintln!("invalid first-NMI DMA CPU receipt: {error}");
                        process::exit(1);
                    });
                let ledger_slice =
                    first_nmi_dma_ledger_slice(&dma_events).unwrap_or_else(|error| {
                        eprintln!("invalid first-NMI DMA ledger receipt: {error}");
                        process::exit(1);
                    });
                let anchors_agree = transaction_slice.is_some() == ledger_slice.is_some();
                if let (Some((fetch, completion, successor)), Some((outer, events))) =
                    (transaction_slice, ledger_slice)
                {
                    let cpu_receipts = cpu_timing_transactions[fetch..=successor]
                        .iter()
                        .copied()
                        .map(|transaction| FramedCpuTimingTransaction {
                            frame: frame_index,
                            transaction,
                        })
                        .collect::<Vec<_>>();
                    let completion_transaction = cpu_timing_transactions[completion];
                    let successor_transaction = cpu_timing_transactions[successor];
                    if completion_transaction.end_v_counter != successor_transaction.start_v_counter
                        || completion_transaction.end_cpu_cycle
                            != successor_transaction.start_cpu_cycle
                    {
                        eprintln!(
                            "first-NMI DMA outer write completion is not contiguous with the $008a38 successor fetch: completion={completion_transaction:?}; successor={successor_transaction:?}"
                        );
                        process::exit(1);
                    }
                    let before_vram = oracle
                        .debug_dma_vram_snapshot(outer, 0)
                        .unwrap_or_else(|error| {
                            eprintln!("failed to read first-NMI DMA pre-VRAM snapshot: {error}");
                            process::exit(2);
                        })
                        .unwrap_or_else(|| {
                            eprintln!("first-NMI DMA trace requires VRAM snapshot instrumentation");
                            process::exit(2);
                        });
                    let after_vram = oracle
                        .debug_dma_vram_snapshot(outer, 1)
                        .unwrap_or_else(|error| {
                            eprintln!("failed to read first-NMI DMA post-VRAM snapshot: {error}");
                            process::exit(2);
                        })
                        .unwrap_or_else(|| {
                            eprintln!(
                                "first-NMI DMA trace requires a completed post-VRAM snapshot"
                            );
                            process::exit(2);
                        });
                    let bytes = events.iter().filter(|event| event.fields[0] == 2).count();
                    let channel_receipts = events
                        .iter()
                        .filter(|event| event.fields[0] == 3)
                        .map(|event| {
                            serde_json::json!({
                                "channel": event.fields[2],
                                "completed": event.fields[9],
                                "remaining_transfer_bytes": event.fields[63],
                                "final_a_address": event.fields[64],
                                "final_vma_address": event.fields[65],
                                "final_oam_address": event.fields[66],
                                "final_cgram_address": event.fields[67],
                                "final_cgram_flip": event.fields[68],
                                "final_cpu_v": event.fields[10],
                                "final_cpu_h": event.fields[11],
                                "final_apu_reference_time": event.fields[18],
                                "final_apu_remainder": event.fields[19],
                            })
                        })
                        .collect::<Vec<_>>();
                    let hmax_crossings = events
                        .iter()
                        .filter(|event| {
                            event.fields[0] == 2
                                && (event.fields[10] != event.fields[28]
                                    || event.fields[11] > event.fields[29])
                        })
                        .map(|event| {
                            serde_json::json!({
                                "global_byte_ordinal": event.fields[3],
                                "channel": event.fields[2],
                                "before_v": event.fields[10],
                                "before_h": event.fields[11],
                                "after_v": event.fields[28],
                                "after_h": event.fields[29],
                                "before_next_event": event.fields[12],
                                "after_next_event": event.fields[30],
                                "before_apu_reference_time": event.fields[18],
                                "after_apu_reference_time": event.fields[36],
                                "before_smp_clock": event.fields[20],
                                "after_smp_clock": event.fields[38],
                            })
                        })
                        .collect::<Vec<_>>();

                    serde_json::to_writer(
                        &mut *writer,
                        &serde_json::json!({
                            "kind": "first-nmi-dma",
                            "frame": frame_index,
                            "source_instruction": {
                                "origin_pc": 0x008a35,
                                "opcode": 0x8d,
                                "rom_bytes": [0x8d, 0x0b, 0x42],
                                "value": 0x07,
                                "target": 0x420b,
                                "raw_fetch_anchor": &cpu_timing_transactions[fetch],
                                "completed_outer_write_transaction": &completion_transaction,
                                "successor_raw_fetch": &successor_transaction,
                            },
                            "dma": {
                                "outer": outer,
                                "channel_mask": 0x07,
                                "byte_count": bytes,
                                "ordered_event_sequence": compact_dma_ledger(&events),
                                "channel_completion_receipts": channel_receipts,
                                "hmax_crossing_receipts": hmax_crossings,
                            },
                            "cpu_timing_transaction_sequence": compact_cpu_timing_transactions(
                                &cpu_receipts,
                            ),
                            "vram": {
                                "bytes": 0x10000,
                                "before_sha256": parity::evidence::sha256_bytes(&before_vram),
                                "after_sha256": parity::evidence::sha256_bytes(&after_vram),
                                "before_sequence": compact_byte_snapshot(&before_vram),
                                "after_sequence": compact_byte_snapshot(&after_vram),
                            },
                            "stop_reason": "completed_$008a35_sta_$420b_07_and_observed_$008a38_raw_fetch",
                        }),
                    )
                    .unwrap();
                    writer.write_all(b"\n").unwrap();
                    writer.flush().unwrap();
                    debug_first_nmi_dma_complete = true;
                } else if !anchors_agree {
                    eprintln!(
                        "first-NMI DMA CPU and DMA-ledger anchors were not observed in the same retro_run"
                    );
                    process::exit(1);
                }
            }
        }
        if !debug_first_nmi_return_complete {
            if let Some(pending) = debug_first_nmi_return.as_mut() {
                let cpu_transactions = oracle
                    .debug_cpu_timing_transactions()
                    .unwrap_or_else(|error| {
                        eprintln!("failed to capture first-NMI return CPU timing: {error}");
                        process::exit(2);
                    })
                    .unwrap_or_else(|| {
                        eprintln!("first-NMI return trace requires CPU timing instrumentation");
                        process::exit(2);
                    });
                let start =
                    first_nmi_return_start_index(&cpu_transactions).unwrap_or_else(|error| {
                        eprintln!("invalid first-NMI return start anchor: {error}");
                        process::exit(1);
                    });
                if let Some(start) = start {
                    if frame_index != FIRST_NMI_RETURN_HOST_FRAME {
                        eprintln!(
                            "exact $008a38 continuation appeared on host frame {frame_index}, expected {FIRST_NMI_RETURN_HOST_FRAME}"
                        );
                        process::exit(1);
                    }
                    let trace_path = debug_first_nmi_return_core_trace_path
                        .as_deref()
                        .expect("first-NMI return capture configured its core trace path");
                    let trace = read_snes9x_retro_run_trace(trace_path, FIRST_NMI_RETURN_RETRO_RUN)
                        .unwrap_or_else(|error| {
                            eprintln!("invalid first-NMI return core trace: {error}");
                            process::exit(1);
                        })
                        .unwrap_or_else(|| {
                            eprintln!(
                                "first-NMI return core trace has no run {FIRST_NMI_RETURN_RETRO_RUN}"
                            );
                            process::exit(1);
                        });
                    let transaction_slice = &cpu_transactions[start..];
                    let scheduler_gaps =
                        validate_first_nmi_return_cpu_slice(&trace, transaction_slice)
                            .unwrap_or_else(|error| {
                                eprintln!("invalid first-NMI return CPU slice: {error}");
                                process::exit(1);
                            });
                    let terminal = transaction_slice.last().unwrap();
                    let return_v = trace.return_event["v"].as_i64().unwrap_or(-1) as i32;
                    let return_h = trace.return_event["cycles"].as_i64().unwrap_or(-1) as i32;
                    if terminal.end_v_counter != return_v || terminal.end_cpu_cycle != return_h {
                        eprintln!(
                            "first-NMI continuation terminal CPU transaction does not reach the direct retro_run return: terminal={terminal:?}; return=V{return_v}:H{return_h}"
                        );
                        process::exit(1);
                    }

                    let cpu_apui = oracle
                        .debug_apu_port_writes_exact()
                        .unwrap_or_else(|error| {
                            eprintln!(
                                "failed to capture exact first-NMI return APUI receipts: {error}"
                            );
                            process::exit(2);
                        })
                        .unwrap_or_else(|| {
                            eprintln!("first-NMI return trace requires CPU APUI instrumentation");
                            process::exit(2);
                        });
                    let mut ordinal_apui = Vec::new();
                    for access in cpu_apui {
                        let ordinal = cpu_transaction_ordinal_at_start(
                            &cpu_transactions,
                            access.v_counter,
                            access.cpu_cycle,
                            access.program_counter,
                        )
                        .unwrap_or_else(|error| {
                            eprintln!("ambiguous first-NMI return APUI receipt: {error}");
                            process::exit(1);
                        })
                        .unwrap_or_else(|| {
                            eprintln!(
                                "CPU APUI receipt has no source timing transaction: {access:?}"
                            );
                            process::exit(1);
                        });
                        if ordinal >= start {
                            ordinal_apui.push(OrdinalApuPortAccess {
                                cpu_transaction_ordinal: ordinal - start,
                                access,
                            });
                        }
                    }

                    let smp_output_writes = oracle
                        .debug_smp_output_port_writes_exact()
                        .unwrap_or_else(|error| {
                            eprintln!("failed to capture exact first-NMI return SMP output receipts: {error}");
                            process::exit(2);
                        })
                        .unwrap_or_else(|| {
                            eprintln!("first-NMI return trace requires SMP output-port instrumentation");
                            process::exit(2);
                        });
                    let mut ordinal_smp_outputs = Vec::new();
                    for write in smp_output_writes {
                        let ordinal = cpu_transaction_ordinal_containing(
                            &cpu_transactions,
                            write.v_counter,
                            write.cpu_cycle,
                            write.cpu_program_counter,
                        )
                        .unwrap_or_else(|error| {
                            eprintln!("ambiguous first-NMI return SMP output receipt: {error}");
                            process::exit(1);
                        })
                        .unwrap_or_else(|| {
                            eprintln!(
                                "SMP output-port receipt has no containing source timing transaction: {write:?}"
                            );
                            process::exit(1);
                        });
                        if ordinal >= start {
                            ordinal_smp_outputs.push(OrdinalSmpOutputPortWrite {
                                cpu_transaction_ordinal: ordinal - start,
                                write,
                            });
                        }
                    }

                    let dma_events = oracle
                        .debug_dma_ledger()
                        .unwrap_or_else(|error| {
                            eprintln!("failed to capture first-NMI return DMA ledger: {error}");
                            process::exit(2);
                        })
                        .unwrap_or_else(|| {
                            eprintln!("first-NMI return trace requires DMA ledger instrumentation");
                            process::exit(2);
                        });
                    let trailing_dma = trailing_dma_events_after_first_nmi(&dma_events)
                        .unwrap_or_else(|error| {
                            eprintln!("invalid first-NMI return DMA continuation: {error}");
                            process::exit(1);
                        });
                    let mut dma_snapshots = Vec::new();
                    for outer in trailing_dma
                        .iter()
                        .filter(|event| event.fields[0] == 0)
                        .map(|event| event.fields[1])
                    {
                        let before = oracle
                            .debug_dma_vram_snapshot(outer, 0)
                            .unwrap_or_else(|error| {
                                eprintln!(
                                    "failed to read DMA outer {outer} pre-VRAM snapshot: {error}"
                                );
                                process::exit(2);
                            })
                            .unwrap_or_else(|| {
                                eprintln!("DMA outer {outer} has no pre-VRAM snapshot");
                                process::exit(2);
                            });
                        let after = oracle
                            .debug_dma_vram_snapshot(outer, 1)
                            .unwrap_or_else(|error| {
                                eprintln!(
                                    "failed to read DMA outer {outer} post-VRAM snapshot: {error}"
                                );
                                process::exit(2);
                            })
                            .unwrap_or_else(|| {
                                eprintln!("DMA outer {outer} has no post-VRAM snapshot");
                                process::exit(2);
                            });
                        dma_snapshots.push(serde_json::json!({
                            "outer": outer,
                            "before_sha256": parity::evidence::sha256_bytes(&before),
                            "after_sha256": parity::evidence::sha256_bytes(&after),
                            "before_sequence": compact_byte_snapshot(&before),
                            "after_sequence": compact_byte_snapshot(&after),
                        }));
                    }
                    let framed_transactions = transaction_slice
                        .iter()
                        .copied()
                        .map(|transaction| FramedCpuTimingTransaction {
                            frame: FIRST_NMI_RETURN_RECEIPT_FRAME,
                            transaction,
                        })
                        .collect::<Vec<_>>();
                    serde_json::to_writer(
                        &mut pending.writer,
                        &serde_json::json!({
                            "kind": "first-nmi-retro-run-return",
                            "run": FIRST_NMI_RETURN_RETRO_RUN,
                            "host_frame": FIRST_NMI_RETURN_HOST_FRAME,
                            "receipt_frame": FIRST_NMI_RETURN_RECEIPT_FRAME,
                            "start_anchor": transaction_slice[0],
                            "direct_core_trace": {
                                "entry": trace.entry,
                                "return": trace.return_event,
                                "hdma_events": trace.hdma_events,
                                "video_events": trace.video_events,
                                "raw_prefix_sha256": trace.raw_sha256,
                            },
                            "cpu_timing_transaction_sequence": compact_cpu_timing_transactions(&framed_transactions),
                            "cpu_timing_gap_receipts": scheduler_gaps,
                            "cpu_apui_sequence": compact_ordinal_cpu_apu_accesses(&ordinal_apui),
                            "smp_output_port_sequence": compact_ordinal_smp_output_port_writes(&ordinal_smp_outputs),
                            "trailing_dma": {
                                "ordered_event_sequence": compact_dma_ledger(&trailing_dma),
                                "vram_snapshots": dma_snapshots,
                            },
                            "terminal_transaction": terminal,
                            "stop_reason": "direct_pinned_core_retro_run_81_return_after_$008a38_successor",
                        }),
                    )
                    .unwrap();
                    pending.writer.write_all(b"\n").unwrap();
                    pending.install().unwrap_or_else(|error| {
                        eprintln!("failed to install first-NMI return fixture: {error}");
                        process::exit(1);
                    });
                    if let Err(error) = fs::remove_file(trace_path) {
                        eprintln!(
                            "failed to remove temporary first-NMI return core trace {}: {error}",
                            trace_path.display()
                        );
                        process::exit(1);
                    }
                    debug_first_nmi_return_complete = true;
                }
            }
        }
        if let (Some(writer), Some(trace)) = (debug_dsp_globals.as_mut(), oracle.debug_dsp_trace())
        {
            for (sample, values) in trace.iter().take(trace.len().saturating_sub(1)).enumerate() {
                let globals = [values[10], values[11], values[12], values[13]];
                if debug_dsp_globals_previous != Some(globals) {
                    serde_json::to_writer(
                        &mut *writer,
                        &serde_json::json!({
                            "frame": frame_index,
                            "sample": sample,
                            "values": globals,
                        }),
                    )
                    .unwrap();
                    writer.write_all(b"\n").unwrap();
                    debug_dsp_globals_previous = Some(globals);
                }
            }
            writer.flush().unwrap();
        }
        if std::env::var("ZELDA3_DEBUG_DSP_TRACE_FRAME")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            == Some(frame_index)
        {
            if let (Some(dir), Some(trace)) = (session_dir.as_deref(), oracle.debug_dsp_trace()) {
                fs::write(
                    dir.join(format!("oracle_dsp_trace_frame_{frame_index}.json")),
                    serde_json::to_vec(&trace).unwrap(),
                )
                .unwrap();
            }
            if let (Some(dir), Some(samples)) = (session_dir.as_deref(), oracle.debug_dsp_samples())
            {
                fs::write(
                    dir.join(format!("oracle_dsp_samples_frame_{frame_index}.json")),
                    serde_json::to_vec(&samples).unwrap(),
                )
                .unwrap();
            }
        }
        let sample_frames = capture.audio.len() / 2;
        let debug_dsp_trace_frame = debug_dsp_trace_frames.binary_search(&frame_index).is_ok();
        if debug_dsp_trace_frame || debug_spc_clock_witness {
            game.zelda_begin_spc_driver_instruction_trace();
        }
        let rust_echo_ring_before = debug_dsp_trace_frame.then(|| {
            let modern_audio_state = game.zelda_modern_audio_state();
            let (left, right) = modern_audio_state.1.echo_debug_ring();
            (left.to_vec(), right.to_vec())
        });
        let rust_staged_output_before = debug_dsp_trace_frame.then(|| {
            game.zelda_modern_audio_state()
                .1
                .debug_staged_output_components()
        });
        let rust_modern_voices_before = game.zelda_modern_audio_voice_debug_states();
        let rust_event_frame;
        if sample_frames != 0 {
            last_sample_frames = sample_frames;
            rust_audio.resize(capture.audio.len(), 0);
            dsp_writes.clear();
            if let Some(apu) = native_apu.as_mut() {
                let frame_start_cycle = apu.cycles;
                for (port, value) in ports.into_iter().enumerate() {
                    apu.write_snes_port(port as u8, value);
                }
                render_full_apu_audio_exact(apu, &mut rust_audio, sample_frames, 2).unwrap_or_else(
                    |error| {
                        eprintln!("native APU render failed at frame {frame_index}: {error}");
                        process::exit(1);
                    },
                );
                if let (Some(writer), Some(trace)) = (
                    debug_native_apu_dsp_writes.as_mut(),
                    apu.debug_dsp_write_trace.as_mut(),
                ) {
                    for (apu_cycle, address, value) in trace.drain(..) {
                        serde_json::to_writer(
                            &mut *writer,
                            &serde_json::json!({
                                "frame": frame_index,
                                "frame_cycle": apu_cycle.wrapping_sub(frame_start_cycle),
                                "frame_sample_floor": apu_cycle
                                    .wrapping_sub(frame_start_cycle) / 32,
                                "apu_cycle": apu_cycle,
                                "address": address,
                                "value": value,
                            }),
                        )
                        .unwrap();
                        writer.write_all(b"\n").unwrap();
                    }
                    writer.flush().unwrap();
                }
                discard_audio.resize(capture.audio.len(), 0);
                rust_event_frame =
                    game.zelda_render_audio(&mut discard_audio, sample_frames as i32, 2);
            } else {
                for _ in 0..lead_rust_audio_blocks {
                    game.zelda_render_audio(&mut rust_audio, sample_frames as i32, 2);
                }
                rust_event_frame =
                    game.zelda_render_audio(&mut rust_audio, sample_frames as i32, 2);
            }
        } else {
            rust_audio.clear();
            dsp_writes.clear();
            discard_audio.resize(last_sample_frames.saturating_mul(2), 0);
            rust_event_frame =
                game.zelda_render_audio(&mut discard_audio, last_sample_frames as i32, 2);
        }
        game.zelda_discard_unused_audio_frames();
        let rust_spc_instruction_trace = (debug_dsp_trace_frame || debug_spc_clock_witness)
            .then(|| game.zelda_take_spc_driver_instruction_trace())
            .flatten();
        if debug_spc_clock_witness {
            let oracle_instructions = oracle.debug_smp_instructions().unwrap_or_else(|| {
                eprintln!(
                    "SPC clock witness requires an instrumented oracle core with SMP tracing"
                );
                process::exit(2);
            });
            let oracle_polls: Vec<_> = oracle_instructions
                .iter()
                .filter(|instruction| instruction.program_counter == 0x0879)
                .map(|instruction| (instruction.output_sample, instruction.y))
                .collect();
            eprintln!("spc_poll_witness oracle_frame={frame_index} polls={oracle_polls:?}");
            let rust_instructions = rust_spc_instruction_trace
                .as_ref()
                .map(|(_, instructions)| instructions.as_slice())
                .unwrap_or_default();
            let witness = last_spc_clock_witness(rust_instructions, &oracle_instructions);
            let phase = witness.map(|witness| witness.phase_delta);
            if previous_spc_clock_phase != Some(phase) {
                match witness {
                    Some(witness) => eprintln!(
                        "spc_clock_witness frame={frame_index} phase_delta={} pc={:04x} opcode={:02x} rust_tail={} oracle_tail={} rust_divider={} oracle_divider={}",
                        witness.phase_delta,
                        witness.pc,
                        witness.opcode,
                        witness.rust_tail,
                        witness.oracle_tail,
                        witness.rust_timer_divider,
                        witness.oracle_timer_divider,
                    ),
                    None => eprintln!(
                        "spc_clock_witness frame={frame_index} unavailable: no common tail instruction"
                    ),
                }
                previous_spc_clock_phase = Some(phase);
            }
        }
        let rust_stats = AudioFrameStats::from_interleaved_stereo(&rust_audio);
        let oracle_stats = AudioFrameStats::from_interleaved_stereo(&capture.audio);
        let av_audio_hashes = (compare_this_frame && compare_audio).then(|| {
            serde_json::json!({
                "rust": canonical_audio_digest(&rust_audio),
                "oracle": canonical_audio_digest(&capture.audio),
            })
        });
        if compare_this_frame && env::var_os("ZELDA3_DEBUG_DSP_EVENT_PARITY").is_some() {
            let oracle_writes = oracle.debug_dsp_register_writes().unwrap_or_else(|| {
                eprintln!(
                    "DSP-event parity requires an instrumented oracle core with register-write tracing"
                );
                process::exit(2);
            });
            let rust_writes = rust_event_frame
                .events
                .iter()
                .filter_map(|event| event.parity_dsp)
                .collect::<Vec<_>>();
            if let Some(index) = first_dsp_write_timing_mismatch(&rust_writes, &oracle_writes) {
                eprintln!(
                    "DSP-event divergence at frame {frame_index} write {index}: rust={:?} oracle={:?}",
                    rust_writes.get(index),
                    oracle_writes.get(index),
                );
                process::exit(2);
            }
        }
        if debug_dsp_trace_frame {
            if let Some(dir) = session_dir.as_deref() {
                let modern_audio_state = game.zelda_modern_audio_state();
                let receipt = serde_json::json!({
                    "rust_audio_sequencer": &modern_audio_state.0,
                    "rust_audio_sequence_stats": game.zelda_modern_audio_sequence_last_stats(),
                    "oracle_trace": oracle.debug_dsp_trace(),
                    "oracle_dsp_samples": oracle.debug_dsp_samples(),
                    "oracle_dsp_register_writes": oracle.debug_dsp_register_writes(),
                    "oracle_apu_port_writes": oracle.debug_apu_port_writes(),
                    "oracle_smp_output_port_writes": oracle.debug_smp_output_port_writes(),
                    "oracle_smp_instructions": oracle.debug_smp_instructions(),
                    "oracle_audio": capture.audio,
                    "rust_audio": rust_audio,
                    "rust_voice_samples": modern_audio_state.1.debug_voice_samples(),
                    "rust_voice_gains": modern_audio_state.1.debug_voice_gains(),
                    "rust_voice_positions": modern_audio_state.1.debug_voice_positions(),
                    "rust_voice_pitch_words": modern_audio_state.1.debug_voice_pitch_words(),
                    "rust_dsp_global_counter": modern_audio_state.1.debug_dsp_global_counter(),
                    "rust_dsp_rendered_samples": modern_audio_state
                        .1
                        .debug_dsp_rendered_samples(),
                    "rust_checkpoint_sample_offset": modern_audio_state
                        .1
                        .debug_checkpoint_sample_offset(),
                    "rust_mix_samples": modern_audio_state.1.debug_mix_samples(),
                    "rust_echo_config": modern_audio_state.1.echo_debug_config(),
                    "rust_echo_state": modern_audio_state.1.echo_debug_state(),
                    "rust_echo_ring_left": modern_audio_state.1.echo_debug_ring().0,
                    "rust_echo_ring_right": modern_audio_state.1.echo_debug_ring().1,
                    "rust_echo_ring_before": rust_echo_ring_before,
                    "rust_staged_output_before": rust_staged_output_before,
                    "rust_spc_instruction_trace": rust_spc_instruction_trace,
                    "rust_audio_event_frame": rust_event_frame,
                    "rust_voices_after": game.zelda_modern_audio_voice_debug_states(),
                });
                fs::write(
                    dir.join(format!("dsp_trace_frame_{frame_index}.json")),
                    serde_json::to_vec(&receipt).unwrap(),
                )
                .unwrap();
            }
        }
        if compare_this_frame && compare_audio {
            continuous_audio.push_stereo_frame(&rust_audio, &capture.audio);
            if !wrote_first_audio_mismatch && rust_audio != capture.audio {
                if let Some(dir) = session_dir.as_deref() {
                    let first_interleaved = rust_audio
                        .iter()
                        .zip(&capture.audio)
                        .position(|(rust, oracle)| rust != oracle)
                        .unwrap_or_else(|| rust_audio.len().min(capture.audio.len()));
                    write_wav_i16_stereo(
                        &dir.join("first_audio_mismatch_rust.wav"),
                        &rust_audio,
                        oracle.av_info.timing.sample_rate.round() as u32,
                        2,
                    )
                    .unwrap_or_else(|error| {
                        eprintln!("failed to write first Rust audio mismatch: {error}");
                        process::exit(1);
                    });
                    write_wav_i16_stereo(
                        &dir.join("first_audio_mismatch_oracle.wav"),
                        &capture.audio,
                        oracle.av_info.timing.sample_rate.round() as u32,
                        2,
                    )
                    .unwrap_or_else(|error| {
                        eprintln!("failed to write first oracle audio mismatch: {error}");
                        process::exit(1);
                    });
                    let modern_audio_state = game.zelda_modern_audio_state();
                    let receipt = serde_json::json!({
                        "frame": frame_index,
                        "first_interleaved": first_interleaved,
                        "first_sample_frame": first_interleaved / 2,
                        "channel": first_interleaved % 2,
                        "rust": rust_audio.get(first_interleaved),
                        "oracle": capture.audio.get(first_interleaved),
                        "rust_sample_frames": rust_audio.len() / 2,
                        "oracle_sample_frames": capture.audio.len() / 2,
                        "rust_modern_voices_before": rust_modern_voices_before,
                        "rust_modern_voices_after": game.zelda_modern_audio_voice_debug_states(),
                        "rust_modern_voice_samples": modern_audio_state
                            .1
                            .debug_voice_samples(),
                        "rust_modern_voice_gains": modern_audio_state
                            .1
                            .debug_voice_gains(),
                        "rust_modern_mix_samples": modern_audio_state
                            .1
                            .debug_mix_samples(),
                        "rust_modern_echo_state": modern_audio_state.1.echo_debug_state(),
                        "rust_modern_echo_config": modern_audio_state.1.echo_debug_config(),
                        "rust_modern_echo_history": modern_audio_state.1.echo_debug_fir_history(),
                        "rust_modern_global_state": modern_audio_state.1.global_debug_state(),
                        "rust_dsp_rendered_samples": modern_audio_state
                            .1
                            .debug_dsp_rendered_samples(),
                        "rust_checkpoint_sample_offset": modern_audio_state
                            .1
                            .debug_checkpoint_sample_offset(),
                        "rust_dialogue_decoded_text": game.ram.get(0x11200..0x11300),
                        "rust_dialogue_vwf_widths": game.dialogue_vwf_widths(),
                        "rust_modern_voice_7_sample_data": modern_audio_state
                            .1
                            .debug_voice_sample_data(7),
                        "rust_modern_voice_sample_data": (0..8)
                            .filter_map(|voice| modern_audio_state.1.debug_voice_sample_data(voice))
                            .collect::<Vec<_>>(),
                        "rust_audio_event_frame": rust_event_frame,
                        "oracle_dsp_samples": oracle.debug_dsp_samples(),
                        "oracle_dsp_register_writes": oracle.debug_dsp_register_writes(),
                        "oracle_apu_port_writes": oracle.debug_apu_port_writes(),
                        "oracle_smp_instructions": oracle.debug_smp_instructions(),
                    });
                    fs::write(
                        dir.join("first_audio_mismatch.json"),
                        serde_json::to_vec_pretty(&receipt).unwrap(),
                    )
                    .unwrap_or_else(|error| {
                        eprintln!("failed to write first audio mismatch receipt: {error}");
                        process::exit(1);
                    });
                }
                wrote_first_audio_mismatch = true;
            }
            compared_audio_sample_frames =
                compared_audio_sample_frames.saturating_add(sample_frames as u64);
            audio_frame_ends.push(compared_audio_sample_frames);
            stop_after_exact_audio_mismatch = !scan_all && continuous_audio.exact_mismatch_seen();
        }
        stage(5, &mut stage_ns, &mut stage_mark);
        if should_write_frame_receipt(frame_index, compare_from_frame, frames, compare_this_frame) {
            write_libretro_frame_receipt(
                frame_receipts.as_mut(),
                frame_index,
                input,
                rust_audio.len() / 2,
                capture.audio.len() / 2,
                capture.video_width,
                capture.video_height,
                &pre_ram,
                &game.ram,
                pre_load_remaining_nmi_slices,
                game.zelda_debug_selected_game_load_remaining_nmi_slices(),
                game.zelda_debug_game_execution_scheduler(),
                game.debug_last_poly_work(),
                rust_poly_cycles,
                game.zelda_modern_audio_sfx_clock_checkpoint(),
                game.zelda_spc_driver_clock_debug_summary(),
                &rust_event_frame,
                oracle.memory_bytes(RETRO_MEMORY_SYSTEM_RAM),
                game.vram(),
                oracle.memory_bytes(RETRO_MEMORY_VIDEO_RAM),
            );
        }
        stage(6, &mut stage_ns, &mut stage_mark);
        if stage_timing && frame_index % 2000 == 1999 {
            let total: u128 = stage_ns.iter().sum();
            let (
                capture_ms,
                gpu_render_ms,
                gpu_submit_ms,
                gpu_readback_ms,
                video_hash_ms,
                source_extract_ms,
                compositor_submit_ms,
                history_submit_ms,
                surface_present_ms,
            ) = native_window_video
                .as_ref()
                .map(gpu_capture::NativeWindowOracleRenderer::timing_millis)
                .unwrap_or_default();
            eprintln!(
                "snes9x_timing frames={} total_ms={} pre_state_ms={} poly_ms={} run_frame_ms={} video_ms={} oracle_ms={} audio_ms={} receipts_ms={} video_capture_ms={} render_total_ms={} source_extract_ms={} compositor_submit_ms={} history_submit_ms={} surface_present_ms={} gpu_copy_submit_ms={} gpu_readback_ms={} video_hash_ms={}",
                frame_index + 1,
                total / 1_000_000,
                stage_ns[0] / 1_000_000,
                stage_ns[1] / 1_000_000,
                stage_ns[2] / 1_000_000,
                stage_ns[3] / 1_000_000,
                stage_ns[4] / 1_000_000,
                stage_ns[5] / 1_000_000,
                stage_ns[6] / 1_000_000,
                capture_ms,
                gpu_render_ms,
                source_extract_ms,
                compositor_submit_ms,
                history_submit_ms,
                surface_present_ms,
                gpu_submit_ms,
                gpu_readback_ms,
                video_hash_ms,
            );
        }
        if let Some((x, y)) = trace_video_pixel.filter(|_| compare_this_frame) {
            let (displayed_ppu, rust_bg_pal3, rust_bg_pal4, rust_obj_pal) = game
                .with_display_snapshot(|snapshot| {
                    (
                        crate::render_diagnostics::format_render_ppu_summary(snapshot),
                        (0x30..=0x3f)
                            .map(|i| format!("{:04x}", snapshot.ppu.cgram[i]))
                            .collect::<Vec<_>>()
                            .join(","),
                        (0x40..=0x4f)
                            .map(|i| format!("{:04x}", snapshot.ppu.cgram[i]))
                            .collect::<Vec<_>>()
                            .join(","),
                        (0x90..=0x9f)
                            .map(|i| format!("{:04x}", snapshot.ppu.cgram[i]))
                            .collect::<Vec<_>>()
                            .join(","),
                    )
                });
            let oracle_bg_pal3 = (0x30..=0x3f)
                .map(|i| {
                    oracle
                        .debug_ppu_value(2, i)
                        .map_or_else(|| "none".to_string(), |value| format!("{value:04x}"))
                })
                .collect::<Vec<_>>()
                .join(",");
            let oracle_bg_pal4 = (0x40..=0x4f)
                .map(|i| {
                    oracle
                        .debug_ppu_value(2, i)
                        .map_or_else(|| "none".to_string(), |value| format!("{value:04x}"))
                })
                .collect::<Vec<_>>()
                .join(",");
            let oracle_obj_pal = (0x90..=0x9f)
                .map(|i| {
                    oracle
                        .debug_ppu_value(2, i)
                        .map_or_else(|| "none".to_string(), |value| format!("{value:04x}"))
                })
                .collect::<Vec<_>>()
                .join(",");
            let pixel_index = y.saturating_mul(width as usize).saturating_add(x);
            let rust_offset = pixel_index.saturating_mul(4);
            let snes9x_offset = y.saturating_mul(capture.video_pitch)
                + x * snes9x_pixel_stride(capture.pixel_format).unwrap_or(0);
            let rust_pixel = rust_video_frame
                .map(|frame| frame.as_slice())
                .and_then(|frame| rgba_pixel_at(frame, rust_offset))
                .unwrap_or([0; 4]);
            let oracle_pixel = snes9x_rgba_pixel_at(&capture, snes9x_offset).unwrap_or([0; 4]);
            println!(
                "pixel frame={frame_index} xy=({x},{y}) rust={rust_pixel:02x?} {oracle_name}={oracle_pixel:02x?} main={:02x} sub={:02x} subsub={:02x} inidisp={:02x} rust_bg_pal3=[{rust_bg_pal3}] oracle_bg_pal3=[{oracle_bg_pal3}] rust_bg_pal4=[{rust_bg_pal4}] oracle_bg_pal4=[{oracle_bg_pal4}] rust_obj_pal90=[{rust_obj_pal}] oracle_obj_pal90=[{oracle_obj_pal}]",
                game.ram[0x10], game.ram[0x11], game.ram[0xb0], game.ram[0x13],
            );
            println!("pixel displayed_ppu frame={frame_index} {displayed_ppu}");
            let oracle_obj_control = (0..5)
                .map(|index| oracle.debug_ppu_value(19, index).unwrap_or(-1))
                .collect::<Vec<_>>();
            let oracle_obj_config = (0..3)
                .map(|index| oracle.debug_ppu_value(17, index).unwrap_or(-1))
                .collect::<Vec<_>>();
            let oracle_scanline =
                snes9x_presented_scanline_for_video_y(capture.video_height as usize, y);
            let oracle_obj_line = (0..128)
                .map(|slot| {
                    let index = (oracle_scanline * 128 + slot) as i32;
                    (
                        oracle.debug_ppu_value(21, index).unwrap_or(-1),
                        oracle.debug_ppu_value(26, index).unwrap_or(-1),
                    )
                })
                .take_while(|&(sprite, _)| sprite >= 0)
                .collect::<Vec<_>>();
            let mut oracle_obj_entries = oracle_obj_line
                .iter()
                .map(|&(sprite, _)| sprite)
                .collect::<Vec<_>>();
            oracle_obj_entries.sort_unstable();
            oracle_obj_entries.dedup();
            let oracle_obj_entries = oracle_obj_entries
                .into_iter()
                .map(|sprite| {
                    let bytes: [i32; 4] = std::array::from_fn(|byte| {
                        oracle
                            .debug_ppu_value(20, sprite * 4 + byte as i32)
                            .unwrap_or(-1)
                    });
                    (sprite, bytes)
                })
                .collect::<Vec<_>>();
            let oracle_pixel_operands = (0..10)
                .map(|index| oracle.debug_ppu_value(28, index).unwrap_or(-1))
                .collect::<Vec<_>>();
            let oracle_bg_provenance = (0..9)
                .map(|index| oracle.debug_ppu_value(35, index).unwrap_or(-1))
                .collect::<Vec<_>>();
            let rust_scanline_scroll = game.with_display_snapshot(|snapshot| {
                let scanlines = snapshot.ppu_scanline_windows();
                scanlines
                    .get(y)
                    .map(|line| (line.5, line.6))
                    .unwrap_or(([0; 4], [0; 4]))
            });
            let oracle_scroll = |line: usize| {
                (
                    std::array::from_fn::<_, 4, _>(|layer| {
                        oracle
                            .debug_ppu_value(33, (line * 4 + layer) as i32)
                            .unwrap_or(-1)
                    }),
                    std::array::from_fn::<_, 4, _>(|layer| {
                        oracle
                            .debug_ppu_value(34, (line * 4 + layer) as i32)
                            .unwrap_or(-1)
                    }),
                )
            };
            println!(
                "pixel oracle_obj frame={frame_index} video_line={y} presented_line={oracle_scanline} config={oracle_obj_config:?} control={oracle_obj_control:?} tiles={} flags={} evaluated={oracle_obj_line:?} entries={oracle_obj_entries:02x?} operands={oracle_pixel_operands:?} bg_provenance={oracle_bg_provenance:04x?}",
                oracle.debug_ppu_value(22, oracle_scanline as i32).unwrap_or(-1),
                oracle.debug_ppu_value(23, oracle_scanline as i32).unwrap_or(-1),
            );
            println!(
                "pixel bg_scroll frame={frame_index} rust_video={rust_scanline_scroll:04x?} oracle_video={:04x?} oracle_presented={:04x?}",
                oracle_scroll(y),
                oracle_scroll(oracle_scanline),
            );
            let bg_cache_candidates = game.with_display_snapshot(|snapshot| {
                (0..2usize)
                    .filter_map(|layer_index| {
                        let layer = &snapshot.ppu.bg_layer[layer_index];
                        let cols = if layer.tilemap_wider { 64usize } else { 32 };
                        let rows = if layer.tilemap_higher { 64usize } else { 32 };
                        let source_x = (x + usize::from(layer.h_scroll)).rem_euclid(cols * 8);
                        let source_y = (y + usize::from(layer.v_scroll) + 1).rem_euclid(rows * 8);
                        let tx = source_x / 8;
                        let ty = source_y / 8;
                        let quadrant = usize::from(layer.tilemap_wider && tx >= 32)
                            + if layer.tilemap_higher && ty >= 32 {
                                if layer.tilemap_wider {
                                    2
                                } else {
                                    1
                                }
                            } else {
                                0
                            };
                        let tilemap_word = usize::from(layer.tilemap_adr)
                            + quadrant * 0x400
                            + (ty % 32) * 32
                            + tx % 32;
                        let entry = *snapshot.ppu.vram.get(tilemap_word)?;
                        let tile_number = usize::from(entry & 0x03ff);
                        let chr_word = usize::from(layer.tile_adr) + tile_number * 16;
                        let cache_tile = chr_word / 16;
                        let local_x = if entry & 0x4000 != 0 {
                            7 - source_x % 8
                        } else {
                            source_x % 8
                        };
                        let local_y = if entry & 0x8000 != 0 {
                            7 - source_y % 8
                        } else {
                            source_y % 8
                        };
                        let rust_indices = renderer::modern_extract::decode_snes_4bpp_tile_indices(
                            &snapshot.ppu.vram,
                            usize::from(layer.tile_adr),
                            entry & 0xc3ff,
                        );
                        Some((
                            layer_index,
                            tilemap_word,
                            entry,
                            tile_number,
                            chr_word,
                            cache_tile,
                            local_x,
                            local_y,
                            rust_indices,
                        ))
                    })
                    .collect::<Vec<_>>()
            });
            for (
                layer_index,
                tilemap_word,
                entry,
                tile_number,
                chr_word,
                cache_tile,
                local_x,
                local_y,
                rust_indices,
            ) in bg_cache_candidates
            {
                let live_indices = renderer::modern_extract::decode_snes_4bpp_tile_indices(
                    &game.ppu.vram,
                    usize::from(game.ppu.bg_layer[layer_index].tile_adr),
                    entry & 0xc3ff,
                );
                let oracle_indices = (0..64usize)
                    .map(|pixel| {
                        oracle
                            .debug_ppu_value(31, (cache_tile * 64 + pixel) as i32)
                            .unwrap_or(-1)
                    })
                    .collect::<Vec<_>>();
                let pixel_offset = local_y * 8 + local_x;
                println!(
                    "pixel bg_cache frame={frame_index} layer={} tilemap_word={tilemap_word:04x} entry={entry:04x} tile={tile_number:03x} chr_word={chr_word:04x} cache_tile={cache_tile:03x} valid={} local=({local_x},{local_y}) rust_presented_index={} rust_live_index={} oracle_index={} rust_presented_indices={rust_indices:02x?} rust_live_indices={live_indices:02x?} oracle_indices={oracle_indices:02x?}",
                    layer_index + 1,
                    oracle.debug_ppu_value(32, cache_tile as i32).unwrap_or(-1),
                    rust_indices[pixel_offset],
                    live_indices[pixel_offset],
                    oracle_indices[pixel_offset],
                );
            }
            println!(
                "modern_pixel_trace frame={frame_index} xy=({x},{y}) via=native-window-source-gpu"
            );
            let semantic_trace = native_window_video
                .as_ref()
                .expect("native window renderer allocated for pixel trace")
                .trace_game_pixel(&mut game, x as i16, y as i16)
                .unwrap_or_else(|error| vec![format!("semantic pixel trace failed: {error}")]);
            for line in semantic_trace {
                println!("modern_pixel_owner frame={frame_index} xy=({x},{y}) {line}");
            }
        }
        let mut av_video_hashes = None;
        if compare_this_frame && compare_video {
            let rust_video_frame = rust_video_frame
                .map(|frame| frame.as_slice())
                .expect("GPU video frame rendered for libretro video comparison");
            if debug_video_frames.contains(&frame_index) {
                if let Some(dir) = session_dir.as_deref() {
                    let _ = write_rgba_frame_png(
                        &dir.join(format!("rust_video_{frame_index}.png")),
                        rust_video_frame,
                        width,
                        height,
                    );
                    if let Some(stride) = snes9x_pixel_stride(capture.pixel_format) {
                        let mut oracle_argb =
                            vec![
                                0u8;
                                capture.video_width as usize * capture.video_height as usize * 4
                            ];
                        for y in 0..capture.video_height as usize {
                            for x in 0..capture.video_width as usize {
                                let src = y * capture.video_pitch + x * stride;
                                if let Some([r, g, b, _]) = snes9x_rgba_pixel_at(&capture, src) {
                                    let dst = (y * capture.video_width as usize + x) * 4;
                                    oracle_argb[dst] = b;
                                    oracle_argb[dst + 1] = g;
                                    oracle_argb[dst + 2] = r;
                                    oracle_argb[dst + 3] = 0xff;
                                }
                            }
                        }
                        let _ = write_argb_frame_png(
                            &dir.join(format!("oracle_video_{frame_index}.png")),
                            &oracle_argb,
                            capture.video_width,
                            capture.video_height,
                        );
                    }
                }
            }
            let mut video_diff = compare_libretro_video_frame(
                rust_video_frame,
                width,
                height,
                &capture,
                color_tolerance,
                max_mismatched_pixels,
            );
            if auto_align_video && video_diff.is_some() {
                let (aligned_capture, extra, matched) = align_snes9x_video_capture(
                    &mut oracle,
                    capture,
                    rust_video_frame,
                    width,
                    height,
                    input,
                    120,
                    color_tolerance,
                    max_mismatched_pixels,
                );
                capture = aligned_capture;
                if matched {
                    println!(
                        "auto-aligned {oracle_name} video at frame {frame_index} with {extra} extra frame(s)"
                    );
                    video_diff = None;
                } else {
                    video_diff = compare_libretro_video_frame(
                        rust_video_frame,
                        width,
                        height,
                        &capture,
                        color_tolerance,
                        max_mismatched_pixels,
                    );
                }
            }
            av_video_hashes = Some(
                canonical_video_digest_pair(rust_video_frame, width, height, &capture)
                    .unwrap_or_else(|error| {
                        eprintln!("failed to hash canonical A/V frame {frame_index}: {error}");
                        process::exit(1);
                    }),
            );
            if let Some(video_diff) = video_diff {
                video_mismatch_this_frame = true;
                append_u32_range(&mut video_mismatch_ranges, frame_index);
                if first_video_mismatch.is_none() {
                    first_video_mismatch = Some(video_diff.clone());
                    if oracle_before_state_frame != frame_index {
                        eprintln!(
                            "{oracle_name} video divergence at frame {frame_index}: {video_diff}; input={input:04x}; ports={ports:02x?}; main={:02x} sub={:02x} subsub={:02x}",
                            game.ram[0x10], game.ram[0x11], game.ram[0xb0],
                        );
                        eprintln!(
                            "detailed pre-frame artifacts require a focused replay beginning at frame {frame_index}; the authoritative cold run does not serialize Snes9x in flight"
                        );
                    } else {
                        let oracle_system_ram = oracle
                            .memory_bytes(RETRO_MEMORY_SYSTEM_RAM)
                            .map(<[u8]>::to_vec);
                        let mut oracle_after_state = vec![0; oracle_before_state.len()];
                        oracle
                            .serialize_state_into(&mut oracle_after_state)
                            .unwrap_or_else(|e| {
                                eprintln!(
                                "failed to serialize {oracle_name} after frame {frame_index}: {e}"
                            );
                                process::exit(1);
                            });
                        let oracle_before_vram = oracle
                        .unserialize_state(&oracle_before_state)
                        .and_then(|()| {
                            oracle
                                .memory_bytes(RETRO_MEMORY_VIDEO_RAM)
                                .map(<[u8]>::to_vec)
                                .ok_or_else(|| {
                                    format!("{oracle_name} did not expose pre-frame VRAM")
                                })
                        })
                        .and_then(|vram| {
                            oracle
                                .unserialize_state(&oracle_after_state)
                                .map(|()| vram)
                        })
                        .unwrap_or_else(|e| {
                            eprintln!(
                                "failed to capture {oracle_name} pre-frame VRAM at frame {frame_index}: {e}"
                            );
                            process::exit(1);
                        });
                        let display_oracle_receipt =
                            capture_oracle_ppu_probe(&oracle).map(|oracle_ppu| {
                                let (rust, mut rust_candidates, rust_context) =
                                    rust_rendered_display
                                        .map(|rendered| {
                                            capture_rendered_rust_ppu_probe(&game, rendered)
                                        })
                                        .unwrap_or_else(|| capture_rust_ppu_probe(&mut game));
                                annotate_display_candidate_differences(
                                    &oracle_ppu,
                                    &mut rust_candidates,
                                );
                                DisplayOracleReceipt {
                                    frame: frame_index,
                                    stage: "after",
                                    oracle: oracle_ppu,
                                    rust,
                                    rust_candidates,
                                    rust_context,
                                }
                            });
                        let oracle_presented_oam = snes9x_presented_oam_bytes(&oracle)
                        .unwrap_or_else(|error| {
                            eprintln!(
                                "failed to capture pinned-Snes9x presented OAM at frame {frame_index}: {error}"
                            );
                            process::exit(1);
                        });
                        let artifact_dir = write_libretro_parity_failure_artifacts(
                            pre_game.as_ref(),
                            &game,
                            rust_video_frame,
                            &rust_audio,
                            &capture,
                            &oracle_before_state,
                            &oracle_after_state,
                            &pre_ram,
                            &input_history,
                            frame_index,
                            input,
                            oracle.av_info.timing.sample_rate.round() as u32,
                            oracle_name.as_str(),
                            oracle_system_ram.as_deref(),
                            Some(&oracle_before_vram),
                            rust_rendered_display,
                            display_oracle_receipt.as_ref(),
                            oracle_presented_oam.as_deref(),
                            format!("{oracle_name} video divergence: {video_diff}"),
                        )
                        .ok();
                        eprintln!(
                        "{oracle_name} video divergence at frame {frame_index}: {video_diff}; input={input:04x}; ports={ports:02x?}; main={:02x} sub={:02x} subsub={:02x}",
                        game.ram[0x10], game.ram[0x11], game.ram[0xb0],
                    );
                        eprintln!("rust audio:  {:?}", rust_stats);
                        eprintln!("{oracle_name} audio: {:?}", oracle_stats);
                        eprintln!("rust audio debug: {}", game.zelda_audio_debug_summary());
                        if let Some(dir) = artifact_dir {
                            eprintln!("parity failure artifacts: {}", dir.display());
                        }
                    }
                }
            }
        }
        if compare_this_frame {
            write_av_hash_record(
                av_hashes.as_mut(),
                frame_index,
                input,
                capture.audio.len() / 2,
                av_video_hashes,
                av_audio_hashes,
            );
        }
        completed_frames = frame_index.saturating_add(1);
        if should_stop_after_first_mismatch(
            scan_all,
            stop_after_exact_audio_mismatch,
            video_mismatch_this_frame,
        ) {
            break;
        }
    }

    if debug_first_nmi_dma_setup.is_some() && !debug_first_nmi_dma_setup_complete {
        eprintln!(
            "first-NMI DMA-setup capture ended before the completed $8080e1 anchor and excluded $008a35 raw-fetch receipt were both observed"
        );
        process::exit(1);
    }
    if debug_first_nmi_dma.is_some() && !debug_first_nmi_dma_complete {
        eprintln!(
            "first-NMI DMA capture ended before the exact $008a35 H714->H722 anchor, complete mask-$07 ledger, and $008a38 successor receipt were observed"
        );
        process::exit(1);
    }
    if debug_first_nmi_return.is_some() && !debug_first_nmi_return_complete {
        eprintln!(
            "first-NMI return capture ended before the exact $008a38 V227:H742->H750 successor and direct run-81 retro_run return were both observed"
        );
        process::exit(1);
    }

    if engine_state_scan_all {
        let field_names = |mismatches: &[String]| -> Vec<String> {
            mismatches
                .iter()
                .map(|m| m.split(' ').next().unwrap_or(m).to_string())
                .collect()
        };
        eprintln!(
            "=== engine-state scan-all: {} divergent frame(s) ===",
            engine_state_divergences.len()
        );
        let mut i = 0;
        while i < engine_state_divergences.len() {
            let (start_frame, first_ms) = {
                let (f, ms) = &engine_state_divergences[i];
                (*f, ms.clone())
            };
            let key = field_names(&first_ms);
            let mut j = i;
            let mut last_frame = start_frame;
            while j < engine_state_divergences.len()
                && field_names(&engine_state_divergences[j].1) == key
            {
                last_frame = engine_state_divergences[j].0;
                j += 1;
            }
            let count = j - i;
            if count == 1 {
                eprintln!("  frame {start_frame}: {}", first_ms.join(", "));
            } else {
                eprintln!(
                    "  frames {start_frame}..{last_frame} ({count} frames, fields {}): first {}",
                    key.join("+"),
                    first_ms.join(", ")
                );
            }
            i = j;
        }
    }

    if let Err(error) = game.finish_rom_random_replay_through(completed_frames) {
        eprintln!("ROM random replay did not complete: {error}");
        process::exit(1);
    }
    let audio_report = compare_audio.then(|| continuous_audio.finish());
    finalize_libretro_session(
        session_dir.as_deref(),
        frame_receipts.as_mut(),
        av_hashes.as_mut(),
        &input_history,
        initial_input_script.as_deref(),
        audio_report.as_ref(),
        &audio_frame_ends,
        &oracle_before_state,
        oracle_before_state_frame,
        &oracle,
        &game,
        completed_frames,
        &video_mismatch_ranges,
        first_video_mismatch.as_deref(),
        first_engine_state_mismatch.as_ref(),
        (live_oracle_rng && resume_paired.is_some()) || seed_rust_from_oracle.is_some(),
    );
    if first_engine_state_mismatch.is_some() {
        if let Some(dir) = session_dir.as_deref() {
            eprintln!("replayable Snes9x session: {}", dir.display());
        }
        process::exit(1);
    }
    if let Some(report) = audio_report.as_ref().filter(|report| !report.matched) {
        let failing_frame = report
            .first_mismatch_sample_frame
            .map(|sample_frame| audio_frame_ends.partition_point(|&end| end <= sample_frame as u64))
            .map(|index| effective_compare_from_frame.saturating_add(index as u32));
        eprintln!(
            "{oracle_name} audio divergence{}: {}",
            failing_frame
                .map(|frame| format!(" at frame {frame}"))
                .unwrap_or_default(),
            report.message,
        );
        if let Some(dir) = session_dir.as_deref() {
            eprintln!("replayable Snes9x session: {}", dir.display());
        }
    }
    if !video_mismatch_ranges.is_empty() {
        eprintln!(
            "{oracle_name} video diverged on {} frame range(s): {}",
            video_mismatch_ranges.len(),
            format_u32_ranges(&video_mismatch_ranges),
        );
    }
    if !video_mismatch_ranges.is_empty()
        || audio_report.as_ref().is_some_and(|report| !report.matched)
    {
        process::exit(1);
    }

    if compare_video || compare_audio {
        println!(
            "{oracle_name} oracle compare completed {completed_frames} frame(s) with no enabled video/audio diff"
        );
    } else {
        println!("{oracle_name} oracle trace replay completed {completed_frames} frame(s)");
    }
}

pub(crate) fn oracle_name_from_core_path(core_path: &str) -> String {
    let stem = Path::new(core_path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("libretro");
    stem.strip_suffix("_libretro").unwrap_or(stem).to_string()
}

pub(crate) fn validate_libretro_frame_window(
    frames: u32,
    compare_from_frame: u32,
) -> Result<(), String> {
    if frames == 0 {
        return Err("libretro parity requires at least one frame".to_string());
    }
    if compare_from_frame >= frames {
        return Err(format!(
            "--compare-from-frame {compare_from_frame} leaves no compared frames in a {frames}-frame route"
        ));
    }
    Ok(())
}

fn resolve_engine_state_compare_start(
    compare_from_frame: u32,
    explicit_engine_start: Option<u32>,
    ignore_engine_state: bool,
) -> Result<Option<u32>, String> {
    if ignore_engine_state && explicit_engine_start.is_some() {
        return Err(
            "--ignore-engine-state cannot be combined with --compare-engine-state-from-frame"
                .to_string(),
        );
    }
    Ok(if ignore_engine_state {
        None
    } else {
        Some(explicit_engine_start.unwrap_or(compare_from_frame))
    })
}

const fn scan_all_policy(explicit_scan_all: bool, _session_dir_present: bool) -> bool {
    explicit_scan_all
}

const fn should_stop_after_first_mismatch(
    scan_all: bool,
    exact_audio_mismatch: bool,
    video_mismatch: bool,
) -> bool {
    !scan_all && (exact_audio_mismatch || video_mismatch)
}

fn compact_engine_state_mismatches(rust: &[u8], oracle: &[u8]) -> Vec<String> {
    let byte = |ram: &[u8], address: usize| ram.get(address).copied().unwrap_or_default();
    let word = |ram: &[u8], address: usize| {
        u16::from_le_bytes([byte(ram, address), byte(ram, address.saturating_add(1))])
    };
    let mut mismatches = Vec::new();
    for (name, address) in [
        ("main_module", 0x0010),
        ("submodule", 0x0011),
        ("subsubmodule", 0x00b0),
        ("frame_counter", 0x001a),
    ] {
        let rust_value = byte(rust, address);
        let oracle_value = byte(oracle, address);
        if rust_value != oracle_value {
            mismatches.push(format!(
                "{name} rust=0x{rust_value:02x} oracle=0x{oracle_value:02x}"
            ));
        }
    }
    for (name, address) in [
        ("dungeon_room", 0x00a0),
        ("link_x", 0x0022),
        ("link_y", 0x0020),
        ("bg2_h", 0x00e2),
        ("bg2_v", 0x00e8),
    ] {
        let rust_value = word(rust, address);
        let oracle_value = word(oracle, address);
        if rust_value != oracle_value {
            mismatches.push(format!(
                "{name} rust=0x{rust_value:04x} oracle=0x{oracle_value:04x}"
            ));
        }
    }
    for slot in 0..16 {
        for (name, base) in [
            ("state", 0x0dd0),
            ("type", 0x0e20),
            ("x_low", 0x0d10),
            ("x_high", 0x0d30),
            ("x_subpixel", 0x0d70),
            ("x_velocity", 0x0d50),
            ("y_low", 0x0d00),
            ("y_high", 0x0d20),
            ("y_subpixel", 0x0d60),
            ("y_velocity", 0x0d40),
            ("direction", 0x0de0),
            ("head_direction", 0x0eb0),
            ("graphics", 0x0dc0),
            ("ai_state", 0x0d80),
            ("wall_collision", 0x0e70),
            ("subtype", 0x0e30),
            ("subtype2", 0x0e80),
            ("delay_main", 0x0df0),
            ("delay_aux1", 0x0e00),
        ] {
            let rust_value = byte(rust, base + slot);
            let oracle_value = byte(oracle, base + slot);
            if rust_value != oracle_value {
                mismatches.push(format!(
                    "sprite[{slot}].{name} rust=0x{rust_value:02x} oracle=0x{oracle_value:02x}"
                ));
            }
        }
    }
    mismatches
}

fn first_dsp_write_timing_mismatch(
    rust: &[DspWriteEvent],
    oracle: &[LibretroDspRegisterWrite],
) -> Option<usize> {
    let shared = rust.len().min(oracle.len());
    (0..shared)
        .find(|&index| {
            let rust = rust[index];
            let oracle = oracle[index];
            i32::from(rust.addr) != oracle.register
                || i32::from(rust.value) != oracle.value
                || rust.sample_offset != oracle.output_sample
                || i32::from(rust.timer_cycles) != oracle.dsp_phase
        })
        .or_else(|| (rust.len() != oracle.len()).then_some(shared))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SpcClockWitness {
    phase_delta: u8,
    pc: u16,
    opcode: u8,
    rust_tail: usize,
    oracle_tail: usize,
    rust_timer_divider: u8,
    oracle_timer_divider: u8,
}

fn last_spc_clock_witness(
    rust: &[snes::apu::SpcInstructionTrace],
    oracle: &[crate::libretro_core::LibretroSmpInstruction],
) -> Option<SpcClockWitness> {
    // A libretro video-frame boundary can split an SPC instruction differently
    // from the translated host window. Match CPU state near the two tails
    // instead of comparing list indexes, which would turn that harmless split
    // into a false clock divergence.
    const TAIL_SEARCH: usize = 64;
    let rust_start = rust.len().saturating_sub(TAIL_SEARCH);
    let oracle_start = oracle.len().saturating_sub(TAIL_SEARCH);
    let mut best = None::<(usize, SpcClockWitness)>;
    for (rust_index, rust_instruction) in rust.iter().enumerate().skip(rust_start) {
        for (oracle_index, oracle_instruction) in oracle.iter().enumerate().skip(oracle_start) {
            if i32::from(rust_instruction.pc) != oracle_instruction.program_counter
                || i32::from(rust_instruction.opcode) != oracle_instruction.opcode
                || i32::from(rust_instruction.a) != oracle_instruction.a
                || i32::from(rust_instruction.x) != oracle_instruction.x
                || i32::from(rust_instruction.y) != oracle_instruction.y
                || i32::from(rust_instruction.sp) != oracle_instruction.stack_pointer
            {
                continue;
            }
            let rust_tail = rust.len() - 1 - rust_index;
            let oracle_tail = oracle.len() - 1 - oracle_index;
            let tail_distance = rust_tail + oracle_tail;
            let phase_delta = (128i32
                - i32::from(rust_instruction.timer0_cycles)
                - oracle_instruction.timer0_stage1)
                .rem_euclid(128) as u8;
            let witness = SpcClockWitness {
                phase_delta,
                pc: rust_instruction.pc,
                opcode: rust_instruction.opcode,
                rust_tail,
                oracle_tail,
                rust_timer_divider: rust_instruction.timer0_divider,
                oracle_timer_divider: oracle_instruction.timer0_stage2 as u8,
            };
            if best
                .as_ref()
                .is_none_or(|(best_distance, _)| tail_distance < *best_distance)
            {
                best = Some((tail_distance, witness));
            }
        }
    }
    best.map(|(_, witness)| witness)
}

const fn should_write_frame_receipt(
    frame: u32,
    compare_from_frame: u32,
    total_frames: u32,
    compare_this_frame: bool,
) -> bool {
    // Warm-up state is already preserved by the initial/final snapshots and
    // replay input. Focused probes retain every detailed receipt, while long
    // cold sweeps sample one per second plus the comparison boundary. The
    // dedicated first-mismatch artifacts preserve the bad frame in full; this
    // keeps a 20k-frame proof from serializing roughly 500 MB of redundant
    // successful WRAM/VRAM receipts.
    if !compare_this_frame {
        return false;
    }
    let comparison_frames = total_frames.saturating_sub(compare_from_frame);
    comparison_frames <= 1_200
        || frame == compare_from_frame
        || frame.saturating_sub(compare_from_frame).is_multiple_of(60)
}

const VIDEO_WARMUP_PRIMING_FRAMES: u32 = 60;

const fn should_render_video_frame(
    frame: u32,
    compare_from_frame: u32,
    video_requested: bool,
) -> bool {
    video_requested && frame.saturating_add(VIDEO_WARMUP_PRIMING_FRAMES) >= compare_from_frame
}

pub(crate) fn validate_required_libretro_core(
    required: Option<(&str, &str)>,
    actual_name: &str,
    actual_version: &str,
) -> Result<(), String> {
    let Some((required_name, required_version)) = required else {
        return Ok(());
    };
    if !actual_name
        .to_ascii_lowercase()
        .contains(&required_name.to_ascii_lowercase())
    {
        return Err(format!(
            "wrong libretro core: expected {required_name}, loaded {actual_name} {actual_version}"
        ));
    }
    if !required_version.is_empty() && !actual_version.starts_with(required_version) {
        return Err(format!(
            "wrong {required_name} version: expected {required_version}, loaded {actual_version}"
        ));
    }
    Ok(())
}

pub(crate) fn verify_expected_sha256(path: &str, label: &str, expected: Option<&str>) {
    if let Err(error) = expected_sha256_matches(Path::new(path), label, expected) {
        eprintln!("{error}");
        process::exit(1);
    }
}

pub(crate) fn expected_sha256_matches(
    path: &Path,
    label: &str,
    expected: Option<&str>,
) -> Result<(), String> {
    let Some(expected) = expected else {
        return Ok(());
    };
    let actual = parity::runner::sha256_file(path)
        .map_err(|error| format!("failed to hash {label} {}: {error}", path.display()))?;
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(format!(
            "{label} hash mismatch for replay: expected {expected}, found {actual} at {}",
            path.display()
        ))
    }
}

/// Parse an explicit frame selection for diagnostic artifact capture.
///
/// The comparator intentionally keeps this separate from parity decisions:
/// it only controls which already-observed state boundaries are written to a
/// session directory.  Accepting ranges keeps an early-divergence capture
/// practical without a shell-generated list of thousands of frame numbers.
pub(crate) fn debug_frame_selection_from_env(primary: &str, legacy: Option<&str>) -> Vec<u32> {
    let value = env::var(primary)
        .ok()
        .or_else(|| legacy.and_then(|name| env::var(name).ok()));
    value
        .as_deref()
        .map(parse_debug_frame_selection)
        .unwrap_or_default()
}

pub(crate) fn parse_debug_frame_selection(value: &str) -> Vec<u32> {
    let mut frames = Vec::new();
    for part in value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        let range = part
            .split_once("..=")
            .or_else(|| part.split_once('-'))
            .and_then(|(start, end)| {
                Some((
                    start.trim().parse::<u32>().ok()?,
                    end.trim().parse::<u32>().ok()?,
                ))
            });
        match range {
            Some((start, end)) if start <= end => frames.extend(start..=end),
            Some(_) => {}
            None => {
                if let Ok(frame) = part.parse() {
                    frames.push(frame);
                }
            }
        }
    }
    frames.sort_unstable();
    frames.dedup();
    frames
}

fn parse_debug_byte_range(value: &str) -> Option<std::ops::Range<usize>> {
    let (start, end) = value.split_once("..")?;
    let parse = |part: &str| {
        let part = part.trim();
        part.strip_prefix("0x")
            .or_else(|| part.strip_prefix("0X"))
            .map_or_else(
                || part.parse::<usize>().ok(),
                |hex| usize::from_str_radix(hex, 16).ok(),
            )
    };
    let start = parse(start)?;
    let end = parse(end)?;
    (start <= end).then_some(start..end)
}

pub(crate) fn append_u32_range(ranges: &mut Vec<(u32, u32)>, value: u32) {
    if let Some((_, end)) = ranges.last_mut() {
        if value == end.saturating_add(1) {
            *end = value;
            return;
        }
    }
    ranges.push((value, value));
}

pub(crate) fn format_u32_ranges(ranges: &[(u32, u32)]) -> String {
    ranges
        .iter()
        .map(|&(start, end)| {
            if start == end {
                start.to_string()
            } else {
                format!("{start}..{end}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn oracle_preframe_snapshot_required(_frame: u32, _frames: u32, _compare_this_frame: bool) -> bool {
    // Pinned Snes9x's save path is not observationally pure: SPC_DSP::copy_state
    // canonicalizes live BRR and echo-history storage while serializing. Calling
    // retro_serialize in the authoritative loop can therefore change later APU
    // handshakes and, eventually, the CPU instruction that crosses vblank.
    //
    // Keep the cold/live oracle execution free of in-run save-state calls. The
    // session's initial state and complete input stream remain replayable, and
    // the final state is captured only after comparison has ended. Focused
    // diagnostics that need a pre-frame state must replay with that frame as
    // their start boundary instead of perturbing every preceding host call.
    false
}

fn install_directory_atomically(
    final_dir: &Path,
    write_contents: impl FnOnce(&Path) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    if final_dir.exists() {
        return Err(format!(
            "refusing to replace existing paired-resume generation {}",
            final_dir.display()
        )
        .into());
    }
    let parent = final_dir
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let name = final_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            format!(
                "paired-resume generation has no file name: {}",
                final_dir.display()
            )
        })?;
    let temporary = parent.join(format!(".{name}.tmp-{}", process::id()));
    if temporary.exists() {
        return Err(format!(
            "stale paired-resume temporary generation exists: {}",
            temporary.display()
        )
        .into());
    }
    fs::create_dir(&temporary)?;
    let result = write_contents(&temporary).and_then(|()| {
        fs::rename(&temporary, final_dir).map_err(|error| {
            format!(
                "install paired-resume generation {}: {error}",
                final_dir.display()
            )
            .into()
        })
    });
    if result.is_err() {
        let _ = fs::remove_dir_all(&temporary);
    }
    result
}

fn write_file_atomically(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("atomic file has no file name: {}", path.display()))?;
    let temporary = parent.join(format!(".{name}.tmp-{}", process::id()));
    if temporary.exists() {
        return Err(format!(
            "stale atomic temporary file exists: {}",
            temporary.display()
        )
        .into());
    }
    let result = fs::write(&temporary, bytes)
        .map_err(|error| -> Box<dyn Error> { Box::new(error) })
        .and_then(|()| {
            fs::rename(&temporary, path)
                .map_err(|error| format!("install atomic file {}: {error}", path.display()).into())
        });
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn write_paired_resume_capture(
    capture: &PairedResumeCapture,
    core_path: &str,
    rom_path: &str,
    input_script_path: Option<&Path>,
    rom_random_script: Option<&Path>,
    initial_sram: &[u8],
    game: &ZeldaState,
    oracle: &LibretroCore,
    semantic_trace: Option<&Snes9xOracleSemanticTrace>,
) -> Result<(), Box<dyn Error>> {
    if !game.paired_resume_cpu_boundary_is_quiescent() {
        return Err(
            "frame is inside an unserialized ROM-call continuation; choose a quiescent pre-frame boundary"
                .into(),
        );
    }
    let original_timing_resume = game.capture_original_timing_resume_checkpoint()?;
    let original_timing_resume_bytes = serde_json::to_vec_pretty(&original_timing_resume)?;
    let rust_state = PlayCrashCheckpoint {
        magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
        host_frame: capture.frame,
        input: 0,
        run_what: select_run_what(&game.ram),
        game: game.clone(),
    };
    let rust_bytes = bincode::serialize(&rust_state)?;
    let trace = semantic_trace
        .ok_or("paired resume requires an authoritative Snes9x semantic trace checkpoint")?;
    let semantic_trace_bytes = serde_json::to_vec_pretty(&trace.checkpoint())?;
    let source_manifest = |path: Option<&Path>| -> Result<serde_json::Value, Box<dyn Error>> {
        Ok(match path {
            Some(path) => serde_json::json!({
                "path": path,
                "sha256": parity::runner::sha256_file(path)?,
            }),
            None => serde_json::Value::Null,
        })
    };
    let core_sha256 = parity::runner::sha256_file(Path::new(core_path))?;
    let rom_sha256 = parity::runner::sha256_file(Path::new(rom_path))?;
    let input_script = source_manifest(input_script_path)?;
    let rom_random_script = source_manifest(rom_random_script)?;

    install_directory_atomically(&capture.dir, |temporary| {
        let oracle_bytes = oracle.serialize_state()?;
        fs::write(temporary.join("rust.z3state"), &rust_bytes)?;
        fs::write(temporary.join("oracle.state"), &oracle_bytes)?;
        fs::write(temporary.join("initial.srm"), initial_sram)?;
        fs::write(
            temporary.join("original-timing.resume.json"),
            &original_timing_resume_bytes,
        )?;
        fs::write(
            temporary.join("semantic-trace.checkpoint.json"),
            &semantic_trace_bytes,
        )?;
        let manifest = serde_json::json!({
            "schema": PAIRED_RESUME_SCHEMA,
            "boundary": "pre-frame",
            "frame": capture.frame,
            "cpu_boundary": "quiescent",
            "renderer_warmup_required": true,
            "rust_state": {
                "artifact": "rust.z3state",
                "sha256": parity::evidence::sha256_bytes(&rust_bytes),
            },
            "oracle_state": {
                "artifact": "oracle.state",
                "sha256": parity::evidence::sha256_bytes(&oracle_bytes),
            },
            "original_timing_resume_checkpoint": {
                "artifact": "original-timing.resume.json",
                "sha256": parity::evidence::sha256_bytes(&original_timing_resume_bytes),
            },
            "semantic_trace_checkpoint": {
                "artifact": "semantic-trace.checkpoint.json",
                "sha256": parity::evidence::sha256_bytes(&semantic_trace_bytes),
            },
            "core": {"path": core_path, "sha256": core_sha256},
            "rom": {"path": rom_path, "sha256": rom_sha256},
            "input_script": input_script,
            "rom_random_script": rom_random_script,
            "initial_sram": {
                "artifact": "initial.srm",
                "sha256": parity::evidence::sha256_bytes(initial_sram),
            },
        });
        fs::write(
            temporary.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest)?,
        )?;
        Ok(())
    })
}

fn write_rolling_paired_resume_capture(
    rolling: &RollingPairedResumeCapture,
    frame: u32,
    core_path: &str,
    rom_path: &str,
    input_script_path: Option<&Path>,
    rom_random_script: Option<&Path>,
    initial_sram: &[u8],
    game: &ZeldaState,
    oracle: &LibretroCore,
    semantic_trace: Option<&Snes9xOracleSemanticTrace>,
) -> Result<PathBuf, Box<dyn Error>> {
    fs::create_dir_all(&rolling.root)?;
    let checkpoint_name = format!("frame-{frame:08}");
    let capture = PairedResumeCapture {
        frame,
        dir: rolling.root.join(&checkpoint_name),
    };
    write_paired_resume_capture(
        &capture,
        core_path,
        rom_path,
        input_script_path,
        rom_random_script,
        initial_sram,
        game,
        oracle,
        semantic_trace,
    )?;
    write_file_atomically(
        &rolling.root.join("latest.json"),
        &serde_json::to_vec_pretty(&serde_json::json!({
            "schema": PAIRED_RESUME_SCHEMA,
            "frame": frame,
            "checkpoint": checkpoint_name,
        }))?,
    )?;
    prune_rolling_paired_resume_captures(
        &rolling.root,
        ROLLING_PAIRED_RESUME_GENERATIONS_KEPT,
        &capture.dir,
    );
    Ok(capture.dir)
}

fn prune_rolling_paired_resume_captures(root: &Path, keep: usize, current: &Path) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let mut captures = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_str()?;
            name.strip_prefix("frame-")?.parse::<u32>().ok()?;
            let path = entry.path();
            let modified = entry.metadata().ok()?.modified().ok()?;
            path.join("manifest.json")
                .is_file()
                .then_some((modified, path))
        })
        .collect::<Vec<_>>();
    // A restarted run may capture a lower frame into an existing root. Keep
    // the generation named by latest.json plus the most recently written
    // fallback; numeric frame order alone could immediately delete the new
    // capture and leave latest.json dangling.
    captures.sort_by_key(|(modified, _)| std::cmp::Reverse(*modified));
    captures.sort_by_key(|(_, path)| path != current);
    for (_, path) in captures.into_iter().skip(keep.max(1)) {
        if let Err(error) = fs::remove_dir_all(&path) {
            eprintln!(
                "failed to prune generated rolling checkpoint {}: {error}",
                path.display()
            );
        }
    }
}

pub(crate) fn initialize_libretro_session(
    session_dir: Option<&Path>,
    core_path: &str,
    rom_path: &str,
    oracle: &LibretroCore,
    game: &ZeldaState,
    initial_sram: &[u8],
    initial_oracle_state: &[u8],
    frames: u32,
    start_frame: u32,
    compare_from_frame: u32,
    compare_engine_state_from_frame: Option<u32>,
    skip_oracle_frames: u32,
    compare_video: bool,
    compare_audio: bool,
    audio_comparison: AudioComparisonMode,
    timing: AudioTimingOptions,
    replay_save: Option<&Path>,
    replay_bundle: Option<&ReplayBundle>,
    initial_input_script: &[u8],
    rom_random_script: Option<&Path>,
    live_oracle_rng: bool,
    scan_all: bool,
    cold_evidence_invocation_id: Option<&str>,
) -> Option<BufWriter<fs::File>> {
    let dir = session_dir?;
    fs::create_dir_all(dir).unwrap_or_else(|e| {
        eprintln!("failed to create libretro session {}: {e}", dir.display());
        process::exit(1);
    });
    let cold_evidence_run_nonce = cold_evidence_invocation_id.map(|invocation_id| {
        let unix_time_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        cold_evidence_run_nonce(dir, invocation_id, unix_time_ns, process::id())
    });
    for stale in [
        "input.txt",
        "audio_frame_ends.json",
        "audio_report.json",
        "first_audio_mismatch.json",
        "first_audio_mismatch_rust.wav",
        "first_audio_mismatch_oracle.wav",
        "av_hashes.jsonl",
        "oracle_last_before.state",
        "oracle_final.state",
        "rust_final.z3state",
        "result.json",
        "rom-random.txt",
    ] {
        match fs::remove_file(dir.join(stale)) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                eprintln!("failed to remove stale libretro session {stale}: {error}");
                process::exit(1);
            }
        }
    }
    // Persist the source controller stream before entering the comparison
    // loop. A later panic (for example a ROM-random order assertion) must still
    // leave a replayable diagnostic session rather than requiring the caller
    // to reconstruct input.txt by hand.
    fs::write(dir.join("input.txt"), initial_input_script).unwrap_or_else(|e| {
        eprintln!("failed to seed libretro session input.txt: {e}");
        process::exit(1);
    });
    fs::write(dir.join("initial.srm"), initial_sram).unwrap_or_else(|e| {
        eprintln!("failed to write libretro session initial.srm: {e}");
        process::exit(1);
    });
    fs::write(dir.join("oracle_initial.state"), initial_oracle_state).unwrap_or_else(|e| {
        eprintln!("failed to write libretro session oracle_initial.state: {e}");
        process::exit(1);
    });
    let rust_initial = PlayCrashCheckpoint {
        magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
        host_frame: start_frame,
        input: 0,
        run_what: RUN_MAIN,
        game: game.clone(),
    };
    fs::write(
        dir.join("rust_initial.z3state"),
        bincode::serialize(&rust_initial).expect("serialize initial Rust parity state"),
    )
    .unwrap_or_else(|e| {
        eprintln!("failed to write libretro session rust_initial.z3state: {e}");
        process::exit(1);
    });
    let core_sha256 = parity::runner::sha256_file(Path::new(core_path)).unwrap_or_else(|e| {
        eprintln!("failed to hash libretro core {core_path}: {e}");
        process::exit(1);
    });
    let rom_sha256 = parity::runner::sha256_file(Path::new(rom_path)).unwrap_or_else(|e| {
        eprintln!("failed to hash ROM {rom_path}: {e}");
        process::exit(1);
    });
    let replay_save_manifest = replay_save.map(|path| {
        let sha256 = parity::runner::sha256_file(path).unwrap_or_else(|e| {
            eprintln!("failed to hash replay save {}: {e}", path.display());
            process::exit(1);
        });
        serde_json::json!({ "path": path, "sha256": sha256 })
    });
    let replay_bundle_manifest = replay_bundle.map(|bundle| {
        serde_json::json!({
            "dir": bundle.dir,
            "frames_completed": bundle.frames_completed,
            "manifest_sha256": bundle.manifest_sha256,
            "input": {
                "artifact": "input.txt",
                "sha256": bundle.input_sha256,
            },
            "rom_random": {
                "artifact": "rom-random.txt",
                "sha256": bundle.rom_random_sha256,
            },
            "initial_sram": {
                "artifact": "initial.srm",
                "sha256": bundle.initial_sram_sha256,
            },
        })
    });
    let rom_random_manifest = rom_random_script.map(|path| {
        let bytes = fs::read(path).unwrap_or_else(|e| {
            eprintln!(
                "failed to read ROM random replay script {}: {e}",
                path.display()
            );
            process::exit(1);
        });
        fs::write(dir.join("rom-random.txt"), &bytes).unwrap_or_else(|e| {
            eprintln!("failed to persist ROM random replay script: {e}");
            process::exit(1);
        });
        let artifact_path = dir.join("rom-random.txt");
        serde_json::json!({
            "source_path": path,
            "artifact": "rom-random.txt",
            "sha256": parity::runner::sha256_file(&artifact_path).unwrap_or_else(|e| {
                eprintln!("failed to hash persisted ROM random replay script: {e}");
                process::exit(1);
            }),
        })
    });
    let mut artifacts = vec![
        "initial.srm",
        "rust_initial.z3state",
        "oracle_initial.state",
        "oracle_last_before.state",
        "input.txt",
        "frame_receipts.jsonl",
        "av_hashes.jsonl",
        "audio_frame_ends.json",
        "audio_report.json",
        "first_audio_mismatch.json",
        "first_audio_mismatch_rust.wav",
        "first_audio_mismatch_oracle.wav",
        "oracle_final.state",
        "rust_final.z3state",
        "result.json",
        "replay.sh",
    ];
    if rom_random_script.is_some() {
        artifacts.push("rom-random.txt");
    }
    if live_oracle_rng {
        artifacts.push(LIVE_ORACLE_RNG_TRACE_ARTIFACT);
    }
    if env::var_os("ZELDA3_CAPTURE_OBJ_STATE_LEDGER").is_some() {
        artifacts.extend([
            "obj_state_ledger.jsonl",
            "obj_state_first_cache_divergence.jsonl",
            "obj_state_first_rust_wram.bin",
            "obj_state_first_oracle_wram.bin",
            "obj_state_first_rust_vram.bin",
            "obj_state_first_oracle_vram.bin",
        ]);
    }
    let manifest = serde_json::json!({
        "schema": 1,
        "status": "running",
        "cold_evidence_invocation_id": cold_evidence_invocation_id,
        "cold_evidence_run_nonce": cold_evidence_run_nonce,
        "core": {
            "path": core_path,
            "sha256": core_sha256,
            "library_name": oracle.library_name,
            "library_version": oracle.library_version,
            "libretro_api_version": oracle.api_version,
        },
        "rom": { "path": rom_path, "sha256": rom_sha256 },
        "replay_save": replay_save_manifest,
        "replay_bundle": replay_bundle_manifest,
        "rom_random_replay": rom_random_manifest,
        "rom_random_authority": if live_oracle_rng {
            serde_json::json!({
                "mode": "live_oracle_trace",
                "artifact": LIVE_ORACLE_RNG_TRACE_ARTIFACT,
                "store_pc_low16": CARTRIDGE_RNG_STORE_PC_LOW16,
            })
        } else if rom_random_script.is_some() {
            serde_json::json!({"mode": "replay_script"})
        } else {
            serde_json::json!({"mode": "translated_fallback"})
        },
        "timing": {
            "fps": oracle.av_info.timing.fps,
            "sample_rate": oracle.av_info.timing.sample_rate,
            "frames_requested": frames,
            "start_frame": start_frame,
            "compare_from_frame": compare_from_frame,
            "fixed_oracle_startup_skip_frames": skip_oracle_frames,
            "dynamic_alignment": false,
        },
        "comparison_lanes": {
            "video": compare_video,
            "audio": compare_audio,
            "engine_state": compare_engine_state_from_frame.is_some(),
            "engine_state_from_frame": compare_engine_state_from_frame,
        },
        "av_hash_ledger": {
            "schema": 1,
            "coverage": "every compared frame for each enabled lane",
            "video_canonicalization": "visible row-major RGB bytes; alpha and libretro row padding excluded",
            "audio_canonicalization": "interleaved stereo signed 16-bit little-endian samples",
        },
        "audio": {
            "comparison": audio_comparison.as_str(),
            "window_sample_frames": timing.window_frames,
            "silence_threshold": timing.silence_threshold,
            "max_timing_error_sample_frames": timing.max_timing_error_frames,
            "max_envelope_error": timing.max_envelope_error,
        },
        "artifacts": artifacts,
    });
    fs::write(
        dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap_or_else(|e| {
        eprintln!("failed to write libretro session manifest: {e}");
        process::exit(1);
    });
    let absolute_dir = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    let repo_root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let asset_pack = repo_root.join("zelda3_assets.dat");
    let feature = "";
    let lane_flags = match (compare_video, compare_audio) {
        (true, true) => "",
        (false, true) => " --ignore-video",
        (true, false) => " --ignore-audio",
        (false, false) => " --ignore-video --ignore-audio",
    };
    let initial_state_flags = if start_frame == 0 && replay_save.is_none() {
        format!(
            "--load-sram {} --skip-oracle-frames {}",
            shell_single_quote(&absolute_dir.join("initial.srm").to_string_lossy()),
            skip_oracle_frames,
        )
    } else {
        format!(
            "--resume-rust-state {} --resume-oracle-state {}",
            shell_single_quote(&absolute_dir.join("rust_initial.z3state").to_string_lossy()),
            shell_single_quote(&absolute_dir.join("oracle_initial.state").to_string_lossy()),
        )
    };
    let rom_random_flags = if rom_random_script.is_some() {
        format!(
            " --rom-random-script {}",
            shell_single_quote(&absolute_dir.join("rom-random.txt").to_string_lossy()),
        )
    } else {
        String::new()
    };
    let live_oracle_rng_flag = if live_oracle_rng {
        " --live-oracle-rng"
    } else {
        ""
    };
    let scan_all_flag = if scan_all { " --scan-all" } else { "" };
    let engine_state_flag = match compare_engine_state_from_frame {
        Some(start) if start != compare_from_frame => {
            format!(" --compare-engine-state-from-frame {start}")
        }
        Some(_) => String::new(),
        None => " --ignore-engine-state".to_string(),
    };
    let replay = format!(
        "#!/bin/sh\nset -eu\ncd {}\nZELDA3_ASSET_PACK={} cargo run -q -p zelda3-bin{} -- --compare-snes9x-oracle {} {} {} --expected-core-sha256 {} --expected-rom-sha256 {} --input-script {}{}{} {} --compare-from-frame {}{}{} --audio-comparison {} --audio-window-ms {} --audio-silence-threshold {} --audio-timing-tolerance-ms {} --audio-envelope-tolerance {} --session-dir {}{}\n",
        shell_single_quote(&repo_root.to_string_lossy()),
        shell_single_quote(&asset_pack.to_string_lossy()),
        feature,
        shell_single_quote(core_path),
        shell_single_quote(rom_path),
        frames,
        core_sha256,
        rom_sha256,
        shell_single_quote(&absolute_dir.join("input.txt").to_string_lossy()),
        rom_random_flags,
        live_oracle_rng_flag,
        initial_state_flags,
        compare_from_frame,
        engine_state_flag,
        lane_flags,
        audio_comparison.as_str(),
        (timing.window_frames as f64 / oracle.av_info.timing.sample_rate) * 1000.0,
        timing.silence_threshold,
        (timing.max_timing_error_frames as f64 / oracle.av_info.timing.sample_rate) * 1000.0,
        timing.max_envelope_error,
        shell_single_quote(&absolute_dir.join("replay").to_string_lossy()),
        scan_all_flag,
    );
    fs::write(dir.join("replay.sh"), replay).unwrap_or_else(|e| {
        eprintln!("failed to write libretro session replay script: {e}");
        process::exit(1);
    });
    let file = fs::File::create(dir.join("frame_receipts.jsonl")).unwrap_or_else(|e| {
        eprintln!("failed to create libretro frame receipt: {e}");
        process::exit(1);
    });
    Some(BufWriter::new(file))
}

pub(crate) fn write_libretro_frame_receipt(
    writer: Option<&mut BufWriter<fs::File>>,
    frame: u32,
    input: u16,
    rust_audio_frames: usize,
    oracle_audio_frames: usize,
    video_width: u32,
    video_height: u32,
    rust_system_ram_before: &[u8],
    rust_system_ram: &[u8],
    rust_selected_game_load_remaining_before: u8,
    rust_selected_game_load_remaining: u8,
    rust_game_execution_scheduler: String,
    rust_poly_work: zelda3::zelda_rtl::PolyWorkMetrics,
    rust_poly_cycles: Option<u64>,
    rust_sfx_clock_checkpoint: (u32, u8, u8),
    rust_spc_driver_clock: Option<String>,
    rust_audio_events: &zelda3::game_output::AudioEventFrame,
    oracle_system_ram: Option<&[u8]>,
    rust_vram: &[u16],
    oracle_vram: Option<&[u8]>,
) {
    let Some(writer) = writer else {
        return;
    };
    let receipt = serde_json::json!({
        "frame": frame,
        "input": format!("0x{input:04x}"),
        "rust_audio_sample_frames": rust_audio_frames,
        "oracle_audio_sample_frames": oracle_audio_frames,
        "oracle_video_width": video_width,
        "oracle_video_height": video_height,
        "rust_engine_before": libretro_engine_state_receipt(rust_system_ram_before),
        "rust_engine": libretro_engine_state_receipt(rust_system_ram),
        "rust_selected_game_load_remaining_before": rust_selected_game_load_remaining_before,
        "rust_selected_game_load_remaining": rust_selected_game_load_remaining,
        "rust_game_execution_scheduler": rust_game_execution_scheduler,
        "rust_poly_work": rust_poly_work,
        "rust_poly_cycles": rust_poly_cycles,
        "rust_sfx_clock_checkpoint": {
            "epoch": rust_sfx_clock_checkpoint.0,
            "timer_cycles": rust_sfx_clock_checkpoint.1,
            "timer_accumulator": rust_sfx_clock_checkpoint.2,
        },
        "rust_spc_driver_clock": rust_spc_driver_clock,
        "rust_audio_command_queue": rust_audio_events.queue,
        "rust_audio_command_ports": rust_audio_events.queue.input,
        "rust_audio_event_count": rust_audio_events.events.len(),
        "rust_audio_event_hash": rust_audio_events.command_hash(),
        "rust_audio_events": rust_audio_events.events,
        "oracle_engine": oracle_system_ram.map(libretro_engine_state_receipt),
        "vram": vram_domain_receipt(rust_vram, oracle_vram),
    });
    serde_json::to_writer(&mut *writer, &receipt).unwrap_or_else(|e| {
        eprintln!("failed to write libretro frame receipt: {e}");
        process::exit(1);
    });
    writer.write_all(b"\n").unwrap_or_else(|e| {
        eprintln!("failed to terminate libretro frame receipt: {e}");
        process::exit(1);
    });
}

pub(crate) fn libretro_engine_state_receipt(ram: &[u8]) -> serde_json::Value {
    let byte = |address: usize| ram.get(address).copied().unwrap_or_default();
    let word =
        |address: usize| u16::from_le_bytes([byte(address), byte(address.saturating_add(1))]);
    let poly_buffer_nonzero_bytes = ram
        .get(0xe800..0xf000)
        .unwrap_or_default()
        .iter()
        .filter(|&&value| value != 0)
        .count();
    let ppu_oam_dma_shadow_hash = ram
        .get(0x0800..0x0a20)
        .unwrap_or_default()
        .iter()
        .fold(2_166_136_261u32, |hash, byte| {
            (hash ^ u32::from(*byte)).wrapping_mul(16_777_619)
        });
    let pending_tilemap_source_offset = word(0x0118);
    let pending_tilemap_source = ram
        .get(0x10000 + usize::from(pending_tilemap_source_offset)..)
        .and_then(|source| source.get(..0x200));
    let pending_tilemap_source_hash = pending_tilemap_source.map(|source| {
        source.iter().fold(2_166_136_261u32, |hash, byte| {
            (hash ^ u32::from(*byte)).wrapping_mul(16_777_619)
        })
    });
    let mut receipt = serde_json::json!({
        "main_module": byte(0x0010),
        "submodule": byte(0x0011),
        "subsubmodule": byte(0x00b0),
        "frame_counter": byte(0x001a),
        "screen_brightness": byte(0x0013),
        "attract_state": byte(0x0022),
        "attract_sequence": byte(0x0023),
        "attract_throne_fade_timer": byte(0x002c),
        "oam_priority_word": word(0x0064),
        "palette_filter_countdown": byte(0xc007),
        "vertical_irq_trigger": byte(0x00ff),
        "nmi_thread_active": byte(0x012a),
        "music_control": byte(0x012c),
        "last_music_control": byte(0x0133),
        "dialogue_message_index": word(0x1cf0),
        "messaging_module": byte(0x1cd8),
        "text_render_state": byte(0x1cd4),
        "vwf_line_speed_cur": byte(0x1cd5),
        "vwf_line_speed": byte(0x1cd6),
        "text_incremental_state": byte(0x1cd7),
        "dialogue_msg_read_pos": word(0x1cd9),
        "dialogue_msg_src_offs": word(0x1cdd),
        "dialogue_scroll_pixel": byte(0x1cdf),
        "text_wait_countdown": word(0x1ce0),
        "text_wait_countdown2": byte(0x1ce9),
        "dialogue_scroll_speed": byte(0x1cea),
        "shared_message_timer": word(0x02cd),
        "crystal_rotation_counter": byte(0x0649),
        "intro_step_index": byte(0x1e00),
        "intro_step_timer": byte(0x1e01),
        "intro_palette_flash_count": byte(0x0ff9),
        "intro_sword_sparkle_timer": byte(0x00ca),
        "intro_sword_sparkle_step": byte(0x00cb),
        "intro_sword_animation_step": byte(0x00cc),
        "intro_did_run_step": byte(0x1f00),
        "pending_polyhedral_update": byte(0x1f0c),
        "poly_config1": byte(0x1f02),
        "poly_angle_a": byte(0x1f04),
        "poly_angle_b": byte(0x1f05),
        "nmi_thread_stack": word(0x1f0a),
        "poly_buffer_nonzero_bytes": poly_buffer_nonzero_bytes,
    });
    receipt.as_object_mut().unwrap().insert(
        "resident_song_bank_kind".to_string(),
        serde_json::Value::from(byte(0x0136)),
    );
    if let Some(map) = receipt.as_object_mut() {
        for (name, value) in [
            ("link_animation_counter", u64::from(byte(0x002d))),
            ("link_animation_step", u64::from(byte(0x002e))),
            ("link_facing", u64::from(byte(0x002f))),
            ("link_last_direction", u64::from(byte(0x0026))),
            ("link_speed_setting", u64::from(byte(0x005e))),
            ("link_num_orthogonal_directions", u64::from(byte(0x006a))),
            ("link_direction_lock", u64::from(byte(0x0050))),
            ("link_direction_bits", u64::from(byte(0x0340))),
            ("link_flag_moving", u64::from(byte(0x034a))),
            ("link_dma_graphics_index", u64::from(word(0x0100))),
            ("link_dma_body_top", u64::from(word(0x0acc))),
            ("link_dma_head_top", u64::from(word(0x0ad0))),
        ] {
            map.insert(name.into(), value.into());
        }
        map.insert("sprite_graphics_index".into(), byte(0x0aa3).into());
        map.insert(
            "sprite_graphics_subsets".into(),
            serde_json::json!([byte(0xc2fc), byte(0xc2fd), byte(0xc2fe), byte(0xc2ff),]),
        );
        map.insert("bg_tile_animation_countdown".into(), word(0xc00d).into());
        map.insert("link_dma_source_offset".into(), word(0xc00f).into());
        map.insert("link_dma_countdown".into(), word(0xc013).into());
        map.insert("link_dma_tile_offset".into(), word(0xc015).into());
        map.insert("bg1_h_copy2".into(), word(0x00e0).into());
        map.insert("bg1_v_copy2".into(), word(0x00e6).into());
        map.insert("move_overlay_counter".into(), byte(0x0494).into());
        map.insert(
            "pending_tilemap_destination_page".into(),
            byte(0x0019).into(),
        );
        map.insert(
            "pending_tilemap_source_offset".into(),
            pending_tilemap_source_offset.into(),
        );
        map.insert(
            "incremental_vram_upload_counter".into(),
            byte(0x0412).into(),
        );
        map.insert(
            "pending_tilemap_source_hash".into(),
            pending_tilemap_source_hash.into(),
        );
        map.insert(
            "pending_tilemap_source_prefix".into(),
            pending_tilemap_source
                .map(|source| source.iter().take(8).copied().collect::<Vec<_>>())
                .into(),
        );
    }

    let object = receipt
        .as_object_mut()
        .expect("engine receipt is an object");
    object.insert(
        "player_equipment_oam_shadow".into(),
        (0x0800 + 112 * 4..0x0800 + 114 * 4)
            .map(byte)
            .collect::<Vec<_>>()
            .into(),
    );
    object.insert(
        "oam_slots".into(),
        (0..128)
            .map(|slot| {
                let base = 0x0800 + slot * 4;
                serde_json::json!({
                    "slot": slot,
                    "x": byte(base),
                    "y": byte(base + 1),
                    "tile": byte(base + 2),
                    "flags": byte(base + 3),
                })
            })
            .collect::<Vec<_>>()
            .into(),
    );
    object.insert(
        "oam_extended".into(),
        (0x0a00..0x0a20).map(byte).collect::<Vec<_>>().into(),
    );
    for (name, value) in [
        ("intro_sword_y", u64::from(word(0x00c8))),
        ("intro_sword_sparkle_y_offset", u64::from(byte(0x00cd))),
        ("nmi_update_latch", u64::from(byte(0x0012))),
        ("nmi_bg_vram_load_mode", u64::from(byte(0x0014))),
        ("nmi_subroutine_index", u64::from(byte(0x0017))),
        ("nmi_load_target_address", u64::from(word(0x0116))),
        ("nmi_core_update_disable", u64::from(byte(0x0710))),
        (
            "ppu_oam_dma_shadow_hash",
            u64::from(ppu_oam_dma_shadow_hash),
        ),
        ("ambient_sound_effect", u64::from(byte(0x012d))),
        ("sound_effect_1", u64::from(byte(0x012e))),
        ("sound_effect_2", u64::from(byte(0x012f))),
        ("queued_music_control", u64::from(byte(0x0132))),
        ("spotlight_window_radius", u64::from(word(0x067c))),
        ("spotlight_window_state", u64::from(word(0x067e))),
        ("dungeon_room_index", u64::from(word(0x00a0))),
        ("dungeon_staircase_index", u64::from(byte(0x0462))),
        ("dungeon_staircase_counter", u64::from(byte(0x0464))),
        ("joypad_high", u64::from(byte(0x00f0))),
        ("joypad_low", u64::from(byte(0x00f2))),
        ("joypad_high_filtered", u64::from(byte(0x00f4))),
        ("joypad_low_filtered", u64::from(byte(0x00f6))),
        ("link_x", u64::from(word(0x0022))),
        ("link_y", u64::from(word(0x0020))),
        ("link_last_direction", u64::from(byte(0x0026))),
        ("link_actual_y_velocity", u64::from(byte(0x0027))),
        ("link_actual_x_velocity", u64::from(byte(0x0028))),
        ("link_y_subpixel", u64::from(byte(0x002a))),
        ("link_x_subpixel", u64::from(byte(0x002b))),
        ("link_animation_counter", u64::from(byte(0x002d))),
        ("link_animation_step", u64::from(byte(0x002e))),
        ("link_direction", u64::from(byte(0x0067))),
        ("link_facing_direction", u64::from(byte(0x002f))),
        ("link_auxiliary_state", u64::from(byte(0x004d))),
        ("link_direction_lock", u64::from(byte(0x0050))),
        ("link_handler_state", u64::from(byte(0x005d))),
        ("link_speed_setting", u64::from(byte(0x005e))),
        ("link_orthogonal_direction_count", u64::from(byte(0x006a))),
        ("link_animation_timer_step", u64::from(byte(0x030a))),
        ("link_somaria_platform_state", u64::from(byte(0x02f5))),
        ("link_facing_direction_mirror", u64::from(byte(0x0323))),
        ("link_movement_direction_bits", u64::from(byte(0x0340))),
        ("link_moving_flag", u64::from(byte(0x034a))),
        ("link_water_ripple_or_grass", u64::from(byte(0x0351))),
        ("link_sort_sprites_offset", u64::from(word(0x0352))),
        ("link_player_oam_value", u64::from(byte(0x0354))),
        ("link_sprite_oam_state_timer", u64::from(byte(0x005c))),
        ("link_item_in_hand", u64::from(byte(0x0301))),
        ("link_state_bits", u64::from(byte(0x0308))),
        ("link_picking_throw_state", u64::from(byte(0x0309))),
        ("link_tile_action", u64::from(byte(0x036c))),
        ("link_lift_x_low", u64::from(byte(0x0368))),
        ("link_lift_x_high", u64::from(byte(0x036a))),
        ("sprite_pickup_slot", u64::from(byte(0x0fb2))),
        ("attract_scene_timer", u64::from(byte(0x0025))),
        ("attract_vram_destination", u64::from(word(0x0030))),
        ("attract_prison_soldier_x", u64::from(byte(0x0034))),
        ("attract_scene_frame_counter", u64::from(byte(0x0050))),
        ("attract_scene_substep", u64::from(byte(0x0060))),
    ] {
        object.insert(name.to_string(), serde_json::Value::from(value));
    }
    object.insert(
        "title_sword_oam_shadow".to_string(),
        serde_json::Value::Array(
            ram.get(0x0948..0x0970)
                .unwrap_or_default()
                .iter()
                .copied()
                .map(serde_json::Value::from)
                .collect(),
        ),
    );
    object.insert(
        "sprite_slots".to_string(),
        serde_json::Value::Array(
            (0..16)
                .map(|slot| {
                    serde_json::json!({
                        "slot": slot,
                        "state": byte(0x0dd0 + slot),
                        "type": byte(0x0e20 + slot),
                        "x": u16::from(byte(0x0d10 + slot)) | (u16::from(byte(0x0d30 + slot)) << 8),
                        "x_subpixel": byte(0x0d70 + slot),
                        "x_velocity": byte(0x0d50 + slot),
                        "y": u16::from(byte(0x0d00 + slot)) | (u16::from(byte(0x0d20 + slot)) << 8),
                        "y_subpixel": byte(0x0d60 + slot),
                        "y_velocity": byte(0x0d40 + slot),
                        "direction": byte(0x0de0 + slot),
                        "head_direction": byte(0x0eb0 + slot),
                        "graphics": byte(0x0dc0 + slot),
                        "ai_state": byte(0x0d80 + slot),
                        "wall_collision": byte(0x0e70 + slot),
                        "subtype": byte(0x0e30 + slot),
                        "subtype2": byte(0x0e80 + slot),
                        "delay_main": byte(0x0df0 + slot),
                        "delay_aux1": byte(0x0e00 + slot),
                    })
                })
                .collect(),
        ),
    );
    object.insert(
        "ancilla_slots".to_string(),
        serde_json::Value::Array(
            (0..10)
                .map(|slot| {
                    serde_json::json!({
                        "slot": slot,
                        "type": byte(0x0c4a + slot),
                        "x": u16::from(byte(0x0c04 + slot)) | (u16::from(byte(0x0c18 + slot)) << 8),
                        "x_subpixel": byte(0x0c40 + slot),
                        "x_velocity": byte(0x0c2c + slot),
                        "y": u16::from(byte(0x0bfa + slot)) | (u16::from(byte(0x0c0e + slot)) << 8),
                        "y_subpixel": byte(0x0c36 + slot),
                        "y_velocity": byte(0x0c22 + slot),
                        "item_to_link": byte(0x0c5e + slot),
                        "timer": byte(0x0c68 + slot),
                        "direction": byte(0x0c72 + slot),
                        "step": byte(0x0c54 + slot),
                        "aux_timer": byte(0x03b1 + slot),
                        "object_priority": byte(0x0280 + slot),
                        "num_sprites": byte(0x0c90 + slot),
                        "tile_attribute": byte(0x03e4 + slot),
                    })
                })
                .collect(),
        ),
    );
    receipt
}

pub(crate) fn oracle_music_route_state(ram: &[u8]) -> Option<[u8; 3]> {
    Some([
        *ram.get(ORACLE_MUSIC_CONTROL)?,
        *ram.get(ORACLE_QUEUED_MUSIC_CONTROL)?,
        *ram.get(ORACLE_LAST_MUSIC_CONTROL)?,
    ])
}

pub(crate) fn finalize_libretro_session(
    session_dir: Option<&Path>,
    writer: Option<&mut BufWriter<fs::File>>,
    av_hash_writer: Option<&mut BufWriter<fs::File>>,
    input_history: &[(u32, u16)],
    source_input_script: Option<&[u8]>,
    audio_report: Option<&libretro_timeline::AudioComparisonReport>,
    audio_frame_ends: &[u64],
    oracle_last_before: &[u8],
    oracle_last_before_frame: u32,
    oracle: &LibretroCore,
    game: &ZeldaState,
    frames: u32,
    video_mismatch_ranges: &[(u32, u32)],
    first_video_mismatch: Option<&str>,
    first_engine_state_mismatch: Option<&(u32, Vec<String>)>,
    diagnostic_probe: bool,
) {
    let Some(dir) = session_dir else {
        return;
    };
    if let Some(writer) = writer {
        writer.flush().unwrap_or_else(|e| {
            eprintln!("failed to flush libretro frame receipts: {e}");
            process::exit(1);
        });
    }
    if let Some(writer) = av_hash_writer {
        writer.flush().unwrap_or_else(|e| {
            eprintln!("failed to flush canonical A/V hash ledger: {e}");
            process::exit(1);
        });
    }
    let input_artifact = replayable_input_artifact(source_input_script, input_history);
    fs::write(dir.join("input.txt"), &input_artifact).unwrap_or_else(|e| {
        eprintln!("failed to write captured controller stream: {e}");
        process::exit(1);
    });
    fs::write(
        dir.join("audio_frame_ends.json"),
        serde_json::to_vec(audio_frame_ends).unwrap(),
    )
    .unwrap_or_else(|e| {
        eprintln!("failed to write audio frame boundaries: {e}");
        process::exit(1);
    });
    if let Some(report) = audio_report {
        fs::write(
            dir.join("audio_report.json"),
            serde_json::to_vec_pretty(report).unwrap(),
        )
        .unwrap_or_else(|e| {
            eprintln!("failed to write continuous audio report: {e}");
            process::exit(1);
        });
    }
    fs::write(dir.join("oracle_last_before.state"), oracle_last_before).unwrap_or_else(|e| {
        eprintln!("failed to write last pre-frame oracle state: {e}");
        process::exit(1);
    });
    let oracle_final = oracle.serialize_state().unwrap_or_else(|e| {
        eprintln!("failed to serialize final libretro state: {e}");
        process::exit(1);
    });
    fs::write(dir.join("oracle_final.state"), oracle_final).unwrap_or_else(|e| {
        eprintln!("failed to write final libretro state: {e}");
        process::exit(1);
    });
    // Recorded chapter inputs are segment-local. This independently replayed
    // endpoint becomes frame zero when it is paired with the next boundary.
    let rust_final = PlayCrashCheckpoint {
        magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
        host_frame: 0,
        input: input_history.last().map(|(_, input)| *input).unwrap_or(0),
        run_what: select_run_what(&game.ram),
        game: game.clone(),
    };
    fs::write(
        dir.join("rust_final.z3state"),
        bincode::serialize(&rust_final).expect("serialize final Rust parity state"),
    )
    .unwrap_or_else(|e| {
        eprintln!("failed to write final Rust parity state: {e}");
        process::exit(1);
    });
    // `ZELDA3_REPLAY_WRAM_DUMP=<path>` mirrors the replay-save diagnostic in
    // the compare harness: the full 128KB Rust WRAM at the final compared
    // frame, for byte-diffing against the oracle savestate's RAM block.
    if let Some(path) = std::env::var_os("ZELDA3_REPLAY_WRAM_DUMP") {
        if let Err(e) = fs::write(&path, &game.ram[..]) {
            eprintln!("failed to write WRAM dump to {path:?}: {e}");
        }
    }
    let matched = audio_report.map(|report| report.matched).unwrap_or(true)
        && video_mismatch_ranges.is_empty()
        && first_engine_state_mismatch.is_none();
    let parity_eligible = audio_report
        .map(|report| report.mode == AudioComparisonMode::Exact.as_str())
        .unwrap_or(true)
        // A live-RNG run resumed from a paired checkpoint is a fix-loop probe.
        && !diagnostic_probe;
    let status = if !matched {
        "failed"
    } else if parity_eligible {
        "passed"
    } else {
        "diagnostic_passed"
    };
    let result = serde_json::json!({
        "status": status,
        "parity_eligible": parity_eligible,
        "coverage_label": if parity_eligible {
            "exact parity for requested lanes"
        } else {
            "timing diagnostic only; not full parity"
        },
        "frames_completed": frames,
        "audio": audio_report,
        "video": {
            "matched": video_mismatch_ranges.is_empty(),
            "mismatch_ranges": video_mismatch_ranges,
            "first_mismatch": first_video_mismatch,
        },
        "engine_state": {
            "matched": first_engine_state_mismatch.is_none(),
            "first_mismatch": first_engine_state_mismatch.map(|(frame, mismatches)| serde_json::json!({
                "frame": frame,
                "mismatches": mismatches,
            })),
        },
        "dynamic_alignment": false,
        "rust_endpoint": "rust_final.z3state",
    });
    fs::write(
        dir.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap_or_else(|e| {
        eprintln!("failed to write libretro session result: {e}");
        process::exit(1);
    });
    let manifest_path = dir.join("manifest.json");
    let bytes = fs::read(&manifest_path).unwrap_or_else(|error| {
        eprintln!("failed to read libretro session manifest: {error}");
        process::exit(1);
    });
    let mut manifest =
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap_or_else(|error| {
            eprintln!("failed to parse libretro session manifest: {error}");
            process::exit(1);
        });
    manifest["status"] = serde_json::Value::String(status.to_string());
    manifest["parity_eligible"] = serde_json::Value::Bool(parity_eligible);
    manifest["frames_completed"] = serde_json::Value::from(frames);
    manifest["oracle_last_before"] = serde_json::json!({
        "artifact": "oracle_last_before.state",
        "frame": oracle_last_before_frame,
        "policy": "initial checkpoint only; authoritative execution performs no in-run serialization",
    });
    manifest["input_replay"] = serde_json::json!({
        "artifact": "input.txt",
        "mode": if source_input_script.is_some() {
            "source_script"
        } else {
            "captured_history"
        },
        "sha256": parity::evidence::sha256_bytes(&input_artifact),
    });
    fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap_or_else(
        |error| {
            eprintln!("failed to finalize libretro session manifest: {error}");
            process::exit(1);
        },
    );
}

fn replayable_input_artifact(
    source_input_script: Option<&[u8]>,
    input_history: &[(u32, u16)],
) -> Vec<u8> {
    source_input_script.map_or_else(
        || format_input_history(input_history).into_bytes(),
        <[u8]>::to_vec,
    )
}

pub(crate) fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub(crate) fn compare_snes9x_video_frame(
    rust_frame: &[u8],
    rust_width: u32,
    rust_height: u32,
    snes9x: &LibretroFrame,
) -> Option<String> {
    compare_libretro_video_frame(rust_frame, rust_width, rust_height, snes9x, 0, 0)
}

fn canonical_audio_digest(samples: &[i16]) -> serde_json::Value {
    let mut digest = parity::evidence::Sha256Digest::new();
    digest.update_i16_le(samples);
    serde_json::json!({
        "sample_frames": samples.len() / 2,
        "channels": 2,
        "sha256": digest.finish(),
    })
}

fn canonical_rust_video_digest(
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Result<serde_json::Value, String> {
    let pixels = width as usize * height as usize;
    let expected = pixels
        .checked_mul(4)
        .ok_or_else(|| format!("Rust video geometry overflows: {width}x{height}"))?;
    if rgba.len() != expected {
        return Err(format!(
            "Rust RGBA byte count {} does not match {width}x{height} ({expected})",
            rgba.len()
        ));
    }
    let mut digest = parity::evidence::Sha256Digest::new();
    digest.update_rgb_from_rgba(std::iter::once(rgba));
    Ok(serde_json::json!({
        "width": width,
        "height": height,
        "sha256": digest.finish(),
    }))
}

fn canonical_oracle_video_digest(frame: &LibretroFrame) -> Result<serde_json::Value, String> {
    if frame.video.is_empty() {
        return Err("oracle provided no video frame".to_string());
    }
    let stride = snes9x_pixel_stride(frame.pixel_format)
        .ok_or_else(|| format!("unsupported oracle pixel format {}", frame.pixel_format))?;
    let visible_row_bytes = frame.video_width as usize * stride;
    if frame.video_pitch < visible_row_bytes {
        return Err(format!(
            "oracle pitch {} is smaller than visible row {}",
            frame.video_pitch, visible_row_bytes
        ));
    }
    let required =
        frame.video_height.saturating_sub(1) as usize * frame.video_pitch + visible_row_bytes;
    if frame.video.len() < required {
        return Err(format!(
            "oracle video byte count {} is smaller than required {required}",
            frame.video.len()
        ));
    }
    let mut digest = parity::evidence::Sha256Digest::new();
    for y in 0..frame.video_height as usize {
        for x in 0..frame.video_width as usize {
            let offset = y * frame.video_pitch + x * stride;
            let [r, g, b, _] = snes9x_rgba_pixel_at(frame, offset)
                .ok_or_else(|| format!("cannot decode oracle pixel ({x}, {y})"))?;
            digest.update(&[r, g, b]);
        }
    }
    Ok(serde_json::json!({
        "width": frame.video_width,
        "height": frame.video_height,
        "sha256": digest.finish(),
    }))
}

fn canonical_video_digest_pair(
    rust_rgba: &[u8],
    rust_width: u32,
    rust_height: u32,
    oracle: &LibretroFrame,
) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "rust": canonical_rust_video_digest(rust_rgba, rust_width, rust_height)?,
        "oracle": canonical_oracle_video_digest(oracle)?,
    }))
}

fn write_av_hash_record(
    writer: Option<&mut BufWriter<fs::File>>,
    frame: u32,
    input: u16,
    oracle_audio_sample_frames: usize,
    video: Option<serde_json::Value>,
    audio: Option<serde_json::Value>,
) {
    let Some(writer) = writer else {
        return;
    };
    serde_json::to_writer(
        &mut *writer,
        &serde_json::json!({
            "schema": 1,
            "frame": frame,
            "input": format!("0x{input:04x}"),
            "oracle_audio_sample_frames": oracle_audio_sample_frames,
            "video": video,
            "audio": audio,
        }),
    )
    .unwrap_or_else(|error| {
        eprintln!("failed to write canonical A/V hash record: {error}");
        process::exit(1);
    });
    writer.write_all(b"\n").unwrap_or_else(|error| {
        eprintln!("failed to terminate canonical A/V hash record: {error}");
        process::exit(1);
    });
}

pub(crate) fn compare_libretro_video_frame(
    rust_frame: &[u8],
    rust_width: u32,
    rust_height: u32,
    libretro: &LibretroFrame,
    color_tolerance: u8,
    max_mismatched_pixels: usize,
) -> Option<String> {
    if libretro.video.is_empty() {
        return Some("missing libretro video frame".to_string());
    }
    if libretro.video_width != rust_width || libretro.video_height != rust_height {
        return Some(format!(
            "geometry rust={}x{} libretro={}x{} pitch={} pixel_format={}",
            rust_width,
            rust_height,
            libretro.video_width,
            libretro.video_height,
            libretro.video_pitch,
            libretro.pixel_format
        ));
    }
    let mut mismatched = 0usize;
    let mut first = None;
    // Keep a bounded set of samples in the receipt.  A pixel count and the
    // first coordinate alone cannot distinguish a broad timing failure from a
    // small compositor edge case (for example, a backdrop color-math pixel).
    let mut samples = Vec::with_capacity(4);
    for y in 0..rust_height as usize {
        for x in 0..rust_width as usize {
            let pixel_index = y * rust_width as usize + x;
            let rust_offset = pixel_index * 4;
            let snes9x_offset =
                y * libretro.video_pitch + x * snes9x_pixel_stride(libretro.pixel_format)?;
            let mine = rgba_pixel_at(rust_frame, rust_offset)?;
            let theirs = snes9x_rgba_pixel_at(libretro, snes9x_offset)?;
            if !rgb_within_tolerance(mine, theirs, color_tolerance) {
                mismatched += 1;
                first.get_or_insert((x, y, mine, theirs));
                if samples.len() < 4 {
                    samples.push((x, y, mine, theirs));
                }
            }
        }
    }
    if mismatched <= max_mismatched_pixels {
        return None;
    }
    first.map(|(x, y, mine, theirs)| {
        format!(
            "mismatched_pixels={mismatched}; allowed_mismatched_pixels={max_mismatched_pixels}; color_tolerance={color_tolerance}; first_mismatch=({x}, {y}) rust={mine:02x?} libretro={theirs:02x?}; samples={samples:02x?}; pixel_format={} pitch={}",
            libretro.pixel_format, libretro.video_pitch
        )
    })
}

pub(crate) fn rgb_within_tolerance(mine: [u8; 4], theirs: [u8; 4], tolerance: u8) -> bool {
    mine[..3]
        .iter()
        .zip(theirs[..3].iter())
        .all(|(&mine, &theirs)| mine.abs_diff(theirs) <= tolerance)
}

pub(crate) fn align_snes9x_video_capture(
    snes9x: &mut LibretroCore,
    mut capture: LibretroFrame,
    rust_frame: &[u8],
    width: u32,
    height: u32,
    input: u16,
    max_extra_frames: u32,
    color_tolerance: u8,
    max_mismatched_pixels: usize,
) -> (LibretroFrame, u32, bool) {
    if compare_libretro_video_frame(
        rust_frame,
        width,
        height,
        &capture,
        color_tolerance,
        max_mismatched_pixels,
    )
    .is_none()
    {
        return (capture, 0, true);
    }
    for extra in 1..=max_extra_frames {
        capture = snes9x.run_frame_with_input(input);
        if compare_libretro_video_frame(
            rust_frame,
            width,
            height,
            &capture,
            color_tolerance,
            max_mismatched_pixels,
        )
        .is_none()
        {
            return (capture, extra, true);
        }
    }
    println!("auto-align video found no RGB match within {max_extra_frames} extra snes9x frame(s)");
    (capture, max_extra_frames, false)
}

pub(crate) fn rgba_pixel_at(frame: &[u8], offset: usize) -> Option<[u8; 4]> {
    let bytes = frame.get(offset..offset + 4)?;
    Some([bytes[0], bytes[1], bytes[2], bytes[3]])
}

pub(crate) fn snes9x_pixel_stride(pixel_format: u32) -> Option<usize> {
    match pixel_format {
        0 | 2 => Some(2),
        1 => Some(4),
        _ => None,
    }
}

fn snes9x_presented_scanline_for_video_y(video_height: usize, video_y: usize) -> usize {
    // The trace core exposes Snes9x's uncropped scanline caches, while the libretro video
    // callback can crop the top overscan rows. Keep the translation next to the consumer so
    // pixel-owner diagnostics cannot silently inspect a different scanline than the RGBA pixel.
    let top_crop = match video_height {
        224 => 7,
        448 => 14,
        _ => 0,
    };
    video_y + top_crop
}

pub(crate) fn snes9x_rgba_pixel_at(frame: &LibretroFrame, offset: usize) -> Option<[u8; 4]> {
    match frame.pixel_format {
        0 => {
            let lo = *frame.video.get(offset)? as u16;
            let hi = *frame.video.get(offset + 1)? as u16;
            let raw = lo | (hi << 8);
            Some([
                expand_5_to_8((raw >> 10) & 0x1f),
                expand_5_to_8((raw >> 5) & 0x1f),
                expand_5_to_8(raw & 0x1f),
                0xff,
            ])
        }
        1 => {
            let bytes = frame.video.get(offset..offset + 4)?;
            Some([bytes[2], bytes[1], bytes[0], 0xff])
        }
        2 => {
            let lo = *frame.video.get(offset)? as u16;
            let hi = *frame.video.get(offset + 1)? as u16;
            let raw = lo | (hi << 8);
            Some([
                expand_5_to_8((raw >> 11) & 0x1f),
                // Snes9x expands the SNES five-bit green channel into RGB565.
                // Collapse the duplicated low bit before comparing with the
                // modern renderer's RGB555-equivalent output.
                expand_5_to_8(((raw >> 5) & 0x3f) >> 1),
                expand_5_to_8(raw & 0x1f),
                0xff,
            ])
        }
        _ => None,
    }
}

pub(crate) fn expand_5_to_8(value: u16) -> u8 {
    ((value << 3) | (value >> 2)) as u8
}

pub(crate) fn render_full_apu_audio(
    apu: &mut snes::apu::ApuState,
    audio: &mut [i16],
    samples: usize,
    channels: usize,
) {
    let mut guard = 0usize;
    while apu.dsp.sample_offset < 534 && guard < 32_000 {
        apu.cycle();
        guard += 1;
    }
    if apu.dsp.sample_offset < 534 {
        audio.fill(0);
        return;
    }
    apu.dsp.get_samples(audio, samples, channels);
}

pub(crate) fn render_full_apu_audio_exact(
    apu: &mut snes::apu::ApuState,
    audio: &mut [i16],
    samples: usize,
    channels: usize,
) -> Result<(), String> {
    let mut guard = 0usize;
    let guard_limit = samples.saturating_mul(32).saturating_add(64);
    while (apu.dsp.sample_offset as usize) < samples && guard < guard_limit {
        apu.cycle();
        guard += 1;
    }
    if (apu.dsp.sample_offset as usize) < samples {
        return Err(format!(
            "APU produced only {} of {samples} requested exact samples after {guard} clocks",
            apu.dsp.sample_offset
        ));
    }
    apu.dsp.drain_samples_exact(audio, samples, channels)
}

#[derive(Serialize)]
pub(crate) struct ParityFailureReport {
    pub(crate) kind: String,
    pub(crate) frame: u32,
    pub(crate) input: String,
    pub(crate) run_what: Option<u8>,
    pub(crate) message: String,
    pub(crate) trace_mine: Option<String>,
    pub(crate) trace_theirs: Option<String>,
    pub(crate) ppu_mine: Option<String>,
    pub(crate) ppu_theirs: Option<String>,
    pub(crate) audio_mine: Option<String>,
    pub(crate) audio_theirs: Option<String>,
    pub(crate) artifacts: Vec<String>,
    pub(crate) notes: Vec<String>,
}

/// Serialize GPU oracle work machine-wide. Two concurrent offscreen GPU runs
/// corrupt each other twice over: they stomp the shared comparison session
/// directory, and concurrent offscreen GPU work is a documented source of
/// nondeterministic render flakes. Refuse to start a second run instead of
/// producing garbage results. The lock lives in the binary (not just the
/// recorder script) so raw `--compare-snes9x-oracle` invocations and orphaned
/// runs are covered too. The returned handle must stay alive for the whole
/// oracle session.
pub(crate) fn acquire_snes9x_compare_lock() -> fs::File {
    acquire_snes9x_compare_lock_mode(true)
}

/// `exclusive: false` is a renderless run (no GPU work, its own session dir):
/// it takes NO lock at all, so renderless probes, oracle-only captures, and a
/// long-running exclusive (rendering) gate all coexist. The two documented
/// hazards — concurrent offscreen GPU flakes and the shared comparison session
/// directory — only exist between rendering runs, which still serialize on the
/// exclusive lock. (An earlier design gave renderless runs a shared flock; that
/// made a 1-2 h cold A/V gate refuse every renderless fix-loop probe and vice
/// versa for no safety gain.)
pub(crate) fn acquire_snes9x_compare_lock_mode(exclusive: bool) -> fs::File {
    use std::os::unix::io::AsRawFd;
    let path = Path::new("/tmp/zelda3-snes9x-compare.lock");
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .unwrap_or_else(|error| {
            eprintln!("failed to open {}: {error}", path.display());
            process::exit(2);
        });
    if !exclusive {
        return file;
    }
    let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if rc != 0 {
        eprintln!(
            "another rendering Snes9x comparison is already running (lock: {}); GPU comparisons must run serially (renderless runs take no lock)",
            path.display()
        );
        process::exit(2);
    }
    file
}

/// Seed a fresh translated state from the loaded oracle core's memory (WRAM,
/// VRAM, SRAM via the libretro memory map; CGRAM and OAM via the trace core's
/// debug PPU accessor). Used by `--seed-rust-from-oracle-state` to start a
/// route segment at a Snes9x boundary state without any Rust checkpoint.
fn seed_rust_game_from_oracle_memory(
    game: &mut ZeldaState,
    oracle: &LibretroCore,
    frame: u32,
) -> Result<(), String> {
    let wram = oracle
        .memory_bytes(RETRO_MEMORY_SYSTEM_RAM)
        .ok_or("oracle exposes no SYSTEM_RAM")?
        .to_vec();
    let vram = oracle
        .memory_bytes(RETRO_MEMORY_VIDEO_RAM)
        .ok_or("oracle exposes no VIDEO_RAM")?
        .to_vec();
    let sram = oracle
        .memory_bytes(RETRO_MEMORY_SAVE_RAM)
        .ok_or("oracle exposes no SAVE_RAM")?
        .to_vec();
    let cgram = (0..256)
        .map(|index| {
            oracle
                .debug_ppu_value(2, index)
                .and_then(|value| u16::try_from(value).ok())
                .ok_or_else(|| {
                    format!("oracle CGRAM color {index} is unavailable (trace core required)")
                })
        })
        .collect::<Result<Vec<u16>, String>>()?;
    let oam = (0..544)
        .map(|index| {
            oracle
                .debug_ppu_value(15, index)
                .and_then(|value| u8::try_from(value).ok())
                .ok_or_else(|| {
                    format!("oracle OAM byte {index} is unavailable (trace core required)")
                })
        })
        .collect::<Result<Vec<u8>, String>>()?;
    // `seed_from_snes9x_oracle_memory` re-arms the live timing owner itself;
    // `restore_live_rom_timing_after_checkpoint` would invalidate it again.
    game.seed_from_snes9x_oracle_memory(&wram, &vram, &sram, &cgram, &oam, frame)?;
    Ok(())
}

/// Failure artifacts accumulate ~5-10MB per diverging run; keep only the most
/// recent runs instead of growing target/parity-failures without bound.
pub(crate) const PARITY_FAILURE_DIRS_KEPT: usize = 20;

pub(crate) fn prune_parity_failure_dirs(root: &Path) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| entry.path())
        .collect();
    // Directory names start with the unix-seconds timestamp, so the
    // lexicographic order is the chronological order.
    dirs.sort();
    while dirs.len() >= PARITY_FAILURE_DIRS_KEPT {
        let dir = dirs.remove(0);
        let _ = fs::remove_dir_all(&dir);
    }
}

pub(crate) fn create_parity_failure_dir() -> Result<PathBuf, Box<dyn Error>> {
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let root = PathBuf::from("target").join("parity-failures");
    prune_parity_failure_dirs(&root);
    let dir = root.join(format!("{seconds}-{}", process::id()));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub(crate) fn write_parity_diff(
    dir: &Path,
    report: &ParityFailureReport,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut artifacts = report.artifacts.clone();
    let diff = serde_json::to_string_pretty(report)?;
    fs::write(dir.join("diff.json"), diff)?;
    artifacts.push("diff.json".to_string());
    Ok(artifacts)
}

pub(crate) fn write_wav_i16_stereo(
    path: &Path,
    samples: &[i16],
    sample_rate: u32,
    channels: u16,
) -> Result<(), Box<dyn Error>> {
    let mut file = BufWriter::new(fs::File::create(path)?);
    let data_bytes = (samples.len() * 2) as u32;
    let byte_rate = sample_rate * channels as u32 * 2;
    let block_align = channels * 2;
    file.write_all(b"RIFF")?;
    file.write_all(&(36 + data_bytes).to_le_bytes())?;
    file.write_all(b"WAVEfmt ")?;
    file.write_all(&16u32.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&16u16.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&data_bytes.to_le_bytes())?;
    for &sample in samples {
        file.write_all(&sample.to_le_bytes())?;
    }
    Ok(())
}

pub(crate) fn write_libretro_parity_failure_artifacts(
    pre_game: Option<&ZeldaState>,
    post_game: &ZeldaState,
    rust_frame_rgba: &[u8],
    rust_audio: &[i16],
    capture: &LibretroFrame,
    oracle_before_state: &[u8],
    oracle_after_state: &[u8],
    rust_before_ram: &[u8],
    input_history: &[(u32, u16)],
    frame: u32,
    input: u16,
    sample_rate: u32,
    oracle_name: &str,
    oracle_system_ram: Option<&[u8]>,
    oracle_before_vram: Option<&[u8]>,
    rendered_display: Option<&crate::gpu_capture::LiveGpuFrameCapture>,
    display_oracle_receipt: Option<&DisplayOracleReceipt>,
    oracle_presented_oam: Option<&[u8]>,
    message: String,
) -> Result<PathBuf, Box<dyn Error>> {
    let dir = create_parity_failure_dir()?;
    fs::write(dir.join("input.txt"), format_input_history(input_history))?;
    fs::write(dir.join("rust_before_ram.bin"), rust_before_ram)?;
    // The comparison loop no longer clones the full pre-frame state every
    // frame; the pre-state artifact exists only when the loop had one on hand
    // (poly frames). input.txt + the initial states reproduce it otherwise.
    if let Some(pre_game) = pre_game {
        let rust_checkpoint = PlayCrashCheckpoint {
            magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
            host_frame: frame,
            input,
            run_what: RUN_MAIN,
            game: pre_game.clone(),
        };
        fs::write(
            dir.join("rust_before.z3state"),
            bincode::serialize(&rust_checkpoint)?,
        )?;
    }
    fs::write(dir.join("oracle_before.state"), oracle_before_state)?;
    fs::write(dir.join("oracle_after.state"), oracle_after_state)?;
    if let Some(oracle_before_ram) = snes9x_state_section(oracle_before_state, b"RAM") {
        fs::write(dir.join("oracle_before_ram.bin"), oracle_before_ram)?;
    }
    if let Some(oracle_before_vram) = oracle_before_vram {
        fs::write(dir.join("oracle_before_vram.bin"), oracle_before_vram)?;
    }
    let rust_after_checkpoint = PlayCrashCheckpoint {
        magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
        host_frame: frame.saturating_add(1),
        input,
        run_what: RUN_MAIN,
        game: post_game.clone(),
    };
    fs::write(
        dir.join("rust_after.z3state"),
        bincode::serialize(&rust_after_checkpoint)?,
    )?;
    let rust_vram = post_game
        .ppu
        .vram
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect::<Vec<_>>();
    fs::write(dir.join("rust_after_vram.bin"), &rust_vram)?;
    fs::write(dir.join("rust_after_ram.bin"), &post_game.ram)?;

    // Preserve the exact composed state that both Rust renderers saw.  The
    // live post-frame PPU can already contain registers and memory authored for
    // the following frame, so it is not a reliable description of the failed
    // image by itself.
    let (visible_ppu_summary, visible_vram, visible_oam, visible_cgram) =
        if let Some(rendered) = rendered_display {
            let ppu = rendered.presented_ppu();
            let mut visible_game = post_game.clone();
            visible_game.ppu = ppu.clone();
            (
                format_render_ppu_summary(&visible_game),
                ppu.vram
                    .iter()
                    .flat_map(|word| word.to_le_bytes())
                    .collect::<Vec<_>>(),
                ppu.oam
                    .iter()
                    .flat_map(|word| word.to_le_bytes())
                    .collect::<Vec<_>>(),
                ppu.cgram
                    .iter()
                    .flat_map(|word| word.to_le_bytes())
                    .collect::<Vec<_>>(),
            )
        } else {
            let mut visible_game = post_game.clone();
            visible_game.with_display_snapshot(|display| {
                (
                    format_render_ppu_summary(display),
                    display
                        .ppu
                        .vram
                        .iter()
                        .flat_map(|word| word.to_le_bytes())
                        .collect::<Vec<_>>(),
                    display
                        .ppu
                        .oam
                        .iter()
                        .flat_map(|word| word.to_le_bytes())
                        .collect::<Vec<_>>(),
                    display
                        .ppu
                        .cgram
                        .iter()
                        .flat_map(|word| word.to_le_bytes())
                        .collect::<Vec<_>>(),
                )
            })
        };
    fs::write(dir.join("rust_visible_vram.bin"), &visible_vram)?;
    fs::write(dir.join("rust_visible_oam.bin"), &visible_oam)?;
    if let Some(oracle_presented_oam) = oracle_presented_oam {
        fs::write(dir.join("oracle_presented_oam.bin"), oracle_presented_oam)?;
        fs::write(
            dir.join("oam_generations.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "visible_scanout": summarize_value_domain(&visible_oam, oracle_presented_oam),
            }))?,
        )?;
    }
    fs::write(dir.join("rust_visible_cgram.bin"), &visible_cgram)?;
    if let Some(receipt) = display_oracle_receipt {
        fs::write(
            dir.join("display_oracle.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "receipt": receipt,
                "differences": display_oracle_differences(receipt),
            }))?,
        )?;
    }

    let mut visible_game = post_game.clone();
    if let Some(rendered) = rendered_display {
        visible_game.ppu = rendered.presented_ppu().clone();
    }
    let vram_capture = gpu_capture::capture_gpu_frame_from_game(&mut visible_game);
    let vram_gpu_frame = vram_capture.gpu_frame();
    let vram_modern_frame_rgba =
        renderer::modern_extract::render_modern_frame_full_from_vram(&vram_gpu_frame);
    write_rgba_frame_png(
        &dir.join("rust_modern_vram_frame.png"),
        &vram_modern_frame_rgba,
        256,
        224,
    )?;
    let vram_modern_video_diff =
        compare_snes9x_video_frame(&vram_modern_frame_rgba, 256, 224, capture)
            .unwrap_or_else(|| "exact".to_string());
    fs::write(
        dir.join("modern_vram_video_diff.txt"),
        format!("{vram_modern_video_diff}\n"),
    )?;
    if let Some(oracle_ram) = snes9x_state_section(oracle_after_state, b"RAM") {
        fs::write(dir.join("oracle_after_ram.bin"), oracle_ram)?;
    }
    if let Some(oracle_vram) = snes9x_state_section(oracle_after_state, b"VRA") {
        fs::write(dir.join("oracle_after_vram.bin"), oracle_vram)?;
        let live_after = summarize_value_domain(&rust_vram, oracle_vram);
        let visible_scanout = oracle_before_vram
            .map(|oracle_vram| summarize_value_domain(&visible_vram, oracle_vram));
        fs::write(
            dir.join("vram_diff.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "rust_bytes": live_after.rust_values,
                "oracle_bytes": live_after.oracle_values,
                "mismatched_bytes": live_after.mismatched_values,
                "first_mismatch_byte": live_after.first_mismatch,
                "first_mismatch_word": live_after.first_mismatch.map(|offset| offset / 2),
            }))?,
        )?;
        fs::write(
            dir.join("vram_generations.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "live_after_frame": live_after,
                "visible_scanout": visible_scanout,
            }))?,
        )?;
    }

    write_rgba_frame_png(&dir.join("rust_frame.png"), rust_frame_rgba, 256, 224)?;
    let Some(stride) = snes9x_pixel_stride(capture.pixel_format) else {
        return Err(format!("unsupported libretro pixel format {}", capture.pixel_format).into());
    };
    let mut oracle_argb =
        vec![0u8; capture.video_width as usize * capture.video_height as usize * 4];
    for y in 0..capture.video_height as usize {
        for x in 0..capture.video_width as usize {
            let src = y * capture.video_pitch + x * stride;
            let Some([r, g, b, _]) = snes9x_rgba_pixel_at(capture, src) else {
                return Err(format!("failed to decode libretro pixel at {x},{y}").into());
            };
            let dst = (y * capture.video_width as usize + x) * 4;
            oracle_argb[dst] = b;
            oracle_argb[dst + 1] = g;
            oracle_argb[dst + 2] = r;
            oracle_argb[dst + 3] = 0xff;
        }
    }
    write_argb_frame_png(
        &dir.join("oracle_frame.png"),
        &oracle_argb,
        capture.video_width,
        capture.video_height,
    )?;
    write_wav_i16_stereo(&dir.join("rust_audio.wav"), rust_audio, sample_rate, 2)?;
    write_wav_i16_stereo(
        &dir.join("oracle_audio.wav"),
        &capture.audio,
        sample_rate,
        2,
    )?;

    let report = ParityFailureReport {
        kind: format!("libretro-{oracle_name}"),
        frame,
        input: format!("0x{input:04x}"),
        run_what: None,
        message,
        trace_mine: Some(TraceState::from_ram(&post_game.ram, input, RUN_MAIN).to_string()),
        trace_theirs: oracle_system_ram
            .map(|ram| TraceState::from_ram(ram, input, RUN_MAIN).to_string()),
        ppu_mine: Some(visible_ppu_summary),
        ppu_theirs: None,
        audio_mine: Some(summarize_audio_samples(rust_audio)),
        audio_theirs: Some(summarize_audio_samples(&capture.audio)),
        artifacts: vec![
            "input.txt".to_string(),
            "rust_before.z3state".to_string(),
            "rust_after.z3state".to_string(),
            "oracle_before.state".to_string(),
            "oracle_after.state".to_string(),
            "oracle_before_ram.bin".to_string(),
            "rust_before_ram.bin".to_string(),
            "oracle_before_vram.bin".to_string(),
            "rust_after_vram.bin".to_string(),
            "oracle_after_vram.bin".to_string(),
            "rust_after_ram.bin".to_string(),
            "oracle_after_ram.bin".to_string(),
            "rust_visible_vram.bin".to_string(),
            "rust_visible_oam.bin".to_string(),
            "rust_visible_cgram.bin".to_string(),
            "display_oracle.json".to_string(),
            "vram_diff.json".to_string(),
            "vram_generations.json".to_string(),
            "rust_frame.png".to_string(),
            "rust_classic_frame.png".to_string(),
            "classic_video_diff.txt".to_string(),
            "rust_modern_vram_frame.png".to_string(),
            "modern_vram_video_diff.txt".to_string(),
            "oracle_frame.png".to_string(),
            "rust_audio.wav".to_string(),
            "oracle_audio.wav".to_string(),
            "diff.json".to_string(),
        ],
        notes: vec![
            "oracle_before.state is the exact libretro state immediately before the failing frame"
                .to_string(),
            "rust_before_ram.bin is the translated runtime WRAM at that same pre-frame boundary"
                .to_string(),
            "oracle_before_vram.bin is the VRAM generation that produced the failing Snes9x scanout"
                .to_string(),
            "oracle_after.state and rust_after.z3state are the exact post-frame states used for the rendered comparison"
                .to_string(),
            "input.txt contains the complete controller stream from the synchronized start"
                .to_string(),
            "trace_theirs is decoded from the oracle core's exposed post-frame SNES WRAM"
                .to_string(),
            "ppu_mine and rust_visible_*.bin describe the composed display snapshot actually rendered, not the live post-frame state"
                .to_string(),
            "display_oracle.json classifies post-frame register, CGRAM, live OAM, presented OAM, and raster differences automatically"
                .to_string(),
            "vram_generations.json separates live post-frame VRAM from the generation that produced the failed scanout"
                .to_string(),
        ],
    };
    let _ = write_parity_diff(&dir, &report)?;
    Ok(dir)
}

pub(crate) fn snes9x_state_section<'a>(state: &'a [u8], tag: &[u8; 3]) -> Option<&'a [u8]> {
    let start = state
        .windows(4)
        .position(|window| window[..3] == tag[..] && window[3] == b':')?;
    let length_start = start + 4;
    let length_end = state[length_start..]
        .iter()
        .position(|byte| *byte == b':')?
        + length_start;
    let length = std::str::from_utf8(&state[length_start..length_end])
        .ok()?
        .parse::<usize>()
        .ok()?;
    let data_start = length_end + 1;
    state.get(data_start..data_start.checked_add(length)?)
}

fn push_unsigned_varint(bytes: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        bytes.push((value as u8) | 0x80);
        value >>= 7;
    }
    bytes.push(value as u8);
}

fn base64_bytes(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let value = u32::from(chunk[0]) << 16
            | u32::from(chunk.get(1).copied().unwrap_or(0)) << 8
            | u32::from(chunk.get(2).copied().unwrap_or(0));
        encoded.push(ALPHABET[((value >> 18) & 0x3f) as usize] as char);
        encoded.push(ALPHABET[((value >> 12) & 0x3f) as usize] as char);
        encoded.push(if chunk.len() >= 2 {
            ALPHABET[((value >> 6) & 0x3f) as usize] as char
        } else {
            '='
        });
        encoded.push(if chunk.len() == 3 {
            ALPHABET[(value & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    encoded
}

fn compact_delta_integer_sequence<const N: usize>(
    fields: [&'static str; N],
    rows: impl IntoIterator<Item = [i64; N]>,
) -> SmpBootstrapDeltaSequence {
    compact_delta_integer_sequence_with_zstd(fields, rows, false)
}

pub(crate) fn compact_delta_integer_sequence_with_zstd<const N: usize>(
    fields: [&'static str; N],
    rows: impl IntoIterator<Item = [i64; N]>,
    use_zstd: bool,
) -> SmpBootstrapDeltaSequence {
    let rows = rows.into_iter().collect::<Vec<_>>();
    let mut encoded = Vec::new();
    let mut expanded = Vec::new();
    for row in &rows {
        for value in row {
            expanded.extend_from_slice(&value.to_le_bytes());
        }
    }
    for field in 0..N {
        let mut column = Vec::new();
        let mut previous = 0i64;
        let mut row = 0;
        while row < rows.len() {
            let delta = rows[row][field] - previous;
            previous = rows[row][field];
            if delta != 0 {
                let zigzag = ((delta << 1) ^ (delta >> 63)) as u64;
                push_unsigned_varint(&mut column, zigzag + 1);
                row += 1;
                continue;
            }
            let mut run_length = 1usize;
            while row + run_length < rows.len() && rows[row + run_length][field] == previous {
                run_length += 1;
            }
            push_unsigned_varint(&mut column, 0);
            push_unsigned_varint(&mut column, run_length as u64);
            row += run_length;
        }
        push_unsigned_varint(&mut encoded, column.len() as u64);
        encoded.extend_from_slice(&column);
    }
    let encoded = if use_zstd {
        zstd::stream::encode_all(encoded.as_slice(), 19)
            .expect("in-memory CPU timing fixture compression failed")
    } else {
        encoded
    };
    SmpBootstrapDeltaSequence {
        encoding: if use_zstd {
            "columnar-signed-delta-zero-rle-varint-zstd-base64-v1"
        } else {
            "columnar-signed-delta-zero-rle-varint-base64-v1"
        },
        fields: fields.into_iter().collect(),
        record_count: rows.len(),
        expanded_sha256: parity::evidence::sha256_bytes(&expanded),
        data_base64: base64_bytes(&encoded),
    }
}

fn compact_cpu_apu_accesses(accesses: &[FramedApuPortAccess]) -> SmpBootstrapDeltaSequence {
    compact_delta_integer_sequence(
        [
            "frame",
            "port",
            "value",
            "output_sample",
            "v_counter",
            "cpu_cycle",
            "program_counter",
            "apu_cycle_before",
            "apu_cycle_after",
            "smp_clock_before",
            "smp_clock_after",
            "smp_pc_before",
            "smp_pc_after",
            "smp_opcode_before",
            "smp_opcode_after",
            "smp_opcode_cycle_before",
            "smp_opcode_cycle_after",
            "is_read",
            "cpu_model_5a22",
            "wram_refresh_position",
            "cpu_model_identity",
        ],
        accesses.iter().map(|framed| {
            let access = &framed.access;
            [
                i64::from(framed.frame),
                i64::from(access.port),
                i64::from(access.value),
                i64::from(access.output_sample),
                i64::from(access.v_counter),
                i64::from(access.cpu_cycle),
                i64::from(access.program_counter),
                i64::from(access.apu_cycle_before),
                i64::from(access.apu_cycle_after),
                i64::from(access.smp_clock_before),
                i64::from(access.smp_clock_after),
                i64::from(access.smp_pc_before),
                i64::from(access.smp_pc_after),
                i64::from(access.smp_opcode_before),
                i64::from(access.smp_opcode_after),
                i64::from(access.smp_opcode_cycle_before),
                i64::from(access.smp_opcode_cycle_after),
                i64::from(access.is_read),
                i64::from(access.cpu_model_5a22),
                i64::from(access.wram_refresh_position),
                i64::from(access.cpu_model_identity),
            ]
        }),
    )
}

fn compact_ordinal_cpu_apu_accesses(
    accesses: &[OrdinalApuPortAccess],
) -> SmpBootstrapDeltaSequence {
    compact_delta_integer_sequence_with_zstd(
        [
            "cpu_transaction_ordinal",
            "port",
            "value",
            "output_sample",
            "v_counter",
            "cpu_cycle",
            "program_counter",
            "apu_cycle_before",
            "apu_cycle_after",
            "smp_clock_before",
            "smp_clock_after",
            "dsp_clock_before",
            "dsp_clock_after",
            "dsp_phase_before",
            "dsp_phase_after",
            "smp_pc_before",
            "smp_pc_after",
            "smp_opcode_before",
            "smp_opcode_after",
            "smp_opcode_cycle_before",
            "smp_opcode_cycle_after",
            "is_read",
            "cpu_model_5a22",
            "wram_refresh_position",
            "cpu_model_identity",
        ],
        accesses.iter().map(|ordinal| {
            let access = &ordinal.access;
            [
                ordinal.cpu_transaction_ordinal as i64,
                i64::from(access.port),
                i64::from(access.value),
                i64::from(access.output_sample),
                i64::from(access.v_counter),
                i64::from(access.cpu_cycle),
                i64::from(access.program_counter),
                i64::from(access.apu_cycle_before),
                i64::from(access.apu_cycle_after),
                i64::from(access.smp_clock_before),
                i64::from(access.smp_clock_after),
                i64::from(access.dsp_clock_before),
                i64::from(access.dsp_clock_after),
                i64::from(access.dsp_phase_before),
                i64::from(access.dsp_phase_after),
                i64::from(access.smp_pc_before),
                i64::from(access.smp_pc_after),
                i64::from(access.smp_opcode_before),
                i64::from(access.smp_opcode_after),
                i64::from(access.smp_opcode_cycle_before),
                i64::from(access.smp_opcode_cycle_after),
                i64::from(access.is_read),
                i64::from(access.cpu_model_5a22),
                i64::from(access.wram_refresh_position),
                i64::from(access.cpu_model_identity),
            ]
        }),
        true,
    )
}

fn compact_ordinal_smp_output_port_writes(
    writes: &[OrdinalSmpOutputPortWrite],
) -> SmpBootstrapDeltaSequence {
    compact_delta_integer_sequence_with_zstd(
        [
            "cpu_transaction_ordinal",
            "absolute_cycle",
            "port",
            "value",
            "origin_pc",
            "opcode",
            "opcode_cycle",
            "v_counter",
            "cpu_cycle",
            "cpu_program_counter",
            "cpu_reference_time",
            "cpu_remainder",
            "smp_clock",
            "next_pc",
            "dsp_clock",
            "dsp_phase",
            "output_sample",
        ],
        writes.iter().map(|ordinal| {
            let write = ordinal.write;
            [
                ordinal.cpu_transaction_ordinal as i64,
                write.absolute_cycle as i64,
                i64::from(write.port),
                i64::from(write.value),
                i64::from(write.origin_pc),
                i64::from(write.opcode),
                i64::from(write.opcode_cycle),
                i64::from(write.v_counter),
                i64::from(write.cpu_cycle),
                i64::from(write.cpu_program_counter),
                i64::from(write.cpu_reference_time),
                i64::from(write.cpu_remainder),
                i64::from(write.smp_clock),
                i64::from(write.next_pc),
                i64::from(write.dsp_clock),
                i64::from(write.dsp_phase),
                i64::from(write.output_sample),
            ]
        }),
        true,
    )
}

fn compact_cpu_timing_transactions(
    transactions: &[FramedCpuTimingTransaction],
) -> SmpBootstrapDeltaSequence {
    compact_delta_integer_sequence_with_zstd(
        [
            "frame",
            "kind",
            "duration",
            "origin_pc",
            "opcode",
            "start_v_counter",
            "start_cpu_cycle",
            "end_v_counter",
            "end_cpu_cycle",
            "cpu_model_identity",
            "cpu_model_5a22",
            "start_wram_refresh_position",
            "end_wram_refresh_position",
        ],
        transactions.iter().map(|framed| {
            let transaction = &framed.transaction;
            [
                i64::from(framed.frame),
                i64::from(transaction.kind),
                i64::from(transaction.duration),
                i64::from(transaction.origin_pc),
                i64::from(transaction.opcode),
                i64::from(transaction.start_v_counter),
                i64::from(transaction.start_cpu_cycle),
                i64::from(transaction.end_v_counter),
                i64::from(transaction.end_cpu_cycle),
                i64::from(transaction.cpu_model_identity),
                i64::from(transaction.cpu_model_5a22),
                i64::from(transaction.start_wram_refresh_position),
                i64::from(transaction.end_wram_refresh_position),
            ]
        }),
        true,
    )
}

fn first_nmi_apui_anchor_indices(
    accesses: &[FramedApuPortAccess],
    transactions: &[FramedCpuTimingTransaction],
) -> Result<Option<(usize, usize)>, String> {
    let Some(access_index) = accesses.iter().position(|framed| {
        let access = &framed.access;
        access.is_read && access.port == 0 && (access.program_counter & 0x00ff_ffff) == 0x0080e4
    }) else {
        return Ok(None);
    };
    let access = &accesses[access_index];
    let transaction_index = transactions
        .iter()
        .position(|framed| {
            let transaction = &framed.transaction;
            framed.frame == access.frame
                && transaction.kind == 2
                && (transaction.origin_pc & 0x00ff_ffff) == 0x0080e1
                && transaction.opcode == 0xad
                && transaction.start_v_counter == access.access.v_counter
                && transaction.start_cpu_cycle == access.access.cpu_cycle
        })
        .ok_or_else(|| {
            "first $8080e1 APU read has no matching completed kind-2 timing transaction".to_string()
        })?;
    Ok(Some((access_index, transaction_index)))
}

fn first_nmi_dma_setup_stop_index(
    transactions: &[FramedCpuTimingTransaction],
) -> Result<Option<usize>, String> {
    let Some(index) = transactions
        .iter()
        .position(|framed| (framed.transaction.origin_pc & 0x00ff_ffff) == 0x008a35)
    else {
        return Ok(None);
    };
    let transaction = transactions[index].transaction;
    if transaction.kind != 0 || transaction.opcode != 0x8d {
        return Err(format!(
            "first $008a35 transaction is not the expected STA raw fetch: {:?}",
            transactions[index]
        ));
    }
    Ok(Some(index))
}

fn first_nmi_dma_transaction_slice(
    transactions: &[crate::libretro_core::LibretroCpuTimingTransaction],
) -> Result<Option<(usize, usize, usize)>, String> {
    let Some(fetch_index) = transactions.iter().position(|transaction| {
        transaction.kind == 0
            && (transaction.origin_pc & 0x00ff_ffff) == 0x008a35
            && transaction.opcode == 0x8d
    }) else {
        return Ok(None);
    };
    let fetch = transactions[fetch_index];
    if fetch.start_v_counter != 226
        || fetch.start_cpu_cycle != 714
        || fetch.end_v_counter != 226
        || fetch.end_cpu_cycle != 722
    {
        return Err(format!(
            "$008a35 raw fetch does not continue the pinned V226:H714->H722 anchor: {fetch:?}"
        ));
    }
    let completion_index = transactions[fetch_index..]
        .iter()
        .position(|transaction| {
            transaction.kind == 2
                && (transaction.origin_pc & 0x00ff_ffff) == 0x008a35
                && transaction.opcode == 0x8d
        })
        .map(|relative| fetch_index + relative)
        .ok_or("$008a35 STA $420b has no completed kind-2 semantic transaction")?;
    let successor_index = transactions[completion_index + 1..]
        .iter()
        .position(|transaction| {
            transaction.kind == 0 && (transaction.origin_pc & 0x00ff_ffff) == 0x008a38
        })
        .map(|relative| completion_index + 1 + relative)
        .ok_or("completed $008a35 STA $420b has no following $008a38 raw fetch")?;
    Ok(Some((fetch_index, completion_index, successor_index)))
}

fn first_nmi_return_start_index(
    transactions: &[crate::libretro_core::LibretroCpuTimingTransaction],
) -> Result<Option<usize>, String> {
    let candidates = transactions
        .iter()
        .enumerate()
        .filter(|(_, transaction)| {
            transaction.kind == 0
                && (transaction.origin_pc & 0x00ff_ffff) == FIRST_NMI_RETURN_START_PC
        })
        .collect::<Vec<_>>();
    let Some((index, transaction)) = candidates.first().copied() else {
        return Ok(None);
    };
    if candidates.len() != 1 {
        return Err(format!(
            "expected one $008a38 successor fetch, observed {}",
            candidates.len()
        ));
    }
    if transaction.opcode != 0x8c
        || transaction.start_v_counter != 227
        || transaction.start_cpu_cycle != 742
        || transaction.end_v_counter != 227
        || transaction.end_cpu_cycle != 750
        || transaction.cpu_model_5a22 != 2
        || transaction.start_wram_refresh_position != 534
        || transaction.end_wram_refresh_position != 534
    {
        return Err(format!(
            "$008a38 successor fetch does not match the committed V227:H742->H750 M2 receipt: {transaction:?}"
        ));
    }
    Ok(Some(index))
}

fn cpu_transaction_ordinal_at_start(
    transactions: &[crate::libretro_core::LibretroCpuTimingTransaction],
    v_counter: i32,
    cpu_cycle: i32,
    cpu_program_counter: i32,
) -> Result<Option<usize>, String> {
    let candidates = transactions
        .iter()
        .enumerate()
        .filter(|(_, transaction)| {
            transaction.kind == 2
                && transaction.start_v_counter == v_counter
                && transaction.start_cpu_cycle == cpu_cycle
                && transaction.origin_pc >> 16 == cpu_program_counter >> 16
                && (0..=4)
                    .contains(&((cpu_program_counter & 0xffff) - (transaction.origin_pc & 0xffff)))
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [] => Ok(None),
        [index] => Ok(Some(*index)),
        _ => Err(format!(
            "APUI receipt at V{v_counter}:H{cpu_cycle} PC ${cpu_program_counter:06x} matches multiple CPU transactions: {candidates:?}"
        )),
    }
}

fn cpu_transaction_ordinal_containing(
    transactions: &[crate::libretro_core::LibretroCpuTimingTransaction],
    v_counter: i32,
    cpu_cycle: i32,
    cpu_program_counter: i32,
) -> Result<Option<usize>, String> {
    if transactions.is_empty() {
        return Err("cannot join a receipt to an empty CPU transaction sequence".into());
    }
    let candidates = transactions
        .iter()
        .enumerate()
        .filter(|(_, transaction)| {
            let pc_delta = (cpu_program_counter & 0xffff) - (transaction.origin_pc & 0xffff);
            let contains_raster = if transaction.start_v_counter == transaction.end_v_counter {
                v_counter == transaction.start_v_counter
                    && transaction.start_cpu_cycle <= cpu_cycle
                    && cpu_cycle < transaction.end_cpu_cycle
            } else {
                (v_counter == transaction.start_v_counter
                    && transaction.start_cpu_cycle <= cpu_cycle)
                    || (v_counter == transaction.end_v_counter
                        && cpu_cycle < transaction.end_cpu_cycle)
            };
            contains_raster
                && transaction.origin_pc >> 16 == cpu_program_counter >> 16
                && (0..=4).contains(&pc_delta)
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [] => Ok(None),
        [index] => Ok(Some(*index)),
        _ => Err(format!(
            "SMP output receipt at V{v_counter}:H{cpu_cycle} PC ${cpu_program_counter:06x} matches multiple CPU transactions: {candidates:?}"
        )),
    }
}

fn read_snes9x_retro_run_trace(
    path: &Path,
    expected_run: u32,
) -> Result<Option<Snes9xRetroRunTrace>, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to read Snes9x core trace {}: {error}",
            path.display()
        )
    })?;
    let mut entry = None;
    let mut return_event = None;
    let mut hdma_events = Vec::new();
    let mut video_events = Vec::new();
    let mut saw_run = false;
    for (line_index, line) in bytes.split(|byte| *byte == b'\n').enumerate() {
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let event: serde_json::Value = serde_json::from_slice(line).map_err(|error| {
            format!(
                "invalid Snes9x core trace line {} in {}: {error}",
                line_index + 1,
                path.display()
            )
        })?;
        if event["run"].as_u64() != Some(u64::from(expected_run)) {
            continue;
        }
        saw_run = true;
        match (event["event"].as_str(), event["stage"].as_str()) {
            (Some("frame"), Some("entry")) => {
                if entry.replace(event).is_some() {
                    return Err(format!("run {expected_run} has duplicate frame-entry rows"));
                }
            }
            (Some("frame"), Some("return")) => {
                if return_event.replace(event).is_some() {
                    return Err(format!(
                        "run {expected_run} has duplicate frame-return rows"
                    ));
                }
            }
            (Some("hdma"), Some("start" | "end")) => hdma_events.push(event),
            // The maintained trace core always publishes this direct
            // retro_run scanout milestone; it is not controlled by the
            // optional event-domain mask. Retain it rather than pretending
            // `frame,hdma` suppresses a source-owned receipt.
            (Some("video"), Some("presented")) => video_events.push(event),
            (event, stage) => {
                return Err(format!(
                    "run {expected_run} has unexpected trace domain/stage {event:?}/{stage:?}"
                ));
            }
        }
    }
    if !saw_run {
        return Ok(None);
    }
    let entry = entry.ok_or_else(|| format!("run {expected_run} has no frame-entry row"))?;
    let return_event =
        return_event.ok_or_else(|| format!("run {expected_run} has no frame-return row"))?;
    if video_events.len() != 1 {
        return Err(format!(
            "run {expected_run} has {} video/presented rows, expected exactly one",
            video_events.len()
        ));
    }
    for (pair_index, pair) in hdma_events.chunks(2).enumerate() {
        if pair.len() != 2 || pair[0]["stage"] != "start" || pair[1]["stage"] != "end" {
            return Err(format!(
                "run {expected_run} HDMA pair {pair_index} is not an ordered start/end bracket"
            ));
        }
    }
    Ok(Some(Snes9xRetroRunTrace {
        entry,
        return_event,
        hdma_events,
        video_events,
        raw_sha256: parity::evidence::sha256_bytes(&bytes),
    }))
}

fn validate_first_nmi_return_cpu_slice(
    trace: &Snes9xRetroRunTrace,
    transactions: &[crate::libretro_core::LibretroCpuTimingTransaction],
) -> Result<Vec<CpuTimingGapReceipt>, String> {
    if trace.entry["v"].as_i64() != Some(225)
        || trace.entry["cycles"].as_i64() != Some(6)
        || trace.entry["pc"].as_i64() != Some(0x008036)
    {
        return Err(format!(
            "run-81 continuation has the wrong direct entry receipt, expected V225:H6 PC $008036: {}",
            trace.entry
        ));
    }
    if trace.return_event["v"].as_i64() != Some(225)
        || trace.return_event["cycles"].as_i64() != Some(94)
        || trace.return_event["pc"].as_i64() != Some(0x0080c9)
    {
        return Err(format!(
            "run-81 continuation has the wrong direct return receipt, expected V225:H94 PC $0080c9: {}",
            trace.return_event
        ));
    }
    let first = transactions
        .first()
        .ok_or("run-81 continuation CPU slice is empty")?;
    let terminal = transactions.last().unwrap();
    if first.start_v_counter != 227 || first.start_cpu_cycle != 742 {
        return Err(format!(
            "run-81 continuation CPU slice does not start at V227:H742: {first:?}"
        ));
    }
    if terminal.end_v_counter != 225 || terminal.end_cpu_cycle != 94 {
        return Err(format!(
            "run-81 continuation CPU slice does not end at the direct V225:H94 return: {terminal:?}"
        ));
    }
    let mut rollovers = 0;
    let mut gaps = Vec::new();
    for (index, transaction) in transactions.iter().enumerate() {
        if transaction.start_v_counter == 261 && transaction.end_v_counter == 0 {
            rollovers += 1;
        } else if transaction.end_v_counter < transaction.start_v_counter {
            return Err(format!(
                "CPU transaction {index} has an unexpected raster reversal: {transaction:?}"
            ));
        }
        if let Some(next) = transactions.get(index + 1) {
            let mut end_position =
                i64::from(transaction.end_v_counter) * 1364 + i64::from(transaction.end_cpu_cycle);
            let mut next_position =
                i64::from(next.start_v_counter) * 1364 + i64::from(next.start_cpu_cycle);
            if next.start_v_counter < transaction.end_v_counter {
                if transaction.end_v_counter != 261 || next.start_v_counter != 0 {
                    return Err(format!(
                        "CPU transaction {index} has an unexpected raster reversal before its successor: current={transaction:?}; next={next:?}"
                    ));
                }
                rollovers += 1;
                next_position += 262 * 1364;
            }
            if transaction.end_v_counter == 0 && transaction.start_v_counter == 261 {
                end_position += 262 * 1364;
                next_position += 262 * 1364;
            }
            if next_position < end_position {
                return Err(format!(
                    "CPU transaction {index} overlaps or reorders its successor: current={transaction:?}; next={next:?}"
                ));
            }
            if next_position > end_position {
                gaps.push(CpuTimingGapReceipt {
                    previous_transaction_ordinal: index,
                    next_transaction_ordinal: index + 1,
                    previous_end_v_counter: transaction.end_v_counter,
                    previous_end_cpu_cycle: transaction.end_cpu_cycle,
                    next_start_v_counter: next.start_v_counter,
                    next_start_cpu_cycle: next.start_cpu_cycle,
                    elapsed_master_cycles: next_position - end_position,
                });
            }
        }
    }
    if rollovers != 1 {
        return Err(format!(
            "run-81 continuation CPU slice has {rollovers} V261->V0 rollovers, expected exactly one"
        ));
    }
    Ok(gaps)
}

fn trailing_dma_events_after_first_nmi(
    events: &[crate::libretro_core::LibretroDmaLedgerEvent],
) -> Result<Vec<crate::libretro_core::LibretroDmaLedgerEvent>, String> {
    let Some((outer, selected)) = first_nmi_dma_ledger_slice(events)? else {
        return Err(
            "same-retro_run continuation is missing the committed first-NMI DMA prefix".into(),
        );
    };
    let last = events
        .iter()
        .rposition(|event| event.fields[1] == outer)
        .ok_or("selected first-NMI DMA outer vanished from the complete ledger")?;
    if events[..=last]
        .iter()
        .filter(|event| event.fields[1] == outer)
        .count()
        != selected.len()
    {
        return Err("first-NMI DMA outer is interleaved with later ledger ownership".into());
    }
    let trailing = events[last + 1..].to_vec();
    validate_complete_dma_outers(&trailing)?;
    Ok(trailing)
}

fn validate_complete_dma_outers(
    events: &[crate::libretro_core::LibretroDmaLedgerEvent],
) -> Result<(), String> {
    let mut index = 0;
    while index < events.len() {
        let begin = &events[index];
        if begin.fields[0] != 0 || begin.fields[9] != 1 {
            return Err(format!(
                "DMA continuation does not begin with a completed outer-begin row: {begin:?}"
            ));
        }
        let outer = begin.fields[1];
        let end = events[index..]
            .iter()
            .position(|event| event.fields[1] == outer && event.fields[0] == 4)
            .map(|relative| index + relative)
            .ok_or_else(|| format!("DMA outer {outer} has no outer-end row"))?;
        if events[index..=end]
            .iter()
            .any(|event| event.fields[1] != outer || event.fields[9] != 1)
        {
            return Err(format!(
                "DMA outer {outer} has mixed ownership or an incomplete row"
            ));
        }
        let mut cursor = index + 1;
        let mut global_byte_ordinal = 0;
        while cursor < end {
            let channel_begin = &events[cursor];
            if channel_begin.fields[0] != 1 {
                return Err(format!(
                    "DMA outer {outer} expected a channel-begin row at event {cursor}: {channel_begin:?}"
                ));
            }
            let channel = channel_begin.fields[2];
            cursor += 1;
            let mut channel_byte_ordinal = 0;
            while cursor < end && events[cursor].fields[0] == 2 {
                let byte = &events[cursor];
                if byte.fields[2] != channel
                    || byte.fields[3] != global_byte_ordinal
                    || byte.fields[4] != channel_byte_ordinal
                {
                    return Err(format!(
                        "DMA outer {outer} has a non-contiguous byte receipt at event {cursor}: {byte:?}"
                    ));
                }
                global_byte_ordinal += 1;
                channel_byte_ordinal += 1;
                cursor += 1;
            }
            let channel_end = events.get(cursor).ok_or_else(|| {
                format!("DMA outer {outer} ended before channel {channel} completion")
            })?;
            if channel_end.fields[0] != 3 || channel_end.fields[2] != channel {
                return Err(format!(
                    "DMA outer {outer} has no matching channel-end row for channel {channel}: {channel_end:?}"
                ));
            }
            cursor += 1;
        }
        index = end + 1;
    }
    Ok(())
}

/// Convert emulator-private DMA chronology into the replaceable semantic
/// contract consumed by translated Zelda. A stock core without the optional
/// generic ledger can still provide host cadence; it simply owns no migrated
/// DMA domain yet.
fn snes9x_oracle_semantic_receipts(
    oracle: &LibretroCore,
) -> Result<Vec<OriginalTimingSemanticReceipt>, String> {
    // The detailed DMA chronology is an opt-in fixture capability. Continuous
    // presentation authority uses bounded scanout-domain receipts below and
    // never retains the emulator's per-byte DMA history.
    if env::var_os("ZELDA3_DEBUG_SNES9X_DMA_LEDGER").is_none() {
        return Ok(Vec::new());
    }
    let Some(events) = oracle.debug_dma_ledger()? else {
        return Ok(Vec::new());
    };
    semantic_receipts_from_dma_ledger(&events)
}

fn snes9x_presented_animated_bg_tiles(
    oracle: &LibretroCore,
) -> Result<Option<PresentedAnimatedBgTiles>, String> {
    // Zelda's leading NMI publishes this complete 0x400-byte domain before the
    // active field, and Snes9x returns at the following VBlank before another
    // NMI can replace it. Decode the exact post-host VRAM generation instead
    // of consulting TileCached: a tile consumed only through H-flip has no
    // entry in the unflipped cache even though it was visibly presented.
    let Some(destination) =
        decode_presented_animated_bg_destination(oracle.debug_ppu_value(40, 0))?
    else {
        return Ok(None);
    };
    let vram = oracle
        .memory_bytes(RETRO_MEMORY_VIDEO_RAM)
        .ok_or_else(|| "Snes9x video RAM is unavailable".to_string())?;
    decode_presented_animated_bg_tiles(destination, vram).map(Some)
}

fn decode_presented_animated_bg_destination(
    value: Option<i32>,
) -> Result<Option<zelda3::PresentedAnimatedBgDestination>, String> {
    match value {
        None | Some(-1) => return Ok(None),
        Some(0x3b00) => Ok(Some(zelda3::PresentedAnimatedBgDestination::Dungeon)),
        Some(0x3c00) => Ok(Some(zelda3::PresentedAnimatedBgDestination::Overworld)),
        // Zelda leaves this operand at Snes9x's 0x55 reset fill until graphics
        // setup. Cold initialization then clears WRAM to zero before either
        // DecompressAnimated* routine publishes the first real destination.
        // Both values therefore mean that no animated-BG generation exists.
        Some(0 | 0x5555) => Ok(None),
        Some(value) => Err(format!(
            "presented animated-BG destination is invalid: {value}"
        )),
    }
}

fn decode_presented_animated_bg_tiles(
    destination: zelda3::PresentedAnimatedBgDestination,
    vram: &[u8],
) -> Result<PresentedAnimatedBgTiles, String> {
    let word_address = match destination {
        zelda3::PresentedAnimatedBgDestination::Dungeon => 0x3b00,
        zelda3::PresentedAnimatedBgDestination::Overworld => 0x3c00,
    };
    let byte_address = word_address * 2;
    let byte_count = PresentedAnimatedBgTiles::TILE_COUNT * 32;
    let bytes = vram
        .get(byte_address..byte_address + byte_count)
        .ok_or_else(|| "Snes9x video RAM omits the animated-BG publication".to_string())?;
    let mut pixels = vec![0; PresentedAnimatedBgTiles::TILE_COUNT * 64];
    for tile in 0..PresentedAnimatedBgTiles::TILE_COUNT {
        let packed = &bytes[tile * 32..tile * 32 + 32];
        for y in 0..8 {
            let planes = [
                packed[y * 2],
                packed[y * 2 + 1],
                packed[16 + y * 2],
                packed[16 + y * 2 + 1],
            ];
            for x in 0..8 {
                let mask = 1 << (7 - x);
                pixels[tile * 64 + y * 8 + x] =
                    planes.iter().enumerate().fold(0, |pixel, (plane, value)| {
                        pixel | u8::from(value & mask != 0) << plane
                    });
            }
        }
    }
    PresentedAnimatedBgTiles::new(destination, pixels)
        .ok_or_else(|| "presented animated-BG receipt has an invalid shape".to_string())
}

fn snes9x_presented_hud_tilemap(
    oracle: &LibretroCore,
) -> Result<Option<PresentedHudTilemap>, String> {
    let first = match oracle.debug_ppu_value(37, 0) {
        None | Some(-1) => return Ok(None),
        Some(value) => value,
    };
    let words = (0..PresentedHudTilemap::WORD_COUNT)
        .map(|index| {
            let value = if index == 0 {
                first
            } else {
                oracle
                    .debug_ppu_value(37, index as i32)
                    .ok_or_else(|| format!("presented HUD tilemap word {index} is unavailable"))?
            };
            u16::try_from(value)
                .map_err(|_| format!("presented HUD tilemap word {index} is invalid: {value}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    PresentedHudTilemap::new(words)
        .map(Some)
        .ok_or_else(|| "presented HUD tilemap receipt has an invalid shape".to_string())
}

fn snes9x_presented_scanout_geometry(
    oracle: &LibretroCore,
) -> Result<Option<zelda3::PresentedScanoutGeometry>, String> {
    let top_crop = match oracle.debug_ppu_value(39, 0) {
        None | Some(-1) => return Ok(None),
        Some(value) => u8::try_from(value)
            .ok()
            .filter(|&rows| rows <= zelda3::PresentedScanoutGeometry::MAX_TOP_CROP)
            .ok_or_else(|| format!("presented scanout top crop is invalid: {value}"))?,
    };
    zelda3::PresentedScanoutGeometry::new(top_crop)
        .map(Some)
        .ok_or_else(|| format!("presented scanout top crop is invalid: {top_crop}"))
}

#[derive(Clone, Debug)]
struct PresentedOracleVideo {
    bytes: Vec<u8>,
    width: u32,
    height: u32,
    pitch: usize,
    pixel_format: u32,
}

impl From<&LibretroFrame> for PresentedOracleVideo {
    fn from(frame: &LibretroFrame) -> Self {
        Self {
            bytes: frame.video.clone(),
            width: frame.video_width,
            height: frame.video_height,
            pitch: frame.video_pitch,
            pixel_format: frame.pixel_format,
        }
    }
}

fn presented_video_rows_match_prior_surface(
    current: &LibretroFrame,
    previous: Option<&PresentedOracleVideo>,
    start_row: usize,
    end_row: usize,
) -> Result<bool, String> {
    if start_row >= end_row {
        return Ok(false);
    }
    let Some(previous) = previous else {
        return Ok(false);
    };
    if current.video_width != previous.width
        || current.video_height != previous.height
        || current.video_pitch != previous.pitch
        || current.pixel_format != previous.pixel_format
    {
        return Err("consecutive Snes9x host surfaces changed layout".to_string());
    }
    let stride = snes9x_pixel_stride(current.pixel_format)
        .ok_or_else(|| format!("unsupported Snes9x pixel format {}", current.pixel_format))?;
    let row_bytes = usize::try_from(current.video_width)
        .ok()
        .and_then(|width| width.checked_mul(stride))
        .ok_or_else(|| "Snes9x host row byte count overflowed".to_string())?;
    if row_bytes > current.video_pitch || end_row > current.video_height as usize {
        return Err("Snes9x host surface cannot contain the presented row interval".to_string());
    }
    for row in start_row..end_row {
        let start = row
            .checked_mul(current.video_pitch)
            .ok_or_else(|| "Snes9x host row offset overflowed".to_string())?;
        let end = start
            .checked_add(row_bytes)
            .ok_or_else(|| "Snes9x host row end overflowed".to_string())?;
        let current_row = current
            .video
            .get(start..end)
            .ok_or_else(|| format!("current Snes9x host row {row} is truncated"))?;
        let previous_row = previous
            .bytes
            .get(start..end)
            .ok_or_else(|| format!("previous Snes9x host row {row} is truncated"))?;
        if current_row != previous_row {
            return Ok(false);
        }
    }
    Ok(true)
}

fn snes9x_presented_inidisp(
    oracle: &LibretroCore,
    geometry: Option<zelda3::PresentedScanoutGeometry>,
    current_video: &LibretroFrame,
    previous_video: Option<&PresentedOracleVideo>,
) -> Result<Option<zelda3::PresentedInidisp>, String> {
    let top_crop = usize::from(geometry.map_or(0, |geometry| geometry.top_crop()));
    let first = match oracle.debug_ppu_value(38, top_crop as i32) {
        None | Some(-1) => return Ok(None),
        Some(value) => value,
    };
    if !(0..=0xff).contains(&first) {
        return Err(format!("presented INIDISP line 0 is invalid: {first}"));
    }
    let mut lines = Vec::with_capacity(zelda3::PresentedInidisp::VISIBLE_LINES);
    lines.push(((first & 0x0f) as u8, first & 0x80 != 0));
    for line in 1..zelda3::PresentedInidisp::VISIBLE_LINES {
        let source_line = line + top_crop;
        let value = oracle
            .debug_ppu_value(38, source_line as i32)
            .ok_or_else(|| format!("presented INIDISP source line {source_line} is unavailable"))?;
        if !(0..=0xff).contains(&value) {
            return Err(format!("presented INIDISP line {line} is invalid: {value}"));
        }
        lines.push(((value & 0x0f) as u8, value & 0x80 != 0));
    }

    let prefix = lines.iter().take_while(|line| line.1).count();
    let suffix = lines
        .iter()
        .enumerate()
        .skip(prefix)
        .find_map(|(line, value)| value.1.then_some(line));
    let visible_end = suffix.unwrap_or(lines.len());
    if lines[prefix..visible_end].iter().any(|line| line.1)
        || lines[visible_end..].iter().any(|line| !line.1)
    {
        return Err("presented INIDISP has a non-contiguous forced-blank raster".to_string());
    }
    let brightness = lines
        .get(prefix)
        .filter(|_| prefix < visible_end)
        .map(|line| line.0)
        .unwrap_or(lines[0].0);
    if lines[prefix..visible_end]
        .iter()
        .any(|line| line.0 != brightness)
    {
        return Err("presented INIDISP has per-line brightness changes".to_string());
    }
    // Pinned Snes9x's `S9xUpdateScreen` skips drawing entirely while
    // `PPU.ForcedBlanking` is set. A fully blank completed scanout therefore
    // returns the preceding libretro surface rather than a newly cleared
    // black surface. Partial suffix blanking retains only the already-scanned
    // visible interval for the same source-level reason.
    let retain_prior_surface = if prefix == zelda3::PresentedInidisp::VISIBLE_LINES {
        presented_video_rows_match_prior_surface(
            current_video,
            previous_video,
            0,
            zelda3::PresentedInidisp::VISIBLE_LINES,
        )?
    } else if suffix.is_some() {
        presented_video_rows_match_prior_surface(
            current_video,
            previous_video,
            prefix,
            visible_end,
        )?
    } else {
        false
    };
    let prefix = u8::try_from(prefix)
        .map_err(|_| "presented INIDISP blank prefix is invalid".to_string())?;
    let suffix = suffix
        .map(u8::try_from)
        .transpose()
        .map_err(|_| "presented INIDISP blank suffix is invalid".to_string())?;
    let receipt = zelda3::PresentedInidisp::new(brightness, prefix, suffix)
        .ok_or_else(|| "presented INIDISP receipt has an invalid shape".to_string())?;
    let receipt = receipt.with_retained_prior_surface(retain_prior_surface);
    Ok(Some(receipt))
}

/// Decode the address-bearing OBJ cache exposed by the pinned oracle.
///
/// Slots 0..256 name the first OBSEL page and slots 256..512 name the second.
/// The address probe remains authoritative rather than being reconstructed by
/// this adapter, while the page metadata proves that every slot belongs to the
/// expected hardware page. `TileCached` uses 0 for invalid, 1 for decoded, and
/// `BLANK_TILE` (2) for a decoded all-zero tile.
fn decode_snes9x_presented_obj_tiles(
    mut read: impl FnMut(i32, i32) -> Option<i32>,
) -> Result<Option<PresentedObjTiles>, String> {
    let Some(abi) = read(PRESENTED_OBJ_CACHE_META_FIELD, 0) else {
        return Ok(None);
    };
    if abi != PRESENTED_OBJ_CACHE_ABI {
        return Err(format!(
            "unsupported presented OBJ cache ABI {abi}; expected {PRESENTED_OBJ_CACHE_ABI}"
        ));
    }

    let metadata = |read: &mut dyn FnMut(i32, i32) -> Option<i32>, index, name| {
        read(PRESENTED_OBJ_CACHE_META_FIELD, index)
            .ok_or_else(|| format!("presented OBJ cache metadata {name} is unavailable"))
    };
    let slot_count = metadata(&mut read, 1, "slot count")?;
    if slot_count != PRESENTED_OBJ_CACHE_SLOT_COUNT as i32 {
        return Err(format!(
            "presented OBJ cache slot count is invalid: {slot_count}"
        ));
    }
    let pixels_per_tile = metadata(&mut read, 2, "pixels per tile")?;
    if pixels_per_tile != PresentedObjTiles::PIXELS_PER_TILE as i32 {
        return Err(format!(
            "presented OBJ cache pixels per tile is invalid: {pixels_per_tile}"
        ));
    }
    let page_bases = [
        metadata(&mut read, 3, "page 0 word base")?,
        metadata(&mut read, 4, "page 1 word base")?,
    ]
    .map(|value| {
        u16::try_from(value)
            .ok()
            .filter(|&address| {
                usize::from(address) < 0x8000
                    && usize::from(address) % PresentedObjTiles::WORDS_PER_TILE == 0
            })
            .ok_or_else(|| format!("presented OBJ cache page base is invalid: {value}"))
    });
    let [page_0_base, page_1_base] = [page_bases[0].clone()?, page_bases[1].clone()?];

    let mut tile_word_addresses = Vec::new();
    let mut tile_pixels = Vec::new();
    let mut seen_addresses = [false; PresentedObjTiles::MAX_TILE_COUNT];
    for slot in 0..PRESENTED_OBJ_CACHE_SLOT_COUNT {
        let slot_index = i32::try_from(slot).expect("OBJ cache slot count fits i32");
        let validity = read(PRESENTED_OBJ_CACHE_VALID_FIELD, slot_index)
            .ok_or_else(|| format!("presented OBJ cache validity {slot} is unavailable"))?;
        if !(0..=2).contains(&validity) {
            return Err(format!(
                "presented OBJ cache validity {slot} is invalid: {validity}"
            ));
        }

        let value = read(PRESENTED_OBJ_CACHE_WORD_ADDRESS_FIELD, slot_index)
            .ok_or_else(|| format!("presented OBJ cache word address {slot} is unavailable"))?;
        if validity == 0 {
            if value != -1 {
                return Err(format!(
                    "invalid presented OBJ cache slot {slot} has word address {value}"
                ));
            }
            continue;
        }
        let address = u16::try_from(value)
            .ok()
            .filter(|&address| {
                usize::from(address) < 0x8000
                    && usize::from(address) % PresentedObjTiles::WORDS_PER_TILE == 0
            })
            .ok_or_else(|| {
                format!("presented OBJ cache word address {slot} is invalid: {value}")
            })?;
        let (page_base, page_slot) = if slot < PRESENTED_OBJ_CACHE_PAGE_TILE_COUNT {
            (page_0_base, slot)
        } else {
            (page_1_base, slot - PRESENTED_OBJ_CACHE_PAGE_TILE_COUNT)
        };
        let expected_address =
            (usize::from(page_base) + page_slot * PresentedObjTiles::WORDS_PER_TILE) & 0x7fff;
        if usize::from(address) != expected_address {
            return Err(format!(
                "presented OBJ cache word address {slot} is {address:#06x}, expected {expected_address:#06x}"
            ));
        }
        let tile_index = usize::from(address) / PresentedObjTiles::WORDS_PER_TILE;
        if std::mem::replace(&mut seen_addresses[tile_index], true) {
            return Err(format!(
                "presented OBJ cache repeats physical word address {address:#06x}"
            ));
        }
        tile_word_addresses.push(address);
        let pixel_base = slot
            .checked_mul(PresentedObjTiles::PIXELS_PER_TILE)
            .expect("fixed OBJ cache dimensions cannot overflow");
        for pixel in 0..PresentedObjTiles::PIXELS_PER_TILE {
            let index = pixel_base + pixel;
            let value = read(
                PRESENTED_OBJ_CACHE_PIXELS_FIELD,
                i32::try_from(index).expect("OBJ cache pixel index fits i32"),
            )
            .ok_or_else(|| format!("presented OBJ cache pixel {index} is unavailable"))?;
            let pixel = u8::try_from(value)
                .ok()
                .filter(|&pixel| pixel < 16)
                .ok_or_else(|| format!("presented OBJ cache pixel {index} is invalid: {value}"))?;
            tile_pixels.push(pixel);
        }
    }

    PresentedObjTiles::new(tile_word_addresses, tile_pixels)
        .map(Some)
        .ok_or_else(|| "presented OBJ tile receipt has an invalid shape".to_string())
}

fn snes9x_presented_obj_tiles(oracle: &LibretroCore) -> Result<Option<PresentedObjTiles>, String> {
    decode_snes9x_presented_obj_tiles(|field, index| oracle.debug_ppu_value(field, index))
}

fn snes9x_presented_oam_bytes(oracle: &LibretroCore) -> Result<Option<Vec<u8>>, String> {
    if oracle.debug_ppu_value(20, 0).is_none() {
        return Ok(None);
    }
    let bytes = (0..PresentedOam::BYTE_COUNT)
        .map(|index| {
            let value = oracle
                .debug_ppu_value(20, index as i32)
                .ok_or_else(|| format!("presented OAM byte {index} is unavailable"))?;
            u8::try_from(value)
                .map_err(|_| format!("presented OAM byte {index} is invalid: {value}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(bytes))
}

fn snes9x_presented_oam(oracle: &LibretroCore) -> Result<Option<PresentedOam>, String> {
    let Some(bytes) = snes9x_presented_oam_bytes(oracle)? else {
        return Ok(None);
    };
    PresentedOam::new(bytes)
        .map(Some)
        .ok_or_else(|| "presented OAM receipt has an invalid shape".to_string())
}

fn snes9x_presented_cgram(oracle: &LibretroCore) -> Result<Option<PresentedCgram>, String> {
    if oracle.debug_ppu_value(36, 0).is_none() {
        return Ok(None);
    }
    let colors = (0..PresentedCgram::COLOR_COUNT)
        .map(|index| {
            let value = oracle
                .debug_ppu_value(36, index as i32)
                .ok_or_else(|| format!("presented CGRAM color {index} is unavailable"))?;
            u16::try_from(value)
                .ok()
                .filter(|&color| color <= 0x7fff)
                .ok_or_else(|| format!("presented CGRAM color {index} is invalid: {value}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    PresentedCgram::new(colors)
        .map(Some)
        .ok_or_else(|| "presented CGRAM receipt has an invalid shape".to_string())
}

fn snes9x_original_timing_host_receipts(
    oracle: &LibretroCore,
    capture: &LibretroFrame,
    previous_video: Option<&PresentedOracleVideo>,
    presented_bg_tilemap_cache: &mut PresentedBgTilemapCache,
    frame: u32,
    input: u16,
    semantic: Vec<OriginalTimingSemanticReceipt>,
    nmi_acceptance_ppu_register_operands: Vec<NmiPpuRegisterOperands>,
) -> Result<OriginalTimingHostReceipts, String> {
    let acceptance_count = semantic
        .iter()
        .filter(|receipt| matches!(receipt, OriginalTimingSemanticReceipt::NmiAccepted(_)))
        .count();
    if acceptance_count != nmi_acceptance_ppu_register_operands.len() {
        return Err(format!(
            "Snes9x host emitted {acceptance_count} NMI acceptances but {} PPU-register operand generations",
            nmi_acceptance_ppu_register_operands.len(),
        ));
    }
    let song_end_poll_native_sample_offsets = oracle
        .debug_apu_port_writes_exact()?
        .ok_or_else(|| {
            "Snes9x host receipts require exact CPU-side APU-port instrumentation".to_string()
        })?
        .into_iter()
        .filter_map(|access| {
            let receipt = song_end_poll_native_sample_offset(
                access.program_counter,
                access.port,
                access.is_read,
                access.output_sample,
                capture.audio.len() / PresentedAudio::CHANNELS,
            );
            if receipt.is_some() && std::env::var_os("ZELDA3_DEBUG_SONG_POLL").is_some() {
                eprintln!(
                    "[SONG-POLL-SOURCE] frame={frame} pc={:06x} sample={} value={:02x} window={}",
                    access.program_counter,
                    access.output_sample,
                    access.value,
                    capture.audio.len() / PresentedAudio::CHANNELS,
                );
            }
            receipt
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut receipts = OriginalTimingHostReceipts::new(u64::from(frame), input, semantic)
        .with_nmi_acceptance_ppu_register_operands(nmi_acceptance_ppu_register_operands)
        .with_song_end_poll_native_sample_offsets(song_end_poll_native_sample_offsets);
    receipts = receipts.with_presented_audio(
        PresentedAudio::new(capture.audio.clone())
            .ok_or_else(|| "Snes9x returned an invalid stereo audio receipt".to_string())?,
    );
    if let Some(receipt) = snes9x_presented_animated_bg_tiles(oracle)? {
        receipts = receipts.with_presented_animated_bg_tiles(receipt);
    }
    let geometry = snes9x_presented_scanout_geometry(oracle)?;
    if let Some(receipt) = snes9x_presented_inidisp(oracle, geometry, capture, previous_video)? {
        receipts = receipts.with_presented_inidisp(receipt);
    }
    if let Some(receipt) = geometry {
        receipts = receipts.with_presented_scanout_geometry(receipt);
    }
    if let Some(receipt) = snes9x_presented_hud_tilemap(oracle)? {
        receipts = receipts.with_presented_hud_tilemap(receipt);
    }
    receipts = receipts.with_presented_dialogue_text(snes9x_presented_dialogue_text(oracle)?);
    if let Some(receipt) = snes9x_presented_bg_tilemaps(oracle, presented_bg_tilemap_cache)? {
        receipts = receipts.with_presented_bg_tilemaps(receipt);
    }
    if let Some(receipt) = snes9x_presented_bg_scroll(oracle)? {
        receipts = receipts.with_presented_bg_scroll(receipt);
    }
    if let Some(receipt) = snes9x_presented_mode7_transform(oracle)? {
        receipts = receipts.with_presented_mode7_transform(receipt);
    }
    if let Some(receipt) = snes9x_presented_window_mask(oracle)? {
        receipts = receipts.with_presented_window_mask(receipt);
    }
    if let Some(receipt) = snes9x_presented_cgram(oracle)? {
        receipts = receipts.with_presented_cgram(receipt);
    }
    if let Some(receipt) = snes9x_presented_oam(oracle)? {
        receipts = receipts.with_presented_oam(receipt);
    }
    if let Some(receipt) = snes9x_presented_obj_tiles(oracle)? {
        receipts = receipts.with_presented_obj_tiles(receipt);
    }
    Ok(receipts)
}

fn semantic_receipts_from_dma_ledger(
    events: &[crate::libretro_core::LibretroDmaLedgerEvent],
) -> Result<Vec<OriginalTimingSemanticReceipt>, String> {
    validate_complete_dma_outers(events)?;
    let mut receipts = Vec::new();
    for event in events.iter().filter(|event| event.fields[0] == 4) {
        let channel_mask = u8::try_from(event.fields[2]).map_err(|_| {
            format!(
                "completed Snes9x DMA outer has invalid channel mask {}",
                event.fields[2]
            )
        })?;
        // Snes9x still records the outer `$420b` semantic for a zero mask,
        // but no DMA publication occurred. Keep validating that chronology
        // above, then omit it from Zelda's semantic receipt stream.
        if channel_mask != 0 {
            receipts.push(OriginalTimingSemanticReceipt::DmaPublicationCompleted { channel_mask });
        }
    }
    Ok(receipts)
}

fn first_nmi_dma_ledger_slice(
    events: &[crate::libretro_core::LibretroDmaLedgerEvent],
) -> Result<Option<(i32, Vec<crate::libretro_core::LibretroDmaLedgerEvent>)>, String> {
    let Some(begin) = events.iter().find(|event| {
        event.fields[0] == 0
            && event.fields[2] == 0x07
            && (event.fields[27] & 0x00ff_ffff) == 0x008a38
    }) else {
        return Ok(None);
    };
    let outer = begin.fields[1];
    let selected = events
        .iter()
        .filter(|event| event.fields[1] == outer)
        .cloned()
        .collect::<Vec<_>>();
    validate_first_nmi_dma_ledger(&selected)?;
    Ok(Some((outer, selected)))
}

fn validate_first_nmi_dma_ledger(
    events: &[crate::libretro_core::LibretroDmaLedgerEvent],
) -> Result<(), String> {
    let Some(first) = events.first() else {
        return Err("selected DMA ledger is empty".to_string());
    };
    let outer = first.fields[1];
    if first.fields[0] != 0 || first.fields[2] != 0x07 || first.fields[9] != 1 {
        return Err(format!(
            "invalid first-NMI DMA outer-begin event: {first:?}"
        ));
    }
    let last = events.last().expect("checked nonempty");
    if last.fields[0] != 4
        || last.fields[1] != outer
        || last.fields[2] != 0x07
        || last.fields[9] != 1
    {
        return Err(format!("invalid first-NMI DMA outer-end event: {last:?}"));
    }
    if events.iter().any(|event| event.fields[1] != outer) {
        return Err("selected DMA ledger crosses outer-transfer ownership".to_string());
    }

    let channel_markers = events
        .iter()
        .filter(|event| matches!(event.fields[0], 1 | 3))
        .map(|event| (event.fields[0], event.fields[2], event.fields[9]))
        .collect::<Vec<_>>();
    if channel_markers
        != vec![
            (1, 0, 1),
            (3, 0, 1),
            (1, 1, 1),
            (3, 1, 1),
            (1, 2, 1),
            (3, 2, 1),
        ]
    {
        return Err(format!(
            "first-NMI DMA channel ownership is not the ordered 0/1/2 mask-$07 sequence: {channel_markers:?}"
        ));
    }

    let bytes = events
        .iter()
        .filter(|event| event.fields[0] == 2)
        .collect::<Vec<_>>();
    if bytes.is_empty() {
        return Err("first-NMI DMA ledger contains no byte transactions".to_string());
    }
    let mut channel_ordinals = [0i32; 8];
    for (global, event) in bytes.iter().enumerate() {
        let channel = usize::try_from(event.fields[2])
            .ok()
            .filter(|channel| *channel < 8)
            .ok_or_else(|| format!("invalid DMA byte channel in {event:?}"))?;
        if event.fields[3] != global as i32
            || event.fields[4] != channel_ordinals[channel]
            || event.fields[9] != 1
        {
            return Err(format!(
                "non-contiguous or incomplete first-NMI DMA byte receipt at global ordinal {global}: {event:?}"
            ));
        }
        let expected_a_bus = (event.fields[51] << 16) | event.fields[50];
        let expected_remaining = (event.fields[49] - 1) & 0xffff;
        let a_increment = if event.fields[47] != 0 {
            0
        } else if event.fields[48] != 0 {
            -1
        } else {
            1
        };
        let expected_a_address = (event.fields[50] + a_increment) & 0xffff;
        if !matches!(event.fields[5], 0 | 1)
            || event.fields[6] != expected_a_bus
            || !(0x2100..=0x21ff).contains(&event.fields[7])
            || !(0..=0xff).contains(&event.fields[8])
            || event.fields[63] != expected_remaining
            || event.fields[64] != expected_a_address
        {
            return Err(format!(
                "incomplete or inconsistent first-NMI DMA A/B semantic receipt at global ordinal {global}: {event:?}"
            ));
        }
        channel_ordinals[channel] += 1;
    }
    Ok(())
}

fn compact_dma_ledger(
    events: &[crate::libretro_core::LibretroDmaLedgerEvent],
) -> SmpBootstrapDeltaSequence {
    compact_delta_integer_sequence_with_zstd(
        DMA_LEDGER_FIELDS,
        events.iter().map(|event| event.fields.map(i64::from)),
        true,
    )
}

fn compact_byte_snapshot(bytes: &[u8]) -> SmpBootstrapDeltaSequence {
    compact_delta_integer_sequence_with_zstd(
        ["byte"],
        bytes.iter().map(|byte| [i64::from(*byte)]),
        true,
    )
}

fn compact_smp_output_port_writes(
    writes: &[crate::libretro_core::LibretroSmpOutputPortWrite],
) -> SmpBootstrapDeltaSequence {
    compact_delta_integer_sequence(
        [
            "absolute_cycle",
            "port",
            "value",
            "origin_pc",
            "opcode",
            "opcode_cycle",
            "v_counter",
            "cpu_cycle",
            "cpu_program_counter",
            "cpu_reference_time",
            "cpu_remainder",
            "smp_clock",
            "next_pc",
            "dsp_clock",
            "dsp_phase",
            "output_sample",
        ],
        writes.iter().map(|write| {
            [
                write.absolute_cycle as i64,
                i64::from(write.port),
                i64::from(write.value),
                i64::from(write.origin_pc),
                i64::from(write.opcode),
                i64::from(write.opcode_cycle),
                i64::from(write.v_counter),
                i64::from(write.cpu_cycle),
                i64::from(write.cpu_program_counter),
                i64::from(write.cpu_reference_time),
                i64::from(write.cpu_remainder),
                i64::from(write.smp_clock),
                i64::from(write.next_pc),
                i64::from(write.dsp_clock),
                i64::from(write.dsp_phase),
                i64::from(write.output_sample),
            ]
        }),
    )
}

fn append_smp_instruction_frame(
    accumulated: &mut Vec<crate::libretro_core::LibretroSmpInstruction>,
    frame: Vec<crate::libretro_core::LibretroSmpInstruction>,
) {
    let mut frame = frame.into_iter().peekable();
    if accumulated
        .last()
        .zip(frame.peek())
        .is_some_and(|(left, right)| {
            left.absolute_cycle == right.absolute_cycle
                && left.program_counter == right.program_counter
                && left.opcode == right.opcode
        })
    {
        *accumulated.last_mut().unwrap() = frame.next().unwrap();
    }
    accumulated.extend(frame);
}

fn append_framed_smp_instruction_frame(
    accumulated: &mut Vec<FramedSmpInstruction>,
    frame_index: u32,
    frame: Vec<crate::libretro_core::LibretroSmpInstruction>,
) {
    let mut frame = frame
        .into_iter()
        .map(|instruction| FramedSmpInstruction {
            frame: frame_index,
            instruction,
        })
        .peekable();
    if accumulated
        .last()
        .zip(frame.peek())
        .is_some_and(|(left, right)| {
            left.instruction.absolute_cycle == right.instruction.absolute_cycle
                && left.instruction.program_counter == right.instruction.program_counter
                && left.instruction.opcode == right.instruction.opcode
        })
    {
        *accumulated.last_mut().unwrap() = frame.next().unwrap();
    }
    accumulated.extend(frame);
}

fn smp_bootstrap_handoff_index(
    instructions: &[crate::libretro_core::LibretroSmpInstruction],
) -> Option<usize> {
    instructions.windows(2).position(|pair| {
        pair[0].program_counter == 0xfffb
            && pair[0].opcode == 0x1f
            && pair[1].program_counter == 0x0800
    })
}

fn framed_smp_bootstrap_handoff_index(instructions: &[FramedSmpInstruction]) -> Option<usize> {
    instructions.windows(2).position(|pair| {
        pair[0].instruction.program_counter == 0xfffb
            && pair[0].instruction.opcode == 0x1f
            && pair[1].instruction.program_counter == 0x0800
    })
}

fn smp_instruction_frame_cycle(instruction: &FramedSmpInstruction) -> i64 {
    i64::from(instruction.instruction.output_sample) * 32
        + i64::from(instruction.instruction.dsp_phase)
        + i64::from(instruction.instruction.smp_clock)
}

fn smp_instruction_bracket_for_apu_access(
    instructions: &[FramedSmpInstruction],
    access: &FramedApuPortAccess,
) -> Result<(usize, usize), String> {
    let owning_boundary = |apu_cycle: i32| {
        instructions
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, framed)| {
                (framed.frame == access.frame
                    && smp_instruction_frame_cycle(framed) <= i64::from(apu_cycle))
                .then_some(index)
            })
    };
    let before = owning_boundary(access.access.apu_cycle_before).ok_or_else(|| {
        format!(
            "frame {} has no SMP boundary owning pre-sync APU cycle {}",
            access.frame, access.access.apu_cycle_before
        )
    })?;
    let after = owning_boundary(access.access.apu_cycle_after).ok_or_else(|| {
        format!(
            "frame {} has no SMP boundary owning post-sync APU cycle {}",
            access.frame, access.access.apu_cycle_after
        )
    })?;
    if after < before {
        return Err(format!(
            "SMP boundary order regressed across APU sync: {before} -> {after}"
        ));
    }
    Ok((before, after))
}

fn framed_smp_instruction_digest_fields() -> &'static [&'static str] {
    &[
        "frame",
        "absolute_cycle",
        "program_counter",
        "opcode",
        "a",
        "x",
        "y",
        "stack_pointer",
        "status",
        "timer0_stage1",
        "timer0_stage2",
        "timer0_stage3",
        "output_sample",
        "dsp_phase",
        "smp_clock",
        "direct_page_0_11[0..12]",
        "boundary_opcode_cycle",
        "op_step_calls",
        "max_continuation_opcode_cycle",
    ]
}

fn framed_smp_instruction_digest(instructions: &[FramedSmpInstruction]) -> String {
    let mut bytes = Vec::with_capacity(instructions.len() * 128);
    for framed in instructions {
        bytes.extend_from_slice(&framed.frame.to_le_bytes());
        let instruction = &framed.instruction;
        bytes.extend_from_slice(&instruction.absolute_cycle.to_le_bytes());
        for value in [
            instruction.program_counter,
            instruction.opcode,
            instruction.a,
            instruction.x,
            instruction.y,
            instruction.stack_pointer,
            instruction.status,
            instruction.timer0_stage1,
            instruction.timer0_stage2,
            instruction.timer0_stage3,
            instruction.output_sample,
            instruction.dsp_phase,
            instruction.smp_clock,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in instruction.direct_page_0_11 {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in [
            instruction.boundary_opcode_cycle,
            instruction.op_step_calls,
            instruction.max_continuation_opcode_cycle,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    parity::evidence::sha256_bytes(&bytes)
}

fn compact_framed_smp_instructions(
    instructions: &[FramedSmpInstruction],
) -> SmpBootstrapDeltaSequence {
    compact_delta_integer_sequence_with_zstd(
        [
            "frame",
            "absolute_cycle",
            "program_counter",
            "opcode",
            "a",
            "x",
            "y",
            "stack_pointer",
            "status",
            "timer0_stage1",
            "timer0_stage2",
            "timer0_stage3",
            "output_sample",
            "dsp_phase",
            "smp_clock",
            "direct_page_0",
            "direct_page_1",
            "direct_page_2",
            "direct_page_3",
            "direct_page_4",
            "direct_page_5",
            "direct_page_6",
            "direct_page_7",
            "direct_page_8",
            "direct_page_9",
            "direct_page_10",
            "direct_page_11",
            "boundary_opcode_cycle",
            "op_step_calls",
            "max_continuation_opcode_cycle",
        ],
        instructions.iter().map(|framed| {
            let instruction = &framed.instruction;
            [
                i64::from(framed.frame),
                instruction.absolute_cycle as i64,
                i64::from(instruction.program_counter),
                i64::from(instruction.opcode),
                i64::from(instruction.a),
                i64::from(instruction.x),
                i64::from(instruction.y),
                i64::from(instruction.stack_pointer),
                i64::from(instruction.status),
                i64::from(instruction.timer0_stage1),
                i64::from(instruction.timer0_stage2),
                i64::from(instruction.timer0_stage3),
                i64::from(instruction.output_sample),
                i64::from(instruction.dsp_phase),
                i64::from(instruction.smp_clock),
                i64::from(instruction.direct_page_0_11[0]),
                i64::from(instruction.direct_page_0_11[1]),
                i64::from(instruction.direct_page_0_11[2]),
                i64::from(instruction.direct_page_0_11[3]),
                i64::from(instruction.direct_page_0_11[4]),
                i64::from(instruction.direct_page_0_11[5]),
                i64::from(instruction.direct_page_0_11[6]),
                i64::from(instruction.direct_page_0_11[7]),
                i64::from(instruction.direct_page_0_11[8]),
                i64::from(instruction.direct_page_0_11[9]),
                i64::from(instruction.direct_page_0_11[10]),
                i64::from(instruction.direct_page_0_11[11]),
                i64::from(instruction.boundary_opcode_cycle),
                i64::from(instruction.op_step_calls),
                i64::from(instruction.max_continuation_opcode_cycle),
            ]
        }),
        true,
    )
}

fn smp_bootstrap_steps_match(
    left: &SmpBootstrapInstructionStep,
    right: &SmpBootstrapInstructionStep,
) -> bool {
    left.absolute_end_cycle - left.absolute_start_cycle
        == right.absolute_end_cycle - right.absolute_start_cycle
        && left.origin_pc == right.origin_pc
        && left.opcode == right.opcode
        && left.boundary_opcode_cycle == right.boundary_opcode_cycle
        && left.op_step_calls == right.op_step_calls
        && left.max_continuation_opcode_cycle == right.max_continuation_opcode_cycle
}

fn smp_bootstrap_span(
    steps: &[SmpBootstrapInstructionStep],
    pattern_len: usize,
    repeat_count: usize,
) -> SmpBootstrapInstructionSpan {
    let absolute_start_cycle = steps[0].absolute_start_cycle;
    let repeat_cycle_stride = steps[pattern_len - 1].absolute_end_cycle - absolute_start_cycle;
    let absolute_end_cycle = steps[pattern_len * repeat_count - 1].absolute_end_cycle;
    let instructions = steps[..pattern_len]
        .iter()
        .map(|step| SmpBootstrapPatternInstruction {
            start_cycle_offset: step.absolute_start_cycle - absolute_start_cycle,
            end_cycle_offset: step.absolute_end_cycle - absolute_start_cycle,
            origin_pc: step.origin_pc,
            opcode: step.opcode,
            boundary_opcode_cycle: step.boundary_opcode_cycle,
            op_step_calls: step.op_step_calls,
            max_continuation_opcode_cycle: step.max_continuation_opcode_cycle,
        })
        .collect();
    SmpBootstrapInstructionSpan {
        absolute_start_cycle,
        absolute_end_cycle,
        repeat_count,
        repeat_cycle_stride,
        instructions,
    }
}

fn compact_smp_bootstrap_steps(
    steps: &[SmpBootstrapInstructionStep],
) -> Vec<SmpBootstrapInstructionSpan> {
    let mut spans = Vec::new();
    let mut literal_start = 0;
    let mut index = 0;
    while index < steps.len() {
        let mut best = None::<(usize, usize, usize)>;
        for pattern_len in 1..=16.min((steps.len() - index) / 2) {
            let mut repeat_count = 1;
            while index + (repeat_count + 1) * pattern_len <= steps.len()
                && (0..pattern_len).all(|offset| {
                    smp_bootstrap_steps_match(
                        &steps[index + offset],
                        &steps[index + repeat_count * pattern_len + offset],
                    )
                })
            {
                repeat_count += 1;
            }
            let saved_instructions = (repeat_count - 1) * pattern_len;
            if repeat_count >= 2
                && best.is_none_or(|(best_saved, best_len, _)| {
                    (saved_instructions, pattern_len) > (best_saved, best_len)
                })
            {
                best = Some((saved_instructions, pattern_len, repeat_count));
            }
        }
        let Some((_, pattern_len, repeat_count)) = best else {
            index += 1;
            continue;
        };
        if literal_start < index {
            let literal = &steps[literal_start..index];
            spans.push(smp_bootstrap_span(literal, literal.len(), 1));
        }
        let repeated = &steps[index..index + pattern_len * repeat_count];
        spans.push(smp_bootstrap_span(repeated, pattern_len, repeat_count));
        index += pattern_len * repeat_count;
        literal_start = index;
    }
    if literal_start < steps.len() {
        let literal = &steps[literal_start..];
        spans.push(smp_bootstrap_span(literal, literal.len(), 1));
    }
    spans
}

fn compact_smp_bootstrap_instruction_sequence(
    instructions: &[crate::libretro_core::LibretroSmpInstruction],
    first_cc: &crate::libretro_core::LibretroSmpOutputPortWrite,
) -> Result<SmpBootstrapInstructionSequence, String> {
    let cc_index = instructions
        .iter()
        .position(|instruction| {
            instruction.program_counter == first_cc.origin_pc
                && instruction.opcode == first_cc.opcode
                && instruction.absolute_cycle <= first_cc.absolute_cycle
        })
        .ok_or_else(|| {
            "SMP trace has no instruction boundary owning the first CC write".to_string()
        })?;
    if cc_index + 1 >= instructions.len() {
        return Err("SMP trace has no successor boundary after the first CC write".to_string());
    }
    if instructions[cc_index + 1].absolute_cycle != first_cc.absolute_cycle {
        return Err(format!(
            "first CC write is at cycle {}, but its successor boundary is at {}",
            first_cc.absolute_cycle,
            instructions[cc_index + 1].absolute_cycle
        ));
    }
    let handoff_index = smp_bootstrap_handoff_index(instructions)
        .ok_or_else(|| "SMP trace has no final $fffb/1f -> $0800 handoff".to_string())?;
    let mut steps = Vec::with_capacity(handoff_index + 1);
    for index in 0..=handoff_index {
        let instruction = &instructions[index];
        let absolute_end_cycle = instructions[index + 1].absolute_cycle;
        if absolute_end_cycle < instruction.absolute_cycle {
            return Err(format!(
                "SMP instruction cycle regressed at trace index {index}: {} -> {absolute_end_cycle}",
                instruction.absolute_cycle
            ));
        }
        steps.push(SmpBootstrapInstructionStep {
            absolute_start_cycle: instruction.absolute_cycle,
            absolute_end_cycle,
            origin_pc: instruction.program_counter,
            opcode: instruction.opcode,
            boundary_opcode_cycle: instruction.boundary_opcode_cycle,
            op_step_calls: instruction.op_step_calls,
            max_continuation_opcode_cycle: instruction.max_continuation_opcode_cycle,
        });
    }
    if steps.first().map(|step| step.absolute_start_cycle) != Some(0) {
        return Err(
            "SMP bootstrap instruction trace does not begin at reset cycle zero".to_string(),
        );
    }
    let absolute_end_cycle = steps.last().unwrap().absolute_end_cycle;
    Ok(SmpBootstrapInstructionSequence {
        encoding: "repeated-span-v1",
        instruction_count: steps.len(),
        absolute_start_cycle: 0,
        absolute_end_cycle,
        spans: compact_smp_bootstrap_steps(&steps),
    })
}

fn snes9x_smp_trace_provenance(
    core_path: &str,
    rom_path: &str,
    oracle: &LibretroCore,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let lock: serde_json::Value = serde_json::from_str(include_str!(
        "../../external/snes9x-libretro/oracle-lock.json"
    ))?;
    let source_revision = lock
        .get("source_revision")
        .and_then(serde_json::Value::as_str)
        .ok_or("oracle-lock.json has no source_revision")?;
    let trace_patch = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/patches/zelda3-trace.patch");
    Ok(serde_json::json!({
        "kind": "provenance",
        "schema": 1,
        "core": {
            "library_name": oracle.library_name,
            "library_version": oracle.library_version,
            "sha256": parity::evidence::sha256_file(Path::new(core_path))?,
        },
        "rom": {
            "sha256": parity::evidence::sha256_file(Path::new(rom_path))?,
        },
        "source": {
            "revision": source_revision,
            "trace_patch_sha256": parity::evidence::sha256_file(&trace_patch)?,
        },
        "cpu_to_smp_ratio": {
            "numerator": 15_664,
            "denominator": 328_125,
        },
    }))
}

fn write_snes9x_smp_first_nmi_header<W: Write>(
    writer: &mut W,
    core_path: &str,
    rom_path: &str,
    oracle: &LibretroCore,
) -> Result<(), Box<dyn Error>> {
    let mut provenance = snes9x_smp_trace_provenance(core_path, rom_path, oracle)?;
    let prefix_fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/fixtures/zelda3-cold-apu-bootstrap.jsonl");
    provenance
        .as_object_mut()
        .ok_or("Snes9x trace provenance is not a JSON object")?
        .insert(
            "prefix_fixture".to_string(),
            serde_json::json!({
                "path": "external/snes9x-libretro/fixtures/zelda3-cold-apu-bootstrap.jsonl",
                "sha256": parity::evidence::sha256_file(&prefix_fixture)?,
                "terminal_event": "final_$fffb/1f_to_$0800_ipl_handoff",
            }),
        );
    serde_json::to_writer(&mut *writer, &provenance)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

fn write_snes9x_first_nmi_dma_setup_header<W: Write>(
    writer: &mut W,
    core_path: &str,
    rom_path: &str,
    oracle: &LibretroCore,
    load_sram_path: Option<&Path>,
    initial_sram: &[u8],
) -> Result<(), Box<dyn Error>> {
    let mut provenance = snes9x_smp_trace_provenance(core_path, rom_path, oracle)?;
    let core_receipt_path = PathBuf::from(format!("{core_path}.json"));
    let core_receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(&core_receipt_path).map_err(|error| {
            format!(
                "first-NMI DMA-setup core has no readable build receipt {}: {error}",
                core_receipt_path.display()
            )
        })?)?;
    let recorded_core_sha = core_receipt["core_sha256"]
        .as_str()
        .ok_or("first-NMI DMA-setup core receipt has no core_sha256")?;
    if recorded_core_sha != provenance["core"]["sha256"] {
        return Err("first-NMI DMA-setup core receipt does not bind the loaded dylib".into());
    }
    if core_receipt["variant"] != "trace"
        || core_receipt["source_revision"] != provenance["source"]["revision"]
        || core_receipt["patch_sha256s"]
            .as_array()
            .is_none_or(|patches| patches.is_empty())
    {
        return Err("first-NMI DMA-setup core receipt is not a pinned trace build".into());
    }
    let prefix_fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/fixtures/zelda3-cold-apu-first-nmi.jsonl");
    let provenance = provenance
        .as_object_mut()
        .ok_or("Snes9x trace provenance is not a JSON object")?;
    provenance.insert(
        "prefix_fixture".to_string(),
        serde_json::json!({
            "path": "external/snes9x-libretro/fixtures/zelda3-cold-apu-first-nmi.jsonl",
            "sha256": parity::evidence::sha256_file(&prefix_fixture)?,
            "terminal_event": "completed_$0080e1_lda_$2140_semantic_and_kind2_timing",
        }),
    );
    provenance.insert(
        "core_build_receipt".to_string(),
        serde_json::json!({
            "schema": core_receipt["schema"],
            "variant": core_receipt["variant"],
            "source_revision": core_receipt["source_revision"],
            "patch_sha256": core_receipt["patch_sha256"],
            "patch_sha256s": core_receipt["patch_sha256s"],
            "sha256": parity::evidence::sha256_file(&core_receipt_path)?,
        }),
    );
    provenance.insert(
        "initial_sram".to_string(),
        first_nmi_dma_setup_initial_sram_provenance(load_sram_path, initial_sram)?,
    );
    serde_json::to_writer(&mut *writer, &provenance)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

fn write_snes9x_first_nmi_dma_header<W: Write>(
    writer: &mut W,
    core_path: &str,
    rom_path: &str,
    oracle: &LibretroCore,
    load_sram_path: Option<&Path>,
    initial_sram: &[u8],
) -> Result<(), Box<dyn Error>> {
    let mut provenance = snes9x_smp_trace_provenance(core_path, rom_path, oracle)?;
    let core_receipt_path = PathBuf::from(format!("{core_path}.json"));
    let core_receipt: serde_json::Value = serde_json::from_slice(&fs::read(&core_receipt_path)?)?;
    let patch_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/patches/zelda3-dma-ledger.patch");
    let patch_sha256 = parity::evidence::sha256_file(&patch_path)?;
    let patch_sha256s = core_receipt["patch_sha256s"]
        .as_array()
        .ok_or("first-NMI DMA core receipt has no patch_sha256s")?;
    if core_receipt["variant"] != "trace"
        || core_receipt["source_revision"] != provenance["source"]["revision"]
        || core_receipt["core_sha256"] != provenance["core"]["sha256"]
        || patch_sha256s.len() != 5
        || patch_sha256s.last().and_then(serde_json::Value::as_str) != Some(patch_sha256.as_str())
    {
        return Err(
            "first-NMI DMA capture requires the pinned five-patch DMA-ledger trace core".into(),
        );
    }
    let prefix_fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/fixtures/zelda3-cold-first-nmi-dma-setup.jsonl");
    let provenance = provenance
        .as_object_mut()
        .ok_or("Snes9x trace provenance is not a JSON object")?;
    provenance.insert(
        "prefix_fixture".to_string(),
        serde_json::json!({
            "path": "external/snes9x-libretro/fixtures/zelda3-cold-first-nmi-dma-setup.jsonl",
            "sha256": parity::evidence::sha256_file(&prefix_fixture)?,
            "terminal_event": "excluded_$008a35_raw_fetch_V226_H714_to_H722",
        }),
    );
    provenance.insert(
        "core_build_receipt".to_string(),
        serde_json::json!({
            "schema": core_receipt["schema"],
            "variant": core_receipt["variant"],
            "source_revision": core_receipt["source_revision"],
            "patch_sha256": core_receipt["patch_sha256"],
            "patch_sha256s": core_receipt["patch_sha256s"],
            "sha256": parity::evidence::sha256_file(&core_receipt_path)?,
        }),
    );
    provenance.insert(
        "initial_sram".to_string(),
        first_nmi_dma_setup_initial_sram_provenance(load_sram_path, initial_sram)?,
    );
    provenance.insert(
        "dma_ledger_patch".to_string(),
        serde_json::json!({
            "path": "external/snes9x-libretro/patches/zelda3-dma-ledger.patch",
            "sha256": patch_sha256,
            "core_route_selector": false,
            "buffer_scope": "one retro_run",
            "overflow_policy": "fail-closed",
        }),
    );
    serde_json::to_writer(&mut *writer, &provenance)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

fn write_snes9x_first_nmi_return_header<W: Write>(
    writer: &mut W,
    core_path: &str,
    rom_path: &str,
    oracle: &LibretroCore,
    load_sram_path: Option<&Path>,
    initial_sram: &[u8],
) -> Result<(), Box<dyn Error>> {
    let mut provenance = snes9x_smp_trace_provenance(core_path, rom_path, oracle)?;
    let core_receipt_path = PathBuf::from(format!("{core_path}.json"));
    let core_receipt: serde_json::Value = serde_json::from_slice(&fs::read(&core_receipt_path)?)?;
    let dma_patch_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/patches/zelda3-dma-ledger.patch");
    let dma_patch_sha256 = parity::evidence::sha256_file(&dma_patch_path)?;
    let patch_sha256s = core_receipt["patch_sha256s"]
        .as_array()
        .ok_or("first-NMI return core receipt has no patch_sha256s")?;
    if core_receipt["variant"] != "trace"
        || core_receipt["source_revision"] != provenance["source"]["revision"]
        || core_receipt["core_sha256"] != provenance["core"]["sha256"]
        || patch_sha256s.len() != 5
        || patch_sha256s.last().and_then(serde_json::Value::as_str)
            != Some(dma_patch_sha256.as_str())
    {
        return Err(
            "first-NMI return capture requires the pinned five-patch DMA-ledger trace core".into(),
        );
    }
    let prefix_fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/snes9x-libretro/fixtures/zelda3-cold-first-nmi-dma.jsonl");
    let provenance = provenance
        .as_object_mut()
        .ok_or("Snes9x trace provenance is not a JSON object")?;
    provenance.insert(
        "prefix_fixture".to_string(),
        serde_json::json!({
            "path": "external/snes9x-libretro/fixtures/zelda3-cold-first-nmi-dma.jsonl",
            "sha256": parity::evidence::sha256_file(&prefix_fixture)?,
            "terminal_event": "completed_$008a35_sta_$420b_07_and_observed_$008a38_raw_fetch",
        }),
    );
    provenance.insert(
        "core_build_receipt".to_string(),
        serde_json::json!({
            "schema": core_receipt["schema"],
            "variant": core_receipt["variant"],
            "source_revision": core_receipt["source_revision"],
            "patch_sha256": core_receipt["patch_sha256"],
            "patch_sha256s": core_receipt["patch_sha256s"],
            "sha256": parity::evidence::sha256_file(&core_receipt_path)?,
        }),
    );
    provenance.insert(
        "initial_sram".to_string(),
        first_nmi_dma_setup_initial_sram_provenance(load_sram_path, initial_sram)?,
    );
    provenance.insert(
        "trace_domains".to_string(),
        serde_json::json!({
            "generic_text": ["frame", "hdma"],
            "implicit_generic_text": ["video/presented"],
            "cpu_timing": "all_source_transactions_in_one_retro_run",
            "dma": "all_complete_outers_after_committed_prefix",
            "cpu_apui": "all_joined_post_anchor_accesses_with_dsp_before_after",
            "smp_output_ports": "all_joined_post_anchor_writes",
            "capacity_policy": "fail_closed",
            "core_route_selector": false,
        }),
    );
    serde_json::to_writer(&mut *writer, &provenance)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

const FIRST_NMI_DMA_SETUP_INITIAL_SRAM_SHA256: &str =
    "d8a02e6e08a22377f919c7350e5bfa6117c6408db9a728815af7ad6e4e6b83bc";
const FIRST_NMI_DMA_SETUP_INITIAL_SRAM_BYTES: usize = 8_192;
const FIRST_NMI_DMA_SETUP_VALID_SLOT_MARKER_OFFSET: usize = 0x03e5;

fn first_nmi_dma_setup_initial_sram_provenance(
    load_sram_path: Option<&Path>,
    initial_sram: &[u8],
) -> Result<serde_json::Value, Box<dyn Error>> {
    let path = load_sram_path.ok_or(
        "first-NMI DMA-setup capture requires --load-sram routes/full_run/comparisons/continuous-audio/initial.srm",
    )?;
    let source_sram = fs::read(path).map_err(|error| {
        format!(
            "failed to read first-NMI DMA-setup initial SRAM {}: {error}",
            path.display()
        )
    })?;
    if source_sram != initial_sram {
        return Err(
            "first-NMI DMA-setup SRAM file does not match the bytes loaded into Snes9x".into(),
        );
    }
    if initial_sram.len() != FIRST_NMI_DMA_SETUP_INITIAL_SRAM_BYTES {
        return Err(format!(
            "first-NMI DMA-setup initial SRAM has {} bytes, expected {}",
            initial_sram.len(),
            FIRST_NMI_DMA_SETUP_INITIAL_SRAM_BYTES
        )
        .into());
    }
    let sha256 = parity::evidence::sha256_bytes(initial_sram);
    if sha256 != FIRST_NMI_DMA_SETUP_INITIAL_SRAM_SHA256 {
        return Err(format!(
            "first-NMI DMA-setup initial SRAM SHA-256 is {sha256}, expected {FIRST_NMI_DMA_SETUP_INITIAL_SRAM_SHA256}"
        )
        .into());
    }
    let slot_marker = initial_sram
        .get(
            FIRST_NMI_DMA_SETUP_VALID_SLOT_MARKER_OFFSET
                ..FIRST_NMI_DMA_SETUP_VALID_SLOT_MARKER_OFFSET + 2,
        )
        .ok_or("first-NMI DMA-setup initial SRAM has no save-slot marker")?;
    if slot_marker != [0xaa, 0x55] {
        return Err(
            "first-NMI DMA-setup initial SRAM has no AA55 valid-slot marker at $03e5".into(),
        );
    }
    Ok(serde_json::json!({
        "source": "routes/full_run/comparisons/continuous-audio/initial.srm",
        "sha256": sha256,
        "bytes": initial_sram.len(),
        "valid_slot_marker": {
            "offset": FIRST_NMI_DMA_SETUP_VALID_SLOT_MARKER_OFFSET,
            "bytes": [0xaa, 0x55],
        },
    }))
}

fn write_snes9x_smp_bootstrap_header<W: Write>(
    writer: &mut W,
    core_path: &str,
    rom_path: &str,
    oracle: &LibretroCore,
    initial_oracle_state: &[u8],
) -> Result<(), Box<dyn Error>> {
    serde_json::to_writer(
        &mut *writer,
        &snes9x_smp_trace_provenance(core_path, rom_path, oracle)?,
    )?;
    writer.write_all(b"\n")?;

    let snd = crate::snes9x_apu_tools::snes9x_snapshot_block(initial_oracle_state, b"SND")
        .map_err(|error| format!("failed to parse initial Snes9x SND block: {error}"))?;
    const RAM_BYTES: usize = 0x1_0000;
    const SMP_INT_COUNT: usize = 41;
    if snd.len() < RAM_BYTES + SMP_INT_COUNT * 4 {
        return Err(format!("initial Snes9x SND block is truncated: {} bytes", snd.len()).into());
    }
    let value = |index: usize| {
        let start = RAM_BYTES + index * 4;
        u32::from_le_bytes(snd[start..start + 4].try_into().unwrap())
    };
    let status = ((value(8) != 0) as u8) << 7
        | ((value(9) != 0) as u8) << 6
        | ((value(10) != 0) as u8) << 5
        | ((value(11) != 0) as u8) << 4
        | ((value(12) != 0) as u8) << 3
        | ((value(13) != 0) as u8) << 2
        | ((value(14) != 0) as u8) << 1
        | (value(15) != 0) as u8;
    let timers = (0..3)
        .map(|timer| {
            let base = 20 + timer * 5;
            serde_json::json!({
                "enabled": value(base) != 0,
                "target": value(base + 1),
                "stage1_ticks": value(base + 2),
                "stage2_ticks": value(base + 3),
                "stage3_ticks": value(base + 4),
            })
        })
        .collect::<Vec<_>>();
    let (cpu_model_5a22, cpu_model_identity, wram_refresh_position) = oracle
        .debug_cpu_timing_model()
        .ok_or("SMP-bootstrap trace requires Snes9x CPU timing-model instrumentation")?;
    serde_json::to_writer(
        &mut *writer,
        &serde_json::json!({
            "kind": "reset-state",
            "absolute_cycle": 0,
            "clock": value(0) as i32,
            "opcode": value(1),
            "opcode_cycle": value(2),
            "pc": value(3),
            "sp": value(4),
            "a": value(5),
            "x": value(6),
            "y": value(7),
            "status": status,
            "ipl_rom_enabled": value(16) != 0,
            "dsp_address": value(17),
            "auxiliary_ram": [value(18), value(19)],
            "output_ports": &snd[0xf4..0xf8],
            "timers": timers,
            "ram_nonzero_bytes": snd[..RAM_BYTES].iter().filter(|byte| **byte != 0).count(),
            "cpu_reference_time": 0,
            "cpu_remainder": 0,
            "cpu_model_5a22": cpu_model_5a22,
            "cpu_model_identity": cpu_model_identity,
            "wram_refresh_position": wram_refresh_position,
        }),
    )?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::{
        append_smp_instruction_frame, cached_ledger_input, cached_oracle_checkpoint_sources,
        canonical_audio_digest, canonical_oracle_video_digest, canonical_rust_video_digest,
        checkpoint_member, cold_evidence_run_nonce, compact_byte_snapshot,
        compact_delta_integer_sequence, compact_delta_integer_sequence_with_zstd,
        compact_dma_ledger, compact_engine_state_mismatches, compact_framed_smp_instructions,
        compact_ordinal_cpu_apu_accesses, decode_snes9x_presented_obj_tiles,
        first_dsp_write_timing_mismatch, first_nmi_apui_anchor_indices, first_nmi_dma_ledger_slice,
        first_nmi_dma_setup_initial_sram_provenance, first_nmi_dma_setup_stop_index,
        first_nmi_dma_transaction_slice, first_nmi_return_start_index, fnv1a32,
        install_directory_atomically, last_spc_clock_witness, libretro_engine_state_receipt,
        oracle_preframe_snapshot_required, oracle_rng_sample_from_trace_line, paired_resume_paths,
        parse_debug_frame_selection, parse_paired_resume_capture,
        parse_rolling_paired_resume_capture, presented_video_rows_match_prior_surface,
        prune_rolling_paired_resume_captures, read_snes9x_retro_run_trace,
        replayable_input_artifact, resolve_engine_state_compare_start, resolve_replay_bundle,
        rolling_capture_frame_after, scan_all_policy, semantic_receipts_from_dma_ledger,
        semantic_trace_authority_available, should_render_video_frame,
        should_stop_after_first_mismatch, should_write_frame_receipt, smp_bootstrap_handoff_index,
        snes9x_presented_scanline_for_video_y, summarize_presented_obj_cache,
        summarize_value_domain, trace_events_with_rom_rng, validate_cold_evidence_invocation_id,
        validate_first_nmi_return_cpu_slice, validate_oracle_av_checkpoint_interval,
        validate_oracle_rng_samples_for_run, validate_paired_resume_provenance,
        validate_paired_resume_sram_selection, validate_replay_source_parents, vram_domain_receipt,
        write_cached_av_final_paired_resume, write_file_atomically, BootBoundaryState,
        FramedApuPortAccess, FramedCpuTimingTransaction, FramedSmpInstruction,
        OrdinalApuPortAccess, PairedResumeCapture, PendingFirstNmiReturnFixture,
        PlayCrashCheckpoint, PresentedOracleVideo, RollingPairedResumeCapture, ValueDomainDiff,
        VramDomainReceipt, PAIRED_RESUME_SCHEMA, PLAY_CRASH_CHECKPOINT_MAGIC,
    };
    use crate::libretro_core::{
        LibretroApuPortWrite, LibretroCpuTimingTransaction, LibretroDmaLedgerEvent,
        LibretroDspRegisterWrite, LibretroFrame, LibretroSmpInstruction,
    };
    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};
    use zelda3::{game_output::DspWriteEvent, OriginalTimingSemanticReceipt, RomRandomSample};

    fn write_paired_resume_test_generation(
        root: &Path,
        directory: &str,
        manifest_frame: u32,
        rust_checkpoint_frame: u32,
    ) -> PathBuf {
        let checkpoint_dir = root.join(directory);
        fs::create_dir_all(&checkpoint_dir).unwrap();
        let rust_checkpoint = PlayCrashCheckpoint {
            magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
            host_frame: rust_checkpoint_frame,
            input: 0,
            run_what: 0,
            game: zelda3::ZeldaState::new(),
        };
        let rust_state = bincode::serialize(&rust_checkpoint).unwrap();
        let oracle_state = b"oracle state";
        let original_timing = b"original timing sidecar";
        let semantic_trace = b"semantic trace sidecar";
        let initial_sram = b"initial SRAM";
        for (name, bytes) in [
            ("rust.z3state", rust_state.as_slice()),
            ("oracle.state", oracle_state.as_slice()),
            ("original-timing.resume.json", original_timing.as_slice()),
            ("semantic-trace.checkpoint.json", semantic_trace.as_slice()),
            ("initial.srm", initial_sram.as_slice()),
        ] {
            fs::write(checkpoint_dir.join(name), bytes).unwrap();
        }
        let manifest = serde_json::json!({
            "schema": PAIRED_RESUME_SCHEMA,
            "boundary": "pre-frame",
            "frame": manifest_frame,
            "rust_state": {
                "artifact": "rust.z3state",
                "sha256": parity::evidence::sha256_bytes(&rust_state),
            },
            "oracle_state": {
                "artifact": "oracle.state",
                "sha256": parity::evidence::sha256_bytes(oracle_state),
            },
            "original_timing_resume_checkpoint": {
                "artifact": "original-timing.resume.json",
                "sha256": parity::evidence::sha256_bytes(original_timing),
            },
            "semantic_trace_checkpoint": {
                "artifact": "semantic-trace.checkpoint.json",
                "sha256": parity::evidence::sha256_bytes(semantic_trace),
            },
            "core": {"sha256": "core-sha"},
            "rom": {"sha256": "rom-sha"},
            "input_script": null,
            "rom_random_script": null,
            "initial_sram": {
                "artifact": "initial.srm",
                "sha256": parity::evidence::sha256_bytes(initial_sram),
            },
        });
        fs::write(
            checkpoint_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        checkpoint_dir
    }

    #[test]
    fn matched_cached_av_replay_writes_a_quiescent_paired_frontier() {
        let root = std::env::temp_dir().join(format!(
            "zelda3-cached-av-paired-frontier-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let cache = root.join("cache");
        let output = root.join("output");
        fs::create_dir_all(&cache).unwrap();
        fs::create_dir_all(&output).unwrap();
        let oracle = b"canonical final oracle state";
        let semantic_trace = b"canonical semantic trace checkpoint";
        fs::write(cache.join("oracle_final.state"), oracle).unwrap();
        fs::write(
            cache.join("semantic-trace-final.checkpoint.json"),
            semantic_trace,
        )
        .unwrap();
        let initial_sram = b"canonical startup SRAM";
        let initial_sram_sha256 = parity::evidence::sha256_bytes(initial_sram);
        fs::write(cache.join("initial.srm"), initial_sram).unwrap();
        let manifest = serde_json::json!({
            "cache_key": "fixture-key",
            "cache_identity": {
                "core_sha256": "core-sha",
                "source_artifact_sha256": {
                    "input.txt": "input-sha",
                    "rom-random.txt": "rng-sha",
                    "initial.srm": initial_sram_sha256.clone(),
                },
            },
            "artifact_sha256": {
                "oracle_final.state": parity::evidence::sha256_bytes(oracle),
                "semantic-trace-final.checkpoint.json": parity::evidence::sha256_bytes(semantic_trace),
            },
        });
        let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
        let game = super::load_default_play_state();

        let frontier = write_cached_av_final_paired_resume(
            &cache,
            &output,
            &manifest,
            &manifest_bytes,
            Path::new("zelda3.sfc"),
            "rom-sha",
            52_000,
            &game,
        )
        .unwrap();
        let (rust_state, oracle_state, original_timing_resume, semantic_trace_checkpoint) =
            paired_resume_paths(&frontier).unwrap();
        let checkpoint: crate::PlayCrashCheckpoint =
            bincode::deserialize(&fs::read(rust_state).unwrap()).unwrap();
        assert_eq!(checkpoint.host_frame, 52_000);
        assert_eq!(checkpoint.run_what, super::select_run_what(&game.ram));
        assert_eq!(fs::read(oracle_state).unwrap(), oracle);
        assert_eq!(fs::read(semantic_trace_checkpoint).unwrap(), semantic_trace);
        let original_timing: zelda3::OriginalTimingResumeCheckpoint =
            serde_json::from_slice(&fs::read(original_timing_resume).unwrap()).unwrap();
        assert_eq!(original_timing.schema(), 2);
        let paired_manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(frontier.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(paired_manifest["schema"], PAIRED_RESUME_SCHEMA);
        assert_eq!(paired_manifest["boundary"], "pre-frame");
        assert_eq!(paired_manifest["frame"], 52_000);
        assert_eq!(paired_manifest["rust_state"]["artifact"], "rust.z3state");
        assert_eq!(paired_manifest["oracle_state"]["artifact"], "oracle.state");
        assert!(paired_manifest["rust_state"]["sha256"].is_string());
        assert_eq!(
            paired_manifest["oracle_state"]["sha256"],
            parity::evidence::sha256_bytes(oracle)
        );
        assert_eq!(paired_manifest["source"]["cache_key"], "fixture-key");
        assert_eq!(paired_manifest["core"]["sha256"], "core-sha");
        assert_eq!(paired_manifest["input_script"]["sha256"], "input-sha");
        assert_eq!(paired_manifest["rom_random_script"]["sha256"], "rng-sha");
        assert_eq!(
            paired_manifest["initial_sram"]["sha256"],
            initial_sram_sha256
        );
        assert_eq!(
            fs::read(frontier.join("initial.srm")).unwrap(),
            b"canonical startup SRAM"
        );
        assert!(!output.join(".paired-final.tmp").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cached_oracle_checkpoint_is_bound_to_nested_artifacts_and_route_sources() {
        let root = std::env::temp_dir().join(format!(
            "zelda3-cached-oracle-checkpoint-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let checkpoint = root.join("oracle-checkpoints/frame-00002500");
        fs::create_dir_all(&checkpoint).unwrap();
        let oracle = b"oracle at frame 2500";
        let semantic = b"semantic trace at frame 2500";
        fs::write(checkpoint.join("oracle.state"), oracle).unwrap();
        fs::write(checkpoint.join("semantic-trace.checkpoint.json"), semantic).unwrap();
        let checkpoint_manifest = serde_json::json!({
            "schema": 1,
            "boundary": "pre-frame",
            "frame": 2500,
            "oracle_state": {
                "artifact": "oracle.state",
                "sha256": parity::evidence::sha256_bytes(oracle),
            },
            "semantic_trace_checkpoint": {
                "artifact": "semantic-trace.checkpoint.json",
                "sha256": parity::evidence::sha256_bytes(semantic),
            },
            "provenance": {
                "core_sha256": "core",
                "rom_sha256": "rom",
                "input_sha256": "input",
                "rom_random_sha256": "rng",
                "initial_sram_sha256": "sram",
            },
        });
        fs::write(
            checkpoint.join("manifest.json"),
            serde_json::to_vec_pretty(&checkpoint_manifest).unwrap(),
        )
        .unwrap();
        let cache_manifest = serde_json::json!({
            "cache_identity": {
                "core_sha256": "core",
                "rom_sha256": "rom",
                "source_artifact_sha256": {
                    "input.txt": "input",
                    "rom-random.txt": "rng",
                    "initial.srm": "sram",
                },
            },
            "artifact_sha256": {
                "oracle-checkpoints/frame-00002500/oracle.state": parity::evidence::sha256_bytes(oracle),
                "oracle-checkpoints/frame-00002500/semantic-trace.checkpoint.json": parity::evidence::sha256_bytes(semantic),
            },
        });

        let (actual_oracle, actual_semantic) =
            cached_oracle_checkpoint_sources(&root, &cache_manifest, 2500).unwrap();
        assert_eq!(actual_oracle, checkpoint.join("oracle.state"));
        assert_eq!(
            actual_semantic,
            checkpoint.join("semantic-trace.checkpoint.json")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn animated_bg_receipt_decodes_complete_post_nmi_vram_without_cache_validity() {
        let mut vram = vec![0; 0x10000];
        let base = 0x3b00 * 2;
        vram[base] = 0x80;
        vram[base + 1] = 0x40;
        vram[base + 16] = 0x20;
        vram[base + 17] = 0x10;

        let actual = super::decode_presented_animated_bg_tiles(
            zelda3::PresentedAnimatedBgDestination::Dungeon,
            &vram,
        )
        .unwrap();
        let mut pixels = vec![0; zelda3::PresentedAnimatedBgTiles::TILE_COUNT * 64];
        pixels[..4].copy_from_slice(&[1, 2, 4, 8]);
        let expected = zelda3::PresentedAnimatedBgTiles::new(
            zelda3::PresentedAnimatedBgDestination::Dungeon,
            pixels,
        )
        .unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn animated_bg_receipt_rejects_incomplete_post_nmi_vram() {
        let error = super::decode_presented_animated_bg_tiles(
            zelda3::PresentedAnimatedBgDestination::Overworld,
            &[0; 0x7800],
        )
        .unwrap_err();
        assert!(error.contains("omits the animated-BG publication"));
    }

    #[test]
    fn animated_bg_destination_is_absent_until_source_graphics_setup() {
        for value in [None, Some(-1), Some(0), Some(0x5555)] {
            assert_eq!(
                super::decode_presented_animated_bg_destination(value).unwrap(),
                None,
            );
        }
        assert_eq!(
            super::decode_presented_animated_bg_destination(Some(0x3b00)).unwrap(),
            Some(zelda3::PresentedAnimatedBgDestination::Dungeon),
        );
        assert_eq!(
            super::decode_presented_animated_bg_destination(Some(0x3c00)).unwrap(),
            Some(zelda3::PresentedAnimatedBgDestination::Overworld),
        );
        assert!(super::decode_presented_animated_bg_destination(Some(1)).is_err());
    }

    #[test]
    fn semantic_receipts_require_a_loaded_generic_trace_authority() {
        assert!(semantic_trace_authority_available(true, true));
        assert!(!semantic_trace_authority_available(true, false));
        assert!(!semantic_trace_authority_available(false, true));
        assert!(!semantic_trace_authority_available(false, false));
    }

    fn decode_base64_bytes(encoded: &str) -> Vec<u8> {
        assert_eq!(encoded.len() % 4, 0);
        let value = |byte: u8| match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => 0,
            _ => panic!("invalid fixture base64 digit"),
        };
        let mut decoded = Vec::with_capacity(encoded.len() / 4 * 3);
        for chunk in encoded.as_bytes().chunks_exact(4) {
            let bits = u32::from(value(chunk[0])) << 18
                | u32::from(value(chunk[1])) << 12
                | u32::from(value(chunk[2])) << 6
                | u32::from(value(chunk[3]));
            decoded.push((bits >> 16) as u8);
            if chunk[2] != b'=' {
                decoded.push((bits >> 8) as u8);
            }
            if chunk[3] != b'=' {
                decoded.push(bits as u8);
            }
        }
        decoded
    }

    fn read_unsigned_varint(bytes: &[u8], index: &mut usize) -> u64 {
        let mut value = 0u64;
        let mut shift = 0;
        loop {
            let byte = bytes[*index];
            *index += 1;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
            assert!(shift < 64);
        }
        value
    }

    pub(crate) fn expand_delta_sequence(sequence: &serde_json::Value) -> Vec<Vec<i64>> {
        let encoding = sequence["encoding"].as_str().unwrap();
        assert!(matches!(
            encoding,
            "columnar-signed-delta-zero-rle-varint-base64-v1"
                | "columnar-signed-delta-zero-rle-varint-zstd-base64-v1"
        ));
        let field_count = sequence["fields"].as_array().unwrap().len();
        let record_count = sequence["record_count"].as_u64().unwrap() as usize;
        let mut bytes = decode_base64_bytes(sequence["data_base64"].as_str().unwrap());
        if encoding == "columnar-signed-delta-zero-rle-varint-zstd-base64-v1" {
            bytes = zstd::stream::decode_all(bytes.as_slice()).unwrap();
        }
        let mut columns = Vec::with_capacity(field_count);
        let mut byte_index = 0;
        for _ in 0..field_count {
            let column_length = read_unsigned_varint(&bytes, &mut byte_index) as usize;
            let column_end = byte_index + column_length;
            let mut column = Vec::with_capacity(record_count);
            let mut previous = 0i64;
            while byte_index < column_end {
                let code = read_unsigned_varint(&bytes, &mut byte_index);
                if code == 0 {
                    let run_length = read_unsigned_varint(&bytes, &mut byte_index) as usize;
                    column.extend(std::iter::repeat_n(previous, run_length));
                } else {
                    let zigzag = code - 1;
                    let delta = ((zigzag >> 1) as i64) ^ -((zigzag & 1) as i64);
                    previous += delta;
                    column.push(previous);
                }
            }
            assert_eq!(byte_index, column_end);
            assert_eq!(column.len(), record_count);
            columns.push(column);
        }
        assert_eq!(byte_index, bytes.len());
        let mut expanded_bytes = Vec::new();
        let rows = (0..record_count)
            .map(|row| {
                (0..field_count)
                    .map(|field| {
                        let value = columns[field][row];
                        expanded_bytes.extend_from_slice(&value.to_le_bytes());
                        value
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            sequence["expanded_sha256"],
            parity::evidence::sha256_bytes(&expanded_bytes)
        );
        rows
    }

    #[test]
    fn bootstrap_delta_sequence_round_trips_signed_field_changes() {
        let sequence = compact_delta_integer_sequence(
            ["a", "b", "c"],
            [[0, -3, 130], [1, -3, 129], [1, 400, -2]],
        );
        let value = serde_json::to_value(sequence).unwrap();
        assert_eq!(
            expand_delta_sequence(&value),
            vec![vec![0, -3, 130], vec![1, -3, 129], vec![1, 400, -2]]
        );
    }

    #[test]
    fn presented_video_history_receipt_requires_byte_exact_consecutive_rows() {
        let previous_frame = LibretroFrame {
            audio: Vec::new(),
            video: vec![1, 2, 3, 4, 5, 6, 7, 8],
            video_width: 2,
            video_height: 2,
            video_pitch: 4,
            pixel_format: 2,
        };
        let previous = PresentedOracleVideo::from(&previous_frame);
        let current = LibretroFrame {
            audio: Vec::new(),
            video: vec![1, 2, 3, 4, 5, 6, 7, 9],
            video_width: 2,
            video_height: 2,
            video_pitch: 4,
            pixel_format: 2,
        };

        assert!(
            presented_video_rows_match_prior_surface(&current, Some(&previous), 0, 1,).unwrap()
        );
        assert!(
            !presented_video_rows_match_prior_surface(&current, Some(&previous), 0, 2,).unwrap()
        );
        assert!(!presented_video_rows_match_prior_surface(&current, None, 0, 1).unwrap());
    }

    #[test]
    fn first_nmi_return_selector_requires_exact_committed_successor() {
        let mut transaction = LibretroCpuTimingTransaction {
            kind: 0,
            duration: 8,
            origin_pc: 0x008a38,
            opcode: 0x8c,
            start_v_counter: 227,
            start_cpu_cycle: 742,
            end_v_counter: 227,
            end_cpu_cycle: 750,
            cpu_model_identity: 1,
            cpu_model_5a22: 2,
            start_wram_refresh_position: 534,
            end_wram_refresh_position: 534,
        };
        assert_eq!(
            first_nmi_return_start_index(&[transaction]).unwrap(),
            Some(0)
        );
        transaction.start_cpu_cycle = 740;
        assert!(first_nmi_return_start_index(&[transaction])
            .unwrap_err()
            .contains("committed V227:H742->H750"));
    }

    #[test]
    fn first_nmi_return_core_trace_requires_direct_entry_return_and_paired_hdma() {
        let path = std::env::temp_dir().join(format!(
            "zelda3-first-nmi-return-trace-{}-{}.jsonl",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(
            &path,
            concat!(
                "{\"event\":\"frame\",\"stage\":\"entry\",\"run\":81,\"v\":225,\"cycles\":6,\"pc\":32822}\n",
                "{\"event\":\"video\",\"stage\":\"presented\",\"run\":81,\"v\":225,\"cycles\":16,\"pc\":836060}\n",
                "{\"event\":\"hdma\",\"stage\":\"start\",\"run\":81,\"v\":0,\"cycles\":1112,\"pc\":34000}\n",
                "{\"event\":\"hdma\",\"stage\":\"end\",\"run\":81,\"v\":0,\"cycles\":1140,\"pc\":34000}\n",
                "{\"event\":\"frame\",\"stage\":\"return\",\"run\":81,\"v\":225,\"cycles\":94,\"pc\":32969}\n",
            ),
        )
        .unwrap();
        let trace = read_snes9x_retro_run_trace(&path, 81).unwrap().unwrap();
        let transaction = |start_v, start_h, end_v, end_h| LibretroCpuTimingTransaction {
            kind: 1,
            duration: 8,
            origin_pc: 0x008a38,
            opcode: 0x8c,
            start_v_counter: start_v,
            start_cpu_cycle: start_h,
            end_v_counter: end_v,
            end_cpu_cycle: end_h,
            cpu_model_identity: 1,
            cpu_model_5a22: 2,
            start_wram_refresh_position: 534,
            end_wram_refresh_position: 534,
        };
        let transactions = [
            transaction(227, 742, 227, 750),
            transaction(227, 750, 227, 1162),
            // `$008a59 STA $420b`: the memory operand receipt ends before
            // source-owned DMA/event processing, and the semantic receipt
            // resumes on the following scanline. The CPU timing ledger is
            // ordered evidence, not an exhaustive partition of raster time.
            transaction(228, 1160, 261, 1360),
            transaction(261, 1360, 0, 4),
            transaction(0, 4, 225, 94),
        ];
        let gaps = validate_first_nmi_return_cpu_slice(&trace, &transactions).unwrap();
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].previous_transaction_ordinal, 1);
        assert_eq!(gaps[0].next_transaction_ordinal, 2);
        assert_eq!(gaps[0].previous_end_v_counter, 227);
        assert_eq!(gaps[0].previous_end_cpu_cycle, 1162);
        assert_eq!(gaps[0].next_start_v_counter, 228);
        assert_eq!(gaps[0].next_start_cpu_cycle, 1160);
        assert_eq!(gaps[0].elapsed_master_cycles, 1362);
        assert_eq!(trace.hdma_events.len(), 2);
        assert_eq!(trace.video_events.len(), 1);
        assert_eq!(trace.return_event["cycles"], 94);
        assert_eq!(
            trace.raw_sha256,
            parity::evidence::sha256_file(&path).unwrap()
        );
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn first_nmi_return_fixture_is_installed_only_after_explicit_success() {
        let path = std::env::temp_dir().join(format!(
            "zelda3-first-nmi-return-fixture-{}-{}.jsonl",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut pending = PendingFirstNmiReturnFixture::create(&path).unwrap();
        pending
            .writer
            .write_all(b"{\"kind\":\"synthetic\"}\n")
            .unwrap();
        assert!(!path.exists());
        assert!(pending.temporary_path.exists());
        pending.install().unwrap();
        assert!(pending.installed);
        assert!(!pending.temporary_path.exists());
        assert_eq!(fs::read(&path).unwrap(), b"{\"kind\":\"synthetic\"}\n");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn continuation_apui_compaction_preserves_all_four_dsp_fields() {
        let access = OrdinalApuPortAccess {
            cpu_transaction_ordinal: 7,
            access: LibretroApuPortWrite {
                port: 2,
                value: 0x55,
                output_sample: 3,
                v_counter: 4,
                cpu_cycle: 5,
                program_counter: 0x008123,
                apu_cycle_before: 6,
                apu_cycle_after: 7,
                smp_clock_before: 8,
                smp_clock_after: 9,
                dsp_clock_before: 10,
                dsp_clock_after: 11,
                dsp_phase_before: 12,
                dsp_phase_after: 13,
                smp_pc_before: 14,
                smp_pc_after: 15,
                smp_opcode_before: 16,
                smp_opcode_after: 17,
                smp_opcode_cycle_before: 18,
                smp_opcode_cycle_after: 19,
                is_read: true,
                cpu_model_5a22: 2,
                wram_refresh_position: 534,
                cpu_model_identity: 1,
            },
        };
        let sequence = serde_json::to_value(compact_ordinal_cpu_apu_accesses(&[access])).unwrap();
        assert_eq!(
            sequence["fields"],
            serde_json::json!([
                "cpu_transaction_ordinal",
                "port",
                "value",
                "output_sample",
                "v_counter",
                "cpu_cycle",
                "program_counter",
                "apu_cycle_before",
                "apu_cycle_after",
                "smp_clock_before",
                "smp_clock_after",
                "dsp_clock_before",
                "dsp_clock_after",
                "dsp_phase_before",
                "dsp_phase_after",
                "smp_pc_before",
                "smp_pc_after",
                "smp_opcode_before",
                "smp_opcode_after",
                "smp_opcode_cycle_before",
                "smp_opcode_cycle_after",
                "is_read",
                "cpu_model_5a22",
                "wram_refresh_position",
                "cpu_model_identity"
            ])
        );
        let rows = expand_delta_sequence(&sequence);
        assert_eq!(&rows[0][11..15], &[10, 11, 12, 13]);
    }

    #[test]
    fn first_nmi_dma_setup_slice_starts_after_apui_completion_and_excludes_dma_start() {
        let apui = FramedApuPortAccess {
            frame: 81,
            access: LibretroApuPortWrite {
                port: 0,
                value: 0x8b,
                output_sample: 0,
                v_counter: 225,
                cpu_cycle: 480,
                program_counter: 0x0080e4,
                apu_cycle_before: 19,
                apu_cycle_after: 43,
                smp_clock_before: 0,
                smp_clock_after: 1,
                dsp_clock_before: 12,
                dsp_clock_after: 13,
                dsp_phase_before: 14,
                dsp_phase_after: 15,
                smp_pc_before: 0x0873,
                smp_pc_after: 0x0873,
                smp_opcode_before: 0xf0,
                smp_opcode_after: 0xf0,
                smp_opcode_cycle_before: 0,
                smp_opcode_cycle_after: 0,
                is_read: true,
                cpu_model_5a22: 2,
                wram_refresh_position: 534,
                cpu_model_identity: 1,
            },
        };
        let transaction = |kind, origin_pc, opcode, start_cpu_cycle| FramedCpuTimingTransaction {
            frame: 81,
            transaction: LibretroCpuTimingTransaction {
                kind,
                duration: if matches!(kind, 0 | 1) { 8 } else { 6 },
                origin_pc,
                opcode,
                start_v_counter: 225,
                start_cpu_cycle,
                end_v_counter: 225,
                end_cpu_cycle: start_cpu_cycle + if matches!(kind, 0 | 1) { 8 } else { 6 },
                cpu_model_identity: 1,
                cpu_model_5a22: 2,
                start_wram_refresh_position: 534,
                end_wram_refresh_position: 534,
            },
        };
        let transactions = vec![
            transaction(2, 0x0080e1, 0xad, 480),
            transaction(0, 0x008a33, 0xa9, 486),
            transaction(1, 0x008a33, 0xa9, 494),
            transaction(0, 0x008a35, 0x8d, 500),
            transaction(2, 0x008a35, 0x8d, 508),
        ];

        assert_eq!(
            first_nmi_apui_anchor_indices(&[apui], &transactions).unwrap(),
            Some((0, 0))
        );
        let post_anchor = &transactions[1..];
        let stop = first_nmi_dma_setup_stop_index(post_anchor)
            .unwrap()
            .unwrap();
        assert_eq!(stop, 2);
        assert_eq!(post_anchor[..stop].last().unwrap().transaction.kind, 1);
        assert_eq!(
            post_anchor[..stop].last().unwrap().transaction.origin_pc,
            0x008a33
        );
        assert!(post_anchor[..stop]
            .iter()
            .all(|framed| { (framed.transaction.origin_pc & 0x00ff_ffff) != 0x008a35 }));
        assert_eq!(post_anchor[stop].transaction.kind, 0);
    }

    #[test]
    fn first_nmi_dma_setup_requires_the_route_initial_sram_loaded_into_snes9x() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../routes/full_run/comparisons/continuous-audio/initial.srm");
        let sram = fs::read(&path).unwrap();
        let provenance = first_nmi_dma_setup_initial_sram_provenance(Some(&path), &sram).unwrap();
        assert_eq!(
            provenance["sha256"],
            "d8a02e6e08a22377f919c7350e5bfa6117c6408db9a728815af7ad6e4e6b83bc"
        );
        assert_eq!(provenance["bytes"], 8_192);
        assert_eq!(provenance["valid_slot_marker"]["offset"], 0x03e5);
        assert_eq!(
            provenance["valid_slot_marker"]["bytes"],
            serde_json::json!([0xaa, 0x55])
        );

        let missing = first_nmi_dma_setup_initial_sram_provenance(None, &sram)
            .unwrap_err()
            .to_string();
        assert!(missing.contains("requires --load-sram"));

        let mut different_loaded_sram = sram.clone();
        different_loaded_sram[0] ^= 1;
        let mismatch =
            first_nmi_dma_setup_initial_sram_provenance(Some(&path), &different_loaded_sram)
                .unwrap_err()
                .to_string();
        assert!(mismatch.contains("does not match the bytes loaded into Snes9x"));
    }

    #[test]
    fn bootstrap_zstd_delta_sequence_round_trips_signed_field_changes() {
        let sequence = compact_delta_integer_sequence_with_zstd(
            ["a", "b", "c"],
            [[0, -3, 130], [1, -3, 129], [1, 400, -2]],
            true,
        );
        let value = serde_json::to_value(sequence).unwrap();
        assert_eq!(
            expand_delta_sequence(&value),
            vec![vec![0, -3, 130], vec![1, -3, 129], vec![1, 400, -2]]
        );
    }

    fn bootstrap_instruction(
        absolute_cycle: u64,
        program_counter: i32,
        opcode: i32,
        op_step_calls: i32,
    ) -> LibretroSmpInstruction {
        LibretroSmpInstruction {
            absolute_cycle,
            program_counter,
            opcode,
            a: 0,
            x: 0,
            y: 0,
            stack_pointer: 0xef,
            status: 2,
            timer0_stage1: 0,
            timer0_stage2: 0,
            timer0_stage3: 0,
            output_sample: 0,
            dsp_phase: 0,
            smp_clock: 0,
            direct_page_0_11: [0; 12],
            boundary_opcode_cycle: 0,
            op_step_calls,
            max_continuation_opcode_cycle: op_step_calls - 1,
        }
    }

    #[test]
    fn framed_smp_boundary_sequence_round_trips_all_recorded_fields() {
        let mut first = bootstrap_instruction(1_000, 0x0800, 0x20, 2);
        first.a = 0x11;
        first.x = 0x22;
        first.y = 0x33;
        first.stack_pointer = 0xcc;
        first.status = 0x81;
        first.timer0_stage1 = 4;
        first.timer0_stage2 = 5;
        first.timer0_stage3 = 6;
        first.output_sample = 7;
        first.dsp_phase = 8;
        first.smp_clock = -9;
        first.direct_page_0_11 = [10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21];
        first.boundary_opcode_cycle = 1;
        first.max_continuation_opcode_cycle = 3;

        let mut second = bootstrap_instruction(1_006, 0x0801, 0xcd, 1);
        second.a = -1;
        second.direct_page_0_11[11] = 0x7f;
        let framed = [
            FramedSmpInstruction {
                frame: 79,
                instruction: first,
            },
            FramedSmpInstruction {
                frame: 80,
                instruction: second,
            },
        ];

        let sequence = serde_json::to_value(compact_framed_smp_instructions(&framed)).unwrap();
        assert_eq!(sequence["fields"].as_array().unwrap().len(), 30);
        assert_eq!(
            expand_delta_sequence(&sequence),
            vec![
                vec![
                    79, 1_000, 0x0800, 0x20, 0x11, 0x22, 0x33, 0xcc, 0x81, 4, 5, 6, 7, 8, -9, 10,
                    11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3,
                ],
                vec![
                    80, 1_006, 0x0801, 0xcd, -1, 0, 0, 0xef, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0x7f, 0, 1, 0,
                ],
            ]
        );
    }

    #[test]
    fn bootstrap_instruction_frames_replace_straddling_pseudo_op_record() {
        let mut accumulated = vec![
            bootstrap_instruction(0, 0xfff9, 0xd0, 1),
            bootstrap_instruction(4, 0xfffb, 0x1f, 1),
        ];
        append_smp_instruction_frame(
            &mut accumulated,
            vec![
                bootstrap_instruction(4, 0xfffb, 0x1f, 2),
                bootstrap_instruction(10, 0x0800, 0xcd, 1),
            ],
        );

        assert_eq!(accumulated.len(), 3);
        assert_eq!(accumulated[1].op_step_calls, 2);
        assert_eq!(smp_bootstrap_handoff_index(&accumulated), Some(1));
    }

    #[test]
    fn pinned_snes9x_cold_apu_bootstrap_fixture_reaches_final_ipl_handoff() {
        const FIXTURE: &str =
            include_str!("../../external/snes9x-libretro/fixtures/zelda3-cold-apu-bootstrap.jsonl");
        assert!(
            FIXTURE.len() < 9_000_000,
            "bootstrap fixture lost compact encoding"
        );
        let records = FIXTURE
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 3);

        let provenance = &records[0];
        assert_eq!(provenance["kind"], "provenance");
        assert_eq!(provenance["schema"], 1);
        assert_eq!(provenance["core"]["library_name"], "Snes9x");
        assert_eq!(provenance["core"]["library_version"], "1.63 921f9f7b");
        assert_eq!(
            provenance["core"]["sha256"],
            "7d4fa577dd2e0a79ace97ebec2630929ffcd6622d46ae5b23c4700514c7169cb"
        );
        assert_eq!(
            provenance["source"]["revision"],
            "921f9f7b83660eb44ad263022a57a4a029057c37"
        );
        assert_eq!(
            provenance["rom"]["sha256"],
            "66871d66be19ad2c34c927d6b14cd8eb6fc3181965b6e517cb361f7316009cfb"
        );
        let trace_patch = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../external/snes9x-libretro/patches/zelda3-trace.patch");
        assert_eq!(
            provenance["source"]["trace_patch_sha256"],
            parity::evidence::sha256_file(&trace_patch).unwrap()
        );
        assert_eq!(provenance["cpu_to_smp_ratio"]["numerator"], 15_664);
        assert_eq!(provenance["cpu_to_smp_ratio"]["denominator"], 328_125);

        let reset = &records[1];
        assert_eq!(reset["kind"], "reset-state");
        assert_eq!(reset["absolute_cycle"], 0);
        assert_eq!(reset["pc"], 0xffc0);
        assert_eq!(reset["sp"], 0xef);
        assert_eq!(reset["status"], 0x02);
        assert_eq!(reset["ipl_rom_enabled"], true);
        assert_eq!(reset["ram_nonzero_bytes"], 0);
        assert_eq!(reset["output_ports"], serde_json::json!([0, 0, 0, 0]));
        // globals.cpp selects the M1SNES object, whose _5A22 field is 2.
        // cpu.cpp therefore uses the v2 refresh schedule; the object name is
        // not evidence that WRAM refresh occurs at the v1 position.
        assert_eq!(reset["cpu_model_identity"], 1);
        assert_eq!(reset["cpu_model_5a22"], 2);
        assert_eq!(reset["wram_refresh_position"], 538);

        let events = &records[2];
        assert_eq!(events["kind"], "bootstrap-events");
        assert_eq!(events["first_cc_acknowledged"], true);
        assert_eq!(events["final_frame"], 79);
        assert_eq!(events["final_ipl_handoff"]["absolute_cycle"], 1_355_567);
        assert_eq!(events["final_ipl_handoff"]["origin_pc"], 0xfffb);
        assert_eq!(events["final_ipl_handoff"]["opcode"], 0x1f);
        assert_eq!(events["final_ipl_handoff"]["target_pc"], 0x0800);

        let cpu = expand_delta_sequence(&events["cpu_apu_access_sequence"]);
        assert_eq!(cpu.len(), 356_174);
        assert_eq!(
            cpu[0],
            vec![0, 0, 0, 0, 0, 326, 0x800d, 0, 16, 0, 1, 0xffc0, 0xffc5, 0, 0xd0, 0, 2, 538, 1,]
        );
        assert!(cpu.iter().all(|access| access[16] == 2));
        assert!(cpu.iter().all(|access| matches!(access[17], 534 | 538)));
        assert!(cpu.iter().all(|access| access[18] == 1));

        let before_refresh_pair = cpu
            .windows(2)
            .find(|pair| {
                pair[0][0] == 0
                    && pair[0][4] == 44
                    && pair[0][5] == 528
                    && pair[1][4] == 44
                    && pair[1][5] == 534
            })
            .expect("fixture lost the H528/H534 APUI pair");
        // Both semantics precede this scanline's live H538 refresh event, so
        // their six-master-cycle separation is not evidence of an H530 model.
        for access in before_refresh_pair {
            assert_eq!(access[16], 2);
            assert_eq!(access[17], 538);
            assert_eq!(access[18], 1);
        }
        let initial_cc = cpu
            .iter()
            .position(|access| {
                access[1] == 0 && access[2] == 0xcc && access[6] == 0x88ef && access[15] == 1
            })
            .unwrap();
        assert_eq!(cpu[initial_cc][0], 0);
        assert_eq!(cpu[initial_cc][4], 37);
        assert_eq!(cpu[initial_cc][5], 1_102);
        assert_eq!(cpu[initial_cc][8], 2_461);
        assert_eq!(
            cpu.last().unwrap(),
            &vec![
                79, 3, 0, 319, 120, 384, 0x88ff, 10_255, 10_255, 4, 3, 0x0800, 0x0800, 0x1f, 0x1f,
                0, 2, 534, 1,
            ]
        );

        assert_eq!(
            events["cpu_timing_transaction_kinds"],
            serde_json::json!({
                "0": "fast_pcbase_opcode_fetch_non_draining",
                "1": "cpuops_add_cycles_draining",
                "2": "getset_memory_access_after_semantic_draining",
                "3": "getset_memory_access_x2_after_semantic_draining",
            })
        );
        let cpu_timing = expand_delta_sequence(&events["cpu_timing_transaction_sequence"]);
        assert_eq!(cpu_timing.len(), 3_213_852);
        assert_eq!(
            &cpu_timing[..5],
            &[
                vec![0, 0, 8, 0x8000, 0x78, 0, 198, 0, 206, 1, 2, 538, 538],
                vec![0, 1, 6, 0x8000, 0x78, 0, 206, 0, 212, 1, 2, 538, 538],
                vec![0, 0, 8, 0x8001, 0x9c, 0, 212, 0, 220, 1, 2, 538, 538],
                vec![0, 1, 16, 0x8001, 0x9c, 0, 220, 0, 236, 1, 2, 538, 538],
                vec![0, 2, 6, 0x8001, 0x9c, 0, 236, 0, 242, 1, 2, 538, 538],
            ]
        );
        assert_eq!(
            [13, 16, 19, 22].map(|index| cpu_timing[index].clone()),
            [
                vec![0, 2, 6, 0x800a, 0x9c, 0, 326, 0, 332, 1, 2, 538, 538],
                vec![0, 2, 6, 0x800d, 0x9c, 0, 356, 0, 362, 1, 2, 538, 538],
                vec![0, 2, 6, 0x8010, 0x9c, 0, 386, 0, 392, 1, 2, 538, 538],
                vec![0, 2, 6, 0x8013, 0x9c, 0, 416, 0, 422, 1, 2, 538, 538],
            ]
        );
        assert!(cpu_timing.iter().all(|transaction| transaction[9] == 1));
        assert!(cpu_timing.iter().all(|transaction| transaction[10] == 2));
        assert!(cpu_timing
            .iter()
            .all(|transaction| matches!(transaction[11], 534 | 538)));
        assert!(cpu_timing
            .iter()
            .all(|transaction| matches!(transaction[12], 534 | 538)));
        assert_eq!(
            cpu_timing
                .iter()
                .fold([0usize; 4], |mut counts, transaction| {
                    counts[transaction[1] as usize] += 1;
                    counts
                }),
            [1_143_074, 1_552_419, 464_286, 54_073]
        );
        assert!(cpu_timing.iter().any(|transaction| {
            transaction == &vec![0, 1, 8, 0x8894, 0xd0, 0, 1_366, 1, 10, 1, 2, 538, 534]
        }));
        assert_eq!(
            cpu_timing.last().unwrap(),
            &vec![79, 2, 6, 0x88fc, 0x9c, 120, 384, 120, 390, 1, 2, 534, 534]
        );

        let output = expand_delta_sequence(&events["smp_output_port_write_sequence"]);
        assert_eq!(output.len(), 54_043);
        assert_eq!(
            &output[..3],
            &[
                vec![
                    2_399, 0, 0xaa, 0xffc9, 0x8f, 3, 36, 1_184, 0x8894, 1_132, 52_954, -1, 0xffcc,
                    53, 10, 73
                ],
                vec![
                    2_404, 1, 0xbb, 0xffcc, 0x8f, 3, 36, 1_300, 0x8894, 1_248, 229_353, -2, 0xffcf,
                    58, 10, 73
                ],
                vec![
                    2_461, 0, 0xcc, 0xfff5, 0xc4, 3, 37, 1_102, 0x88ef, 1_050, 118_577, 0, 0xfff7,
                    52, 9, 75
                ],
            ]
        );
        assert_eq!(output.last().unwrap()[0], 1_355_555);
        assert_eq!(output.last().unwrap()[2], 0x8b);
        assert_eq!(output.last().unwrap()[3], 0xfff5);

        let sequence = &events["smp_instruction_boundary_sequence"];
        assert_eq!(sequence["encoding"], "repeated-span-v1");
        assert_eq!(sequence["instruction_count"], 379_743);
        assert_eq!(sequence["absolute_start_cycle"], 0);
        assert_eq!(sequence["absolute_end_cycle"], 1_355_567);
        assert_eq!(sequence["spans"].as_array().unwrap().len(), 437);
        let mut expanded = Vec::new();
        let mut expected_cycle = 0;
        for span in sequence["spans"].as_array().unwrap() {
            assert_eq!(span["absolute_start_cycle"].as_u64(), Some(expected_cycle));
            let span_start = span["absolute_start_cycle"].as_u64().unwrap();
            let stride = span["repeat_cycle_stride"].as_u64().unwrap();
            let repeat_count = span["repeat_count"].as_u64().unwrap();
            for repeat in 0..repeat_count {
                let repeat_start = span_start + repeat * stride;
                for instruction in span["instructions"].as_array().unwrap() {
                    let start = repeat_start + instruction["start_cycle_offset"].as_u64().unwrap();
                    let end = repeat_start + instruction["end_cycle_offset"].as_u64().unwrap();
                    assert_eq!(start, expected_cycle);
                    assert!(end > start);
                    let calls = instruction["op_step_calls"].as_u64().unwrap();
                    let continuation = instruction["max_continuation_opcode_cycle"]
                        .as_u64()
                        .unwrap();
                    assert_eq!(continuation, calls - 1);
                    expanded.push((
                        start,
                        end,
                        instruction["origin_pc"].as_u64().unwrap(),
                        instruction["opcode"].as_u64().unwrap(),
                        calls,
                        continuation,
                    ));
                    expected_cycle = end;
                }
            }
            assert_eq!(span["absolute_end_cycle"].as_u64(), Some(expected_cycle));
        }
        assert_eq!(expanded.len(), 379_743);
        assert_eq!(expanded[0], (0, 2, 0xffc0, 0xcd, 1, 0));
        assert_eq!(
            expanded.iter().find(|step| step.0 == 2_461).copied(),
            Some((2_461, 2_463, 0xfff7, 0xdd, 1, 0))
        );
        assert!(expanded
            .iter()
            .any(|step| step.2 == 0xffe7 && step.3 == 0xab));
        assert_eq!(
            expanded.last().copied(),
            Some((1_355_561, 1_355_567, 0xfffb, 0x1f, 1, 0))
        );
    }

    #[test]
    fn pinned_snes9x_post_handoff_fixture_reaches_first_nmi_apui_sync() {
        const FIXTURE: &str =
            include_str!("../../external/snes9x-libretro/fixtures/zelda3-cold-apu-first-nmi.jsonl");
        assert!(
            FIXTURE.len() < 20_000,
            "post-handoff fixture lost compact encoding"
        );
        let records = FIXTURE
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 2);

        let provenance = &records[0];
        assert_eq!(provenance["kind"], "provenance");
        assert_eq!(provenance["schema"], 1);
        assert_eq!(provenance["core"]["library_name"], "Snes9x");
        assert_eq!(provenance["core"]["library_version"], "1.63 921f9f7b");
        assert_eq!(
            provenance["core"]["sha256"],
            "7d4fa577dd2e0a79ace97ebec2630929ffcd6622d46ae5b23c4700514c7169cb"
        );
        assert_eq!(
            provenance["source"]["revision"],
            "921f9f7b83660eb44ad263022a57a4a029057c37"
        );
        assert_eq!(
            provenance["rom"]["sha256"],
            "66871d66be19ad2c34c927d6b14cd8eb6fc3181965b6e517cb361f7316009cfb"
        );
        let trace_patch = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../external/snes9x-libretro/patches/zelda3-trace.patch");
        assert_eq!(
            provenance["source"]["trace_patch_sha256"],
            parity::evidence::sha256_file(&trace_patch).unwrap()
        );
        let prefix_fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../external/snes9x-libretro/fixtures/zelda3-cold-apu-bootstrap.jsonl");
        assert_eq!(
            provenance["prefix_fixture"]["sha256"],
            parity::evidence::sha256_file(&prefix_fixture).unwrap()
        );

        let events = &records[1];
        assert_eq!(events["kind"], "post-handoff-first-nmi");
        assert_eq!(
            events["stop_reason"],
            "first_$8080e1_apui0_read_semantic_and_kind2_timing_complete"
        );
        assert_eq!(
            events["start_anchor"]["final_ipl_handoff"]["absolute_cycle"],
            1_355_567
        );
        assert_eq!(
            events["start_anchor"]["final_cpu_timing_transaction"]["transaction"]["origin_pc"],
            0x88fc
        );
        assert_eq!(
            events["start_anchor"]["final_cpu_timing_transaction"]["transaction"]["kind"],
            2
        );
        assert_eq!(
            events["nmi_enable_source"]["rom_bytes"],
            serde_json::json!([0xa9, 0x81, 0x8d, 0x00, 0x42])
        );

        let cpu_timing = expand_delta_sequence(&events["cpu_timing_transaction_sequence"]);
        assert_eq!(cpu_timing.len(), 55_405);
        assert_eq!(
            cpu_timing.first().unwrap(),
            &vec![79, 0, 8, 0x88ff, 0x28, 120, 390, 120, 398, 1, 2, 534, 534]
        );
        assert_eq!(
            cpu_timing.last().unwrap(),
            &vec![81, 2, 6, 0x80e1, 0xad, 225, 480, 225, 486, 1, 2, 534, 534]
        );
        assert!(cpu_timing.iter().all(|transaction| transaction[9] == 1));
        assert!(cpu_timing.iter().all(|transaction| transaction[10] == 2));
        assert!(cpu_timing
            .iter()
            .all(|transaction| matches!(transaction[11], 534 | 538)));
        assert!(cpu_timing
            .iter()
            .all(|transaction| matches!(transaction[12], 534 | 538)));
        assert_eq!(
            events["first_hmax_crossing_transaction"],
            serde_json::json!({
                "frame": 79,
                "transaction": {
                    "kind": 1,
                    "duration": 16,
                    "origin_pc": 0x87da,
                    "opcode": 0x9d,
                    "start_v_counter": 120,
                    "start_cpu_cycle": 1368,
                    "end_v_counter": 121,
                    "end_cpu_cycle": 20,
                    "cpu_model_identity": 1,
                    "cpu_model_5a22": 2,
                    "start_wram_refresh_position": 534,
                    "end_wram_refresh_position": 538,
                },
            })
        );
        assert_eq!(
            events["nmi_enable_source"]["timing_transaction"]["transaction"]["origin_pc"],
            0x8031
        );
        assert_eq!(
            events["first_nmi_entry_transaction"]["transaction"]["origin_pc"],
            0x80c9
        );

        let apui = expand_delta_sequence(&events["cpu_apu_access_sequence"]);
        assert_eq!(
            apui,
            vec![vec![
                81, 0, 0x8b, 0, 225, 480, 0x80e4, 19, 43, 0, 1, 0x0873, 0x0873, 0xf0, 0xf0, 0, 0,
                1, 2, 534, 1,
            ]]
        );
        let output = expand_delta_sequence(&events["smp_output_port_write_sequence"]);
        assert_eq!(output.len(), 3);
        assert_eq!(
            output.iter().map(|write| &write[..3]).collect::<Vec<_>>(),
            vec![
                &[1_374_569, 1, 0][..],
                &[1_374_676, 2, 0][..],
                &[1_374_783, 3, 0][..]
            ]
        );

        let boundaries = &events["smp_instruction_boundaries"];
        assert_eq!(boundaries["count"], 6_146);
        assert_eq!(
            boundaries["expanded_sha256"],
            "c7b7a58df7141c741a13e858532f317bd7e8c54e0056d15d79fb8c88fdb959d8"
        );
        let smp = expand_delta_sequence(&events["smp_instruction_boundary_sequence"]);
        assert_eq!(
            events["smp_instruction_boundary_sequence"]["fields"]
                .as_array()
                .unwrap()
                .len(),
            30
        );
        assert_eq!(smp.len(), 6_146);
        assert_eq!(
            smp.first().unwrap(),
            &vec![
                79, 1_355_567, 0x0800, 0x20, 0, 0, 0, 0xef, 2, 47, 0, 0, 319, 26, -45, 0, 8, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0,
            ]
        );
        assert_eq!(
            smp.last().unwrap(),
            &vec![
                81, 1_379_527, 0x0876, 0xf0, 63, 69, 0, 207, 2, 71, 7, 0, 2, 22, -46, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0,
            ]
        );
        let mut digest_bytes = Vec::with_capacity(smp.len() * 128);
        for row in &smp {
            digest_bytes.extend_from_slice(&(row[0] as u32).to_le_bytes());
            digest_bytes.extend_from_slice(&(row[1] as u64).to_le_bytes());
            for value in &row[2..] {
                digest_bytes.extend_from_slice(&(*value as i32).to_le_bytes());
            }
        }
        assert_eq!(
            boundaries["expanded_sha256"],
            parity::evidence::sha256_bytes(&digest_bytes)
        );
        assert_eq!(
            boundaries["before_sync"]["instruction"]["absolute_cycle"],
            1_379_507
        );
        assert_eq!(
            boundaries["before_sync"]["instruction"]["program_counter"],
            0x0873
        );
        assert_eq!(
            boundaries["after_sync"]["instruction"]["absolute_cycle"],
            1_379_527
        );
        assert_eq!(
            boundaries["after_sync"]["instruction"]["program_counter"],
            0x0876
        );
        assert_eq!(boundaries["absolute_end_cycle"], 1_379_531);
        assert_eq!(
            boundaries["terminal_successor"]["instruction"]["absolute_cycle"],
            1_379_531
        );
        assert_eq!(
            boundaries["terminal_successor"]["instruction"]["program_counter"],
            0x0873
        );
    }

    #[test]
    fn pinned_snes9x_first_nmi_dma_setup_fixture_stops_before_dma() {
        const FIXTURE: &str = include_str!(
            "../../external/snes9x-libretro/fixtures/zelda3-cold-first-nmi-dma-setup.jsonl"
        );
        assert!(
            FIXTURE.len() < 10_000,
            "DMA-setup fixture lost compact encoding"
        );
        let records = FIXTURE
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 2);

        let provenance = &records[0];
        assert_eq!(provenance["kind"], "provenance");
        assert_eq!(provenance["schema"], 1);
        assert_eq!(
            provenance["core"]["sha256"],
            "7d4fa577dd2e0a79ace97ebec2630929ffcd6622d46ae5b23c4700514c7169cb"
        );
        assert_eq!(
            provenance["source"]["revision"],
            "921f9f7b83660eb44ad263022a57a4a029057c37"
        );
        assert_eq!(
            provenance["rom"]["sha256"],
            "66871d66be19ad2c34c927d6b14cd8eb6fc3181965b6e517cb361f7316009cfb"
        );
        assert_eq!(
            provenance["core_build_receipt"]["patch_sha256s"],
            serde_json::json!([
                "a285f684f69b58959367877e173759baad0bc7f6b152d6644c5b6a00f691f5a3",
                "669b7767888be19f99623135ec00cc04913908fb74f19944da752fe863044dd2",
            ])
        );
        let prefix_fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../external/snes9x-libretro/fixtures/zelda3-cold-apu-first-nmi.jsonl");
        assert_eq!(
            provenance["prefix_fixture"]["sha256"],
            parity::evidence::sha256_file(&prefix_fixture).unwrap()
        );
        let initial_sram = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../routes/full_run/comparisons/continuous-audio/initial.srm");
        assert_eq!(
            provenance["initial_sram"],
            serde_json::json!({
                "source": "routes/full_run/comparisons/continuous-audio/initial.srm",
                "sha256": parity::evidence::sha256_file(&initial_sram).unwrap(),
                "bytes": 8_192,
                "valid_slot_marker": {"offset": 0x03e5, "bytes": [0xaa, 0x55]},
            })
        );

        let events = &records[1];
        assert_eq!(events["kind"], "first-nmi-dma-setup");
        assert_eq!(
            events["stop_reason"],
            "before_$008a35_sta_$420b_raw_fetch_transaction"
        );
        let anchor = &events["start_anchor"]["first_nmi_apui_read"];
        assert_eq!(anchor["instruction_origin_pc"], 0x0080e1);
        assert_eq!(anchor["access"]["cpu_cycle"], 480);
        assert_eq!(anchor["access"]["smp_clock_before"], 0);
        assert_eq!(anchor["access"]["smp_clock_after"], 1);
        assert_eq!(anchor["completed_timing_transaction"]["kind"], 2);
        assert_eq!(anchor["completed_timing_transaction"]["end_cpu_cycle"], 486);

        let cpu_timing = expand_delta_sequence(&events["cpu_timing_transaction_sequence"]);
        assert_eq!(cpu_timing.len(), 157);
        assert_eq!(
            cpu_timing.first().unwrap(),
            &vec![81, 0, 8, 0x80e4, 0xcd, 225, 486, 225, 494, 1, 2, 534, 534]
        );
        assert_eq!(
            &cpu_timing[cpu_timing.len() - 2..],
            &[
                vec![81, 0, 8, 0x8a33, 0xa9, 226, 698, 226, 706, 1, 2, 538, 538],
                vec![81, 1, 8, 0x8a33, 0xa9, 226, 706, 226, 714, 1, 2, 538, 538],
            ]
        );
        assert!(cpu_timing
            .iter()
            .all(|transaction| transaction[3] != 0x8a35));

        assert_eq!(
            events["final_completed_setup_instruction"]["rom_bytes"],
            serde_json::json!([0xa9, 0x07])
        );
        let excluded =
            &events["stop_before_instruction"]["excluded_raw_fetch_transaction"]["transaction"];
        assert_eq!(excluded["origin_pc"], 0x8a35);
        assert_eq!(excluded["opcode"], 0x8d);
        assert_eq!(excluded["kind"], 0);
        assert_eq!(excluded["start_v_counter"], 226);
        assert_eq!(excluded["start_cpu_cycle"], 714);
        assert_eq!(excluded["end_cpu_cycle"], 722);
    }

    #[test]
    fn first_nmi_dma_host_selector_and_compaction_are_exact_and_route_external() {
        let transaction = |kind, pc, opcode, start, end| LibretroCpuTimingTransaction {
            kind,
            duration: end - start,
            origin_pc: pc,
            opcode,
            start_v_counter: 226,
            start_cpu_cycle: start,
            end_v_counter: 226,
            end_cpu_cycle: end,
            cpu_model_identity: 1,
            cpu_model_5a22: 2,
            start_wram_refresh_position: 538,
            end_wram_refresh_position: 538,
        };
        let transactions = vec![
            transaction(0, 0x8a35, 0x8d, 714, 722),
            transaction(1, 0x8a35, 0x8d, 722, 730),
            transaction(1, 0x8a35, 0x8d, 730, 738),
            transaction(2, 0x8a35, 0x8d, 738, 900),
            transaction(0, 0x8a38, 0xa9, 900, 908),
        ];
        assert_eq!(
            first_nmi_dma_transaction_slice(&transactions).unwrap(),
            Some((0, 3, 4))
        );

        let event = |kind: i32, owner: i32, global: i32, channel: i32| {
            let mut fields = [-1; 72];
            fields[0] = kind;
            fields[1] = 3;
            fields[2] = owner;
            fields[3] = global;
            fields[4] = channel;
            fields[9] = 1;
            fields[27] = 0x8a38;
            if kind == 2 {
                fields[5] = 0;
                fields[50] = 0x1000 + global;
                fields[51] = 0x7e;
                fields[6] = (fields[51] << 16) | fields[50];
                fields[7] = 0x2118;
                fields[8] = 0x40 + global;
                fields[47] = 0;
                fields[48] = 0;
                fields[49] = 1;
                fields[63] = 0;
                fields[64] = fields[50] + 1;
            }
            LibretroDmaLedgerEvent { fields }
        };
        let mut events = vec![event(0, 7, -1, -1)];
        for (global, channel) in [0, 1, 2].into_iter().enumerate() {
            events.push(event(1, channel, -1, -1));
            events.push(event(2, channel, global as i32, 0));
            events.push(event(3, channel, -1, -1));
        }
        events.push(event(4, 7, -1, -1));
        let (outer, selected) = first_nmi_dma_ledger_slice(&events).unwrap().unwrap();
        assert_eq!(outer, 3);
        assert_eq!(selected, events);
        assert_eq!(
            semantic_receipts_from_dma_ledger(&events).unwrap(),
            vec![OriginalTimingSemanticReceipt::DmaPublicationCompleted { channel_mask: 7 }],
        );

        let zero_mask_events = vec![event(0, 0, -1, -1), event(4, 0, -1, -1)];
        assert_eq!(
            semantic_receipts_from_dma_ledger(&zero_mask_events).unwrap(),
            Vec::<OriginalTimingSemanticReceipt>::new(),
        );

        let compact = serde_json::to_value(compact_dma_ledger(&selected)).unwrap();
        assert_eq!(expand_delta_sequence(&compact).len(), selected.len());
        assert_eq!(
            expand_delta_sequence(&compact),
            selected
                .iter()
                .map(|event| event.fields.map(i64::from).to_vec())
                .collect::<Vec<_>>()
        );
        let snapshot = (0..=255).cycle().take(0x10000).collect::<Vec<u8>>();
        let compact = serde_json::to_value(compact_byte_snapshot(&snapshot)).unwrap();
        assert_eq!(
            expand_delta_sequence(&compact),
            snapshot
                .iter()
                .map(|byte| vec![i64::from(*byte)])
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn pinned_snes9x_first_nmi_dma_fixture_is_fully_reconstructable() {
        const FIXTURE: &str =
            include_str!("../../external/snes9x-libretro/fixtures/zelda3-cold-first-nmi-dma.jsonl");
        assert!(FIXTURE.len() < 12_000, "DMA fixture lost compact encoding");
        let records = FIXTURE
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 2);

        let provenance = &records[0];
        assert_eq!(provenance["kind"], "provenance");
        assert_eq!(
            provenance["core"]["sha256"],
            "425a2e8b451970dd3719a165259af265a53b468447c7f6a3b4e614d322204cc6"
        );
        assert_eq!(
            provenance["source"]["revision"],
            "921f9f7b83660eb44ad263022a57a4a029057c37"
        );
        assert_eq!(
            provenance["core_build_receipt"]["patch_sha256s"]
                .as_array()
                .unwrap()
                .len(),
            5
        );
        let patch = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../external/snes9x-libretro/patches/zelda3-dma-ledger.patch");
        assert_eq!(
            provenance["dma_ledger_patch"]["sha256"],
            parity::evidence::sha256_file(&patch).unwrap()
        );
        assert_eq!(provenance["dma_ledger_patch"]["core_route_selector"], false);
        let prefix = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../external/snes9x-libretro/fixtures/zelda3-cold-first-nmi-dma-setup.jsonl");
        assert_eq!(
            provenance["prefix_fixture"]["sha256"],
            parity::evidence::sha256_file(&prefix).unwrap()
        );

        let capture = &records[1];
        assert_eq!(capture["kind"], "first-nmi-dma");
        assert_eq!(capture["frame"], 81);
        assert_eq!(capture["dma"]["byte_count"], 160);
        assert_eq!(
            capture["stop_reason"],
            "completed_$008a35_sta_$420b_07_and_observed_$008a38_raw_fetch"
        );
        let source = &capture["source_instruction"];
        assert_eq!(source["raw_fetch_anchor"]["origin_pc"], 0x8a35);
        assert_eq!(source["raw_fetch_anchor"]["start_v_counter"], 226);
        assert_eq!(source["raw_fetch_anchor"]["start_cpu_cycle"], 714);
        assert_eq!(source["raw_fetch_anchor"]["end_cpu_cycle"], 722);
        assert_eq!(source["completed_outer_write_transaction"]["kind"], 2);
        assert_eq!(source["successor_raw_fetch"]["origin_pc"], 0x8a38);
        assert_eq!(
            source["completed_outer_write_transaction"]["end_cpu_cycle"],
            source["successor_raw_fetch"]["start_cpu_cycle"]
        );

        let cpu = expand_delta_sequence(&capture["cpu_timing_transaction_sequence"]);
        assert_eq!(cpu.len(), 4);
        assert_eq!(&cpu[0][1..9], &[0, 8, 0x8a35, 0x8d, 226, 714, 226, 722]);
        assert_eq!(cpu.last().unwrap()[1], 0);
        assert_eq!(cpu.last().unwrap()[3], 0x8a38);

        assert_eq!(
            capture["dma"]["ordered_event_sequence"]["fields"],
            serde_json::json!(super::DMA_LEDGER_FIELDS.to_vec())
        );
        let ledger = expand_delta_sequence(&capture["dma"]["ordered_event_sequence"]);
        assert_eq!(ledger.len(), 168);
        let events = ledger
            .into_iter()
            .map(|row| LibretroDmaLedgerEvent {
                fields: row
                    .into_iter()
                    .map(|value| i32::try_from(value).unwrap())
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            })
            .collect::<Vec<_>>();
        let (_, selected) = first_nmi_dma_ledger_slice(&events).unwrap().unwrap();
        assert_eq!(selected, events);
        let channel_byte_counts = [0, 1, 2].map(|channel| {
            events
                .iter()
                .filter(|event| event.fields[0] == 2 && event.fields[2] == channel)
                .count()
        });
        assert_eq!(channel_byte_counts, [64, 64, 32]);
        assert_eq!(
            capture["dma"]["hmax_crossing_receipts"]
                .as_array()
                .unwrap()
                .len(),
            1
        );

        let before = expand_delta_sequence(&capture["vram"]["before_sequence"])
            .into_iter()
            .map(|row| u8::try_from(row[0]).unwrap())
            .collect::<Vec<_>>();
        let after = expand_delta_sequence(&capture["vram"]["after_sequence"])
            .into_iter()
            .map(|row| u8::try_from(row[0]).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(before.len(), 0x10000);
        assert_eq!(after.len(), 0x10000);
        assert_eq!(
            capture["vram"]["before_sha256"],
            parity::evidence::sha256_bytes(&before)
        );
        assert_eq!(
            capture["vram"]["after_sha256"],
            parity::evidence::sha256_bytes(&after)
        );
        assert_ne!(before, after);
    }

    #[test]
    fn failed_session_keeps_the_complete_authoritative_input_script() {
        let source = b"0..5722 0x0000\n5723..21442 0x0080\n";
        let completed_prefix = [(0, 0), (1, 0), (2, 0)];

        assert_eq!(
            replayable_input_artifact(Some(source), &completed_prefix),
            source
        );
    }

    #[test]
    fn session_without_an_input_script_persists_its_captured_history() {
        let captured = [(0, 0), (1, 0x80), (2, 0x80)];

        assert_eq!(
            replayable_input_artifact(None, &captured),
            super::format_input_history(&captured).into_bytes()
        );
    }

    #[test]
    fn maps_cropped_libretro_video_rows_back_to_snes9x_presented_scanlines() {
        assert_eq!(snes9x_presented_scanline_for_video_y(224, 133), 140);
        assert_eq!(snes9x_presented_scanline_for_video_y(448, 266), 280);
        assert_eq!(snes9x_presented_scanline_for_video_y(239, 133), 133);
    }

    struct PresentedObjCacheFixture {
        meta: [i32; 5],
        validity: Vec<i32>,
        word_addresses: Vec<i32>,
        pixels: Vec<i32>,
    }

    impl PresentedObjCacheFixture {
        fn new(page_0_base: i32, page_1_base: i32) -> Self {
            Self {
                meta: [1, 512, 64, page_0_base, page_1_base],
                validity: vec![0; 512],
                word_addresses: vec![-1; 512],
                pixels: vec![0; 512 * 64],
            }
        }

        fn publish(&mut self, slot: usize) {
            let (page_base, page_slot) = if slot < 256 {
                (self.meta[3], slot)
            } else {
                (self.meta[4], slot - 256)
            };
            self.validity[slot] = 1;
            self.word_addresses[slot] = (page_base + (page_slot * 16) as i32) & 0x7fff;
            for pixel in 0..64 {
                self.pixels[slot * 64 + pixel] = (pixel & 0x0f) as i32;
            }
        }

        fn decode(&self) -> Result<Option<zelda3::PresentedObjTiles>, String> {
            decode_snes9x_presented_obj_tiles(|field, index| {
                let index = usize::try_from(index).ok()?;
                match field {
                    29 => self.pixels.get(index).copied(),
                    30 => self.validity.get(index).copied(),
                    45 => self.word_addresses.get(index).copied(),
                    46 => self.meta.get(index).copied(),
                    _ => None,
                }
            })
        }
    }

    #[test]
    fn address_bearing_obj_cache_maps_the_second_obsel_page() {
        let mut fixture = PresentedObjCacheFixture::new(0x4000, 0x5800);
        fixture.publish(256);

        let receipt = fixture.decode().unwrap().unwrap();
        let receipt = serde_json::to_value(receipt).unwrap();
        assert_eq!(receipt["tile_word_addresses"], serde_json::json!([0x5800]));
        assert_eq!(receipt["tile_pixels"].as_array().unwrap().len(), 64);
        assert_eq!(receipt["tile_pixels"][0], 0);
        assert_eq!(receipt["tile_pixels"][15], 15);
        assert_eq!(receipt["tile_pixels"][63], 15);
    }

    #[test]
    fn address_bearing_obj_cache_rejects_duplicate_physical_tiles() {
        let mut fixture = PresentedObjCacheFixture::new(0x4000, 0x4000);
        fixture.publish(0);
        fixture.publish(256);

        let error = fixture.decode().unwrap_err();
        assert!(
            error.contains("repeats physical word address 0x4000"),
            "{error}"
        );
    }

    #[test]
    fn address_bearing_obj_cache_requires_exact_validity_and_address_shape() {
        let mut fixture = PresentedObjCacheFixture::new(0x4000, 0x5800);
        fixture.validity[0] = 3;
        assert!(fixture
            .decode()
            .unwrap_err()
            .contains("validity 0 is invalid: 3"));

        fixture.validity[0] = 0;
        fixture.word_addresses[0] = 0x4000;
        assert!(fixture
            .decode()
            .unwrap_err()
            .contains("invalid presented OBJ cache slot 0 has word address 16384"));

        fixture.word_addresses[0] = -1;
        fixture.publish(256);
        fixture.word_addresses[256] = 0x5801;
        assert!(fixture
            .decode()
            .unwrap_err()
            .contains("word address 256 is invalid: 22529"));
    }

    #[test]
    fn address_bearing_obj_cache_rejects_old_or_malformed_abi() {
        assert_eq!(super::ORIGINAL_TIMING_HOST_RECEIPT_SCHEMA, 71);
        assert_eq!(
            decode_snes9x_presented_obj_tiles(|_, _| None).unwrap(),
            None
        );

        let mut fixture = PresentedObjCacheFixture::new(0x4000, 0x5800);
        fixture.meta[0] = -1;
        assert!(fixture
            .decode()
            .unwrap_err()
            .contains("unsupported presented OBJ cache ABI -1"));

        fixture.meta[0] = 1;
        fixture.meta[1] = 64;
        assert!(fixture
            .decode()
            .unwrap_err()
            .contains("slot count is invalid: 64"));
    }

    #[test]
    fn song_end_poll_receipt_keeps_only_exact_source_read_timing() {
        assert_eq!(
            super::song_end_poll_native_sample_offset(0x08_c400, 0, true, 75, 534)
                .unwrap()
                .unwrap(),
            75,
        );
        assert_eq!(
            super::song_end_poll_native_sample_offset(0x08_c609, 0, true, 96, 534)
                .unwrap()
                .unwrap(),
            96,
        );
        assert!(super::song_end_poll_native_sample_offset(0x00_80e4, 0, true, 75, 534).is_none());
        assert!(super::song_end_poll_native_sample_offset(0x08_c400, 0, false, 75, 534).is_none());
        assert!(super::song_end_poll_native_sample_offset(0x08_c400, 1, true, 75, 534).is_none());
    }

    #[test]
    fn song_end_poll_receipt_rejects_offsets_outside_the_host_audio_window() {
        assert!(
            super::song_end_poll_native_sample_offset(0x08_c609, 0, true, -1, 534)
                .unwrap()
                .unwrap_err()
                .contains("negative sample offset")
        );
        assert!(
            super::song_end_poll_native_sample_offset(0x08_c609, 0, true, 535, 534)
                .unwrap()
                .unwrap_err()
                .contains("beyond the 534-sample host window")
        );
    }

    #[test]
    fn obj_cache_comparison_ignores_invalid_snes9x_tiles() {
        let mut rust = vec![0; 64 * 64];
        let mut oracle = rust.clone();
        let mut valid = vec![0; 64];
        rust[3] = 1;
        oracle[3] = 2;
        rust[64 + 7] = 3;
        oracle[64 + 7] = 4;
        valid[1] = 1;

        assert_eq!(
            summarize_presented_obj_cache(Some(&rust), Some(&oracle), Some(&valid)),
            Some(ValueDomainDiff {
                rust_values: 64,
                oracle_values: 64,
                mismatched_values: 1,
                first_mismatch: Some(71),
            })
        );
    }

    #[test]
    fn obj_state_ledger_hash_is_stable_fnv1a() {
        assert_eq!(fnv1a32([0, 1, 2, 3]), 0xc3aa_51b1);
    }

    #[test]
    fn compact_engine_state_reports_the_first_semantic_scheduler_drift() {
        let mut rust = vec![0; 0x1000];
        let mut oracle = rust.clone();
        rust[0x10] = 7;
        oracle[0x10] = 7;
        rust[0x11] = 0x0e;
        oracle[0x11] = 0x0e;
        rust[0xb0] = 3;
        oracle[0xb0] = 4;
        rust[0x22..0x24].copy_from_slice(&0x05a9u16.to_le_bytes());
        oracle[0x22..0x24].copy_from_slice(&0x05a8u16.to_le_bytes());
        rust[0xe2..0xe4].copy_from_slice(&0x034eu16.to_le_bytes());
        oracle[0xe2..0xe4].copy_from_slice(&0x028eu16.to_le_bytes());
        rust[0x0df1] = 0x58;
        oracle[0x0df1] = 0x59;
        rust[0x0eb1] = 2;
        oracle[0x0eb1] = 3;

        assert_eq!(
            compact_engine_state_mismatches(&rust, &oracle),
            [
                "subsubmodule rust=0x03 oracle=0x04",
                "link_x rust=0x05a9 oracle=0x05a8",
                "bg2_h rust=0x034e oracle=0x028e",
                "sprite[1].head_direction rust=0x02 oracle=0x03",
                "sprite[1].delay_main rust=0x58 oracle=0x59",
            ]
        );
    }

    #[test]
    fn engine_state_comparison_fails_closed_with_an_explicit_diagnostic_opt_out() {
        assert_eq!(
            resolve_engine_state_compare_start(123, None, false),
            Ok(Some(123))
        );
        assert_eq!(
            resolve_engine_state_compare_start(123, Some(456), false),
            Ok(Some(456))
        );
        assert_eq!(
            resolve_engine_state_compare_start(123, None, true),
            Ok(None)
        );
        assert_eq!(
            resolve_engine_state_compare_start(123, Some(456), true),
            Err(
                "--ignore-engine-state cannot be combined with --compare-engine-state-from-frame"
                    .to_string()
            )
        );
    }

    #[test]
    fn live_oracle_rng_accepts_only_the_cartridge_store_site() {
        let sample = oracle_rng_sample_from_trace_line(
            r#"{"event":"rng-write","run":24377,"pc":899711,"value":134,"carry":1}"#,
            24377,
            24377,
        )
        .unwrap()
        .unwrap();
        assert_eq!(sample, RomRandomSample::with_carry(24377, 134, true));

        assert!(oracle_rng_sample_from_trace_line(
            r#"{"event":"rng-write","run":24377,"pc":57291,"value":36,"carry":0}"#,
            24377,
            24377,
        )
        .unwrap()
        .is_none());
        assert!(oracle_rng_sample_from_trace_line(
            r#"{"event":"rng-ppu-read","run":24377,"pc":899700,"value":1,"carry":0}"#,
            24377,
            24377,
        )
        .unwrap()
        .is_none());
        assert!(oracle_rng_sample_from_trace_line(
            r#"{"event":"dma","run":24377,"frame":24377,"v":228,"cycles":24}"#,
            24377,
            24377,
        )
        .unwrap()
        .is_none());
    }

    #[test]
    fn live_oracle_rng_preserves_requested_hardware_trace_domains() {
        assert_eq!(trace_events_with_rom_rng(None), "rom-rng");
        assert_eq!(
            trace_events_with_rom_rng(Some("frame,nmi,dma")),
            "frame,nmi,dma,rom-rng"
        );
        assert_eq!(
            trace_events_with_rom_rng(Some("nmi,rom-rng,dma")),
            "nmi,rom-rng,dma"
        );
    }

    #[test]
    fn live_oracle_rng_rejects_cross_frame_samples() {
        let error = oracle_rng_sample_from_trace_line(
            r#"{"event":"rng-write","run":24486,"pc":899711,"value":125,"carry":0}"#,
            24458,
            24458,
        )
        .unwrap_err();
        assert!(error.contains("run 24486"));
        assert!(error.contains("frame 24458"));
    }

    #[test]
    fn live_oracle_rng_maps_resumed_trace_run_to_absolute_execution_frame() {
        let sample = oracle_rng_sample_from_trace_line(
            r#"{"event":"rng-write","run":119,"pc":899711,"value":32,"carry":1}"#,
            119,
            56_119,
        )
        .unwrap()
        .unwrap();

        assert_eq!(sample, RomRandomSample::with_carry(56_119, 32, true));
    }

    #[test]
    fn oracle_capture_rejects_stale_or_mismatched_rng_scripts() {
        let expected = [
            RomRandomSample::with_carry(7, 0x22, false),
            RomRandomSample::with_carry(7, 0x45, true),
            RomRandomSample::with_carry(9, 0x88, false),
        ];
        let mut cursor = 0;
        assert!(validate_oracle_rng_samples_for_run(&expected, &mut cursor, 6, &[]).is_ok());
        assert!(
            validate_oracle_rng_samples_for_run(&expected, &mut cursor, 7, &expected[..2],).is_ok()
        );
        assert_eq!(cursor, 2);
        let mismatch = validate_oracle_rng_samples_for_run(
            &expected,
            &mut cursor,
            9,
            &[RomRandomSample::with_carry(9, 0x89, false)],
        )
        .unwrap_err();
        assert!(mismatch.contains("frame 9"));

        let mut stale_cursor = 0;
        let stale =
            validate_oracle_rng_samples_for_run(&expected, &mut stale_cursor, 8, &[]).unwrap_err();
        assert!(stale.contains("frame 7"));
    }

    #[test]
    fn parse_debug_frame_selection_expands_and_deduplicates_ranges() {
        assert_eq!(
            parse_debug_frame_selection("81,79-81,84..=85,invalid,8-3"),
            vec![79, 80, 81, 84, 85]
        );
    }

    #[test]
    fn session_receipts_do_not_force_a_frontier_probe_to_scan_past_its_first_mismatch() {
        assert!(!scan_all_policy(false, true));
        assert!(scan_all_policy(true, true));
        assert!(should_stop_after_first_mismatch(false, true, false));
        assert!(should_stop_after_first_mismatch(false, false, true));
        assert!(!should_stop_after_first_mismatch(true, true, true));
    }

    #[test]
    fn late_probe_receipts_skip_the_uncompared_warmup() {
        assert!(!should_write_frame_receipt(10_000, 10_000, 10_100, false));
        assert!(should_write_frame_receipt(10_000, 10_000, 10_100, true));
    }

    #[test]
    fn long_cold_sweeps_sample_receipts_without_losing_the_boundary() {
        assert!(should_write_frame_receipt(0, 0, 20_000, true));
        assert!(!should_write_frame_receipt(1, 0, 20_000, true));
        assert!(should_write_frame_receipt(60, 0, 20_000, true));
        assert!(should_write_frame_receipt(12_345, 12_345, 30_000, true));
        assert!(!should_write_frame_receipt(12_346, 12_345, 30_000, true));
    }

    #[test]
    fn late_video_windows_render_only_the_priming_tail_and_comparison() {
        assert!(!should_render_video_frame(10_000, 24_427, true));
        assert!(!should_render_video_frame(24_366, 24_427, true));
        assert!(should_render_video_frame(24_367, 24_427, true));
        assert!(should_render_video_frame(24_427, 24_427, true));
        assert!(!should_render_video_frame(24_427, 24_427, false));
    }

    #[test]
    fn engine_receipts_retain_full_sprite_motion_witnesses() {
        let mut ram = vec![0; 0x20_000];
        ram[0x0aa3] = 0x2a;
        ram[0xc2fc..0xc300].copy_from_slice(&[0x10, 0x20, 0x30, 0x40]);
        ram[0x0d10] = 0x68;
        ram[0x0d30] = 0x04;
        ram[0x0d70] = 0x80;
        ram[0x0d50] = 0x08;
        ram[0x0e70] = 0x02;

        let receipt = libretro_engine_state_receipt(&ram);
        assert_eq!(receipt["sprite_graphics_index"], 0x2a);
        assert_eq!(
            receipt["sprite_graphics_subsets"],
            serde_json::json!([0x10, 0x20, 0x30, 0x40])
        );
        let slot = &receipt["sprite_slots"][0];
        assert_eq!(slot["x"], 0x0468);
        assert_eq!(slot["x_subpixel"], 0x80);
        assert_eq!(slot["x_velocity"], 0x08);
        assert_eq!(slot["wall_collision"], 0x02);
    }

    #[test]
    fn engine_receipts_retain_full_ancilla_motion_witnesses() {
        let mut ram = vec![0; 0x20_000];
        ram[0x0c04] = 0x68;
        ram[0x0c18] = 0x04;
        ram[0x0c40] = 0x80;
        ram[0x0c2c] = 0x08;
        ram[0x0c4a] = 0x02;
        ram[0x0c90] = 0x04;

        let receipt = libretro_engine_state_receipt(&ram);
        let slot = &receipt["ancilla_slots"][0];
        assert_eq!(slot["x"], 0x0468);
        assert_eq!(slot["x_subpixel"], 0x80);
        assert_eq!(slot["x_velocity"], 0x08);
        assert_eq!(slot["type"], 0x02);
        assert_eq!(slot["num_sprites"], 0x04);
    }

    #[test]
    fn engine_receipts_retain_each_oam_shadow_entry() {
        let mut ram = vec![0; 0x20_000];
        ram[0x0800 + 37 * 4..0x0800 + 38 * 4].copy_from_slice(&[0x68, 0x57, 0x40, 0x3c]);

        let receipt = libretro_engine_state_receipt(&ram);

        assert_eq!(receipt["oam_slots"].as_array().unwrap().len(), 128);
        assert_eq!(
            receipt["oam_slots"][37],
            serde_json::json!({
                "slot": 37,
                "x": 0x68,
                "y": 0x57,
                "tile": 0x40,
                "flags": 0x3c,
            })
        );
        assert_eq!(receipt["oam_extended"].as_array().unwrap().len(), 32);
    }

    #[test]
    fn dsp_event_timing_comparison_reports_phase_and_length_drift() {
        let rust = [DspWriteEvent::new(0x5c, 8, 228, 15)];
        let exact = [LibretroDspRegisterWrite {
            register: 0x5c,
            value: 8,
            output_sample: 228,
            dsp_phase: 15,
        }];
        assert_eq!(first_dsp_write_timing_mismatch(&rust, &exact), None);

        let phase_drift = [LibretroDspRegisterWrite {
            dsp_phase: 16,
            ..exact[0]
        }];
        assert_eq!(
            first_dsp_write_timing_mismatch(&rust, &phase_drift),
            Some(0)
        );
        assert_eq!(first_dsp_write_timing_mismatch(&rust, &[]), Some(0));
    }

    #[test]
    fn spc_clock_witness_aligns_different_frame_boundary_instructions() {
        let rust_instruction = snes::apu::SpcInstructionTrace {
            cycle: 0,
            pc: 0x1234,
            opcode: 0xe8,
            operands: [0; 2],
            a: 1,
            x: 2,
            y: 3,
            sp: 4,
            p: false,
            direct_page_0_3: [0; 4],
            direct_page_4_7: [0; 4],
            direct_page_8_11: [0; 4],
            input_ports: [0; 4],
            timer0_cycles: 100,
            timer0_divider: 6,
            timer0_counter: 0,
        };
        let oracle_instruction = crate::libretro_core::LibretroSmpInstruction {
            absolute_cycle: 0,
            program_counter: 0x1234,
            opcode: 0xe8,
            a: 1,
            x: 2,
            y: 3,
            stack_pointer: 4,
            status: 0,
            direct_page_0_11: [0; 12],
            timer0_stage1: 27,
            timer0_stage2: 6,
            timer0_stage3: 0,
            output_sample: 0,
            dsp_phase: 0,
            smp_clock: 0,
            boundary_opcode_cycle: 0,
            op_step_calls: 1,
            max_continuation_opcode_cycle: 0,
        };

        let witness =
            last_spc_clock_witness(&[rust_instruction, rust_instruction], &[oracle_instruction])
                .unwrap();

        assert_eq!(witness.phase_delta, 1);
        assert_eq!(witness.rust_tail, 0);
        assert_eq!(witness.oracle_tail, 0);
    }

    #[test]
    fn authoritative_oracle_never_serializes_during_frame_execution() {
        assert!(!oracle_preframe_snapshot_required(10_000, 23_005, false));
        assert!(!oracle_preframe_snapshot_required(23_000, 23_005, true));
        assert!(!oracle_preframe_snapshot_required(23_004, 23_005, false));
    }

    #[test]
    fn replay_bundle_rejects_requests_past_its_recorded_coverage() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zelda3-replay-bundle-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let rom = root.join("test.sfc");
        fs::write(&rom, b"rom").unwrap();
        fs::write(root.join("input.txt"), b"0 0\n").unwrap();
        fs::write(root.join("rom-random.txt"), b"").unwrap();
        fs::write(root.join("initial.srm"), b"sram").unwrap();
        let rom_sha256 = parity::runner::sha256_file(&rom).unwrap();
        let rng_sha256 = parity::runner::sha256_file(&root.join("rom-random.txt")).unwrap();
        fs::write(
            root.join("manifest.json"),
            serde_json::to_vec(&serde_json::json!({
                "schema": 1,
                "frames_completed": 2298,
                "rom": { "sha256": rom_sha256 },
                "rom_random_replay": { "sha256": rng_sha256 },
            }))
            .unwrap(),
        )
        .unwrap();

        let error = resolve_replay_bundle(&root, 15_000, &rom).unwrap_err();
        assert!(error.contains("proven through frame 2298"), "{error}");
        assert!(resolve_replay_bundle(&root, 2_298, &rom).is_ok());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn replay_sources_from_different_directories_require_an_explicit_unsafe_opt_in() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zelda3-mixed-replay-{}-{unique}",
            std::process::id()
        ));
        let first = root.join("first");
        let second = root.join("second");
        fs::create_dir_all(&first).unwrap();
        fs::create_dir_all(&second).unwrap();
        let input = first.join("input.txt");
        let rng = second.join("rom-random.txt");
        fs::write(&input, b"").unwrap();
        fs::write(&rng, b"").unwrap();

        let sources = [
            ("--input-script", Some(input.as_path())),
            ("--rom-random-script", Some(rng.as_path())),
            ("--load-sram", None),
        ];
        let error = validate_replay_source_parents(&sources, false).unwrap_err();
        assert!(
            error.contains("mixed replay provenance is unsafe"),
            "{error}"
        );
        assert!(validate_replay_source_parents(&sources, true).is_ok());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn paired_resume_capture_keeps_an_absolute_pre_frame_boundary() {
        assert_eq!(
            parse_paired_resume_capture("12120", "target/parity-checkpoints/frontier").unwrap(),
            PairedResumeCapture {
                frame: 12120,
                dir: PathBuf::from("target/parity-checkpoints/frontier"),
            }
        );
        assert!(parse_paired_resume_capture("not-a-frame", "unused").is_err());
    }

    #[test]
    fn rolling_paired_resume_schedules_the_next_interval_boundary() {
        assert_eq!(
            parse_rolling_paired_resume_capture("256", "target/parity-checkpoints/frontier")
                .unwrap(),
            RollingPairedResumeCapture {
                interval: 256,
                root: PathBuf::from("target/parity-checkpoints/frontier"),
            }
        );
        assert!(parse_rolling_paired_resume_capture("0", "unused").is_err());
        assert_eq!(rolling_capture_frame_after(0, 256), 256);
        assert_eq!(rolling_capture_frame_after(255, 256), 256);
        assert_eq!(rolling_capture_frame_after(256, 256), 512);
        assert_eq!(rolling_capture_frame_after(3400, 256), 3584);
    }

    #[test]
    fn paired_resume_manifest_paths_cannot_escape_the_checkpoint() {
        let root = Path::new("target/parity-checkpoints/frontier");
        assert_eq!(
            checkpoint_member(root, "rust.z3state").unwrap(),
            root.join("rust.z3state")
        );
        assert!(checkpoint_member(root, "../rust.z3state").is_err());
        assert!(checkpoint_member(root, "/tmp/rust.z3state").is_err());
    }

    #[test]
    fn paired_resume_root_resolves_its_latest_complete_generation() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zelda3-paired-resume-{}-{unique}",
            std::process::id()
        ));
        let checkpoint = write_paired_resume_test_generation(&root, "frame-00003700", 3700, 3700);
        fs::write(
            root.join("latest.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema": PAIRED_RESUME_SCHEMA,
                "frame": 3700,
                "checkpoint": "frame-00003700",
            }))
            .unwrap(),
        )
        .unwrap();

        assert_eq!(
            paired_resume_paths(&root).unwrap(),
            (
                checkpoint.join("rust.z3state"),
                checkpoint.join("oracle.state"),
                checkpoint.join("original-timing.resume.json"),
                checkpoint.join("semantic-trace.checkpoint.json")
            )
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn paired_resume_v2_verifies_every_bound_artifact() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zelda3-paired-resume-hashes-{}-{unique}",
            std::process::id()
        ));
        let checkpoint = write_paired_resume_test_generation(&root, "checkpoint", 3700, 3700);
        for (artifact, expected_label) in [
            ("rust.z3state", "Rust state"),
            ("oracle.state", "oracle state"),
            ("original-timing.resume.json", "original-timing checkpoint"),
            (
                "semantic-trace.checkpoint.json",
                "semantic-trace checkpoint",
            ),
            ("initial.srm", "initial SRAM"),
        ] {
            let path = checkpoint.join(artifact);
            let original = fs::read(&path).unwrap();
            fs::write(&path, b"mixed checkpoint artifact").unwrap();
            let error = paired_resume_paths(&checkpoint).unwrap_err();
            assert!(
                error.contains(expected_label) && error.contains("hash mismatch"),
                "unexpected {artifact} validation error: {error}"
            );
            fs::write(path, original).unwrap();
        }
        assert!(paired_resume_paths(&checkpoint).is_ok());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn paired_resume_provenance_binds_every_selected_causal_source() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zelda3-paired-resume-provenance-{}-{unique}",
            std::process::id()
        ));
        let checkpoint = write_paired_resume_test_generation(&root, "checkpoint", 3700, 3700);
        let core = root.join("core.dylib");
        let rom = root.join("zelda3.sfc");
        let input = root.join("input.txt");
        let rng = root.join("rom-random.txt");
        for (path, bytes) in [
            (&core, b"core".as_slice()),
            (&rom, b"rom".as_slice()),
            (&input, b"input".as_slice()),
            (&rng, b"rng".as_slice()),
        ] {
            fs::write(path, bytes).unwrap();
        }
        let manifest_path = checkpoint.join("manifest.json");
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        for (key, path) in [
            ("core", core.as_path()),
            ("rom", rom.as_path()),
            ("input_script", input.as_path()),
            ("rom_random_script", rng.as_path()),
        ] {
            manifest[key] = serde_json::json!({
                "sha256": parity::evidence::sha256_file(path).unwrap(),
            });
        }
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();

        assert!(validate_paired_resume_provenance(
            &checkpoint,
            &core,
            &rom,
            Some(&input),
            Some(&rng),
            false,
        )
        .is_ok());
        for (path, label) in [
            (&core, "core"),
            (&rom, "ROM"),
            (&input, "input script"),
            (&rng, "ROM-random script"),
        ] {
            let original = fs::read(path).unwrap();
            fs::write(path, b"different selected source").unwrap();
            let error = validate_paired_resume_provenance(
                &checkpoint,
                &core,
                &rom,
                Some(&input),
                Some(&rng),
                false,
            )
            .unwrap_err();
            assert!(
                error.contains(label) && error.contains("mismatch"),
                "{error}"
            );
            assert!(validate_paired_resume_provenance(
                &checkpoint,
                &core,
                &rom,
                Some(&input),
                Some(&rng),
                true,
            )
            .is_ok());
            fs::write(path, original).unwrap();
        }
        let error =
            validate_paired_resume_provenance(&checkpoint, &core, &rom, None, Some(&rng), false)
                .unwrap_err();
        assert!(
            error.contains("input script provenance requires"),
            "{error}"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn paired_resume_rejects_progressed_state_sram_replacement() {
        assert!(validate_paired_resume_sram_selection(false, false).is_ok());
        assert!(validate_paired_resume_sram_selection(false, true).is_ok());
        assert!(validate_paired_resume_sram_selection(true, false).is_ok());
        let error = validate_paired_resume_sram_selection(true, true).unwrap_err();
        assert!(
            error.contains("cannot be combined with --load-sram"),
            "{error}"
        );
        assert!(error.contains("provenance"), "{error}");
    }

    #[test]
    fn paired_generation_install_is_atomic_and_never_replaces_same_frame() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zelda3-paired-atomic-{}-{unique}",
            std::process::id()
        ));
        let generation = root.join("frame-00003700");
        install_directory_atomically(&generation, |temporary| {
            fs::write(temporary.join("manifest.json"), b"first")?;
            Ok(())
        })
        .unwrap();
        let error = install_directory_atomically(&generation, |temporary| {
            fs::write(temporary.join("manifest.json"), b"replacement")?;
            Ok(())
        })
        .unwrap_err();
        assert!(error.to_string().contains("refusing to replace"), "{error}");
        assert_eq!(
            fs::read(generation.join("manifest.json")).unwrap(),
            b"first"
        );

        let failed = root.join("frame-00003800");
        let error = install_directory_atomically(&failed, |temporary| {
            fs::write(temporary.join("partial"), b"partial")?;
            Err("injected write failure".into())
        })
        .unwrap_err();
        assert!(error.to_string().contains("injected write failure"));
        assert!(!failed.exists());
        assert!(!root
            .join(format!(".frame-00003800.tmp-{}", std::process::id()))
            .exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rolling_latest_pointer_is_replaced_as_one_complete_file() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zelda3-paired-latest-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let latest = root.join("latest.json");
        fs::write(&latest, br#"{"schema":2,"frame":100}"#).unwrap();
        let replacement = br#"{"schema":2,"frame":200}"#;
        write_file_atomically(&latest, replacement).unwrap();
        assert_eq!(fs::read(&latest).unwrap(), replacement);
        assert!(!root
            .join(format!(".latest.json.tmp-{}", std::process::id()))
            .exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn oracle_av_capture_rejects_every_mid_run_checkpoint_interval() {
        assert!(validate_oracle_av_checkpoint_interval(None).is_ok());
        for interval in [1, 5_000, u32::MAX] {
            let error = validate_oracle_av_checkpoint_interval(Some(interval)).unwrap_err();
            assert!(
                error.contains("serialization mutates live DSP state"),
                "{error}"
            );
        }
    }

    #[test]
    fn cold_evidence_invocation_id_has_the_receipt_safe_grammar() {
        for value in ["run-10000-123.abc", "A_b-9"] {
            assert!(validate_cold_evidence_invocation_id(value).is_ok());
        }
        for value in ["", "contains/slash", "contains space", "contains:colon"] {
            assert!(
                validate_cold_evidence_invocation_id(value).is_err(),
                "{value:?}"
            );
        }
        assert!(validate_cold_evidence_invocation_id(&"a".repeat(129)).is_err());
    }

    #[test]
    fn cold_evidence_run_nonce_is_runner_authored_and_source_bound() {
        let session = Path::new("/tmp/zelda3-cold-session");
        let nonce = cold_evidence_run_nonce(session, "invocation-1", 42, 7);
        assert_eq!(nonce.len(), 64);
        assert!(nonce.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert_eq!(
            nonce,
            cold_evidence_run_nonce(session, "invocation-1", 42, 7)
        );
        assert_ne!(
            nonce,
            cold_evidence_run_nonce(session, "invocation-1", 43, 7)
        );
        assert_ne!(
            nonce,
            cold_evidence_run_nonce(session, "invocation-2", 42, 7)
        );
    }

    #[test]
    fn paired_resume_direct_directory_rejects_host_frame_mismatch_before_restore() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zelda3-paired-resume-frame-{}-{unique}",
            std::process::id()
        ));
        let checkpoint = write_paired_resume_test_generation(&root, "checkpoint", 3700, 3699);
        let rust_state_before = fs::read(checkpoint.join("rust.z3state")).unwrap();
        let timing_sidecar_before =
            fs::read(checkpoint.join("original-timing.resume.json")).unwrap();
        let error = paired_resume_paths(&checkpoint).unwrap_err();
        assert!(error.contains("records frame 3700"), "{error}");
        assert!(
            error.contains("Rust checkpoint records frame 3699"),
            "{error}"
        );
        assert_eq!(
            fs::read(checkpoint.join("rust.z3state")).unwrap(),
            rust_state_before,
            "validation must not rewrite the restored Rust state"
        );
        assert_eq!(
            fs::read(checkpoint.join("original-timing.resume.json")).unwrap(),
            timing_sidecar_before,
            "validation must fail before any sidecar restoration or rewrite"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn paired_resume_schema_one_is_rejected_before_artifact_access() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zelda3-paired-resume-v1-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("manifest.json"),
            br#"{"schema":1,"boundary":"pre-frame","frame":3700}"#,
        )
        .unwrap();
        let error = paired_resume_paths(&root).unwrap_err();
        assert!(
            error.contains("unsupported paired-resume schema 1"),
            "{error}"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rolling_prune_preserves_the_new_generation_after_a_restart() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zelda3-rolling-prune-{}-{unique}",
            std::process::id()
        ));
        for frame in [3600, 3700, 100] {
            let dir = root.join(format!("frame-{frame:08}"));
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("manifest.json"), b"{}").unwrap();
        }
        let current = root.join("frame-00000100");

        prune_rolling_paired_resume_captures(&root, 2, &current);

        assert!(current.is_dir());
        assert_eq!(
            fs::read_dir(&root)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.path().is_dir())
                .count(),
            2
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn boot_boundary_reports_the_first_named_semantic_difference() {
        let mut rust_ram = vec![0; 0x20];
        let mut oracle_ram = vec![0; 0x20];
        rust_ram[0x13] = 0x0f;
        oracle_ram[0x13] = 0x0e;
        oracle_ram[0x17] = 3;

        let rust = BootBoundaryState::from_ram(82, "after", &rust_ram);
        let oracle = BootBoundaryState::from_ram(82, "after", &oracle_ram);

        assert_eq!(
            rust.first_difference(&oracle),
            Some(("inidisp", 0x0f, 0x0e))
        );
    }

    #[test]
    fn value_domain_diff_reports_content_and_generation_length_skew() {
        assert_eq!(
            summarize_value_domain(&[1, 2, 3], &[1, 4, 3, 5]),
            ValueDomainDiff {
                rust_values: 3,
                oracle_values: 4,
                mismatched_values: 2,
                first_mismatch: Some(1),
            }
        );
        assert!(summarize_value_domain(&[1, 2], &[1, 2]).is_exact());
    }

    #[test]
    fn vram_receipt_reports_first_word_and_compact_ranges() {
        let rust = [0x1111, 0x2222, 0x3333, 0x4444, 0x5555];
        let oracle_words = [0x1111u16, 0xaaaa, 0xbbbb, 0x4444, 0xcccc];
        let oracle = oracle_words
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect::<Vec<_>>();

        assert_eq!(
            vram_domain_receipt(&rust, Some(&oracle)),
            Some(VramDomainReceipt {
                rust_words: 5,
                oracle_words: 5,
                rust_sha256: parity::evidence::sha256_bytes(
                    &rust
                        .iter()
                        .flat_map(|word| word.to_le_bytes())
                        .collect::<Vec<_>>(),
                ),
                oracle_sha256: parity::evidence::sha256_bytes(&oracle),
                mismatched_words: 3,
                first_mismatch_word: Some(1),
                first_rust_word: Some(0x2222),
                first_oracle_word: Some(0xaaaa),
                mismatch_ranges: vec![[1, 3], [4, 5]],
                mismatch_ranges_truncated: false,
                mismatch_blocks: vec![[0, 3]],
            })
        );
    }

    #[test]
    fn canonical_video_hash_ignores_alpha_and_libretro_pitch_padding() {
        let rust = [10, 20, 30, 0, 40, 50, 60, 1];
        let oracle = LibretroFrame {
            audio: Vec::new(),
            // Libretro XRGB8888 is exposed as B,G,R,X bytes here. The final
            // four bytes are row padding and must not enter the digest.
            video: vec![30, 20, 10, 255, 60, 50, 40, 128, 9, 9, 9, 9],
            video_width: 2,
            video_height: 1,
            video_pitch: 12,
            pixel_format: 1,
        };
        let rust_digest = canonical_rust_video_digest(&rust, 2, 1).unwrap();
        let oracle_digest = canonical_oracle_video_digest(&oracle).unwrap();
        assert_eq!(rust_digest["sha256"], oracle_digest["sha256"]);
    }

    #[test]
    fn canonical_audio_hash_is_little_endian_interleaved_i16() {
        let samples = [0x1234_i16, -2_i16];
        let digest = canonical_audio_digest(&samples);
        assert_eq!(digest["sample_frames"], 1);
        assert_eq!(digest["channels"], 2);
        assert_eq!(
            digest["sha256"],
            parity::evidence::sha256_bytes(&[0x34, 0x12, 0xfe, 0xff])
        );
    }

    #[test]
    fn cached_av_inputs_require_an_explicit_hexadecimal_receipt() {
        assert_eq!(cached_ledger_input("0x8080").unwrap(), 0x8080);
        assert!(cached_ledger_input("32896").is_err());
        assert!(cached_ledger_input("0x10000").is_err());
    }
}
