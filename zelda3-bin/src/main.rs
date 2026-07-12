//! zelda3-rs prototype binary.
//!
//! Default execution runs the native playable host: load ROM/assets/SRAM, step
//! `ZeldaState`, present PPU pixels, queue audio, and save SRAM on quit.
//! `--lockstep` keeps the C-oracle comparison path available for parity work,
//! while `--headless` preserves the raw opcode-budget emulator harness.

mod asset_palette_commands;
mod asset_source_dump_commands;
mod audio_trace;
mod classic_frame_renderer;
mod developer_destinations;
mod developer_modern_map;
mod developer_room_commands;
mod frame_dump_commands;
mod gpu_capture;
mod gpu_compare;
mod gpu_readback;
mod hd_authoring_commands;
mod image_output;
mod index_dump_commands;
mod index_source_keys;
mod input_script;
mod overworld_dump_commands;
mod play_commands;
mod play_renderer;
mod render_diagnostics;
mod replay_diagnostics;
mod replay_save_config;
mod route_coverage_commands;
mod sheet_dump_commands;

use std::backtrace::Backtrace;
use std::env;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fs;
use std::io::{BufWriter, Write};
use std::os::raw::{c_char, c_uint, c_void};
use std::panic::{self, AssertUnwindSafe, PanicHookInfo};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use asset_palette_commands::run_dump_reference_palette;
use asset_source_dump_commands::run_dump_assets_by_source;
use audio_trace::{
    fingerprint_audio_hash, first_peak_frame, max_peak_frame, print_audio_window,
    print_replay_audio_trace, replay_checksum_dsp_write_values, replay_checksum_dsp_writes,
    replay_checksum_samples, should_write_fingerprint, AudioFrameStats,
};
use developer_room_commands::{run_dump_developer_destination, run_dump_developer_tileset};
use frame_dump_commands::{
    run_dump_frame, run_dump_overworld_screen, run_dump_replay_checkpoint_ppu,
    run_scan_replay_checkpoints, run_smoke_asset_gpu,
};
use gpu_capture::{render_live_game_gpu_frame_rgba, ModernAssetGpuReadbackRenderer};
use gpu_compare::{
    replay_cpu_bgra_hash_line, replay_optional_gpu_readback_renderer,
    run_play_default_gpu_pixel_parity, run_play_gpu_render_compare,
};
use hd_authoring_commands::{run_dump_hd_capture, run_slice_hd_cells};
use image_output::{write_argb_frame_png, write_rgba_frame_png};
use index_dump_commands::{run_dump_dungeon_index_tiles, run_dump_sprite_index_tiles};
use input_script::InputScript;
use overworld_dump_commands::{run_dump_unique_overworld_cells, run_dump_unique_overworld_tiles};
use platform::NativeFrontendOptions;
use play_commands::{run_frontend_smoke, run_play, run_standalone_play};
use render_diagnostics::{
    compare_diagnostic_oracle_render_frame, format_render_ppu_summary,
    render_diagnostic_lockstep_oracle_frames_in_place, replay_fingerprint_leaf_bgra,
    replay_projection_bgra,
};
use replay_diagnostics::{
    replay_checksum_bytes, replay_checksum_ram_range, replay_save_ancilla_dump,
    replay_save_door_dump, replay_save_dungeon_attr_dump, replay_save_dungmap_dump,
    replay_save_garnish_dump, replay_save_message_dump, replay_save_overlord_dump,
    replay_save_palette_dump, replay_save_ram0000_dump, replay_save_ram0400_dump,
    replay_save_ram_page_dump, replay_save_requested_ram_page_dump, replay_save_room_history_dump,
    replay_save_room_mask, replay_save_room_mask_dump, replay_save_sprite_dump,
    replay_sram_checksum_ok,
};
use replay_save_config::{parse_replay_save_args_or_exit, ReplaySaveConfig};
use route_coverage_commands::{
    route_coverage_frame_from_game, run_coverage_probe, write_route_coverage_log_or_exit,
};
use serde::{Deserialize, Serialize};
use sheet_dump_commands::{run_dump_dungeon_sheet_png, run_dump_sprite_sheet_png};
use snes::{consts::PPU_EXTRA_LEFT_RIGHT, cpu_run_opcode, load_rom, Snes};
use zelda3::{
    config::parse_config_file_context, game_output::DspWriteEvent, LockstepOracle, OracleError,
    ZeldaState, RUN_MAIN, RUN_POLY,
};

const LOCKSTEP_CHECKPOINT_MAGIC: &[u8; 8] = b"Z3RSLS01";
const PLAY_CRASH_CHECKPOINT_MAGIC: &[u8; 8] = b"Z3RSPC01";
const APU_BOOTSTRAP_CHECKPOINT_MAGIC: &[u8; 8] = b"Z3RSAPU1";
const RETRO_MEMORY_SAVE_RAM: c_uint = 0;
const RETRO_MEMORY_RTC: c_uint = 1;
const RETRO_MEMORY_SYSTEM_RAM: c_uint = 2;
const RETRO_MEMORY_VIDEO_RAM: c_uint = 3;
const ACTION_TILE_X: [i16; 4] = [7, 7, -3, 16];
const ACTION_TILE_Y: [i16; 4] = [6, 24, 12, 12];
pub(crate) const TRACE_MAIN_MODULE_INDEX: usize = 0x10;
pub(crate) const TRACE_SUBMODULE_INDEX: usize = 0x11;
pub(crate) const TRACE_SUBSUBMODULE_INDEX: usize = 0xb0;
pub(crate) const TRACE_JOYPAD1H_LAST: usize = 0x0f0;
pub(crate) const TRACE_JOYPAD1L_LAST: usize = 0x0f2;
pub(crate) const TRACE_FILTERED_JOYPAD_H: usize = 0x0f4;
pub(crate) const TRACE_FILTERED_JOYPAD_L: usize = 0x0f6;
pub(crate) const TRACE_SELECTFILE_VAR3: usize = 0x0b10;
pub(crate) const TRACE_SELECTFILE_VAR7: usize = 0x0b11;
pub(crate) const TRACE_SELECTFILE_VAR9: usize = 0x0b13;
pub(crate) const TRACE_SELECTFILE_VAR11: usize = 0x0b14;
pub(crate) const TRACE_SELECTFILE_VAR5: usize = 0x0b15;
pub(crate) const TRACE_SELECTFILE_VAR10: usize = 0x0b16;
pub(crate) const TRACE_SELECTFILE_ARR2_1: usize = 0x0cb;
const PLAYER_IS_INDOORS: usize = 0x001b;
const EMBEDDED_ASSETS: &[u8] = include_bytes!(env!("ZELDA3_EMBEDDED_ASSETS"));

