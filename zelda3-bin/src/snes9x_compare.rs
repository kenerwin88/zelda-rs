//! The Snes9x oracle comparison harness: scripted exact video+audio
//! comparison, replay validation, session receipts, and failure artifacts.

use crate::*;

use std::env;
use std::error::Error;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::audio_trace::AudioFrameStats;
use crate::gpu_capture::NativeWindowOracleRenderer;
use crate::image_output::{write_argb_frame_png, write_rgba_frame_png};
use crate::input_script::InputScript;
use crate::libretro_timeline::{
    format_input_history, AudioComparisonMode, AudioTimingOptions, StreamingAudioComparator,
};
use crate::render_diagnostics::format_render_ppu_summary;
use serde::Serialize;
use zelda3::{game_output::DspWriteEvent, ZeldaState, RUN_MAIN};

pub(crate) const ORACLE_MUSIC_CONTROL: usize = 0x012c;
pub(crate) const ORACLE_QUEUED_MUSIC_CONTROL: usize = 0x0132;
pub(crate) const ORACLE_LAST_MUSIC_CONTROL: usize = 0x0133;

/// Display-domain receipt captured from the Snes9x PPU and the immutable Rust
/// scanout snapshot. This is deliberately upstream of RGBA comparison: it
/// tells us whether a failure is a ROM/state-publication issue or a renderer
/// issue before any pixel-level investigation begins.
#[derive(Debug, Serialize)]
struct DisplayOracleReceipt {
    frame: u32,
    stage: &'static str,
    oracle: DisplayPpuProbe,
    rust: DisplayPpuProbe,
}

#[derive(Debug, Serialize)]
struct DisplayPpuProbe {
    mode: i32,
    brightness: i32,
    mode7_scanout_brightness_override: Option<u8>,
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
}

fn capture_oracle_ppu_probe(oracle: &LibretroCore) -> Option<DisplayPpuProbe> {
    oracle.debug_ppu_value(0, 0)?;
    Some(DisplayPpuProbe {
        mode: oracle.debug_ppu_value(0, 0)?,
        brightness: oracle.debug_ppu_value(1, 0)?,
        mode7_scanout_brightness_override: None,
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
    })
}

