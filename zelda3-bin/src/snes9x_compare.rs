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
// Schema 77 lets a terminal dialogue caller return supersede its VWF endpoint.
// Schema 78 carries source scroll entry, completed pixel-copy passes, and return.
// Schema 79 retains the VWF endpoint when a carried NMI resumes its glyph.
// Schema 80 recognizes Death_Func15's suspended Sprite_ResetAll caller.
// Schema 81 binds spotlight loop-test X to the event-local source cursor.
// Schema 82 recognizes the spotlight beam wait before projection begins.
// Schema 83 retains Hog Spear Man before the body graphics store.
// Schema 84 retains the nested guard initializer's patrol delay boundary.
// Schema 85 retains initialized sprites before their type-specific prep.
const ORIGINAL_TIMING_HOST_RECEIPT_SCHEMA: u32 = 138;

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

const FIRST_NMI_RETURN_RECEIPT_FRAME: u32 = 81;
const FIRST_NMI_RETURN_HOST_FRAME: u32 = 81;
const FIRST_NMI_RETURN_RETRO_RUN: u32 = 81;
const FIRST_NMI_RETURN_START_PC: i32 = 0x008a38;

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

/// Canonical-JSON entry point kept for fixtures and tests; the live reader
/// decodes Z3TRACE1 records directly through `oracle_rng_sample_from_record`.
#[cfg(test)]
fn oracle_rng_sample_from_trace_line(
    line: &str,
    expected_trace_run: u32,
    execution_frame: u32,
) -> Result<Option<RomRandomSample>, String> {
    let value: serde_json::Value = serde_json::from_str(line)
        .map_err(|error| format!("invalid live oracle trace event: {error}"))?;
    let record = parity::trace_format::TraceRecord::from_json(&value)
        .map_err(|error| format!("invalid live oracle trace event: {error}"))?;
    oracle_rng_sample_from_record(&record, expected_trace_run, execution_frame)
}

const ROLLING_PAIRED_RESUME_GENERATIONS_KEPT: usize = 2;

const fn semantic_trace_authority_available(
    trace_configured: bool,
    generic_trace_api_exported: bool,
) -> bool {
    trace_configured && generic_trace_api_exported
}