fn main() {
    let args: Vec<String> = env::args().collect();
    if dispatch_rom_first_oracle_flags(&args) {
        return;
    }
    if args.get(1).map(String::as_str) == Some("--lockstep") {
        run_lockstep(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--headless") {
        run_headless(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--standalone-smoke") {
        run_standalone_smoke(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--sram-smoke") {
        run_sram_smoke();
        return;
    }
    if args.get(1).map(String::as_str) == Some("--frontend-smoke") {
        run_frontend_smoke(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--smoke-render") {
        run_smoke_render(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--smoke-asset-gpu") {
        run_smoke_asset_gpu(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--trace-startup-audio") {
        run_trace_startup_audio(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--trace-bsnes-audio") {
        run_trace_bsnes_audio(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--compare-bsnes-startup-audio") {
        run_compare_bsnes_startup_audio(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--compare-bsnes-oracle") {
        run_compare_bsnes_oracle(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--compare-libretro-oracle") {
        run_compare_libretro_oracle(&args[2..], None);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--dump-bsnes-frame") {
        run_dump_bsnes_frame(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--trace-bsnes-memory") {
        run_trace_bsnes_memory(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--compare-startup-apu-impls") {
        require_audio_oracle("--compare-startup-apu-impls");
        run_compare_startup_apu_impls(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--trace-song-bank") {
        require_audio_oracle("--trace-song-bank");
        run_trace_song_bank(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--trace-rom-apu-upload") {
        run_trace_rom_apu_upload(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--capture-rom-apu-bootstrap") {
        run_capture_rom_apu_bootstrap(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--compare-bootstrap-apu-startup") {
        require_audio_oracle("--compare-bootstrap-apu-startup");
        run_compare_bootstrap_apu_startup(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--trace-bootstrap-apu-direct-frame") {
        run_trace_bootstrap_apu_direct_frame(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--dump-frame") {
        run_dump_frame(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--dump-developer-destination") {
        run_dump_developer_destination(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--dump-overworld-screen") {
        run_dump_overworld_screen(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--scan-replay-checkpoints") {
        run_scan_replay_checkpoints(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--dump-replay-checkpoint-ppu") {
        run_dump_replay_checkpoint_ppu(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--dump-developer-tileset") {
        run_dump_developer_tileset(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--dump-unique-overworld-cells") {
        run_dump_unique_overworld_cells(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--dump-unique-overworld-tiles") {
        run_dump_unique_overworld_tiles(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--dump-dungeon-index-tiles") {
        run_dump_dungeon_index_tiles(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--dump-sprite-index-tiles") {
        run_dump_sprite_index_tiles(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--dump-sprite-sheet-png") {
        run_dump_sprite_sheet_png(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--dump-dungeon-sheet-png") {
        run_dump_dungeon_sheet_png(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--dump-assets-by-source") {
        run_dump_assets_by_source(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--dump-asset-pack") {
        run_dump_asset_pack(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--bless-chr") {
        run_bless_chr(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--dump-reference-palette") {
        run_dump_reference_palette(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--dump-hd-capture") {
        run_dump_hd_capture(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--slice-hd-cells") {
        run_slice_hd_cells(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--compare-lockstep-render") {
        run_compare_lockstep_render(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--play-gpu-render-compare") {
        run_play_gpu_render_compare(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--play-default-gpu-pixel-parity") {
        run_play_default_gpu_pixel_parity(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--play-lockstep") {
        run_play_lockstep(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--replay-crash") {
        run_replay_crash(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--replay-save") {
        run_replay_save(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--coverage-probe") {
        run_coverage_probe(&args[2..]);
        return;
    }
    if let Some(rom_path) = args.get(1) {
        run_play(rom_path);
    } else {
        run_standalone_play();
    }
}

#[cfg(feature = "audio-oracle")]
fn require_audio_oracle(_operation: &str) {}

#[cfg(not(feature = "audio-oracle"))]
fn require_audio_oracle(operation: &str) {
    eprintln!("{operation} requires an audio-oracle build; rebuild with --features audio-oracle");
    process::exit(2);
}

/// Write the embedded, source-authoritative asset pack to disk. This is the
/// canonical way to (re)generate the on-disk `zelda3_assets.dat` that the
/// replay/parity flows load via `find_asset_pack`. Since the pack is embedded
/// at build time (build.rs), the emitted file is byte-identical to what this
/// binary runs with — including the required `kDialogueSourceSemantic` sidecar
/// that older restool packs lack. Defaults to `zelda3_assets.dat` in the cwd.
fn run_dump_asset_pack(args: &[String]) {
    let out = args
        .first()
        .map(String::as_str)
        .unwrap_or("zelda3_assets.dat");
    if let Err(e) = fs::write(out, EMBEDDED_ASSETS) {
        eprintln!("failed to write asset pack {out}: {e}");
        process::exit(1);
    }
    println!("wrote {} bytes to {out}", EMBEDDED_ASSETS.len());
}

/// Regenerate the CHR parity lock from the tracked editable sheets. This is the
/// "accept my edits" step: after changing a sheet PNG, the build fails until the
/// lock is refreshed to match. Usage: `zelda3 --bless-chr [<sheets-dir> <lock-path>]`.
/// Defaults the sheets dir to the repo's `assets/chr` and the lock to
/// `<sheets-dir>/chr.sha1`.
fn run_bless_chr(args: &[String]) {
    let default_sheets_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zelda3-bin lives under the workspace root")
        .join("assets/chr");
    let sheets_dir = args
        .first()
        .map(std::path::PathBuf::from)
        .unwrap_or(default_sheets_dir);
    if !sheets_dir.is_dir() {
        eprintln!(
            "CHR sheets directory {} does not exist; pass a sheets dir or create the tracked assets/chr",
            sheets_dir.display()
        );
        process::exit(1);
    }
    let lock_path = args
        .get(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| sheets_dir.join("chr.sha1"));

    let sheets = zelda3_chr::read_sheets_dir(&sheets_dir).unwrap_or_else(|e| {
        eprintln!(
            "failed to read CHR sheets from {}: {e}",
            sheets_dir.display()
        );
        process::exit(1);
    });
    let lock = zelda3_chr::generate_sha_lock(&sheets);
    if let Err(e) = fs::write(&lock_path, &lock) {
        eprintln!("failed to write {}: {e}", lock_path.display());
        process::exit(1);
    }
    let block_count: usize = sheets.iter().map(|sheet| sheet.blocks.len()).sum();
    println!(
        "blessed {} CHR sheets ({block_count} blocks): {} -> {}",
        sheets.len(),
        sheets_dir.display(),
        lock_path.display()
    );
}

pub(crate) fn read_le_u16(bytes: &[u8], index: usize) -> u16 {
    u16::from_le_bytes([bytes[index], bytes[index + 1]])
}

pub(crate) fn write_le_u16(bytes: &mut [u8], index: usize, value: u16) {
    let [lo, hi] = value.to_le_bytes();
    bytes[index] = lo;
    bytes[index + 1] = hi;
}

fn parse_u16_auto(value: &str) -> Option<u16> {
    value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .map(|hex| u16::from_str_radix(hex, 16).ok())
        .unwrap_or_else(|| value.parse::<u16>().ok())
}

fn dispatch_rom_first_oracle_flags(args: &[String]) -> bool {
    let Some(rom_path) = args.get(1) else {
        return false;
    };
    if rom_path.starts_with("--") || args.len() <= 2 {
        return false;
    }

    let tail = &args[2..];
    let has_play_lockstep = tail.iter().any(|arg| arg == "--play-lockstep");
    let has_lockstep_render = tail.iter().any(|arg| arg == "--compare-lockstep-render");
    let has_bsnes = tail.iter().any(|arg| arg == "--compare-bsnes-oracle");
    let has_libretro = tail.iter().any(|arg| arg == "--compare-libretro-oracle");
    if !(has_play_lockstep || has_lockstep_render || has_bsnes || has_libretro) {
        return false;
    }

    if has_play_lockstep {
        let mut forwarded = vec![rom_path.clone()];
        let mut i = 0usize;
        while i < tail.len() {
            match tail[i].as_str() {
                "--play-lockstep" | "--compare-lockstep-render" => i += 1,
                flag if flag.starts_with("--") => {
                    forwarded.push(flag.to_string());
                    if matches!(flag, "--load-sram" | "--load-state") {
                        let Some(value) = tail.get(i + 1) else {
                            eprintln!("{flag} requires a path");
                            process::exit(2);
                        };
                        forwarded.push(value.clone());
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                value => {
                    forwarded.push(value.to_string());
                    i += 1;
                }
            }
        }
        run_play_lockstep(&forwarded);
        return true;
    }

    if has_bsnes || has_libretro {
        let mut forwarded = Vec::new();
        let mut passthrough = Vec::new();
        let mut i = 0usize;
        let oracle_flag = if has_libretro {
            "--compare-libretro-oracle"
        } else {
            "--compare-bsnes-oracle"
        };
        while i < tail.len() {
            match tail[i].as_str() {
                "--compare-bsnes-oracle" | "--compare-libretro-oracle" => {
                    let Some(core_path) = tail.get(i + 1) else {
                        eprintln!("{oracle_flag} requires a path to a SNES libretro core");
                        process::exit(2);
                    };
                    forwarded.push(core_path.clone());
                    i += 2;
                }
                value => {
                    passthrough.push(value.to_string());
                    i += 1;
                }
            }
        }
        if forwarded.is_empty() {
            eprintln!("{oracle_flag} requires a path to a SNES libretro core");
            process::exit(2);
        }
        forwarded.push(rom_path.clone());
        forwarded.extend(passthrough);
        if has_libretro {
            run_compare_libretro_oracle(&forwarded, None);
        } else {
            run_compare_bsnes_oracle(&forwarded);
        }
        return true;
    }

    eprintln!(
        "`--compare-lockstep-render` is a scripted mode, not a playable ROM-first flag. Use:\n  zelda3 --compare-lockstep-render {rom_path} [frames]\nor:\n  zelda3 {rom_path} --play-lockstep\nfor playable lockstep."
    );
    process::exit(2);
}

fn run_headless(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!("usage: zelda3 --headless <path-to-rom.sfc> [opcode-budget]");
            process::exit(2);
        }
    };
    let budget: u64 = args
        .get(1)
        .map(|s| s.parse().unwrap_or(10_000))
        .unwrap_or(10_000);

    let rom = match fs::read(rom_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("failed to read {rom_path}: {e}");
            process::exit(1);
        }
    };

    let mut snes = Snes::new();
    if let Err(e) = load_rom(&mut snes, &rom) {
        eprintln!("loader: {e}");
        process::exit(1);
    }
    snes.cpu_seed_reset_vector();

    println!(
        "loaded {} bytes; reset vector PC=${:04X} K={:02X}",
        rom.len(),
        snes.cpu.pc,
        snes.cpu.k
    );

    // Run the opcode budget. The standalone SNES emulator in this
    // codebase has no built-in frame timer (the C build only drives it
    // lockstep with the C reimplementation), so we drive raw opcodes
    // and dump a digest of WRAM at the end.
    for _ in 0..budget {
        let _ = cpu_run_opcode(&mut snes);
        if snes.cpu.stopped {
            break;
        }
    }

    let digest = wram_digest(&snes);
    println!(
        "after {} opcodes: PC=${:04X} K={:02X} A={:04X} X={:04X} Y={:04X}",
        budget, snes.cpu.pc, snes.cpu.k, snes.cpu.a, snes.cpu.x, snes.cpu.y
    );
    println!("WRAM fnv1a64 = {:016x}", digest);
}

fn run_standalone_smoke(args: &[String]) {
    let frames: u32 = args.first().map(|s| s.parse().unwrap_or(2)).unwrap_or(2);
    let mut game = load_embedded_play_state();
    game.sram.fill(0);
    let mut audio = vec![0i16; 735 * 2];

    for _ in 0..frames {
        game.zelda_run_frame(0);
        game.zelda_render_audio(&mut audio, 735, 2);
        game.zelda_discard_unused_audio_frames();
    }

    println!(
        "standalone smoke completed frames={frames} ram_fnv1a64={:016x} sram_fnv1a64={:016x}",
        fnv1a64(&game.ram),
        fnv1a64(&game.sram)
    );
}

fn run_sram_smoke() {
    if env::var_os("ZELDA3_SAVE_DIR").is_none() {
        eprintln!("--sram-smoke requires ZELDA3_SAVE_DIR so it cannot touch a real user save");
        process::exit(2);
    }

    let mut writer = ZeldaState::new();
    writer.sram[..8].copy_from_slice(b"Z3SRAMOK");
    writer.zelda_write_sram();

    let mut reader = ZeldaState::new();
    reader.zelda_read_sram();
    if &reader.sram[..8] != b"Z3SRAMOK" {
        eprintln!("sram smoke failed: read-back bytes did not match");
        process::exit(1);
    }

    println!("sram smoke completed");
}

fn run_trace_rom_apu_upload(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --trace-rom-apu-upload <path-to-rom.sfc> [opcode-budget] [apu-cycles-per-cpu-cycle]"
            );
            process::exit(2);
        }
    };
    let budget: u64 = args
        .get(1)
        .map(|s| s.parse().unwrap_or(1_000_000))
        .unwrap_or(1_000_000);
    let apu_cycles_per_cpu_cycle: f64 = args
        .get(2)
        .map(|s| s.parse().unwrap_or(0.286))
        .unwrap_or(0.286);

    let rom = match fs::read(rom_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("failed to read {rom_path}: {e}");
            process::exit(1);
        }
    };

    let mut snes = Snes::new();
    if let Err(e) = load_rom(&mut snes, &rom) {
        eprintln!("loader: {e}");
        process::exit(1);
    }
    snes.reset(true);
    snes.cpu_seed_reset_vector();

    println!(
        "raw ROM APU trace: rom={} bytes budget={} apu_cycles_per_cpu_cycle={:.3} reset PC=${:04X} K={:02X}",
        rom.len(),
        budget,
        apu_cycles_per_cpu_cycle,
        snes.cpu.pc,
        snes.cpu.k
    );
    print_rom_apu_trace_line(0, "start", &snes);

    let mut apu_cycle_accum = 0.0f64;
    let mut next_payload_milestone = 256usize;
    let mut last_nonzero_0800 = snes.apu.ram[0x0800..0x0900].iter().any(|&b| b != 0);
    let mut last_nonzero_0878 = snes.apu.ram[0x0878..0x08c0].iter().any(|&b| b != 0);
    let mut last_reset = (snes.apu.ram[0xfffe], snes.apu.ram[0xffff]);
    let mut last_in_ports = [
        snes.apu.in_ports[0],
        snes.apu.in_ports[1],
        snes.apu.in_ports[2],
        snes.apu.in_ports[3],
    ];
    let mut next_dsp_milestone = 1usize;

    for op in 1..=budget {
        let cycles = cpu_run_opcode(&mut snes);
        while snes.dma.dma_busy {
            snes.dma_do();
        }

        apu_cycle_accum += f64::from(cycles) * apu_cycles_per_cpu_cycle;
        let apu_cycles = apu_cycle_accum as u32;
        for _ in 0..apu_cycles {
            snes.apu.cycle();
        }
        apu_cycle_accum -= f64::from(apu_cycles);

        let nonzero_0800 = snes.apu.ram[0x0800..0x0900].iter().any(|&b| b != 0);
        let nonzero_0878 = snes.apu.ram[0x0878..0x08c0].iter().any(|&b| b != 0);
        let reset = (snes.apu.ram[0xfffe], snes.apu.ram[0xffff]);
        let in_ports = [
            snes.apu.in_ports[0],
            snes.apu.in_ports[1],
            snes.apu.in_ports[2],
            snes.apu.in_ports[3],
        ];
        let dsp_writes = snes.apu.dsp_write_history.len();
        let hit_dsp_milestone = dsp_writes >= next_dsp_milestone;
        while dsp_writes >= next_dsp_milestone {
            if next_dsp_milestone >= 256 {
                next_dsp_milestone = usize::MAX;
                break;
            }
            next_dsp_milestone *= 2;
        }
        let payload_nonzero = apu_payload_nonzero(&snes);
        let hit_payload_milestone = payload_nonzero >= next_payload_milestone;
        while payload_nonzero >= next_payload_milestone {
            next_payload_milestone += 256;
        }
        let command_phase = !snes.apu.rom_readable || dsp_writes != 0;

        let changed = nonzero_0800 != last_nonzero_0800
            || nonzero_0878 != last_nonzero_0878
            || reset != last_reset
            || (command_phase && in_ports != last_in_ports)
            || hit_dsp_milestone
            || hit_payload_milestone
            || snes.cpu.stopped;
        let periodic = op % 100_000 == 0;
        if changed || periodic {
            print_rom_apu_trace_line(op, if changed { "change" } else { "tick" }, &snes);
        }

        last_nonzero_0800 = nonzero_0800;
        last_nonzero_0878 = nonzero_0878;
        last_reset = reset;
        last_in_ports = in_ports;
        if snes.cpu.stopped {
            break;
        }
    }

    print_rom_apu_trace_line(budget, "end", &snes);
}

fn run_capture_rom_apu_bootstrap(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --capture-rom-apu-bootstrap <path-to-rom.sfc> <out.z3apu> [opcode-budget] [apu-cycles-per-cpu-cycle]"
            );
            process::exit(2);
        }
    };
    let out_path = match args.get(1) {
        Some(p) => Path::new(p),
        None => {
            eprintln!(
                "usage: zelda3 --capture-rom-apu-bootstrap <path-to-rom.sfc> <out.z3apu> [opcode-budget] [apu-cycles-per-cpu-cycle]"
            );
            process::exit(2);
        }
    };
    let budget: u64 = args
        .get(2)
        .map(|s| s.parse().unwrap_or(1_500_000))
        .unwrap_or(1_500_000);
    let apu_cycles_per_cpu_cycle: f64 = args
        .get(3)
        .map(|s| s.parse().unwrap_or(0.286))
        .unwrap_or(0.286);

    let (snes, opcodes) =
        match capture_raw_rom_apu_bootstrap(rom_path, budget, apu_cycles_per_cpu_cycle, true) {
            Ok(capture) => capture,
            Err(e) => {
                eprintln!("failed to capture raw ROM APU bootstrap: {e}");
                process::exit(1);
            }
        };
    let checkpoint = ApuBootstrapCheckpoint {
        magic: *APU_BOOTSTRAP_CHECKPOINT_MAGIC,
        opcodes,
        apu_cycles_per_cpu_cycle,
        cpu_k: snes.cpu.k,
        cpu_pc: snes.cpu.pc,
        spc_pc: snes.apu.spc.pc,
        rom_readable: snes.apu.rom_readable,
        payload_nonzero: apu_payload_nonzero(&snes),
        dsp_writes: snes.apu.dsp_write_history.len(),
        apu: snes.apu,
    };
    let bytes = match bincode::serialize(&checkpoint) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("failed to encode APU bootstrap checkpoint: {e}");
            process::exit(1);
        }
    };
    if let Err(e) = fs::write(out_path, bytes) {
        eprintln!("failed to write {}: {e}", out_path.display());
        process::exit(1);
    }
    println!(
        "saved APU bootstrap {}: opcodes={} cpu=${:02x}:{:04x} spc=${:04x} rom={} payload_nz={} dsp_writes={}",
        out_path.display(),
        checkpoint.opcodes,
        checkpoint.cpu_k,
        checkpoint.cpu_pc,
        checkpoint.spc_pc,
        checkpoint.rom_readable,
        checkpoint.payload_nonzero,
        checkpoint.dsp_writes,
    );
}

fn run_compare_bootstrap_apu_startup(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --compare-bootstrap-apu-startup <path-to-rom.sfc> <bootstrap.z3apu> [frames]"
            );
            process::exit(2);
        }
    };
    let bootstrap_path = match args.get(1) {
        Some(p) => Path::new(p),
        None => {
            eprintln!(
                "usage: zelda3 --compare-bootstrap-apu-startup <path-to-rom.sfc> <bootstrap.z3apu> [frames]"
            );
            process::exit(2);
        }
    };
    let frames: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(120);
    let checkpoint = match load_apu_bootstrap_checkpoint(bootstrap_path) {
        Ok(checkpoint) => checkpoint,
        Err(e) => {
            eprintln!(
                "failed to load APU bootstrap checkpoint {}: {e}",
                bootstrap_path.display()
            );
            process::exit(1);
        }
    };
    let mut game = load_play_state(rom_path);
    let mut full_apu = checkpoint.apu;
    full_apu.dsp.sample_offset = 0;
    full_apu.dsp_write_history.clear();

    let mut high_audio = vec![0i16; 735 * 2];
    let mut full_audio = vec![0i16; 735 * 2];
    let mut high_stats = Vec::with_capacity(frames as usize);
    let mut full_stats = Vec::with_capacity(frames as usize);
    let mut debug = Vec::with_capacity(frames as usize);

    println!(
        "bootstrap source: opcodes={} cpu=${:02x}:{:04x} spc=${:04x} rom={} payload_nz={} dsp_writes={}",
        checkpoint.opcodes,
        checkpoint.cpu_k,
        checkpoint.cpu_pc,
        checkpoint.spc_pc,
        checkpoint.rom_readable,
        checkpoint.payload_nonzero,
        checkpoint.dsp_writes,
    );

    for _ in 0..frames {
        game.zelda_run_frame(0);
        let ports = game.zelda_debug_apu_write_ports();
        for (port, &value) in ports.iter().enumerate() {
            full_apu.write_snes_port(port as u8, value);
        }
        game.zelda_render_audio(&mut high_audio, 735, 2);
        game.zelda_discard_unused_audio_frames();
        render_full_apu_audio(&mut full_apu, &mut full_audio, 735, 2);
        high_stats.push(AudioFrameStats::from_interleaved_stereo(&high_audio));
        full_stats.push(AudioFrameStats::from_interleaved_stereo(&full_audio));
        debug.push(format!(
            "ports={ports:02x?} main={:02x} sub={:02x} subsub={:02x} full_pc={:04x} full_in={:02x?} full_out={:02x?} full_dsp_writes={} full_last_dsp={:02x?} {}",
            game.ram[0x10],
            game.ram[0x11],
            game.ram[0xb0],
            full_apu.spc.pc,
            &full_apu.in_ports[..4],
            full_apu.out_ports,
            full_apu.dsp_write_history.len(),
            full_apu.dsp_write_history.last().copied(),
            game.zelda_audio_debug_summary(),
        ));
    }

    let threshold = 512i16;
    let high_onset = first_peak_frame(&high_stats, threshold);
    let full_onset = first_peak_frame(&full_stats, threshold);
    let high_max = max_peak_frame(&high_stats);
    let full_max = max_peak_frame(&full_stats);
    println!(
        "bootstrap APU startup threshold={threshold}: high_onset={high_onset:?} full_onset={full_onset:?} high_max={high_max:?} full_max={full_max:?}",
    );
    if let (Some(high_onset), Some(full_onset)) = (high_onset, full_onset) {
        let delta = full_onset as i32 - high_onset as i32;
        println!("bootstrap APU onset_delta_full_minus_high={delta} frames");
    }
    print_audio_window(
        "high",
        &high_stats,
        &debug,
        high_onset.or(high_max.map(|(i, _)| i)),
    );
    print_audio_window(
        "bootstrap-full-apu",
        &full_stats,
        &[],
        full_onset.or(full_max.map(|(i, _)| i)),
    );
}

fn run_trace_bootstrap_apu_direct_frame(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --trace-bootstrap-apu-direct-frame <path-to-rom.sfc> [frames] [bootstrap-opcode-budget] [apu-cycles-per-cpu-cycle]"
            );
            process::exit(2);
        }
    };
    let frames: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(20);
    let bootstrap_budget: u64 = args
        .get(2)
        .map(|s| s.parse().unwrap_or(1_500_000))
        .unwrap_or(1_500_000);
    let apu_cycles_per_cpu_cycle: f64 = args
        .get(3)
        .map(|s| s.parse().unwrap_or(0.286))
        .unwrap_or(0.286);

    let (bootstrap_snes, bootstrap_opcodes) = match capture_raw_rom_apu_bootstrap(
        rom_path,
        bootstrap_budget,
        apu_cycles_per_cpu_cycle,
        true,
    ) {
        Ok(capture) => capture,
        Err(e) => {
            eprintln!("failed to capture raw ROM APU bootstrap: {e}");
            process::exit(1);
        }
    };
    let rom = match fs::read(rom_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("failed to read {rom_path}: {e}");
            process::exit(1);
        }
    };
    let bootstrap_cpu_k = bootstrap_snes.cpu.k;
    let bootstrap_cpu_pc = bootstrap_snes.cpu.pc;
    let bootstrap_spc_pc = bootstrap_snes.apu.spc.pc;
    let bootstrap_payload_nonzero = apu_payload_nonzero(&bootstrap_snes);
    let bootstrap_dsp_writes = bootstrap_snes.apu.dsp_write_history.len();

    let mut oracle = match LockstepOracle::emu_initialize_owned(&rom) {
        Ok(oracle) => oracle,
        Err(e) => {
            eprintln!("failed to initialize patched direct-frame oracle: {e}");
            process::exit(1);
        }
    };
    oracle.snes.apu = bootstrap_snes.apu;
    oracle.snes.apu.dsp.sample_offset = 0;
    oracle.snes.apu.dsp_write_history.clear();

    let mut game = load_play_state(rom_path);
    let mut high_audio = vec![0i16; 735 * 2];
    let mut direct_audio = vec![0i16; 735 * 2];
    let mut apu_cycle_accum = 0.0f64;
    let loop_limit = 2_000_000usize;

    println!(
        "bootstrap direct-frame APU trace: frames={} bootstrap_opcodes={} budget={} apu_cycles_per_cpu_cycle={:.3} bootstrap_cpu=${:02x}:{:04x} bootstrap_spc=${:04x} payload_nz={} dsp_writes={}",
        frames,
        bootstrap_opcodes,
        bootstrap_budget,
        apu_cycles_per_cpu_cycle,
        bootstrap_cpu_k,
        bootstrap_cpu_pc,
        bootstrap_spc_pc,
        bootstrap_payload_nonzero,
        bootstrap_dsp_writes,
    );

    for frame_index in 0..frames {
        game.zelda_run_frame(0);
        let high_ports = game.zelda_debug_apu_write_ports();
        game.zelda_render_audio(&mut high_audio, 735, 2);
        game.zelda_discard_unused_audio_frames();
        let high_stats = AudioFrameStats::from_interleaved_stereo(&high_audio);

        let direct_ops = match run_raw_direct_frame_with_apu(
            &mut oracle.snes,
            RUN_MAIN,
            apu_cycles_per_cpu_cycle,
            &mut apu_cycle_accum,
            loop_limit,
        ) {
            Ok(opcodes) => opcodes,
            Err(e) => {
                eprintln!("direct-frame trace failed at frame {frame_index}: {e}");
                process::exit(1);
            }
        };
        render_full_apu_audio(&mut oracle.snes.apu, &mut direct_audio, 735, 2);
        let direct_stats = AudioFrameStats::from_interleaved_stereo(&direct_audio);
        println!(
            "frame={frame_index:>3} high_ports={high_ports:02x?} direct_in={:02x?} direct_out={:02x?} direct_ops={} high_peak={} high_first={:?} direct_peak={} direct_first={:?} direct_cpu=${:02x}:{:04x} direct_spc=${:04x} apu_cycles={} sample_offset={} main={:02x} sub={:02x} subsub={:02x} {}",
            &oracle.snes.apu.in_ports[..4],
            oracle.snes.apu.out_ports,
            direct_ops,
            high_stats.peak,
            high_stats.first_nonzero,
            direct_stats.peak,
            direct_stats.first_nonzero,
            oracle.snes.cpu.k,
            oracle.snes.cpu.pc,
            oracle.snes.apu.spc.pc,
            oracle.snes.apu.cycles,
            oracle.snes.apu.dsp.sample_offset,
            game.ram[0x10],
            game.ram[0x11],
            game.ram[0xb0],
            game.zelda_audio_debug_summary(),
        );
    }
}

fn capture_raw_rom_apu_bootstrap(
    rom_path: &str,
    budget: u64,
    apu_cycles_per_cpu_cycle: f64,
    stop_when_ready: bool,
) -> Result<(Snes, u64), Box<dyn Error>> {
    let rom = fs::read(rom_path)?;
    let mut snes = Snes::new();
    load_rom(&mut snes, &rom)?;
    snes.reset(true);
    snes.cpu_seed_reset_vector();

    let mut apu_cycle_accum = 0.0f64;
    for op in 1..=budget {
        step_raw_snes_with_apu(&mut snes, apu_cycles_per_cpu_cycle, &mut apu_cycle_accum);
        if stop_when_ready && raw_rom_apu_bootstrap_ready(&snes) {
            return Ok((snes, op));
        }
        if snes.cpu.stopped {
            return Ok((snes, op));
        }
    }
    Ok((snes, budget))
}

fn step_raw_snes_with_apu(
    snes: &mut Snes,
    apu_cycles_per_cpu_cycle: f64,
    apu_cycle_accum: &mut f64,
) {
    let cycles = cpu_run_opcode(snes);
    while snes.dma.dma_busy {
        snes.dma_do();
    }

    *apu_cycle_accum += f64::from(cycles) * apu_cycles_per_cpu_cycle;
    let apu_cycles = *apu_cycle_accum as u32;
    for _ in 0..apu_cycles {
        snes.apu.cycle();
    }
    *apu_cycle_accum -= f64::from(apu_cycles);
}

fn run_raw_direct_frame_with_apu(
    snes: &mut Snes,
    run_what: u8,
    apu_cycles_per_cpu_cycle: f64,
    apu_cycle_accum: &mut f64,
    loop_limit: usize,
) -> Result<usize, String> {
    let mut opcodes = 0usize;
    if snes.cpu.pc == 0x8000 && snes.cpu.k == 0 {
        opcodes += run_raw_direct_loop_with_apu(
            snes,
            apu_cycles_per_cpu_cycle,
            apu_cycle_accum,
            loop_limit,
        )?;
        snes.ram[0x12] = 1;
        write_le_u16_raw(&mut snes.ram, 0x0ae0, 0xb280);
        write_le_u16_raw(&mut snes.ram, 0x0ae2, 0xb280 + 0x60);
    }

    if run_what & RUN_POLY != 0 {
        snes.cpu.sp = 0x1f3e;
        snes.cpu.pc = 0xf81d;
        snes.cpu.db = 9;
        snes.cpu.k = 9;
        snes.cpu.dp = 0x1f00;
        opcodes += run_raw_direct_loop_with_apu(
            snes,
            apu_cycles_per_cpu_cycle,
            apu_cycle_accum,
            loop_limit,
        )?;
    }

    if run_what & RUN_MAIN != 0 {
        snes.cpu.sp = 0x01ff;
        snes.cpu.pc = 0x8034;
        snes.cpu.k = 0;
        snes.cpu.dp = 0;
        snes.cpu.db = 0;
        opcodes += run_raw_direct_loop_with_apu(
            snes,
            apu_cycles_per_cpu_cycle,
            apu_cycle_accum,
            loop_limit,
        )?;
    }

    snes.do_auto_joypad();
    if snes.ram[0x0add] == 0 {
        write_le_u16_raw(&mut snes.ram, 0x0adc, 0xa680);
    }
    snes.write(0x004300, 0x01);
    snes.write(0x004301, 0x18);

    snes.cpu.sp = 0x01ff;
    snes.cpu.pc = 0x80d9;
    snes.cpu.k = 0;
    snes.cpu.dp = 0;
    snes.cpu.db = 0;
    opcodes +=
        run_raw_direct_loop_with_apu(snes, apu_cycles_per_cpu_cycle, apu_cycle_accum, loop_limit)?;
    snes.frames = snes.frames.wrapping_add(1);
    Ok(opcodes)
}

fn run_raw_direct_loop_with_apu(
    snes: &mut Snes,
    apu_cycles_per_cpu_cycle: f64,
    apu_cycle_accum: &mut f64,
    loop_limit: usize,
) -> Result<usize, String> {
    snes.cpu.a = 0;
    snes.cpu.x = 0;
    snes.cpu.y = 0;
    snes.cpu.e = false;
    snes.cpu.irq_wanted = false;
    snes.cpu.nmi_wanted = false;
    snes.cpu.waiting = false;
    snes.cpu.stopped = false;
    snes.cpu.unpack_flags(0x30);

    for loops in 0..loop_limit {
        step_raw_snes_with_apu(snes, apu_cycles_per_cpu_cycle, apu_cycle_accum);
        let pc = ((snes.cpu.k as u32) << 16) | snes.cpu.pc as u32;
        if pc == 0x008034 || (pc == 0x09f81d && loops >= 10) || pc == 0x008225 || pc == 0x0082d2 {
            return Ok(loops + 1);
        }
    }

    let pc = ((snes.cpu.k as u32) << 16) | snes.cpu.pc as u32;
    Err(format!(
        "SNES direct-frame loop did not reach a checkpoint within {loop_limit} opcodes (pc=${pc:06X})"
    ))
}

fn write_le_u16_raw(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset] = value as u8;
    bytes[offset + 1] = (value >> 8) as u8;
}

fn raw_rom_apu_bootstrap_ready(snes: &Snes) -> bool {
    !snes.apu.rom_readable
        && snes.apu.spc.pc >= 0x0800
        && snes.apu.spc.pc < 0x1000
        && snes.apu.dsp_write_history.len() >= 16
}

fn print_rom_apu_trace_line(op: u64, label: &str, snes: &Snes) {
    let nz_0800 = count_nonzero(&snes.apu.ram[0x0800..0x0900]);
    let nz_0878 = count_nonzero(&snes.apu.ram[0x0878..0x08c0]);
    let nz_prog = count_nonzero(&snes.apu.ram[0x0800..0x1000]);
    let payload_nonzero = apu_payload_nonzero(snes);
    let payload_range = apu_payload_range(snes)
        .map(|(first, last)| format!("${first:04x}-${last:04x}"))
        .unwrap_or_else(|| "none".to_string());
    let last_dsp = snes
        .apu
        .dsp_write_history
        .last()
        .map(|(addr, value)| format!("${addr:02x}=${value:02x}"))
        .unwrap_or_else(|| "none".to_string());
    let upload_addr = (u16::from(snes.apu.in_ports[3]) << 8) | u16::from(snes.apu.in_ports[2]);
    println!(
        "op={op:>8} {label:<6} cpu=${:02x}:{:04x} spc=${:04x} apu_cycles={} out={:02x?} upload=${:04x} in={:02x?} rom={} reset=${:02x}{:02x} payload_nz={} payload_range={} nz0800={} nz0878={} nz0800_1000={} dsp_writes={} last_dsp={} dsp_samples={}",
        snes.cpu.k,
        snes.cpu.pc,
        snes.apu.spc.pc,
        snes.apu.cycles,
        snes.apu.out_ports,
        upload_addr,
        [
            snes.apu.in_ports[0],
            snes.apu.in_ports[1],
            snes.apu.in_ports[2],
            snes.apu.in_ports[3]
        ],
        snes.apu.rom_readable,
        snes.apu.ram[0xffff],
        snes.apu.ram[0xfffe],
        payload_nonzero,
        payload_range,
        nz_0800,
        nz_0878,
        nz_prog,
        snes.apu.dsp_write_history.len(),
        last_dsp,
        snes.apu.dsp.sample_offset
    );
}

fn apu_payload_nonzero(snes: &Snes) -> usize {
    count_nonzero(&snes.apu.ram[..0xffc0])
}

fn apu_payload_range(snes: &Snes) -> Option<(usize, usize)> {
    let first = snes.apu.ram[..0xffc0].iter().position(|&b| b != 0)?;
    let last = snes.apu.ram[..0xffc0].iter().rposition(|&b| b != 0)?;
    Some((first, last))
}

fn count_nonzero(bytes: &[u8]) -> usize {
    bytes.iter().filter(|&&b| b != 0).count()
}

fn print_asset_gpu_smoke_progress(
    label: &str,
    frames: u32,
    game: &ZeldaState,
    renderer: &ModernAssetGpuReadbackRenderer,
) {
    let (
        cache_hits,
        cache_misses,
        cache_entries,
        cache_key_ms,
        cache_miss_ms,
        bg_extract_ms,
        sprite_extract_ms,
        stats_ms,
    ) = renderer.validation_cache_stats();
    eprintln!(
        "{label} asset GPU smoke progress frames={frames} main={:02x}; sub={:02x}; mode={}; screen={:02x}/{:02x}; validation_cache_hits={cache_hits}; validation_cache_misses={cache_misses}; validation_cache_entries={cache_entries}; validation_key_ms={cache_key_ms}; validation_miss_ms={cache_miss_ms}; validation_bg_extract_ms={bg_extract_ms}; validation_sprite_extract_ms={sprite_extract_ms}; validation_stats_ms={stats_ms}",
        game.ram[0x10],
        game.ram[0x11],
        game.ppu.mode,
        game.ppu.screen_enabled[0],
        game.ppu.screen_enabled[1],
    );
}

fn write_asset_gpu_missing_report_or_exit(
    path: &Path,
    command: &str,
    frame: u32,
    input: u16,
    error: &str,
) {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!(
                "failed to create missing-assets output directory {}: {e}",
                parent.display()
            );
            process::exit(2);
        }
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap_or_else(|e| {
            eprintln!(
                "failed to open missing-assets output {}: {e}",
                path.display()
            );
            process::exit(2);
        });
    let record = serde_json::json!({
        "command": command,
        "frame": frame,
        "input": format!("0x{input:04x}"),
        "error": error,
    });
    writeln!(file, "{record}").unwrap_or_else(|e| {
        eprintln!(
            "failed to write missing-assets output {}: {e}",
            path.display()
        );
        process::exit(2);
    });
}

fn write_asset_gpu_checkpoint_or_exit(game: &ZeldaState, frames: u32, dir: &Path) {
    if let Err(e) = fs::create_dir_all(dir) {
        eprintln!(
            "failed to create asset GPU checkpoint directory {}: {e}",
            dir.display()
        );
        process::exit(2);
    }
    let mut checkpoint_game = game.clone();
    let frame_path = dir.join(format!("asset-gpu-frame-{frames:09}.sav"));
    write_checkpoint(&mut checkpoint_game, frames, &frame_path);
    let latest_path = dir.join("asset-gpu-latest.sav");
    write_checkpoint(&mut checkpoint_game, frames, &latest_path);
    let latest_frame_path = dir.join("asset-gpu-latest-frame.txt");
    fs::write(&latest_frame_path, format!("{frames}\n")).unwrap_or_else(|e| {
        eprintln!(
            "failed to write asset GPU latest frame {}: {e}",
            latest_frame_path.display()
        );
        process::exit(2);
    });
}

fn modern_audio_sample_asset_hash(sample_ram: &[u8], source: u8) -> u32 {
    const DIRECTORY: usize = 0x3c00;
    let mut hash = 2166136261u32;
    let entry = DIRECTORY + usize::from(source) * 4;
    let Some(directory) = sample_ram.get(entry..entry + 4) else {
        return hash;
    };
    for &byte in directory {
        hash = (hash ^ u32::from(byte)).wrapping_mul(16777619);
    }
    let mut address = usize::from(directory[0]) | (usize::from(directory[1]) << 8);
    if address == 0 {
        return hash;
    }
    for _ in 0..4096 {
        let Some(block) = sample_ram.get(address..address + 9) else {
            break;
        };
        for &byte in block {
            hash = (hash ^ u32::from(byte)).wrapping_mul(16777619);
        }
        if block[0] & 1 != 0 {
            break;
        }
        address += 9;
    }
    hash
}

fn run_replay_save(args: &[String]) {
    let ReplaySaveConfig {
        rom_path,
        replay_path,
        max_frames,
        dump_frame_path,
        render_hash_log,
        audio_trace_log,
        fingerprint_log,
        fingerprint_frame,
        coverage_log,
        mut gpu_render_compare,
        mut modern_index_compare,
        asset_gpu_smoke,
        asset_gpu_progress_interval,
        asset_gpu_missing_assets_out,
        asset_gpu_checkpoint_dir,
        asset_gpu_checkpoint_interval,
        ppu_mode_summary,
        render_hash_dump_frame,
        save_state_path,
        save_state_at,
        load_state_path,
        load_sram_path,
        input_script,
        input_script_overlay,
        stop_replay_after_load,
    } = parse_replay_save_args_or_exit(args);
    #[cfg(not(feature = "audio-oracle"))]
    if audio_trace_log != 0 || fingerprint_log.is_some() {
        eprintln!(
            "audio trace and fingerprint diagnostics require an audio-oracle build; rebuild with --features audio-oracle"
        );
        process::exit(2);
    }
    #[cfg(not(feature = "audio-oracle"))]
    if std::env::var_os("ZELDA3_DBG_AUDIO_FP").is_some() {
        require_audio_oracle("ZELDA3_DBG_AUDIO_FP");
    }
    let mut ppu_mode_counts = [0u64; 8];
    let mut first_mode7_frame = None::<u32>;
    let mut last_mode7_frame = None::<u32>;

    let last_panic = install_crash_panic_hook();
    let mut game = load_translated_replay_state(&rom_path);
    if let Some(path) = load_state_path.as_deref() {
        if let Err(e) = load_replay_save_checkpoint(&mut game, path) {
            eprintln!(
                "failed to load replay-save checkpoint {}: {e}",
                path.display()
            );
            process::exit(1);
        }
        if std::env::var("ZELDA3_DBG_AUDIO_FP").is_ok() {
            eprintln!(
                "[AUDIO_FP] post-load dsp_hash=0x{:08x}",
                game.zelda_audio_dsp_hash()
            );
        }
    } else {
        if let Err(e) = game.replay_save_file(Path::new(&replay_path)) {
            eprintln!("failed to replay {}: {e}", replay_path);
            process::exit(1);
        }
        if let Some(path) = load_sram_path.as_deref() {
            let sram = read_file_or_exit(path, "SRAM");
            apply_sram_to_game_or_exit(&mut game, path, &sram);
        }
    }
    if stop_replay_after_load {
        let mut state_recorder = std::mem::take(&mut game.state_recorder);
        ZeldaState::state_recorder_stop_replay(&mut state_recorder);
        game.state_recorder = state_recorder;
    }

    let mut frames = game.state_recorder.replay_frame_counter;
    let scripted_playback = stop_replay_after_load
        || !input_script.is_empty()
        || input_script_overlay
            .as_ref()
            .is_some_and(|script| !script.is_empty());
    let mut audio_trace_buffer = if audio_trace_log != 0 || fingerprint_log.is_some() {
        Some(vec![0i16; 735 * 2])
    } else {
        None
    };
    #[cfg(feature = "audio-oracle")]
    let (mut modern_audio_trace_sequence, mut modern_audio_trace_engine) =
        game.zelda_oracle_aligned_modern_audio_trace_state();
    #[cfg(not(feature = "audio-oracle"))]
    let (mut modern_audio_trace_sequence, mut modern_audio_trace_engine) = (
        zelda3::modern_audio_sequence::ModernAudioSequencer::default(),
        zelda3::modern_audio::ModernAudioEngine::default(),
    );
    let mut modern_audio_trace_last_dsp_post = None;
    let mut modern_audio_trace_sample_assets = [None; 256];
    let mut modern_audio_trace_has_checkpoint_seed = false;
    let render_hash_cpu_debug = std::env::var_os("ZELDA3_RENDER_HASH_CPU_DEBUG").is_some();
    let mut render_hash_frame = if gpu_render_compare.enabled()
        || render_hash_cpu_debug
        || render_hash_dump_frame.is_some()
        || fingerprint_log.is_some()
    {
        Some(vec![0u8; 256 * 224 * 4])
    } else {
        None
    };
    // Classic GPU readback is used for compare/hash diagnostics. Frame dumps
    // render through the PNG-backed asset path below.
    let mut gpu_readback = replay_optional_gpu_readback_renderer(
        render_hash_log,
        &gpu_render_compare,
        render_hash_dump_frame.is_some(),
        &modern_index_compare,
    );
    let mut fingerprint_writer = match fingerprint_log.as_deref() {
        Some(p) => {
            let f = std::fs::File::create(p).unwrap_or_else(|e| {
                eprintln!("failed to create fingerprint log {p:?}: {e}");
                process::exit(2);
            });
            Some(std::io::BufWriter::new(f))
        }
        None => None,
    };
    let mut route_coverage = coverage_log
        .as_ref()
        .map(|_| parity::coverage::RouteCoverage::default());
    // Sprite tiles are now decoded from LIVE VRAM per frame
    // (extract_modern_sprites_from_vram); the static sprite atlas is no longer
    // loaded for rendering.
    //
    // Off-VRAM atlas paths: unset uses `assets-variant-gpu`. Replay-save
    // compare keeps source-backed rendering on GPU; older CPU atlas comparison
    // stays outside this route runner.
    modern_index_compare
        .load_resources(Path::new("."), false)
        .unwrap_or_else(|e| {
            eprintln!("modern index compare resources load failed: {e}");
            process::exit(2);
        });
    let mut asset_gpu_smoke_renderer = if asset_gpu_smoke {
        Some(load_modern_asset_gpu_readback_or_exit(
            "replay-save asset GPU smoke",
        ))
    } else {
        None
    };
    let capture_panic_pre_frame =
        std::env::var_os("ZELDA3_REPLAY_CAPTURE_PANIC_PRE_FRAME").is_some();
    let mut last_frame_had_fingerprint_render = false;
    while frames < max_frames && (scripted_playback || game.state_recorder.replay_mode) {
        let pre_frame_game = capture_panic_pre_frame.then(|| game.clone());
        let input_override = input_script_overlay
            .as_ref()
            .and_then(|script| script.input_override_for_frame(frames));
        let input = input_override.unwrap_or_else(|| input_script.input_for_frame(frames));
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            game.zelda_run_frame_with_replay_input_override(input as i32, input_override);
        }));
        if let Err(payload) = result {
            let panic_info = captured_panic_from(last_panic.clone(), payload);
            let report_game = pre_frame_game.as_ref().unwrap_or(&game);
            print_replay_save_panic_report(report_game, frames, &panic_info);
            process::exit(101);
        }
        frames = frames.wrapping_add(1);
        if ppu_mode_summary {
            let mode = usize::from(game.ppu.mode);
            if mode < ppu_mode_counts.len() {
                ppu_mode_counts[mode] += 1;
            }
            if game.ppu.mode == 7 {
                first_mode7_frame.get_or_insert(frames);
                last_mode7_frame = Some(frames);
            }
        }
        if let Some(renderer) = asset_gpu_smoke_renderer.as_mut() {
            if let Err(e) = renderer.validate_game_full_gpu_path(&mut game) {
                if let Some(path) = asset_gpu_missing_assets_out.as_deref() {
                    write_asset_gpu_missing_report_or_exit(path, "replay-save", frames, input, &e);
                }
                eprintln!(
                    "replay-save asset GPU smoke failed frame={frames} input=0x{input:04x}: {e}"
                );
                process::exit(1);
            }
            if asset_gpu_progress_interval != 0 && frames % asset_gpu_progress_interval == 0 {
                print_asset_gpu_smoke_progress("replay-save", frames, &game, renderer);
            }
        }
        last_frame_had_fingerprint_render = false;
        let mut fp_audio_leaf: u32 = 0;
        let should_fingerprint_frame =
            fingerprint_log.is_some() && should_write_fingerprint(fingerprint_frame, frames);
        if let Some(audio) = audio_trace_buffer.as_mut() {
            let dsp_pre = game.zelda_audio_dsp_hash();
            let dsp_pre_snapshot = game.zelda_audio_dsp_snapshot();
            let spc_ram_pre = game.zelda_audio_live_spc_ram();
            let dsp_changed = modern_audio_trace_last_dsp_post
                .is_some_and(|previous| previous != dsp_pre);
            let mut sample_ram_changed = false;
            for voice in 0..8 {
                let state_base = 0x80 + voice * 86;
                let Some(&source) = dsp_pre_snapshot.get(state_base + 44) else {
                    continue;
                };
                let hash = modern_audio_sample_asset_hash(&spc_ram_pre, source);
                let previous = &mut modern_audio_trace_sample_assets[usize::from(source)];
                sample_ram_changed |= modern_audio_trace_has_checkpoint_seed
                    && previous.is_some_and(|value| value != hash);
                *previous = Some(hash);
            }
            if dsp_changed || sample_ram_changed
            {
                modern_audio_trace_engine
                    .seed_dsp_checkpoint_state(&spc_ram_pre, &dsp_pre_snapshot);
                game.zelda_sync_modern_audio_trace_engine(&mut modern_audio_trace_engine, 0);
            }
            modern_audio_trace_has_checkpoint_seed |= dsp_changed;
            let writes = game.zelda_render_audio_trace_dsp_events(audio, 735, 2);
            game.zelda_discard_unused_audio_frames();
            modern_audio_trace_last_dsp_post = Some(game.zelda_audio_dsp_hash());
            if should_fingerprint_frame {
                let dsp_post = game.zelda_audio_dsp_hash();
                let s_samples = replay_checksum_samples(audio);
                let s_writes = replay_checksum_dsp_writes(&writes);
                let s_wvals = replay_checksum_dsp_write_values(&writes);
                if std::env::var("ZELDA3_DBG_AUDIO_FP").is_ok() {
                    eprintln!(
                        "[AUDIO_FP] f={frames} samp=0x{s_samples:08x} pre=0x{dsp_pre:08x} post=0x{dsp_post:08x} wc={} wh=0x{s_writes:08x} wvh=0x{s_wvals:08x}",
                        writes.len()
                    );
                }
                fp_audio_leaf = fingerprint_audio_hash(
                    s_samples,
                    dsp_pre,
                    dsp_post,
                    writes.len() as u32,
                    s_writes,
                    s_wvals,
                );
            }
            if audio_trace_log != 0 && frames % audio_trace_log == 0 {
                print_replay_audio_trace(
                    frames,
                    &game,
                    audio,
                    735,
                    2,
                    dsp_pre,
                    &writes,
                    &spc_ram_pre,
                    &mut modern_audio_trace_sequence,
                    &mut modern_audio_trace_engine,
                );
            }
        }
        let should_log_render_hash = render_hash_log != 0 && frames % render_hash_log == 0;
        let should_compare_gpu = gpu_render_compare.should_compare_frame(frames);
        let should_dump_render_hash = render_hash_dump_frame
            .as_ref()
            .is_some_and(|(dump_frame, _)| frames == *dump_frame);
        if should_compare_gpu && !should_log_render_hash && !should_dump_render_hash {
            let frame = render_hash_frame
                .as_mut()
                .expect("render compare frame allocated");
            if !gpu_render_compare.emit_current_frame_with_optional_readback(
                &mut game,
                &mut gpu_readback,
                frame,
                frames,
            ) {
                process::exit(1);
            }
        }
        if should_dump_render_hash && !should_log_render_hash {
            if let Some((_, dump_path)) = render_hash_dump_frame.as_ref() {
                let width = 256u32;
                let render_hash_capture = gpu_readback.capture_replay_render_hash_frame(&mut game);
                write_replay_save_render_hash_gpu_dump_or_exit(
                    dump_path,
                    &render_hash_capture,
                    &mut gpu_readback,
                    width,
                );
            }
        }
        if should_log_render_hash {
            let width = 256u32;
            // Run HDMA channel 6+7 for one line to load CGRAM entries that ALttP sets
            // via HDMA (e.g. dungeon floor palettes). State is restored after the call
            // so zelda_draw_ppu_frame renders from the correct baseline.
            let render_hash_capture = gpu_readback.capture_replay_render_hash_frame(&mut game);
            if !render_hash_cpu_debug {
                let gpu_rgba = render_hash_capture.render_gpu_rgba(&mut gpu_readback);
                if should_dump_render_hash {
                    if let Some((_, dump_path)) = render_hash_dump_frame.as_ref() {
                        write_replay_save_render_hash_gpu_dump_or_exit(
                            dump_path,
                            &render_hash_capture,
                            &mut gpu_readback,
                            width,
                        );
                    }
                }
                println!("{}", gpu_rgba.render_hash_log_line(frames));
            } else {
                let frame = render_hash_frame
                    .as_mut()
                    .expect("render hash frame allocated");
                if frames == 800 {
                    // Dump DMA channel state to find which channel targets $212C (TM).
                    let hdmaen_copy = game.ram[0x9b];
                    eprintln!("[gpu-dbg] f800 HDMAEN_COPY={:#04x}", hdmaen_copy);
                    for ch in 0..8 {
                        let dc = &game.dma.channel[ch];
                        eprintln!(
                        "[gpu-dbg] f800 dma[{}]: active(HDMAEN_COPY)={} b_adr={:#04x} a_bank={:#04x} a_adr={:#06x} mode={:#04x} indirect={} hdma_active_field={}",
                        ch,
                        (hdmaen_copy >> ch) & 1,
                        dc.b_adr,
                        dc.a_bank,
                        dc.a_adr,
                        dc.mode,
                        dc.indirect,
                        dc.hdma_active
                    );
                    }
                    // Dump per-scanline TM values from the simulation result.
                    eprintln!(
                        "{}",
                        render_hash_capture.debug_frame_800_scanline_screen_enabled_main_line()
                    );
                    for i in 0..3 {
                        eprintln!(
                        "[gpu-dbg] f800 bg{} after-hdma-sim: h_scroll={} v_scroll={} tilemap_adr={} tile_adr={}",
                        i + 1,
                        game.ppu.bg_layer[i].h_scroll,
                        game.ppu.bg_layer[i].v_scroll,
                        game.ppu.bg_layer[i].tilemap_adr,
                        game.ppu.bg_layer[i].tile_adr
                    );
                    }
                    // Manual tile lookup for BG1 at (126,65)
                    let bg1 = &game.ppu.bg_layer[0];
                    let h_scroll = bg1.h_scroll as u32;
                    let v_scroll = bg1.v_scroll as u32;
                    let sx = 126u32;
                    let sy = 65u32;
                    let map_px_w = if bg1.tilemap_wider { 512u32 } else { 256u32 };
                    let map_px_h = if bg1.tilemap_higher { 512u32 } else { 256u32 };
                    let scroll_x = (sx + h_scroll) % map_px_w;
                    let scroll_y = (sy + v_scroll + 1) % map_px_h;
                    let tile_x = scroll_x / 8;
                    let tile_y = scroll_y / 8;
                    let pixel_x = scroll_x % 8;
                    let pixel_y = scroll_y % 8;
                    let tilemap_width = if bg1.tilemap_wider { 64u32 } else { 32u32 };
                    let flat_idx = tile_y * tilemap_width + tile_x;
                    let vram_entry_idx = bg1.tilemap_adr as u32 + flat_idx;
                    let entry = game
                        .ppu
                        .vram
                        .get(vram_entry_idx as usize)
                        .copied()
                        .unwrap_or(0);
                    let tile_num = entry & 0x3FF;
                    let palette_sub = (entry >> 10) & 7;
                    let priority = (entry >> 13) & 1;
                    let hflip = (entry >> 14) & 1;
                    let vflip = (entry >> 15) & 1;
                    let px = if hflip != 0 { 7 - pixel_x } else { pixel_x };
                    let py = if vflip != 0 { 7 - pixel_y } else { pixel_y };
                    let tile_base = bg1.tile_adr as u32 + tile_num as u32 * 16; // 4bpp = 16 VRAM words
                    let w01 = game
                        .ppu
                        .vram
                        .get((tile_base + py) as usize & 0x7fff)
                        .copied()
                        .unwrap_or(0);
                    let w23 = game
                        .ppu
                        .vram
                        .get((tile_base + 8 + py) as usize & 0x7fff)
                        .copied()
                        .unwrap_or(0);
                    let bit = 7 - px;
                    let pal_idx = ((w01 >> bit) & 1)
                        | (((w01 >> (8 + bit)) & 1) << 1)
                        | (((w23 >> bit) & 1) << 2)
                        | (((w23 >> (8 + bit)) & 1) << 3);
                    eprintln!(
                    "[gpu-dbg] f800 BG1 at (126,65): scroll=({},{}) scroll_xy=({},{}) tile=({},{}) flat_idx={} vram_idx={} entry={:#06x} tile_num={} pal_sub={} priority={} hflip={} vflip={} px={} py={} pal_idx={}",
                    h_scroll,
                    v_scroll,
                    scroll_x,
                    scroll_y,
                    tile_x,
                    tile_y,
                    flat_idx,
                    vram_entry_idx,
                    entry,
                    tile_num,
                    palette_sub,
                    priority,
                    hflip,
                    vflip,
                    px,
                    py,
                    pal_idx
                );
                    let cgram_idx = (palette_sub * 16 + pal_idx) as usize;
                    let cgram_val = render_hash_capture.cgram_color_hex(cgram_idx);
                    eprintln!(
                        "[gpu-dbg] f800 BG1 at (126,65): cgram_idx={} cgram_val={}",
                        cgram_idx, cgram_val
                    );
                }
                let pre_screen_enabled = game.ppu.screen_enabled[0];
                let pre_scrolls: [(u16, u16); 4] = std::array::from_fn(|i| {
                    (game.ppu.bg_layer[i].h_scroll, game.ppu.bg_layer[i].v_scroll)
                });
                let pre_vram_hash: u32 = {
                    let mut h = 2166136261u32;
                    for &w in &game.ppu.vram {
                        let [lo, hi] = w.to_le_bytes();
                        h = h.wrapping_mul(16777619) ^ u32::from(lo);
                        h = h.wrapping_mul(16777619) ^ u32::from(hi);
                    }
                    h
                };
                let rgba = gpu_readback.render_replay_hash_cpu_frame_rgba(&mut game, frame);
                if frames == 800 {
                    let post_scrolls: [(u16, u16); 4] = std::array::from_fn(|i| {
                        (game.ppu.bg_layer[i].h_scroll, game.ppu.bg_layer[i].v_scroll)
                    });
                    for i in 0..3 {
                        if pre_scrolls[i] != post_scrolls[i] {
                            eprintln!(
                                "[gpu-dbg] f800 bg{} scroll changed: pre=({},{}) post=({},{})",
                                i + 1,
                                pre_scrolls[i].0,
                                pre_scrolls[i].1,
                                post_scrolls[i].0,
                                post_scrolls[i].1
                            );
                        }
                    }
                    // Log post-render tilemap/tile_adr so we know what gpu_frame_from_ppu reads.
                    for i in 0..3 {
                        eprintln!(
                        "[gpu-dbg] f800 bg{} POST-RENDER: h_scroll={} v_scroll={} tilemap_adr={} tile_adr={}",
                        i + 1,
                        game.ppu.bg_layer[i].h_scroll,
                        game.ppu.bg_layer[i].v_scroll,
                        game.ppu.bg_layer[i].tilemap_adr,
                        game.ppu.bg_layer[i].tile_adr
                    );
                    }
                    eprintln!(
                    "[gpu-dbg] f800 screen_enabled: pre[0]={:#04x} post[0]={:#04x} post[1]={:#04x}",
                    pre_screen_enabled, game.ppu.screen_enabled[0], game.ppu.screen_enabled[1]
                );
                    // Redo BG1 tile lookup at (126,65) using POST-RENDER state (= what GPU uses).
                    let bg1 = &game.ppu.bg_layer[0];
                    let h_scroll = bg1.h_scroll as u32;
                    let v_scroll = bg1.v_scroll as u32;
                    let sx = 126u32;
                    let sy = 65u32;
                    let map_px_w = if bg1.tilemap_wider { 512u32 } else { 256u32 };
                    let map_px_h = if bg1.tilemap_higher { 512u32 } else { 256u32 };
                    let scroll_x = (sx + h_scroll) % map_px_w;
                    let scroll_y = (sy + v_scroll + 1) % map_px_h;
                    let tile_x = scroll_x / 8;
                    let tile_y = scroll_y / 8;
                    let pixel_x = scroll_x % 8;
                    let pixel_y = scroll_y % 8;
                    let tilemap_width = if bg1.tilemap_wider { 64u32 } else { 32u32 };
                    let flat_idx = tile_y * tilemap_width + tile_x;
                    let vram_entry_idx = bg1.tilemap_adr as u32 + flat_idx;
                    let entry = game
                        .ppu
                        .vram
                        .get(vram_entry_idx as usize)
                        .copied()
                        .unwrap_or(0);
                    let tile_num = entry & 0x3FF;
                    let palette_sub = (entry >> 10) & 7;
                    let hflip = (entry >> 14) & 1;
                    let vflip = (entry >> 15) & 1;
                    let px = if hflip != 0 { 7 - pixel_x } else { pixel_x };
                    let py = if vflip != 0 { 7 - pixel_y } else { pixel_y };
                    let tile_base = bg1.tile_adr as u32 + tile_num as u32 * 16;
                    let w01 = game
                        .ppu
                        .vram
                        .get((tile_base + py) as usize & 0x7fff)
                        .copied()
                        .unwrap_or(0);
                    let w23 = game
                        .ppu
                        .vram
                        .get((tile_base + 8 + py) as usize & 0x7fff)
                        .copied()
                        .unwrap_or(0);
                    let bit = 7 - px;
                    let pal_idx = ((w01 >> bit) & 1)
                        | (((w01 >> (8 + bit)) & 1) << 1)
                        | (((w23 >> bit) & 1) << 2)
                        | (((w23 >> (8 + bit)) & 1) << 3);
                    let cgram_idx = (palette_sub * 16 + pal_idx) as usize;
                    let cgram_val = render_hash_capture.cgram_color_hex(cgram_idx);
                    eprintln!(
                    "[gpu-dbg] f800 BG1@(126,65) POST-RENDER: tilemap_adr={} tile_adr={} entry={:#06x} tile={} pal_sub={} pal_idx={} cgram[{}]={}",
                    bg1.tilemap_adr,
                    bg1.tile_adr,
                    entry,
                    tile_num,
                    palette_sub,
                    pal_idx,
                    cgram_idx,
                    cgram_val
                );
                }
                if frames == 1000 {
                    let post_vram_hash: u32 = {
                        let mut h = 2166136261u32;
                        for &w in &game.ppu.vram {
                            let [lo, hi] = w.to_le_bytes();
                            h = h.wrapping_mul(16777619) ^ u32::from(lo);
                            h = h.wrapping_mul(16777619) ^ u32::from(hi);
                        }
                        h
                    };
                    eprintln!(
                        "[gpu-dbg] VRAM hash before={:#010x} after={:#010x} changed={}",
                        pre_vram_hash,
                        post_vram_hash,
                        pre_vram_hash != post_vram_hash
                    );
                    eprintln!(
                    "[gpu-dbg] BG scroll before render: bg1=({},{}) bg2=({},{}) bg3=({},{}) bg4=({},{})",
                    pre_scrolls[0].0,
                    pre_scrolls[0].1,
                    pre_scrolls[1].0,
                    pre_scrolls[1].1,
                    pre_scrolls[2].0,
                    pre_scrolls[2].1,
                    pre_scrolls[3].0,
                    pre_scrolls[3].1
                );
                    let post_scrolls: Vec<_> = (0..4)
                        .map(|i| (game.ppu.bg_layer[i].h_scroll, game.ppu.bg_layer[i].v_scroll))
                        .collect();
                    eprintln!(
                    "[gpu-dbg] BG scroll  after render: bg1=({},{}) bg2=({},{}) bg3=({},{}) bg4=({},{})",
                    post_scrolls[0].0,
                    post_scrolls[0].1,
                    post_scrolls[1].0,
                    post_scrolls[1].1,
                    post_scrolls[2].0,
                    post_scrolls[2].1,
                    post_scrolls[3].0,
                    post_scrolls[3].1
                );
                    for line in
                        render_hash_capture.debug_cgram_render_diff_lines(frames, &game.ppu.cgram)
                    {
                        eprintln!("{line}");
                    }
                    // BG1 tilemap entry at (0,0): what tile does BG1 show at screen x=0..7?
                    let bg1_tilemap_adr = game.ppu.bg_layer[0].tilemap_adr as usize;
                    let bg1_tile_adr = game.ppu.bg_layer[0].tile_adr as usize;
                    eprint!("[gpu-dbg] BG1 tilemap row0 tiles 0..7:");
                    for tx in 0..8usize {
                        let entry = game
                            .ppu
                            .vram
                            .get(bg1_tilemap_adr + tx)
                            .copied()
                            .unwrap_or(0);
                        let tile_num = entry & 0x3ff;
                        let palette_sub = (entry >> 10) & 7;
                        let hflip = (entry >> 14) & 1;
                        let vflip = (entry >> 15) & 1;
                        eprint!(
                            " tile={} pal={} hflip={} vflip={}",
                            tile_num, palette_sub, hflip, vflip
                        );
                    }
                    eprintln!();
                    // BG1 tile at (0,0): dump pixel palette indices for row 0
                    let bg1_tile0_entry = game.ppu.vram.get(bg1_tilemap_adr).copied().unwrap_or(0);
                    let bg1_tile0_num = (bg1_tile0_entry & 0x3ff) as usize;
                    let bg1_hflip = (bg1_tile0_entry >> 14) & 1;
                    let bg1_tbase = bg1_tile_adr + bg1_tile0_num * 16;
                    let bg1_w01 = game.ppu.vram.get(bg1_tbase).copied().unwrap_or(0);
                    let bg1_w23 = game.ppu.vram.get(bg1_tbase + 8).copied().unwrap_or(0);
                    eprint!(
                        "[gpu-dbg] BG1 tile0 num={} hflip={} row0 palette indices:",
                        bg1_tile0_num, bg1_hflip
                    );
                    for px in 0..8usize {
                        let bit = if bg1_hflip != 0 {
                            px as u8
                        } else {
                            7 - px as u8
                        };
                        let bp0 = (bg1_w01 & 0xff) as u8;
                        let bp1 = (bg1_w01 >> 8) as u8;
                        let bp2 = (bg1_w23 & 0xff) as u8;
                        let bp3 = (bg1_w23 >> 8) as u8;
                        let idx = ((bp0 >> bit) & 1)
                            | (((bp1 >> bit) & 1) << 1)
                            | (((bp2 >> bit) & 1) << 2)
                            | (((bp3 >> bit) & 1) << 3);
                        eprint!(" {}", idx);
                    }
                    eprintln!();
                    eprintln!(
                        "[gpu-dbg] screen_enabled[0]={:#04x} screen_enabled[1]={:#04x}",
                        game.ppu.screen_enabled[0], game.ppu.screen_enabled[1]
                    );
                    // CGRAM sub-palette 5 (entries 80-95)
                    eprint!("[gpu-dbg] CGRAM sub-pal5 (80-95):");
                    for i in 80usize..96 {
                        eprint!(" {:04x}", game.ppu.cgram.get(i).copied().unwrap_or(0));
                    }
                    eprintln!();
                    // BG3 tilemap entry at (0,0)
                    let bg3_tilemap_adr = game.ppu.bg_layer[2].tilemap_adr as usize;
                    let bg3_tile_adr = game.ppu.bg_layer[2].tile_adr as usize;
                    let bg3_tile0_entry = game.ppu.vram.get(bg3_tilemap_adr).copied().unwrap_or(0);
                    let bg3_tile0_num = (bg3_tile0_entry & 0x3ff) as usize;
                    let bg3_hflip = (bg3_tile0_entry >> 14) & 1;
                    let bg3_vflip = (bg3_tile0_entry >> 15) & 1;
                    let bg3_pal = (bg3_tile0_entry >> 10) & 7;
                    let bg3_prio = (bg3_tile0_entry >> 13) & 1;
                    // BG3 is 2bpp: 8 words per tile, bp0/bp1 only
                    let bg3_tbase = bg3_tile_adr + bg3_tile0_num * 8; // word index
                    let bg3_w01 = game.ppu.vram.get(bg3_tbase).copied().unwrap_or(0);
                    eprint!(
                    "[gpu-dbg] BG3 tile0 num={} pal={} prio={} hflip={} vflip={} row0 2bpp indices:",
                    bg3_tile0_num, bg3_pal, bg3_prio, bg3_hflip, bg3_vflip
                );
                    for px in 0..8usize {
                        let bit = if bg3_hflip != 0 {
                            px as u8
                        } else {
                            7 - px as u8
                        };
                        let bp0 = (bg3_w01 & 0xff) as u8;
                        let bp1 = (bg3_w01 >> 8) as u8;
                        let idx = ((bp0 >> bit) & 1) | (((bp1 >> bit) & 1) << 1);
                        eprint!(" {}", idx);
                    }
                    eprintln!();
                    // Now manually simulate GPU atlas decode for BG3 tile0
                    // BG3 uses 2bpp; atlas_slot_base = bg3_tile_adr/16
                    let bg3_atlas_base = bg3_tile_adr / 16; // word units → slot
                    let bg3_slot = bg3_atlas_base + bg3_tile0_num / 2;
                    let bg3_vram_4bpp_base = bg3_slot * 16;
                    let bg3_4bpp_w01 = game.ppu.vram.get(bg3_vram_4bpp_base).copied().unwrap_or(0);
                    let bg3_4bpp_w23 = game
                        .ppu
                        .vram
                        .get(bg3_vram_4bpp_base + 8)
                        .copied()
                        .unwrap_or(0);
                    let bg3_shift = (bg3_tile0_num & 1) as u8 * 2;
                    eprint!(
                    "[gpu-dbg] BG3 atlas: slot={} vram_4bpp_base={} w01={:#06x} w23={:#06x} shift={} → GPU 2bpp row0:",
                    bg3_slot, bg3_vram_4bpp_base, bg3_4bpp_w01, bg3_4bpp_w23, bg3_shift
                );
                    for px in 0..8usize {
                        let bit = 7 - px as u8;
                        let bp0 = if bg3_shift == 0 {
                            (bg3_4bpp_w01 & 0xff) as u8
                        } else {
                            (bg3_4bpp_w23 & 0xff) as u8
                        };
                        let bp1 = if bg3_shift == 0 {
                            (bg3_4bpp_w01 >> 8) as u8
                        } else {
                            (bg3_4bpp_w23 >> 8) as u8
                        };
                        let raw_4bpp = ((bp0 >> bit) & 1) | (((bp1 >> bit) & 1) << 1);
                        let bp2 = if bg3_shift == 0 {
                            (bg3_4bpp_w23 & 0xff) as u8
                        } else {
                            (bg3_4bpp_w01 & 0xff) as u8
                        };
                        let bp3 = if bg3_shift == 0 {
                            (bg3_4bpp_w23 >> 8) as u8
                        } else {
                            (bg3_4bpp_w01 >> 8) as u8
                        };
                        let full = raw_4bpp | (((bp2 >> bit) & 1) << 2) | (((bp3 >> bit) & 1) << 3);
                        eprint!(" raw4={} shift2={}", full, (full >> bg3_shift) & 3);
                    }
                    eprintln!();
                }
                if should_dump_render_hash {
                    if let Some((_, dump_path)) = render_hash_dump_frame.as_ref() {
                        write_replay_save_render_hash_gpu_dump_or_exit(
                            dump_path,
                            &render_hash_capture,
                            &mut gpu_readback,
                            width,
                        );
                    }
                }
                if should_log_render_hash {
                    println!("{}", replay_cpu_bgra_hash_line(frames, frame));
                    if std::env::var_os("ZELDA3_PPU_STATE_HASH").is_some() {
                        let fnv16 = |s: &[u16]| {
                            let mut h = 2166136261u32;
                            for &w in s {
                                let [lo, hi] = w.to_le_bytes();
                                h = h.wrapping_mul(16777619) ^ u32::from(lo);
                                h = h.wrapping_mul(16777619) ^ u32::from(hi);
                            }
                            h
                        };
                        let mut dh = 2166136261u32;
                        for ch in &game.dma.channel {
                            for b in [
                                ch.b_adr,
                                ch.a_bank,
                                ch.ind_bank,
                                ch.rep_count,
                                ch.unused_byte,
                                ch.off_index,
                                ch.mode,
                                ch.hdma_active as u8,
                                ch.indirect as u8,
                                ch.do_transfer as u8,
                                ch.terminated as u8,
                                ch.from_b as u8,
                                (ch.a_adr & 0xff) as u8,
                                (ch.a_adr >> 8) as u8,
                                (ch.size & 0xff) as u8,
                                (ch.size >> 8) as u8,
                                (ch.table_adr & 0xff) as u8,
                                (ch.table_adr >> 8) as u8,
                            ] {
                                dh = dh.wrapping_mul(16777619) ^ u32::from(b);
                            }
                        }
                        println!(
                        "ppu-state frame={frames} vram=0x{:08x} cgram=0x{:08x} oam=0x{:08x} dma=0x{:08x} fblank={} bright={} math={:02x} winsel={:08x}",
                        fnv16(&game.ppu.vram),
                        fnv16(&game.ppu.cgram),
                        fnv16(&game.ppu.oam),
                        dh,
                        game.ppu.forced_blank,
                        game.ppu.brightness,
                        game.ppu.math_enabled,
                        game.ppu.windowsel,
                    );
                    }
                    // GPU tile renderer path — parallel log line for comparison.
                    // Does not affect the parity gate (different prefix).
                    if frames == 1000 {
                        eprintln!("[gpu-dbg] frame=1000 BG scrolls:");
                        for i in 0..4 {
                            eprintln!(
                            "[gpu-dbg]   bg{}: h_scroll={} v_scroll={} tilemap_adr={} tile_adr={} wider={} higher={}",
                            i + 1,
                            game.ppu.bg_layer[i].h_scroll,
                            game.ppu.bg_layer[i].v_scroll,
                            game.ppu.bg_layer[i].tilemap_adr,
                            game.ppu.bg_layer[i].tile_adr,
                            game.ppu.bg_layer[i].tilemap_wider,
                            game.ppu.bg_layer[i].tilemap_higher
                        );
                        }
                        // Sample pixels at row 0, x=0..15 from CPU output
                        eprint!("[gpu-dbg] CPU row0 x=0..15:");
                        for x in 0..16usize {
                            let px = &rgba[x * 4..x * 4 + 3];
                            eprint!(" ({},{},{})", px[0], px[1], px[2]);
                        }
                        eprintln!();
                        // BG2 tilemap entries 0..7 (first 8 tiles of row 0)
                        let bg2_tilemap_adr = game.ppu.bg_layer[1].tilemap_adr as usize;
                        eprint!("[gpu-dbg] BG2 tilemap row0 tiles 0..7:");
                        for tx in 0..8usize {
                            let entry = game
                                .ppu
                                .vram
                                .get(bg2_tilemap_adr + tx)
                                .copied()
                                .unwrap_or(0);
                            let tile_num = entry & 0x3ff;
                            let palette_sub = (entry >> 10) & 7;
                            eprint!(" tile={} pal={}", tile_num, palette_sub);
                        }
                        eprintln!();
                        // BG2 tile 0 pixel data (tile_adr=8192)
                        let bg2_tile_adr = game.ppu.bg_layer[1].tile_adr as usize;
                        let tile_num0 =
                            game.ppu.vram.get(bg2_tilemap_adr).copied().unwrap_or(0) & 0x3ff;
                        let _tbase = bg2_tile_adr / 2 + tile_num0 as usize * 8; // word offset (tile_adr is a byte addr)
                        let tbase_w = bg2_tile_adr + tile_num0 as usize * 16; // VRAM word index
                        eprintln!(
                            "[gpu-dbg] BG2 tile_adr={} tile_num0={} tbase_w={}",
                            bg2_tile_adr, tile_num0, tbase_w
                        );
                        eprint!("[gpu-dbg] BG2 tile0 row0 palette indices (CPU compute):");
                        let w01 = game.ppu.vram.get(tbase_w).copied().unwrap_or(0);
                        let w23 = game.ppu.vram.get(tbase_w + 8).copied().unwrap_or(0);
                        for px in 0..8usize {
                            let bit = 7 - px;
                            let bp0 = (w01 & 0xff) as u8;
                            let bp1 = (w01 >> 8) as u8;
                            let bp2 = (w23 & 0xff) as u8;
                            let bp3 = (w23 >> 8) as u8;
                            let idx = ((bp0 >> bit) & 1)
                                | (((bp1 >> bit) & 1) << 1)
                                | (((bp2 >> bit) & 1) << 2)
                                | (((bp3 >> bit) & 1) << 3);
                            eprint!(" {}", idx);
                        }
                        eprintln!();
                        eprintln!("[gpu-dbg] BG2 tile0 w01={:#06x} w23={:#06x}", w01, w23);
                    }
                    if frames == 8000 {
                        // Compare CPU vs GPU pixel at center of screen (128, 112)
                        let cx = 128i32;
                        let cy = 112i32;
                        let cpu_px = &rgba[cy as usize * 256 * 4 + cx as usize * 4
                            ..cy as usize * 256 * 4 + cx as usize * 4 + 4];
                        eprintln!(
                            "[gpu-dbg] CPU pixel ({cx},{cy}): R={} G={} B={} A={}",
                            cpu_px[0], cpu_px[1], cpu_px[2], cpu_px[3]
                        );
                        eprintln!(
                            "[gpu-dbg] bg1 tile_adr={} tilemap_adr={}",
                            game.ppu.bg_layer[0].tile_adr, game.ppu.bg_layer[0].tilemap_adr
                        );
                        eprintln!(
                            "[gpu-dbg] bg2 tile_adr={} tilemap_adr={}",
                            game.ppu.bg_layer[1].tile_adr, game.ppu.bg_layer[1].tilemap_adr
                        );
                        eprintln!(
                            "[gpu-dbg] obj tile_adr1={} tile_adr2={} obj_size={}",
                            game.ppu.obj_tile_adr1, game.ppu.obj_tile_adr2, game.ppu.obj_size
                        );
                        // Scan OAM to find sprites covering (cx, cy) and dump their data
                        let obj_sizes = [
                            [8u32, 16],
                            [8, 32],
                            [8, 64],
                            [16, 32],
                            [16, 64],
                            [32, 64],
                            [16, 32],
                            [16, 32],
                        ];
                        let sizes = obj_sizes[game.ppu.obj_size as usize & 7];
                        for sprite_num in 0..128usize {
                            let idx = sprite_num * 2;
                            let oam0 = game.ppu.oam.get(idx).copied().unwrap_or(0);
                            let oam1 = game.ppu.oam.get(idx + 1).copied().unwrap_or(0);
                            let hi_word = game.ppu.oam.get(0x100 + idx / 16).copied().unwrap_or(0);
                            let hi_bits = (hi_word >> (idx % 16)) as i32;
                            let x_high = hi_bits & 1;
                            let size_bit = (hi_bits >> 1) & 1;
                            let x_low = (oam0 & 0xFF) as i32;
                            let y_pos = ((oam0 >> 8) & 0xFF) as i32;
                            let y_screen = (y_pos + 1) & 0xFF;
                            if y_screen == 0xF0 {
                                continue;
                            }
                            let x_screen = {
                                let xs = x_low + x_high * 256;
                                if xs >= 256 {
                                    xs - 512
                                } else {
                                    xs
                                }
                            };
                            let sprite_size = sizes[size_bit as usize] as i32;
                            if cx >= x_screen
                                && cx < x_screen + sprite_size
                                && cy >= y_screen
                                && cy < y_screen + sprite_size
                            {
                                let tile_byte = (oam1 & 0xFF) as u32;
                                // SNES OAM attr byte = YXppccct: t=name_bit(bit0), ccc=palette(bits3-1)
                                let name_bit = (oam1 & 0x0100) != 0; // bit 0 of attr
                                let palette_sub = ((oam1 & 0x0e00) >> 9) as u32; // bits 3-1 of attr
                                let hflip = (oam1 & 0x4000) != 0;
                                let vflip = (oam1 & 0x8000) != 0;
                                let num_tiles = sprite_size / 8;
                                let gx = (cx - x_screen) / 8;
                                let gy = (cy - y_screen) / 8;
                                let tx_in_tile = (cx - x_screen) % 8;
                                let ty_in_tile = (cy - y_screen) % 8;
                                let tile_row_base = (tile_byte >> 4) as i32;
                                let tile_col_base = (tile_byte & 0x0F) as i32;
                                let src_gx = if hflip { num_tiles - 1 - gx } else { gx };
                                let src_gy = if vflip { num_tiles - 1 - gy } else { gy };
                                let tile_col = (tile_col_base + src_gx as i32) & 0x0F;
                                let tile_row = tile_row_base + src_gy as i32;
                                let tile_num = ((tile_row << 4) | tile_col) as u32;
                                let obj_addr = if name_bit {
                                    game.ppu.obj_tile_adr2
                                } else {
                                    game.ppu.obj_tile_adr1
                                };
                                let atlas_slot = u32::from(obj_addr) / 16 + tile_num;
                                let actual_tx = if hflip { 7 - tx_in_tile } else { tx_in_tile };
                                let actual_ty = if vflip { 7 - ty_in_tile } else { ty_in_tile };
                                let vram_base = atlas_slot as usize * 16;
                                let w01 = game
                                    .ppu
                                    .vram
                                    .get(vram_base + actual_ty as usize)
                                    .copied()
                                    .unwrap_or(0);
                                let w23 = game
                                    .ppu
                                    .vram
                                    .get(vram_base + 8 + actual_ty as usize)
                                    .copied()
                                    .unwrap_or(0);
                                let bp0 = (w01 & 0xFF) as u8;
                                let bp1 = (w01 >> 8) as u8;
                                let bp2 = (w23 & 0xFF) as u8;
                                let bp3 = (w23 >> 8) as u8;
                                let bit = 7 - actual_tx;
                                let pal_from_vram = ((bp0 >> bit) & 1)
                                    | (((bp1 >> bit) & 1) << 1)
                                    | (((bp2 >> bit) & 1) << 2)
                                    | (((bp3 >> bit) & 1) << 3);
                                eprintln!(
                                "[gpu-dbg] sprite#{sprite_num} covers ({cx},{cy}): x={x_screen} y={y_screen} size={sprite_size} tile={tile_byte:#x} name_bit={name_bit} palette_sub={palette_sub} hflip={hflip} vflip={vflip}"
                            );
                                eprintln!(
                                "[gpu-dbg]   gx={gx} gy={gy} tx={tx_in_tile} ty={ty_in_tile} src_gx={src_gx} src_gy={src_gy} tile_num={tile_num} atlas_slot={atlas_slot} actual_tx={actual_tx} actual_ty={actual_ty}"
                            );
                                eprintln!(
                                "[gpu-dbg]   vram_base={vram_base} w01={w01:#06x} w23={w23:#06x} bit={bit} bp0={bp0:#010b} bp1={bp1:#010b} bp2={bp2:#010b} bp3={bp3:#010b}"
                            );
                                eprintln!(
                                "[gpu-dbg]   pal_from_vram={pal_from_vram} cpu_expected_cgram_idx={} gpu_result_cgram_idx=252",
                                0x80u32 + 16 * palette_sub + u32::from(pal_from_vram)
                            );
                                // Dump full 8x8 tile for atlas_slot and atlas_slot-1 to find where pal_idx=11 is
                                for dump_slot in
                                    [atlas_slot.saturating_sub(1), atlas_slot, atlas_slot + 1]
                                {
                                    let dbase = dump_slot as usize * 16;
                                    eprint!(
                                    "[gpu-dbg]   tile slot={dump_slot} (vram_base={dbase}) palette indices (rows 0-7):"
                                );
                                    for dty in 0..8usize {
                                        let dw01 =
                                            game.ppu.vram.get(dbase + dty).copied().unwrap_or(0);
                                        let dw23 = game
                                            .ppu
                                            .vram
                                            .get(dbase + 8 + dty)
                                            .copied()
                                            .unwrap_or(0);
                                        let dbp0 = (dw01 & 0xFF) as u8;
                                        let dbp1 = (dw01 >> 8) as u8;
                                        let dbp2 = (dw23 & 0xFF) as u8;
                                        let dbp3 = (dw23 >> 8) as u8;
                                        eprint!(" [");
                                        for dtx in 0..8usize {
                                            let dbit = 7 - dtx;
                                            let idx = ((dbp0 >> dbit) & 1)
                                                | (((dbp1 >> dbit) & 1) << 1)
                                                | (((dbp2 >> dbit) & 1) << 2)
                                                | (((dbp3 >> dbit) & 1) << 3);
                                            eprint!("{idx}");
                                            if dtx < 7 {
                                                eprint!(",");
                                            }
                                        }
                                        eprint!("]");
                                    }
                                    eprintln!();
                                }
                            }
                        }
                    }
                    if frames == 8000 {
                        eprintln!("{}", render_hash_capture.debug_math_state_line());
                        eprintln!(
                        "[gpu-dbg] ppu.math_enabled={:#04x} ppu.add_subscreen={} ppu.subtract={} ppu.prevent_math_mode={}",
                        game.ppu.math_enabled,
                        game.ppu.add_subscreen,
                        game.ppu.subtract_color,
                        game.ppu.prevent_math_mode
                    );
                    }
                    let gpu_rgba = render_hash_capture.render_gpu_rgba(&mut gpu_readback);
                    if frames == 8000 {
                        let cx = 128usize;
                        let cy = 112usize;
                        let gpu_px = &gpu_rgba[cy * 256 * 4 + cx * 4..cy * 256 * 4 + cx * 4 + 4];
                        eprintln!(
                            "[gpu-dbg] GPU pixel ({cx},{cy}): R={} G={} B={} A={}",
                            gpu_px[0], gpu_px[1], gpu_px[2], gpu_px[3]
                        );
                    }
                    if frames == 332 {
                        // Print math/window params
                        eprintln!("{}", render_hash_capture.debug_frame_332_math_line());
                        eprintln!(
                        "[gpu-dbg] frame=332 ppu: math_enabled={:#04x} clip_mode={} prevent_math={} windowsel={:#010x} w1l={} w1r={}",
                        game.ppu.math_enabled,
                        game.ppu.clip_mode,
                        game.ppu.prevent_math_mode,
                        game.ppu.windowsel,
                        game.ppu.window1_left,
                        game.ppu.window1_right
                    );
                        // Print scanline 0 window params
                        eprintln!(
                            "{}",
                            render_hash_capture.debug_frame_332_scanline_window_line()
                        );
                        // Find CGRAM entry = 0x014D (the mystery color R=13,G=10,B=0)
                        for line in render_hash_capture.debug_cgram_value_lines(
                            frames,
                            "hdma_cgram",
                            0x014d,
                        ) {
                            eprintln!("{line}");
                        }
                        for (ci, &cv) in game.ppu.cgram.iter().enumerate() {
                            if cv == 0x014d {
                                eprintln!("[gpu-dbg] frame=332 post_cgram[{ci}]=0x014d");
                            }
                        }
                        // Print initial screen_enabled (before render) vs post-render
                        eprintln!(
                        "[gpu-dbg] frame=332 pre-render screen_enabled[0]={:#04x} post-render={:#04x}",
                        pre_screen_enabled, game.ppu.screen_enabled[0]
                    );
                        // Print obj_size to help debug sprite scan
                        eprintln!(
                        "[gpu-dbg] frame=332 obj_size={} obj_tile_adr1={:#06x} obj_tile_adr2={:#06x}",
                        game.ppu.obj_size, game.ppu.obj_tile_adr1, game.ppu.obj_tile_adr2
                    );
                        let sizes = [
                            [8u32, 16u32],
                            [8, 32],
                            [8, 64],
                            [16, 32],
                            [16, 64],
                            [32, 64],
                            [16, 32],
                            [16, 32],
                        ];
                        let objsz = sizes[game.ppu.obj_size as usize & 7];
                        // Print first 5 sprites with non-0xF0 y_screen
                        let mut printed = 0;
                        for sprite_num in 0..128usize {
                            let idx = sprite_num * 2;
                            let oam0 = game.ppu.oam.get(idx).copied().unwrap_or(0);
                            let hi_word = game.ppu.oam.get(0x100 + idx / 16).copied().unwrap_or(0);
                            let hi_bits = (hi_word >> (idx % 16)) as i32;
                            let size_bit = ((hi_bits >> 1) & 1) as usize;
                            let x_high = hi_bits & 1;
                            let x_low = (oam0 & 0xFF) as i32;
                            let y_pos = ((oam0 >> 8) & 0xFF) as i32;
                            let y_screen = (y_pos + 1) & 0xFF;
                            if y_screen == 0xF0 {
                                continue;
                            }
                            let sprite_size = objsz[size_bit] as i32;
                            let x_screen = {
                                let xs = x_low + x_high * 256;
                                if xs >= 256 {
                                    xs - 512
                                } else {
                                    xs
                                }
                            };
                            if printed < 5 {
                                eprintln!(
                                "[gpu-dbg] frame=332 sprite#{sprite_num}: x={x_screen} y_pos={y_pos} y_screen={y_screen} size={sprite_size}"
                            );
                                printed += 1;
                            }
                        }
                        // Print raw BGRA bytes at (126,0)
                        let i = 126usize;
                        eprintln!(
                            "[gpu-dbg] frame=332 raw frame[126,0]: B={} G={} R={} A={}",
                            frame[i * 4],
                            frame[i * 4 + 1],
                            frame[i * 4 + 2],
                            frame[i * 4 + 3]
                        );
                        // Dump a range around (126,0) in the CPU frame
                        eprint!("[gpu-dbg] frame=332 CPU x=120..135 y=0:");
                        for x in 120..136usize {
                            let r = frame[x * 4 + 2];
                            let g = frame[x * 4 + 1];
                            let b = frame[x * 4];
                            eprint!(" ({r},{g},{b})");
                        }
                        eprintln!();
                        eprint!("[gpu-dbg] frame=332 CPU x=120..135 y=1:");
                        for x in 120..136usize {
                            let r = frame[(256 + x) * 4 + 2];
                            let g = frame[(256 + x) * 4 + 1];
                            let b = frame[(256 + x) * 4];
                            eprint!(" ({r},{g},{b})");
                        }
                        eprintln!();
                        // Quick OAM scan for sprite covering (126, 0)
                        let cx = 126i32;
                        let cy = 0i32;
                        let sizes = [
                            [8u32, 16],
                            [8, 32],
                            [8, 64],
                            [16, 32],
                            [16, 64],
                            [32, 64],
                            [16, 32],
                            [16, 32],
                        ];
                        let sizes = sizes[game.ppu.obj_size as usize & 7];
                        for sprite_num in 0..128usize {
                            let idx = sprite_num * 2;
                            let oam0 = game.ppu.oam.get(idx).copied().unwrap_or(0);
                            let oam1 = game.ppu.oam.get(idx + 1).copied().unwrap_or(0);
                            let hi_word = game.ppu.oam.get(0x100 + idx / 16).copied().unwrap_or(0);
                            let hi_bits = (hi_word >> (idx % 16)) as i32;
                            let x_high = hi_bits & 1;
                            let size_bit = (hi_bits >> 1) & 1;
                            let x_low = (oam0 & 0xFF) as i32;
                            let y_pos = ((oam0 >> 8) & 0xFF) as i32;
                            let y_screen = (y_pos + 1) & 0xFF;
                            if y_screen == 0xF0 {
                                continue;
                            }
                            let x_screen = {
                                let xs = x_low + x_high * 256;
                                if xs >= 256 {
                                    xs - 512
                                } else {
                                    xs
                                }
                            };
                            let sprite_size = sizes[size_bit as usize] as i32;
                            // CPU check: occupies output rows (y_screen-1)..(y_screen-1+sprite_size), but y_screen can be 0 meaning row -1..size-2
                            let top_row = y_screen - 1; // output row where sprite top lands (can be -1)
                            let bot_row = top_row + sprite_size;
                            if cx >= x_screen
                                && cx < x_screen + sprite_size
                                && cy >= top_row
                                && cy < bot_row
                            {
                                eprintln!(
                                "[gpu-dbg] frame=332 sprite#{sprite_num} covers ({cx},{cy}): y_pos={y_pos} y_screen={y_screen} top_row={top_row} x_screen={x_screen} size={sprite_size} tile={:#04x}",
                                oam1 & 0xFF
                            );
                            }
                        }
                    }
                    if frames == 733 || frames == 800 || frames == 900 || frames == 1050 {
                        let mut ndiff_top = 0usize;
                        let mut ndiff_bot = 0usize;
                        let mut max_shown = 3usize;
                        for i in 0..256 * 224 {
                            let cr = frame[i * 4 + 2];
                            let cg = frame[i * 4 + 1];
                            let cb = frame[i * 4];
                            let gr = gpu_rgba[i * 4];
                            let gg = gpu_rgba[i * 4 + 1];
                            let gb = gpu_rgba[i * 4 + 2];
                            if cr != gr || cg != gg || cb != gb {
                                let row = i / 256;
                                if row < 128 {
                                    ndiff_top += 1;
                                } else {
                                    ndiff_bot += 1;
                                }
                                if max_shown > 0 {
                                    eprintln!(
                                    "[gpu-dbg] f{frames} diff  ({},{}) cpu=({},{},{}) gpu=({},{},{})",
                                    i % 256,
                                    row,
                                    cr,
                                    cg,
                                    cb,
                                    gr,
                                    gg,
                                    gb
                                );
                                    max_shown -= 1;
                                }
                            }
                        }
                        eprintln!(
                        "[gpu-dbg] f{frames} total_ndiff={} top={} bot={} screen_enabled=[{:#04x},{:#04x}] screen_windowed=[{:#04x},{:#04x}] windowsel={:#010x}",
                        ndiff_top + ndiff_bot,
                        ndiff_top,
                        ndiff_bot,
                        game.ppu.screen_enabled[0],
                        game.ppu.screen_enabled[1],
                        game.ppu.screen_windowed[0],
                        game.ppu.screen_windowed[1],
                        game.ppu.windowsel
                    );
                        eprintln!(
                            "{}",
                            render_hash_capture.debug_effect_math_line(
                                frames,
                                game.ppu.bg_layer[0].h_scroll,
                                game.ram[0xf9],
                            )
                        );
                        if frames == 900 || frames == 1050 {
                            let (cx, cy) = match frames {
                                900 => (127i32, 56i32),
                                1050 => (40i32, 40i32),
                                _ => unreachable!(),
                            };
                            eprintln!("[gpu-dbg] f{frames} probe ({cx},{cy})");
                            eprintln!(
                                "{}",
                                render_hash_capture.debug_scanline_tm_probe_line(frames, cy)
                            );
                            let ppu_x = (cx + PPU_EXTRA_LEFT_RIGHT as i32) as usize;
                            let main_z =
                                game.ppu.bg_buffers[0].data.get(ppu_x).copied().unwrap_or(0);
                            let sub_z =
                                game.ppu.bg_buffers[1].data.get(ppu_x).copied().unwrap_or(0);
                            let obj_z = game.ppu.obj_buffer.data.get(ppu_x).copied().unwrap_or(0);
                            eprintln!(
                            "[gpu-dbg] f{frames} CPU buffers@({cx},{cy}): main={:#06x} layer={} cgram[{}]={:#06x} sub={:#06x} obj={:#06x}",
                            main_z,
                            (main_z >> 8) & 0x0f,
                            main_z & 0xff,
                            game.ppu
                                .cgram
                                .get((main_z & 0xff) as usize)
                                .copied()
                                .unwrap_or(0),
                            sub_z,
                            obj_z
                        );
                            for layer in 0..3usize {
                                let bg = &game.ppu.bg_layer[layer];
                                let map_pw = if bg.tilemap_wider { 512u32 } else { 256u32 };
                                let map_ph = if bg.tilemap_higher { 512u32 } else { 256u32 };
                                let sx = (cx as u32 + bg.h_scroll as u32) % map_pw;
                                let sy = (cy as u32 + bg.v_scroll as u32 + 1) % map_ph;
                                let tile_x = sx / 8;
                                let tile_y = sy / 8;
                                let page_offset = if tile_x >= 32 && bg.tilemap_wider {
                                    0x400
                                } else {
                                    0
                                } + if tile_y >= 32 && bg.tilemap_higher {
                                    if bg.tilemap_wider {
                                        0x800
                                    } else {
                                        0x400
                                    }
                                } else {
                                    0
                                };
                                let vram_idx = bg.tilemap_adr as u32
                                    + page_offset
                                    + (tile_y & 0x1f) * 32
                                    + (tile_x & 0x1f);
                                let entry =
                                    game.ppu.vram.get(vram_idx as usize).copied().unwrap_or(0);
                                let tile_num = entry & 0x03ff;
                                let pal_sub = (entry >> 10) & 7;
                                let prio = (entry >> 13) & 1;
                                let hflip = (entry >> 14) & 1;
                                let vflip = (entry >> 15) & 1;
                                let px = if hflip != 0 { 7 - (sx % 8) } else { sx % 8 };
                                let py = if vflip != 0 { 7 - (sy % 8) } else { sy % 8 };
                                let (pal_idx, cgram_idx) = if frames == 1050 && layer == 2 {
                                    let tile_base = bg.tile_adr as u32 + tile_num as u32 * 8 + py;
                                    let w01 = game
                                        .ppu
                                        .vram
                                        .get(tile_base as usize & 0x7fff)
                                        .copied()
                                        .unwrap_or(0);
                                    let bit = 7 - px;
                                    let pal_idx =
                                        ((w01 >> bit) & 1) | (((w01 >> (8 + bit)) & 1) << 1);
                                    (pal_idx, pal_sub * 4 + pal_idx)
                                } else if layer == 2 {
                                    let tile_base = bg.tile_adr as u32 + tile_num as u32 * 8 + py;
                                    let w01 = game
                                        .ppu
                                        .vram
                                        .get(tile_base as usize & 0x7fff)
                                        .copied()
                                        .unwrap_or(0);
                                    let bit = 7 - px;
                                    let pal_idx =
                                        ((w01 >> bit) & 1) | (((w01 >> (8 + bit)) & 1) << 1);
                                    (pal_idx, pal_sub * 4 + pal_idx)
                                } else {
                                    let tile_base = bg.tile_adr as u32 + tile_num as u32 * 16 + py;
                                    let w01 = game
                                        .ppu
                                        .vram
                                        .get(tile_base as usize & 0x7fff)
                                        .copied()
                                        .unwrap_or(0);
                                    let w23 = game
                                        .ppu
                                        .vram
                                        .get((tile_base + 8) as usize & 0x7fff)
                                        .copied()
                                        .unwrap_or(0);
                                    let bit = 7 - px;
                                    let pal_idx = ((w01 >> bit) & 1)
                                        | (((w01 >> (8 + bit)) & 1) << 1)
                                        | (((w23 >> bit) & 1) << 2)
                                        | (((w23 >> (8 + bit)) & 1) << 3);
                                    (pal_idx, 16 * pal_sub + pal_idx)
                                };
                                eprintln!(
                                "[gpu-dbg] f{frames} BG{}@({cx},{cy}): enabled_main={} tilemap={} tile_adr={} entry={:#06x} tile={} pal_sub={} prio={} px={} py={} pal_idx={} cgram[{}]={}",
                                layer + 1,
                                (game.ppu.screen_enabled[0] & (1 << layer)) != 0,
                                bg.tilemap_adr,
                                bg.tile_adr,
                                entry,
                                tile_num,
                                pal_sub,
                                prio,
                                px,
                                py,
                                pal_idx,
                                cgram_idx,
                                render_hash_capture.cgram_color_hex(cgram_idx as usize)
                            );
                            }

                            let obj_sizes = [
                                [8i32, 16],
                                [8, 32],
                                [8, 64],
                                [16, 32],
                                [16, 64],
                                [32, 64],
                                [16, 32],
                                [16, 32],
                            ];
                            let sizes = obj_sizes[(game.ppu.obj_size & 7) as usize];
                            for sprite_num in 0..128usize {
                                let idx = sprite_num * 2;
                                let oam0 = game.ppu.oam.get(idx).copied().unwrap_or(0);
                                let oam1 = game.ppu.oam.get(idx + 1).copied().unwrap_or(0);
                                let hi_word =
                                    game.ppu.oam.get(0x100 + idx / 16).copied().unwrap_or(0);
                                let hi_bits = (hi_word >> (idx % 16)) as i32;
                                let x_high = hi_bits & 1;
                                let size_bit = ((hi_bits >> 1) & 1) as usize;
                                let x_low = (oam0 & 0xff) as i32;
                                let y_pos = ((oam0 >> 8) & 0xff) as i32;
                                let y_screen = (y_pos + 1) & 0xff;
                                if y_screen == 0xf0 {
                                    continue;
                                }
                                let x_screen = x_low + x_high * 256;
                                let x_screen = if x_screen >= 256 {
                                    x_screen - 512
                                } else {
                                    x_screen
                                };
                                let y_base = y_pos;
                                let sprite_size = sizes[size_bit];
                                if cx < x_screen
                                    || cx >= x_screen + sprite_size
                                    || cy < y_base
                                    || cy >= y_base + sprite_size
                                {
                                    continue;
                                }

                                let row = (cy + 1 - y_screen) & 0xff;
                                if row >= sprite_size {
                                    continue;
                                }
                                let row = if oam1 & 0x8000 != 0 {
                                    sprite_size - 1 - row
                                } else {
                                    row
                                };
                                let col = ((cx - x_screen) / 8) * 8;
                                let used_col = if oam1 & 0x4000 != 0 {
                                    sprite_size - 1 - col
                                } else {
                                    col
                                };
                                let used_tile = ((((oam1 & 0xff) >> 4) as i32 + (row >> 3)) << 4)
                                    | ((((oam1 & 0x0f) as i32) + (used_col >> 3)) & 0x0f);
                                let obj_addr = if oam1 & 0x0100 != 0 {
                                    game.ppu.obj_tile_adr2
                                } else {
                                    game.ppu.obj_tile_adr1
                                };
                                let addr = obj_addr
                                    .wrapping_add((used_tile as u16).wrapping_mul(16))
                                    .wrapping_add((row & 7) as u16)
                                    & 0x7fff;
                                let plane = game.ppu.vram.get(addr as usize).copied().unwrap_or(0)
                                    as u32
                                    | ((game
                                        .ppu
                                        .vram
                                        .get(addr.wrapping_add(8) as usize & 0x7fff)
                                        .copied()
                                        .unwrap_or(0)
                                        as u32)
                                        << 16);
                                let px = cx - (x_screen + col);
                                let shift = if oam1 & 0x4000 != 0 { px } else { 7 - px };
                                let bits = plane >> shift;
                                let pixel = (bits & 1)
                                    | ((bits >> 7) & 2)
                                    | ((bits >> 14) & 4)
                                    | ((bits >> 21) & 8);
                                let palette_sub = ((oam1 & 0x0e00) >> 9) as u32;
                                let cgram_idx = 0x80 + 16 * palette_sub + pixel;
                                eprintln!(
                                "[gpu-dbg] f{frames} sprite#{} covers ({cx},{cy}): x={} y_base={} size={} oam1={:#06x} prio={} pal_sub={} row={} col={} used_tile={:#04x} pixel={} cgram[{}]={}",
                                sprite_num,
                                x_screen,
                                y_base,
                                sprite_size,
                                oam1,
                                (oam1 & 0x3000) >> 12,
                                palette_sub,
                                row,
                                col,
                                used_tile,
                                pixel,
                                cgram_idx,
                                render_hash_capture.cgram_color_hex(cgram_idx as usize)
                            );
                            }
                        }
                        if frames == 800 {
                            // BG3 lookup at (126,65) to check if it has a hi-priority tile that wins
                            let bg3 = &game.ppu.bg_layer[2];
                            let bg3_hscroll = bg3.h_scroll as u32;
                            let bg3_vscroll = bg3.v_scroll as u32;
                            let map_pw = if bg3.tilemap_wider { 512u32 } else { 256u32 };
                            let map_ph = if bg3.tilemap_higher { 512u32 } else { 256u32 };
                            let bg3_sx = (126u32 + bg3_hscroll) % map_pw;
                            let bg3_sy = (65u32 + bg3_vscroll + 1) % map_ph;
                            let bg3_tw = if bg3.tilemap_wider { 64u32 } else { 32u32 };
                            let bg3_flat = (bg3_sy / 8) * bg3_tw + (bg3_sx / 8);
                            let bg3_vram_idx = bg3.tilemap_adr as u32 + bg3_flat;
                            let bg3_entry = game
                                .ppu
                                .vram
                                .get(bg3_vram_idx as usize)
                                .copied()
                                .unwrap_or(0);
                            let bg3_tile_num = bg3_entry & 0x3FF;
                            let bg3_pal_sub = (bg3_entry >> 10) & 7;
                            let bg3_prio = (bg3_entry >> 13) & 1;
                            let bg3_hflip = (bg3_entry >> 14) & 1;
                            let bg3_vflip = (bg3_entry >> 15) & 1;
                            let bg3_px = if bg3_hflip != 0 {
                                7 - (bg3_sx % 8)
                            } else {
                                bg3_sx % 8
                            };
                            let bg3_py = if bg3_vflip != 0 {
                                7 - (bg3_sy % 8)
                            } else {
                                bg3_sy % 8
                            };
                            // 2bpp: 8 words per tile (bp0+bp1 only, no bp2/bp3)
                            let bg3_tile_base = bg3.tile_adr as u32 + bg3_tile_num as u32 * 8;
                            let bg3_w01 = game
                                .ppu
                                .vram
                                .get((bg3_tile_base + bg3_py) as usize & 0x7fff)
                                .copied()
                                .unwrap_or(0);
                            let bg3_bit = 7 - bg3_px;
                            let bg3_pal_idx = ((bg3_w01 >> bg3_bit) & 1)
                                | (((bg3_w01 >> (8 + bg3_bit)) & 1) << 1);
                            let bg3_cgram_idx = (bg3_pal_sub * 4 + bg3_pal_idx) as usize;
                            let bg3_cgram_val = render_hash_capture.cgram_color_hex(bg3_cgram_idx);
                            eprintln!(
                            "[gpu-dbg] f800 BG3@(126,65): tilemap_adr={} tile={} pal_sub={} prio={} pal_idx={} cgram[{}]={}",
                            bg3.tilemap_adr,
                            bg3_tile_num,
                            bg3_pal_sub,
                            bg3_prio,
                            bg3_pal_idx,
                            bg3_cgram_idx,
                            bg3_cgram_val
                        );
                            // GPU atlas slot for BG3 2bpp: atlas_slot = tile_adr/16 + tile_num/2
                            let bg3_atlas_slot = bg3.tile_adr as u32 / 16 + bg3_tile_num as u32 / 2;
                            let bg3_atlas_sub = bg3_tile_num % 2; // 0 = lo half, 1 = hi half
                            let bg3_vram_4bpp_base = bg3_atlas_slot * 16;
                            let bg3_4bpp_w01 = game
                                .ppu
                                .vram
                                .get((bg3_vram_4bpp_base + bg3_py) as usize & 0x7fff)
                                .copied()
                                .unwrap_or(0);
                            let bg3_4bpp_w23 = game
                                .ppu
                                .vram
                                .get((bg3_vram_4bpp_base + 8 + bg3_py) as usize & 0x7fff)
                                .copied()
                                .unwrap_or(0);
                            // GPU 2bpp reads from the higher 2 bits of the atlas 4bpp entry if tile_num is odd
                            let (gpu_w_lo, _gpu_w_hi) = if bg3_atlas_sub == 0 {
                                (bg3_4bpp_w01, bg3_4bpp_w23)
                            } else {
                                (bg3_4bpp_w23, bg3_4bpp_w01)
                            };
                            let gpu_bit = 7 - bg3_px;
                            let gpu_pal_idx = ((gpu_w_lo >> gpu_bit) & 1)
                                | (((gpu_w_lo >> (8 + gpu_bit)) & 1) << 1);
                            let gpu_bg3_cgram = bg3_pal_sub * 4 + gpu_pal_idx; // 2bpp: 4 colors per sub-palette
                            eprintln!(
                            "[gpu-dbg] f800 GPU BG3@(126,65): atlas_slot={} sub={} gpu_pal_idx={} gpu_cgram_idx={}",
                            bg3_atlas_slot, bg3_atlas_sub, gpu_pal_idx, gpu_bg3_cgram
                        );
                            // Also log the specific GPU cgram value
                            let gpu_bg3_cgram_val =
                                render_hash_capture.cgram_color_hex(gpu_bg3_cgram as usize);
                            eprintln!(
                                "[gpu-dbg] f800 GPU BG3 cgram[{}]={}",
                                gpu_bg3_cgram, gpu_bg3_cgram_val
                            );
                            // Check if any sprite covers (126,65) — that would explain the gold pixel
                            let obj_sizes = [
                                [8i32, 16],
                                [8, 32],
                                [8, 64],
                                [16, 32],
                                [16, 64],
                                [32, 64],
                                [16, 32],
                                [16, 32],
                            ];
                            let sizes = obj_sizes[(game.ppu.obj_size & 7) as usize];
                            eprintln!(
                                "[gpu-dbg] f800 obj_size={} tile_adr1={:#06x} tile_adr2={:#06x}",
                                game.ppu.obj_size, game.ppu.obj_tile_adr1, game.ppu.obj_tile_adr2
                            );
                            for sprite_num in 0..128usize {
                                let idx = sprite_num * 2;
                                let oam0 = game.ppu.oam.get(idx).copied().unwrap_or(0);
                                let oam1 = game.ppu.oam.get(idx + 1).copied().unwrap_or(0);
                                let hi_word =
                                    game.ppu.oam.get(0x100 + idx / 16).copied().unwrap_or(0);
                                let hi_bits = (hi_word >> (idx % 16)) as i32;
                                let x_high = hi_bits & 1;
                                let size_bit = ((hi_bits >> 1) & 1) as usize;
                                let x_low = (oam0 & 0xFF) as i32;
                                let y_pos = ((oam0 >> 8) & 0xFF) as i32;
                                let y_screen = (y_pos + 1) & 0xFF;
                                if y_screen == 0xF0 {
                                    continue;
                                }
                                let x_screen = x_low + x_high * 256;
                                let x_screen = if x_screen >= 256 {
                                    x_screen - 512
                                } else {
                                    x_screen
                                };
                                let y_base = if y_screen == 0 { y_pos - 256 } else { y_pos };
                                let sprite_size = sizes[size_bit];
                                // Check if (126,65) falls within this sprite
                                if x_screen <= 126
                                    && 126 < x_screen + sprite_size
                                    && y_base <= 65
                                    && 65 < y_base + sprite_size
                                {
                                    let tile_byte = (oam1 & 0xFF) as i32;
                                    let palette_sub = ((oam1 & 0x0e00) >> 9) as u32;
                                    let row = 65 - y_screen;
                                    let row = row & 0xff;
                                    let row = if oam1 & 0x8000 != 0 {
                                        sprite_size - 1 - row
                                    } else {
                                        row
                                    };
                                    let col = ((126 - x_screen) / 8) * 8;
                                    let used_col = if oam1 & 0x4000 != 0 {
                                        sprite_size - 1 - col
                                    } else {
                                        col
                                    };
                                    let used_tile = ((((oam1 & 0xff) >> 4) as i32 + (row >> 3))
                                        << 4)
                                        | ((((oam1 & 0x0f) as i32) + (used_col >> 3)) & 0x0f);
                                    let obj_addr = if oam1 & 0x0100 != 0 {
                                        game.ppu.obj_tile_adr2
                                    } else {
                                        game.ppu.obj_tile_adr1
                                    };
                                    let addr = obj_addr
                                        .wrapping_add((used_tile as u16).wrapping_mul(16))
                                        .wrapping_add((row & 7) as u16)
                                        & 0x7fff;
                                    let plane =
                                        game.ppu.vram.get(addr as usize).copied().unwrap_or(0)
                                            as u32
                                            | ((game
                                                .ppu
                                                .vram
                                                .get(addr.wrapping_add(8) as usize & 0x7fff)
                                                .copied()
                                                .unwrap_or(0)
                                                as u32)
                                                << 16);
                                    let px = 126 - (x_screen + col);
                                    let shift = if oam1 & 0x4000 != 0 { px } else { 7 - px };
                                    let bits = plane >> shift;
                                    let pixel = (bits & 1)
                                        | ((bits >> 7) & 2)
                                        | ((bits >> 14) & 4)
                                        | ((bits >> 21) & 8);
                                    let cgram_idx = 0x80 + 16 * palette_sub + pixel;
                                    let prio = (oam1 & 0x3000) >> 12;
                                    eprintln!(
                                    "[gpu-dbg] f800 sprite#{} covers (126,65): x={} y_base={} size={} tile={:#04x} pal_sub={} oam1={:#06x} prio={} row={} col={} used_tile={:#04x} addr={:#06x} plane={:#010x} px={} pixel={} cgram[{}]={:#06x}",
                                    sprite_num,
                                    x_screen,
                                    y_base,
                                    sprite_size,
                                    tile_byte,
                                    palette_sub,
                                    oam1,
                                    prio,
                                    row,
                                    col,
                                    used_tile,
                                    addr,
                                    plane,
                                    px,
                                    pixel,
                                    cgram_idx,
                                    game.ppu.cgram.get(cgram_idx as usize).copied().unwrap_or(0)
                                );
                                }
                            }
                        }
                    }
                    if frames == 675 {
                        // investigate (132,65) mismatch
                        let cpu_px = {
                            let i = 65 * 256 + 132;
                            (frame[i * 4 + 2], frame[i * 4 + 1], frame[i * 4])
                        };
                        let gpu_px = {
                            let i = 65 * 256 + 132;
                            (gpu_rgba[i * 4], gpu_rgba[i * 4 + 1], gpu_rgba[i * 4 + 2])
                        };
                        eprintln!(
                            "[gpu-dbg] f675 (132,65) cpu_rgb={:?} gpu_rgb={:?}",
                            cpu_px, gpu_px
                        );
                        eprintln!(
                        "[gpu-dbg] f675 add_subscreen={} screen_enabled[0]={:#04x} screen_enabled[1]={:#04x}",
                        game.ppu.add_subscreen,
                        game.ppu.screen_enabled[0],
                        game.ppu.screen_enabled[1]
                    );
                        eprintln!(
                        "[gpu-dbg] f675 math_enabled={:#04x} fixed=({},{},{}) subtract={} half={}",
                        game.ppu.math_enabled,
                        game.ppu.fixed_color_r,
                        game.ppu.fixed_color_g,
                        game.ppu.fixed_color_b,
                        game.ppu.subtract_color,
                        game.ppu.half_color
                    );
                        // Count diffs in this frame
                        let mut ndiff = 0usize;
                        for i in 0..256 * 224 {
                            let cr = frame[i * 4 + 2];
                            let cg = frame[i * 4 + 1];
                            let cb = frame[i * 4];
                            let gr = gpu_rgba[i * 4];
                            let gg = gpu_rgba[i * 4 + 1];
                            let gb = gpu_rgba[i * 4 + 2];
                            if cr != gr || cg != gg || cb != gb {
                                ndiff += 1;
                                if ndiff <= 12 {
                                    eprintln!(
                                    "[gpu-dbg] f675 diff#{} ({},{}) cpu=({},{},{}) gpu=({},{},{})",
                                    ndiff,
                                    i % 256,
                                    i / 256,
                                    cr,
                                    cg,
                                    cb,
                                    gr,
                                    gg,
                                    gb
                                );
                                }
                            }
                        }
                        eprintln!("[gpu-dbg] f675 total_ndiff={}", ndiff);
                    }
                    if frames == 332 {
                        // extra debug
                        eprintln!("{}", gpu_rgba.debug_hash_line_with_cpu_bgra(frames, frame));
                        eprintln!(
                            "[gpu-dbg] frame=332 ppu_mode={} screen_enabled={:#04x} brightness={}",
                            game.ppu.mode, game.ppu.screen_enabled[0], game.ppu.brightness
                        );
                        eprintln!(
                        "[gpu-dbg] frame=332 bg1: tilemap={} tile_adr={} h={} v={} wider={} higher={}",
                        game.ppu.bg_layer[0].tilemap_adr,
                        game.ppu.bg_layer[0].tile_adr,
                        game.ppu.bg_layer[0].h_scroll,
                        game.ppu.bg_layer[0].v_scroll,
                        game.ppu.bg_layer[0].tilemap_wider,
                        game.ppu.bg_layer[0].tilemap_higher
                    );
                        eprintln!(
                        "[gpu-dbg] frame=332 bg3: tilemap={} tile_adr={} h={} v={} wider={} higher={}",
                        game.ppu.bg_layer[2].tilemap_adr,
                        game.ppu.bg_layer[2].tile_adr,
                        game.ppu.bg_layer[2].h_scroll,
                        game.ppu.bg_layer[2].v_scroll,
                        game.ppu.bg_layer[2].tilemap_wider,
                        game.ppu.bg_layer[2].tilemap_higher
                    );
                        // Print first 16 pixels of CPU and GPU
                        eprint!("[gpu-dbg] frame=332 CPU row0:");
                        for x in 0..16usize {
                            let b = frame[x * 4] as i32;
                            let g_b = frame[x * 4 + 1] as i32;
                            let r = frame[x * 4 + 2] as i32;
                            eprint!(" ({},{},{})", r, g_b, b);
                        }
                        eprintln!();
                        eprint!("[gpu-dbg] frame=332 GPU row0:");
                        for x in 0..16usize {
                            let px = &gpu_rgba[x * 4..x * 4 + 3];
                            eprint!(" ({},{},{})", px[0], px[1], px[2]);
                        }
                        eprintln!();
                        // Count pixel diffs
                        let mut ndiff = 0usize;
                        for i in 0..256 * 224 {
                            let b = frame[i * 4] as u32;
                            let g_b = frame[i * 4 + 1] as u32;
                            let r = frame[i * 4 + 2] as u32;
                            let gr = gpu_rgba[i * 4] as u32;
                            let gg = gpu_rgba[i * 4 + 1] as u32;
                            let gb = gpu_rgba[i * 4 + 2] as u32;
                            let rr = ((r >> 3) << 3) | (r >> 3 >> 2);
                            let gg2 = ((g_b >> 3) << 3) | (g_b >> 3 >> 2);
                            let bb = ((b >> 3) << 3) | (b >> 3 >> 2);
                            if gr != rr || gg != gg2 || gb != bb {
                                ndiff += 1;
                            }
                        }
                        eprintln!("[gpu-dbg] frame=332 pixel diffs: {ndiff}");
                        // Find first differing pixel for frame 332
                        for i in 0..256 * 224 {
                            let cpu_r8 = ((frame[i * 4 + 2] as u32 >> 3) << 3)
                                | (frame[i * 4 + 2] as u32 >> 3 >> 2);
                            let cpu_g8 = ((frame[i * 4 + 1] as u32 >> 3) << 3)
                                | (frame[i * 4 + 1] as u32 >> 3 >> 2);
                            let cpu_b8 =
                                ((frame[i * 4] as u32 >> 3) << 3) | (frame[i * 4] as u32 >> 3 >> 2);
                            let gr = gpu_rgba[i * 4] as u32;
                            let gg = gpu_rgba[i * 4 + 1] as u32;
                            let gb = gpu_rgba[i * 4 + 2] as u32;
                            if gr != cpu_r8 || gg != cpu_g8 || gb != cpu_b8 {
                                let cx = i % 256;
                                let cy = i / 256;
                                eprintln!(
                                "[gpu-dbg] frame=332 first diff: ({cx},{cy}) cpu=({cpu_r8},{cpu_g8},{cpu_b8}) gpu=({gr},{gg},{gb})"
                            );
                                break;
                            }
                        }
                    }
                    if frames == 1000 {
                        eprint!("[gpu-dbg] GPU row0 x=0..15:");
                        for x in 0..16usize {
                            let px = &gpu_rgba[x * 4..x * 4 + 3];
                            eprint!(" ({},{},{})", px[0], px[1], px[2]);
                        }
                        eprintln!();
                    }
                    println!("{}", gpu_rgba.gpu_render_hash_log_line(frames));
                }
            }
        }
        if modern_index_compare.should_compare_frame(frames) {
            if !modern_index_compare.emit_compare_from_game_with_optional_readback(
                &mut game,
                &mut gpu_readback,
                frames,
                false,
            ) {
                process::exit(1);
            }
        }
        let mut fp_render_leaf: u32 = 0;
        if should_fingerprint_frame {
            let frame = render_hash_frame.as_mut().expect("render frame allocated");
            fp_render_leaf = replay_fingerprint_leaf_bgra(&mut game, frame);
            last_frame_had_fingerprint_render = true;
        }
        if let Some(w) = fingerprint_writer
            .as_mut()
            .filter(|_| should_fingerprint_frame)
        {
            use std::io::Write;
            let vram_bytes: Vec<u8> = game.ppu.vram.iter().flat_map(|w| w.to_le_bytes()).collect();
            let fp = parity::FrameFingerprint::compute(
                frames,
                &game.ram,
                &vram_bytes,
                &game.sram,
                fp_render_leaf,
                fp_audio_leaf,
            );
            let _ = w.write_all(&fp.to_bytes());
        }
        if let Some(coverage) = route_coverage.as_mut() {
            coverage.record(route_coverage_frame_from_game(frames, &game));
        }
        if asset_gpu_checkpoint_interval != 0 && frames % asset_gpu_checkpoint_interval == 0 {
            if let Some(dir) = asset_gpu_checkpoint_dir.as_deref() {
                write_asset_gpu_checkpoint_or_exit(&game, frames, dir);
            }
        }
        // Write --save-state-at checkpoints at the very END of the loop body, AFTER
        // the per-frame audio trace (which advances the DSP) AND the fingerprint
        // render (via play_renderer, which re-projects native state -- e.g. the
        // spotlight HDMA table -- into RAM). This is the exact same game state a
        // continuous run captures here, so a shard resuming from this checkpoint is
        // byte-identical. (This mirrors the post-loop `--save-state` write below.)
        if let Some(idx) = save_state_at.iter().position(|(f, _)| *f == frames) {
            let (_, path) = &save_state_at[idx];
            // Seed mode (no per-frame fingerprint render): the per-frame render is what
            // re-projects display state (IRQ_FLAG, spotlight HDMA, ...) into RAM, and is
            // the ONLY thing that makes RAM differ from a render-free run. Render is
            // otherwise side-effect-free on game state, so we skip it on every frame
            // (the ~100x seed speedup) and render ONLY this boundary frame here, right
            // before the checkpoint, to project display state -> byte-identical to a
            // continuously-rendered run.
            if !last_frame_had_fingerprint_render {
                let mut scratch = vec![0u8; 256 * 224 * 4];
                replay_projection_bgra(&mut game, &mut scratch);
                last_frame_had_fingerprint_render = true;
            }
            write_checkpoint(&mut game, frames, path);
        }
    }

    modern_index_compare.emit_summary_line_if_enabled();
    if let Some(renderer) = asset_gpu_smoke_renderer.as_ref() {
        let (
            cache_hits,
            cache_misses,
            cache_entries,
            cache_key_ms,
            cache_miss_ms,
            bg_extract_ms,
            sprite_extract_ms,
            stats_ms,
        ) = renderer.validation_cache_stats();
        println!(
            "replay-save asset GPU smoke passed frames={} main={:02x}; sub={:02x}; mode={}; screen={:02x}/{:02x}; cgram_nonzero={}; oam_nonzero={}; validation_cache_hits={}; validation_cache_misses={}; validation_cache_entries={}; validation_key_ms={}; validation_miss_ms={}; validation_bg_extract_ms={}; validation_sprite_extract_ms={}; validation_stats_ms={}",
            frames,
            game.ram[0x10],
            game.ram[0x11],
            game.ppu.mode,
            game.ppu.screen_enabled[0],
            game.ppu.screen_enabled[1],
            game.ppu.cgram.iter().filter(|&&v| v != 0).count(),
            game.ppu.oam.iter().filter(|&&v| v != 0).count(),
            cache_hits,
            cache_misses,
            cache_entries,
            cache_key_ms,
            cache_miss_ms,
            bg_extract_ms,
            sprite_extract_ms,
            stats_ms,
        );
    }
    if ppu_mode_summary {
        println!(
            "ppu_mode_summary m0={} m1={} m2={} m3={} m4={} m5={} m6={} m7={} first_m7={} last_m7={}",
            ppu_mode_counts[0],
            ppu_mode_counts[1],
            ppu_mode_counts[2],
            ppu_mode_counts[3],
            ppu_mode_counts[4],
            ppu_mode_counts[5],
            ppu_mode_counts[6],
            ppu_mode_counts[7],
            first_mode7_frame
                .map(|f| f.to_string())
                .unwrap_or_else(|| "none".to_string()),
            last_mode7_frame
                .map(|f| f.to_string())
                .unwrap_or_else(|| "none".to_string())
        );
    }

    if let Some(mut w) = fingerprint_writer.take() {
        use std::io::Write;
        let _ = w.flush();
    }
    if let (Some(path), Some(coverage)) = (coverage_log.as_deref(), route_coverage.as_ref()) {
        write_route_coverage_log_or_exit(path, coverage, "coverage log");
    }

    if let Some(path) = save_state_path.as_deref() {
        if !last_frame_had_fingerprint_render {
            let mut scratch = vec![0u8; 256 * 224 * 4];
            replay_projection_bgra(&mut game, &mut scratch);
        }
        if std::env::var("ZELDA3_DBG_AUDIO_FP").is_ok() {
            eprintln!(
                "[AUDIO_FP] pre-save dsp_hash=0x{:08x} (frame={frames})",
                game.zelda_audio_dsp_hash()
            );
        }
        write_checkpoint(&mut game, frames, path);
    }

    if let Some(path) = dump_frame_path.as_deref() {
        let width = 256u32;
        let height = 224u32;
        let mut dump_game = game.clone();
        let rgba = match render_live_game_gpu_frame_rgba(&mut dump_game, width, height) {
            Ok(rgba) => rgba,
            Err(e) => {
                eprintln!("failed to render replay-save dump frame via modern asset GPU path: {e}");
                process::exit(1);
            }
        };
        if let Err(e) = write_rgba_frame_png(path, &rgba, width, height) {
            eprintln!("failed to write {}: {e}", path.display());
            process::exit(1);
        }
        println!("dumped replay-save frame to {}", path.display());
    }

    let debirando_slot = (0..16).find(|&k| game.ram[0x0e20 + k] == 0x63);
    let debirando_summary = debirando_slot.map_or_else(
        || "none".to_string(),
        |k| {
            format!(
                "k{} st={:02x} ai={:02x} gfx={:02x} head={} headst={:02x} delay={:02x} floor={:02x} xy={:04x}/{:04x} vel={:02x}/{:02x} ab={:02x}/{:02x}",
                k,
                game.ram[0x0dd0 + k],
                game.ram[0x0d80 + k],
                game.ram[0x0dc0 + k],
                game.ram[0x0eb0 + k],
                game.ram[0x0dd0 + usize::from(game.ram[0x0eb0 + k])],
                game.ram[0x0df0 + k],
                game.ram[0x0f20 + k],
                u16::from_le_bytes([game.ram[0x0d10 + k], game.ram[0x0d30 + k]]),
                u16::from_le_bytes([game.ram[0x0d00 + k], game.ram[0x0d20 + k]]),
                game.ram[0x0d50 + k],
                game.ram[0x0d40 + k],
                game.ram[0x0da0 + k],
                game.ram[0x0d90 + k],
            )
        },
    );
    let room_index2 = u16::from_le_bytes([game.ram[0x48e], game.ram[0x48f]]);
    let room_mask = replay_save_room_mask(&game, room_index2);
    let room_history = [
        u16::from_le_bytes([game.ram[0xb80], game.ram[0xb81]]),
        u16::from_le_bytes([game.ram[0xb82], game.ram[0xb83]]),
        u16::from_le_bytes([game.ram[0xb84], game.ram[0xb85]]),
        u16::from_le_bytes([game.ram[0xb86], game.ram[0xb87]]),
    ];
    let history_masks = room_history.map(|room| {
        if room == 0xffff {
            0
        } else {
            replay_save_room_mask(&game, room)
        }
    });
    let msg_read_pos = read_le_u16(&game.ram, 0x1cd9) as usize;
    let msg_buffer_pos = 0x11200 + msg_read_pos;
    let msg_buffer_0 = game.ram.get(msg_buffer_pos).copied().unwrap_or(0);
    let msg_buffer_1 = game.ram.get(msg_buffer_pos + 1).copied().unwrap_or(0);
    let action_facing = usize::from((game.ram[0x2f] >> 1) & 3);
    let tilemap_mask = read_le_u16(&game.ram, 0x00ec);
    let action_y0 = read_le_u16(&game.ram, 0x20).wrapping_add(ACTION_TILE_Y[action_facing] as u16)
        & tilemap_mask;
    let action_y1 = read_le_u16(&game.ram, 0x20).wrapping_add(20) & tilemap_mask;
    let action_x0 = (read_le_u16(&game.ram, 0x22)
        .wrapping_add(ACTION_TILE_X[action_facing] as u16)
        & tilemap_mask)
        >> 3;
    let action_x1 = (read_le_u16(&game.ram, 0x22).wrapping_add(8) & tilemap_mask) >> 3;
    let action_attr0 = if game.ram[0x1b] != 0 {
        0xff
    } else {
        game.overworld_get_tile_attribute_at_location(action_x0, action_y0)
    };
    let action_attr1 = if game.ram[0x1b] != 0 {
        0xff
    } else {
        game.overworld_get_tile_attribute_at_location(action_x1, action_y1)
    };

    gpu_render_compare.emit_summary_line_if_quiet();

    // Stable byte-level WRAM dump for deterministic old-vs-new diffing (no
    // bisection): `ZELDA3_REPLAY_WRAM_DUMP=<path>` writes the full 128KB WRAM.
    if let Some(path) = std::env::var_os("ZELDA3_REPLAY_WRAM_DUMP") {
        if let Err(e) = std::fs::write(&path, &game.ram[..]) {
            eprintln!("failed to write WRAM dump to {path:?}: {e}");
        }
    }
    if let Some(path) = std::env::var_os("ZELDA3_VRAM_DUMP") {
        let bytes: Vec<u8> = game.ppu.vram.iter().flat_map(|w| w.to_le_bytes()).collect();
        if let Err(e) = std::fs::write(&path, &bytes) {
            eprintln!("failed to write VRAM dump to {path:?}: {e}");
        }
    }
    // `ZELDA3_PALETTE_MIRROR_DUMP=<path>` writes the bincode-serialized provenance mirror at
    // the final frame and logs its per-bank source-tag histogram. Diffing two dumps (from-scratch
    // vs checkpoint-resumed at the same frame) proves the mirror trailer restores it exactly.
    if let Some(path) = std::env::var_os("ZELDA3_PALETTE_MIRROR_DUMP") {
        let bytes = game.palette_mirror_snapshot_bytes();
        // Stable FNV-1a over the serialized mirror (values + tags) — a single observable
        // to compare from-scratch@N vs checkpoint-resumed@N without diffing files.
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for &b in &bytes {
            hash ^= u64::from(b);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        if let Err(e) = std::fs::write(&path, &bytes) {
            eprintln!("failed to write palette mirror dump to {path:?}: {e}");
        }
        eprintln!(
            "palette_mirror_dump frame={frames} hash=0x{hash:016x} {}",
            game.palette_mirror_tag_histogram()
        );
    }

    println!(
        "replay-save completed frames={frames} active={} ending={} fc=0x{:02x} rng=0x{:02x} ramhash=0x{:08x} ram0=0x{:08x} ram1=0x{:08x} ram2=0x{:08x} ram3=0x{:08x} ram4=0x{:08x} ram5=0x{:08x} ram6=0x{:08x} ram7=0x{:08x} sramhash=0x{:08x} roommask=0x{:04x} hist={:04x},{:04x},{:04x},{:04x} histmask={:04x},{:04x},{:04x},{:04x} main={} sub={} subsub={} adc=0x{:04x} saved={} map={} msgmod={} indoors={} room=0x{:04x} ow=0x{:04x} msg=0x{:04x} msgpos=0x{:04x} msgb=0x{:02x}/0x{:02x} textrs=0x{:02x} wait=0x{:04x}/0x{:02x} immob=0x{:02x} x=0x{:04x} y=0x{:04x} subpix=0x{:02x}/0x{:02x} hp=0x{:02x} item=0x{:02x} cur_y=0x{:02x} active_item=0x{:02x} hand=0x{:02x} bow=0x{:02x} boom=0x{:02x} big=0x{:04x} keys=0x{:02x} sram3=0x{:02x} joyh=0x{:02x} joyl=0x{:02x} fh=0x{:02x} fl=0x{:02x} dir=0x{:02x} face=0x{:02x} state=0x{:02x} inwater=0x{:02x} aux=0x{:02x} ph_timer=0x{:02x} incap=0x{:02x} lflag=0x{:02x} recoil=0x{:02x} drag=0x{:02x} grab=0x{:02x} abtn=0x{:02x} by=0x{:02x} anim={}/{} var30d=0x{:02x} follower=0x{:02x} foltimer=0x{:04x} folvar={}/{}/{}/{} folevt=0x{:02x} trans={} dirbits=0x{:02x} dirbits2=0x{:02x} tctr={} owcnt={} b69c=0x{:02x} vx=0x{:02x} vy=0x{:02x} bg3v=0x{:04x} yvel=0x{:02x} door=0x{:02x} speed=0x{:02x}/0x{:02x} lspeed=0x{:02x}/0x{:02x} dash=0x{:02x} cdf=0x{:02x} last=0x{:02x} dlast=0x{:02x} orth=0x{:02x} force=0x{:04x} prevent=0x{:02x} dragxy=0x{:04x}/0x{:04x} rtrans=0x{:02x} tcoll=0x{:02x} col=0x{:02x},0x{:02x} bugs=0x{:02x} feat=0x{:08x} wanted=0x{:08x} z=0x{:02x} vz=0x{:02x} vzcopy=0x{:02x} below=0x{:02x} tile=0x{:04x} action=0x{:02x} interact=0x{:02x},{:02x},{:02x} read=0x{:04x} a0=0x{:02x} a1=0x{:02x} chest=0x{:04x} keylock=0x{:02x} srm=0x{:02x} mark0=0x{:04x} chk0={} bak0=0x{:04x} bchk0={} r16=0x{:02x} arr1={},{},{} sel3=0x{:02x} sel4=0x{:02x} sel5=0x{:02x} sel9=0x{:02x} sel11=0x{:02x} ptimer=0x{:02x} bframes=0x{:02x} r14=0x{:04x} r12=0x{:04x} misc=0x{:04x} pit=0x{:02x} spike=0x{:02x} vledge=0x{:02x} stair=0x{:02x} deep=0x{:04x} normal=0x{:04x} debirando={}",
        game.state_recorder.replay_mode,
        u8::from(game.ram[0x10] == 26),
        game.ram[0x1a],
        game.ram[0x0fa1],
        replay_checksum_ram_range(&game.ram, 0, game.ram.len()),
        replay_checksum_ram_range(&game.ram, 0x00000, 0x4000),
        replay_checksum_ram_range(&game.ram, 0x04000, 0x4000),
        replay_checksum_ram_range(&game.ram, 0x08000, 0x4000),
        replay_checksum_ram_range(&game.ram, 0x0c000, 0x4000),
        replay_checksum_ram_range(&game.ram, 0x10000, 0x4000),
        replay_checksum_ram_range(&game.ram, 0x14000, 0x4000),
        replay_checksum_ram_range(&game.ram, 0x18000, 0x4000),
        replay_checksum_ram_range(&game.ram, 0x1c000, 0x4000),
        replay_checksum_bytes(&game.sram),
        room_mask,
        room_history[0],
        room_history[1],
        room_history[2],
        room_history[3],
        history_masks[0],
        history_masks[1],
        history_masks[2],
        history_masks[3],
        game.ram[0x10],
        game.ram[0x11],
        game.ram[0xb0],
        u16::from_le_bytes([game.ram[0xadc], game.ram[0xadd]]),
        game.ram[0x10c],
        game.ram[0x200],
        game.ram[0x1cd8],
        game.ram[0x1b],
        u16::from_le_bytes([game.ram[0xa0], game.ram[0xa1]]),
        u16::from_le_bytes([game.ram[0x8a], game.ram[0x8b]]),
        u16::from_le_bytes([game.ram[0x1cf0], game.ram[0x1cf1]]),
        u16::from_le_bytes([game.ram[0x1cd9], game.ram[0x1cda]]),
        msg_buffer_0,
        msg_buffer_1,
        game.ram[0x1cd4],
        u16::from_le_bytes([game.ram[0x1ce0], game.ram[0x1ce1]]),
        game.ram[0x1ce9],
        game.ram[0x2e4],
        u16::from_le_bytes([game.ram[0x22], game.ram[0x23]]),
        u16::from_le_bytes([game.ram[0x20], game.ram[0x21]]),
        game.ram[0x2b],
        game.ram[0x2a],
        game.ram[0xf36d],
        game.ram[0x0202],
        game.ram[0x0303],
        game.ram[0x0304],
        game.ram[0x0301],
        game.ram[0xf340],
        game.ram[0xf341],
        u16::from_le_bytes([game.ram[0xf366], game.ram[0xf367]]),
        game.ram[0xf36f],
        game.ram[0xf3c9],
        game.ram[0xf0],
        game.ram[0xf2],
        game.ram[0xf4],
        game.ram[0xf6],
        game.ram[0x67],
        game.ram[0x2f],
        game.ram[0x5d],
        game.ram[0x345],
        game.ram[0x4d],
        game.ram[0x300],
        game.ram[0x46],
        game.ram[0x34a],
        game.ram[0x2c6],
        game.ram[0x48],
        game.ram[0x376],
        game.ram[0x3b],
        game.ram[0x3a],
        game.ram[0x30a],
        game.ram[0x30b],
        game.ram[0x30d],
        game.ram[0xf3cc],
        read_le_u16(&game.ram, 0x2cd),
        game.ram[0x2d3],
        game.ram[0x2cf],
        game.ram[0x2d0],
        game.ram[0x2f9],
        game.ram[0x2f2],
        game.ram[0x418],
        game.ram[0x410],
        game.ram[0x416],
        game.ram[0x126],
        game.ram[0x69a],
        game.ram[0x69c],
        game.ram[0x28],
        game.ram[0x27],
        read_le_u16(&game.ram, 0x00ea),
        game.ram[0x30],
        game.ram[0x6c],
        game.ram[0x1cd5],
        game.ram[0x1cd6],
        game.ram[0x5e],
        game.ram[0x57],
        game.ram[0x0374],
        game.ram[0x1cdf],
        game.ram[0x66],
        game.ram[0x26],
        game.ram[0x6a],
        u16::from_le_bytes([game.ram[0x49], game.ram[0x4a]]),
        game.ram[0xb7b],
        u16::from_le_bytes([game.ram[0xb7c], game.ram[0xb7d]]),
        u16::from_le_bytes([game.ram[0xb7e], game.ram[0xb7f]]),
        game.ram[0xef],
        game.ram[0x315],
        game.ram[0x316],
        game.ram[0x317],
        game.ram[0x64a],
        u32::from_le_bytes([
            game.ram[0x64c],
            game.ram[0x64d],
            game.ram[0x64e],
            game.ram[0x64f],
        ]),
        game.wanted_zelda_features,
        game.ram[0x24],
        game.ram[0x29],
        game.ram[0x2c7],
        game.ram[0x114],
        u16::from_le_bytes([game.ram[0x2ea], game.ram[0x2eb]]),
        game.ram[0x36c],
        game.ram[0x368],
        game.ram[0x369],
        game.ram[0x36a],
        u16::from_le_bytes([game.ram[0x366], game.ram[0x367]]),
        action_attr0,
        action_attr1,
        read_le_u16(&game.ram, 0x02e5),
        game.ram[0x2e7],
        read_le_u16(&game.sram, 0x1ffe),
        read_le_u16(&game.sram, 0x03e5),
        u8::from(replay_sram_checksum_ok(&game.sram, 0)),
        read_le_u16(&game.sram, 0x12e5),
        u8::from(replay_sram_checksum_ok(&game.sram, 0x0f00)),
        game.ram[0xc8],
        u16::from_le_bytes([game.ram[0x00bf], game.ram[0x00c0]]),
        u16::from_le_bytes([game.ram[0x00c1], game.ram[0x00c2]]),
        u16::from_le_bytes([game.ram[0x00c3], game.ram[0x00c4]]),
        game.ram[0xb10],
        game.ram[0xb12],
        game.ram[0xb15],
        game.ram[0xb13],
        game.ram[0xb14],
        game.ram[0x371],
        game.ram[0x3c],
        u16::from_le_bytes([game.ram[0xe], game.ram[0xf]]),
        u16::from_le_bytes([game.ram[0xc], game.ram[0xd]]),
        u16::from_le_bytes([game.ram[0x2f6], game.ram[0x2f7]]),
        game.ram[0x59],
        game.ram[0x2e8],
        game.ram[0x36d],
        game.ram[0x58],
        u16::from_le_bytes([game.ram[0x341], game.ram[0x342]]),
        u16::from_le_bytes([game.ram[0x343], game.ram[0x344]]),
        debirando_summary,
    );
    if std::env::var_os("ZELDA3_REPLAY_SPRITE_DUMP").is_some() {
        println!("{}", replay_save_sprite_dump(&game));
    }
    if std::env::var_os("ZELDA3_REPLAY_RAM_PAGE_DUMP").is_some() {
        println!("{}", replay_save_ram_page_dump(&game));
    }
    if std::env::var_os("ZELDA3_REPLAY_RAM0400_DUMP").is_some() {
        println!("{}", replay_save_ram0400_dump(&game));
    }
    if std::env::var_os("ZELDA3_REPLAY_RAM0000_DUMP").is_some() {
        println!("{}", replay_save_ram0000_dump(&game));
    }
    if let Some(page) = replay_save_requested_ram_page_dump(&game) {
        println!("{page}");
    }
    if std::env::var_os("ZELDA3_REPLAY_ANCILLA_DUMP").is_some() {
        println!("{}", replay_save_ancilla_dump(&game));
    }
    if std::env::var_os("ZELDA3_REPLAY_GARNISH_DUMP").is_some() {
        println!("{}", replay_save_garnish_dump(&game));
    }
    if std::env::var_os("ZELDA3_REPLAY_RNG_DUMP").is_some() {
        println!(
            "rng seed=0x{:02x} frame_counter=0x{:02x}",
            game.ram[0x0fa1], game.ram[0x1a]
        );
    }
    if std::env::var_os("ZELDA3_REPLAY_ROOM_HISTORY_DUMP").is_some() {
        println!("{}", replay_save_room_history_dump(&game));
    }
    if std::env::var_os("ZELDA3_REPLAY_ROOM_MASK_DUMP").is_some() {
        println!("{}", replay_save_room_mask_dump(&game));
    }
    if std::env::var_os("ZELDA3_REPLAY_OVERLORD_DUMP").is_some() {
        println!("{}", replay_save_overlord_dump(&game));
    }
    if std::env::var_os("ZELDA3_REPLAY_DOOR_DUMP").is_some() {
        println!("{}", replay_save_door_dump(&game));
    }
    if std::env::var_os("ZELDA3_REPLAY_DUNGEON_ATTR_DUMP").is_some() {
        println!("{}", replay_save_dungeon_attr_dump(&game));
    }
    if std::env::var_os("ZELDA3_REPLAY_DUNGMAP_DUMP").is_some() {
        println!("{}", replay_save_dungmap_dump(&game));
    }
    if std::env::var_os("ZELDA3_REPLAY_MESSAGE_DUMP").is_some() {
        println!("{}", replay_save_message_dump(&game));
    }
    if std::env::var_os("ZELDA3_REPLAY_PALETTE_DUMP").is_some() {
        println!("{}", replay_save_palette_dump(&game));
    }
}

fn write_replay_save_render_hash_gpu_dump_or_exit(
    dump_path: &Path,
    render_hash_capture: &gpu_readback::ReplayRenderHashCapture,
    gpu_readback: &mut gpu_readback::OptionalGpuReadbackRenderer,
    width: u32,
) {
    let gpu_rgba = render_hash_capture.render_gpu_rgba(gpu_readback);
    if let Err(e) = write_rgba_frame_png(dump_path, &gpu_rgba, width, 224) {
        eprintln!("failed to write {}: {e}", dump_path.display());
        process::exit(1);
    }
    println!(
        "dumped replay-save asset GPU frame to {}",
        dump_path.display()
    );
}

// Trailer appended after the C-style state-recorder checkpoint to make resume
// byte-identical to a from-scratch run. The state-recorder save only round-trips
// the audio *sequencer variables* (lossily, resetting timer_cycles + the APU
// queue) and re-derives the rest, so the first resumed frame's audio diverges.
// This trailer carries the exact live audio runtime state.
const AUDIO_TRAILER_MAGIC: [u8; 8] = *b"Z3FAITH3";

fn read_trailer_blob<R: std::io::Read>(file: &mut R) -> std::io::Result<Vec<u8>> {
    let mut len_bytes = [0u8; 8];
    file.read_exact(&mut len_bytes)?;
    let len = u64::from_le_bytes(len_bytes) as usize;
    let mut buf = vec![0u8; len];
    file.read_exact(&mut buf)?;
    Ok(buf)
}

fn write_trailer_blob<W: std::io::Write>(file: &mut W, blob: &[u8]) -> std::io::Result<()> {
    file.write_all(&(blob.len() as u64).to_le_bytes())?;
    file.write_all(blob)?;
    Ok(())
}

fn read_optional_trailer_blob<R: std::io::Read>(file: &mut R) -> std::io::Result<Option<Vec<u8>>> {
    match read_trailer_blob(file) {
        Ok(blob) => Ok(Some(blob)),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(None),
        Err(e) => Err(e),
    }
}

pub(crate) fn load_replay_save_checkpoint(
    game: &mut ZeldaState,
    path: &Path,
) -> std::io::Result<()> {
    use std::io::Read;
    let mut file = fs::File::open(path)?;
    let mut state_recorder = std::mem::take(&mut game.state_recorder);
    game.state_recorder_load(&mut state_recorder, &mut file, false);
    game.state_recorder = state_recorder;
    // Optional faithfulness trailer (older checkpoints won't have it):
    //   [audio snapshot blob][SAVELOAD_HDMA scratch blob]
    let mut magic = [0u8; 8];
    if file.read_exact(&mut magic).is_ok() && magic == AUDIO_TRAILER_MAGIC {
        let audio = read_trailer_blob(&mut file)?;
        if let Err(e) = game.zelda_audio_restore_from_bytes(&audio) {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e));
        }
        // Restore the live spotlight HDMA dynamic table + re-sync the native model,
        // overriding the lossy reconstruction done inside state_recorder_load.
        let hdma_dyn = read_trailer_blob(&mut file)?;
        game.restore_hdma_dynamic_table_bytes(&hdma_dyn);
        // Restore the pristine SAVELOAD scratch (0x1b00 + 0x654) for WRAM fidelity.
        let hdma_scratch = read_trailer_blob(&mut file)?;
        game.restore_saveload_hdma_scratch_bytes(&hdma_scratch);
        if let Some(bg3_vwf_glyph_runs) = read_optional_trailer_blob(&mut file)? {
            let runs: Vec<zelda3::Bg3VwfGlyphRun> = bincode::deserialize(&bg3_vwf_glyph_runs)
                .map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("invalid BG3 VWF glyph-run checkpoint trailer: {e}"),
                    )
                })?;
            game.restore_bg3_vwf_glyph_runs(runs);
        }
        // Optional provenance-mirror trailer (newest checkpoints only): restore the
        // palette mirror exactly as-derived — true source tags, no live-CGRAM read at
        // the boundary — overwriting the shadow reconstitution done in state_recorder_load.
        // Absent (older checkpoints) leaves the reconstituted mirror in place.
        if let Some(mirror_bytes) = read_optional_trailer_blob(&mut file)? {
            game.restore_palette_mirror_from_bytes(&mirror_bytes)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        }
    }
    Ok(())
}

fn save_replay_save_checkpoint(game: &mut ZeldaState, path: &Path) -> std::io::Result<()> {
    use std::io::Write;
    // Capture snapshots BEFORE state_recorder_save mutates them: the C-style music
    // save rewrites SPC RAM, and the spotlight backup projects into the SAVELOAD
    // HDMA scratch region. We want the live frame-boundary values.
    let audio_bytes = game.zelda_audio_snapshot_bytes();
    let hdma_dyn_bytes = game.hdma_dynamic_table_bytes();
    let hdma_scratch_bytes = game.saveload_hdma_scratch_bytes();
    let bg3_vwf_glyph_runs = bincode::serialize(game.bg3_vwf_glyph_runs()).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("failed to encode BG3 VWF glyph-run checkpoint trailer: {e}"),
        )
    })?;
    // Capture the provenance mirror before state_recorder_save runs (it is native-only
    // and not touched by the save, but capture alongside the other pristine trailers).
    let palette_mirror_bytes = game.palette_mirror_snapshot_bytes();
    let mut file = fs::File::create(path)?;
    let mut state_recorder = std::mem::take(&mut game.state_recorder);
    game.state_recorder_save(&mut state_recorder, &mut file);
    game.state_recorder = state_recorder;
    file.write_all(&AUDIO_TRAILER_MAGIC)?;
    write_trailer_blob(&mut file, &audio_bytes)?;
    write_trailer_blob(&mut file, &hdma_dyn_bytes)?;
    write_trailer_blob(&mut file, &hdma_scratch_bytes)?;
    write_trailer_blob(&mut file, &bg3_vwf_glyph_runs)?;
    write_trailer_blob(&mut file, &palette_mirror_bytes)?;
    // CRITICAL: state_recorder_save (save_snes_state) MUTATES the live game --
    // zelda_save_music_state_to_ram_locked rewrites SPC RAM, and
    // backup_spotlight_hdma_to_saveload_buffer projects 0xff-padded entries into
    // the SAVELOAD scratch. With --save-state-at, the seeding pass CONTINUES from
    // this game after the write, so an unrestored mutation corrupts every later
    // frame (and thus every later checkpoint). Restore the pristine pre-save state
    // captured above so the checkpoint write is non-destructive.
    if let Err(e) = game.zelda_audio_restore_from_bytes(&audio_bytes) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e));
    }
    game.restore_hdma_dynamic_table_bytes(&hdma_dyn_bytes);
    game.restore_saveload_hdma_scratch_bytes(&hdma_scratch_bytes);
    Ok(())
}

fn write_checkpoint(game: &mut ZeldaState, frames: u32, path: &Path) {
    if let Err(e) = save_replay_save_checkpoint(game, path) {
        eprintln!(
            "failed to save replay-save checkpoint {}: {e}",
            path.display()
        );
        process::exit(1);
    }
    println!(
        "saved replay-save checkpoint frame={} to {}",
        frames,
        path.display()
    );
}

pub(crate) fn print_replay_save_panic_report(
    game: &ZeldaState,
    frames: u32,
    panic_info: &CapturedPanic,
) {
    let sr = &game.state_recorder;
    let packet = &game.ram[0x1000..0x1010];
    let packet_dst = u16::from_le_bytes([game.ram[0x1000], game.ram[0x1001]]);
    let packet_vmain = game.ram[0x1002];
    let packet_len = game.ram[0x1003];
    eprintln!(
        "replay-save panic frame={frames} \
         panic={} location={} \
         replay_mode={} replay_frame_counter={} total_frames={} replay_pos={} \
         replay_pos_last_complete={} replay_next_cmd_at={} replay_cmd=0x{:02x} \
         frames_since_last={} last_inputs=0x{:04x} \
         nmi_flags=0x{:02x} nmi_subroutine=0x{:02x} nmi_packet=dst:0x{:04x}/vmain:0x{:02x}/len:{} bytes={:02x?} \
         trace={}",
        panic_info.message,
        panic_info.location,
        sr.replay_mode,
        sr.replay_frame_counter,
        sr.total_frames,
        sr.replay_pos,
        sr.replay_pos_last_complete,
        sr.replay_next_cmd_at,
        sr.replay_cmd,
        sr.frames_since_last,
        sr.last_inputs,
        game.ram[0x18],
        game.ram[0x17],
        packet_dst,
        packet_vmain,
        packet_len,
        packet,
        TraceState::from_ram(&game.ram, sr.last_inputs, select_run_what(&game.ram)),
    );
}

#[derive(Serialize, Deserialize)]
struct PlayCrashCheckpoint {
    magic: [u8; 8],
    host_frame: u32,
    input: u16,
    run_what: u8,
    game: ZeldaState,
}

#[derive(Clone, Debug)]
pub(crate) struct CapturedPanic {
    message: String,
    location: String,
    backtrace: String,
}

pub(crate) fn install_crash_panic_hook() -> Arc<Mutex<Option<CapturedPanic>>> {
    let last_panic = Arc::new(Mutex::new(None));
    let hook_slot = last_panic.clone();
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let captured = capture_panic_info(info);
        if let Ok(mut slot) = hook_slot.lock() {
            *slot = Some(captured);
        }
        default_hook(info);
    }));
    last_panic
}

fn capture_panic_info(info: &PanicHookInfo<'_>) -> CapturedPanic {
    CapturedPanic {
        message: panic_message_from_payload(info.payload()),
        location: info
            .location()
            .map(|location| {
                format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                )
            })
            .unwrap_or_else(|| "unknown".to_string()),
        backtrace: Backtrace::force_capture().to_string(),
    }
}

pub(crate) fn captured_panic_from(
    last_panic: Arc<Mutex<Option<CapturedPanic>>>,
    payload: Box<dyn std::any::Any + Send>,
) -> CapturedPanic {
    if let Ok(mut slot) = last_panic.lock() {
        if let Some(captured) = slot.take() {
            return captured;
        }
    }
    CapturedPanic {
        message: panic_message_from_payload(payload.as_ref()),
        location: "unknown".to_string(),
        backtrace: Backtrace::force_capture().to_string(),
    }
}

fn panic_message_from_payload(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "<non-string panic payload>".to_string()
    }
}

pub(crate) fn write_play_crash_report(
    game: &ZeldaState,
    host_frame: u32,
    input: u16,
    run_what: u8,
    crash_stage: &str,
    panic_info: Option<&CapturedPanic>,
) {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let stem = format!("zelda3-rs-crash-{}-{seconds}", process::id());
    let report_path = env::temp_dir().join(format!("{stem}.txt"));
    let state_path = env::temp_dir().join(format!("{stem}.z3play"));
    let trace = TraceState::from_ram(&game.ram, input, run_what);
    let panic_message = panic_info
        .map(|info| info.message.as_str())
        .unwrap_or("unknown");
    let panic_location = panic_info
        .map(|info| info.location.as_str())
        .unwrap_or("unknown");
    let backtrace = panic_info.map(|info| info.backtrace.as_str()).unwrap_or("");
    let report = format!(
        "zelda3-rs playable crash\n\
         host_frame={host_frame}\n\
         crash_stage={crash_stage}\n\
         checkpoint_state=before_crashing_frame\n\
         panic={panic_message}\n\
         panic_location={panic_location}\n\
         frame_ctr_dbg={}\n\
         ram_fnv1a64={:016x}\n\
         sram_fnv1a64={:016x}\n\
         ppu_mode={}\n\
         ppu_screen={:02x}/{:02x}\n\
         forced_blank={}\n\
         brightness={}\n\
         trace={trace}\n\
         link_dma={}\n\
         checkpoint={}\n",
        game.frame_ctr_dbg,
        fnv1a64(&game.ram),
        fnv1a64(&game.sram),
        game.ppu.mode,
        game.ppu.screen_enabled[0],
        game.ppu.screen_enabled[1],
        game.ppu.forced_blank,
        game.ppu.brightness,
        format_link_dma_trace(&game.ram),
        state_path.display(),
    );
    let report = if backtrace.is_empty() {
        report
    } else {
        format!("{report}backtrace:\n{backtrace}\n")
    };
    if let Err(e) = fs::write(&report_path, report) {
        eprintln!(
            "failed to write crash report {}: {e}",
            report_path.display()
        );
    }
    let checkpoint = PlayCrashCheckpoint {
        magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
        host_frame,
        input,
        run_what,
        game: game.clone(),
    };
    match bincode::serialize(&checkpoint) {
        Ok(bytes) => {
            if let Err(e) = fs::write(&state_path, bytes) {
                eprintln!(
                    "failed to write crash checkpoint {}: {e}",
                    state_path.display()
                );
            }
        }
        Err(e) => eprintln!(
            "failed to encode crash checkpoint {}: {e}",
            state_path.display()
        ),
    }
    if let Some(info) = panic_info {
        eprintln!("zelda3-rs panic: {}", info.message);
        eprintln!("zelda3-rs panic location: {}", info.location);
    }
    eprintln!("zelda3-rs crash report: {}", report_path.display());
    eprintln!("zelda3-rs crash checkpoint: {}", state_path.display());
    eprintln!(
        "replay with: cargo run -p zelda3-bin -- --replay-crash <rom.sfc> {}",
        state_path.display()
    );
    eprintln!("include the report text and checkpoint path when reporting the crash");
}

pub(crate) fn load_play_state(rom_path: &str) -> ZeldaState {
    load_game_state(rom_path, true)
}

pub(crate) fn load_embedded_play_state() -> ZeldaState {
    let mut game = ZeldaState::new();
    game.set_rom_startup_timing(true);
    apply_startup_audio_phase_override(&mut game);
    if let Err(e) = game.set_assets(EMBEDDED_ASSETS) {
        eprintln!("failed to load embedded zelda3_assets.dat: {e}");
        process::exit(1);
    }
    configure_game_runtime_defaults(&mut game);
    game.zelda_read_sram();
    game
}

pub(crate) fn load_translated_replay_state(rom_path: &str) -> ZeldaState {
    load_game_state(rom_path, false)
}

pub(crate) fn load_embedded_asset_replay_state(rom_path: &str) -> Result<ZeldaState, String> {
    let rom = fs::read(rom_path).map_err(|e| format!("failed to read {rom_path}: {e}"))?;

    let mut game = ZeldaState::new();
    game.set_rom(&rom);
    game.set_assets(EMBEDDED_ASSETS)
        .map_err(|e| format!("failed to load embedded zelda3_assets.dat: {e}"))?;
    configure_game_runtime_defaults(&mut game);
    game.zelda_read_sram();
    Ok(game)
}

fn load_game_state(rom_path: &str, rom_startup_timing: bool) -> ZeldaState {
    let rom = match fs::read(rom_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("failed to read {rom_path}: {e}");
            process::exit(1);
        }
    };
    let asset_path = match find_asset_pack(rom_path) {
        Some(path) => path,
        None => {
            eprintln!(
                "failed to find zelda3_assets.dat next to the ROM or in the current directory"
            );
            process::exit(1);
        }
    };
    let assets = match fs::read(&asset_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("failed to read {}: {e}", asset_path.display());
            process::exit(1);
        }
    };

    let mut game = ZeldaState::new();
    if rom_startup_timing {
        game.set_rom_startup_timing(true);
        apply_startup_audio_phase_override(&mut game);
    }
    game.set_rom(&rom);
    if let Err(e) = game.set_assets(&assets) {
        eprintln!("failed to load {}: {e}", asset_path.display());
        process::exit(1);
    }
    configure_game_runtime_defaults(&mut game);
    game.zelda_read_sram();
    game
}

fn configure_game_runtime_defaults(game: &mut ZeldaState) {
    let cfg = parse_config_file_context(None).config;
    game.ppu.extra_left_right =
        (cfg.extended_aspect_ratio as usize).min(PPU_EXTRA_LEFT_RIGHT) as u8;
    let mut audio_freq = cfg.audio_freq as u32;
    if !(11025..=48000).contains(&audio_freq) {
        audio_freq = 44100;
    }
    game.zelda_configure_audio(
        audio_freq,
        cfg.msuvolume,
        cfg.resume_msu,
        cfg.msu_path.clone(),
    );
    game.zelda_enable_msu(cfg.enable_msu);
    game.zelda_set_language(cfg.language.as_deref());
}

pub(crate) fn load_play_or_checkpoint(
    rom_path: &str,
    load_state: Option<&Path>,
) -> (ZeldaState, u32) {
    if let Some(path) = load_state {
        if let Ok(checkpoint) = load_play_crash_checkpoint(path) {
            let mut game = checkpoint.game;
            game.set_rom_startup_timing(true);
            apply_startup_audio_phase_override(&mut game);
            return (game, checkpoint.host_frame);
        }
        match load_lockstep_checkpoint(path) {
            Ok(checkpoint) => return (checkpoint.oracle.game, checkpoint.frame),
            Err(lockstep_err) => {
                // Fall back to the replay-save C-style state_recorder checkpoint format
                // (written by --replay-save --save-state and the replay-bisect cache).
                // These are a different on-disk format than the bincode play-crash /
                // lockstep checkpoints above, so accept them here too for parity probes.
                let mut game = load_play_state(rom_path);
                match load_replay_save_checkpoint(&mut game, path) {
                    Ok(()) => {
                        game.set_rom_startup_timing(true);
                        let frame = game.state_recorder.replay_frame_counter;
                        return (game, frame);
                    }
                    Err(state_recorder_err) => {
                        eprintln!(
                            "failed to load checkpoint {} (not a play-crash, lockstep, or replay-save checkpoint): lockstep={lockstep_err}; replay-save={state_recorder_err}",
                            path.display()
                        );
                        process::exit(1);
                    }
                }
            }
        }
    }
    (load_play_state(rom_path), 0)
}

fn apply_startup_audio_phase_override(game: &mut ZeldaState) {
    let sfx_timer = env::var("ZELDA3_STARTUP_SFX_TIMER")
        .ok()
        .and_then(|value| value.parse::<u8>().ok());
    let timer_cycles = env::var("ZELDA3_STARTUP_TIMER_CYCLES")
        .ok()
        .and_then(|value| value.parse::<u8>().ok());
    if sfx_timer.is_some() || timer_cycles.is_some() {
        game.zelda_set_spc_startup_phase(sfx_timer.unwrap_or(72), timer_cycles.unwrap_or(0));
    }
}

fn run_replay_crash(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!("usage: zelda3 --replay-crash <path-to-rom.sfc> <crash.z3play> [frames]");
            process::exit(2);
        }
    };
    let crash_path = match args.get(1) {
        Some(p) => Path::new(p),
        None => {
            eprintln!("usage: zelda3 --replay-crash <path-to-rom.sfc> <crash.z3play> [frames]");
            process::exit(2);
        }
    };
    let frames: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let checkpoint = match load_play_crash_checkpoint(crash_path) {
        Ok(checkpoint) => checkpoint,
        Err(e) => {
            eprintln!(
                "failed to load crash checkpoint {}: {e}",
                crash_path.display()
            );
            process::exit(1);
        }
    };
    let _ = load_play_state(rom_path);
    let last_panic = install_crash_panic_hook();
    let mut game = checkpoint.game;
    let gpu_readback = match ModernAssetGpuReadbackRenderer::load_from_env() {
        Ok(readback) => readback,
        Err(e) => {
            eprintln!("failed to initialize replay-crash asset GPU renderer: {e}");
            process::exit(1);
        }
    };
    eprintln!(
        "replaying crash checkpoint {} from host_frame {}; trace={}",
        crash_path.display(),
        checkpoint.host_frame,
        TraceState::from_ram(&game.ram, checkpoint.input, checkpoint.run_what)
    );
    for local_frame in 0..frames {
        let input = if local_frame == 0 {
            checkpoint.input
        } else {
            0
        };
        let run_what = if local_frame == 0 {
            checkpoint.run_what
        } else {
            select_run_what(&game.ram)
        };
        let host_frame = checkpoint.host_frame.wrapping_add(local_frame);
        let pre_frame_game = game.clone();
        let result = panic::catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
            game.run_frame_internal(input, run_what);
            game.zelda_push_apu_state();
            gpu_readback.render_game_rgba(&mut game).map(|_| ())
        }));
        match result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                eprintln!("failed to replay crash frame via asset GPU renderer: {e}");
                process::exit(1);
            }
            Err(payload) => {
                let panic_info = captured_panic_from(last_panic.clone(), payload);
                write_play_crash_report(
                    &pre_frame_game,
                    host_frame,
                    input,
                    run_what,
                    "replay_asset_gpu_frame",
                    Some(&panic_info),
                );
                process::exit(101);
            }
        }
    }
    println!(
        "replay completed {frames} frame(s) from host_frame {}; ram_fnv1a64={:016x}",
        checkpoint.host_frame,
        fnv1a64(&game.ram)
    );
}

