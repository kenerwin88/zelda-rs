//! zelda3-rs prototype binary.
//!
//! Default execution runs the native playable host: load ROM/assets/SRAM, step
//! `ZeldaState`, present PPU pixels, queue audio, and save SRAM on quit.
//! `--headless` preserves the raw opcode-budget emulator harness. Snes9x
//! (`--compare-snes9x-oracle`) is the only parity oracle.

mod asset_palette_commands;
mod asset_source_dump_commands;
mod audio_trace;
#[cfg(all(feature = "parity-debug", feature = "rom-timing"))]
mod pcm_windows;
mod developer_destinations;
mod developer_modern_map;
mod developer_room_commands;
#[cfg(feature = "rom-timing")]
mod dsp_phase_ledger;
mod frame_dump_commands;
mod gpu_capture;
mod hd_authoring_commands;
mod image_output;
mod index_dump_commands;
mod index_source_keys;
mod indexed_tile_sheet;
mod input_script;
#[cfg(feature = "rom-timing")]
mod libretro_core;
#[cfg(feature = "rom-timing")]
mod libretro_timeline;
#[cfg(feature = "rom-timing")]
mod cartridge_input_recording;
mod live_input_recording;
mod lagless_replay;
mod logical_replay;
mod oracle_lock;
mod overworld_dump_commands;
mod play_commands;
mod play_renderer;
mod render_diagnostics;
#[cfg(feature = "rom-timing")]
mod replay_diagnostics;
#[cfg(feature = "rom-timing")]
mod replay_save_config;
mod route_coverage_commands;
mod sheet_dump_commands;
#[cfg(feature = "rom-timing")]
mod smp_opcode_ledger;
#[cfg(feature = "rom-timing")]
mod snes9x_apu_tools;
#[cfg(feature = "rom-timing")]
mod snes9x_compare;
#[cfg(feature = "rom-timing")]
mod snes9x_frame_dump;
#[cfg(feature = "rom-timing")]
mod snes9x_presented_bg_scroll;
#[cfg(feature = "rom-timing")]
mod snes9x_presented_bg_tilemaps;
#[cfg(feature = "rom-timing")]
mod snes9x_presented_dialogue_text;
#[cfg(feature = "rom-timing")]
mod snes9x_presented_mode7;
#[cfg(feature = "rom-timing")]
mod snes9x_presented_window_mask;
#[cfg(feature = "rom-timing")]
mod snes9x_record_commands;
#[cfg(feature = "rom-timing")]
mod snes9x_route_recorder;
#[cfg(feature = "rom-timing")]
mod snes9x_segment_matrix;
#[cfg(feature = "rom-timing")]
mod snes9x_semantic_receipts;
mod state_ledger;
#[cfg(feature = "rom-timing")]
use libretro_core::*;
#[cfg(feature = "rom-timing")]
use snes9x_apu_tools::*;
#[cfg(feature = "rom-timing")]
use snes9x_compare::*;
#[cfg(feature = "rom-timing")]
use snes9x_record_commands::*;

use std::backtrace::Backtrace;
use std::env;
#[cfg(feature = "rom-timing")]
use std::ffi::{CStr, CString};
use std::fs;
use std::io::Write;
#[cfg(feature = "rom-timing")]
use std::os::raw::{c_char, c_uint, c_void};
use std::panic::{self, AssertUnwindSafe, PanicHookInfo};
use std::path::{Path, PathBuf};
use std::process;
#[cfg(feature = "rom-timing")]
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use asset_palette_commands::run_dump_reference_palette;
use asset_source_dump_commands::run_dump_assets_by_source;
#[cfg(feature = "rom-timing")]
use audio_trace::replay_checksum_samples;
use developer_room_commands::{run_dump_developer_destination, run_dump_developer_tileset};
use frame_dump_commands::{
    run_dump_frame, run_dump_overworld_screen, run_dump_replay_checkpoint_ppu,
    run_scan_replay_checkpoints, run_smoke_asset_gpu,
};
#[cfg(feature = "rom-timing")]
use gpu_capture::render_live_game_gpu_frame_rgba;
use gpu_capture::ModernAssetGpuReadbackRenderer;
use hd_authoring_commands::{run_dump_hd_capture, run_slice_hd_cells};
#[cfg(feature = "rom-timing")]
use image_output::write_rgba_frame_png;
use index_dump_commands::{run_dump_dungeon_index_tiles, run_dump_sprite_index_tiles};
use overworld_dump_commands::{run_dump_unique_overworld_cells, run_dump_unique_overworld_tiles};
use play_commands::{run_frontend_smoke, run_play};
#[cfg(feature = "rom-timing")]
use play_commands::run_rom_timing_play;
use render_diagnostics::format_render_ppu_summary;
#[cfg(feature = "rom-timing")]
use replay_diagnostics::{
    replay_checksum_bytes, replay_checksum_ram_range, replay_save_ancilla_dump,
    replay_save_door_dump, replay_save_dungeon_attr_dump, replay_save_dungmap_dump,
    replay_save_garnish_dump, replay_save_message_dump, replay_save_overlord_dump,
    replay_save_palette_dump, replay_save_ram0000_dump, replay_save_ram0400_dump,
    replay_save_ram_page_dump, replay_save_requested_ram_page_dump, replay_save_room_history_dump,
    replay_save_room_mask, replay_save_room_mask_dump, replay_save_sprite_dump,
    replay_sram_checksum_ok,
};
#[cfg(feature = "rom-timing")]
use replay_save_config::{parse_replay_save_args_or_exit, ReplaySaveConfig};
use route_coverage_commands::run_coverage_probe;
#[cfg(any(test, feature = "rom-timing"))]
use route_coverage_commands::route_coverage_frame_from_game;
#[cfg(feature = "rom-timing")]
use route_coverage_commands::write_route_coverage_log_or_exit;
use serde::{Deserialize, Serialize};
use sheet_dump_commands::{run_dump_dungeon_sheet_png, run_dump_sprite_sheet_png};
use snes::{consts::PPU_EXTRA_LEFT_RIGHT, cpu_run_opcode, load_rom, Snes};
use zelda3::{config::parse_config_file_context, ZeldaState, RUN_MAIN, RUN_POLY};