const PAIRED_RESUME_SCHEMA: u32 = 2;

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
            semantic_trace.take_host_dialogue_scroll_progress(),
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
        // The Rust-only cached replay never loads an oracle state, so a
        // Rust-only checkpoint (no oracle artifacts) is accepted here.
        let PairedResumeArtifacts {
            rust_state,
            original_timing_resume,
            ..
        } = paired_resume_artifacts(path).unwrap_or_else(|error| {
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
    } else if std::env::var_os("ZELDA3_CACHED_AV_ROM_TIMING_OFF").is_some() {
        // The ROM-free frontier: the no-argument launch's engine (ROM
        // startup timing off, no ROM code available to any plan). Pair with
        // ZELDA3_CACHED_AV_NATIVE_TIMING=1; receipts cannot install without
        // the timing flag.
        (crate::load_romless_play_state(), 0)
    } else {
        (load_default_play_state(), 0)
    };
    let rom_timing_off = std::env::var_os("ZELDA3_CACHED_AV_ROM_TIMING_OFF").is_some();
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
    if !rom_timing_off {
        game.set_rom(&rom_bytes);
    }
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
    let audio_buffer = Vec::<i16>::new();
    let ledger_frames_seen = cache_start_frame;
    let frames_completed = start_frame;
    let mut frames_compared = 0_u32;
    let mut matched = true;
    let first_rng_drift = None;
    let mut pending_frames = VecDeque::<PendingCachedAvFrame>::new();
    // A periodic paired checkpoint comes due at each interval multiple and is
    // taken at the first later frame boundary the resume machinery can
    // serialize (quiescent CPU, no unconsumed receipt or pending NMI
    // publication) instead of failing the replay on an unlucky boundary.
    let paired_checkpoint_due = false;
    let timing_enabled = env::var_os("ZELDA3_SNES9X_TIMING").is_some();
    let debug_wram_frames =
        debug_frame_selection_from_env("ZELDA3_DEBUG_WRAM_FRAMES", Some("ZELDA3_DEBUG_WRAM_FRAME"));
    let replay_started = Instant::now();
    let mut serialization_nanos = 0_u128;
    // Ledger + receipt parsing (JSON, ~10% of a cached-av frame) runs on a
    // producer thread; the pairing of one receipt line per ledger line and
    // every provenance check are unchanged, the checks simply run here on the
    // consumer side in the same order.
    struct ParsedCachedAvInput {
        line_index: usize,
        record: CachedOracleAvRecord,
        /// `None` for a frame before the resumed boundary: its receipt line was
        /// consumed to keep the two ledgers paired but is never installed, so
        /// only the record head (frame, input) is decoded for the provenance
        /// checks. Parsing every skipped receipt cost ~0.3 ms per frame (two
        /// minutes ahead of a 380k-frame boundary).
        timing_receipts: Option<OriginalTimingHostReceipts>,
    }
    #[derive(serde::Deserialize)]
    struct CachedOracleAvRecordHead {
        schema: u32,
        frame: u32,
        input: String,
    }
    let (parsed_tx, parsed_rx) =
        std::sync::mpsc::sync_channel::<Result<ParsedCachedAvInput, String>>(512);
    let parse_ledger_path = ledger_path.clone();
    let parse_thread = std::thread::Builder::new()
        .name("cached-av-parse".into())
        .spawn(move || {
            let ledger_path = parse_ledger_path;
            for (line_index, line) in BufReader::new(ledger_file).lines().enumerate() {
                let item = (|| -> Result<ParsedCachedAvInput, String> {
                    let line = line.map_err(|error| {
                        format!("failed to read {}: {error}", ledger_path.display())
                    })?;
                    let invalid_record = |error: serde_json::Error| {
                        format!(
                            "invalid cached A/V record {}:{}: {error}",
                            ledger_path.display(),
                            line_index + 1
                        )
                    };
                    let skipped_frame = cache_start_frame
                        .checked_add(u32::try_from(line_index).unwrap_or(u32::MAX))
                        .is_some_and(|frame| frame < start_frame);
                    let record = if skipped_frame {
                        let head: CachedOracleAvRecordHead =
                            serde_json::from_str(&line).map_err(invalid_record)?;
                        CachedOracleAvRecord {
                            schema: head.schema,
                            frame: head.frame,
                            input: head.input,
                            oracle_audio_sample_frames: None,
                            video: None,
                            audio: None,
                        }
                    } else {
                        serde_json::from_str(&line).map_err(invalid_record)?
                    };
                    let timing_receipt_line = timing_receipt_lines
                        .next()
                        .ok_or_else(|| {
                            format!(
                                "cached source receipt ledger ended before frame {}",
                                record.frame
                            )
                        })?
                        .map_err(|error| {
                            format!(
                                "failed to read cached source receipt at frame {}: {error}",
                                record.frame
                            )
                        })?;
                    let timing_receipts = if skipped_frame && record.frame < start_frame {
                        None
                    } else {
                        Some(serde_json::from_str(&timing_receipt_line).map_err(|error| {
                            format!(
                                "invalid cached source receipt at frame {}: {error}",
                                record.frame
                            )
                        })?)
                    };
                    Ok(ParsedCachedAvInput {
                        line_index,
                        record,
                        timing_receipts,
                    })
                })();
                let failed = item.is_err();
                if parsed_tx.send(item).is_err() || failed {
                    break;
                }
            }
        })
        .unwrap_or_else(|error| {
            eprintln!("failed to start the cached A/V parse thread: {error}");
            process::exit(2);
        });
    // The game runs on a simulation thread: receipts, the frame, the display
    // capture, audio, WRAM dumps and checkpoint clones happen there, and each
    // frame's outputs cross a bounded channel to this thread, which owns the
    // native frontend (a winit event loop that must stay on the main thread)
    // and does the GPU render, readback, hashing and comparison. Frames are
    // compared in order; the digests do not depend on which thread computed
    // them. A stopped comparison drops the receiver and the simulation thread
    // ends at its next send.
    struct SimulatedCachedAvFrame {
        record: CachedOracleAvRecord,
        replay_input: u16,
        sample_frames: usize,
        rust_audio: Option<serde_json::Value>,
        capture: Option<gpu_capture::LiveGpuFrameCapture>,
        paired_boundary: Option<(u32, Box<ZeldaState>)>,
        compare: bool,
    }
    struct SimulationOutcome {
        game: ZeldaState,
        frames_completed: u32,
        first_rng_drift: Option<serde_json::Value>,
        receipt_nanos: u128,
        engine_nanos: u128,
        audio_nanos: u128,
        capture_nanos: u128,
    }
    let input_script = &input_script;
    let debug_wram_frames = &debug_wram_frames;
    let outcome = std::thread::scope(|scope| {
        let (sim_tx, sim_rx) = std::sync::mpsc::sync_channel::<SimulatedCachedAvFrame>(4);
        let simulation = std::thread::Builder::new()
            .name("cached-av-simulate".into())
            .spawn_scoped(scope, move || {
                let mut game = game;
                let mut audio_buffer = audio_buffer;
                let mut ledger_frames_seen = ledger_frames_seen;
                let mut frames_completed = frames_completed;
                let mut first_rng_drift = first_rng_drift;
                let mut paired_checkpoint_due = paired_checkpoint_due;
                // `ZELDA3_DEBUG_HOST_FEATURES=<csv>` writes one row per host: the
                // engine's state at the host boundary and the timing the receipt carried,
                // for rule discovery over the route (docs/parity/romless-exact-play.md).
                let mut host_feature_writer = std::env::var_os("ZELDA3_DEBUG_HOST_FEATURES").map(|path| {
                    let mut writer = BufWriter::new(fs::File::create(&path).expect("create the host feature trace"));
                    write_host_feature_header(&mut writer);
                    writer
                });
                // `ZELDA3_CACHED_AV_NATIVE_TIMING=1`: do not install the cache's timing
                // receipts; measure how far the engine's own timing stays exact.
                let native_timing = std::env::var_os("ZELDA3_CACHED_AV_NATIVE_TIMING").is_some();
                let debug_dsp_trace_frames = debug_frame_selection_from_env(
                    "ZELDA3_DEBUG_DSP_TRACE_FRAMES", Some("ZELDA3_DEBUG_DSP_TRACE_FRAME"),
                );
                let mut receipt_nanos = 0_u128;
                let mut engine_nanos = 0_u128;
                let mut audio_nanos = 0_u128;
                let mut capture_nanos = 0_u128;
        for parsed in parsed_rx.iter() {
            let receipt_started = Instant::now();
            let ParsedCachedAvInput {
                line_index,
                record,
                timing_receipts,
            } = parsed.unwrap_or_else(|error| {
                eprintln!("{error}");
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
            if record.frame < start_frame {
                continue;
            }
            let Some(timing_receipts) = timing_receipts else {
                eprintln!(
                    "cached source receipt for frame {} was not decoded (parse thread expected it before the boundary {start_frame})",
                    record.frame
                );
                process::exit(2);
            };
            if !timing_receipts.matches_host_call(u64::from(record.frame), replay_input) {
                eprintln!(
                    "cached source receipt provenance mismatch at frame {}: host_call={} canonical_input={:04x} raw_input={replay_input:04x}",
                    record.frame,
                    timing_receipts.host_call(),
                    timing_receipts.input_state()
                );
                process::exit(2);
            }
            if stop_before_frame.is_some_and(|end| record.frame >= end) {
                break;
            }
            if timing_enabled {
                receipt_nanos += receipt_started.elapsed().as_nanos();
            }
            let engine_started = Instant::now();
            if let Some(writer) = host_feature_writer.as_mut() {
                write_host_feature_row(writer, &game, record.frame, &timing_receipts);
            }
            if native_timing {
                // The native frontier: the engine runs the route's inputs on
                // its own timing (the path live play takes) and the first
                // video or audio hash that differs from the cache is how
                // far native play is exact. The receipts stay parsed for
                // provenance and are not installed.
                drop(timing_receipts);
            } else {
                game.install_original_timing_host_receipts(timing_receipts)
                    .unwrap_or_else(|error| {
                        eprintln!(
                            "failed to install cached source receipt at frame {}: {error:?}",
                            record.frame
                        );
                        process::exit(1);
                    });
            }
            game.zelda_run_frame(replay_input as i32);
            if timing_enabled {
                engine_nanos += engine_started.elapsed().as_nanos();
            }
            let capture_started = Instant::now();
            let capture = compare_video.then(|| {
                gpu_capture::LiveGpuFrameCapture::from_game_at_comparison_frame(
                    &mut game,
                    record.frame,
                )
            });
            if timing_enabled {
                capture_nanos += capture_started.elapsed().as_nanos();
            }
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
            let debug_dsp_trace_frame = debug_dsp_trace_frames.contains(&record.frame);
            if debug_dsp_trace_frame {
                game.zelda_begin_spc_driver_instruction_trace();
            }
            let rust_event_frame = game.zelda_render_audio(&mut audio_buffer, sample_frames as i32, 2);
            game.zelda_discard_unused_audio_frames();
            if debug_dsp_trace_frame {
                let modern_audio_state = game.zelda_modern_audio_state();
                let receipt = serde_json::json!({
                    "comparison_frame": record.frame,
                    "native_timing": native_timing,
                    "rust_spc_instruction_trace": game.zelda_take_spc_driver_instruction_trace(),
                    "rust_audio_event_frame": rust_event_frame,
                    "rust_audio": audio_buffer,
                    "rust_voice_samples": modern_audio_state.1.debug_voice_samples(),
                    "rust_voice_gains": modern_audio_state.1.debug_voice_gains(),
                    "rust_voice_positions": modern_audio_state.1.debug_voice_positions(),
                    "rust_dsp_global_counter": modern_audio_state.1.debug_dsp_global_counter(),
                    "rust_dsp_rendered_samples": modern_audio_state.1.debug_dsp_rendered_samples(),
                    "rust_voices_after": game.zelda_modern_audio_voice_debug_states(),
                });
                fs::write(
                    output.join(format!("dsp_trace_frame_{}.json", record.frame)),
                    serde_json::to_vec(&receipt).expect("serialize cached SPC trace"),
                ).expect("write cached SPC trace");
            }
            let rust_audio = compare_audio.then(|| canonical_audio_digest(&audio_buffer));
            if timing_enabled {
                audio_nanos += audio_started.elapsed().as_nanos();
            }
            frames_completed = frames_completed.saturating_add(1);
            // `ZELDA3_DEBUG_WRAM_FRAMES=a,b,lo-hi`: Rust WRAM after each listed
            // frame, written beside the run's ledgers, for phase comparisons
            // against oracle WRAM captures from a seeded live probe.
            if debug_wram_frames.contains(&record.frame) {
                fs::write(
                    output.join(format!("rust_wram_frame_{}.bin", record.frame)),
                    &game.ram[..],
                )
                .unwrap_or_else(|error| {
                    eprintln!("failed to write Rust WRAM capture: {error}");
                    process::exit(1);
                });
            }
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
            paired_checkpoint_due |=
                paired_checkpoint_interval.is_some_and(|interval| frames_completed % interval == 0);
            let paired_boundary = (paired_checkpoint_due
                && game.paired_resume_cpu_boundary_is_quiescent()
                && game.capture_original_timing_resume_checkpoint().is_ok())
            .then(|| {
                paired_checkpoint_due = false;
                (frames_completed, Box::new(game.clone()))
            });
            let simulated = SimulatedCachedAvFrame {
                compare: record.frame >= compare_from_frame,
                record,
                replay_input,
                sample_frames,
                rust_audio,
                capture,
                paired_boundary,
            };
            if sim_tx.send(simulated).is_err() {
                break;
            }
        }
        // Dropping the receiver ends the producer at its next send; a parse error
        // it reported was already surfaced by the loop above.
        drop(parsed_rx);
        SimulationOutcome {
            game,
            frames_completed,
            first_rng_drift,
            receipt_nanos,
            engine_nanos,
            audio_nanos,
            capture_nanos,
        }
    })
    .unwrap_or_else(|error| {
        eprintln!("failed to start the cached A/V simulation thread: {error}");
        process::exit(2);
    });
    for simulated in sim_rx.iter() {
        let SimulatedCachedAvFrame {
            record,
            replay_input,
            sample_frames,
            rust_audio,
            capture,
            paired_boundary,
            compare,
        } = simulated;
        let rust_video = capture.map(|capture| {
            renderer
                .as_mut()
                .expect("video capture without a renderer")
                .queue_capture_video_digest(capture)
                .unwrap_or_else(|error| {
                    eprintln!(
                        "cached A/V video render submission failed at frame {}: {error}",
                        record.frame
                    );
                    process::exit(1);
                })
        });
        pending_frames.push_back(PendingCachedAvFrame {
            compare,
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
    drop(sim_rx);
    simulation.join().unwrap_or_else(|_| {
        eprintln!("cached A/V simulation thread panicked");
        process::exit(1);
    })
    });
    let _ = parse_thread.join();
    let SimulationOutcome {
        game,
        frames_completed,
        first_rng_drift,
        receipt_nanos,
        engine_nanos,
        audio_nanos,
        capture_nanos,
    } = outcome;
    if let Some(renderer) = renderer.as_mut() {
        renderer.add_capture_nanos(capture_nanos);
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
    // `ZELDA3_REPLAY_WRAM_DUMP=<path>`: the full 128KB Rust WRAM after the last
    // replayed frame, as in the live compare, for byte-diffing the cold cached
    // lineage against an oracle WRAM capture at the same frame.
    if let Some(path) = env::var_os("ZELDA3_REPLAY_WRAM_DUMP") {
        if let Err(error) = fs::write(&path, &game.ram[..]) {
            eprintln!("failed to write WRAM dump to {path:?}: {error}");
        }
    }
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
        "binary_sha256": std::env::current_exe()
            .ok()
            .and_then(|path| parity::evidence::sha256_file(&path).ok()),
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
    // A bare `--resume-rust-state`/`--resume-oracle-state` pair restores both
    // engines but would start the semantic adapter fresh, so the resume could
    // silently differ from the cold run its checkpoint came from (f1371214:
    // a stale NMI-resume target only the cold adapter carried). When the
    // paired capture's sidecars sit beside the oracle state, restore them too
    // so every resume is faithful to the cold adapter.
    if resume_paired.is_none() {
        if let Some(dir) = resume_oracle_state.as_deref().and_then(Path::parent) {
            let semantic = dir.join("semantic-trace.checkpoint.json");
            if resume_semantic_trace_checkpoint.is_none() && semantic.is_file() {
                eprintln!(
                    "restoring the paired Snes9x semantic trace checkpoint {}",
                    semantic.display()
                );
                resume_semantic_trace_checkpoint = Some(semantic);
            }
            let original_timing = dir.join("original-timing.resume.json");
            if resume_original_timing_checkpoint.is_none() && original_timing.is_file() {
                resume_original_timing_checkpoint = Some(original_timing);
            }
        }
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
        // SAFETY: the compare harness configures the trace core before any
        // worker thread reads the environment.
        unsafe { env::set_var("ZELDA3_SNES9X_TRACE", &path) };
        // Only the cartridge routine's final store is needed here. The broader
        // `rng` stream also records beam-counter reads and unrelated $0fa1
        // writes, producing tens of thousands of events for a few hundred
        // replay samples.
        // Live RNG authority and hardware chronology must be observable in the
        // same cold run. Preserve explicitly requested trace domains and add
        // the cartridge RNG store required by `LiveOracleRngTrace`.
        let trace_events =
            trace_events_with_rom_rng(env::var("ZELDA3_SNES9X_TRACE_EVENTS").ok().as_deref());
        // SAFETY: same single-threaded configuration window as above.
        unsafe { env::set_var("ZELDA3_SNES9X_TRACE_EVENTS", trace_events) };
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
    let mut initial_sram = game.sram.clone();
    // A paired resume restores both engines from progressed states, but the
    // provenance SRAM recorded into any checkpoint this run writes must stay
    // the route's initial SRAM: paired captures descending from a resume
    // otherwise record the progressed save and every cache-bound consumer
    // (`cached-av --resume-paired`) rejects them as a different route.
    if let Some(dir) = resume_oracle_state.as_deref().and_then(Path::parent) {
        let provenance = dir.join("initial.srm");
        if provenance.is_file() {
            if let Ok(bytes) = fs::read(&provenance) {
                initial_sram = bytes;
            }
        }
    }
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
    let mut oracle_before_state_frame = start_frame;
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
    // ZELDA3_DEBUG_FULL_STATE_DUMP=<dir>:<frame>[,<frame>...] writes the complete
    // translated engine state (`{:#?}`, including every serde-skipped field) to
    // `<dir>/state-<frame>.txt` before running each listed host. Diffing a cold
    // dump against a resumed dump at the same host names the transient state a
    // paired checkpoint restore lost.
    let full_state_dump = env::var("ZELDA3_DEBUG_FULL_STATE_DUMP")
        .ok()
        .and_then(|raw| {
            let (dir, frames) = raw.split_once(':')?;
            let frames: Vec<u32> = frames
                .split(',')
                .filter_map(|part| part.trim().parse::<u32>().ok())
                .collect();
            Some((PathBuf::from(dir), frames))
        });
    for frame_index in start_frame..frames {
        if let Some((dir, dump_frames)) = full_state_dump.as_ref() {
            if dump_frames.contains(&frame_index) {
                fs::create_dir_all(dir).ok();
                let path = dir.join(format!("state-{frame_index}.txt"));
                fs::write(&path, format!("{game:#?}")).unwrap_or_else(|error| {
                    eprintln!(
                        "failed to write full state dump {}: {error}",
                        path.display()
                    );
                });
                eprintln!(
                    "wrote full engine state dump for host {frame_index}: {}",
                    path.display()
                );
            }
        }
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
                oracle_before_state_frame = frame_index;
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
                    trace.take_host_dialogue_scroll_progress(),
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
                oracle_before_state_frame = frame_index;
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

/// Failure artifacts accumulate ~5-10MB per diverging run; keep only the most
/// recent runs instead of growing target/parity-failures without bound.
pub(crate) const PARITY_FAILURE_DIRS_KEPT: usize = 20;

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

fn snes9x_original_timing_host_receipts(
    oracle: &LibretroCore,
    capture: &LibretroFrame,
    previous_video: Option<&PresentedOracleVideo>,
    presented_bg_tilemap_cache: &mut PresentedBgTilemapCache,
    frame: u32,
    input: u16,
    semantic: Vec<OriginalTimingSemanticReceipt>,
    nmi_acceptance_ppu_register_operands: Vec<NmiPpuRegisterOperands>,
    dialogue_scroll_progress: Vec<zelda3::DialogueScrollProgressReceipt>,
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
        .with_song_end_poll_native_sample_offsets(song_end_poll_native_sample_offsets)
        .with_dialogue_scroll_progress(dialogue_scroll_progress);
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

const FIRST_NMI_DMA_SETUP_INITIAL_SRAM_SHA256: &str =
    "d8a02e6e08a22377f919c7350e5bfa6117c6408db9a728815af7ad6e4e6b83bc";
const FIRST_NMI_DMA_SETUP_INITIAL_SRAM_BYTES: usize = 8_192;
const FIRST_NMI_DMA_SETUP_VALID_SLOT_MARKER_OFFSET: usize = 0x03e5;

// Topical split of this module (see snes9x_compare/*.rs).
mod cached_av;
pub(crate) use cached_av::*;
mod display_oracle;
pub(crate) use display_oracle::*;
mod libretro_session;
pub(crate) use libretro_session::*;
mod paired_resume;
pub(crate) use paired_resume::*;
mod replay_bundle;
pub(crate) use replay_bundle::*;
mod rng_trace;
pub(crate) use rng_trace::*;
mod smp_trace;
pub(crate) use smp_trace::*;
mod video_compare;
pub(crate) use video_compare::*;

#[cfg(test)]
pub(crate) mod tests;

const HOST_FEATURE_COLUMNS: &[&str] = &[
    "frame",
    "main_module",
    "submodule",
    "subsubmodule",
    "indoors",
    "link_state",
    "dungeon_room",
    "overworld_screen",
    "frame_counter_odd",
    "nmi_boolean",
    "nmi_subroutine",
    "inidisp",
    "bg_from_vram",
    "cgram_update",
    "active_sprites",
    "sprites_state9",
    "sprites_state_low",
    "ancilla_active",
    "vram_upload_offset",
    "prev_timing",
    "host_timing",
];

fn write_host_feature_header(writer: &mut impl Write) {
    writeln!(writer, "{}", HOST_FEATURE_COLUMNS.join(",")).expect("write the host feature header");
}

thread_local! {
    static PREVIOUS_HOST_TIMING: std::cell::Cell<&'static str> = const { std::cell::Cell::new("open") };
}

/// One row: the engine state the host begins from and the timing class the
/// oracle receipt carried for it (interrupted > continued > held > open).
fn write_host_feature_row(
    writer: &mut impl Write,
    game: &ZeldaState,
    frame: u32,
    receipts: &zelda3::OriginalTimingHostReceipts,
) {
    use zelda3::OriginalTimingSemanticReceipt as R;
    let ram = &game.ram;
    let mut held = false;
    let mut interrupted = false;
    let mut continued = false;
    for receipt in receipts.semantic() {
        match receipt {
            R::NmiAccepted(gate) => held |= format!("{gate:?}").contains("Held"),
            R::MainLoopInterrupted(_) => interrupted = true,
            R::MainLoopProgress(zelda3::MainLoopProgress::CallStackContinued) => continued = true,
            _ => {}
        }
    }
    let timing = if interrupted {
        "interrupted"
    } else if continued {
        "continued"
    } else if held {
        "held"
    } else {
        "open"
    };
    let previous = PREVIOUS_HOST_TIMING.with(|cell| cell.replace(timing));
    let sprite_states = &ram[0xdd0..0xde0];
    let active_sprites = sprite_states.iter().filter(|&&state| state != 0).count();
    let sprites_state9 = sprite_states.iter().filter(|&&state| state == 9).count();
    let sprites_state_low = sprite_states.iter().filter(|&&state| (1..9).contains(&state)).count();
    let ancilla_active = ram[0xc4a..0xc54].iter().filter(|&&kind| kind != 0).count();
    let word = |address: usize| u16::from_le_bytes([ram[address], ram[address + 1]]);
    writeln!(
        writer,
        "{frame},{},{},{},{},{},{},{},{},{},{},{},{},{},{active_sprites},{sprites_state9},{sprites_state_low},{ancilla_active},{},{previous},{timing}",
        ram[0x10],
        ram[0x11],
        ram[0xb0],
        ram[0x1b],
        ram[0x5d],
        word(0xa0),
        word(0x8a),
        ram[0x1a] & 1,
        ram[0x12],
        ram[0x17],
        ram[0x13],
        ram[0x14],
        ram[0x15],
        word(0x1000),
    )
    .expect("write a host feature row");
}