fn run_smoke_render(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!("usage: zelda3 --smoke-render <path-to-rom.sfc> [frames]");
            process::exit(2);
        }
    };
    let frames: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(120);
    let mut game = load_play_state(rom_path);
    let mut frame = None;
    let mut audio = vec![0i16; 735 * 2];
    let mut audio_nonzero = 0usize;
    let mut audio_peak = 0i16;
    for _ in 0..frames {
        game.zelda_run_frame(0);
        frame = Some(match render_live_game_gpu_frame_rgba(&mut game, 256, 224) {
            Ok(rgba) => rgba,
            Err(e) => {
                eprintln!("failed to render smoke frame via modern asset GPU path: {e}");
                process::exit(1);
            }
        });
        game.zelda_render_audio(&mut audio, 735, 2);
        game.zelda_discard_unused_audio_frames();
        audio_nonzero += audio.iter().filter(|&&sample| sample != 0).count();
        audio_peak = audio_peak.max(
            audio
                .iter()
                .map(|sample| sample.saturating_abs())
                .max()
                .unwrap_or(0),
        );
    }
    let nonzero_pixels = frame
        .as_ref()
        .map(|frame| frame.as_slice())
        .unwrap_or(&[])
        .chunks_exact(4)
        .filter(|pixel| pixel.iter().any(|&b| b != 0))
        .count();
    let cgram_indices = game
        .ppu
        .cgram
        .iter()
        .enumerate()
        .filter(|(_, &value)| value != 0)
        .take(24)
        .map(|(i, value)| format!("{i}:{value:04x}"))
        .collect::<Vec<_>>()
        .join(",");
    let obj_indices = game
        .ppu
        .obj_buffer
        .data
        .iter()
        .enumerate()
        .filter(|(_, &value)| value & 0xff != 0)
        .take(16)
        .map(|(i, value)| format!("{i}:{value:04x}"))
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "smoke-render completed {frames} frame(s); nonzero_pixels={nonzero_pixels}; audio_nonzero={audio_nonzero}; audio_peak={audio_peak}; apui00={:02x}; music={:02x}/{:02x}/{:02x}; {}; frame={}; main={:02x}; sub={:02x}; subsub={:02x}; anim={:04x}; bugs={:02x}; nmi_thread={:02x}; stack={:04x}; forced_blank={}; brightness={}; mode={}; screen={:02x}/{:02x}; vram_nonzero={}; cgram_nonzero={}; cgram_indices=[{}]; oam_nonzero={}; obj_pixels=[{}]; first_cgram={:04x},{:04x},{:04x},{:04x}; obj_cgram={:04x},{:04x},{:04x},{:04x}; oam0={:04x},{:04x},{:04x},{:04x}",
        game.ram[0x0648],
        game.ram[0x012c],
        game.ram[0x0130],
        game.ram[0x0133],
        game.zelda_audio_debug_summary(),
        frames,
        game.ram[0x10],
        game.ram[0x11],
        game.ram[0xb0],
        u16::from_le_bytes([game.ram[0x0adc], game.ram[0x0add]]),
        game.ram[0x064a],
        game.ram[0x012a],
        u16::from_le_bytes([game.ram[0x1f0a], game.ram[0x1f0b]]),
        game.ppu.forced_blank,
        game.ppu.brightness,
        game.ppu.mode,
        game.ppu.screen_enabled[0],
        game.ppu.screen_enabled[1],
        game.ppu.vram.iter().filter(|&&v| v != 0).count(),
        game.ppu.cgram.iter().filter(|&&v| v != 0).count(),
        cgram_indices,
        game.ppu.oam.iter().filter(|&&v| v != 0).count(),
        obj_indices,
        game.ppu.cgram[0],
        game.ppu.cgram[1],
        game.ppu.cgram[2],
        game.ppu.cgram[3],
        game.ppu.cgram[128],
        game.ppu.cgram[129],
        game.ppu.cgram[130],
        game.ppu.cgram[131],
        game.ppu.oam[0],
        game.ppu.oam[1],
        game.ppu.oam[2],
        game.ppu.oam[3],
    );
}