/// Positional-bincode checkpoint layout/ownership version. Bump the trailing number
/// whenever a serialized `ZeldaState` field is added, removed or reordered;
/// older `rust.z3state` files are then rejected with a clear message instead
/// of failing mid-decode. 06 (2026-09-08): save progress owns named fields
/// and dungeon-room records instead of an overlapping 0x500-byte save bank.
/// 08 (2026-09-09): the world transient owns a 32-word overworld map16 stripe
/// instead of a 0x400-word mirror of the dungeon replacement bank. 09
/// (2026-09-09): the scroll copy no longer shadows the palette backup bank.
/// 10 (2026-09-09): the palette theme and sprite system no longer mirror the
/// dungeon entrance backup. 11 (2026-09-09): the world transient no longer
/// duplicates the HUD item slot, the travel-bird byte, or the layer masks,
/// and the entrance id is its original byte. 12 (2026-09-09): the world
/// scroll state no longer mirrors the room-bound words. 13 (2026-09-09): the
/// world transient dropped its two dead HUD timer fields. 18 (2026-09-10):
/// the sprite workspace holds two zero-page words, not sixteen bytes. 19
/// (2026-09-10): the display state no longer mirrors the star-tile phase. 20
/// (2026-09-10): the attract scene no longer mirrors the mode-7 zoom timer and
/// the dungeon room items no longer mirror the map-state word. 21
/// (2026-09-10): the dungeon environment no longer mirrors the water counter.
/// 23 (2026-10-23): inventory resources/equipment are authoritative owners;
/// older checkpoints can contain native fields saved before deferred adoption.
/// 24 (2026-10-23): save-progress records/runtime bytes are authoritative too.
const PLAY_CRASH_CHECKPOINT_MAGIC: &[u8; 8] = b"Z3RSPC27";
#[cfg(feature = "rom-timing")]
const ACTION_TILE_X: [i16; 4] = [7, 7, -3, 16];
#[cfg(feature = "rom-timing")]
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
    #[cfg(not(feature = "rom-timing"))]
    if args.iter().any(|arg| {
        (arg.starts_with("--") && arg.contains("snes9x"))
            || matches!(arg.as_str(), "--rom-timing" | "--trace-startup-audio"
                | "--trace-song-bank" | "--trace-rom-apu-upload" | "--trace-spc-driver-startup"
                | "--capture-rom-apu-bootstrap" | "--compare-bootstrap-apu-startup"
                | "--replay-save" | "--smoke-render")
    }) {
        eprintln!("this command requires the rom-timing Cargo feature");
        process::exit(2);
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--generate-snes9x-smp-opcode-ledger") {
        smp_opcode_ledger::run_generate_command(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--generate-snes9x-dsp-phase-ledger") {
        dsp_phase_ledger::run_generate_command(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if dispatch_rom_first_oracle_flags(&args) {
        return;
    }
    if let Some(error) = rom_first_oracle_flag_error(&args) {
        eprintln!("{error}");
        process::exit(2);
    }
    if args.get(1).map(String::as_str) == Some("--headless") {
        run_headless(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--standalone-smoke") {
        run_standalone_smoke(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--render-audio-cpu") {
        #[cfg(all(feature = "parity-debug", feature = "rom-timing"))]
        {
            pcm_windows::run(&args[2..]);
            return;
        }
        #[cfg(not(all(feature = "parity-debug", feature = "rom-timing")))]
        {
            eprintln!("--render-audio-cpu requires parity-debug and rom-timing");
            process::exit(2);
        }
    }
    if args.get(1).map(String::as_str) == Some("--render-original-audio-cpu") {
        #[cfg(all(feature = "parity-debug", feature = "rom-timing"))]
        {
            pcm_windows::run_original(&args[2..]);
            return;
        }
        #[cfg(not(all(feature = "parity-debug", feature = "rom-timing")))]
        {
            eprintln!("--render-original-audio-cpu requires parity-debug and rom-timing");
            process::exit(2);
        }
    }
    if args.get(1).map(String::as_str) == Some("--sram-smoke") {
        run_sram_smoke();
        return;
    }
    if args.get(1).map(String::as_str) == Some("--frontend-smoke") {
        run_frontend_smoke(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--smoke-render") {
        run_smoke_render(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--smoke-asset-gpu") {
        run_smoke_asset_gpu(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--trace-startup-audio") {
        run_trace_startup_audio(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--trace-snes9x-audio") {
        run_trace_snes9x_audio(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--compare-snes9x-startup-audio") {
        run_compare_snes9x_startup_audio(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--compare-snes9x-oracle") {
        run_compare_snes9x_oracle(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--replay-cached-snes9x-av") {
        run_replay_cached_snes9x_av(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--capture-snes9x-av") {
        run_capture_snes9x_av(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--validate-snes9x-replay") {
        run_validate_snes9x_replay(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--run-snes9x-script") {
        run_snes9x_script(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--record-snes9x-route") {
        run_record_snes9x_route(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--build-snes9x-segment-matrix") {
        run_build_snes9x_segment_matrix(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--dump-snes9x-frame") {
        run_dump_snes9x_frame(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--trace-snes9x-memory") {
        run_trace_snes9x_memory(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--trace-song-bank") {
        run_trace_song_bank(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--trace-rom-apu-upload") {
        run_trace_rom_apu_upload(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--trace-snes9x-spc-window") {
        run_trace_snes9x_spc_window(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--dump-snes9x-apu-ram") {
        run_dump_snes9x_apu_ram(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--trace-spc-driver-startup") {
        run_trace_spc_driver_startup(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--capture-rom-apu-bootstrap") {
        run_capture_rom_apu_bootstrap(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--compare-bootstrap-apu-startup") {
        run_compare_bootstrap_apu_startup(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--snes9x-dump-frames") {
        snes9x_frame_dump::run_snes9x_dump_frames(&args[2..]);
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
    if args.get(1).map(String::as_str) == Some("--replay-crash") {
        run_replay_crash(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--replay-save") {
        run_replay_save(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--logical-replay") {
        logical_replay::run_logical_replay(&args[2..]);
        return;
    }
    if matches!(args.get(1).map(String::as_str), Some("--replay-lagless" | "--convert-lagless")) {
        lagless_replay::run(&args[2..], args[1] == "--convert-lagless");
        return;
    }
    if args.get(1).map(String::as_str) == Some("--record-lagless") {
        let dir = args.get(2).filter(|_| args.len() == 3).unwrap_or_else(|| {
            eprintln!("usage: zelda3 --record-lagless SESSION_DIRECTORY");
            process::exit(2);
        });
        play_commands::run_record_lagless(Path::new(dir));
        return;
    }
    if args.get(1).map(String::as_str) == Some("--coverage-probe") {
        run_coverage_probe(&args[2..]);
        return;
    }
    #[cfg(feature = "rom-timing")]
    if args.get(1).map(String::as_str) == Some("--rom-timing") {
        run_rom_timing_play(args.get(2).map(String::as_str));
        return;
    }
    // A cartridge path is accepted for compatibility; the lagless game runs
    // from the embedded asset pack.
    run_play();
}

fn rom_first_oracle_flag_error(args: &[String]) -> Option<&'static str> {
    let rom_path = args.get(1)?;
    if rom_path.starts_with("--") {
        return None;
    }
    args[2..]
        .iter()
        .any(|arg| arg == "--snes9x-core")
        .then_some(
            "--snes9x-core is not a supported parity flag; use \
             --compare-snes9x-oracle <path-to-core> so the command cannot silently launch play mode",
        )
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

#[cfg(feature = "rom-timing")]
fn dispatch_rom_first_oracle_flags(args: &[String]) -> bool {
    let Some(rom_path) = args.get(1) else {
        return false;
    };
    if rom_path.starts_with("--") || args.len() <= 2 {
        return false;
    }

    let tail = &args[2..];
    let has_snes9x = tail.iter().any(|arg| arg == "--compare-snes9x-oracle");
    if !has_snes9x {
        return false;
    }

    let mut forwarded = Vec::new();
    let mut passthrough = Vec::new();
    let mut i = 0usize;
    while i < tail.len() {
        match tail[i].as_str() {
            "--compare-snes9x-oracle" => {
                let Some(core_path) = tail.get(i + 1) else {
                    eprintln!("--compare-snes9x-oracle requires a path to a SNES libretro core");
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
        eprintln!("--compare-snes9x-oracle requires a path to a SNES libretro core");
        process::exit(2);
    }
    forwarded.push(rom_path.clone());
    forwarded.extend(passthrough);
    run_compare_snes9x_oracle(&forwarded);
    true
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
    // codebase has no built-in frame timer, so we drive raw opcodes
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
    let options = args.get(1..).unwrap_or_default();
    let lag_free = options.iter().any(|arg| arg == "--lag-free");
    let ledger_options: Vec<_> = options.iter().filter(|arg| *arg != "--lag-free").cloned().collect();
    let mut state_ledger = state_ledger::Ledger::open(&ledger_options)
        .unwrap_or_else(|e| { eprintln!("state ledger: {e}"); process::exit(2) });
    if state_ledger.is_some() && args.first().is_none_or(|value| value.parse::<u32>().is_err()) {
        eprintln!("state ledger requires an explicit valid frame count");
        process::exit(2);
    }
    let started = std::time::Instant::now();
    // `ZELDA3_SMOKE_INPUT_PULSE=<mask>` presses the joypad mask for eight of
    // every sixteen frames, enough to walk the ROM-less build from the title
    // through the file select into the game.
    let pulse: u16 = env::var("ZELDA3_SMOKE_INPUT_PULSE")
        .ok()
        .and_then(|v| u16::from_str_radix(v.trim_start_matches("0x"), 16).ok())
        .unwrap_or(0);
    // `ZELDA3_SMOKE_INPUT_LEDGER=<oracle-av-hashes.jsonl>` replays a cached
    // route's per-frame inputs instead of the pulse, and
    // `ZELDA3_SMOKE_SRAM=<initial.srm>` seeds the save the route started from,
    // so the ROM-less build can be walked into the game's dialogue modules.
    let ledger_inputs: Vec<u16> = env::var_os("ZELDA3_SMOKE_INPUT_LEDGER")
        .map(|path| {
            if state_ledger.is_some() {
                return state_ledger::route_inputs(std::path::Path::new(&path), frames)
                    .unwrap_or_else(|e| { eprintln!("state ledger inputs: {e}"); process::exit(2) });
            }
            let text = fs::read_to_string(&path).unwrap_or_else(|e| {
                eprintln!("failed to read {path:?}: {e}");
                process::exit(2);
            });
            text.lines()
                .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
                .filter_map(|record| {
                    record.get("input")?.as_str().and_then(|input| {
                        u16::from_str_radix(input.trim_start_matches("0x"), 16).ok()
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    // `ZELDA3_SMOKE_ROM=<rom>` loads the ROM from a file instead, to compare
    // the two launches frame for frame.
    let mut game = match env::var("ZELDA3_SMOKE_ROM") {
        Ok(rom_path) => load_play_state(&rom_path),
        Err(_) => load_romless_play_state(),
    };
    // Match logical_replay's fresh-engine initialization, before SRAM and frame 0.
    if lag_free {
        game.set_lag_free(true);
    }
    // `ZELDA3_SMOKE_KEEP_SRAM=1` keeps the loaded save so the pulse can enter
    // a saved game (the spawn-select dialogue) instead of the name entry.
    if env::var_os("ZELDA3_SMOKE_KEEP_SRAM").is_none() {
        game.sram.fill(0);
    }
    if let Some(path) = env::var_os("ZELDA3_SMOKE_SRAM") {
        let sram = fs::read(&path).unwrap_or_else(|e| {
            eprintln!("failed to read {path:?}: {e}");
            process::exit(2);
        });
        let len = sram.len().min(game.sram.len());
        game.sram[..len].copy_from_slice(&sram[..len]);
    }
    let mut audio = vec![0i16; 735 * 2];

    for frame in 0..frames {
        let input = match ledger_inputs.get(frame as usize) {
            Some(&input) => input,
            None if pulse != 0 && frame % 16 < 8 => pulse,
            None => 0,
        };
        game.zelda_run_frame(i32::from(input));
        if frame % 100 == 0 && env::var_os("ZELDA3_SMOKE_TRACE").is_some() {
            eprintln!(
                "smoke frame={frame} main={:#04x} sub={:#04x} ram_fnv1a64={:016x}",
                game.wram_image()[0x10],
                game.wram_image()[0x11],
                fnv1a64(&game.wram_image())
            );
        }
        game.zelda_render_audio(&mut audio, 735, 2);
        game.zelda_discard_unused_audio_frames();
        // Present the frame the way the play loop does (without a GPU), so the
        // display publication pipeline runs and the walk matches real play.
        let display = crate::gpu_capture::capture_gpu_frame_from_game(&mut game);
        if let Some(ledger) = &mut state_ledger {
            if frame % ledger.interval == 0 || frame + 1 == frames {
                let record = ledger.capture(frame, input, &game, &audio, &display);
                ledger.observe(&record).unwrap_or_else(|e| {
                    eprintln!("{e}"); process::exit(1)
                });
            }
        }
    }

    if let Some(ledger) = state_ledger {
        ledger.finish().unwrap_or_else(|e| { eprintln!("state ledger: {e}"); process::exit(1) });
        let seconds = started.elapsed().as_secs_f64();
        eprintln!("state ledger completed frames={frames} seconds={seconds:.3} fps={:.1}", f64::from(frames) / seconds);
    }

    println!(
        "standalone smoke completed frames={frames} main_module={:#04x} ram_fnv1a64={:016x} sram_fnv1a64={:016x}",
        game.wram_image()[0x10],
        fnv1a64(&game.wram_image()),
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

#[cfg(feature = "rom-timing")]
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
        game.wram_image()[0x10],
        game.wram_image()[0x11],
        game.ppu.bg_mode(),
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

#[cfg(feature = "rom-timing")]
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

#[cfg(feature = "rom-timing")]
fn run_replay_save(args: &[String]) {
    let ReplaySaveConfig {
        rom_path,
        replay_path,
        max_frames,
        dump_frame_path,
        audio_trace_log,
        coverage_log,
        asset_gpu_smoke,
        asset_gpu_progress_interval,
        asset_gpu_missing_assets_out,
        asset_gpu_checkpoint_dir,
        asset_gpu_checkpoint_interval,
        ppu_mode_summary,
        save_state_path,
        save_state_at,
        load_state_path,
        load_sram_path,
        input_script,
        input_script_overlay,
        stop_replay_after_load,
    } = parse_replay_save_args_or_exit(args);
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
    let mut audio_trace_buffer = if audio_trace_log != 0 {
        Some(vec![0i16; 735 * 2])
    } else {
        None
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
    let mut asset_gpu_smoke_renderer = if asset_gpu_smoke {
        Some(load_modern_asset_gpu_readback_or_exit(
            "replay-save asset GPU smoke",
        ))
    } else {
        None
    };
    let capture_panic_pre_frame =
        std::env::var_os("ZELDA3_REPLAY_CAPTURE_PANIC_PRE_FRAME").is_some();
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
            let mode = usize::from(game.ppu.bg_mode());
            if mode < ppu_mode_counts.len() {
                ppu_mode_counts[mode] += 1;
            }
            if game.ppu.bg_mode() == 7 {
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
            if asset_gpu_progress_interval != 0
                && frames.is_multiple_of(asset_gpu_progress_interval)
            {
                print_asset_gpu_smoke_progress("replay-save", frames, &game, renderer);
            }
        }
        if let Some(audio) = audio_trace_buffer.as_mut() {
            game.zelda_render_audio(audio, 735, 2);
            game.zelda_discard_unused_audio_frames();
            if audio_trace_log != 0 && frames.is_multiple_of(audio_trace_log) {
                let stats = game.zelda_modern_audio_last_stats();
                let s_samples = replay_checksum_samples(audio);
                println!(
                    "audio frame={frames} samples=0x{s_samples:08x} peak={} active_voices={} understood={} ignored={}",
                    stats.peak, stats.active_voices, stats.understood_events, stats.ignored_events,
                );
            }
        }
        if let Some(coverage) = route_coverage.as_mut() {
            coverage.record(route_coverage_frame_from_game(frames, &game));
        }
        if asset_gpu_checkpoint_interval != 0
            && frames.is_multiple_of(asset_gpu_checkpoint_interval)
        {
            if let Some(dir) = asset_gpu_checkpoint_dir.as_deref() {
                write_asset_gpu_checkpoint_or_exit(&game, frames, dir);
            }
        }
        // Write --save-state-at checkpoints at the very END of the loop body so a
        // resumed run is byte-identical to a continuous one. (The retired classic
        // renderer used to re-project display state into RAM here; replay runs are
        // now render-free on every frame, so checkpoints stay consistent without it.)
        if let Some(idx) = save_state_at.iter().position(|(f, _)| *f == frames) {
            let (_, path) = &save_state_at[idx];
            write_checkpoint(&mut game, frames, path);
        }
    }

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
            game.wram_image()[0x10],
            game.wram_image()[0x11],
            game.ppu.bg_mode(),
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

    if let (Some(path), Some(coverage)) = (coverage_log.as_deref(), route_coverage.as_ref()) {
        write_route_coverage_log_or_exit(path, coverage, "coverage log");
    }

    if let Some(path) = save_state_path.as_deref() {
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

    let debirando_slot = (0..16).find(|&k| game.wram_image()[0x0e20 + k] == 0x63);
    let debirando_summary = debirando_slot.map_or_else(
        || "none".to_string(),
        |k| {
            format!(
                "k{} st={:02x} ai={:02x} gfx={:02x} head={} headst={:02x} delay={:02x} floor={:02x} xy={:04x}/{:04x} vel={:02x}/{:02x} ab={:02x}/{:02x}",
                k,
                game.wram_image()[0x0dd0 + k],
                game.wram_image()[0x0d80 + k],
                game.wram_image()[0x0dc0 + k],
                game.wram_image()[0x0eb0 + k],
                game.wram_image()[0x0dd0 + usize::from(game.wram_image()[0x0eb0 + k])],
                game.wram_image()[0x0df0 + k],
                game.wram_image()[0x0f20 + k],
                u16::from_le_bytes([game.wram_image()[0x0d10 + k], game.wram_image()[0x0d30 + k]]),
                u16::from_le_bytes([game.wram_image()[0x0d00 + k], game.wram_image()[0x0d20 + k]]),
                game.wram_image()[0x0d50 + k],
                game.wram_image()[0x0d40 + k],
                game.wram_image()[0x0da0 + k],
                game.wram_image()[0x0d90 + k],
            )
        },
    );
    let room_index2 = u16::from_le_bytes([game.wram_image()[0x48e], game.wram_image()[0x48f]]);
    let room_mask = replay_save_room_mask(&game, room_index2);
    let room_history = [
        u16::from_le_bytes([game.wram_image()[0xb80], game.wram_image()[0xb81]]),
        u16::from_le_bytes([game.wram_image()[0xb82], game.wram_image()[0xb83]]),
        u16::from_le_bytes([game.wram_image()[0xb84], game.wram_image()[0xb85]]),
        u16::from_le_bytes([game.wram_image()[0xb86], game.wram_image()[0xb87]]),
    ];
    let history_masks = room_history.map(|room| {
        if room == 0xffff {
            0
        } else {
            replay_save_room_mask(&game, room)
        }
    });
    let msg_read_pos = read_le_u16(&game.wram_image(), 0x1cd9) as usize;
    let msg_buffer_pos = 0x11200 + msg_read_pos;
    let msg_buffer_0 = game.wram_image().get(msg_buffer_pos).copied().unwrap_or(0);
    let msg_buffer_1 = game.wram_image().get(msg_buffer_pos + 1).copied().unwrap_or(0);
    let action_facing = usize::from((game.wram_image()[0x2f] >> 1) & 3);
    let tilemap_mask = read_le_u16(&game.wram_image(), 0x00ec);
    let action_y0 = read_le_u16(&game.wram_image(), 0x20).wrapping_add(ACTION_TILE_Y[action_facing] as u16)
        & tilemap_mask;
    let action_y1 = read_le_u16(&game.wram_image(), 0x20).wrapping_add(20) & tilemap_mask;
    let action_x0 = (read_le_u16(&game.wram_image(), 0x22)
        .wrapping_add(ACTION_TILE_X[action_facing] as u16)
        & tilemap_mask)
        >> 3;
    let action_x1 = (read_le_u16(&game.wram_image(), 0x22).wrapping_add(8) & tilemap_mask) >> 3;
    let action_attr0 = if game.wram_image()[0x1b] != 0 {
        0xff
    } else {
        game.overworld_tile_attribute_at_location(action_x0, action_y0)
    };
    let action_attr1 = if game.wram_image()[0x1b] != 0 {
        0xff
    } else {
        game.overworld_tile_attribute_at_location(action_x1, action_y1)
    };

    // Stable byte-level WRAM dump for deterministic old-vs-new diffing (no
    // bisection): `ZELDA3_REPLAY_WRAM_DUMP=<path>` writes the full 128KB WRAM.
    if let Some(path) = std::env::var_os("ZELDA3_REPLAY_WRAM_DUMP") {
        if let Err(e) = std::fs::write(&path, &game.wram_image()[..]) {
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
        u8::from(game.wram_image()[0x10] == 26),
        game.wram_image()[0x1a],
        game.wram_image()[0x0fa1],
        replay_checksum_ram_range(&game.wram_image(), 0, game.wram_image().len()),
        replay_checksum_ram_range(&game.wram_image(), 0x00000, 0x4000),
        replay_checksum_ram_range(&game.wram_image(), 0x04000, 0x4000),
        replay_checksum_ram_range(&game.wram_image(), 0x08000, 0x4000),
        replay_checksum_ram_range(&game.wram_image(), 0x0c000, 0x4000),
        replay_checksum_ram_range(&game.wram_image(), 0x10000, 0x4000),
        replay_checksum_ram_range(&game.wram_image(), 0x14000, 0x4000),
        replay_checksum_ram_range(&game.wram_image(), 0x18000, 0x4000),
        replay_checksum_ram_range(&game.wram_image(), 0x1c000, 0x4000),
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
        game.wram_image()[0x10],
        game.wram_image()[0x11],
        game.wram_image()[0xb0],
        u16::from_le_bytes([game.wram_image()[0xadc], game.wram_image()[0xadd]]),
        game.wram_image()[0x10c],
        game.wram_image()[0x200],
        game.wram_image()[0x1cd8],
        game.wram_image()[0x1b],
        u16::from_le_bytes([game.wram_image()[0xa0], game.wram_image()[0xa1]]),
        u16::from_le_bytes([game.wram_image()[0x8a], game.wram_image()[0x8b]]),
        u16::from_le_bytes([game.wram_image()[0x1cf0], game.wram_image()[0x1cf1]]),
        u16::from_le_bytes([game.wram_image()[0x1cd9], game.wram_image()[0x1cda]]),
        msg_buffer_0,
        msg_buffer_1,
        game.wram_image()[0x1cd4],
        u16::from_le_bytes([game.wram_image()[0x1ce0], game.wram_image()[0x1ce1]]),
        game.wram_image()[0x1ce9],
        game.wram_image()[0x2e4],
        u16::from_le_bytes([game.wram_image()[0x22], game.wram_image()[0x23]]),
        u16::from_le_bytes([game.wram_image()[0x20], game.wram_image()[0x21]]),
        game.wram_image()[0x2b],
        game.wram_image()[0x2a],
        game.wram_image()[0xf36d],
        game.wram_image()[0x0202],
        game.wram_image()[0x0303],
        game.wram_image()[0x0304],
        game.wram_image()[0x0301],
        game.wram_image()[0xf340],
        game.wram_image()[0xf341],
        u16::from_le_bytes([game.wram_image()[0xf366], game.wram_image()[0xf367]]),
        game.wram_image()[0xf36f],
        game.wram_image()[0xf3c9],
        game.wram_image()[0xf0],
        game.wram_image()[0xf2],
        game.wram_image()[0xf4],
        game.wram_image()[0xf6],
        game.wram_image()[0x67],
        game.wram_image()[0x2f],
        game.wram_image()[0x5d],
        game.wram_image()[0x345],
        game.wram_image()[0x4d],
        game.wram_image()[0x300],
        game.wram_image()[0x46],
        game.wram_image()[0x34a],
        game.wram_image()[0x2c6],
        game.wram_image()[0x48],
        game.wram_image()[0x376],
        game.wram_image()[0x3b],
        game.wram_image()[0x3a],
        game.wram_image()[0x30a],
        game.wram_image()[0x30b],
        game.wram_image()[0x30d],
        game.wram_image()[0xf3cc],
        read_le_u16(&game.wram_image(), 0x2cd),
        game.wram_image()[0x2d3],
        game.wram_image()[0x2cf],
        game.wram_image()[0x2d0],
        game.wram_image()[0x2f9],
        game.wram_image()[0x2f2],
        game.wram_image()[0x418],
        game.wram_image()[0x410],
        game.wram_image()[0x416],
        game.wram_image()[0x126],
        game.wram_image()[0x69a],
        game.wram_image()[0x69c],
        game.wram_image()[0x28],
        game.wram_image()[0x27],
        read_le_u16(&game.wram_image(), 0x00ea),
        game.wram_image()[0x30],
        game.wram_image()[0x6c],
        game.wram_image()[0x1cd5],
        game.wram_image()[0x1cd6],
        game.wram_image()[0x5e],
        game.wram_image()[0x57],
        game.wram_image()[0x0374],
        game.wram_image()[0x1cdf],
        game.wram_image()[0x66],
        game.wram_image()[0x26],
        game.wram_image()[0x6a],
        u16::from_le_bytes([game.wram_image()[0x49], game.wram_image()[0x4a]]),
        game.wram_image()[0xb7b],
        u16::from_le_bytes([game.wram_image()[0xb7c], game.wram_image()[0xb7d]]),
        u16::from_le_bytes([game.wram_image()[0xb7e], game.wram_image()[0xb7f]]),
        game.wram_image()[0xef],
        game.wram_image()[0x315],
        game.wram_image()[0x316],
        game.wram_image()[0x317],
        game.wram_image()[0x64a],
        u32::from_le_bytes([
            game.wram_image()[0x64c],
            game.wram_image()[0x64d],
            game.wram_image()[0x64e],
            game.wram_image()[0x64f],
        ]),
        game.wanted_zelda_features,
        game.wram_image()[0x24],
        game.wram_image()[0x29],
        game.wram_image()[0x2c7],
        game.wram_image()[0x114],
        u16::from_le_bytes([game.wram_image()[0x2ea], game.wram_image()[0x2eb]]),
        game.wram_image()[0x36c],
        game.wram_image()[0x368],
        game.wram_image()[0x369],
        game.wram_image()[0x36a],
        u16::from_le_bytes([game.wram_image()[0x366], game.wram_image()[0x367]]),
        action_attr0,
        action_attr1,
        read_le_u16(&game.wram_image(), 0x02e5),
        game.wram_image()[0x2e7],
        read_le_u16(&game.sram, 0x1ffe),
        read_le_u16(&game.sram, 0x03e5),
        u8::from(replay_sram_checksum_ok(&game.sram, 0)),
        read_le_u16(&game.sram, 0x12e5),
        u8::from(replay_sram_checksum_ok(&game.sram, 0x0f00)),
        game.wram_image()[0xc8],
        u16::from_le_bytes([game.wram_image()[0x00bf], game.wram_image()[0x00c0]]),
        u16::from_le_bytes([game.wram_image()[0x00c1], game.wram_image()[0x00c2]]),
        u16::from_le_bytes([game.wram_image()[0x00c3], game.wram_image()[0x00c4]]),
        game.wram_image()[0xb10],
        game.wram_image()[0xb12],
        game.wram_image()[0xb15],
        game.wram_image()[0xb13],
        game.wram_image()[0xb14],
        game.wram_image()[0x371],
        game.wram_image()[0x3c],
        u16::from_le_bytes([game.wram_image()[0xe], game.wram_image()[0xf]]),
        u16::from_le_bytes([game.wram_image()[0xc], game.wram_image()[0xd]]),
        u16::from_le_bytes([game.wram_image()[0x2f6], game.wram_image()[0x2f7]]),
        game.wram_image()[0x59],
        game.wram_image()[0x2e8],
        game.wram_image()[0x36d],
        game.wram_image()[0x58],
        u16::from_le_bytes([game.wram_image()[0x341], game.wram_image()[0x342]]),
        u16::from_le_bytes([game.wram_image()[0x343], game.wram_image()[0x344]]),
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
            game.wram_image()[0x0fa1], game.wram_image()[0x1a]
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

#[cfg(feature = "rom-timing")]
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

#[cfg(feature = "rom-timing")]
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

#[cfg(feature = "rom-timing")]
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

#[cfg(feature = "rom-timing")]
pub(crate) fn print_replay_save_panic_report(
    game: &ZeldaState,
    frames: u32,
    panic_info: &CapturedPanic,
) {
    let sr = &game.state_recorder;
    let packet = &game.wram_image()[0x1000..0x1010];
    let packet_dst = u16::from_le_bytes([game.wram_image()[0x1000], game.wram_image()[0x1001]]);
    let packet_vmain = game.wram_image()[0x1002];
    let packet_len = game.wram_image()[0x1003];
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
        game.wram_image()[0x18],
        game.wram_image()[0x17],
        packet_dst,
        packet_vmain,
        packet_len,
        packet,
        TraceState::from_ram(&game.wram_image(), sr.last_inputs, select_run_what(&game.wram_image())),
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
    pre_frame_game: Option<&ZeldaState>,
    host_frame: u32,
    input: u16,
    run_what: u8,
    crash_stage: &str,
    panic_info: Option<&CapturedPanic>,
) -> PathBuf {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let stem = format!("zelda3-rs-crash-{}-{seconds}", process::id());
    let report_path = env::temp_dir().join(format!("{stem}.txt"));
    let state_path = env::temp_dir().join(format!("{stem}.z3play"));
    // Without a snapshot, the state may be partially advanced by the failed
    // frame. It is useful diagnostics, but must never be offered for replay.
    let game = pre_frame_game.unwrap_or(game);
    let (diagnostic_state, checkpoint_state, checkpoint_path) = if pre_frame_game.is_some() {
        (
            "before_crashing_frame",
            "before_crashing_frame",
            state_path.display().to_string(),
        )
    } else {
        ("after_panic", "unavailable", "disabled".to_string())
    };
    let trace = TraceState::from_ram(&game.wram_image(), input, run_what);
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
         diagnostic_state={diagnostic_state}\n\
         checkpoint_state={checkpoint_state}\n\
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
        fnv1a64(&game.wram_image()),
        fnv1a64(&game.sram),
        game.ppu.bg_mode(),
        game.ppu.screen_enabled[0],
        game.ppu.screen_enabled[1],
        game.ppu.forced_blank,
        game.ppu.brightness,
        format_link_dma_trace(&game.wram_image()),
        checkpoint_path,
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
    if let Some(info) = panic_info {
        eprintln!("zelda3-rs panic: {}", info.message);
        eprintln!("zelda3-rs panic location: {}", info.location);
    }
    eprintln!("zelda3-rs crash report: {}", report_path.display());
    let Some(game) = pre_frame_game else {
        eprintln!("include the report text when reporting the crash");
        eprintln!("to capture a replay checkpoint, launch with ZELDA3_DEBUG_CRASH_SNAPSHOT=1 (requires parity-debug)");
        return report_path;
    };
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
    eprintln!("zelda3-rs crash checkpoint: {}", state_path.display());
    eprintln!(
        "replay with: cargo run -p zelda3-bin -- --replay-crash <rom.sfc> {}",
        state_path.display()
    );
    eprintln!("include the report text and checkpoint path when reporting the crash");
    report_path
}

pub(crate) fn load_play_state(rom_path: &str) -> ZeldaState {
    load_game_state(rom_path, true)
}

/// Load the one playable default from the embedded extracted asset pack.
pub(crate) fn load_default_play_state() -> ZeldaState {
    let mut game = ZeldaState::new();
    game.set_rom_startup_timing(true);
    apply_startup_audio_phase_override(&mut game);
    if let Err(e) = game.set_assets(EMBEDDED_ASSETS) {
        eprintln!("fatal: failed to load embedded extracted asset pack: {e}");
        process::exit(1);
    }
    configure_game_runtime_defaults(&mut game);
    game.zelda_read_sram();
    game
}

/// ROM-less play: the embedded asset pack with ROM startup timing off. The
/// cycle-exact timing plans execute the ROM, so without one the game runs on
/// its unmeasured schedules; a loaded ROM (`zelda3 <rom>`) keeps them on.
pub(crate) fn load_romless_play_state() -> ZeldaState {
    let mut game = ZeldaState::new();
    apply_startup_audio_phase_override(&mut game);
    if let Err(e) = game.set_assets(EMBEDDED_ASSETS) {
        eprintln!("fatal: failed to load embedded extracted asset pack: {e}");
        process::exit(1);
    }
    configure_game_runtime_defaults(&mut game);
    game.zelda_read_sram();
    game
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
            let game = checkpoint.game;
            #[cfg(feature = "rom-timing")]
            let game = {
                let mut game = game;
                game.restore_live_rom_timing_after_checkpoint();
                game
            };
            return (game, checkpoint.host_frame);
        }
        // Fall back to the replay-save state_recorder checkpoint format
        // (written by --replay-save --save-state and the replay-bisect cache).
        // These are a different on-disk format than the bincode play-crash
        // checkpoint above, so accept them here too for parity probes.
        let mut game = load_play_state(rom_path);
        match load_replay_save_checkpoint(&mut game, path) {
            Ok(()) => {
                #[cfg(feature = "rom-timing")]
                game.restore_live_rom_timing_after_checkpoint();
                let frame = game.state_recorder.replay_frame_counter;
                return (game, frame);
            }
            Err(state_recorder_err) => {
                eprintln!(
                    "failed to load checkpoint {} (not a play-crash or replay-save checkpoint): {state_recorder_err}",
                    path.display()
                );
                process::exit(1);
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
    let visible_ppu = game.with_display_snapshot(|display| format_render_ppu_summary(display));
    eprintln!(
        "replaying crash checkpoint {} from host_frame {}; trace={}; live_ppu={}; visible_ppu={}",
        crash_path.display(),
        checkpoint.host_frame,
        TraceState::from_ram(&game.wram_image(), checkpoint.input, checkpoint.run_what),
        format_render_ppu_summary(&game),
        visible_ppu,
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
            select_run_what(&game.wram_image())
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
                    &game,
                    Some(&pre_frame_game),
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
        fnv1a64(&game.wram_image())
    );
}

#[cfg(feature = "rom-timing")]
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
        .filter(|&(_, &value)| value != 0)
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
        .filter(|&(_, &value)| value & 0xff != 0)
        .take(16)
        .map(|(i, value)| format!("{i}:{value:04x}"))
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "smoke-render completed {frames} frame(s); nonzero_pixels={nonzero_pixels}; audio_nonzero={audio_nonzero}; audio_peak={audio_peak}; apui00={:02x}; music={:02x}/{:02x}/{:02x}; {}; frame={}; main={:02x}; sub={:02x}; subsub={:02x}; anim={:04x}; bugs={:02x}; nmi_thread={:02x}; stack={:04x}; forced_blank={}; brightness={}; mode={}; screen={:02x}/{:02x}; vram_nonzero={}; cgram_nonzero={}; cgram_indices=[{}]; oam_nonzero={}; obj_pixels=[{}]; first_cgram={:04x},{:04x},{:04x},{:04x}; obj_cgram={:04x},{:04x},{:04x},{:04x}; oam0={:04x},{:04x},{:04x},{:04x}",
        game.wram_image()[0x0648],
        game.wram_image()[0x012c],
        game.wram_image()[0x0130],
        game.wram_image()[0x0133],
        game.zelda_audio_debug_summary(),
        frames,
        game.wram_image()[0x10],
        game.wram_image()[0x11],
        game.wram_image()[0xb0],
        u16::from_le_bytes([game.wram_image()[0x0adc], game.wram_image()[0x0add]]),
        game.wram_image()[0x064a],
        game.wram_image()[0x012a],
        u16::from_le_bytes([game.wram_image()[0x1f0a], game.wram_image()[0x1f0b]]),
        game.ppu.forced_blank,
        game.ppu.brightness,
        game.ppu.bg_mode(),
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

#[cfg(feature = "rom-timing")]
fn load_modern_asset_gpu_readback_or_exit(context: &str) -> ModernAssetGpuReadbackRenderer {
    match ModernAssetGpuReadbackRenderer::load_from_env() {
        Ok(renderer) => renderer,
        Err(e) => {
            eprintln!("failed to initialize modern asset GPU readback for {context}: {e}");
            process::exit(1);
        }
    }
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

#[cfg(feature = "rom-timing")]
extern "C" fn libretro_environment(cmd: c_uint, data: *mut c_void) -> bool {
    const RETRO_ENVIRONMENT_SET_PIXEL_FORMAT: c_uint = 10;
    const RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY: c_uint = 9;
    const RETRO_ENVIRONMENT_GET_VARIABLE: c_uint = 15;
    const RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY: c_uint = 31;
    match cmd & !0x10000 {
        47 if crate::libretro_core::LIBRETRO_AUDIO_ONLY.load(Ordering::Relaxed) => {
            if data.is_null() { return false; }
            // Libretro GET_AUDIO_VIDEO_ENABLE: audio on, video off.
            unsafe { *(data as *mut c_uint) = 2; }
            true
        }
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

#[cfg(feature = "rom-timing")]
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
        b"snes9x_region" => c"auto".as_ptr(),
        b"snes9x_overscan" => c"enabled".as_ptr(),
        b"snes9x_hires_blend" => c"disabled".as_ptr(),
        b"snes9x_blargg" => c"disabled".as_ptr(),
        b"snes9x_audio_interpolation" => c"gaussian".as_ptr(),
        b"snes9x_gfx_clip" | b"snes9x_gfx_transp" => c"enabled".as_ptr(),
        _ => return false,
    };
    true
}

#[cfg(feature = "rom-timing")]
extern "C" fn libretro_video_refresh(
    data: *const c_void,
    width: c_uint,
    height: c_uint,
    pitch: usize,
) {
    if crate::libretro_core::LIBRETRO_AUDIO_ONLY.load(Ordering::Relaxed) {
        return;
    }
    if !LIBRETRO_CAPTURE_ENABLED.load(Ordering::Relaxed) {
        return;
    }
    if let Some(capture) = LIBRETRO_CAPTURE.get() {
        if let Ok(mut capture) = capture.lock() {
            capture.video_width = width;
            capture.video_height = height;
            capture.video_pitch = pitch;
            if !data.is_null() {
                capture.video.clear();
                let byte_len = pitch.saturating_mul(height as usize);
                let bytes = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), byte_len) };
                capture.video.extend_from_slice(bytes);
            }
        }
    }
}

#[cfg(feature = "rom-timing")]
extern "C" fn libretro_audio_sample(left: i16, right: i16) {
    if !LIBRETRO_CAPTURE_ENABLED.load(Ordering::Relaxed) {
        return;
    }
    if let Some(capture) = LIBRETRO_CAPTURE.get() {
        if let Ok(mut capture) = capture.lock() {
            capture.audio.push(left);
            capture.audio.push(right);
        }
    }
}

#[cfg(feature = "rom-timing")]
extern "C" fn libretro_audio_sample_batch(data: *const i16, frames: usize) -> usize {
    if !LIBRETRO_CAPTURE_ENABLED.load(Ordering::Relaxed) {
        return frames;
    }
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

#[cfg(feature = "rom-timing")]
extern "C" fn libretro_input_poll() {}

#[cfg(feature = "rom-timing")]
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

#[cfg(feature = "rom-timing")]
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
    menu_state: u8,
    bg3_v_scroll_copy2: u16,
    nmi_subroutine_index: u8,
    nmi_load_target_address: u16,
    nmi_core_update_disable: u8,
    animated_link_tile_dma_source: u16,
    link_tile_animation_countdown: u16,
    vram_upload_tilemap_hash: u64,
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
            menu_state: ram[0x0200],
            bg3_v_scroll_copy2: u16::from_le_bytes([ram[0x00ea], ram[0x00eb]]),
            nmi_subroutine_index: ram[0x0017],
            nmi_load_target_address: u16::from_le_bytes([ram[0x0116], ram[0x0117]]),
            // Exact state consumed by the decomp's NMI_DoUpdates animated
            // Link-tile upload at VRAM word 0x40b0.
            nmi_core_update_disable: ram[0x0710],
            animated_link_tile_dma_source: u16::from_le_bytes([ram[0x0ae0], ram[0x0ae1]]),
            link_tile_animation_countdown: u16::from_le_bytes([ram[0xc013], ram[0xc014]]),
            vram_upload_tilemap_hash: fnv1a64(&ram[0x1000..0x1800]),
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
            "input=${:04X} run={} main={} sub={} subsub={} joyH=${:02X} joyL=${:02X} filtH=${:02X} filtL=${:02X} inidisp=${:02X} music=ctrl:{:02X}/amb:{:02X}/sfx:{:02X},{:02X}/unk:{:02X}/amb_last:{:02X}/queued:{:02X}/last:{:02X} attract_state={} attract_sequence={} room=${:04X} load_ptr=${:04X} line_ptr=${:04X} layout=${:04X} msg={} text_state={} menu=(state={} bg3v=${:04X} nmi={:02X} target=${:04X} core_disable={:02X} link_tile=(src=${:04X} countdown=${:04X}) upload={:016X}) read_pos=${:04X} wait2={} text_byte=${:02X} select=(r16={} grid_col={} name_col={} row={} scroll=${:04X} y={} v9={} v10={} v11={}) link=(${:04X},{:04X},z={:04X}/vel={:02X},{:02X}/dir={:02X}/last={:02X}/face={:02X}/sub={:02X}/vz={:02X}/h={}/aux={}/timer={}/state={:02X}) action=(tile={}/abit={:02X}/bframes={}/bmask={:02X}/ability={:02X}/lift={},{} pickup=s{:02X}/c{:02X}/a{:02X}/hand={:02X}/pos={:02X}/throw={:02X}/ptimer={}) coll=(r12={:04X}/r14={:04X}/read={:04X}/chest={:04X}/misc={:04X}/var1={:04X}/diag={:04X}/mad={:02X}/dead={:02X}/mask={:02X},{:02X}/orth={}/tile={:02X}/door={}/safe={:04X},{:04X}) bg2=({:04X},{:04X}) pose={} sort={} sort_oam=${:04X} oam=(${:04X},${:04X}) dma=[{:02X},{:02X},{:02X},{:02X},{:02X},{:02X}] oam0={:02X?} spr_t={:02X?} spr_st={:02X?} spr_oam={:02X?} spr_gfx={:02X?} uncle=(ai={} d={} gfx={} xy={:04X},{:04X}) sprite0=(type={:02X} st={:02X} room={:02X} flags={:02X}/{:02X}/{:02X}/{:02X}/{:02X} defl={:02X} hp={:02X} oam={:02X} ai={:02X} d={:02X} gfx={:02X} xy={:04X},{:04X})",
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
            self.menu_state,
            self.bg3_v_scroll_copy2,
            self.nmi_subroutine_index,
            self.nmi_load_target_address,
            self.nmi_core_update_disable,
            self.animated_link_tile_dma_source,
            self.link_tile_animation_countdown,
            self.vram_upload_tilemap_hash,
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

#[cfg(feature = "rom-timing")]
fn format_optional_u8(value: Option<u8>) -> String {
    value
        .map(|value| format!("{value:02x}"))
        .unwrap_or_else(|| "none".to_string())
}

#[cfg(feature = "rom-timing")]
fn format_optional_u64(value: Option<u64>) -> String {
    value
        .map(|value| format!("{value:016x}"))
        .unwrap_or_else(|| "none".to_string())
}

fn find_asset_pack(rom_path: &str) -> Option<PathBuf> {
    find_asset_pack_with_override(
        rom_path,
        std::env::var_os("ZELDA3_ASSET_PACK").map(PathBuf::from),
    )
}

fn find_asset_pack_with_override(
    rom_path: &str,
    explicit_asset: Option<PathBuf>,
) -> Option<PathBuf> {
    if explicit_asset.is_some() {
        return explicit_asset;
    }

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
    fn checkpoint_rejects_pre_ownership_resource_mirrors() {
        let path = std::env::temp_dir().join(format!(
            "zelda3-pre-ownership-checkpoint-{}", std::process::id()));
        fs::write(&path, b"Z3RSPC22").unwrap();
        let error = match load_play_crash_checkpoint(&path) {
            Ok(_) => panic!("pre-ownership checkpoint was accepted"),
            Err(error) => error.to_string(),
        };
        let _ = fs::remove_file(path);
        assert!(error.contains("re-create it with the current binary"), "{error}");
    }

    #[test]
    fn play_crash_report_only_writes_replay_checkpoint_with_pre_frame_snapshot() {
        let mut game = ZeldaState::new();
        game.frame_ctr_dbg = 41;
        let pre_frame_game = game.clone();
        game.frame_ctr_dbg = 42;
        game.ram[0x20] = 7;
        let panic_info = CapturedPanic {
            message: "test frame panic".to_string(),
            location: "test.rs:1:1".to_string(),
            backtrace: "test backtrace".to_string(),
        };

        for snapshot in [None, Some(&pre_frame_game)] {
            let report_path = write_play_crash_report(
                &game,
                snapshot,
                41,
                0x80,
                1,
                "run_frame",
                Some(&panic_info),
            );
            let state_path = report_path.with_extension("z3play");
            let report = fs::read_to_string(&report_path).unwrap();
            assert!(report.contains("host_frame=41\n"));
            assert!(report.contains("crash_stage=run_frame\n"));
            assert!(report.contains("panic=test frame panic\n"));
            assert!(report.contains("panic_location=test.rs:1:1\n"));
            assert!(report.contains("test backtrace"));
            assert!(report.contains(&format!(
                "trace={}",
                TraceState::from_ram(&snapshot.unwrap_or(&game).wram_image(), 0x80, 1)
            )));
            if snapshot.is_some() {
                assert!(report.contains("checkpoint_state=before_crashing_frame\n"));
                assert!(report.contains("diagnostic_state=before_crashing_frame\n"));
                let checkpoint = load_play_crash_checkpoint(&state_path).unwrap();
                assert_eq!(checkpoint.host_frame, 41);
                assert_eq!(checkpoint.input, 0x80);
                assert_eq!(checkpoint.run_what, 1);
                assert_eq!(checkpoint.game.frame_ctr_dbg, 41);
                assert_eq!(checkpoint.game.wram_image(), pre_frame_game.wram_image());
                fs::remove_file(state_path).unwrap();
            } else {
                assert!(report.contains("diagnostic_state=after_panic\n"));
                assert!(report.contains("checkpoint_state=unavailable\n"));
                assert!(report.contains("checkpoint=disabled\n"));
                assert!(report.contains("frame_ctr_dbg=42\n"));
                assert!(!state_path.exists());
            }
            fs::remove_file(report_path).unwrap();
        }
    }

    #[test]
    #[cfg(feature = "rom-timing")]
    fn mistyped_rom_first_oracle_flag_cannot_fall_through_to_play_mode() {
        let args = vec![
            "zelda3".to_string(),
            "saves/zelda3.sfc".to_string(),
            "--snes9x-core".to_string(),
            "oracle.dylib".to_string(),
        ];
        assert!(rom_first_oracle_flag_error(&args).is_some());

        let valid = vec![
            "zelda3".to_string(),
            "saves/zelda3.sfc".to_string(),
            "--compare-snes9x-oracle".to_string(),
            "oracle.dylib".to_string(),
        ];
        assert!(rom_first_oracle_flag_error(&valid).is_none());
    }

    #[test]
    #[cfg(feature = "rom-timing")]
    fn recorder_telemetry_captures_progress_and_ending_markers() {
        let mut ram = vec![0u8; 0x20000];
        ram[0x10] = 0x1a;
        ram[0x11] = 0x26;
        ram[0x20..0x22].copy_from_slice(&0x1234u16.to_le_bytes());
        ram[0x22..0x24].copy_from_slice(&0x5678u16.to_le_bytes());
        ram[0xf366..0xf368].copy_from_slice(&0x77fcu16.to_le_bytes());
        ram[0xf36d] = 0x50;
        ram[0x202] = 1;

        let telemetry = recorder_telemetry(&ram);

        assert_eq!(telemetry["main"], 0x1a);
        assert_eq!(telemetry["sub"], 0x26);
        assert_eq!(telemetry["x"], 0x5678);
        assert_eq!(telemetry["y"], 0x1234);
        assert_eq!(telemetry["progression_flags"], 0x77fc);
        assert_eq!(telemetry["health"], 0x50);
        assert_eq!(telemetry["equipped_item"], 1);
        assert_eq!(telemetry["ending"], true);
        assert_eq!(telemetry["final_credits"], true);
    }

    #[test]
    #[cfg(feature = "rom-timing")]
    fn recorder_audio_resampling_preserves_stereo_channels() {
        let input = [10, -10, 20, -20];
        assert_eq!(
            resample_stereo_frame(&input, 4, 2),
            vec![10, -10, 10, -10, 20, -20, 20, -20]
        );
        assert_eq!(resample_stereo_frame(&input, 2, 1), vec![10, 20]);
    }

    #[test]
    fn explicit_asset_pack_wins_over_rom_neighbor_discovery() {
        let explicit = PathBuf::from("current-modern-assets.dat");
        assert_eq!(
            find_asset_pack_with_override("oracle/zelda3.sfc", Some(explicit.clone())),
            Some(explicit)
        );
    }

    #[test]
    #[cfg(feature = "rom-timing")]
    fn replay_hash_precondition_rejects_a_replaced_oracle() {
        let path = std::env::temp_dir().join(format!(
            "zelda3-oracle-hash-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        fs::write(&path, b"oracle-v1").unwrap();
        let expected = parity::hash::sha256_file(&path).unwrap();
        assert!(expected_sha256_matches(&path, "core", Some(&expected)).is_ok());

        fs::write(&path, b"oracle-v2").unwrap();
        let error = expected_sha256_matches(&path, "core", Some(&expected)).unwrap_err();
        assert!(error.contains("hash mismatch for replay"));
        let _ = fs::remove_file(path);
    }

    #[test]
    #[cfg(feature = "rom-timing")]
    fn libretro_frame_window_requires_at_least_one_replayed_frame() {
        assert!(validate_libretro_frame_window(0, 0).is_err());
        assert!(validate_libretro_frame_window(10, 10).is_err());
        assert!(validate_libretro_frame_window(10, 0).is_ok());
        assert!(validate_libretro_frame_window(10, 9).is_ok());
    }

    #[test]
    #[cfg(feature = "rom-timing")]
    fn snes9x_oracle_requires_version_1_63() {
        assert!(validate_required_libretro_core(
            Some(("Snes9x", "1.63")),
            "Snes9x",
            "1.63 185488c",
        )
        .is_ok());
        assert!(
            validate_required_libretro_core(Some(("Snes9x", "1.63")), "Snes9x", "1.62",).is_err()
        );
        assert!(
            validate_required_libretro_core(Some(("Snes9x", "1.63")), "Other Core", "1.63",)
                .is_err()
        );
    }

    #[test]
    #[cfg(feature = "rom-timing")]
    fn libretro_null_video_callback_repeats_the_previous_frame() {
        let capture = LIBRETRO_CAPTURE.get_or_init(|| Mutex::new(LibretroCapture::default()));
        *capture.lock().unwrap() = LibretroCapture::default();
        let pixels = [0x12u8, 0x34, 0x56, 0x78];
        libretro_video_refresh(pixels.as_ptr().cast(), 2, 1, pixels.len());
        libretro_video_refresh(std::ptr::null(), 2, 1, pixels.len());

        assert_eq!(capture.lock().unwrap().video, pixels);
    }

    #[test]
    #[cfg(feature = "rom-timing")]
    fn gpu_readback_pixels_remain_rgba_during_oracle_comparison() {
        assert_eq!(
            rgba_pixel_at(&[0x12, 0x34, 0x56, 0x78], 0),
            Some([0x12, 0x34, 0x56, 0x78])
        );
    }

    #[test]
    #[cfg(feature = "rom-timing")]
    fn snes9x_rgb565_green_is_normalized_to_snes_five_bit_color() {
        let raw = (27u16 << 11) | (55u16 << 5) | 27u16;
        let frame = LibretroFrame {
            audio: Vec::new(),
            video: raw.to_le_bytes().to_vec(),
            video_width: 1,
            video_height: 1,
            video_pitch: 2,
            pixel_format: 2,
        };

        assert_eq!(snes9x_rgba_pixel_at(&frame, 0), Some([222, 222, 222, 0xff]));
    }

    #[test]
    #[cfg(feature = "rom-timing")]
    fn libretro_engine_receipt_exposes_intro_and_audio_timing_state() {
        let mut ram = vec![0; 0x20000];
        ram[0x10] = 0x01;
        ram[0x13] = 0x0f;
        ram[0x22] = 0x04;
        ram[0x23] = 0x02;
        ram[0x12c] = 0x02;
        ram[0x12d] = 0x03;
        ram[0x12e] = 0x04;
        ram[0x12f] = 0x05;
        ram[0x132] = 0x06;
        ram[0x22..0x24].copy_from_slice(&0x0204u16.to_le_bytes());
        ram[0x20..0x22].copy_from_slice(&0x5678u16.to_le_bytes());
        ram[0x301] = 0x07;
        ram[0x308] = 0x08;
        ram[0x309] = 0x09;
        ram[0x0fb2] = 0x0a;
        ram[0xf0] = 0x10;
        ram[0xf2] = 0x11;
        ram[0xf4] = 0x12;
        ram[0xf6] = 0x13;
        ram[0x67] = 0x0b;
        ram[0x2f] = 0x0c;
        ram[0x5c] = 0x21;
        ram[0x36c] = 0x0d;
        ram[0x368] = 0x0e;
        ram[0x36a] = 0x0f;
        ram[0x1e00] = 0x03;
        ram[0x0ff9] = 0x20;
        ram[0x00ca] = 0x06;
        ram[0x00cb] = 0x05;
        ram[0x00cc] = 0x04;
        ram[0xc007] = 0x1f;
        ram[0x1f02] = 0x71;
        ram[0x1f0a..0x1f0c].copy_from_slice(&0x1f31u16.to_le_bytes());
        ram[0x1f0c] = 0xff;
        ram[0x1cd4] = 4;
        ram[0x1cd8] = 1;
        ram[0x1cd9..0x1cdb].copy_from_slice(&0x0052u16.to_le_bytes());
        ram[0x1cf0..0x1cf2].copy_from_slice(&0x007bu16.to_le_bytes());
        ram[0xe800] = 1;
        ram[0xefff] = 2;

        let receipt = libretro_engine_state_receipt(&ram);

        assert_eq!(receipt["main_module"], 1);
        assert_eq!(receipt["screen_brightness"], 0x0f);
        assert_eq!(receipt["attract_state"], 0x04);
        assert_eq!(receipt["attract_sequence"], 0x02);
        assert_eq!(receipt["music_control"], 2);
        assert_eq!(receipt["ambient_sound_effect"], 3);
        assert_eq!(receipt["sound_effect_1"], 4);
        assert_eq!(receipt["sound_effect_2"], 5);
        assert_eq!(receipt["queued_music_control"], 6);
        assert_eq!(receipt["link_x"], 0x0204);
        assert_eq!(receipt["link_y"], 0x5678);
        assert_eq!(receipt["link_item_in_hand"], 7);
        assert_eq!(receipt["link_state_bits"], 8);
        assert_eq!(receipt["link_picking_throw_state"], 9);
        assert_eq!(receipt["sprite_pickup_slot"], 10);
        assert_eq!(receipt["joypad_high"], 16);
        assert_eq!(receipt["joypad_low"], 17);
        assert_eq!(receipt["joypad_high_filtered"], 18);
        assert_eq!(receipt["joypad_low_filtered"], 19);
        assert_eq!(receipt["link_direction"], 11);
        assert_eq!(receipt["link_facing_direction"], 12);
        assert_eq!(receipt["link_sprite_oam_state_timer"], 0x21);
        assert!(receipt["ppu_oam_dma_shadow_hash"].is_number());
        assert_eq!(receipt["link_tile_action"], 13);
        assert_eq!(receipt["link_lift_x_low"], 14);
        assert_eq!(receipt["link_lift_x_high"], 15);
        assert_eq!(receipt["intro_step_index"], 3);
        assert_eq!(receipt["intro_palette_flash_count"], 0x20);
        assert_eq!(receipt["intro_sword_sparkle_timer"], 0x06);
        assert_eq!(receipt["intro_sword_sparkle_step"], 0x05);
        assert_eq!(receipt["intro_sword_animation_step"], 0x04);
        assert_eq!(receipt["palette_filter_countdown"], 0x1f);
        assert_eq!(receipt["poly_config1"], 0x71);
        assert_eq!(receipt["nmi_thread_stack"], 0x1f31);
        assert_eq!(receipt["pending_polyhedral_update"], 0xff);
        assert_eq!(receipt["poly_buffer_nonzero_bytes"], 2);
        assert_eq!(receipt["dialogue_message_index"], 0x007b);
        assert_eq!(receipt["messaging_module"], 1);
        assert_eq!(receipt["text_render_state"], 4);
        assert_eq!(receipt["dialogue_msg_read_pos"], 0x0052);
    }

    #[test]
    #[cfg(feature = "rom-timing")]
    fn snes9x_dsp_trace_music_state_comes_from_oracle_wram() {
        let mut ram = vec![0; 0x20000];
        ram[0x012c] = 0x11;
        ram[0x0132] = 0x22;
        ram[0x0133] = 0x33;

        assert_eq!(oracle_music_route_state(&ram), Some([0x11, 0x22, 0x33]));
        assert_eq!(oracle_music_route_state(&ram[..0x0133]), None);
    }

    #[test]
    fn route_coverage_frame_reads_route_surface_ids_from_ram() {
        let mut game = ZeldaState::default();
        let mut image = game.wram_image().into_owned();
        image[TRACE_MAIN_MODULE_INDEX] = 0x07;
        image[TRACE_SUBMODULE_INDEX] = 0x02;
        image[TRACE_SUBSUBMODULE_INDEX] = 0x03;
        image[0x1b] = 1;
        image[0x48e] = 0xa4;
        image[0x48f] = 0x00;
        image[0x8a] = 0x40;
        image[0x8b] = 0x00;
        image[0x0dd0] = 9;
        image[0x0e20] = 0xcb;
        image[0x0dd1] = 0;
        image[0x0e21] = 0xcc;
        image[0x0c4a] = 0x05;
        image[0x0c5e] = 0x12;
        image[0x0202] = 0x11;

        game.import_wram_image(&image);

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

pub(crate) fn load_play_crash_checkpoint(
    path: &Path,
) -> Result<PlayCrashCheckpoint, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    if bytes.len() < PLAY_CRASH_CHECKPOINT_MAGIC.len() {
        return Err("not a zelda3-rs playable crash checkpoint".into());
    }
    // bincode 1 encodes the leading `[u8; 8]` field as eight raw bytes with no
    // length prefix, so the layout version is readable before the full decode.
    let magic = &bytes[..PLAY_CRASH_CHECKPOINT_MAGIC.len()];
    if magic != PLAY_CRASH_CHECKPOINT_MAGIC {
        return Err(format!(
            "checkpoint layout {:?} does not match this binary's {:?}; re-create it with the current binary",
            String::from_utf8_lossy(magic),
            String::from_utf8_lossy(PLAY_CRASH_CHECKPOINT_MAGIC)
        )
        .into());
    }
    let checkpoint: PlayCrashCheckpoint = bincode::deserialize(&bytes)?;
    Ok(checkpoint)
}