fn capture_rust_ppu_probe(game: &mut ZeldaState) -> DisplayPpuProbe {
    let live_oam = game
        .ppu
        .oam
        .iter()
        .flat_map(|word| word.to_le_bytes().map(i32::from))
        .collect();
    game.with_display_snapshot(move |snapshot| {
        let scanlines = snapshot.ppu_scanline_windows();
        DisplayPpuProbe {
            mode: i32::from(snapshot.ppu.mode),
            brightness: i32::from(snapshot.ppu.brightness),
            mode7_scanout_brightness_override: snapshot.ppu.mode7_scanout_brightness_override,
            forced_blank: snapshot.ppu.forced_blank,
            brightness_white: i32::from(
                snapshot.ppu.brightness_mult.get(31).copied().unwrap_or(0) >> 3,
            ),
            cgram: snapshot
                .ppu
                .cgram
                .iter()
                .map(|&value| i32::from(value))
                .collect(),
            fixed_color: [
                i32::from(snapshot.ppu.fixed_color_r),
                i32::from(snapshot.ppu.fixed_color_g),
                i32::from(snapshot.ppu.fixed_color_b),
            ],
            display_control: [
                i32::from(snapshot.ppu.screen_enabled[0]),
                i32::from(snapshot.ppu.screen_enabled[1]),
                i32::from(snapshot.ppu.screen_windowed[0]),
                i32::from(snapshot.ppu.screen_windowed[1]),
                i32::from(
                    u8::from(snapshot.ppu.add_subscreen) << 1
                        | snapshot.ppu.prevent_math_mode << 4
                        | snapshot.ppu.clip_mode << 6,
                ),
                i32::from(
                    snapshot.ppu.math_enabled
                        | u8::from(snapshot.ppu.half_color) << 6
                        | u8::from(snapshot.ppu.subtract_color) << 7,
                ),
            ],
            bg_scroll: std::array::from_fn(|i| {
                let layer = &snapshot.ppu.bg_layer[i / 2];
                i32::from(if i % 2 == 0 {
                    layer.h_scroll
                } else {
                    layer.v_scroll
                })
            }),
            oam: live_oam,
            presented_oam: snapshot
                .ppu
                .oam
                .iter()
                .flat_map(|word| word.to_le_bytes().map(i32::from))
                .collect(),
            mode7: snapshot.ppu.m7_matrix.map(i32::from),
            mode7_scanlines: scanlines.iter().map(|line| line.7.map(i32::from)).collect(),
        }
    })
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
    let receipt = DisplayOracleReceipt {
        frame,
        stage,
        oracle: oracle_ppu,
        rust: capture_rust_ppu_probe(game),
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

pub(crate) fn run_compare_libretro_oracle(
    args: &[String],
    default_oracle_name: Option<&str>,
    required_library_name: Option<&str>,
) {
    let operation = "--compare-snes9x-oracle";
    let core_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 {operation} <path-to-snes-libretro.dylib> <path-to-rom.sfc> [frames] [--replay-save <path>] [--input-script <path>] [--rom-random-script <path>] [--load-sram <path>] [--resume-rust-state <path> --resume-oracle-state <path> [--resume-oracle-sram <path>]] [--native-apu-bootstrap <path>] [--ignore-video] [--ignore-audio] [--compare-from-frame <n>] [--skip-oracle-frames <n>] [--audio-comparison timing|exact] [--session-dir <path>] [--scan-all]"
            );
            process::exit(2);
        }
    };
    let rom_path = match args.get(1) {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 {operation} <path-to-snes-libretro.dylib> <path-to-rom.sfc> [frames] [--replay-save <path>] [--input-script <path>] [--rom-random-script <path>] [--load-sram <path>] [--resume-rust-state <path> --resume-oracle-state <path> [--resume-oracle-sram <path>]] [--native-apu-bootstrap <path>] [--ignore-video] [--ignore-audio] [--compare-from-frame <n>] [--skip-oracle-frames <n>] [--audio-comparison timing|exact] [--session-dir <path>] [--scan-all]"
            );
            process::exit(2);
        }
    };
    let mut frames = 300u32;
    let mut input_script = InputScript::default();
    let mut rom_random_script = None::<PathBuf>;
    let mut replay_save = None::<PathBuf>;
    let mut load_sram = None::<PathBuf>;
    let mut resume_rust_state = None::<PathBuf>;
    let mut resume_oracle_state = None::<PathBuf>;
    let mut resume_oracle_sram = None::<PathBuf>;
    let mut native_apu_bootstrap = None::<PathBuf>;
    let mut compare_video = true;
    let mut compare_audio = true;
    let mut compare_from_frame = 0u32;
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
    if resume_rust_state.is_some() != resume_oracle_state.is_some() {
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
    if let Err(error) =
        validate_libretro_comparison_scope(frames, compare_from_frame, compare_video, compare_audio)
    {
        eprintln!("{error}");
        process::exit(2);
    }
    if session_dir.is_some() {
        scan_all = true;
    }
    verify_expected_sha256(core_path, "libretro core", expected_core_sha256.as_deref());
    verify_expected_sha256(rom_path, "ROM", expected_rom_sha256.as_deref());
    let _compare_lock = acquire_snes9x_compare_lock();

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
        (game, checkpoint.host_frame)
    } else {
        // Compare the same extracted-asset-only state that plain `cargo run` uses.
        (load_default_play_state(), 0)
    };
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
        println!(
            "resumed Rust and {oracle_name} from paired pre-frame states at frame {start_frame}"
        );
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
    let mut oracle_before_state = initial_oracle_state.clone();
    let mut video_mismatch_ranges = Vec::<(u32, u32)>::new();
    let mut first_video_mismatch = None::<String>;
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
        skip_oracle_frames,
        compare_video,
        compare_audio,
        audio_comparison,
        timing_options,
        replay_save.as_deref(),
        rom_random_script.as_deref(),
    );
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
    let trace_poly_sched = std::env::var_os("TRACE_POLY_SCHED").is_some();
    let trace_shield_dma = std::env::var_os("ZELDA3_DEBUG_SHIELD_DMA").is_some();
    let debug_vram_frames = debug_frame_selection_from_env("ZELDA3_DEBUG_VRAM_FRAMES", None);
    let debug_video_frames = debug_frame_selection_from_env("ZELDA3_DEBUG_VIDEO_FRAMES", None);
    let debug_text_frames = debug_frame_selection_from_env("ZELDA3_DEBUG_TEXT_FRAMES", None);
    let debug_sprite_frames = debug_frame_selection_from_env("ZELDA3_DEBUG_SPRITE_FRAMES", None);
    let debug_wram_frames =
        debug_frame_selection_from_env("ZELDA3_DEBUG_WRAM_FRAMES", Some("ZELDA3_DEBUG_WRAM_FRAME"));
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
    for frame_index in start_frame..frames {
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
        let pre_load_remaining_frames = game.zelda_debug_selected_game_load_remaining_frames();
        stage(0, &mut stage_ns, &mut stage_mark);
        let rust_poly_cycles: Option<u64> = None;
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
        input_history.push((frame_index, input));
        stage(2, &mut stage_ns, &mut stage_mark);
        let rust_video_frame =
            (compare_this_frame && (trace_video_pixel.is_some() || compare_video)).then(|| {
                let restored = if debug_anim_lag {
                    pre_anim_region.as_ref().map(|prev| {
                        let cur = game.ppu.vram[0x3c00..0x3e00].to_vec();
                        game.ppu.vram[0x3c00..0x3e00].copy_from_slice(prev);
                        cur
                    })
                } else {
                    None
                };
                let frame = native_window_video
                    .as_mut()
                    .expect("native window renderer allocated for libretro video comparison")
                    .render_game_rgba(&mut game)
                    .unwrap_or_else(|error| {
                        eprintln!("native-window oracle video render failed: {error}");
                        process::exit(1);
                    });
                if let Some(cur) = restored {
                    game.ppu.vram[0x3c00..0x3e00].copy_from_slice(&cur);
                }
                frame
            });
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
        oracle
            .serialize_state_into(&mut oracle_before_state)
            .unwrap_or_else(|e| {
                eprintln!("failed to serialize {oracle_name} before frame {frame_index}: {e}");
                process::exit(1);
            });
        let mut capture = oracle.run_frame_with_input(input);
        if let Some(writer) = display_oracle_receipts.as_mut().filter(|_| {
            capture_all_display_oracle || display_oracle_after_frames.contains(&frame_index)
        }) {
            write_display_oracle_receipt(writer, frame_index, "after", &oracle, &mut game);
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
            // slot 0: x 0xd10/0xd30(hi), y 0xd00/0xd20(hi), yvel 0xd40, xvel 0xd50,
            // dir 0xde0, state 0x0e20, ai 0x0e60. Plus tagalong buffer head 0x0ec0.
            let f = |a: usize| (g(a), o(a), if g(a) != o(a) { "*" } else { "" });
            eprintln!(
                "sprite_probe frame={frame_index} xlo={:?} xhi={:?} ylo={:?} yhi={:?} yvel={:?} dir(0xde0)={:?} state(0xe20)={:?} ai(0xe60)={:?} main={:02x}/{:02x} sub={:02x}/{:02x}",
                f(0xd10), f(0xd30), f(0xd00), f(0xd20), f(0xd40), f(0xde0), f(0xe20), f(0xe60),
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
        let debug_dsp_trace_frame = std::env::var("ZELDA3_DEBUG_DSP_TRACE_FRAME")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            == Some(frame_index);
        if debug_dsp_trace_frame {
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
        let rust_stats = AudioFrameStats::from_interleaved_stereo(&rust_audio);
        let oracle_stats = AudioFrameStats::from_interleaved_stereo(&capture.audio);
        if debug_dsp_trace_frame {
            if let Some(dir) = session_dir.as_deref() {
                let rust_spc_instruction_trace = game.zelda_take_spc_driver_instruction_trace();
                let modern_audio_state = game.zelda_modern_audio_state();
                let receipt = serde_json::json!({
                    "oracle_trace": oracle.debug_dsp_trace(),
                    "oracle_dsp_samples": oracle.debug_dsp_samples(),
                    "oracle_dsp_register_writes": oracle.debug_dsp_register_writes(),
                    "oracle_apu_port_writes": oracle.debug_apu_port_writes(),
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
        }
        stage(5, &mut stage_ns, &mut stage_mark);
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
            pre_load_remaining_frames,
            game.zelda_debug_selected_game_load_remaining_frames(),
            game.debug_last_poly_work(),
            rust_poly_cycles,
            game.zelda_modern_audio_sfx_clock_checkpoint(),
            game.zelda_spc_driver_clock_debug_summary(),
            &rust_event_frame,
            oracle.memory_bytes(RETRO_MEMORY_SYSTEM_RAM),
        );
        stage(6, &mut stage_ns, &mut stage_mark);
        if stage_timing && frame_index % 2000 == 1999 {
            let total: u128 = stage_ns.iter().sum();
            eprintln!(
                "snes9x_timing frames={} total_ms={} pre_state_ms={} poly_ms={} run_frame_ms={} video_ms={} oracle_ms={} audio_ms={} receipts_ms={}",
                frame_index + 1,
                total / 1_000_000,
                stage_ns[0] / 1_000_000,
                stage_ns[1] / 1_000_000,
                stage_ns[2] / 1_000_000,
                stage_ns[3] / 1_000_000,
                stage_ns[4] / 1_000_000,
                stage_ns[5] / 1_000_000,
                stage_ns[6] / 1_000_000,
            );
        }
        if let Some((x, y)) = trace_video_pixel.filter(|_| compare_this_frame) {
            let (displayed_ppu, rust_bg_pal4, rust_obj_pal) =
                game.with_display_snapshot(|snapshot| {
                    (
                        crate::render_diagnostics::format_render_ppu_summary(snapshot),
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
            let oracle_bg_pal4 = (0x40..=0x4f)
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
                .as_deref()
                .and_then(|frame| rgba_pixel_at(frame, rust_offset))
                .unwrap_or([0; 4]);
            let oracle_pixel = snes9x_rgba_pixel_at(&capture, snes9x_offset).unwrap_or([0; 4]);
            println!(
                "pixel frame={frame_index} xy=({x},{y}) rust={rust_pixel:02x?} {oracle_name}={oracle_pixel:02x?} main={:02x} sub={:02x} subsub={:02x} inidisp={:02x} rust_bg_pal4=[{rust_bg_pal4}] oracle_bg_pal4=[{oracle_bg_pal4}] rust_obj_pal90=[{rust_obj_pal}]",
                game.ram[0x10], game.ram[0x11], game.ram[0xb0], game.ram[0x13],
            );
            println!("pixel displayed_ppu frame={frame_index} {displayed_ppu}");
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
        if compare_this_frame && compare_video {
            let rust_video_frame = rust_video_frame
                .as_deref()
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
            if let Some(video_diff) = video_diff {
                append_u32_range(&mut video_mismatch_ranges, frame_index);
                if first_video_mismatch.is_none() {
                    first_video_mismatch = Some(video_diff.clone());
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
                    let artifact_dir = write_libretro_parity_failure_artifacts(
                        pre_game.as_ref(),
                        &game,
                        rust_video_frame,
                        &rust_audio,
                        &capture,
                        &oracle_before_state,
                        &oracle_after_state,
                        &input_history,
                        frame_index,
                        input,
                        oracle.av_info.timing.sample_rate.round() as u32,
                        oracle_name.as_str(),
                        oracle_system_ram.as_deref(),
                        Some(&oracle_before_vram),
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
                if !scan_all {
                    process::exit(1);
                }
            }
        }
    }

    if let Err(error) = game.finish_rom_random_replay() {
        eprintln!("ROM random replay did not complete: {error}");
        process::exit(1);
    }
    let audio_report = compare_audio.then(|| continuous_audio.finish());
    finalize_libretro_session(
        session_dir.as_deref(),
        frame_receipts.as_mut(),
        &input_history,
        audio_report.as_ref(),
        &audio_frame_ends,
        &oracle_before_state,
        &oracle,
        &game,
        frames,
        &video_mismatch_ranges,
        first_video_mismatch.as_deref(),
    );
    if let Some(report) = audio_report.as_ref().filter(|report| !report.matched) {
        let failing_frame = report
            .first_mismatch_sample_frame
            .map(|sample_frame| audio_frame_ends.partition_point(|&end| end <= sample_frame as u64))
            .map(|index| effective_compare_from_frame.saturating_add(index as u32));
        eprintln!(
            "{oracle_name} audio divergence{}: {}",
            failing_frame
                .map(|frame| format!(" at or before frame {frame}"))
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

    println!(
        "{oracle_name} oracle compare completed {frames} frame(s) with no enabled video/audio diff"
    );
}

pub(crate) fn oracle_name_from_core_path(core_path: &str) -> String {
    let stem = Path::new(core_path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("libretro");
    stem.strip_suffix("_libretro").unwrap_or(stem).to_string()
}

pub(crate) fn validate_libretro_comparison_scope(
    frames: u32,
    compare_from_frame: u32,
    compare_video: bool,
    compare_audio: bool,
) -> Result<(), String> {
    if frames == 0 {
        return Err("libretro parity requires at least one frame".to_string());
    }
    if compare_from_frame >= frames {
        return Err(format!(
            "--compare-from-frame {compare_from_frame} leaves no compared frames in a {frames}-frame route"
        ));
    }
    if !compare_video && !compare_audio {
        return Err("libretro parity requires video, audio, or both comparison lanes".to_string());
    }
    Ok(())
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
    skip_oracle_frames: u32,
    compare_video: bool,
    compare_audio: bool,
    audio_comparison: AudioComparisonMode,
    timing: AudioTimingOptions,
    replay_save: Option<&Path>,
    rom_random_script: Option<&Path>,
) -> Option<BufWriter<fs::File>> {
    let dir = session_dir?;
    fs::create_dir_all(dir).unwrap_or_else(|e| {
        eprintln!("failed to create libretro session {}: {e}", dir.display());
        process::exit(1);
    });
    for stale in [
        "input.txt",
        "audio_frame_ends.json",
        "audio_report.json",
        "first_audio_mismatch.json",
        "first_audio_mismatch_rust.wav",
        "first_audio_mismatch_oracle.wav",
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
    let manifest = serde_json::json!({
        "schema": 1,
        "status": "running",
        "core": {
            "path": core_path,
            "sha256": core_sha256,
            "library_name": oracle.library_name,
            "library_version": oracle.library_version,
            "libretro_api_version": oracle.api_version,
        },
        "rom": { "path": rom_path, "sha256": rom_sha256 },
        "replay_save": replay_save_manifest,
        "rom_random_replay": rom_random_manifest,
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
    let replay = format!(
        "#!/bin/sh\nset -eu\ncd {}\nZELDA3_ASSET_PACK={} cargo run -q -p zelda3-bin{} -- --compare-snes9x-oracle {} {} {} --expected-core-sha256 {} --expected-rom-sha256 {} --input-script {}{} {} --compare-from-frame {}{} --audio-comparison {} --audio-window-ms {} --audio-silence-threshold {} --audio-timing-tolerance-ms {} --audio-envelope-tolerance {} --session-dir {} --scan-all\n",
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
        initial_state_flags,
        compare_from_frame,
        lane_flags,
        audio_comparison.as_str(),
        (timing.window_frames as f64 / oracle.av_info.timing.sample_rate) * 1000.0,
        timing.silence_threshold,
        (timing.max_timing_error_frames as f64 / oracle.av_info.timing.sample_rate) * 1000.0,
        timing.max_envelope_error,
        shell_single_quote(&absolute_dir.join("replay").to_string_lossy()),
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
    rust_poly_work: zelda3::zelda_rtl::PolyWorkMetrics,
    rust_poly_cycles: Option<u64>,
    rust_sfx_clock_checkpoint: (u32, u8, u8),
    rust_spc_driver_clock: Option<String>,
    rust_audio_events: &zelda3::game_output::AudioEventFrame,
    oracle_system_ram: Option<&[u8]>,
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
    if let Some(map) = receipt.as_object_mut() {
        map.insert("bg_tile_animation_countdown".into(), word(0xc00d).into());
        map.insert("link_dma_source_offset".into(), word(0xc00f).into());
        map.insert("link_dma_countdown".into(), word(0xc013).into());
        map.insert("link_dma_tile_offset".into(), word(0xc015).into());
        map.insert("bg1_h_copy2".into(), word(0x00e0).into());
        map.insert("bg1_v_copy2".into(), word(0x00e6).into());
        map.insert("move_overlay_counter".into(), byte(0x0494).into());
    }

    let object = receipt
        .as_object_mut()
        .expect("engine receipt is an object");
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
        ("joypad_high", u64::from(byte(0x00f0))),
        ("joypad_low", u64::from(byte(0x00f2))),
        ("joypad_high_filtered", u64::from(byte(0x00f4))),
        ("joypad_low_filtered", u64::from(byte(0x00f6))),
        ("link_x", u64::from(word(0x0022))),
        ("link_y", u64::from(word(0x0020))),
        ("link_direction", u64::from(byte(0x0067))),
        ("link_facing_direction", u64::from(byte(0x002f))),
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
    input_history: &[(u32, u16)],
    audio_report: Option<&libretro_timeline::AudioComparisonReport>,
    audio_frame_ends: &[u64],
    oracle_last_before: &[u8],
    oracle: &LibretroCore,
    game: &ZeldaState,
    frames: u32,
    video_mismatch_ranges: &[(u32, u32)],
    first_video_mismatch: Option<&str>,
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
    fs::write(dir.join("input.txt"), format_input_history(input_history)).unwrap_or_else(|e| {
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
    let matched = audio_report.map(|report| report.matched).unwrap_or(true)
        && video_mismatch_ranges.is_empty();
    let parity_eligible = audio_report
        .map(|report| report.mode == AudioComparisonMode::Exact.as_str())
        .unwrap_or(true);
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
    fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap_or_else(
        |error| {
            eprintln!("failed to finalize libretro session manifest: {error}");
            process::exit(1);
        },
    );
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
    let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if rc != 0 {
        eprintln!(
            "another Snes9x comparison is already running (lock: {}); GPU comparisons must run serially",
            path.display()
        );
        process::exit(2);
    }
    file
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
    input_history: &[(u32, u16)],
    frame: u32,
    input: u16,
    sample_rate: u32,
    oracle_name: &str,
    oracle_system_ram: Option<&[u8]>,
    oracle_before_vram: Option<&[u8]>,
    message: String,
) -> Result<PathBuf, Box<dyn Error>> {
    let dir = create_parity_failure_dir()?;
    fs::write(dir.join("input.txt"), format_input_history(input_history))?;
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
    let mut visible_game = post_game.clone();
    let (visible_ppu_summary, visible_vram, visible_oam, visible_cgram) = visible_game
        .with_display_snapshot(|display| {
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
        });
    fs::write(dir.join("rust_visible_vram.bin"), visible_vram)?;
    fs::write(dir.join("rust_visible_oam.bin"), visible_oam)?;
    fs::write(dir.join("rust_visible_cgram.bin"), visible_cgram)?;

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
        let mismatched_bytes = rust_vram
            .iter()
            .zip(oracle_vram)
            .filter(|(rust, oracle)| rust != oracle)
            .count()
            + rust_vram.len().abs_diff(oracle_vram.len());
        let first_mismatch_byte = rust_vram
            .iter()
            .zip(oracle_vram)
            .position(|(rust, oracle)| rust != oracle);
        fs::write(
            dir.join("vram_diff.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "rust_bytes": rust_vram.len(),
                "oracle_bytes": oracle_vram.len(),
                "mismatched_bytes": mismatched_bytes,
                "first_mismatch_byte": first_mismatch_byte,
                "first_mismatch_word": first_mismatch_byte.map(|offset| offset / 2),
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
            "oracle_before_vram.bin".to_string(),
            "rust_after_vram.bin".to_string(),
            "oracle_after_vram.bin".to_string(),
            "rust_after_ram.bin".to_string(),
            "oracle_after_ram.bin".to_string(),
            "rust_visible_vram.bin".to_string(),
            "rust_visible_oam.bin".to_string(),
            "rust_visible_cgram.bin".to_string(),
            "vram_diff.json".to_string(),
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

#[cfg(test)]
mod tests {
    use super::{parse_debug_frame_selection, BootBoundaryState};

    #[test]
    fn parse_debug_frame_selection_expands_and_deduplicates_ranges() {
        assert_eq!(
            parse_debug_frame_selection("81,79-81,84..=85,invalid,8-3"),
            vec![79, 80, 81, 84, 85]
        );
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
}