fn load_modern_asset_gpu_readback_or_exit(context: &str) -> ModernAssetGpuReadbackRenderer {
    match ModernAssetGpuReadbackRenderer::load_from_env() {
        Ok(renderer) => renderer,
        Err(e) => {
            eprintln!("failed to initialize modern asset GPU readback for {context}: {e}");
            process::exit(1);
        }
    }
}

fn render_modern_asset_gpu_frame_rgba_or_exit(
    renderer: &ModernAssetGpuReadbackRenderer,
    game: &mut ZeldaState,
    context: &str,
) -> Vec<u8> {
    match renderer.render_game_rgba(game) {
        Ok(frame) => frame.as_slice().to_vec(),
        Err(e) => {
            eprintln!("failed to render {context} via modern asset GPU path: {e}");
            process::exit(1);
        }
    }
}

fn run_trace_startup_audio(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --trace-startup-audio <path-to-rom.sfc> [frames] [--jsonl] [--c-oracle]"
            );
            process::exit(2);
        }
    };
    let mut frames = 360u32;
    let mut jsonl = false;
    let mut c_oracle = false;
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--jsonl" => jsonl = true,
            "--c-oracle" => c_oracle = true,
            value if !value.starts_with("--") => {
                frames = match value.parse() {
                    Ok(frames) => frames,
                    Err(e) => {
                        eprintln!("invalid frame count `{value}`: {e}");
                        process::exit(2);
                    }
                };
            }
            flag => {
                eprintln!("unknown --trace-startup-audio option: {flag}");
                process::exit(2);
            }
        }
    }
    let mut game = load_game_state(rom_path, !c_oracle);
    let mut audio = vec![0i16; 735 * 2];
    let mut last_ports = [0u8; 4];
    let mut last_nonzero = false;
    for frame_index in 0..frames {
        game.zelda_run_frame(0);
        let ports = game.zelda_debug_apu_write_ports();
        game.zelda_render_audio(&mut audio, 735, 2);
        game.zelda_discard_unused_audio_frames();
        let peak = audio
            .iter()
            .map(|sample| sample.saturating_abs())
            .max()
            .unwrap_or(0);
        let first_nonzero = audio.iter().position(|&sample| sample != 0);
        let mean_abs = if audio.is_empty() {
            0.0
        } else {
            audio
                .iter()
                .map(|sample| i64::from(sample.saturating_abs()))
                .sum::<i64>() as f64
                / audio.len() as f64
        };
        let hash = replay_checksum_samples(&audio);
        let nonzero = first_nonzero.is_some();
        if jsonl {
            let first_nonzero = first_nonzero
                .map(|value| value.to_string())
                .unwrap_or_else(|| "null".to_string());
            println!(
                "{{\"frame\":{frame_index},\"samples\":735,\"channels\":2,\"peak\":{peak},\"first_nonzero\":{first_nonzero},\"mean_abs\":{mean_abs:.6},\"hash\":\"0x{hash:08x}\",\"ports\":[{},{},{},{}],\"apui\":[{},{},{},{}],\"music\":[{},{},{}],\"main\":{},\"sub\":{},\"subsub\":{},\"inidisp\":{}}}",
                ports[0],
                ports[1],
                ports[2],
                ports[3],
                game.ram[0x0648],
                game.ram[0x012c],
                game.ram[0x012d],
                game.ram[0x012e],
                game.ram[0x012f],
                game.ram[0x0132],
                game.ram[0x0133],
                game.ram[0x10],
                game.ram[0x11],
                game.ram[0xb0],
                game.ram[0x13],
            );
        } else if ports != last_ports || nonzero != last_nonzero || peak >= 12_000 {
            println!(
                "{frame_index:>5}: peak={peak:>5} first_nonzero={first_nonzero:?} ports={ports:02x?} apui={:02x}/{:02x}/{:02x}/{:02x} music={:02x}/{:02x}/{:02x} main={:02x} sub={:02x} subsub={:02x} inidisp={:02x} {}",
                game.ram[0x0648],
                game.ram[0x012c],
                game.ram[0x012d],
                game.ram[0x012e],
                game.ram[0x012f],
                game.ram[0x0132],
                game.ram[0x0133],
                game.ram[0x10],
                game.ram[0x11],
                game.ram[0xb0],
                game.ram[0x13],
                game.zelda_audio_debug_summary(),
            );
        }
        last_ports = ports;
        last_nonzero = nonzero;
    }
}

fn run_trace_bsnes_audio(args: &[String]) {
    let core_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --trace-bsnes-audio <path-to-bsnes-libretro.dylib> <path-to-rom.sfc> [frames]"
            );
            process::exit(2);
        }
    };
    let rom_path = match args.get(1) {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --trace-bsnes-audio <path-to-bsnes-libretro.dylib> <path-to-rom.sfc> [frames]"
            );
            process::exit(2);
        }
    };
    let frames: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(360);
    let mut game = load_play_state(rom_path);
    let mut audio = vec![0i16; 735 * 2];
    let mut bsnes = match LibretroCore::load(core_path, rom_path) {
        Ok(core) => core,
        Err(e) => {
            eprintln!("failed to initialize libretro core: {e}");
            process::exit(1);
        }
    };
    println!(
        "bsnes geometry={}x{} fps={:.9} sample_rate={:.3}",
        bsnes.geometry.base_width,
        bsnes.geometry.base_height,
        bsnes.av_info.timing.fps,
        bsnes.av_info.timing.sample_rate,
    );
    for frame_index in 0..frames {
        game.zelda_run_frame(0);
        let ports = game.zelda_debug_apu_write_ports();
        game.zelda_render_audio(&mut audio, 735, 2);
        game.zelda_discard_unused_audio_frames();
        let rust_peak = audio
            .iter()
            .map(|sample| sample.saturating_abs())
            .max()
            .unwrap_or(0);
        let rust_first = audio.iter().position(|&sample| sample != 0);

        let capture = bsnes.run_frame();
        let ref_peak = capture
            .audio
            .iter()
            .map(|sample| sample.saturating_abs())
            .max()
            .unwrap_or(0);
        let ref_first = capture.audio.iter().position(|&sample| sample != 0);
        if rust_peak >= 12_000
            || ref_peak >= 12_000
            || rust_first.is_some() != ref_first.is_some()
            || ports != [0, 0, 0, 0]
        {
            println!(
                "{frame_index:>5}: rust_peak={rust_peak:>5} rust_first={rust_first:?} ref_peak={ref_peak:>5} ref_first={ref_first:?} ref_samples={} ports={ports:02x?} {}",
                capture.audio.len() / 2,
                game.zelda_audio_debug_summary(),
            );
        }
    }
}

fn run_compare_bsnes_startup_audio(args: &[String]) {
    let core_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --compare-bsnes-startup-audio <path-to-bsnes-libretro.dylib> <path-to-rom.sfc> [frames]"
            );
            process::exit(2);
        }
    };
    let rom_path = match args.get(1) {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --compare-bsnes-startup-audio <path-to-bsnes-libretro.dylib> <path-to-rom.sfc> [frames]"
            );
            process::exit(2);
        }
    };
    let frames: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(180);
    let mut game = load_play_state(rom_path);
    let mut audio = vec![0i16; 735 * 2];
    let mut bsnes = match LibretroCore::load(core_path, rom_path) {
        Ok(core) => core,
        Err(e) => {
            eprintln!("failed to initialize libretro core: {e}");
            process::exit(1);
        }
    };

    let mut rust_stats = Vec::with_capacity(frames as usize);
    let mut ref_stats = Vec::with_capacity(frames as usize);
    let mut rust_debug = Vec::with_capacity(frames as usize);
    let mut ref_video_frames = 0usize;
    let mut ref_video_meta = None;
    for _ in 0..frames {
        game.zelda_run_frame(0);
        let ports = game.zelda_debug_apu_write_ports();
        game.zelda_render_audio(&mut audio, 735, 2);
        game.zelda_discard_unused_audio_frames();
        rust_stats.push(AudioFrameStats::from_interleaved_stereo(&audio));
        rust_debug.push(format!(
            "ports={ports:02x?} main={:02x} sub={:02x} subsub={:02x} {}",
            game.ram[0x10],
            game.ram[0x11],
            game.ram[0xb0],
            game.zelda_audio_debug_summary(),
        ));

        let capture = bsnes.run_frame();
        if !capture.video.is_empty() {
            ref_video_frames += 1;
            ref_video_meta.get_or_insert((
                capture.video_width,
                capture.video_height,
                capture.video_pitch,
                capture.pixel_format,
            ));
        }
        ref_stats.push(AudioFrameStats::from_interleaved_stereo(&capture.audio));
    }

    let threshold = 512i16;
    let rust_onset = first_peak_frame(&rust_stats, threshold);
    let ref_onset = first_peak_frame(&ref_stats, threshold);
    let rust_max = max_peak_frame(&rust_stats);
    let ref_max = max_peak_frame(&ref_stats);
    println!(
        "bsnes geometry={}x{} fps={:.9} sample_rate={:.3} video_frames={ref_video_frames}/{frames} first_video={ref_video_meta:?}",
        bsnes.geometry.base_width,
        bsnes.geometry.base_height,
        bsnes.av_info.timing.fps,
        bsnes.av_info.timing.sample_rate,
    );
    println!(
        "startup audio threshold={threshold}: rust_onset={rust_onset:?} ref_onset={ref_onset:?} rust_max={rust_max:?} ref_max={ref_max:?}",
    );
    if let (Some(rust_onset), Some(ref_onset)) = (rust_onset, ref_onset) {
        let delta = ref_onset as i32 - rust_onset as i32;
        println!("startup audio onset_delta_ref_minus_rust={delta} frames");
    }
    print_audio_window(
        "rust",
        &rust_stats,
        &rust_debug,
        rust_onset.or(rust_max.map(|(i, _)| i)),
    );
    print_audio_window(
        "bsnes",
        &ref_stats,
        &[],
        ref_onset.or(ref_max.map(|(i, _)| i)),
    );
}

fn run_compare_bsnes_oracle(args: &[String]) {
    run_compare_libretro_oracle(args, Some("bsnes"));
}

fn run_compare_libretro_oracle(args: &[String], default_oracle_name: Option<&str>) {
    let core_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --compare-libretro-oracle <path-to-snes-libretro.dylib> <path-to-rom.sfc> [frames] [--oracle-name <name>] [--input-script <path>] [--load-sram <path>] [--ignore-video] [--ignore-audio] [--compare-from-frame <n>] [--skip-oracle-frames <n>] [--auto-align-video] [--lead-rust-audio-blocks <n>] [--trace-dsp-writes]"
            );
            process::exit(2);
        }
    };
    let rom_path = match args.get(1) {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --compare-libretro-oracle <path-to-snes-libretro.dylib> <path-to-rom.sfc> [frames] [--oracle-name <name>] [--input-script <path>] [--load-sram <path>] [--ignore-video] [--ignore-audio] [--compare-from-frame <n>] [--skip-oracle-frames <n>] [--auto-align-video] [--lead-rust-audio-blocks <n>] [--trace-dsp-writes]"
            );
            process::exit(2);
        }
    };
    let mut frames = 300u32;
    let mut input_script = InputScript::default();
    let mut load_sram = None::<PathBuf>;
    let mut compare_video = true;
    let mut compare_audio = true;
    let mut compare_from_frame = 0u32;
    let mut skip_oracle_frames = 0u32;
    let mut auto_align_video = false;
    let mut lead_rust_audio_blocks = 0u32;
    let mut trace_video_pixel: Option<(usize, usize)> = None;
    let mut trace_dsp_writes = false;
    let mut color_tolerance = 0u8;
    let mut max_mismatched_pixels = 0usize;
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
            "--load-sram" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--load-sram requires a path");
                    process::exit(2);
                };
                load_sram = Some(PathBuf::from(path));
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
            "--skip-bsnes-frames" | "--skip-oracle-frames" => {
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
            "--trace-dsp-writes" => {
                trace_dsp_writes = true;
                i += 1;
            }
            flag => {
                eprintln!("unknown --compare-libretro-oracle option: {flag}");
                process::exit(2);
            }
        }
    }
    if trace_dsp_writes {
        require_audio_oracle("--trace-dsp-writes");
    }
    if auto_align_video && compare_audio {
        eprintln!("--auto-align-video is video-only; pass --ignore-audio for this mode");
        process::exit(2);
    }

    let mut game = load_play_state(rom_path);
    if let Some(path) = load_sram.as_deref() {
        let sram = read_file_or_exit(path, "SRAM");
        apply_sram_to_game_or_exit(&mut game, path, &sram);
    }
    let width = 256u32;
    let height = 224u32;
    let mut rust_audio = Vec::new();
    let mut discard_audio = Vec::new();
    let mut dsp_writes = Vec::new();
    let mut last_sample_frames = 800usize;
    let load_sram_bytes = load_sram
        .as_deref()
        .map(|path| read_file_or_exit(path, "SRAM"));
    let mut oracle =
        match LibretroCore::load_with_sram(core_path, rom_path, load_sram_bytes.as_deref()) {
            Ok(core) => core,
            Err(e) => {
                eprintln!("failed to initialize libretro core: {e}");
                process::exit(1);
            }
        };
    println!(
        "{oracle_name} oracle geometry={}x{} fps={:.9} sample_rate={:.3}",
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
    let trace_poly_sched = std::env::var_os("TRACE_POLY_SCHED").is_some();
    let gpu_video_readback = (compare_video || trace_video_pixel.is_some())
        .then(|| load_modern_asset_gpu_readback_or_exit("libretro oracle video comparison"));
    for frame_index in 0..frames {
        let input = input_script.input_for_frame(frame_index);
        let compare_this_frame = frame_index >= compare_from_frame;
        let pre_game = game.clone();
        game.zelda_run_frame(input as i32);
        let rust_video_frame =
            (trace_video_pixel.is_some() || (compare_this_frame && compare_video)).then(|| {
                render_modern_asset_gpu_frame_rgba_or_exit(
                    gpu_video_readback
                        .as_ref()
                        .expect("GPU readback allocated for libretro video comparison"),
                    &mut game,
                    "libretro oracle video comparison",
                )
            });
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
                game.debug_bsnes_poly_scheduler_counter(),
                game.debug_bsnes_hold_intro_step_this_frame(),
                game.debug_bsnes_intro_step_carry_phase_active(),
                game.debug_bsnes_intro_step_hold_alternate(),
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

        let mut capture = oracle.run_frame_with_input(input);
        let sample_frames = capture.audio.len() / 2;
        if sample_frames != 0 {
            last_sample_frames = sample_frames;
            rust_audio.resize(capture.audio.len(), 0);
            dsp_writes.clear();
            for _ in 0..lead_rust_audio_blocks {
                game.zelda_render_audio(&mut rust_audio, sample_frames as i32, 2);
            }
            if trace_dsp_writes {
                dsp_writes = game.zelda_render_audio_trace_dsp_events(
                    &mut rust_audio,
                    sample_frames as i32,
                    2,
                );
            } else {
                game.zelda_render_audio(&mut rust_audio, sample_frames as i32, 2);
            }
        } else {
            rust_audio.clear();
            dsp_writes.clear();
            discard_audio.resize(last_sample_frames.saturating_mul(2), 0);
            if trace_dsp_writes {
                dsp_writes = game.zelda_render_audio_trace_dsp_events(
                    &mut discard_audio,
                    last_sample_frames as i32,
                    2,
                );
            } else {
                game.zelda_render_audio(&mut discard_audio, last_sample_frames as i32, 2);
            }
        }
        game.zelda_discard_unused_audio_frames();
        let rust_stats = AudioFrameStats::from_interleaved_stereo(&rust_audio);
        let oracle_stats = AudioFrameStats::from_interleaved_stereo(&capture.audio);
        if let Some((x, y)) = trace_video_pixel {
            let pixel_index = y.saturating_mul(width as usize).saturating_add(x);
            let rust_offset = pixel_index.saturating_mul(4);
            let bsnes_offset = y.saturating_mul(capture.video_pitch)
                + x * bsnes_pixel_stride(capture.pixel_format).unwrap_or(0);
            let rust_pixel = rust_video_frame
                .as_deref()
                .and_then(|frame| rgba_pixel_at(frame, rust_offset))
                .unwrap_or([0; 4]);
            let oracle_pixel = bsnes_rgba_pixel_at(&capture, bsnes_offset).unwrap_or([0; 4]);
            let obj_pal = (0x90..=0x9f)
                .map(|i| format!("{:04x}", game.ppu.cgram[i]))
                .collect::<Vec<_>>()
                .join(",");
            println!(
                "pixel frame={frame_index} xy=({x},{y}) rust={rust_pixel:02x?} {oracle_name}={oracle_pixel:02x?} main={:02x} sub={:02x} subsub={:02x} inidisp={:02x} obj_pal90=[{}]",
                game.ram[0x10], game.ram[0x11], game.ram[0xb0], game.ram[0x13], obj_pal,
            );
        }
        if trace_dsp_writes && !dsp_writes.is_empty() {
            println!(
                "dsp frame={frame_index} writes=[{}] stats={:?}",
                format_dsp_writes(&dsp_writes),
                rust_stats,
            );
        }
        if compare_this_frame && compare_video {
            let rust_video_frame = rust_video_frame
                .as_deref()
                .expect("GPU video frame rendered for libretro video comparison");
            let mut video_diff = compare_libretro_video_frame(
                rust_video_frame,
                width,
                height,
                &capture,
                color_tolerance,
                max_mismatched_pixels,
            );
            if auto_align_video && video_diff.is_some() {
                let (aligned_capture, extra, matched) = align_bsnes_video_capture(
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
                let artifact_dir = write_bsnes_parity_failure_artifacts(
                    &pre_game,
                    &game,
                    rust_video_frame,
                    &rust_audio,
                    &capture,
                    frame_index,
                    input,
                    oracle.av_info.timing.sample_rate.round() as u32,
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
                process::exit(1);
            }
        }
        if compare_this_frame && compare_audio {
            if let Some(audio_diff) = compare_bsnes_audio_frame(&rust_audio, &capture.audio) {
                let audio_artifact_frame;
                let rust_artifact_frame = match rust_video_frame.as_deref() {
                    Some(frame) => frame,
                    None => {
                        let renderer = load_modern_asset_gpu_readback_or_exit(
                            "libretro oracle audio artifact frame",
                        );
                        audio_artifact_frame = render_modern_asset_gpu_frame_rgba_or_exit(
                            &renderer,
                            &mut game,
                            "libretro oracle audio artifact frame",
                        );
                        &audio_artifact_frame
                    }
                };
                let artifact_dir = write_bsnes_parity_failure_artifacts(
                    &pre_game,
                    &game,
                    rust_artifact_frame,
                    &rust_audio,
                    &capture,
                    frame_index,
                    input,
                    oracle.av_info.timing.sample_rate.round() as u32,
                    format!("{oracle_name} audio divergence: {audio_diff}"),
                )
                .ok();
                eprintln!(
                    "{oracle_name} audio divergence at frame {frame_index}: {audio_diff}; input={input:04x}; ports={ports:02x?}; main={:02x} sub={:02x} subsub={:02x}",
                    game.ram[0x10], game.ram[0x11], game.ram[0xb0],
                );
                eprintln!("rust audio:  {:?}", rust_stats);
                eprintln!("{oracle_name} audio: {:?}", oracle_stats);
                eprintln!("rust audio debug: {}", game.zelda_audio_debug_summary());
                if let Some(dir) = artifact_dir {
                    eprintln!("parity failure artifacts: {}", dir.display());
                }
                process::exit(1);
            }
        }
    }

    println!(
        "{oracle_name} oracle compare completed {frames} frame(s) with no enabled video/audio diff"
    );
}

fn oracle_name_from_core_path(core_path: &str) -> String {
    let stem = Path::new(core_path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("libretro");
    stem.strip_suffix("_libretro").unwrap_or(stem).to_string()
}

fn run_dump_bsnes_frame(args: &[String]) {
    let core_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --dump-bsnes-frame <path-to-bsnes-libretro.dylib> <path-to-rom.sfc> <frames> <out.png> [--input-script <path>] [--load-sram <path>] [--skip-bsnes-frames <n>]"
            );
            process::exit(2);
        }
    };
    let rom_path = match args.get(1) {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --dump-bsnes-frame <path-to-bsnes-libretro.dylib> <path-to-rom.sfc> <frames> <out.png> [--input-script <path>] [--load-sram <path>] [--skip-bsnes-frames <n>]"
            );
            process::exit(2);
        }
    };
    let frames: u32 = match args.get(2).and_then(|s| s.parse().ok()) {
        Some(frames) => frames,
        None => {
            eprintln!(
                "usage: zelda3 --dump-bsnes-frame <path-to-bsnes-libretro.dylib> <path-to-rom.sfc> <frames> <out.png> [--input-script <path>] [--load-sram <path>] [--skip-bsnes-frames <n>]"
            );
            process::exit(2);
        }
    };
    let out_path = match args.get(3) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "usage: zelda3 --dump-bsnes-frame <path-to-bsnes-libretro.dylib> <path-to-rom.sfc> <frames> <out.png> [--input-script <path>] [--load-sram <path>] [--skip-bsnes-frames <n>]"
            );
            process::exit(2);
        }
    };
    let mut input_script = InputScript::default();
    let mut load_sram = None::<PathBuf>;
    let mut skip_bsnes_frames = 0u32;
    let mut i = 4usize;
    while i < args.len() {
        match args[i].as_str() {
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
            "--load-sram" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--load-sram requires a path");
                    process::exit(2);
                };
                load_sram = Some(PathBuf::from(path));
                i += 2;
            }
            "--skip-bsnes-frames" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("--skip-bsnes-frames requires a count");
                    process::exit(2);
                };
                skip_bsnes_frames = match value.parse() {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("invalid --skip-bsnes-frames `{value}`: {e}");
                        process::exit(2);
                    }
                };
                i += 2;
            }
            flag => {
                eprintln!("unknown --dump-bsnes-frame option: {flag}");
                process::exit(2);
            }
        }
    }

    let load_sram_bytes = load_sram
        .as_deref()
        .map(|path| read_file_or_exit(path, "SRAM"));
    let mut bsnes =
        match LibretroCore::load_with_sram(core_path, rom_path, load_sram_bytes.as_deref()) {
            Ok(core) => core,
            Err(e) => {
                eprintln!("failed to initialize libretro core: {e}");
                process::exit(1);
            }
        };
    for _ in 0..skip_bsnes_frames {
        let _ = bsnes.run_frame_with_input(0);
    }
    let mut capture = None;
    for frame_index in 0..frames {
        let input = input_script.input_for_frame(frame_index);
        capture = Some(bsnes.run_frame_with_input(input));
    }
    let Some(capture) = capture else {
        eprintln!("frame count must be greater than zero");
        process::exit(2);
    };
    let Some(stride) = bsnes_pixel_stride(capture.pixel_format) else {
        eprintln!("unsupported bsnes pixel format {}", capture.pixel_format);
        process::exit(1);
    };
    let mut frame = vec![0u8; capture.video_width as usize * capture.video_height as usize * 4];
    for y in 0..capture.video_height as usize {
        for x in 0..capture.video_width as usize {
            let src = y * capture.video_pitch + x * stride;
            let Some([r, g, b, _]) = bsnes_rgba_pixel_at(&capture, src) else {
                eprintln!("failed to decode bsnes pixel at {x},{y}");
                process::exit(1);
            };
            let dst = (y * capture.video_width as usize + x) * 4;
            frame[dst] = b;
            frame[dst + 1] = g;
            frame[dst + 2] = r;
        }
    }
    if let Err(e) =
        write_argb_frame_png(&out_path, &frame, capture.video_width, capture.video_height)
    {
        eprintln!("failed to write {}: {e}", out_path.display());
        process::exit(1);
    }
    println!(
        "dumped bsnes frame {frames} to {}; skip={skip_bsnes_frames}; geometry={}x{}; pixel_format={}; pitch={}",
        out_path.display(),
        capture.video_width,
        capture.video_height,
        capture.pixel_format,
        capture.video_pitch,
    );
}

fn run_trace_bsnes_memory(args: &[String]) {
    let core_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --trace-bsnes-memory <path-to-bsnes-libretro.dylib> <path-to-rom.sfc> [frames]"
            );
            process::exit(2);
        }
    };
    let rom_path = match args.get(1) {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --trace-bsnes-memory <path-to-bsnes-libretro.dylib> <path-to-rom.sfc> [frames]"
            );
            process::exit(2);
        }
    };
    let frames: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(120);
    let mut bsnes = match LibretroCore::load(core_path, rom_path) {
        Ok(core) => core,
        Err(e) => {
            eprintln!("failed to load bsnes core: {e}");
            process::exit(1);
        }
    };
    println!(
        "bsnes memory probe geometry={}x{} fps={:.9} sample_rate={:.3}",
        bsnes.geometry.base_width,
        bsnes.geometry.base_height,
        bsnes.av_info.timing.fps,
        bsnes.av_info.timing.sample_rate,
    );
    for id in 0..=7 {
        let size = bsnes.memory_size(id);
        let present = bsnes.memory_bytes(id).is_some();
        println!(
            "memory id={id} name={} size={} present={present}",
            libretro_memory_name(id),
            size
        );
    }

    let mut last_apui00 = None;
    let mut last_system_digest = None;
    let mut last_save_digest = None;
    for frame in 0..frames {
        let capture = bsnes.run_frame();
        let system_ram = bsnes.memory_bytes(RETRO_MEMORY_SYSTEM_RAM);
        let save_ram = bsnes.memory_bytes(RETRO_MEMORY_SAVE_RAM);
        let apui00 = system_ram.and_then(|ram| ram.get(0x0648).copied());
        let system_digest = system_ram.map(fnv1a64);
        let save_digest = save_ram.map(fnv1a64);
        let changed = apui00 != last_apui00
            || system_digest != last_system_digest
            || save_digest != last_save_digest
            || frame < 4;
        if changed {
            println!(
                "frame={frame:>4} audio_samples={} video={}x{} apui00={} system_ram={} system_digest={} save_ram={} save_digest={}",
                capture.audio.len() / 2,
                capture.video_width,
                capture.video_height,
                format_optional_u8(apui00),
                system_ram.map(|ram| ram.len()).unwrap_or(0),
                format_optional_u64(system_digest),
                save_ram.map(|ram| ram.len()).unwrap_or(0),
                format_optional_u64(save_digest),
            );
        }
        last_apui00 = apui00;
        last_system_digest = system_digest;
        last_save_digest = save_digest;
    }
}

fn compare_bsnes_audio_frame(rust_audio: &[i16], bsnes_audio: &[i16]) -> Option<String> {
    if rust_audio.len() != bsnes_audio.len() {
        return Some(format!(
            "sample_count rust={} bsnes={}",
            rust_audio.len() / 2,
            bsnes_audio.len() / 2,
        ));
    }
    for (i, (&mine, &theirs)) in rust_audio.iter().zip(bsnes_audio.iter()).enumerate() {
        if mine != theirs {
            let mismatched = rust_audio
                .iter()
                .zip(bsnes_audio.iter())
                .filter(|(mine, theirs)| mine != theirs)
                .count();
            return Some(format!(
                "mismatched_samples={mismatched}; first_mismatch={i} rust={mine} bsnes={theirs}"
            ));
        }
    }
    None
}

fn format_dsp_writes(writes: &[DspWriteEvent]) -> String {
    writes
        .iter()
        .map(|write| {
            format!(
                "{:02x}:{:02x}@{}/{}",
                write.addr, write.value, write.sample_offset, write.timer_cycles
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn compare_bsnes_video_frame(
    rust_frame: &[u8],
    rust_width: u32,
    rust_height: u32,
    bsnes: &LibretroFrame,
) -> Option<String> {
    compare_libretro_video_frame(rust_frame, rust_width, rust_height, bsnes, 0, 0)
}

fn compare_libretro_video_frame(
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
    for y in 0..rust_height as usize {
        for x in 0..rust_width as usize {
            let pixel_index = y * rust_width as usize + x;
            let rust_offset = pixel_index * 4;
            let bsnes_offset =
                y * libretro.video_pitch + x * bsnes_pixel_stride(libretro.pixel_format)?;
            let mine = rgba_pixel_at(rust_frame, rust_offset)?;
            let theirs = bsnes_rgba_pixel_at(libretro, bsnes_offset)?;
            if !rgb_within_tolerance(mine, theirs, color_tolerance) {
                mismatched += 1;
                first.get_or_insert((x, y, mine, theirs));
            }
        }
    }
    if mismatched <= max_mismatched_pixels {
        return None;
    }
    first.map(|(x, y, mine, theirs)| {
        format!(
            "mismatched_pixels={mismatched}; allowed_mismatched_pixels={max_mismatched_pixels}; color_tolerance={color_tolerance}; first_mismatch=({x}, {y}) rust={mine:02x?} libretro={theirs:02x?}; pixel_format={} pitch={}",
            libretro.pixel_format, libretro.video_pitch
        )
    })
}

fn rgb_within_tolerance(mine: [u8; 4], theirs: [u8; 4], tolerance: u8) -> bool {
    mine[..3]
        .iter()
        .zip(theirs[..3].iter())
        .all(|(&mine, &theirs)| mine.abs_diff(theirs) <= tolerance)
}

fn align_bsnes_video_capture(
    bsnes: &mut LibretroCore,
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
        capture = bsnes.run_frame_with_input(input);
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
    println!("auto-align video found no RGB match within {max_extra_frames} extra bsnes frame(s)");
    (capture, max_extra_frames, false)
}

fn rgba_pixel_at(frame: &[u8], offset: usize) -> Option<[u8; 4]> {
    let bytes = frame.get(offset..offset + 4)?;
    Some([bytes[2], bytes[1], bytes[0], bytes[3]])
}

fn bsnes_pixel_stride(pixel_format: u32) -> Option<usize> {
    match pixel_format {
        0 | 2 => Some(2),
        1 => Some(4),
        _ => None,
    }
}

fn bsnes_rgba_pixel_at(frame: &LibretroFrame, offset: usize) -> Option<[u8; 4]> {
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
                expand_6_to_8((raw >> 5) & 0x3f),
                expand_5_to_8(raw & 0x1f),
                0xff,
            ])
        }
        _ => None,
    }
}

fn expand_5_to_8(value: u16) -> u8 {
    ((value << 3) | (value >> 2)) as u8
}

fn expand_6_to_8(value: u16) -> u8 {
    ((value << 2) | (value >> 4)) as u8
}

fn run_compare_startup_apu_impls(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!("usage: zelda3 --compare-startup-apu-impls <path-to-rom.sfc> [frames]");
            process::exit(2);
        }
    };
    let frames: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(120);
    let mut game = load_play_state(rom_path);
    let mut full_apu = None;
    let mut high_audio = vec![0i16; 735 * 2];
    let mut full_audio = vec![0i16; 735 * 2];
    let mut high_stats = Vec::with_capacity(frames as usize);
    let mut full_stats = Vec::with_capacity(frames as usize);
    let mut debug = Vec::with_capacity(frames as usize);

    for _ in 0..frames {
        game.zelda_run_frame(0);
        let full_apu = full_apu.get_or_insert_with(|| game.zelda_debug_full_apu_from_spc());
        let ports = game.zelda_debug_apu_write_ports();
        for (port, &value) in ports.iter().enumerate() {
            full_apu.write_snes_port(port as u8, value);
        }
        game.zelda_render_audio(&mut high_audio, 735, 2);
        game.zelda_discard_unused_audio_frames();
        render_full_apu_audio(full_apu, &mut full_audio, 735, 2);
        high_stats.push(AudioFrameStats::from_interleaved_stereo(&high_audio));
        full_stats.push(AudioFrameStats::from_interleaved_stereo(&full_audio));
        debug.push(format!(
            "ports={ports:02x?} main={:02x} sub={:02x} subsub={:02x} full_pc={:04x} full_in={:02x?} full_out={:02x?} full_dsp_writes={} full_last_dsp={:02x?} {}",
            game.ram[0x10],
            game.ram[0x11],
            game.ram[0xb0],
            full_apu.spc.pc,
            &full_apu.in_ports[..4],
            full_apu.out_ports,
            full_apu.dsp_write_history.len(),
            full_apu.dsp_write_history.last().copied(),
            game.zelda_audio_debug_summary(),
        ));
    }

    let threshold = 512i16;
    let high_onset = first_peak_frame(&high_stats, threshold);
    let full_onset = first_peak_frame(&full_stats, threshold);
    let high_max = max_peak_frame(&high_stats);
    let full_max = max_peak_frame(&full_stats);
    println!(
        "startup APU impls threshold={threshold}: high_onset={high_onset:?} full_onset={full_onset:?} high_max={high_max:?} full_max={full_max:?}",
    );
    if let (Some(high_onset), Some(full_onset)) = (high_onset, full_onset) {
        let delta = full_onset as i32 - high_onset as i32;
        println!("startup APU onset_delta_full_minus_high={delta} frames");
    }
    print_audio_window(
        "high",
        &high_stats,
        &debug,
        high_onset.or(high_max.map(|(i, _)| i)),
    );
    print_audio_window(
        "full-apu",
        &full_stats,
        &[],
        full_onset.or(full_max.map(|(i, _)| i)),
    );
}

fn render_full_apu_audio(
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

fn run_trace_song_bank(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!("usage: zelda3 --trace-song-bank <path-to-rom.sfc> [asset-index]");
            process::exit(2);
        }
    };
    let asset = args
        .get(1)
        .and_then(|arg| arg.parse::<usize>().ok())
        .unwrap_or(0);
    let game = load_play_state(rom_path);
    println!("{}", game.zelda_debug_song_bank_summary(asset));
}

#[derive(Serialize)]
struct ParityFailureReport {
    kind: String,
    frame: u32,
    input: String,
    run_what: Option<u8>,
    message: String,
    trace_mine: Option<String>,
    trace_theirs: Option<String>,
    ppu_mine: Option<String>,
    ppu_theirs: Option<String>,
    audio_mine: Option<String>,
    audio_theirs: Option<String>,
    artifacts: Vec<String>,
    notes: Vec<String>,
}

fn create_parity_failure_dir() -> Result<PathBuf, Box<dyn Error>> {
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let dir = PathBuf::from("target")
        .join("parity-failures")
        .join(format!("{seconds}-{}", process::id()));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn write_parity_diff(
    dir: &Path,
    report: &ParityFailureReport,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut artifacts = report.artifacts.clone();
    let diff = serde_json::to_string_pretty(report)?;
    fs::write(dir.join("diff.json"), diff)?;
    artifacts.push("diff.json".to_string());
    Ok(artifacts)
}

fn write_input_script_artifact(
    dir: &Path,
    input_history: &[(u32, u16)],
) -> Result<(), Box<dyn Error>> {
    let mut text = String::new();
    text.push_str("# frame input-word\n");
    for &(frame, input) in input_history {
        text.push_str(&format!("{frame} 0x{input:04x}\n"));
    }
    fs::write(dir.join("input.txt"), text)?;
    Ok(())
}

fn write_wav_i16_stereo(
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

fn render_lockstep_audio_pair(
    oracle: &LockstepOracle,
    samples_per_frame: usize,
    channels: usize,
) -> (Vec<i16>, Vec<i16>) {
    let mut game = oracle.game.clone();
    let mut snes_apu = oracle.snes.apu.clone();
    let count = samples_per_frame.saturating_mul(channels);
    let mut mine = vec![0i16; count];
    let mut theirs = vec![0i16; count];
    game.zelda_push_apu_state();
    game.zelda_render_audio(&mut mine, samples_per_frame as i32, channels as i32);
    game.zelda_discard_unused_audio_frames();
    snes_apu
        .dsp
        .get_samples(&mut theirs, samples_per_frame, channels);
    (mine, theirs)
}

fn write_lockstep_parity_failure_artifacts(
    pre_oracle: &LockstepOracle,
    post_oracle: &LockstepOracle,
    frame: u32,
    input: u16,
    run_what: u8,
    input_history: &[(u32, u16)],
    message: String,
) -> Result<PathBuf, Box<dyn Error>> {
    let dir = create_parity_failure_dir()?;
    write_input_script_artifact(&dir, input_history)?;

    let rust_checkpoint = PlayCrashCheckpoint {
        magic: *PLAY_CRASH_CHECKPOINT_MAGIC,
        host_frame: frame,
        input,
        run_what,
        game: pre_oracle.game.clone(),
    };
    fs::write(
        dir.join("rust_before.z3state"),
        bincode::serialize(&rust_checkpoint)?,
    )?;
    save_lockstep_checkpoint(
        &dir.join("oracle_before.z3state"),
        frame,
        pre_oracle.clone(),
    )?;

    let width = 256u32;
    let height = 224u32;
    let mut rust_state = post_oracle.game.clone();
    let mut oracle_state = post_oracle.game.clone();
    oracle_state.ppu = post_oracle.snes.ppu.clone();
    oracle_state.dma = post_oracle.snes.dma.clone();
    oracle_state.ram.copy_from_slice(&post_oracle.snes.ram);
    oracle_state
        .sram
        .copy_from_slice(&post_oracle.snes.cart.ram);
    let gpu_readback = ModernAssetGpuReadbackRenderer::load_from_env()
        .map_err(|e| format!("failed to initialize lockstep artifact GPU readback: {e}"))?;
    let rust_frame_rgba = gpu_readback
        .render_game_rgba(&mut rust_state)
        .map_err(|e| {
            format!("failed to render lockstep artifact via modern asset GPU path: {e}")
        })?;
    let snes_state_rust_frame_rgba =
        gpu_readback
            .render_game_rgba(&mut oracle_state)
            .map_err(|e| {
                format!("failed to render lockstep oracle-state artifact via asset GPU path: {e}")
            })?;
    write_rgba_frame_png(
        &dir.join("rust_frame.png"),
        rust_frame_rgba.as_slice(),
        width,
        height,
    )?;
    write_rgba_frame_png(
        &dir.join("snes_state_rust_render_frame.png"),
        snes_state_rust_frame_rgba.as_slice(),
        width,
        height,
    )?;

    let (rust_audio, oracle_audio) = render_lockstep_audio_pair(post_oracle, 534, 2);
    write_wav_i16_stereo(&dir.join("rust_audio.wav"), &rust_audio, 32_000, 2)?;
    write_wav_i16_stereo(&dir.join("oracle_audio.wav"), &oracle_audio, 32_000, 2)?;

    let report = ParityFailureReport {
        kind: "lockstep".to_string(),
        frame,
        input: format!("0x{input:04x}"),
        run_what: Some(run_what),
        message,
        trace_mine: Some(TraceState::from_ram(&post_oracle.game.ram, input, run_what).to_string()),
        trace_theirs: Some(TraceState::from_ram(&post_oracle.snes.ram, input, run_what).to_string()),
        ppu_mine: Some(format_render_ppu_summary(&rust_state)),
        ppu_theirs: Some(format_render_ppu_summary(&oracle_state)),
        audio_mine: Some(summarize_audio_samples(&rust_audio)),
        audio_theirs: Some(format!(
            "{}; lockstep C oracle does not boot/advance a sample-producing SPC/DSP",
            summarize_audio_samples(&oracle_audio)
        )),
        artifacts: vec![
            "input.txt".to_string(),
            "rust_before.z3state".to_string(),
            "oracle_before.z3state".to_string(),
            "rust_frame.png".to_string(),
            "snes_state_rust_render_frame.png".to_string(),
            "rust_audio.wav".to_string(),
            "oracle_audio.wav".to_string(),
            "diff.json".to_string(),
            "replay.sh".to_string(),
        ],
        notes: vec![
            "oracle_before.z3state is a --load-state compatible lockstep checkpoint before the failing frame".to_string(),
            "rust_before.z3state is a --replay-crash compatible Rust checkpoint before the failing frame".to_string(),
            "snes_state_rust_render_frame.png is the PNG-backed GPU renderer drawing the C/SNES oracle state; it is not a true C-rendered or bsnes-rendered frame".to_string(),
            "true video reference artifacts require --compare-bsnes-oracle or --dump-bsnes-frame".to_string(),
            "lockstep validates game RAM, PPU, DMA, SRAM, and renderer-visible state; exact video/audio comparison requires the bsnes oracle path".to_string(),
        ],
    };
    let mut artifacts = write_parity_diff(&dir, &report)?;

    let replay = format!(
        "#!/bin/sh\nset -eu\ncargo run -p zelda3-bin -- --compare-lockstep-render \"$1\" 1 --load-state \"{}\" --input-script \"{}\"\n",
        dir.join("oracle_before.z3state").display(),
        dir.join("input.txt").display(),
    );
    fs::write(dir.join("replay.sh"), replay)?;
    artifacts.push("replay.sh".to_string());
    let _ = artifacts;
    Ok(dir)
}

fn write_bsnes_parity_failure_artifacts(
    pre_game: &ZeldaState,
    post_game: &ZeldaState,
    rust_frame_rgba: &[u8],
    rust_audio: &[i16],
    capture: &LibretroFrame,
    frame: u32,
    input: u16,
    sample_rate: u32,
    message: String,
) -> Result<PathBuf, Box<dyn Error>> {
    let dir = create_parity_failure_dir()?;
    write_input_script_artifact(&dir, &[(frame, input)])?;
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
    fs::write(
        dir.join("oracle_before.z3state"),
        b"bsnes libretro serialization is not wired in this zelda3-rs wrapper yet\n",
    )?;

    write_rgba_frame_png(&dir.join("rust_frame.png"), rust_frame_rgba, 256, 224)?;
    let Some(stride) = bsnes_pixel_stride(capture.pixel_format) else {
        return Err(format!("unsupported bsnes pixel format {}", capture.pixel_format).into());
    };
    let mut bsnes_argb =
        vec![0u8; capture.video_width as usize * capture.video_height as usize * 4];
    for y in 0..capture.video_height as usize {
        for x in 0..capture.video_width as usize {
            let src = y * capture.video_pitch + x * stride;
            let Some([r, g, b, _]) = bsnes_rgba_pixel_at(capture, src) else {
                return Err(format!("failed to decode bsnes pixel at {x},{y}").into());
            };
            let dst = (y * capture.video_width as usize + x) * 4;
            bsnes_argb[dst] = b;
            bsnes_argb[dst + 1] = g;
            bsnes_argb[dst + 2] = r;
            bsnes_argb[dst + 3] = 0xff;
        }
    }
    write_argb_frame_png(
        &dir.join("bsnes_frame.png"),
        &bsnes_argb,
        capture.video_width,
        capture.video_height,
    )?;
    fs::copy(dir.join("bsnes_frame.png"), dir.join("oracle_frame.png"))?;
    write_wav_i16_stereo(&dir.join("rust_audio.wav"), rust_audio, sample_rate, 2)?;
    write_wav_i16_stereo(
        &dir.join("oracle_audio.wav"),
        &capture.audio,
        sample_rate,
        2,
    )?;

    let report = ParityFailureReport {
        kind: "bsnes".to_string(),
        frame,
        input: format!("0x{input:04x}"),
        run_what: None,
        message,
        trace_mine: Some(TraceState::from_ram(&post_game.ram, input, RUN_MAIN).to_string()),
        trace_theirs: None,
        ppu_mine: Some(format_render_ppu_summary(post_game)),
        ppu_theirs: None,
        audio_mine: Some(summarize_audio_samples(rust_audio)),
        audio_theirs: Some(summarize_audio_samples(&capture.audio)),
        artifacts: vec![
            "input.txt".to_string(),
            "rust_before.z3state".to_string(),
            "oracle_before.z3state".to_string(),
            "rust_frame.png".to_string(),
            "oracle_frame.png".to_string(),
            "bsnes_frame.png".to_string(),
            "rust_audio.wav".to_string(),
            "oracle_audio.wav".to_string(),
            "diff.json".to_string(),
            "replay.sh".to_string(),
        ],
        notes: vec![
            "oracle_before.z3state is a placeholder; bsnes libretro serialization still needs to be wired".to_string(),
            "oracle_frame.png and bsnes_frame.png are the same reference frame".to_string(),
        ],
    };
    let _ = write_parity_diff(&dir, &report)?;
    let replay = "#!/bin/sh\nset -eu\ncargo run -p zelda3-bin -- --replay-crash \"$1\" \"$(dirname \"$0\")/rust_before.z3state\" 1\n";
    fs::write(dir.join("replay.sh"), replay)?;
    Ok(dir)
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RetroGameGeometry {
    base_width: c_uint,
    base_height: c_uint,
    max_width: c_uint,
    max_height: c_uint,
    aspect_ratio: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RetroSystemTiming {
    fps: f64,
    sample_rate: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RetroSystemAvInfo {
    geometry: RetroGameGeometry,
    timing: RetroSystemTiming,
}

#[repr(C)]
struct RetroGameInfo {
    path: *const c_char,
    data: *const c_void,
    size: usize,
    meta: *const c_char,
}

#[repr(C)]
struct RetroVariable {
    key: *const c_char,
    value: *const c_char,
}

#[derive(Default)]
struct LibretroCapture {
    audio: Vec<i16>,
    video: Vec<u8>,
    video_width: u32,
    video_height: u32,
    video_pitch: usize,
    pixel_format: u32,
}

static LIBRETRO_CAPTURE: OnceLock<Mutex<LibretroCapture>> = OnceLock::new();
static LIBRETRO_INPUT_STATE: OnceLock<Mutex<u16>> = OnceLock::new();
static LIBRETRO_SYSTEM_DIR: OnceLock<CString> = OnceLock::new();
static LIBRETRO_SAVE_DIR: OnceLock<CString> = OnceLock::new();

struct LibretroFrame {
    audio: Vec<i16>,
    video: Vec<u8>,
    video_width: u32,
    video_height: u32,
    video_pitch: usize,
    pixel_format: u32,
}

struct LibretroCore {
    handle: *mut c_void,
    retro_deinit: unsafe extern "C" fn(),
    retro_run: unsafe extern "C" fn(),
    retro_unload_game: unsafe extern "C" fn(),
    retro_get_memory_data: unsafe extern "C" fn(c_uint) -> *mut c_void,
    retro_get_memory_size: unsafe extern "C" fn(c_uint) -> usize,
    av_info: RetroSystemAvInfo,
    geometry: RetroGameGeometry,
    _rom: Vec<u8>,
    _rom_path: CString,
}

impl LibretroCore {
    fn load(core_path: &str, rom_path: &str) -> Result<Self, String> {
        Self::load_with_sram(core_path, rom_path, None)
    }

    fn load_with_sram(
        core_path: &str,
        rom_path: &str,
        initial_sram: Option<&[u8]>,
    ) -> Result<Self, String> {
        let capture = LIBRETRO_CAPTURE.get_or_init(|| Mutex::new(LibretroCapture::default()));
        *capture.lock().map_err(|_| "capture lock poisoned")? = LibretroCapture::default();
        let input_state = LIBRETRO_INPUT_STATE.get_or_init(|| Mutex::new(0));
        *input_state.lock().map_err(|_| "input lock poisoned")? = 0;
        let save_dir = initialize_libretro_dirs()?;
        if let Some(sram) = initial_sram {
            let stem = Path::new(rom_path)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("zelda3");
            let sram_path = save_dir.join(format!("{stem}.srm"));
            fs::write(&sram_path, sram)
                .map_err(|e| format!("failed to seed bsnes SRAM {}: {e}", sram_path.display()))?;
        }

        let core_path_c = CString::new(core_path).map_err(|e| e.to_string())?;
        let handle = unsafe { libc::dlopen(core_path_c.as_ptr(), libc::RTLD_NOW) };
        if handle.is_null() {
            return Err(dlerror_string());
        }

        unsafe {
            let retro_set_environment: unsafe extern "C" fn(
                extern "C" fn(c_uint, *mut c_void) -> bool,
            ) = load_symbol(handle, "retro_set_environment")?;
            let retro_set_video_refresh: unsafe extern "C" fn(
                extern "C" fn(*const c_void, c_uint, c_uint, usize),
            ) = load_symbol(handle, "retro_set_video_refresh")?;
            let retro_set_audio_sample: unsafe extern "C" fn(extern "C" fn(i16, i16)) =
                load_symbol(handle, "retro_set_audio_sample")?;
            let retro_set_audio_sample_batch: unsafe extern "C" fn(
                extern "C" fn(*const i16, usize) -> usize,
            ) = load_symbol(handle, "retro_set_audio_sample_batch")?;
            let retro_set_input_poll: unsafe extern "C" fn(extern "C" fn()) =
                load_symbol(handle, "retro_set_input_poll")?;
            let retro_set_input_state: unsafe extern "C" fn(
                extern "C" fn(c_uint, c_uint, c_uint, c_uint) -> i16,
            ) = load_symbol(handle, "retro_set_input_state")?;
            let retro_init: unsafe extern "C" fn() = load_symbol(handle, "retro_init")?;
            let retro_deinit: unsafe extern "C" fn() = load_symbol(handle, "retro_deinit")?;
            let retro_load_game: unsafe extern "C" fn(*const RetroGameInfo) -> bool =
                load_symbol(handle, "retro_load_game")?;
            let retro_unload_game: unsafe extern "C" fn() =
                load_symbol(handle, "retro_unload_game")?;
            let retro_run: unsafe extern "C" fn() = load_symbol(handle, "retro_run")?;
            let retro_get_system_av_info: unsafe extern "C" fn(*mut RetroSystemAvInfo) =
                load_symbol(handle, "retro_get_system_av_info")?;
            let retro_get_memory_data: unsafe extern "C" fn(c_uint) -> *mut c_void =
                load_symbol(handle, "retro_get_memory_data")?;
            let retro_get_memory_size: unsafe extern "C" fn(c_uint) -> usize =
                load_symbol(handle, "retro_get_memory_size")?;

            retro_set_environment(libretro_environment);
            retro_set_video_refresh(libretro_video_refresh);
            retro_set_audio_sample(libretro_audio_sample);
            retro_set_audio_sample_batch(libretro_audio_sample_batch);
            retro_set_input_poll(libretro_input_poll);
            retro_set_input_state(libretro_input_state);
            retro_init();

            let rom = fs::read(rom_path).map_err(|e| e.to_string())?;
            let rom_path_c = CString::new(rom_path).map_err(|e| e.to_string())?;
            let game_info = RetroGameInfo {
                path: rom_path_c.as_ptr(),
                data: rom.as_ptr().cast(),
                size: rom.len(),
                meta: std::ptr::null(),
            };
            if !retro_load_game(&game_info) {
                retro_deinit();
                libc::dlclose(handle);
                return Err("retro_load_game returned false".to_string());
            }

            let mut av_info = RetroSystemAvInfo::default();
            retro_get_system_av_info(&mut av_info);
            Ok(Self {
                handle,
                retro_deinit,
                retro_run,
                retro_unload_game,
                retro_get_memory_data,
                retro_get_memory_size,
                av_info,
                geometry: av_info.geometry,
                _rom: rom,
                _rom_path: rom_path_c,
            })
        }
    }

    fn run_frame(&mut self) -> LibretroFrame {
        self.run_frame_with_input(0)
    }

    fn run_frame_with_input(&mut self, input: u16) -> LibretroFrame {
        if let Some(input_state) = LIBRETRO_INPUT_STATE.get() {
            if let Ok(mut input_state) = input_state.lock() {
                *input_state = input;
            }
        }
        if let Some(capture) = LIBRETRO_CAPTURE.get() {
            if let Ok(mut capture) = capture.lock() {
                capture.audio.clear();
                capture.video.clear();
            }
        }
        unsafe { (self.retro_run)() };
        LIBRETRO_CAPTURE
            .get()
            .and_then(|capture| {
                capture.lock().ok().map(|capture| LibretroFrame {
                    audio: capture.audio.clone(),
                    video: capture.video.clone(),
                    video_width: capture.video_width,
                    video_height: capture.video_height,
                    video_pitch: capture.video_pitch,
                    pixel_format: capture.pixel_format,
                })
            })
            .unwrap_or_else(|| LibretroFrame {
                audio: Vec::new(),
                video: Vec::new(),
                video_width: 0,
                video_height: 0,
                video_pitch: 0,
                pixel_format: 0,
            })
    }

    fn memory_size(&self, id: c_uint) -> usize {
        unsafe { (self.retro_get_memory_size)(id) }
    }

    fn memory_bytes(&self, id: c_uint) -> Option<&[u8]> {
        let size = self.memory_size(id);
        if size == 0 {
            return None;
        }
        let ptr = unsafe { (self.retro_get_memory_data)(id) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), size) })
        }
    }
}

impl Drop for LibretroCore {
    fn drop(&mut self) {
        unsafe {
            (self.retro_unload_game)();
            (self.retro_deinit)();
            libc::dlclose(self.handle);
        }
    }
}

unsafe fn load_symbol<T: Copy>(handle: *mut c_void, name: &str) -> Result<T, String> {
    let name_c = CString::new(name).unwrap();
    let ptr = unsafe { libc::dlsym(handle, name_c.as_ptr()) };
    if ptr.is_null() {
        Err(dlerror_string())
    } else {
        Ok(unsafe { std::mem::transmute_copy(&ptr) })
    }
}

fn dlerror_string() -> String {
    let err = unsafe { libc::dlerror() };
    if err.is_null() {
        "unknown dynamic loader error".to_string()
    } else {
        unsafe { CStr::from_ptr(err) }
            .to_string_lossy()
            .into_owned()
    }
}

fn initialize_libretro_dirs() -> Result<PathBuf, String> {
    if let Some(save_dir) = LIBRETRO_SAVE_DIR.get() {
        return Ok(PathBuf::from(save_dir.to_string_lossy().into_owned()));
    }

    let save_dir = env::current_dir()
        .map_err(|e| e.to_string())?
        .join("target")
        .join("bsnes-oracle-save")
        .join(process::id().to_string());
    fs::create_dir_all(&save_dir).map_err(|e| e.to_string())?;
    let save_dir_c =
        CString::new(save_dir.to_string_lossy().as_bytes()).map_err(|e| e.to_string())?;
    let _ = LIBRETRO_SYSTEM_DIR.set(save_dir_c.clone());
    let _ = LIBRETRO_SAVE_DIR.set(save_dir_c);
    Ok(save_dir)
}

pub(crate) fn read_file_or_exit(path: &Path, label: &str) -> Vec<u8> {
    match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("failed to read {label} {}: {e}", path.display());
            process::exit(1);
        }
    }
}

pub(crate) fn apply_sram_to_game_or_exit(game: &mut ZeldaState, path: &Path, sram: &[u8]) {
    let expected = game.sram.len();
    if sram.len() < expected {
        eprintln!(
            "failed to load SRAM {}: expected at least {} bytes, got {}",
            path.display(),
            expected,
            sram.len()
        );
        process::exit(1);
    }
    game.sram.copy_from_slice(&sram[..expected]);
}

extern "C" fn libretro_environment(cmd: c_uint, data: *mut c_void) -> bool {
    const RETRO_ENVIRONMENT_SET_PIXEL_FORMAT: c_uint = 10;
    const RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY: c_uint = 9;
    const RETRO_ENVIRONMENT_GET_VARIABLE: c_uint = 15;
    const RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY: c_uint = 31;
    match cmd & !0x10000 {
        RETRO_ENVIRONMENT_SET_PIXEL_FORMAT => {
            if !data.is_null() {
                let pixel_format = unsafe { *(data as *const c_uint) };
                if let Some(capture) = LIBRETRO_CAPTURE.get() {
                    if let Ok(mut capture) = capture.lock() {
                        capture.pixel_format = pixel_format;
                    }
                }
            }
            true
        }
        RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY => {
            if !data.is_null() {
                let dir = LIBRETRO_SYSTEM_DIR.get_or_init(|| CString::new("/private/tmp").unwrap());
                unsafe { *(data as *mut *const c_char) = dir.as_ptr() };
            }
            true
        }
        RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY => {
            if !data.is_null() {
                let dir = LIBRETRO_SAVE_DIR.get_or_init(|| CString::new("/private/tmp").unwrap());
                unsafe { *(data as *mut *const c_char) = dir.as_ptr() };
            }
            true
        }
        RETRO_ENVIRONMENT_GET_VARIABLE => libretro_get_variable(data),
        _ => false,
    }
}

extern "C" fn libretro_get_variable(data: *mut c_void) -> bool {
    if data.is_null() {
        return false;
    }
    let variable = unsafe { &mut *(data as *mut RetroVariable) };
    if variable.key.is_null() {
        return false;
    }
    let key = unsafe { CStr::from_ptr(variable.key) }.to_bytes();
    variable.value = match key {
        b"bsnes_blur_emulation" => c"disabled".as_ptr(),
        b"bsnes_video_filter" => c"None".as_ptr(),
        b"bsnes_video_luminance" => c"100%".as_ptr(),
        b"bsnes_video_saturation" => c"100%".as_ptr(),
        b"bsnes_video_gamma" => c"100%".as_ptr(),
        b"bsnes_ppu_fast" => c"disabled".as_ptr(),
        b"bsnes_dsp_fast" => c"disabled".as_ptr(),
        _ => return false,
    };
    true
}

extern "C" fn libretro_video_refresh(
    data: *const c_void,
    width: c_uint,
    height: c_uint,
    pitch: usize,
) {
    if let Some(capture) = LIBRETRO_CAPTURE.get() {
        if let Ok(mut capture) = capture.lock() {
            capture.video_width = width;
            capture.video_height = height;
            capture.video_pitch = pitch;
            capture.video.clear();
            if !data.is_null() {
                let byte_len = pitch.saturating_mul(height as usize);
                let bytes = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), byte_len) };
                capture.video.extend_from_slice(bytes);
            }
        }
    }
}

extern "C" fn libretro_audio_sample(left: i16, right: i16) {
    if let Some(capture) = LIBRETRO_CAPTURE.get() {
        if let Ok(mut capture) = capture.lock() {
            capture.audio.push(left);
            capture.audio.push(right);
        }
    }
}

extern "C" fn libretro_audio_sample_batch(data: *const i16, frames: usize) -> usize {
    if !data.is_null() {
        let samples = unsafe { std::slice::from_raw_parts(data, frames.saturating_mul(2)) };
        if let Some(capture) = LIBRETRO_CAPTURE.get() {
            if let Ok(mut capture) = capture.lock() {
                capture.audio.extend_from_slice(samples);
            }
        }
    }
    frames
}

extern "C" fn libretro_input_poll() {}

extern "C" fn libretro_input_state(
    _port: c_uint,
    device: c_uint,
    _index: c_uint,
    id: c_uint,
) -> i16 {
    const RETRO_DEVICE_JOYPAD: c_uint = 1;
    if device != RETRO_DEVICE_JOYPAD || id >= 16 {
        return 0;
    }
    LIBRETRO_INPUT_STATE
        .get()
        .and_then(|input| input.lock().ok().map(|input| ((*input >> id) & 1) as i16))
        .unwrap_or(0)
}

fn run_lockstep(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --lockstep <path-to-rom.sfc> [frames] [--input-script <path>] [--load-sram <path>] [--trace-state] [--trace-semantic-state] [--save-state <path>] [--load-state <path>]"
            );
            process::exit(2);
        }
    };
    let config = match parse_lockstep_args(args) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{e}");
            process::exit(2);
        }
    };

    let (mut oracle, start_frame) = load_lockstep_oracle(rom_path, &config);

    let mut last_trace = None;
    let mut last_semantic_trace = None;
    for local_frame in 0..config.frames {
        let frame = start_frame.wrapping_add(local_frame);
        let input = config.input_script.input_for_frame(frame);
        let run_what = select_run_what(&oracle.game.ram);
        match oracle.run_frame_with_compare(input, run_what) {
            Ok(()) => {}
            Err(OracleError::Diverged(report)) => {
                if config.trace_semantic_state {
                    let semantic_report = oracle.compare_current_semantic();
                    eprintln!("{semantic_report}");
                    let (mine, theirs) = oracle.semantic_snapshot_pair();
                    eprintln!("       semantic mine:   {mine}");
                    eprintln!("       semantic theirs: {theirs}");
                }
                eprintln!("{report}");
                if config.trace_state {
                    let mine = TraceState::from_ram(&oracle.game.ram, input, run_what);
                    let theirs = TraceState::from_ram(&oracle.snes.ram, input, run_what);
                    eprintln!("       mine:   {mine}");
                    eprintln!("       theirs: {theirs}");
                }
                eprintln!(
                    "game crate skeleton is still mostly empty; divergence is expected until frame logic is ported"
                );
                process::exit(1);
            }
            Err(e) => {
                eprintln!("frame {frame}: {e}");
                process::exit(1);
            }
        }
        if config.trace_state {
            let trace = TraceState::from_ram(&oracle.game.ram, input, run_what);
            if last_trace.as_ref() != Some(&trace) {
                eprintln!("{frame:>6}: {trace}");
                last_trace = Some(trace);
            }
        }
        if config.trace_semantic_state {
            let trace = oracle.semantic_game_snapshot();
            if last_semantic_trace.as_ref() != Some(&trace) {
                eprintln!("{frame:>6}: semantic {trace}");
                last_semantic_trace = Some(trace);
            }
        }
    }

    let completed_frame = start_frame.wrapping_add(config.frames);
    if let Some(save_path) = &config.save_state {
        if let Err(e) = save_lockstep_checkpoint(save_path, completed_frame, oracle) {
            eprintln!("failed to save checkpoint {}: {e}", save_path.display());
            process::exit(1);
        }
        println!(
            "saved lockstep checkpoint at frame {} to {}",
            completed_frame,
            save_path.display()
        );
        return;
    }

    println!(
        "lockstep completed {} frame(s) from frame {}; WRAM fnv1a64 = {:016x}",
        config.frames,
        start_frame,
        wram_digest(&oracle.snes)
    );
    if config.trace_semantic_state {
        println!("final semantic state: {}", oracle.semantic_game_snapshot());
    }
}

fn run_compare_lockstep_render(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --compare-lockstep-render <path-to-rom.sfc> [frames] [--input-script <path>] [--load-sram <path>] [--load-state <path>]"
            );
            process::exit(2);
        }
    };
    let config = match parse_lockstep_args(args) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{e}");
            process::exit(2);
        }
    };

    let (mut oracle, start_frame) = load_lockstep_oracle(rom_path, &config);
    let width = 256u32;
    let height = 224u32;
    let pitch = width as usize * 4;
    let mut game_frame = vec![0u8; width as usize * height as usize * 4];
    let mut snes_frame = vec![0u8; game_frame.len()];
    let mut input_history = Vec::new();

    for local_frame in 0..config.frames {
        let frame = start_frame.wrapping_add(local_frame);
        let input = config.input_script.input_for_frame(frame);
        let run_what = select_run_what(&oracle.game.ram);
        let pre_oracle = oracle.clone();
        input_history.push((frame, input));
        if let Err(e) = oracle.compare_current_non_render_state() {
            let artifact_dir = write_lockstep_parity_failure_artifacts(
                &pre_oracle,
                &oracle,
                frame,
                input,
                run_what,
                &input_history,
                e.to_string(),
            )
            .ok();
            eprintln!("frame {frame}: {e}");
            if let Some(dir) = artifact_dir {
                eprintln!("parity failure artifacts: {}", dir.display());
            }
            process::exit(1);
        }

        if let Err(e) = oracle.run_oracle_frame(input, run_what) {
            eprintln!("frame {frame}: {e}");
            process::exit(1);
        }
        oracle.game.run_frame_internal(input, run_what);

        if let Err(e) = oracle.compare_current_non_render_state() {
            let artifact_dir = write_lockstep_parity_failure_artifacts(
                &pre_oracle,
                &oracle,
                frame,
                input,
                run_what,
                &input_history,
                e.to_string(),
            )
            .ok();
            eprintln!("frame {frame}: {e}");
            if let Some(dir) = artifact_dir {
                eprintln!("parity failure artifacts: {}", dir.display());
            }
            process::exit(1);
        }

        if let Some(render_diff) = compare_diagnostic_oracle_render_frame(
            &oracle,
            &mut game_frame,
            &mut snes_frame,
            pitch,
            width as usize,
        ) {
            let artifact_dir = write_lockstep_parity_failure_artifacts(
                &pre_oracle,
                &oracle,
                frame,
                input,
                run_what,
                &input_history,
                format!(
                    "render divergence: mismatched_pixels={} first_mismatch=({}, {}) mine={:02x?} theirs={:02x?}",
                    render_diff.mismatched_pixels,
                    render_diff.first_pixel % width as usize,
                    render_diff.first_pixel / width as usize,
                    render_diff.mine_pixel,
                    render_diff.theirs_pixel
                ),
            )
            .ok();
            eprintln!(
                "render divergence at frame {frame}: mismatched_pixels={}; first_mismatch=({}, {}) mine={:02x?} theirs={:02x?}; input={input:04x}; run_what={run_what}",
                render_diff.mismatched_pixels,
                render_diff.first_pixel % width as usize,
                render_diff.first_pixel / width as usize,
                render_diff.mine_pixel,
                render_diff.theirs_pixel,
            );
            eprintln!("ppu mine:   {}", render_diff.mine_ppu);
            eprintln!("ppu theirs: {}", render_diff.theirs_ppu);
            eprintln!(
                "trace mine:   {}",
                TraceState::from_ram(&oracle.game.ram, input, run_what)
            );
            eprintln!(
                "trace theirs: {}",
                TraceState::from_ram(&oracle.snes.ram, input, run_what)
            );
            if let Some(dir) = artifact_dir {
                eprintln!("parity failure artifacts: {}", dir.display());
            }
            process::exit(1);
        }
    }

    println!(
        "lockstep render compare completed {} frame(s) from frame {}; WRAM fnv1a64 = {:016x}; mismatched_pixels=0; first_mismatch=none; main={:02x}; sub={:02x}; mode={}; screen={:02x}/{:02x}",
        config.frames,
        start_frame,
        wram_digest(&oracle.snes),
        oracle.game.ram[0x10],
        oracle.game.ram[0x11],
        oracle.game.ppu.mode,
        oracle.game.ppu.screen_enabled[0],
        oracle.game.ppu.screen_enabled[1],
    );
}

fn run_play_lockstep(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --play-lockstep <path-to-rom.sfc> [frames] [--load-sram <path>] [--load-state <path>]"
            );
            process::exit(2);
        }
    };
    let config = match parse_lockstep_args(args) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{e}");
            process::exit(2);
        }
    };
    let frame_limit = args
        .get(1)
        .filter(|candidate| !candidate.starts_with("--"))
        .map(|_| config.frames);
    if config.trace_state || config.trace_semantic_state || config.save_state.is_some() {
        eprintln!(
            "--play-lockstep only supports an optional frame limit, --input-script, --load-sram, --load-state, and --compare-bsnes-oracle"
        );
        process::exit(2);
    }
    if config.bsnes_core.is_some() && config.load_state.is_some() {
        eprintln!(
            "--play-lockstep --compare-bsnes-oracle currently requires a clean ROM start or --load-sram; bsnes full-state restore is not wired yet"
        );
        process::exit(2);
    }
    if config.bsnes_core.is_some() && config.compare_bsnes_audio && config.compare_bsnes_video {
        eprintln!("playable bsnes oracle enabled for video and audio");
    } else if config.bsnes_core.is_some() && config.compare_bsnes_video {
        eprintln!("playable bsnes oracle enabled for video");
    } else if config.bsnes_core.is_some() && config.compare_bsnes_audio {
        eprintln!("playable bsnes oracle enabled for audio");
    }
    if config.bsnes_core.is_some() && config.compare_from_frame != 0 {
        eprintln!(
            "playable bsnes oracle will start comparisons at frame {}",
            config.compare_from_frame
        );
    }

    let (mut oracle, start_frame) = load_lockstep_oracle(rom_path, &config);
    let mut bsnes = config.bsnes_core.as_ref().map(|core_path| {
        let core_path = core_path.to_string_lossy();
        let load_sram_bytes = config
            .load_sram
            .as_deref()
            .map(|path| read_file_or_exit(path, "SRAM"));
        match LibretroCore::load_with_sram(&core_path, rom_path, load_sram_bytes.as_deref()) {
            Ok(core) => core,
            Err(e) => {
                eprintln!("failed to initialize libretro core: {e}");
                process::exit(1);
            }
        }
    });
    let width = 256u32;
    let height = 224u32;
    let pitch = width as usize * 4;
    let mut game_frame = vec![0u8; width as usize * height as usize * 4];
    let mut snes_frame = vec![0u8; game_frame.len()];
    let mut renderer = match play_renderer::configured_from_env(
        width,
        height,
        NativeFrontendOptions::from_env(3, true),
    ) {
        Ok(renderer) => renderer,
        Err(e) => {
            eprintln!("failed to initialize play renderer: {e}");
            process::exit(1);
        }
    };
    let audio_samples = renderer.audio_samples_per_frame();
    let audio_channels = renderer.audio_channels();
    let mut audio = vec![0i16; audio_samples * audio_channels];
    let mut local_frame = 0u32;
    let mut input_history = Vec::new();
    let trace_live_input = env::var_os("ZELDA3_TRACE_LIVE_INPUT").is_some();
    let mut last_traced_live_input = u16::MAX;
    let bsnes_gpu_video_readback = (config.bsnes_core.is_some() && config.compare_bsnes_video)
        .then(|| load_modern_asset_gpu_readback_or_exit("play-lockstep bsnes video comparison"));

    while !renderer.quit_requested() && frame_limit.is_none_or(|limit| local_frame < limit) {
        let frame = start_frame.wrapping_add(local_frame);
        let live_input = renderer.poll_input();
        if trace_live_input && live_input != last_traced_live_input {
            eprintln!(
                "live-input frame={frame} input=0x{live_input:04x} main={} sub={} subsub={}",
                oracle.game.ram[TRACE_MAIN_MODULE_INDEX],
                oracle.game.ram[TRACE_SUBMODULE_INDEX],
                oracle.game.ram[TRACE_SUBSUBMODULE_INDEX],
            );
            last_traced_live_input = live_input;
        }
        let input = if config.input_script.is_empty() {
            live_input
        } else {
            config.input_script.input_for_frame(frame)
        };
        let run_what = select_run_what(&oracle.game.ram);
        let pre_oracle = oracle.clone();
        input_history.push((frame, input));
        if let Err(e) = oracle.run_frame_with_compare(input, run_what) {
            let artifact_dir = write_lockstep_parity_failure_artifacts(
                &pre_oracle,
                &oracle,
                frame,
                input,
                run_what,
                &input_history,
                e.to_string(),
            )
            .ok();
            eprintln!("frame {frame}: {e}");
            eprintln!(
                "trace mine:   {}",
                TraceState::from_ram(&oracle.game.ram, input, run_what)
            );
            eprintln!(
                "trace theirs: {}",
                TraceState::from_ram(&oracle.snes.ram, input, run_what)
            );
            if let Some(dir) = artifact_dir {
                eprintln!("parity failure artifacts: {}", dir.display());
            }
            process::exit(1);
        }
        if let Some(render_diff) = compare_diagnostic_oracle_render_frame(
            &oracle,
            &mut game_frame,
            &mut snes_frame,
            pitch,
            width as usize,
        ) {
            let artifact_dir = write_lockstep_parity_failure_artifacts(
                &pre_oracle,
                &oracle,
                frame,
                input,
                run_what,
                &input_history,
                format!(
                    "render divergence: mismatched_pixels={} first_mismatch=({}, {}) mine={:02x?} theirs={:02x?}",
                    render_diff.mismatched_pixels,
                    render_diff.first_pixel % width as usize,
                    render_diff.first_pixel / width as usize,
                    render_diff.mine_pixel,
                    render_diff.theirs_pixel
                ),
            )
            .ok();
            eprintln!(
                "render divergence at frame {frame}: mismatched_pixels={}; first_mismatch=({}, {}) mine={:02x?} theirs={:02x?}; input={input:04x}; run_what={run_what}",
                render_diff.mismatched_pixels,
                render_diff.first_pixel % width as usize,
                render_diff.first_pixel / width as usize,
                render_diff.mine_pixel,
                render_diff.theirs_pixel,
            );
            eprintln!("ppu mine:   {}", render_diff.mine_ppu);
            eprintln!("ppu theirs: {}", render_diff.theirs_ppu);
            eprintln!(
                "trace mine:   {}",
                TraceState::from_ram(&oracle.game.ram, input, run_what)
            );
            eprintln!(
                "trace theirs: {}",
                TraceState::from_ram(&oracle.snes.ram, input, run_what)
            );
            if let Some(dir) = artifact_dir {
                eprintln!("parity failure artifacts: {}", dir.display());
            }
            process::exit(1);
        }
        render_diagnostic_lockstep_oracle_frames_in_place(
            &mut oracle,
            &mut game_frame,
            &mut snes_frame,
            pitch,
        );

        let bsnes_capture = bsnes
            .as_mut()
            .map(|bsnes| bsnes.run_frame_with_input(input));
        oracle.game.zelda_push_apu_state();
        if let Some(capture) = &bsnes_capture {
            if config.compare_bsnes_audio && !capture.audio.is_empty() {
                if audio_channels != 2 {
                    eprintln!(
                        "--play-lockstep --compare-bsnes-oracle audio comparison requires stereo host audio; host has {audio_channels} channel(s)"
                    );
                    process::exit(2);
                }
                audio.resize(capture.audio.len(), 0);
                oracle
                    .game
                    .zelda_render_audio(&mut audio, (capture.audio.len() / 2) as i32, 2);
            } else {
                audio.resize(audio_samples * audio_channels, 0);
                oracle.game.zelda_render_audio(
                    &mut audio,
                    audio_samples as i32,
                    audio_channels as i32,
                );
            }
        } else {
            oracle
                .game
                .zelda_render_audio(&mut audio, audio_samples as i32, audio_channels as i32);
        }
        let mut bsnes_gpu_frame = None;
        if let Some(capture) = &bsnes_capture {
            let compare_this_frame = frame >= config.compare_from_frame;
            if compare_this_frame && config.compare_bsnes_video {
                let gpu_frame = render_modern_asset_gpu_frame_rgba_or_exit(
                    bsnes_gpu_video_readback
                        .as_ref()
                        .expect("GPU readback allocated for play-lockstep bsnes video comparison"),
                    &mut oracle.game,
                    "play-lockstep bsnes video comparison",
                );
                let video_diff = compare_bsnes_video_frame(&gpu_frame, width, height, capture);
                bsnes_gpu_frame = Some(gpu_frame);
                if let Some(video_diff) = video_diff {
                    let artifact_dir = write_bsnes_parity_failure_artifacts(
                        &pre_oracle.game,
                        &oracle.game,
                        bsnes_gpu_frame
                            .as_deref()
                            .expect("GPU frame rendered for play-lockstep bsnes video artifact"),
                        &audio,
                        capture,
                        frame,
                        input,
                        bsnes
                            .as_ref()
                            .map(|core| core.av_info.timing.sample_rate.round() as u32)
                            .unwrap_or(32_000),
                        format!("play-lockstep bsnes video divergence: {video_diff}"),
                    )
                    .ok();
                    eprintln!(
                        "bsnes video divergence at frame {frame}: {video_diff}; input={input:04x}; run_what={run_what}; main={:02x} sub={:02x} subsub={:02x}",
                        oracle.game.ram[0x10], oracle.game.ram[0x11], oracle.game.ram[0xb0],
                    );
                    eprintln!(
                        "trace mine:   {}",
                        TraceState::from_ram(&oracle.game.ram, input, run_what)
                    );
                    if let Some(dir) = artifact_dir {
                        eprintln!("parity failure artifacts: {}", dir.display());
                    }
                    process::exit(1);
                }
            }
            if compare_this_frame && config.compare_bsnes_audio {
                if let Some(audio_diff) = compare_bsnes_audio_frame(&audio, &capture.audio) {
                    let audio_artifact_frame;
                    let rust_artifact_frame = match bsnes_gpu_frame.as_deref() {
                        Some(frame) => frame,
                        None => {
                            let renderer = load_modern_asset_gpu_readback_or_exit(
                                "play-lockstep bsnes audio artifact frame",
                            );
                            audio_artifact_frame = render_modern_asset_gpu_frame_rgba_or_exit(
                                &renderer,
                                &mut oracle.game,
                                "play-lockstep bsnes audio artifact frame",
                            );
                            &audio_artifact_frame
                        }
                    };
                    let artifact_dir = write_bsnes_parity_failure_artifacts(
                        &pre_oracle.game,
                        &oracle.game,
                        rust_artifact_frame,
                        &audio,
                        capture,
                        frame,
                        input,
                        bsnes
                            .as_ref()
                            .map(|core| core.av_info.timing.sample_rate.round() as u32)
                            .unwrap_or(32_000),
                        format!("play-lockstep bsnes audio divergence: {audio_diff}"),
                    )
                    .ok();
                    eprintln!(
                        "bsnes audio divergence at frame {frame}: {audio_diff}; input={input:04x}; run_what={run_what}; main={:02x} sub={:02x} subsub={:02x}",
                        oracle.game.ram[0x10], oracle.game.ram[0x11], oracle.game.ram[0xb0],
                    );
                    eprintln!(
                        "rust audio:  {:?}",
                        AudioFrameStats::from_interleaved_stereo(&audio)
                    );
                    eprintln!(
                        "bsnes audio: {:?}",
                        AudioFrameStats::from_interleaved_stereo(&capture.audio)
                    );
                    eprintln!(
                        "rust audio debug: {}",
                        oracle.game.zelda_audio_debug_summary()
                    );
                    if let Some(dir) = artifact_dir {
                        eprintln!("parity failure artifacts: {}", dir.display());
                    }
                    process::exit(1);
                }
            }
        }
        renderer.push_audio(&audio);
        oracle.game.zelda_discard_unused_audio_frames();
        renderer.present_frame(&mut oracle.game);
        local_frame = local_frame.wrapping_add(1);
    }
}

fn summarize_audio_samples(samples: &[i16]) -> String {
    let nonzero = samples.iter().filter(|&&sample| sample != 0).count();
    let peak = samples
        .iter()
        .map(|&sample| (sample as i32).unsigned_abs())
        .max()
        .unwrap_or(0);
    let first_nonzero = samples.iter().position(|&sample| sample != 0);
    format!("nonzero={nonzero} peak={peak} first_nonzero={first_nonzero:?}")
}

fn load_lockstep_oracle(rom_path: &str, config: &LockstepConfig) -> (LockstepOracle, u32) {
    if let Some(load_path) = &config.load_state {
        match load_lockstep_checkpoint(load_path) {
            Ok(checkpoint) => return (checkpoint.oracle, checkpoint.frame),
            Err(e) => {
                eprintln!("failed to load checkpoint {}: {e}", load_path.display());
                process::exit(1);
            }
        }
    }

    let rom = match fs::read(rom_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("failed to read {rom_path}: {e}");
            process::exit(1);
        }
    };

    let mut oracle = LockstepOracle::new();
    if let Err(e) = oracle.load_rom(&rom) {
        eprintln!("{e}");
        process::exit(1);
    }
    if let Some(sram_path) = &config.load_sram {
        match fs::read(sram_path).and_then(|sram| {
            oracle
                .load_sram(&sram)
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
        }) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("failed to load {}: {e}", sram_path.display());
                process::exit(1);
            }
        }
    }
    if let Some(asset_path) = find_asset_pack(rom_path) {
        match fs::read(&asset_path).and_then(|assets| {
            oracle
                .load_assets(&assets)
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
        }) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("failed to load {}: {e}", asset_path.display());
                process::exit(1);
            }
        }
    }
    oracle.sync_game_from_oracle();
    (oracle, 0)
}

#[derive(Serialize, Deserialize)]
struct LockstepCheckpoint {
    magic: [u8; 8],
    frame: u32,
    oracle: LockstepOracle,
}

#[derive(Serialize, Deserialize)]
struct ApuBootstrapCheckpoint {
    magic: [u8; 8],
    opcodes: u64,
    apu_cycles_per_cpu_cycle: f64,
    cpu_k: u8,
    cpu_pc: u16,
    spc_pc: u16,
    rom_readable: bool,
    payload_nonzero: usize,
    dsp_writes: usize,
    apu: snes::apu::ApuState,
}

fn save_lockstep_checkpoint(
    path: &Path,
    frame: u32,
    oracle: LockstepOracle,
) -> Result<(), Box<dyn Error>> {
    let checkpoint = LockstepCheckpoint {
        magic: *LOCKSTEP_CHECKPOINT_MAGIC,
        frame,
        oracle,
    };
    let bytes = bincode::serialize(&checkpoint)?;
    fs::write(path, bytes)?;
    Ok(())
}

fn load_lockstep_checkpoint(path: &Path) -> Result<LockstepCheckpoint, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let checkpoint: LockstepCheckpoint = bincode::deserialize(&bytes)?;
    if &checkpoint.magic != LOCKSTEP_CHECKPOINT_MAGIC {
        return Err("not a zelda3-rs lockstep checkpoint".into());
    }
    Ok(checkpoint)
}

fn load_play_crash_checkpoint(path: &Path) -> Result<PlayCrashCheckpoint, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let checkpoint: PlayCrashCheckpoint = bincode::deserialize(&bytes)?;
    if &checkpoint.magic != PLAY_CRASH_CHECKPOINT_MAGIC {
        return Err("not a zelda3-rs playable crash checkpoint".into());
    }
    Ok(checkpoint)
}

fn load_apu_bootstrap_checkpoint(path: &Path) -> Result<ApuBootstrapCheckpoint, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let checkpoint: ApuBootstrapCheckpoint = bincode::deserialize(&bytes)?;
    if &checkpoint.magic != APU_BOOTSTRAP_CHECKPOINT_MAGIC {
        return Err("not a zelda3-rs APU bootstrap checkpoint".into());
    }
    Ok(checkpoint)
}

fn format_link_dma_trace(ram: &[u8]) -> String {
    let dma_graphics = u16::from_le_bytes([ram[0x100], ram[0x101]]);
    let dma_var1 = u16::from_le_bytes([ram[0x102], ram[0x103]]);
    let dma_var2 = u16::from_le_bytes([ram[0x104], ram[0x105]]);
    let dma_var3 = ram[0x107];
    let dma_var4 = ram[0x108];
    let dma_var5 = ram[0x109];
    format!(
        "raw=[graphics=${dma_graphics:04X},var1=${dma_var1:04X},var2=${dma_var2:04X},var3=${dma_var3:02X},var4=${dma_var4:02X},var5=${dma_var5:02X}] \
         index=[graphics={},var1={},var2={},var3={},var4={},var5={},var5_group={}] \
         table_lens=[sources3=27,sources4=8,sources5=3,sources6=128,sources7=16]",
        dma_graphics >> 1,
        dma_var1 >> 1,
        dma_var2 >> 1,
        dma_var3 >> 1,
        dma_var4 >> 1,
        dma_var5,
        dma_var5 >> 3
    )
}

#[derive(Debug)]
struct LockstepConfig {
    frames: u32,
    input_script: InputScript,
    trace_state: bool,
    trace_semantic_state: bool,
    save_state: Option<PathBuf>,
    load_state: Option<PathBuf>,
    load_sram: Option<PathBuf>,
    bsnes_core: Option<PathBuf>,
    compare_bsnes_video: bool,
    compare_bsnes_audio: bool,
    compare_from_frame: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TraceState {
    input: u16,
    run_what: u8,
    main: u8,
    sub: u8,
    subsub: u8,
    joy_h: u8,
    joy_l: u8,
    filtered_h: u8,
    filtered_l: u8,
    inidisp: u8,
    music_control: u8,
    sound_effect_ambient: u8,
    sound_effect_1: u8,
    sound_effect_2: u8,
    music_unk1: u8,
    sound_effect_ambient_last: u8,
    queued_music_control: u8,
    last_music_control: u8,
    attract_state: u8,
    attract_sequence: u8,
    dungeon_room: u16,
    dung_load_ptr: u16,
    dung_line_ptr: u16,
    dung_layout: u16,
    messaging_module: u8,
    text_render_state: u8,
    dialogue_read_pos: u16,
    text_wait2: u8,
    text_byte: u8,
    select_r16: u8,
    select_var3: u8,
    select_var4: u8,
    select_var5: u8,
    select_var7: u8,
    select_var8: u16,
    select_var9: u8,
    select_var10: u8,
    select_var11: u8,
    link_x: u16,
    link_y: u16,
    link_z: u16,
    link_y_vel: u8,
    link_x_vel: u8,
    link_direction: u8,
    link_direction_last: u8,
    link_direction_facing: u8,
    link_subpixel_z: u8,
    link_vel_z: u8,
    link_handler: u8,
    link_aux: u8,
    link_incapacitated_timer: u8,
    link_state_bits: u8,
    tile_action: u8,
    a_button_bits: u8,
    button_b_frames: u8,
    button_mask_b_y: u8,
    link_ability_flags: u8,
    lift_x1: u8,
    lift_x2: u8,
    sprite_pickup: u8,
    sprite_pickup_cached: u8,
    ancilla_pickup: u8,
    hand_item: u8,
    position_mode: u8,
    picking_throw_state: u8,
    player_timer: u8,
    r12: u16,
    r14: u16,
    tile_read: u16,
    tile_chest: u16,
    tile_misc: u16,
    tile_var1: u16,
    tile_diag: u16,
    moving_against_diag: u8,
    moving_deadlocked: u8,
    dir_mask_a: u8,
    dir_mask_b: u8,
    ortho_dirs: u8,
    tile_below: u8,
    standing_in_doorway: u8,
    safe_y: u16,
    safe_x: u16,
    bg2_x: u16,
    bg2_y: u16,
    sort_sprites: u8,
    sort_oam: u16,
    oam_cur: u16,
    oam_ext: u16,
    link_pose: u8,
    link_dma: [u8; 6],
    oam0: [u8; 16],
    sprite_types: [u8; 16],
    sprite_states: [u8; 16],
    sprite_oam_flags: [u8; 16],
    sprite_graphics: [u8; 16],
    uncle_ai: u8,
    uncle_d: u8,
    uncle_graphics: u8,
    uncle_x: u16,
    uncle_y: u16,
    sprite0_type: u8,
    sprite0_state: u8,
    sprite0_room: u8,
    sprite0_flags: u8,
    sprite0_flags2: u8,
    sprite0_flags3: u8,
    sprite0_flags4: u8,
    sprite0_flags5: u8,
    sprite0_defl: u8,
    sprite0_health: u8,
    sprite0_oam_flags: u8,
    sprite0_ai: u8,
    sprite0_d: u8,
    sprite0_graphics: u8,
    sprite0_x: u16,
    sprite0_y: u16,
}

impl TraceState {
    fn from_ram(ram: &[u8], input: u16, run_what: u8) -> Self {
        Self {
            input,
            run_what,
            main: ram[0x10],
            sub: ram[0x11],
            subsub: ram[0xb0],
            joy_h: ram[0xf0],
            joy_l: ram[0xf2],
            filtered_h: ram[0xf4],
            filtered_l: ram[0xf6],
            inidisp: ram[0x13],
            music_control: ram[0x12c],
            sound_effect_ambient: ram[0x12d],
            sound_effect_1: ram[0x12e],
            sound_effect_2: ram[0x12f],
            music_unk1: ram[0x130],
            sound_effect_ambient_last: ram[0x131],
            queued_music_control: ram[0x132],
            last_music_control: ram[0x133],
            attract_state: ram[0x22],
            attract_sequence: ram[0x23],
            dungeon_room: u16::from_le_bytes([ram[0xa0], ram[0xa1]]),
            dung_load_ptr: u16::from_le_bytes([ram[0xba], ram[0xbb]]),
            dung_line_ptr: u16::from_le_bytes([ram[0xbf], ram[0xc0]]),
            dung_layout: u16::from_le_bytes([ram[0x40e], ram[0x40f]]),
            messaging_module: ram[0x1cd8],
            text_render_state: ram[0x1cd4],
            dialogue_read_pos: u16::from_le_bytes([ram[0x1cd9], ram[0x1cda]]),
            text_wait2: ram[0x1ce9],
            text_byte: {
                let pos = u16::from_le_bytes([ram[0x1cd9], ram[0x1cda]]) as usize;
                ram.get(0x11200 + pos).copied().unwrap_or(0)
            },
            select_r16: ram[0xc8],
            select_var3: ram[0x0b10],
            select_var4: ram[0x0b12],
            select_var5: ram[0x0b15],
            select_var7: ram[0x0b11],
            select_var8: u16::from_le_bytes([ram[0x630], ram[0x631]]),
            select_var9: ram[0x0b13],
            select_var10: ram[0x0b16],
            select_var11: ram[0x0b14],
            link_x: u16::from_le_bytes([ram[0x22], ram[0x23]]),
            link_y: u16::from_le_bytes([ram[0x20], ram[0x21]]),
            link_z: u16::from_le_bytes([ram[0x24], ram[0x25]]),
            link_y_vel: ram[0x30],
            link_x_vel: ram[0x31],
            link_direction: ram[0x67],
            link_direction_last: ram[0x26],
            link_direction_facing: ram[0x2f],
            link_subpixel_z: ram[0x2c],
            link_vel_z: ram[0x29],
            link_handler: ram[0x5d],
            link_aux: ram[0x4d],
            link_incapacitated_timer: ram[0x46],
            link_state_bits: ram[0x308],
            tile_action: ram[0x36c],
            a_button_bits: ram[0x3b],
            button_b_frames: ram[0x3c],
            button_mask_b_y: ram[0x3a],
            link_ability_flags: ram[0xf379],
            lift_x1: ram[0x368],
            lift_x2: ram[0x36a],
            sprite_pickup: ram[0x314],
            sprite_pickup_cached: ram[0x2f4],
            ancilla_pickup: ram[0x2ec],
            hand_item: ram[0x301],
            position_mode: ram[0x37a],
            picking_throw_state: ram[0x309],
            player_timer: ram[0x300],
            r12: u16::from_le_bytes([ram[0x0c], ram[0x0d]]),
            r14: u16::from_le_bytes([ram[0x0e], ram[0x0f]]),
            tile_read: u16::from_le_bytes([ram[0x366], ram[0x367]]),
            tile_chest: u16::from_le_bytes([ram[0x2e5], ram[0x2e6]]),
            tile_misc: u16::from_le_bytes([ram[0x2f6], ram[0x2f7]]),
            tile_var1: u16::from_le_bytes([ram[0x62], ram[0x63]]),
            tile_diag: u16::from_le_bytes([ram[0x6e], ram[0x6f]]),
            moving_against_diag: ram[0x6b],
            moving_deadlocked: ram[0x6d],
            dir_mask_a: ram[0x42],
            dir_mask_b: ram[0x43],
            ortho_dirs: ram[0x6a],
            tile_below: ram[0x0fa5],
            standing_in_doorway: ram[0x6c],
            safe_y: u16::from_le_bytes([ram[0x3e], ram[0x40]]),
            safe_x: u16::from_le_bytes([ram[0x3f], ram[0x41]]),
            bg2_x: u16::from_le_bytes([ram[0xe2], ram[0xe3]]),
            bg2_y: u16::from_le_bytes([ram[0xe8], ram[0xe9]]),
            sort_sprites: ram[0x294],
            sort_oam: u16::from_le_bytes([ram[0x352], ram[0x353]]),
            oam_cur: u16::from_le_bytes([ram[0x90], ram[0x91]]),
            oam_ext: u16::from_le_bytes([ram[0x92], ram[0x93]]),
            link_pose: ram[0x37d],
            link_dma: [
                ram[0x100], ram[0x102], ram[0x104], ram[0x107], ram[0x108], ram[0x109],
            ],
            oam0: ram[0x800..0x810].try_into().unwrap(),
            sprite_types: ram[0x0e20..0x0e30].try_into().unwrap(),
            sprite_states: ram[0x0dd0..0x0de0].try_into().unwrap(),
            sprite_oam_flags: ram[0x0f50..0x0f60].try_into().unwrap(),
            sprite_graphics: ram[0x0dc0..0x0dd0].try_into().unwrap(),
            uncle_ai: ram[0xd80],
            uncle_d: ram[0xde0],
            uncle_graphics: ram[0xdc0],
            uncle_x: u16::from_le_bytes([ram[0xd10], ram[0xd30]]),
            uncle_y: u16::from_le_bytes([ram[0xd00], ram[0xd20]]),
            sprite0_type: ram[0x0e20],
            sprite0_state: ram[0x0dd0],
            sprite0_room: ram[0x0c9a],
            sprite0_flags: ram[0x0b6b],
            sprite0_flags2: ram[0x0e40],
            sprite0_flags3: ram[0x0e60],
            sprite0_flags4: ram[0x0f60],
            sprite0_flags5: ram[0x0be0],
            sprite0_defl: ram[0x0caa],
            sprite0_health: ram[0x0e50],
            sprite0_oam_flags: ram[0x0f50],
            sprite0_ai: ram[0x0d80],
            sprite0_d: ram[0x0de0],
            sprite0_graphics: ram[0x0dc0],
            sprite0_x: u16::from_le_bytes([ram[0x0d10], ram[0x0d30]]),
            sprite0_y: u16::from_le_bytes([ram[0x0d00], ram[0x0d20]]),
        }
    }
}

impl std::fmt::Display for TraceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "input=${:04X} run={} main={} sub={} subsub={} joyH=${:02X} joyL=${:02X} filtH=${:02X} filtL=${:02X} inidisp=${:02X} music=ctrl:{:02X}/amb:{:02X}/sfx:{:02X},{:02X}/unk:{:02X}/amb_last:{:02X}/queued:{:02X}/last:{:02X} attract_state={} attract_sequence={} room=${:04X} load_ptr=${:04X} line_ptr=${:04X} layout=${:04X} msg={} text_state={} read_pos=${:04X} wait2={} text_byte=${:02X} select=(r16={} grid_col={} name_col={} row={} scroll=${:04X} y={} v9={} v10={} v11={}) link=(${:04X},{:04X},z={:04X}/vel={:02X},{:02X}/dir={:02X}/last={:02X}/face={:02X}/sub={:02X}/vz={:02X}/h={}/aux={}/timer={}/state={:02X}) action=(tile={}/abit={:02X}/bframes={}/bmask={:02X}/ability={:02X}/lift={},{} pickup=s{:02X}/c{:02X}/a{:02X}/hand={:02X}/pos={:02X}/throw={:02X}/ptimer={}) coll=(r12={:04X}/r14={:04X}/read={:04X}/chest={:04X}/misc={:04X}/var1={:04X}/diag={:04X}/mad={:02X}/dead={:02X}/mask={:02X},{:02X}/orth={}/tile={:02X}/door={}/safe={:04X},{:04X}) bg2=({:04X},{:04X}) pose={} sort={} sort_oam=${:04X} oam=(${:04X},${:04X}) dma=[{:02X},{:02X},{:02X},{:02X},{:02X},{:02X}] oam0={:02X?} spr_t={:02X?} spr_st={:02X?} spr_oam={:02X?} spr_gfx={:02X?} uncle=(ai={} d={} gfx={} xy={:04X},{:04X}) sprite0=(type={:02X} st={:02X} room={:02X} flags={:02X}/{:02X}/{:02X}/{:02X}/{:02X} defl={:02X} hp={:02X} oam={:02X} ai={:02X} d={:02X} gfx={:02X} xy={:04X},{:04X})",
            self.input,
            self.run_what,
            self.main,
            self.sub,
            self.subsub,
            self.joy_h,
            self.joy_l,
            self.filtered_h,
            self.filtered_l,
            self.inidisp,
            self.music_control,
            self.sound_effect_ambient,
            self.sound_effect_1,
            self.sound_effect_2,
            self.music_unk1,
            self.sound_effect_ambient_last,
            self.queued_music_control,
            self.last_music_control,
            self.attract_state,
            self.attract_sequence,
            self.dungeon_room,
            self.dung_load_ptr,
            self.dung_line_ptr,
            self.dung_layout,
            self.messaging_module,
            self.text_render_state,
            self.dialogue_read_pos,
            self.text_wait2,
            self.text_byte,
            self.select_r16,
            self.select_var3,
            self.select_var4,
            self.select_var5,
            self.select_var8,
            self.select_var7,
            self.select_var9,
            self.select_var10,
            self.select_var11,
            self.link_x,
            self.link_y,
            self.link_z,
            self.link_y_vel,
            self.link_x_vel,
            self.link_direction,
            self.link_direction_last,
            self.link_direction_facing,
            self.link_subpixel_z,
            self.link_vel_z,
            self.link_handler,
            self.link_aux,
            self.link_incapacitated_timer,
            self.link_state_bits,
            self.tile_action,
            self.a_button_bits,
            self.button_b_frames,
            self.button_mask_b_y,
            self.link_ability_flags,
            self.lift_x1,
            self.lift_x2,
            self.sprite_pickup,
            self.sprite_pickup_cached,
            self.ancilla_pickup,
            self.hand_item,
            self.position_mode,
            self.picking_throw_state,
            self.player_timer,
            self.r12,
            self.r14,
            self.tile_read,
            self.tile_chest,
            self.tile_misc,
            self.tile_var1,
            self.tile_diag,
            self.moving_against_diag,
            self.moving_deadlocked,
            self.dir_mask_a,
            self.dir_mask_b,
            self.ortho_dirs,
            self.tile_below,
            self.standing_in_doorway,
            self.safe_x,
            self.safe_y,
            self.bg2_x,
            self.bg2_y,
            self.link_pose,
            self.sort_sprites,
            self.sort_oam,
            self.oam_cur,
            self.oam_ext,
            self.link_dma[0],
            self.link_dma[1],
            self.link_dma[2],
            self.link_dma[3],
            self.link_dma[4],
            self.link_dma[5],
            self.oam0,
            self.sprite_types,
            self.sprite_states,
            self.sprite_oam_flags,
            self.sprite_graphics,
            self.uncle_ai,
            self.uncle_d,
            self.uncle_graphics,
            self.uncle_x,
            self.uncle_y,
            self.sprite0_type,
            self.sprite0_state,
            self.sprite0_room,
            self.sprite0_flags,
            self.sprite0_flags2,
            self.sprite0_flags3,
            self.sprite0_flags4,
            self.sprite0_flags5,
            self.sprite0_defl,
            self.sprite0_health,
            self.sprite0_oam_flags,
            self.sprite0_ai,
            self.sprite0_d,
            self.sprite0_graphics,
            self.sprite0_x,
            self.sprite0_y
        )
    }
}

pub(crate) fn select_run_what(ram: &[u8]) -> u8 {
    let is_nmi_thread_active = ram[0x12a] != 0;
    let thread_other_stack = u16::from_le_bytes([ram[0x1f0a], ram[0x1f0b]]);
    if is_nmi_thread_active && thread_other_stack != 0x1f31 {
        RUN_POLY
    } else {
        RUN_MAIN
    }
}

fn parse_lockstep_args(args: &[String]) -> Result<LockstepConfig, Box<dyn Error>> {
    let mut frames = 1;
    let mut script = InputScript::default();
    let mut trace_state = false;
    let mut trace_semantic_state = false;
    let mut save_state = None;
    let mut load_state = None;
    let mut load_sram = None;
    let mut bsnes_core = None;
    let mut compare_bsnes_video = true;
    let mut compare_bsnes_audio = true;
    let mut compare_from_frame = 0u32;
    let mut i = 1;
    if let Some(candidate) = args.get(i) {
        if !candidate.starts_with("--") {
            frames = candidate.parse::<u32>()?;
            i += 1;
        }
    }
    while i < args.len() {
        match args[i].as_str() {
            "--input-script" => {
                let path = args.get(i + 1).ok_or("--input-script requires a path")?;
                script = InputScript::from_path(path)?;
                i += 2;
            }
            "--trace-state" => {
                trace_state = true;
                i += 1;
            }
            "--trace-semantic-state" => {
                trace_semantic_state = true;
                i += 1;
            }
            "--save-state" => {
                let path = args.get(i + 1).ok_or("--save-state requires a path")?;
                save_state = Some(PathBuf::from(path));
                i += 2;
            }
            "--load-state" => {
                let path = args.get(i + 1).ok_or("--load-state requires a path")?;
                load_state = Some(PathBuf::from(path));
                i += 2;
            }
            "--load-sram" => {
                let path = args.get(i + 1).ok_or("--load-sram requires a path")?;
                load_sram = Some(PathBuf::from(path));
                i += 2;
            }
            "--compare-bsnes-oracle" => {
                let path = args
                    .get(i + 1)
                    .ok_or("--compare-bsnes-oracle requires a path to bsnes_libretro.dylib")?;
                bsnes_core = Some(PathBuf::from(path));
                i += 2;
            }
            "--ignore-video" => {
                compare_bsnes_video = false;
                i += 1;
            }
            "--ignore-audio" => {
                compare_bsnes_audio = false;
                i += 1;
            }
            "--compare-from-frame" => {
                let path = args
                    .get(i + 1)
                    .ok_or("--compare-from-frame requires a frame number")?;
                compare_from_frame = path.parse()?;
                i += 2;
            }
            flag => return Err(format!("unknown lockstep option: {flag}").into()),
        }
    }
    if load_state.is_some() && load_sram.is_some() {
        return Err(
            "--load-sram cannot be combined with --load-state; checkpoints already include SRAM"
                .into(),
        );
    }

    Ok(LockstepConfig {
        frames,
        input_script: script,
        trace_state,
        trace_semantic_state,
        save_state,
        load_state,
        load_sram,
        bsnes_core,
        compare_bsnes_video,
        compare_bsnes_audio,
        compare_from_frame,
    })
}

/// FNV-1a 64-bit, just so two runs can be compared without pulling in a
/// hashing dep. Not cryptographic — fine for a state digest.
fn wram_digest(snes: &Snes) -> u64 {
    fnv1a64(&snes.ram)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn format_optional_u8(value: Option<u8>) -> String {
    value
        .map(|value| format!("{value:02x}"))
        .unwrap_or_else(|| "none".to_string())
}

fn format_optional_u64(value: Option<u64>) -> String {
    value
        .map(|value| format!("{value:016x}"))
        .unwrap_or_else(|| "none".to_string())
}

fn libretro_memory_name(id: c_uint) -> &'static str {
    match id {
        RETRO_MEMORY_SAVE_RAM => "SAVE_RAM",
        RETRO_MEMORY_RTC => "RTC",
        RETRO_MEMORY_SYSTEM_RAM => "SYSTEM_RAM",
        RETRO_MEMORY_VIDEO_RAM => "VIDEO_RAM",
        _ => "UNKNOWN",
    }
}

fn find_asset_pack(rom_path: &str) -> Option<PathBuf> {
    let rom_dir_asset = Path::new(rom_path).with_file_name("zelda3_assets.dat");
    if rom_dir_asset.is_file() {
        return Some(rom_dir_asset);
    }

    let cwd_asset = PathBuf::from("zelda3_assets.dat");
    if cwd_asset.is_file() {
        return Some(cwd_asset);
    }

    let repo_asset = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("zelda3_assets.dat");
    repo_asset.is_file().then_some(repo_asset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_optional_lockstep_frames() {
        let args = vec![
            "rom.sfc".to_string(),
            "42".to_string(),
            "--input-script".to_string(),
            "/does/not/matter".to_string(),
        ];
        let err = parse_lockstep_args(&args).unwrap_err().to_string();
        assert!(err.contains("No such file") || err.contains("does/not/matter"));

        let args = vec!["rom.sfc".to_string(), "42".to_string()];
        let config = parse_lockstep_args(&args).unwrap();
        assert_eq!(config.frames, 42);
    }

    #[test]
    fn parses_lockstep_semantic_trace_flag() {
        let args = vec![
            "rom.sfc".to_string(),
            "42".to_string(),
            "--trace-semantic-state".to_string(),
        ];

        let config = parse_lockstep_args(&args).unwrap();

        assert_eq!(config.frames, 42);
        assert!(config.trace_semantic_state);
        assert!(!config.trace_state);
    }

    #[test]
    fn route_coverage_frame_reads_route_surface_ids_from_ram() {
        let mut game = ZeldaState::default();
        game.ram[TRACE_MAIN_MODULE_INDEX] = 0x07;
        game.ram[TRACE_SUBMODULE_INDEX] = 0x02;
        game.ram[TRACE_SUBSUBMODULE_INDEX] = 0x03;
        game.ram[0x1b] = 1;
        game.ram[0x48e] = 0xa4;
        game.ram[0x48f] = 0x00;
        game.ram[0x8a] = 0x40;
        game.ram[0x8b] = 0x00;
        game.ram[0x0dd0] = 9;
        game.ram[0x0e20] = 0xcb;
        game.ram[0x0dd1] = 0;
        game.ram[0x0e21] = 0xcc;
        game.ram[0x0c4a] = 0x05;
        game.ram[0x0c5e] = 0x12;
        game.ram[0x0202] = 0x11;

        let frame = route_coverage_frame_from_game(42, &game);

        assert_eq!(frame.frame, 42);
        assert_eq!(frame.main_module, 0x07);
        assert_eq!(frame.submodule, 0x02);
        assert_eq!(frame.subsubmodule, 0x03);
        assert_eq!(frame.indoor_room, Some(0x00a4));
        assert_eq!(frame.overworld_screen, None);
        assert_eq!(frame.sprite_types, vec![0xcb]);
        assert_eq!(frame.ancilla_types, vec![0x05]);
        assert_eq!(frame.active_item, Some(0x11));
    }

    #[test]
    fn route_coverage_frame_preserves_active_sprite_type_zero() {
        let mut game = ZeldaState::default();
        game.ram[0x0dd0] = 9;
        game.ram[0x0e20] = 0x00;

        let frame = route_coverage_frame_from_game(1, &game);

        assert_eq!(frame.sprite_types, vec![0x00]);
    }
}
