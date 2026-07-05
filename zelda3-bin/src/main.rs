//! zelda3-rs prototype binary.
//!
//! Default execution runs the native playable host: load ROM/assets/SRAM, step
//! `ZeldaState`, present PPU pixels, queue audio, and save SRAM on quit.
//! `--lockstep` keeps the C-oracle comparison path available for parity work,
//! while `--headless` preserves the raw opcode-budget emulator harness.

mod developer_destinations;
mod developer_modern_map;
mod gpu_capture;
mod play_renderer;

use std::backtrace::Backtrace;
use std::collections::HashMap;
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

use gpu_capture::{
    capture_gpu_frame_from_game, gpu_render_compare_run, modern_compare_mode_defaults_from_env,
    modern_index_compare_run_from_env, play_gpu_render_compare_session,
    render_hd_capture_from_game, render_live_game_gpu_frame_rgba,
    replay_optional_gpu_readback_renderer,
};
use platform::{
    DeveloperCurrentLocation, DeveloperThumbnail, Frontend, HostMenuAction, HostMenuInput,
    HostMenuMode, HostMenuState, NativeFrontend, NativeFrontendOptions,
};
use play_renderer::{
    render_fingerprint_leaf_bgra, render_hash_frame_bgra_line, render_play_frame_bgra,
    render_standard_play_frame_bgra, run_play_frame_bgra, run_play_frame_with_run_what_bgra,
};
use serde::{Deserialize, Serialize};
use snes::{consts::PPU_EXTRA_LEFT_RIGHT, cpu_run_opcode, load_rom, ppu::PpuRenderFlags, Snes};
use zelda3::{
    config::parse_config_file_context, LockstepOracle, OracleError, ZeldaState, RUN_MAIN, RUN_POLY,
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
const TRACE_MAIN_MODULE_INDEX: usize = 0x10;
const TRACE_SUBMODULE_INDEX: usize = 0x11;
const TRACE_SUBSUBMODULE_INDEX: usize = 0xb0;
const TRACE_JOYPAD1H_LAST: usize = 0x0f0;
const TRACE_JOYPAD1L_LAST: usize = 0x0f2;
const TRACE_FILTERED_JOYPAD_H: usize = 0x0f4;
const TRACE_FILTERED_JOYPAD_L: usize = 0x0f6;
const TRACE_SELECTFILE_VAR3: usize = 0x0b10;
const TRACE_SELECTFILE_VAR7: usize = 0x0b11;
const TRACE_SELECTFILE_VAR9: usize = 0x0b13;
const TRACE_SELECTFILE_VAR11: usize = 0x0b14;
const TRACE_SELECTFILE_VAR5: usize = 0x0b15;
const TRACE_SELECTFILE_VAR10: usize = 0x0b16;
const TRACE_SELECTFILE_ARR2_1: usize = 0x0cb;
const PLAYER_IS_INDOORS: usize = 0x001b;
const TM_COPY: usize = 0x001c;
const TS_COPY: usize = 0x001d;
const BGMODE_COPY: usize = 0x0094;
const FLAG_UPDATE_CGRAM_IN_NMI: usize = 0x0015;
#[cfg(test)]
const MUSIC_CONTROL: usize = 0x012c;
#[cfg(test)]
const CURRENT_MUSIC_CONTROL: usize = 0x0130;
#[cfg(test)]
const LAST_MUSIC_CONTROL: usize = 0x0133;
const MAIN_PALETTE_BUFFER: usize = 0x0c500;
const DEVELOPER_ROOM_BG_TILE_BASE: u16 = 0x2000;
const DEVELOPER_ROOM_SOURCE_BG_LAYER: usize = 1;
const OVERWORLD_BG_CHR_BASE: usize = 0x2000;
// Dungeon BG CHR base (VRAM word address), pinned by the Task 1 spike. The dungeon
// blockset loads the same way the overworld does: `InitializeTilesets` writes the
// main blockset's first BG graphics pack to VRAM word 0x2000 via
// `load_background_graphics(0x2000, main_tile_set[0], ..)` (crates/zelda3/src/load_gfx.rs).
// Tile numbers (tilemap_entry & 0x3ff) index 16-word tiles from this base, so the
// main blockset spans words 0x2000..0x4000 — identical to OVERWORLD_BG_CHR_BASE.
// (allow(dead_code): consumed by the Task 2 --dump-dungeon-index-tiles command.)
#[allow(dead_code)]
const DUNGEON_BG_CHR_BASE: usize = 0x2000;
// BG1 dungeon tilemap is 64x64 tiles8 => 0x1000 word entries (matches the overworld
// 64x64 walk). Word index 0..0x1000 into game_state.dungeon.room_tilemaps BG1.
#[allow(dead_code)]
const DUNGEON_BG1_TILEMAP_WORDS: usize = 0x1000;
const UNIQUE_OVERWORLD_MANIFEST_SOURCE_LIMIT: usize = 32;
const DEV_TOWN_ROOF: u16 = 224;
const DEV_TOWN_WALL: u16 = 225;
const DEV_TOWN_DOOR: u16 = 226;
const DEV_TOWN_GRASS: u16 = 227;
const DEV_TOWN_PATH: u16 = 228;
const DEV_TOWN_FENCE: u16 = 229;
const DEV_TOWN_SHRUB: u16 = 230;
const DEV_TOWN_SIGN: u16 = 231;
const DEV_TOWN_TREE: u16 = 232;
const DEV_TOWN_CLIFF_TOP: u16 = 233;
const DEV_TOWN_CLIFF_FACE: u16 = 234;
const DEV_TOWN_FLOWERS: u16 = 235;
const DEV_TOWN_STONE: u16 = 236;
const DEV_TOWN_HEDGE: u16 = 237;
#[cfg(test)]
const DEVELOPER_ROOM_KAKARIKO_MUSIC: u8 = 0x07;
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
        run_compare_startup_apu_impls(&args[2..]);
        return;
    }
    if args.get(1).map(String::as_str) == Some("--trace-song-bank") {
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

fn read_le_u16(bytes: &[u8], index: usize) -> u16 {
    u16::from_le_bytes([bytes[index], bytes[index + 1]])
}

fn write_le_u16(bytes: &mut [u8], index: usize, value: u16) {
    let [lo, hi] = value.to_le_bytes();
    bytes[index] = lo;
    bytes[index + 1] = hi;
}

fn replay_sram_checksum_ok(bytes: &[u8], base: usize) -> bool {
    let mut sum = 0u16;
    for i in 0..0x280 {
        sum = sum.wrapping_add(read_le_u16(bytes, base + i * 2));
    }
    sum == 0x5a5a
}

fn replay_checksum_bytes(bytes: &[u8]) -> u32 {
    let mut hash = 2166136261u32;
    for &byte in bytes {
        hash = (hash ^ u32::from(byte)).wrapping_mul(16777619);
    }
    hash
}

fn replay_checksum_samples(samples: &[i16]) -> u32 {
    let mut hash = 2166136261u32;
    for sample in samples {
        for byte in sample.to_le_bytes() {
            hash = (hash ^ u32::from(byte)).wrapping_mul(16777619);
        }
    }
    hash
}

fn should_write_fingerprint(fingerprint_frame: Option<u32>, frame: u32) -> bool {
    fingerprint_frame.is_none_or(|target| frame == target)
}

fn route_coverage_frame_from_game(
    frame: u32,
    game: &ZeldaState,
) -> parity::coverage::CoverageFrame {
    let indoors = game.ram[0x1b] != 0;
    let sprite_types = (0..16)
        .filter(|&k| game.ram[0x0dd0 + k] != 0)
        .map(|k| game.ram[0x0e20 + k])
        .collect();
    let ancilla_types = (0..10)
        .map(|k| game.ram[0x0c4a + k])
        .filter(|&ty| ty != 0)
        .collect();
    parity::coverage::CoverageFrame {
        frame,
        main_module: game.ram[TRACE_MAIN_MODULE_INDEX],
        submodule: game.ram[TRACE_SUBMODULE_INDEX],
        subsubmodule: game.ram[TRACE_SUBSUBMODULE_INDEX],
        indoor_room: indoors.then(|| read_le_u16(&game.ram, 0x48e)),
        overworld_screen: (!indoors).then(|| read_le_u16(&game.ram, 0x8a)),
        sprite_types,
        ancilla_types,
        active_item: (game.ram[0x0202] != 0).then_some(game.ram[0x0202]),
    }
}

fn current_developer_location_from_ram(ram: &[u8], host_frame: u32) -> DeveloperCurrentLocation {
    let indoors = ram.get(PLAYER_IS_INDOORS).copied().unwrap_or(0) != 0;
    let indoor_room = indoors.then(|| read_le_u16(ram, 0x48e));
    let location = if indoors {
        format!("ROOM {:04X}", indoor_room.unwrap())
    } else {
        format!("OW {:04X}", read_le_u16(ram, 0x8a))
    };
    DeveloperCurrentLocation {
        label: "CURRENT LOC".to_string(),
        location,
        detail: format!("FRAME {host_frame}"),
        thumbnail: if indoor_room == Some(0x01ff) {
            DeveloperThumbnail::DevRoom
        } else if indoors {
            DeveloperThumbnail::Sanctuary
        } else {
            DeveloperThumbnail::LockedOverworld
        },
    }
}

fn write_route_coverage_log_or_exit(
    path: &Path,
    coverage: &parity::coverage::RouteCoverage,
    label: &str,
) {
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!(
                "failed to create {label} directory {}: {e}",
                parent.display()
            );
            process::exit(1);
        }
    }
    let json = serde_json::to_vec_pretty(coverage).unwrap_or_else(|e| {
        eprintln!("failed to encode {label}: {e}");
        process::exit(1);
    });
    if let Err(e) = std::fs::write(path, json) {
        eprintln!("failed to write {label} {}: {e}", path.display());
        process::exit(1);
    }
}

fn run_coverage_probe(args: &[String]) {
    let Some(rom_path) = args.first() else {
        eprintln!(
            "usage: zelda3 --coverage-probe <path-to-rom.sfc> --coverage-log <path> [--direct-entrance <index>]..."
        );
        process::exit(2);
    };
    let mut coverage_log = None::<PathBuf>;
    let mut direct_entrances = Vec::<u16>::new();
    let mut dungeon_rooms = Vec::<u16>::new();
    let mut overworld_screens = Vec::<u16>::new();
    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--coverage-log" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--coverage-log requires a path");
                    process::exit(2);
                };
                coverage_log = Some(PathBuf::from(path));
                i += 2;
            }
            "--direct-entrance" => {
                let Some(index) = args.get(i + 1) else {
                    eprintln!("--direct-entrance requires an index");
                    process::exit(2);
                };
                let entrance = parse_u16_auto(index).unwrap_or_else(|| {
                    eprintln!("invalid --direct-entrance index: {index}");
                    process::exit(2);
                });
                direct_entrances.push(entrance);
                i += 2;
            }
            "--dungeon-room" => {
                let Some(index) = args.get(i + 1) else {
                    eprintln!("--dungeon-room requires an index");
                    process::exit(2);
                };
                let room = parse_u16_auto(index).unwrap_or_else(|| {
                    eprintln!("invalid --dungeon-room index: {index}");
                    process::exit(2);
                });
                dungeon_rooms.push(room);
                i += 2;
            }
            "--overworld-screen" => {
                let Some(index) = args.get(i + 1) else {
                    eprintln!("--overworld-screen requires an index");
                    process::exit(2);
                };
                let screen = parse_u16_auto(index).unwrap_or_else(|| {
                    eprintln!("invalid --overworld-screen index: {index}");
                    process::exit(2);
                });
                overworld_screens.push(screen);
                i += 2;
            }
            flag => {
                eprintln!("unknown --coverage-probe option: {flag}");
                process::exit(2);
            }
        }
    }
    let Some(coverage_log) = coverage_log else {
        eprintln!("--coverage-log is required");
        process::exit(2);
    };

    let base = load_translated_replay_state(rom_path);
    let mut coverage = parity::coverage::RouteCoverage::default();
    for (index, entrance) in direct_entrances.iter().copied().enumerate() {
        let mut game = base.clone();
        let room = game.parity_probe_direct_entrance(entrance);
        coverage.record(route_coverage_frame_from_game(index as u32 + 1, &game));
        println!("coverage-probe direct-entrance entrance=0x{entrance:04x} room=0x{room:04x}");
    }
    let dungeon_frame_base = direct_entrances.len() as u32 + 1;
    for (index, room) in dungeon_rooms.iter().copied().enumerate() {
        let mut game = base.clone();
        let loaded_room = game.parity_probe_dungeon_room(room);
        coverage.record(route_coverage_frame_from_game(
            dungeon_frame_base + index as u32,
            &game,
        ));
        println!("coverage-probe dungeon-room requested=0x{room:04x} room=0x{loaded_room:04x}");
    }
    let frame_base = direct_entrances.len() as u32 + dungeon_rooms.len() as u32 + 1;
    for (index, screen) in overworld_screens.iter().copied().enumerate() {
        let mut game = base.clone();
        let loaded_screen = game.parity_probe_overworld_screen(screen);
        coverage.record(route_coverage_frame_from_game(
            frame_base + index as u32,
            &game,
        ));
        println!(
            "coverage-probe overworld-screen requested=0x{screen:04x} screen=0x{loaded_screen:04x}"
        );
    }
    write_route_coverage_log_or_exit(&coverage_log, &coverage, "coverage probe log");
}

fn parse_u16_auto(value: &str) -> Option<u16> {
    value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .map(|hex| u16::from_str_radix(hex, 16).ok())
        .unwrap_or_else(|| value.parse::<u16>().ok())
}

fn replay_checksum_ram_range(ram: &[u8], start: usize, size: usize) -> u32 {
    let mut hash = 2166136261u32;
    for index in start..start + size {
        let byte = if parity::fingerprint_mask_contains(index) {
            0
        } else {
            ram[index]
        };
        hash = (hash ^ u32::from(byte)).wrapping_mul(16777619);
    }
    hash
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

fn run_frontend_smoke(args: &[String]) {
    let frames: u32 = args.first().map(|s| s.parse().unwrap_or(2)).unwrap_or(2);
    let mut game = load_embedded_play_state();
    let width = 256u32;
    let height = 224u32;
    let mut renderer = match play_renderer::configured_from_env(
        width,
        height,
        NativeFrontendOptions::from_env(3, false),
    ) {
        Ok(frontend) => frontend,
        Err(e) => {
            eprintln!("failed to initialize native frontend: {e}");
            process::exit(1);
        }
    };
    let renderer_name = renderer.name();

    let mut completed = 0u32;
    while completed < frames && !renderer.quit_requested() {
        let live_input = renderer.poll_input();
        game.zelda_run_frame(live_input as i32);
        renderer.present_frame(&mut game);
        completed += 1;
    }

    println!("frontend smoke completed frames={completed} renderer={renderer_name}");
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

    let width = 256u32;
    let height = 224u32;
    let render_flags = PpuRenderFlags::empty();
    let mut frame = vec![0u8; width as usize * height as usize * 4];
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
        run_play_frame_bgra(&mut game, 0, &mut frame, render_flags);
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
    let width = 256u32;
    let height = 224u32;
    let render_flags = PpuRenderFlags::empty();
    let mut frame = vec![0u8; width as usize * height as usize * 4];
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
        run_play_frame_bgra(&mut game, 0, &mut frame, render_flags);
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

fn run_play(rom_path: &str) {
    run_play_with_state(load_play_state(rom_path));
}

fn run_standalone_play() {
    run_play_with_state(load_embedded_play_state());
}

fn run_play_with_state(mut game: ZeldaState) {
    let last_panic = install_crash_panic_hook();
    let width = 256u32;
    let height = 224u32;
    let mut renderer = match play_renderer::configured_from_env(
        width,
        height,
        NativeFrontendOptions::from_env(3, true),
    ) {
        Ok(frontend) => frontend,
        Err(e) => {
            eprintln!("failed to initialize native frontend: {e}");
            process::exit(1);
        }
    };
    let audio_samples = renderer.audio_samples_per_frame();
    let audio_channels = renderer.audio_channels();
    let mut audio = vec![0i16; audio_samples * audio_channels];
    let mut host_frame = 0u32;
    let mut game_started = env::var_os("ZELDA3_SKIP_HOST_MENU").is_some();
    let mut host_menu = HostMenuState::new(
        HostMenuMode::PreGame,
        developer_destinations::developer_destinations(),
    );
    if game_started {
        host_menu.close();
    }
    let trace_live_input = env::var_os("ZELDA3_TRACE_LIVE_INPUT").is_some();
    let mut last_traced_live_input = (
        u16::MAX,
        u8::MAX,
        u8::MAX,
        u8::MAX,
        u8::MAX,
        u8::MAX,
        u8::MAX,
        u8::MAX,
        u8::MAX,
        u8::MAX,
        u8::MAX,
        u8::MAX,
    );

    while !renderer.quit_requested() {
        let live_input = renderer.poll_input_with_menu(host_menu.is_open());
        let mut should_quit = false;
        for input in renderer.drain_host_menu_inputs() {
            if host_menu.is_open() {
                if let Some(action) = host_menu.handle_input(input) {
                    match action {
                        HostMenuAction::Resume => host_menu.close(),
                        HostMenuAction::StartQuest => {
                            game_started = true;
                            host_menu.close();
                        }
                        HostMenuAction::Quit | HostMenuAction::SaveAndQuit => {
                            should_quit = true;
                        }
                        HostMenuAction::SetPresentation(_)
                        | HostMenuAction::SetLighting(_)
                        | HostMenuAction::SetShadows(_)
                        | HostMenuAction::SetViewport(_)
                        | HostMenuAction::ResetRuntimeSettings(_) => {
                            renderer.apply_runtime_settings(host_menu.runtime_settings());
                        }
                        HostMenuAction::ShowControls(panel) => {
                            eprintln!("host menu controls panel selected: {panel:?}");
                        }
                        HostMenuAction::WarpToVerifiedDestination(id) => {
                            match load_developer_destination(id) {
                                Ok((next_game, next_frame)) => {
                                    game = next_game;
                                    host_frame = next_frame;
                                    game_started = true;
                                    host_menu.close();
                                    eprintln!(
                                        "developer destination loaded: {id} frame={next_frame}"
                                    );
                                }
                                Err(e) => {
                                    eprintln!("developer destination failed: {id}: {e}");
                                }
                            }
                        }
                    }
                }
            } else {
                match input {
                    HostMenuInput::Cancel => host_menu.open_ingame(),
                    HostMenuInput::CyclePresentation
                    | HostMenuInput::CycleLighting
                    | HostMenuInput::CycleShadows => {
                        if let Some(
                            HostMenuAction::SetPresentation(_)
                            | HostMenuAction::SetLighting(_)
                            | HostMenuAction::SetShadows(_),
                        ) = host_menu.handle_input(input)
                        {
                            renderer.apply_runtime_settings(host_menu.runtime_settings());
                        }
                    }
                    _ => {}
                }
            }
        }
        if should_quit {
            break;
        }
        if host_menu.is_open() {
            host_menu.set_current_developer_location(current_developer_location_from_ram(
                &game.ram, host_frame,
            ));
            renderer.present_menu_overlay(&host_menu);
            continue;
        }
        if !game_started {
            game_started = true;
        }
        let run_what = select_run_what(&game.ram);
        let pre_frame_game = game.clone();
        let mut crash_stage = "run_frame";
        let frame_result = panic::catch_unwind(AssertUnwindSafe(|| {
            game.zelda_run_frame(live_input as i32);
            crash_stage = renderer.name();
            renderer.present_frame(&mut game);
            crash_stage = "audio";
            game.zelda_render_audio(&mut audio, audio_samples as i32, audio_channels as i32);
            renderer.push_audio(&audio);
            game.zelda_discard_unused_audio_frames();
        }));
        if let Err(payload) = frame_result {
            let panic_info = captured_panic_from(last_panic.clone(), payload);
            write_play_crash_report(
                &pre_frame_game,
                host_frame,
                live_input,
                run_what,
                crash_stage,
                Some(&panic_info),
            );
            game.zelda_write_sram();
            process::exit(101);
        }
        let trace_state = (
            live_input,
            game.ram[TRACE_JOYPAD1H_LAST],
            game.ram[TRACE_JOYPAD1L_LAST],
            game.ram[TRACE_FILTERED_JOYPAD_H],
            game.ram[TRACE_FILTERED_JOYPAD_L],
            game.ram[TRACE_SELECTFILE_VAR3],
            game.ram[TRACE_SELECTFILE_VAR5],
            game.ram[TRACE_SELECTFILE_VAR7],
            game.ram[TRACE_SELECTFILE_VAR9],
            game.ram[TRACE_SELECTFILE_VAR10],
            game.ram[TRACE_SELECTFILE_VAR11],
            game.ram[TRACE_SELECTFILE_ARR2_1],
        );
        if trace_live_input && trace_state != last_traced_live_input {
            eprintln!(
                "live-input host_frame={host_frame} input=0x{live_input:04x} joyh=0x{:02x} joyl=0x{:02x} fh=0x{:02x} fl=0x{:02x} main={} sub={} subsub={} sel3={} sel5={} sel7={} sel9={} sel10={} sel11={} arr2_1={}",
                game.ram[TRACE_JOYPAD1H_LAST],
                game.ram[TRACE_JOYPAD1L_LAST],
                game.ram[TRACE_FILTERED_JOYPAD_H],
                game.ram[TRACE_FILTERED_JOYPAD_L],
                game.ram[TRACE_MAIN_MODULE_INDEX],
                game.ram[TRACE_SUBMODULE_INDEX],
                game.ram[TRACE_SUBSUBMODULE_INDEX],
                game.ram[TRACE_SELECTFILE_VAR3],
                game.ram[TRACE_SELECTFILE_VAR5],
                game.ram[TRACE_SELECTFILE_VAR7],
                game.ram[TRACE_SELECTFILE_VAR9],
                game.ram[TRACE_SELECTFILE_VAR10],
                game.ram[TRACE_SELECTFILE_VAR11],
                game.ram[TRACE_SELECTFILE_ARR2_1],
            );
            last_traced_live_input = trace_state;
        }
        host_frame = host_frame.wrapping_add(1);
    }
    game.zelda_write_sram();
}

fn load_developer_route_bookmark(id: &str) -> Result<(ZeldaState, u32), String> {
    let bookmark = developer_destinations::route_bookmark(id)
        .ok_or_else(|| format!("unknown or locked destination '{id}'"))?;
    let mut game = load_translated_replay_state(bookmark.rom_path);
    let checkpoint_path = bookmark.checkpoint_path.map(Path::new);
    let frames = if let Some(path) = checkpoint_path.filter(|path| path.exists()) {
        load_replay_save_checkpoint(&mut game, path)
            .map_err(|e| format!("failed to load checkpoint {}: {e}", path.display()))?;
        bookmark.target_frame
    } else {
        game.replay_save_file(Path::new(bookmark.replay_path))
            .map_err(|e| format!("failed to load {}: {e}", bookmark.replay_path))?;
        let mut frames = game.state_recorder.replay_frame_counter;
        while frames < bookmark.target_frame && game.state_recorder.replay_mode {
            game.zelda_run_frame(0);
            frames = frames.wrapping_add(1);
        }
        frames
    };
    if frames != bookmark.target_frame {
        return Err(format!(
            "route replay ended at frame {frames}, before target {}",
            bookmark.target_frame
        ));
    }
    let mut state_recorder = std::mem::take(&mut game.state_recorder);
    ZeldaState::state_recorder_stop_replay(&mut state_recorder);
    game.state_recorder = state_recorder;
    Ok((game, frames))
}

#[derive(Debug, Deserialize)]
struct DeveloperSandboxTilemapManifest {
    format: String,
    width: u16,
    height: u16,
    rows: Vec<Vec<u16>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DeveloperTilesetManifest {
    format: String,
    id: String,
    source: String,
    source_layer: usize,
    cell_width_tiles: u8,
    cell_height_tiles: u8,
    columns: u16,
    rows: u16,
    entries: Vec<DeveloperTilesetEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DeveloperTilesetEntry {
    id: u16,
    source_cell: u16,
    source_x: u8,
    source_y: u8,
    name: String,
    approved: bool,
    tags: Vec<String>,
}

#[derive(Debug, Serialize)]
struct UniqueOverworldCellAtlasManifest {
    format: &'static str,
    id: &'static str,
    cell_width_px: u8,
    cell_height_px: u8,
    columns: u16,
    rows: u16,
    unique_cells: Vec<UniqueOverworldCellManifestEntry>,
}

#[derive(Debug, Serialize)]
struct UniqueOverworldCellManifestEntry {
    id: u16,
    tilemap_entries: [u16; 4],
    tilemap_variants: Vec<[u16; 4]>,
    rendered_hash: u32,
    sources: Vec<UniqueOverworldCellSource>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct UniqueOverworldCellSource {
    screen: u16,
    loaded_screen: u16,
    layer: u8,
    x: u8,
    y: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UniqueOverworldCell {
    tilemap_entries: [u16; 4],
    tilemap_variants: Vec<[u16; 4]>,
    rendered_rgba: Vec<u8>,
    rendered_hash: u32,
    sources: Vec<UniqueOverworldCellSource>,
}

#[derive(Debug, Default)]
struct UniqueOverworldCellCollector {
    cells: Vec<UniqueOverworldCell>,
    index_by_rendered_rgba: HashMap<Vec<u8>, usize>,
}

impl UniqueOverworldCellCollector {
    fn insert(
        &mut self,
        tilemap_entries: [u16; 4],
        rendered_rgba: Vec<u8>,
        source: UniqueOverworldCellSource,
    ) -> u16 {
        if let Some(&index) = self.index_by_rendered_rgba.get(&rendered_rgba) {
            if !self.cells[index]
                .tilemap_variants
                .contains(&tilemap_entries)
            {
                self.cells[index].tilemap_variants.push(tilemap_entries);
            }
            self.cells[index].sources.push(source);
            return index as u16;
        }

        let index = self.cells.len();
        let rendered_hash = fnv32_bytes(&rendered_rgba);
        self.cells.push(UniqueOverworldCell {
            tilemap_entries,
            tilemap_variants: vec![tilemap_entries],
            rendered_rgba,
            rendered_hash,
            sources: vec![source],
        });
        self.index_by_rendered_rgba
            .insert(self.cells[index].rendered_rgba.clone(), index);
        index as u16
    }

    fn manifest(&self, columns: u16) -> UniqueOverworldCellAtlasManifest {
        let rows = if self.cells.is_empty() {
            0
        } else {
            ((self.cells.len() as u16) + columns - 1) / columns
        };
        UniqueOverworldCellAtlasManifest {
            format: "zelda3_unique_overworld_cells_v1",
            id: "unique_overworld_cells",
            cell_width_px: 16,
            cell_height_px: 16,
            columns,
            rows,
            unique_cells: self
                .cells
                .iter()
                .enumerate()
                .map(|(id, cell)| UniqueOverworldCellManifestEntry {
                    id: id as u16,
                    tilemap_entries: cell.tilemap_entries,
                    tilemap_variants: cell.tilemap_variants.clone(),
                    rendered_hash: cell.rendered_hash,
                    sources: cell.sources.clone(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct UniqueOverworldTileAtlasManifest {
    format: &'static str,
    id: &'static str,
    tile_width_px: u8,
    tile_height_px: u8,
    atlas_scale: u8,
    atlas_grid_px: u8,
    columns: u16,
    rows: u16,
    unique_tiles: Vec<UniqueOverworldTileManifestEntry>,
}

#[derive(Debug, Serialize)]
struct UniqueOverworldTileManifestEntry {
    id: u16,
    atlas_col: u16,
    atlas_row: u16,
    atlas_x_px: u16,
    atlas_y_px: u16,
    atlas_width_px: u16,
    atlas_height_px: u16,
    tilemap_entry: u16,
    tilemap_entry_decoded: DecodedTilemapEntry,
    tilemap_variants: Vec<u16>,
    tilemap_variants_decoded: Vec<DecodedTilemapEntry>,
    rendered_hash: u32,
    source_count: usize,
    sources_truncated: bool,
    sources: Vec<UniqueOverworldCellSource>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
struct DecodedTilemapEntry {
    tile_number: u16,
    palette: u8,
    priority: bool,
    hflip: bool,
    vflip: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UniqueOverworldTile {
    tilemap_entry: u16,
    tilemap_variants: Vec<u16>,
    rendered_rgba: Vec<u8>,
    rendered_hash: u32,
    sources: Vec<UniqueOverworldCellSource>,
}

#[derive(Debug, Default)]
struct UniqueOverworldTileCollector {
    tiles: Vec<UniqueOverworldTile>,
    index_by_rendered_rgba: HashMap<Vec<u8>, usize>,
}

impl UniqueOverworldTileCollector {
    fn insert(
        &mut self,
        tilemap_entry: u16,
        rendered_rgba: Vec<u8>,
        source: UniqueOverworldCellSource,
    ) -> u16 {
        if let Some(&index) = self.index_by_rendered_rgba.get(&rendered_rgba) {
            if !self.tiles[index].tilemap_variants.contains(&tilemap_entry) {
                self.tiles[index].tilemap_variants.push(tilemap_entry);
            }
            self.tiles[index].sources.push(source);
            return index as u16;
        }

        let index = self.tiles.len();
        let rendered_hash = fnv32_bytes(&rendered_rgba);
        self.tiles.push(UniqueOverworldTile {
            tilemap_entry,
            tilemap_variants: vec![tilemap_entry],
            rendered_rgba,
            rendered_hash,
            sources: vec![source],
        });
        self.index_by_rendered_rgba
            .insert(self.tiles[index].rendered_rgba.clone(), index);
        index as u16
    }

    fn manifest(
        &self,
        columns: u16,
        atlas_scale: u8,
        atlas_grid_px: u8,
    ) -> UniqueOverworldTileAtlasManifest {
        let rows = if self.tiles.is_empty() {
            0
        } else {
            ((self.tiles.len() as u16) + columns - 1) / columns
        };
        let atlas_tile_width_px = u16::from(8 * atlas_scale);
        let atlas_tile_height_px = u16::from(8 * atlas_scale);
        let atlas_stride_x = atlas_tile_width_px + u16::from(atlas_grid_px);
        let atlas_stride_y = atlas_tile_height_px + u16::from(atlas_grid_px);
        UniqueOverworldTileAtlasManifest {
            format: "zelda3_unique_overworld_tiles_v2",
            id: "unique_overworld_tiles",
            tile_width_px: 8,
            tile_height_px: 8,
            atlas_scale,
            atlas_grid_px,
            columns,
            rows,
            unique_tiles: self
                .tiles
                .iter()
                .enumerate()
                .map(|(id, tile)| {
                    let id = id as u16;
                    let atlas_col = id % columns;
                    let atlas_row = id / columns;
                    UniqueOverworldTileManifestEntry {
                        id,
                        atlas_col,
                        atlas_row,
                        atlas_x_px: u16::from(atlas_grid_px) + atlas_col * atlas_stride_x,
                        atlas_y_px: u16::from(atlas_grid_px) + atlas_row * atlas_stride_y,
                        atlas_width_px: atlas_tile_width_px,
                        atlas_height_px: atlas_tile_height_px,
                        tilemap_entry: tile.tilemap_entry,
                        tilemap_entry_decoded: decode_tilemap_entry(tile.tilemap_entry),
                        tilemap_variants: tile.tilemap_variants.clone(),
                        tilemap_variants_decoded: tile
                            .tilemap_variants
                            .iter()
                            .copied()
                            .map(decode_tilemap_entry)
                            .collect(),
                        rendered_hash: tile.rendered_hash,
                        source_count: tile.sources.len(),
                        sources_truncated: tile.sources.len()
                            > UNIQUE_OVERWORLD_MANIFEST_SOURCE_LIMIT,
                        sources: tile
                            .sources
                            .iter()
                            .take(UNIQUE_OVERWORLD_MANIFEST_SOURCE_LIMIT)
                            .cloned()
                            .collect(),
                    }
                })
                .collect(),
        }
    }
}

/// One cell in the palette-index overworld atlas: 64 raw palette indices (0..=15) for an 8×8 tile,
/// deduped by graphics identity (tile_number + hflip + vflip, palette-agnostic).
#[derive(Debug)]
struct OverworldIndexTile {
    /// All distinct `tilemap_entry & 0xC3FF` values (tile_number + hflip + vflip) that produced
    /// this identical 64-byte index pattern.
    graphics_keys: Vec<u16>,
    indices: [u8; 64],
}

#[derive(Debug, Default)]
struct OverworldIndexTileCollector {
    tiles: Vec<OverworldIndexTile>,
    index_by_pattern: HashMap<[u8; 64], usize>,
}

impl OverworldIndexTileCollector {
    fn insert(&mut self, tilemap_entry: u16, indices: [u8; 64]) {
        // graphics_key strips palette (bits 12-10) and priority (bit 13); keeps tile, hflip, vflip.
        let graphics_key = tilemap_entry & 0xC3FF;
        if let Some(&pos) = self.index_by_pattern.get(&indices) {
            if !self.tiles[pos].graphics_keys.contains(&graphics_key) {
                self.tiles[pos].graphics_keys.push(graphics_key);
            }
            return;
        }
        let pos = self.tiles.len();
        self.tiles.push(OverworldIndexTile {
            graphics_keys: vec![graphics_key],
            indices,
        });
        self.index_by_pattern.insert(indices, pos);
    }
}

#[derive(Serialize)]
struct OverworldIndexTileAtlasManifest {
    format: &'static str,
    tile_width_px: u8,
    tile_height_px: u8,
    cell_count: u32,
    cells: Vec<OverworldIndexTileCellManifest>,
}

#[derive(Serialize)]
struct OverworldIndexTileCellManifest {
    id: u32,
    graphics_keys: Vec<u16>,
}

fn decode_tilemap_entry(entry: u16) -> DecodedTilemapEntry {
    DecodedTilemapEntry {
        tile_number: entry & 0x03ff,
        palette: ((entry >> 10) & 0x07) as u8,
        priority: entry & 0x2000 != 0,
        hflip: entry & 0x4000 != 0,
        vflip: entry & 0x8000 != 0,
    }
}

fn collect_unique_overworld_cells_from_built_bg2_map(
    collector: &mut UniqueOverworldCellCollector,
    game: &ZeldaState,
    requested_screen: u16,
    loaded_screen: u16,
) {
    let width_tiles = 64usize;
    let height_tiles = 64usize;
    for cell_y in 0..height_tiles / 2 {
        for cell_x in 0..width_tiles / 2 {
            let tile_x = cell_x * 2;
            let tile_y = cell_y * 2;
            let entries = [
                game.parity_probe_overworld_bg2_map8_entry(tile_y * width_tiles + tile_x),
                game.parity_probe_overworld_bg2_map8_entry(tile_y * width_tiles + tile_x + 1),
                game.parity_probe_overworld_bg2_map8_entry((tile_y + 1) * width_tiles + tile_x),
                game.parity_probe_overworld_bg2_map8_entry((tile_y + 1) * width_tiles + tile_x + 1),
            ];
            if entries == [0, 0, 0, 0] {
                continue;
            }
            let rendered_rgba = render_snes_4bpp_cell_to_rgba(
                &game.ppu.vram,
                &game.ppu.cgram,
                OVERWORLD_BG_CHR_BASE,
                entries,
            );
            collector.insert(
                entries,
                rendered_rgba,
                UniqueOverworldCellSource {
                    screen: requested_screen,
                    loaded_screen,
                    layer: DEVELOPER_ROOM_SOURCE_BG_LAYER as u8,
                    x: cell_x as u8,
                    y: cell_y as u8,
                },
            );
        }
    }
}

fn collect_unique_overworld_tiles_from_built_bg2_map(
    collector: &mut UniqueOverworldTileCollector,
    index_collector: &mut OverworldIndexTileCollector,
    game: &ZeldaState,
    requested_screen: u16,
    loaded_screen: u16,
) {
    let width_tiles = 64usize;
    let height_tiles = 64usize;
    for tile_y in 0..height_tiles {
        for tile_x in 0..width_tiles {
            let entry = game.parity_probe_overworld_bg2_map8_entry(tile_y * width_tiles + tile_x);
            if entry == 0 {
                continue;
            }
            let rendered_rgba = render_snes_4bpp_tile_to_rgba(
                &game.ppu.vram,
                &game.ppu.cgram,
                OVERWORLD_BG_CHR_BASE,
                entry,
            );
            collector.insert(
                entry,
                rendered_rgba,
                UniqueOverworldCellSource {
                    screen: requested_screen,
                    loaded_screen,
                    layer: DEVELOPER_ROOM_SOURCE_BG_LAYER as u8,
                    x: tile_x as u8,
                    y: tile_y as u8,
                },
            );
            let indices =
                decode_snes_4bpp_tile_indices(&game.ppu.vram, OVERWORLD_BG_CHR_BASE, entry);
            index_collector.insert(entry, indices);
        }
    }
}

fn render_snes_4bpp_cell_to_rgba(
    vram: &[u16],
    cgram: &[u16],
    chr_base_words: usize,
    tilemap_entries: [u16; 4],
) -> Vec<u8> {
    let mut rgba = vec![0u8; 16 * 16 * 4];
    for (index, entry) in tilemap_entries.iter().copied().enumerate() {
        let tile_x = index % 2;
        let tile_y = index / 2;
        draw_snes_4bpp_tilemap_entry_to_rgba(
            vram,
            cgram,
            chr_base_words,
            entry,
            &mut rgba,
            16,
            tile_x * 8,
            tile_y * 8,
            1,
        );
    }
    rgba
}

fn render_snes_4bpp_tile_to_rgba(
    vram: &[u16],
    cgram: &[u16],
    chr_base_words: usize,
    tilemap_entry: u16,
) -> Vec<u8> {
    let mut rgba = vec![0u8; 8 * 8 * 4];
    draw_snes_4bpp_tilemap_entry_to_rgba(
        vram,
        cgram,
        chr_base_words,
        tilemap_entry,
        &mut rgba,
        8,
        0,
        0,
        1,
    );
    rgba
}

fn render_unique_overworld_cell_atlas(
    collector: &UniqueOverworldCellCollector,
    columns: usize,
    scale: usize,
) -> (Vec<u8>, u32, u32) {
    let rows = if collector.cells.is_empty() {
        0usize
    } else {
        (collector.cells.len() + columns - 1) / columns
    };
    let cell_px = 16usize;
    let grid_px = 1usize;
    let width = columns * cell_px * scale + (columns + 1) * grid_px;
    let height = rows * cell_px * scale + (rows + 1) * grid_px;
    let mut atlas = vec![0u8; width * height * 4];
    for px in atlas.chunks_exact_mut(4) {
        px.copy_from_slice(&[24, 24, 24, 0xff]);
    }
    for (id, cell) in collector.cells.iter().enumerate() {
        let dst_x = grid_px + (id % columns) * (cell_px * scale + grid_px);
        let dst_y = grid_px + (id / columns) * (cell_px * scale + grid_px);
        blit_scaled_rgba_cell(
            &cell.rendered_rgba,
            &mut atlas,
            width,
            dst_x,
            dst_y,
            16,
            scale,
        );
    }
    (atlas, width as u32, height as u32)
}

fn render_unique_overworld_tile_atlas(
    collector: &UniqueOverworldTileCollector,
    columns: usize,
    scale: usize,
) -> (Vec<u8>, u32, u32) {
    let rows = if collector.tiles.is_empty() {
        0usize
    } else {
        (collector.tiles.len() + columns - 1) / columns
    };
    let tile_px = 8usize;
    let grid_px = 1usize;
    let width = columns * tile_px * scale + (columns + 1) * grid_px;
    let height = rows * tile_px * scale + (rows + 1) * grid_px;
    let mut atlas = vec![0u8; width * height * 4];
    for px in atlas.chunks_exact_mut(4) {
        px.copy_from_slice(&[24, 24, 24, 0xff]);
    }
    for (id, tile) in collector.tiles.iter().enumerate() {
        let dst_x = grid_px + (id % columns) * (tile_px * scale + grid_px);
        let dst_y = grid_px + (id / columns) * (tile_px * scale + grid_px);
        blit_scaled_rgba_cell(
            &tile.rendered_rgba,
            &mut atlas,
            width,
            dst_x,
            dst_y,
            8,
            scale,
        );
    }
    (atlas, width as u32, height as u32)
}

fn blit_scaled_rgba_cell(
    source: &[u8],
    out: &mut [u8],
    out_width: usize,
    out_x: usize,
    out_y: usize,
    cell_px: usize,
    scale: usize,
) {
    for y in 0..cell_px {
        for x in 0..cell_px {
            let src_index = (y * cell_px + x) * 4;
            for yy in 0..scale {
                for xx in 0..scale {
                    let out_index =
                        ((out_y + y * scale + yy) * out_width + out_x + x * scale + xx) * 4;
                    out[out_index..out_index + 4]
                        .copy_from_slice(&source[src_index..src_index + 4]);
                }
            }
        }
    }
}

fn fnv32_bytes(data: &[u8]) -> u32 {
    let mut hash = 0x811c9dc5u32;
    for &byte in data {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}

fn load_developer_destination(id: &str) -> Result<(ZeldaState, u32), String> {
    match developer_destinations::destination_target(id)
        .ok_or_else(|| format!("unknown or locked destination '{id}'"))?
    {
        developer_destinations::DeveloperDestinationTarget::RouteBookmark(bookmark) => {
            load_developer_route_bookmark(bookmark.id)
        }
        developer_destinations::DeveloperDestinationTarget::SyntheticRoom(room) => {
            load_developer_synthetic_room(room)
        }
    }
}

fn developer_sandbox_tilemap_manifest(
    room: developer_destinations::DeveloperSyntheticRoom,
) -> Result<DeveloperSandboxTilemapManifest, String> {
    let manifest: DeveloperSandboxTilemapManifest = serde_json::from_str(room.tilemap_json)
        .map_err(|e| format!("failed to parse {} tilemap JSON: {e}", room.id))?;
    if manifest.format != "zelda3_byte_tilemap_v1" {
        return Err(format!(
            "{} uses unsupported tilemap format {}",
            room.id, manifest.format
        ));
    }
    if manifest.width == 0 || manifest.height == 0 {
        return Err(format!("{} tilemap must be non-empty", room.id));
    }
    if manifest.rows.len() != manifest.height as usize {
        return Err(format!(
            "{} tilemap row count {} does not match height {}",
            room.id,
            manifest.rows.len(),
            manifest.height
        ));
    }
    for (row_index, row) in manifest.rows.iter().enumerate() {
        if row.len() != manifest.width as usize {
            return Err(format!(
                "{} tilemap row {row_index} width {} does not match {}",
                room.id,
                row.len(),
                manifest.width
            ));
        }
    }
    Ok(manifest)
}

fn developer_kakariko_tileset_manifest() -> Result<DeveloperTilesetManifest, String> {
    const KAKARIKO_TILESET_JSON: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/kakariko_town_tileset.json"
    ));
    serde_json::from_str(KAKARIKO_TILESET_JSON)
        .map_err(|e| format!("failed to parse Kakariko developer tileset: {e}"))
}

fn load_developer_synthetic_room(
    room: developer_destinations::DeveloperSyntheticRoom,
) -> Result<(ZeldaState, u32), String> {
    let manifest = developer_sandbox_tilemap_manifest(room)?;
    let mut game =
        load_translated_replay_state(concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc"));
    load_replay_save_checkpoint(&mut game, Path::new(room.seed_checkpoint_path)).map_err(|e| {
        format!(
            "failed to load synthetic room seed checkpoint {}: {e}",
            room.seed_checkpoint_path
        )
    })?;
    let mut state_recorder = std::mem::take(&mut game.state_recorder);
    ZeldaState::state_recorder_stop_replay(&mut state_recorder);
    game.state_recorder = state_recorder;

    game.developer_prepare_synthetic_room(room.room_id);
    game.developer_queue_music_track(room.music_track);

    let theme_source = load_developer_room_theme_source(room)?;
    write_developer_room_visuals_to_ppu(&mut game, &manifest, room.visual_theme, &theme_source);
    Ok((game, 0))
}

fn load_developer_room_theme_source(
    room: developer_destinations::DeveloperSyntheticRoom,
) -> Result<ZeldaState, String> {
    let mut source =
        load_translated_replay_state(concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc"));
    load_replay_save_checkpoint(&mut source, Path::new(room.theme_checkpoint_path)).map_err(
        |e| {
            format!(
                "failed to load synthetic room theme checkpoint {}: {e}",
                room.theme_checkpoint_path
            )
        },
    )?;
    for _ in 0..32 {
        source.zelda_run_frame(0);
    }
    Ok(source)
}

fn write_developer_room_visuals_to_ppu(
    game: &mut ZeldaState,
    manifest: &DeveloperSandboxTilemapManifest,
    theme: developer_destinations::DeveloperSyntheticRoomTheme,
    source: &ZeldaState,
) {
    game.ppu.mode = 1;
    game.ppu.forced_blank = false;
    game.ppu.brightness = 15;
    game.ppu.screen_enabled[0] = 0x11; // BG1 plus sprites.
    game.ppu.screen_enabled[1] = 0x00;
    game.ppu.bg_layer[0].tilemap_adr = 0x1000;
    game.ppu.bg_layer[0].tile_adr = DEVELOPER_ROOM_BG_TILE_BASE;
    game.ppu.bg_layer[0].tilemap_wider = false;
    game.ppu.bg_layer[0].tilemap_higher = false;
    game.ram[BGMODE_COPY] = 9;
    game.ram[TM_COPY] = 0x11;
    game.ram[TS_COPY] = 0x00;

    write_developer_room_palette_from_source(game, source);
    game.ppu.vram.copy_from_slice(&source.ppu.vram);
    copy_developer_room_chr_from_source(game, source);

    let base = game.ppu.bg_layer[0].tilemap_adr as usize;
    let source_base = source.ppu.bg_layer[DEVELOPER_ROOM_SOURCE_BG_LAYER].tilemap_adr as usize;
    let stride = 32usize;
    for (row_index, row) in manifest.rows.iter().enumerate() {
        for (column_index, tile) in row.iter().enumerate() {
            let (sample_x, sample_y) = match theme {
                developer_destinations::DeveloperSyntheticRoomTheme::Kakariko => {
                    developer_room_kakariko_sample_origin(source, *tile)
                }
            };
            let screen_x = column_index * 2;
            let screen_y = row_index * 2;
            for y_offset in 0..2 {
                for x_offset in 0..2 {
                    let source_index =
                        source_base + (sample_y + y_offset) * stride + sample_x + x_offset;
                    let destination_index =
                        base + (screen_y + y_offset) * stride + screen_x + x_offset;
                    if let (Some(&entry), Some(slot)) = (
                        source.ppu.vram.get(source_index),
                        game.ppu.vram.get_mut(destination_index),
                    ) {
                        *slot = entry;
                    }
                }
            }
        }
    }
}

fn write_developer_room_palette_from_source(game: &mut ZeldaState, source: &ZeldaState) {
    game.ppu.cgram.copy_from_slice(&source.ppu.cgram);
    for (index, color) in source.ppu.cgram.iter().copied().enumerate() {
        write_le_u16(&mut game.ram, MAIN_PALETTE_BUFFER + index * 2, color);
    }
    game.ram[FLAG_UPDATE_CGRAM_IN_NMI] = 1;
}

fn copy_developer_room_chr_from_source(game: &mut ZeldaState, source: &ZeldaState) {
    let source_vram = source.ppu.vram.clone();
    let source_base = source.ppu.bg_layer[DEVELOPER_ROOM_SOURCE_BG_LAYER].tile_adr as usize;
    let destination_base = usize::from(DEVELOPER_ROOM_BG_TILE_BASE);
    for tile in 0..1024usize {
        let source_index = source_base + tile * 16;
        let destination_index = destination_base + tile * 16;
        if source_index + 16 <= source_vram.len() && destination_index + 16 <= game.ppu.vram.len() {
            game.ppu.vram[destination_index..destination_index + 16]
                .copy_from_slice(&source_vram[source_index..source_index + 16]);
        }
    }
}

fn developer_room_kakariko_sample_origin(source: &ZeldaState, tile: u16) -> (usize, usize) {
    let visible_cell = developer_room_kakariko_visible_cell(tile);
    let visible_x = usize::from(visible_cell % 16) * 2;
    let visible_y = usize::from(visible_cell / 16) * 2;
    let bg = &source.ppu.bg_layer[DEVELOPER_ROOM_SOURCE_BG_LAYER];
    let source_x = (visible_x * 8 + usize::from(bg.h_scroll)) % 256;
    let source_y = (visible_y * 8 + usize::from(bg.v_scroll) + 1) % 256;
    (source_x / 8, source_y / 8)
}

fn developer_room_kakariko_visible_cell(tile: u16) -> u16 {
    match tile {
        DEV_TOWN_ROOF => 2,
        DEV_TOWN_WALL => 34,
        DEV_TOWN_DOOR => 35,
        DEV_TOWN_GRASS => 71,
        DEV_TOWN_PATH => 179,
        DEV_TOWN_FENCE => 97,
        DEV_TOWN_SHRUB => 64,
        DEV_TOWN_SIGN => 75,
        DEV_TOWN_TREE => 11,
        DEV_TOWN_CLIFF_TOP => 110,
        DEV_TOWN_CLIFF_FACE => 126,
        DEV_TOWN_FLOWERS => 31,
        DEV_TOWN_STONE => 176,
        DEV_TOWN_HEDGE => 30,
        _ => tile.min(223),
    }
}

#[cfg(test)]
fn apply_host_menu_action_for_test(
    menu: &mut HostMenuState,
    action: HostMenuAction,
    should_start: &mut bool,
    should_quit: &mut bool,
) {
    match action {
        HostMenuAction::Resume => menu.close(),
        HostMenuAction::StartQuest => {
            *should_start = true;
            menu.close();
        }
        HostMenuAction::Quit | HostMenuAction::SaveAndQuit => *should_quit = true,
        HostMenuAction::SetPresentation(_)
        | HostMenuAction::SetLighting(_)
        | HostMenuAction::SetShadows(_)
        | HostMenuAction::SetViewport(_)
        | HostMenuAction::ShowControls(_)
        | HostMenuAction::ResetRuntimeSettings(_)
        | HostMenuAction::WarpToVerifiedDestination(_) => {}
    }
}

#[cfg(test)]
mod host_menu_play_tests {
    use super::*;

    #[test]
    fn menu_resume_action_closes_ingame_menu() {
        let mut menu = HostMenuState::new(HostMenuMode::InGame, Vec::new());
        let mut should_quit = false;
        let mut should_start = false;
        apply_host_menu_action_for_test(
            &mut menu,
            HostMenuAction::Resume,
            &mut should_start,
            &mut should_quit,
        );
        assert!(!menu.is_open());
        assert!(!should_start);
        assert!(!should_quit);
    }

    #[test]
    fn menu_start_action_closes_pregame_menu_and_starts_game() {
        let mut menu = HostMenuState::new(HostMenuMode::PreGame, Vec::new());
        let mut should_quit = false;
        let mut should_start = false;
        apply_host_menu_action_for_test(
            &mut menu,
            HostMenuAction::StartQuest,
            &mut should_start,
            &mut should_quit,
        );
        assert!(!menu.is_open());
        assert!(should_start);
        assert!(!should_quit);
    }

    #[test]
    fn developer_route_start_bookmark_loads_and_stops_replay() {
        let (game, frame) = load_developer_route_bookmark("route-start")
            .expect("route-start bookmark should load bundled route save");
        assert_eq!(frame, 0);
        assert!(!game.state_recorder.replay_mode);
    }

    #[test]
    fn developer_late_checkpoint_bookmark_uses_prepared_checkpoint_when_present() {
        let bookmark = developer_destinations::route_bookmark("route-late-checkpoint")
            .expect("late checkpoint bookmark should be in developer manifest");
        let Some(path) = bookmark.checkpoint_path else {
            panic!("late checkpoint bookmark should declare a prepared checkpoint path");
        };
        if !Path::new(path).exists() {
            eprintln!("skipping late checkpoint load test because {path} is not present");
            return;
        }

        let (game, frame) = load_developer_route_bookmark("route-late-checkpoint")
            .expect("late checkpoint bookmark should load prepared checkpoint");
        assert_eq!(frame, 1_045_813);
        assert!(!game.state_recorder.replay_mode);
    }

    #[test]
    fn developer_sandbox_room_manifest_uses_byte_tilemap_json() {
        let room = developer_destinations::synthetic_room("preset-dev-sandbox")
            .expect("sandbox room should be registered");
        let manifest = developer_sandbox_tilemap_manifest(room).expect("manifest should parse");

        assert_eq!(manifest.format, "zelda3_byte_tilemap_v1");
        assert_eq!(manifest.width, 16);
        assert_eq!(manifest.height, 14);
        assert_eq!(manifest.rows.len(), manifest.height as usize);
        assert!(manifest
            .rows
            .iter()
            .all(|row| row.len() == manifest.width as usize));
    }

    fn load_developer_kakariko_tileset_manifest_for_test() -> DeveloperTilesetManifest {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("developer_tilesets")
            .join("kakariko_town_tileset.json");
        let data = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        serde_json::from_str(&data)
            .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()))
    }

    #[test]
    fn developer_kakariko_tileset_manifest_extracts_full_visible_grid() {
        let tileset = load_developer_kakariko_tileset_manifest_for_test();

        assert_eq!(tileset.format, "zelda3_developer_tileset_v1");
        assert_eq!(tileset.id, "kakariko_town");
        assert_eq!(tileset.source_layer, DEVELOPER_ROOM_SOURCE_BG_LAYER);
        assert_eq!(tileset.cell_width_tiles, 2);
        assert_eq!(tileset.cell_height_tiles, 2);
        assert_eq!(tileset.columns, 16);
        assert_eq!(tileset.rows, 14);
        assert_eq!(tileset.entries.len(), 224);
        for (expected_id, entry) in tileset.entries.iter().enumerate() {
            assert_eq!(entry.id, expected_id as u16);
            assert_eq!(entry.source_cell, expected_id as u16);
            assert_eq!(entry.source_x, (expected_id % 16) as u8 * 2);
            assert_eq!(entry.source_y, (expected_id / 16) as u8 * 2);
        }
        for required in [
            "house.roof.left",
            "house.wall.door",
            "grass.clean.01",
            "grass.clean.02",
            "grass.flowers.01",
            "fence.horizontal.left",
            "fence.horizontal.mid",
            "cliff.top.mid",
            "path.stone.light",
        ] {
            assert!(
                tileset
                    .entries
                    .iter()
                    .any(|entry| entry.name == required && entry.approved),
                "missing approved Kakariko tile {required}"
            );
        }
    }

    #[test]
    fn developer_sandbox_room_uses_approved_kakariko_tileset_entries() {
        let room = developer_destinations::synthetic_room("preset-dev-sandbox")
            .expect("sandbox room should be registered");
        let manifest = developer_sandbox_tilemap_manifest(room).expect("manifest should parse");
        let tileset = load_developer_kakariko_tileset_manifest_for_test();

        for row in &manifest.rows {
            for &tile in row {
                let entry = tileset
                    .entries
                    .iter()
                    .find(|entry| entry.id == tile)
                    .unwrap_or_else(|| panic!("room references missing tileset id {tile}"));
                assert!(entry.approved, "room references unapproved tile id {tile}");
            }
        }
    }

    #[test]
    fn developer_tileset_dump_decodes_vram_tile_entries_directly() {
        let mut vram = vec![0u16; 0x8000];
        let mut cgram = vec![0u16; 0x100];
        let chr_base = 0x20usize;
        let tile_number = 3usize;
        let tile_base = chr_base + tile_number * 16;
        vram[tile_base] = 0x0080;
        cgram[0x21] = 0x001f;

        let mut out = vec![0u8; 8 * 8 * 4];
        draw_snes_4bpp_tilemap_entry_to_rgba(
            &vram,
            &cgram,
            chr_base,
            tile_number as u16 | (2 << 10),
            &mut out,
            8,
            0,
            0,
            1,
        );
        assert_eq!(&out[0..4], &[248, 0, 0, 0xff]);
        assert_eq!(&out[4..8], &[0, 0, 0, 0xff]);

        let mut flipped = vec![0u8; 8 * 8 * 4];
        draw_snes_4bpp_tilemap_entry_to_rgba(
            &vram,
            &cgram,
            chr_base,
            tile_number as u16 | (2 << 10) | 0x4000,
            &mut flipped,
            8,
            0,
            0,
            1,
        );
        assert_eq!(&flipped[0..4], &[0, 0, 0, 0xff]);
        assert_eq!(&flipped[7 * 4..7 * 4 + 4], &[248, 0, 0, 0xff]);
    }

    #[test]
    fn unique_overworld_cell_collector_collapses_duplicate_tilemap_entries() {
        let mut collector = UniqueOverworldCellCollector::default();
        let rgba = vec![0xaa; 16 * 16 * 4];
        let first_source = UniqueOverworldCellSource {
            screen: 0x00,
            loaded_screen: 0x00,
            layer: 1,
            x: 3,
            y: 4,
        };
        let second_source = UniqueOverworldCellSource {
            screen: 0x40,
            loaded_screen: 0x40,
            layer: 1,
            x: 5,
            y: 6,
        };

        let first_id = collector.insert([1, 2, 3, 4], rgba.clone(), first_source.clone());
        let second_id = collector.insert([1, 2, 3, 4], rgba, second_source.clone());

        assert_eq!(first_id, 0);
        assert_eq!(second_id, first_id);
        assert_eq!(collector.cells.len(), 1);
        assert_eq!(
            collector.cells[0].sources,
            vec![first_source, second_source]
        );
    }

    #[test]
    fn unique_overworld_cell_collector_collapses_identical_rendered_cells() {
        let mut collector = UniqueOverworldCellCollector::default();
        let rgba = vec![0x55; 16 * 16 * 4];
        let first_source = UniqueOverworldCellSource {
            screen: 0x00,
            loaded_screen: 0x00,
            layer: 1,
            x: 3,
            y: 4,
        };
        let second_source = UniqueOverworldCellSource {
            screen: 0x40,
            loaded_screen: 0x40,
            layer: 1,
            x: 5,
            y: 6,
        };

        let first_id = collector.insert([1, 2, 3, 4], rgba.clone(), first_source.clone());
        let second_id = collector.insert([5, 6, 7, 8], rgba, second_source.clone());

        assert_eq!(first_id, 0);
        assert_eq!(second_id, first_id);
        assert_eq!(collector.cells.len(), 1);
        assert_eq!(
            collector.cells[0].tilemap_variants,
            vec![[1, 2, 3, 4], [5, 6, 7, 8]]
        );
        assert_eq!(
            collector.cells[0].sources,
            vec![first_source, second_source]
        );
    }

    #[test]
    fn unique_overworld_cell_manifest_records_sources_and_layout() {
        let mut collector = UniqueOverworldCellCollector::default();
        collector.insert(
            [1, 2, 3, 4],
            vec![0x11; 16 * 16 * 4],
            UniqueOverworldCellSource {
                screen: 0x02,
                loaded_screen: 0x02,
                layer: 1,
                x: 7,
                y: 8,
            },
        );
        collector.insert(
            [5, 6, 7, 8],
            vec![0x22; 16 * 16 * 4],
            UniqueOverworldCellSource {
                screen: 0x03,
                loaded_screen: 0x03,
                layer: 1,
                x: 9,
                y: 10,
            },
        );

        let manifest = collector.manifest(16);

        assert_eq!(manifest.format, "zelda3_unique_overworld_cells_v1");
        assert_eq!(manifest.columns, 16);
        assert_eq!(manifest.rows, 1);
        assert_eq!(manifest.unique_cells.len(), 2);
        assert_eq!(manifest.unique_cells[0].id, 0);
        assert_eq!(manifest.unique_cells[0].tilemap_entries, [1, 2, 3, 4]);
        assert_eq!(manifest.unique_cells[0].sources[0].screen, 0x02);
        assert_eq!(manifest.unique_cells[1].id, 1);
        assert_eq!(manifest.unique_cells[1].tilemap_entries, [5, 6, 7, 8]);
    }

    #[test]
    fn unique_overworld_tile_collector_collapses_identical_rendered_tiles() {
        let mut collector = UniqueOverworldTileCollector::default();
        let rgba = vec![0x77; 8 * 8 * 4];
        let first_source = UniqueOverworldCellSource {
            screen: 0x00,
            loaded_screen: 0x00,
            layer: 1,
            x: 3,
            y: 4,
        };
        let second_source = UniqueOverworldCellSource {
            screen: 0x40,
            loaded_screen: 0x40,
            layer: 1,
            x: 5,
            y: 6,
        };

        let first_id = collector.insert(0x0123, rgba.clone(), first_source.clone());
        let second_id = collector.insert(0x4567, rgba, second_source.clone());

        assert_eq!(first_id, 0);
        assert_eq!(second_id, first_id);
        assert_eq!(collector.tiles.len(), 1);
        assert_eq!(collector.tiles[0].tilemap_variants, vec![0x0123, 0x4567]);
        assert_eq!(
            collector.tiles[0].sources,
            vec![first_source, second_source]
        );
    }

    #[test]
    fn unique_overworld_tile_manifest_records_atlas_and_decoded_tilemap_metadata() {
        let mut collector = UniqueOverworldTileCollector::default();
        collector.insert(
            0xed23,
            vec![0x33; 8 * 8 * 4],
            UniqueOverworldCellSource {
                screen: 0x00,
                loaded_screen: 0x00,
                layer: 1,
                x: 3,
                y: 4,
            },
        );
        collector.insert(
            0x0124,
            vec![0x33; 8 * 8 * 4],
            UniqueOverworldCellSource {
                screen: 0x01,
                loaded_screen: 0x01,
                layer: 1,
                x: 5,
                y: 6,
            },
        );

        let manifest = collector.manifest(4, 4, 1);
        let tile = &manifest.unique_tiles[0];

        assert_eq!(manifest.format, "zelda3_unique_overworld_tiles_v2");
        assert_eq!(manifest.atlas_scale, 4);
        assert_eq!(manifest.atlas_grid_px, 1);
        assert_eq!(tile.atlas_col, 0);
        assert_eq!(tile.atlas_row, 0);
        assert_eq!(tile.atlas_x_px, 1);
        assert_eq!(tile.atlas_y_px, 1);
        assert_eq!(tile.atlas_width_px, 32);
        assert_eq!(tile.atlas_height_px, 32);
        assert_eq!(tile.tilemap_entry_decoded.tile_number, 0x0123);
        assert_eq!(tile.tilemap_entry_decoded.palette, 3);
        assert!(tile.tilemap_entry_decoded.priority);
        assert!(tile.tilemap_entry_decoded.hflip);
        assert!(tile.tilemap_entry_decoded.vflip);
        assert_eq!(tile.tilemap_variants_decoded[1].tile_number, 0x0124);
    }

    #[test]
    fn unique_overworld_tile_manifest_caps_source_samples() {
        let mut collector = UniqueOverworldTileCollector::default();
        for x in 0..40u8 {
            collector.insert(
                0x0123,
                vec![0x44; 8 * 8 * 4],
                UniqueOverworldCellSource {
                    screen: u16::from(x),
                    loaded_screen: u16::from(x),
                    layer: 1,
                    x,
                    y: 0,
                },
            );
        }

        let manifest = collector.manifest(4, 4, 1);
        let tile = &manifest.unique_tiles[0];

        assert_eq!(tile.source_count, 40);
        assert!(tile.sources_truncated);
        assert_eq!(tile.sources.len(), UNIQUE_OVERWORLD_MANIFEST_SOURCE_LIMIT);
    }

    #[test]
    fn unique_overworld_probe_loads_graphics_for_rendered_cells() {
        let mut game = load_translated_replay_state(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../saves/zelda3.sfc"
        ));
        let loaded_screen = game.parity_probe_overworld_screen_and_build_map(0);
        let mut collector = UniqueOverworldCellCollector::default();

        collect_unique_overworld_cells_from_built_bg2_map(&mut collector, &game, 0, loaded_screen);

        assert!(
            collector.cells.iter().any(|cell| {
                let colors = cell
                    .rendered_rgba
                    .chunks_exact(4)
                    .filter(|pixel| *pixel != [0, 0, 0, 0xff])
                    .collect::<std::collections::HashSet<_>>();
                colors.len() >= 2
            }),
            "loaded overworld cells should render with varied graphics and palette colors"
        );
    }

    #[test]
    fn developer_sandbox_room_manifest_is_semantic_town_square() {
        let room = developer_destinations::synthetic_room("preset-dev-sandbox")
            .expect("sandbox room should be registered");
        let manifest = developer_sandbox_tilemap_manifest(room).expect("manifest should parse");

        assert!(
            manifest.rows[4..=8].iter().all(|row| row
                .iter()
                .all(|&tile| matches!(tile, 76..=78 | 91..=93 | 96..=101 | 104..=108))),
            "middle of the room should be a coherent walkable grass plaza"
        );
        assert!(
            manifest.rows[1][0..=7].iter().copied().eq(16..=23)
                && manifest.rows[2][0..=7].iter().copied().eq(32..=39),
            "top-left should read as a house frontage"
        );
        assert!(
            manifest.rows[9][2..=13]
                .iter()
                .filter(|&&tile| (128..=135).contains(&tile))
                .count()
                >= 6,
            "lower half should include a town-square fence line"
        );
    }

    #[test]
    fn developer_sandbox_town_square_uses_coherent_kakariko_chunks() {
        let room = developer_destinations::synthetic_room("preset-dev-sandbox")
            .expect("sandbox room should be registered");
        let manifest = developer_sandbox_tilemap_manifest(room).expect("manifest should parse");

        assert_eq!(
            &manifest.rows[0][0..8],
            &[0, 1, 2, 3, 4, 5, 6, 7],
            "top-left should preserve the Kakariko roof chunk"
        );
        assert_eq!(
            &manifest.rows[2][0..8],
            &[32, 33, 34, 35, 36, 37, 38, 39],
            "house frontage should come from adjacent source cells, not repeated wall samples"
        );
        assert_eq!(
            &manifest.rows[9][0..8],
            &[128, 129, 130, 131, 132, 133, 134, 135],
            "fence line should preserve a coherent Kakariko fence run"
        );
        assert_eq!(
            &manifest.rows[10][8..14],
            &[120, 121, 122, 123, 124, 125],
            "cliff edge should preserve adjacent cliff source cells"
        );
    }

    #[test]
    fn developer_sandbox_town_square_keeps_plaza_clear_of_house_chunks() {
        let room = developer_destinations::synthetic_room("preset-dev-sandbox")
            .expect("sandbox room should be registered");
        let manifest = developer_sandbox_tilemap_manifest(room).expect("manifest should parse");

        let plaza_cells = [
            76, 77, 78, 91, 92, 93, 96, 97, 98, 99, 100, 101, 104, 105, 106, 107, 108,
        ];
        for row in &manifest.rows[4..=8] {
            assert!(
                row.iter().all(|tile| plaza_cells.contains(tile)),
                "plaza rows should use clean Kakariko grass/flower samples, not cluttered crop cells"
            );
        }
        for row in &manifest.rows[4..] {
            assert!(
                row.iter().all(|&tile| !(0..=63).contains(&tile)),
                "house-front source cells should stay in the top frontage band"
            );
        }
    }

    #[test]
    fn developer_sandbox_semantic_samples_resolve_to_visible_kakariko_cells() {
        let room = developer_destinations::synthetic_room("preset-dev-sandbox")
            .expect("sandbox room should be registered");
        let source = load_developer_room_theme_source(room).expect("theme source should load");
        let expected_cells = [
            (DEV_TOWN_ROOF, 2),
            (DEV_TOWN_WALL, 34),
            (DEV_TOWN_DOOR, 35),
            (DEV_TOWN_GRASS, 71),
            (DEV_TOWN_PATH, 179),
            (DEV_TOWN_FENCE, 97),
            (DEV_TOWN_SHRUB, 64),
            (DEV_TOWN_SIGN, 75),
            (DEV_TOWN_TREE, 11),
            (DEV_TOWN_CLIFF_TOP, 110),
            (DEV_TOWN_CLIFF_FACE, 126),
            (DEV_TOWN_FLOWERS, 31),
            (DEV_TOWN_STONE, 176),
            (DEV_TOWN_HEDGE, 30),
        ];

        for (semantic_tile, visible_cell) in expected_cells {
            assert_eq!(
                developer_room_kakariko_sample_origin(&source, semantic_tile),
                developer_room_kakariko_sample_origin(&source, visible_cell),
                "semantic sample {semantic_tile} should resolve to visible Kakariko cell {visible_cell}"
            );
        }
    }

    #[test]
    fn developer_sandbox_preset_loads_synthetic_room_state() {
        let (game, frame) = load_developer_destination("preset-dev-sandbox")
            .expect("sandbox preset should load from JSON-backed synthetic room");

        assert_eq!(frame, 0);
        assert_eq!(game.ram[PLAYER_IS_INDOORS], 1);
        assert_eq!(read_le_u16(&game.ram, 0x48e), 0x01ff);
        assert_eq!(game.ram[MUSIC_CONTROL], DEVELOPER_ROOM_KAKARIKO_MUSIC);

        let location = current_developer_location_from_ram(&game.ram, frame);
        assert_eq!(location.location, "ROOM 01FF");
        assert_eq!(location.thumbnail, platform::DeveloperThumbnail::DevRoom);
    }

    #[test]
    fn developer_sandbox_starts_kakariko_music_after_first_frame() {
        let (mut game, _) = load_developer_destination("preset-dev-sandbox")
            .expect("sandbox preset should load from JSON-backed synthetic room");

        assert_eq!(game.ram[MUSIC_CONTROL], DEVELOPER_ROOM_KAKARIKO_MUSIC);

        game.zelda_run_frame(0);

        assert_eq!(game.ram[MUSIC_CONTROL], 0);
        assert_eq!(
            game.ram[CURRENT_MUSIC_CONTROL],
            DEVELOPER_ROOM_KAKARIKO_MUSIC
        );
        assert_eq!(game.ram[LAST_MUSIC_CONTROL], DEVELOPER_ROOM_KAKARIKO_MUSIC);
    }

    #[test]
    fn developer_sandbox_stays_in_room_after_first_game_frame() {
        let (mut game, _) = load_developer_destination("preset-dev-sandbox")
            .expect("sandbox preset should load from JSON-backed synthetic room");

        assert!(!game.state_recorder.replay_mode);
        assert_eq!(game.ram[TRACE_MAIN_MODULE_INDEX], 0x07);
        assert_eq!(game.ram[TRACE_SUBMODULE_INDEX], 0x00);
        assert_eq!(game.ram[PLAYER_IS_INDOORS], 1);
        assert_eq!(read_le_u16(&game.ram, 0x48e), 0x01ff);

        game.zelda_run_frame(0);

        assert_eq!(game.ram[TRACE_MAIN_MODULE_INDEX], 0x07);
        assert_eq!(game.ram[PLAYER_IS_INDOORS], 1);
        assert_eq!(read_le_u16(&game.ram, 0x48e), 0x01ff);
        assert!(!game.ppu.forced_blank);
        assert!(game.ppu.brightness > 0);
        assert_eq!(game.ppu.screen_enabled[0] & 0x11, 0x11);
        assert_eq!(game.ppu.screen_enabled[0] & 0x06, 0x00);
        assert_eq!(game.ppu.screen_enabled[1], 0x00);
    }

    #[test]
    fn developer_sandbox_responds_to_live_movement_input() {
        let (mut game, _) = load_developer_destination("preset-dev-sandbox")
            .expect("sandbox preset should load from JSON-backed synthetic room");

        game.zelda_run_frame(0);
        let start_x = read_le_u16(&game.ram, 0x22);
        let start_y = read_le_u16(&game.ram, 0x20);
        for _ in 0..30 {
            game.zelda_run_frame(1 << 7);
        }
        let end_x = read_le_u16(&game.ram, 0x22);
        let end_y = read_le_u16(&game.ram, 0x20);

        assert_eq!(game.ram[TRACE_MAIN_MODULE_INDEX], 0x07);
        assert_ne!((end_x, end_y), (start_x, start_y));
    }

    #[test]
    fn developer_sandbox_does_not_inherit_room_actors_or_dialogue() {
        let (game, _) = load_developer_destination("preset-dev-sandbox")
            .expect("sandbox preset should load from JSON-backed synthetic room");

        assert!(game.ram[0x0dd0..0x0de0].iter().all(|&state| state == 0));
        assert!(game.ram[0x0e20..0x0e30].iter().all(|&ty| ty == 0));
        assert!(game.ram[0x0c4a..0x0c54].iter().all(|&ty| ty == 0));
        assert!(game.ram[0x0b00..0x0b08].iter().all(|&ty| ty == 0));
        assert!(game.ram[0x1f800..0x1f81e].iter().all(|&ty| ty == 0));
        assert_eq!(game.ram[0x1cd8], 0);
        assert_eq!(game.ram[0x1cd4], 0);
        assert_eq!(game.ram[0x1cf0], 0);
        assert_eq!(game.ram[0x1cf1], 0);
    }

    #[test]
    fn developer_sandbox_uses_json_tilemap_after_frame() {
        let room = developer_destinations::synthetic_room("preset-dev-sandbox")
            .expect("sandbox room should be registered");
        let manifest = developer_sandbox_tilemap_manifest(room).expect("manifest should parse");
        let (mut game, _) = load_developer_destination("preset-dev-sandbox")
            .expect("sandbox preset should load from JSON-backed synthetic room");

        game.zelda_run_frame(0);
        for _ in 0..30 {
            game.zelda_run_frame(1 << 7);
        }

        let base = game.ppu.bg_layer[0].tilemap_adr as usize;
        let source = load_developer_room_theme_source(room).expect("theme source should load");
        let source_base = source.ppu.bg_layer[DEVELOPER_ROOM_SOURCE_BG_LAYER].tilemap_adr as usize;
        for (row_index, row) in manifest.rows.iter().enumerate() {
            for (column_index, tile) in row.iter().enumerate() {
                let (sample_x, sample_y) = developer_room_kakariko_sample_origin(&source, *tile);
                for y_offset in 0..2 {
                    for x_offset in 0..2 {
                        let source_index =
                            source_base + (sample_y + y_offset) * 32 + sample_x + x_offset;
                        let index =
                            base + (row_index * 2 + y_offset) * 32 + column_index * 2 + x_offset;
                        assert_eq!(game.ppu.vram[index], source.ppu.vram[source_index]);
                    }
                }
            }
        }
    }

    #[test]
    fn developer_sandbox_owns_visible_background_layers() {
        let (mut game, _) = load_developer_destination("preset-dev-sandbox")
            .expect("sandbox preset should load from JSON-backed synthetic room");

        game.zelda_run_frame(0);

        assert_eq!(game.ppu.mode, 1);
        assert_eq!(game.ppu.screen_enabled[0] & 0x01, 0x01);
        assert_eq!(game.ppu.screen_enabled[0] & 0x06, 0x00);
        assert_eq!(game.ppu.screen_enabled[0] & 0x10, 0x10);
        assert_eq!(game.ppu.screen_enabled[1], 0x00);
        assert_eq!(game.ppu.bg_layer[0].tile_adr, DEVELOPER_ROOM_BG_TILE_BASE);
        assert_eq!(game.ram[TM_COPY], 0x11);
        assert_eq!(game.ram[TS_COPY], 0x00);
    }

    #[test]
    fn developer_sandbox_samples_kakariko_checkpoint_visuals() {
        let room = developer_destinations::synthetic_room("preset-dev-sandbox")
            .expect("sandbox room should be registered");
        let manifest = developer_sandbox_tilemap_manifest(room).expect("manifest should parse");
        let source = load_developer_room_theme_source(room).expect("theme source should load");
        let (game, _) = load_developer_destination("preset-dev-sandbox")
            .expect("sandbox preset should load from JSON-backed synthetic room");

        assert!((0x0018..=0x001b).contains(&read_le_u16(&source.ram, 0x8a)));
        assert_eq!(game.ppu.cgram, source.ppu.cgram);

        let destination_base = game.ppu.bg_layer[0].tilemap_adr as usize;
        let source_base = source.ppu.bg_layer[DEVELOPER_ROOM_SOURCE_BG_LAYER].tilemap_adr as usize;
        let first_grass = manifest.rows[1][1];
        let first_border = manifest.rows[0][0];
        let (grass_x, grass_y) = developer_room_kakariko_sample_origin(&source, first_grass);
        let (border_x, border_y) = developer_room_kakariko_sample_origin(&source, first_border);
        let grass_source_index = source_base + grass_y * 32 + grass_x;
        let border_source_index = source_base + border_y * 32 + border_x;
        let grass_destination_index = destination_base + 2 * 32 + 2;
        let border_destination_index = destination_base;

        assert_eq!(
            game.ppu.vram[grass_destination_index],
            source.ppu.vram[grass_source_index]
        );
        assert_eq!(
            game.ppu.vram[border_destination_index],
            source.ppu.vram[border_source_index]
        );
        let source_chr_base = source.ppu.bg_layer[DEVELOPER_ROOM_SOURCE_BG_LAYER].tile_adr as usize;
        let destination_chr_base = usize::from(DEVELOPER_ROOM_BG_TILE_BASE);
        let grass_tile = usize::from(source.ppu.vram[grass_source_index] & 0x03ff);
        let grass_source_chr = source_chr_base + grass_tile * 16;
        let grass_destination_chr = destination_chr_base + grass_tile * 16;
        assert_eq!(
            &game.ppu.vram[grass_destination_chr..grass_destination_chr + 16],
            &source.ppu.vram[grass_source_chr..grass_source_chr + 16]
        );
        assert_ne!(
            game.ppu.vram[grass_destination_index],
            game.ppu.vram[border_destination_index]
        );
        assert!(manifest.rows.iter().flatten().all(|tile| {
            let (sample_x, sample_y) = developer_room_kakariko_sample_origin(&source, *tile);
            source.ppu.vram[source_base + sample_y * 32 + sample_x] != 0
        }));
    }

    #[test]
    fn current_developer_location_reports_room_or_overworld_from_ram() {
        let mut ram = vec![0u8; 0x2000];
        ram[PLAYER_IS_INDOORS] = 1;
        ram[0x48e] = 0x50;
        ram[0x48f] = 0x00;
        let indoor = current_developer_location_from_ram(&ram, 12_000);
        assert_eq!(indoor.label, "CURRENT LOC");
        assert_eq!(indoor.location, "ROOM 0050");
        assert_eq!(indoor.detail, "FRAME 12000");

        ram[PLAYER_IS_INDOORS] = 0;
        ram[0x8a] = 0x1b;
        ram[0x8b] = 0x00;
        let overworld = current_developer_location_from_ram(&ram, 3_852);
        assert_eq!(overworld.location, "OW 001B");
        assert_eq!(
            overworld.thumbnail,
            platform::DeveloperThumbnail::LockedOverworld
        );
    }
}

fn run_replay_save(args: &[String]) {
    let (rom_path, replay_path) = match (args.first(), args.get(1)) {
        (Some(rom), Some(replay)) => (rom, replay),
        _ => {
            eprintln!(
                "usage: zelda3 --replay-save <path-to-rom.sfc> <replay.sav> [frames] [--dump-frame <out.png>] [--render-hash-log <stride>] [--audio-trace-log <stride>] [--gpu-render-compare <stride>] [--gpu-render-compare-quiet] [--modern-index-compare <stride>] [--require-full-gpu-path] [--require-modern-index-parity] [--render-hash-dump-frame <frame> <out.png>] [--input-script <path>] [--input-script-overlay <path>] [--stop-replay-after-load] [--save-state <checkpoint.sav>] [--load-state <checkpoint.sav>] [--load-sram <path>] [--fingerprint-log <path>] [--fingerprint-frame <frame>] [--coverage-log <path>]"
            );
            process::exit(2);
        }
    };
    let mut max_frames = u32::MAX;
    let mut dump_frame_path = None::<PathBuf>;
    let mut render_hash_log = 0u32;
    let mut audio_trace_log = 0u32;
    let mut fingerprint_log: Option<PathBuf> = None;
    let mut fingerprint_frame = None::<u32>;
    let mut coverage_log: Option<PathBuf> = None;
    let mut gpu_render_compare = gpu_render_compare_run(0, false);
    let mut modern_index_compare = modern_index_compare_run_from_env();
    let ppu_mode_summary = std::env::var("ZELDA3_PPU_MODE_SUMMARY").is_ok();
    let mut ppu_mode_counts = [0u64; 8];
    let mut first_mode7_frame = None::<u32>;
    let mut last_mode7_frame = None::<u32>;
    let mut render_hash_dump_frame = None::<(u32, PathBuf)>;
    let mut save_state_path = None::<PathBuf>;
    let mut save_state_at: Vec<(u32, PathBuf)> = Vec::new();
    let mut load_state_path = None::<PathBuf>;
    let mut load_sram_path = None::<PathBuf>;
    let mut input_script = InputScript::default();
    let mut input_script_overlay = None::<InputScript>;
    let mut stop_replay_after_load = false;
    let mut i = 2usize;
    if let Some(candidate) = args.get(i) {
        if !candidate.starts_with("--") {
            max_frames = candidate.parse::<u32>().unwrap_or_else(|_| {
                eprintln!("invalid frame count: {candidate}");
                process::exit(2);
            });
            i += 1;
        }
    }
    while i < args.len() {
        match args[i].as_str() {
            "--dump-frame" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--dump-frame requires a path");
                    process::exit(2);
                });
                dump_frame_path = Some(PathBuf::from(path));
                i += 2;
            }
            "--render-hash-log" => {
                let stride = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--render-hash-log requires a stride");
                    process::exit(2);
                });
                render_hash_log = stride.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid --render-hash-log stride: {stride}");
                    process::exit(2);
                });
                i += 2;
            }
            "--audio-trace-log" => {
                let stride = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--audio-trace-log requires a stride");
                    process::exit(2);
                });
                audio_trace_log = stride.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid --audio-trace-log stride: {stride}");
                    process::exit(2);
                });
                if audio_trace_log == 0 {
                    eprintln!("--audio-trace-log stride must be greater than zero");
                    process::exit(2);
                }
                i += 2;
            }
            "--gpu-render-compare" => {
                let stride = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--gpu-render-compare requires a stride");
                    process::exit(2);
                });
                let stride = stride.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid --gpu-render-compare stride: {stride}");
                    process::exit(2);
                });
                if !gpu_render_compare.set_stride(stride) {
                    eprintln!("--gpu-render-compare stride must be greater than zero");
                    process::exit(2);
                }
                i += 2;
            }
            "--gpu-render-compare-quiet" => {
                gpu_render_compare.set_quiet();
                i += 1;
            }
            "--modern-index-compare" => {
                let stride = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--modern-index-compare requires a stride");
                    process::exit(2);
                });
                let stride = stride.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid --modern-index-compare stride: {stride}");
                    process::exit(2);
                });
                if !modern_index_compare.set_stride(stride) {
                    eprintln!("--modern-index-compare stride must be greater than zero");
                    process::exit(2);
                }
                i += 2;
            }
            "--require-full-gpu-path" => {
                modern_index_compare.set_require_full_gpu_path();
                i += 1;
            }
            "--require-modern-index-parity" => {
                modern_index_compare.set_require_modern_index_parity();
                i += 1;
            }
            "--render-hash-dump-frame" => {
                let frame = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--render-hash-dump-frame requires a frame");
                    process::exit(2);
                });
                let path = args.get(i + 2).unwrap_or_else(|| {
                    eprintln!("--render-hash-dump-frame requires a path");
                    process::exit(2);
                });
                let frame = frame.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid --render-hash-dump-frame frame: {frame}");
                    process::exit(2);
                });
                render_hash_dump_frame = Some((frame, PathBuf::from(path)));
                i += 3;
            }
            "--save-state" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--save-state requires a path");
                    process::exit(2);
                });
                save_state_path = Some(PathBuf::from(path));
                i += 2;
            }
            "--save-state-at" => {
                let Some(spec) = args.get(i + 1) else {
                    eprintln!("--save-state-at <frame>:<path>");
                    process::exit(2);
                };
                let (f, path) = spec.split_once(':').unwrap_or_else(|| {
                    eprintln!("--save-state-at <frame>:<path>");
                    process::exit(2);
                });
                let frame = f.parse().unwrap_or_else(|_| {
                    eprintln!("--save-state-at <frame>:<path>: invalid frame number");
                    process::exit(2);
                });
                save_state_at.push((frame, PathBuf::from(path)));
                i += 2;
            }
            "--load-state" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--load-state requires a path");
                    process::exit(2);
                });
                load_state_path = Some(PathBuf::from(path));
                i += 2;
            }
            "--load-sram" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--load-sram requires a path");
                    process::exit(2);
                });
                load_sram_path = Some(PathBuf::from(path));
                i += 2;
            }
            "--input-script" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--input-script requires a path");
                    process::exit(2);
                });
                input_script = match InputScript::from_path(Path::new(path)) {
                    Ok(script) => script,
                    Err(e) => {
                        eprintln!("failed to parse input script {}: {e}", path);
                        process::exit(2);
                    }
                };
                i += 2;
            }
            "--input-script-overlay" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--input-script-overlay requires a path");
                    process::exit(2);
                });
                input_script_overlay = Some(match InputScript::from_path(Path::new(path)) {
                    Ok(script) => script,
                    Err(e) => {
                        eprintln!("failed to parse input script overlay {}: {e}", path);
                        process::exit(2);
                    }
                });
                i += 2;
            }
            "--stop-replay-after-load" => {
                stop_replay_after_load = true;
                i += 1;
            }
            "--fingerprint-log" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--fingerprint-log requires a path");
                    process::exit(2);
                };
                fingerprint_log = Some(PathBuf::from(path));
                i += 2;
            }
            "--fingerprint-frame" => {
                let Some(frame) = args.get(i + 1) else {
                    eprintln!("--fingerprint-frame requires a frame");
                    process::exit(2);
                };
                fingerprint_frame = Some(frame.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid --fingerprint-frame frame: {frame}");
                    process::exit(2);
                }));
                i += 2;
            }
            "--coverage-log" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("--coverage-log requires a path");
                    process::exit(2);
                };
                coverage_log = Some(PathBuf::from(path));
                i += 2;
            }
            flag => {
                eprintln!("unknown --replay-save option: {flag}");
                process::exit(2);
            }
        }
    }

    if let Err(e) = modern_index_compare.validate() {
        eprintln!("{e}");
        process::exit(2);
    }

    if load_state_path.is_some() && load_sram_path.is_some() {
        eprintln!(
            "--load-sram cannot be combined with --load-state; checkpoints already include SRAM"
        );
        process::exit(2);
    }

    let last_panic = install_crash_panic_hook();
    let mut game = load_translated_replay_state(rom_path);
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
        if let Err(e) = game.replay_save_file(Path::new(replay_path)) {
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
        || !input_script.rules.is_empty()
        || input_script_overlay
            .as_ref()
            .is_some_and(|script| !script.rules.is_empty());
    let mut audio_trace_buffer = if audio_trace_log != 0 || fingerprint_log.is_some() {
        Some(vec![0i16; 735 * 2])
    } else {
        None
    };
    let mut render_hash_frame = if render_hash_log != 0
        || gpu_render_compare.enabled()
        || render_hash_dump_frame.is_some()
        || fingerprint_log.is_some()
    {
        Some(vec![0u8; 256 * 224 * 4])
    } else {
        None
    };
    // GPU readback is used for dump-frame and the diagnostic gpu-render-hash
    // line. The parity-facing render-hash line hashes the raw CPU BGRA display
    // buffer, matching C PrintRenderHash exactly.
    let mut gpu_readback = replay_optional_gpu_readback_renderer(
        render_hash_log,
        &gpu_render_compare,
        render_hash_dump_frame.is_some(),
        dump_frame_path.is_some(),
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
    // Off-VRAM atlas paths: unset now uses `assets-variant-gpu`; `assets-anim-gpu`
    // remains the full indexed GPU fallback when `ZELDA3_VARIANT_ATLAS=off` or an
    // explicit renderer env selects it. Explicit `assets-anim` keeps the CPU atlas
    // compositor as an opt-out/debug oracle. `assets-variant-gpu` uses compact
    // base art plus LUT effects for stable draws and reports fallback counts.
    modern_index_compare
        .load_resources(Path::new("."), true)
        .unwrap_or_else(|e| {
            eprintln!("modern index compare resources load failed: {e}");
            process::exit(2);
        });
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
        last_frame_had_fingerprint_render = false;
        let mut fp_audio_leaf: u32 = 0;
        let should_fingerprint_frame =
            fingerprint_log.is_some() && should_write_fingerprint(fingerprint_frame, frames);
        if let Some(audio) = audio_trace_buffer.as_mut() {
            let dsp_pre = game.zelda_audio_dsp_hash();
            let writes = game.zelda_render_audio_trace_dsp(audio, 735, 2);
            game.zelda_discard_unused_audio_frames();
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
                print_replay_audio_trace(frames, &game, audio, 735, 2, dsp_pre, &writes);
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
        if should_log_render_hash || should_dump_render_hash {
            let width = 256u32;
            let frame = render_hash_frame
                .as_mut()
                .expect("render hash frame allocated");
            // Run HDMA channel 6+7 for one line to load CGRAM entries that ALttP sets
            // via HDMA (e.g. dungeon floor palettes). State is restored after the call
            // so zelda_draw_ppu_frame renders from the correct baseline.
            let gpu_capture = capture_gpu_frame_from_game(&mut game);
            let hdma_cgram = gpu_capture.cgram();
            let scanlines_raw = gpu_capture.raw_scanlines();
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
                    "[gpu-dbg] f800 scanlines[60..70] screen_enabled_main: {:?}",
                    &scanlines_raw[60..70]
                        .iter()
                        .map(|e| e.4)
                        .collect::<Vec<_>>()
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
                let cgram_val = hdma_cgram.get(cgram_idx).copied().unwrap_or(0);
                eprintln!(
                    "[gpu-dbg] f800 BG1 at (126,65): cgram_idx={} cgram_val={:#06x}",
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
            render_play_frame_bgra(
                &mut game,
                frame,
                width as usize * 4,
                PpuRenderFlags::empty(),
            );
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
                let cgram_val = hdma_cgram.get(cgram_idx).copied().unwrap_or(0);
                eprintln!(
                    "[gpu-dbg] f800 BG1@(126,65) POST-RENDER: tilemap_adr={} tile_adr={} entry={:#06x} tile={} pal_sub={} pal_idx={} cgram[{}]={:#06x}",
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
            let rgba = gpu_readback.render_cpu_bgra_frame_rgba(frame);
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
                let post_cgram = &game.ppu.cgram;
                let diffs: Vec<(usize, u16, u16)> = hdma_cgram
                    .iter()
                    .enumerate()
                    .zip(post_cgram.iter())
                    .filter(|((_, &h), &p)| h != p)
                    .map(|((i, &h), &p)| (i, h, p))
                    .collect();
                eprintln!(
                    "[gpu-dbg] frame=1000 CGRAM changes during render: {} entries differ",
                    diffs.len()
                );
                for (i, before, after) in diffs.iter().take(20) {
                    eprintln!("[gpu-dbg]   cgram[{i}]: {before:#06x} → {after:#06x}");
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
                    if let Err(e) = write_rgba_frame_png(dump_path, &rgba, width, 224) {
                        eprintln!("failed to write {}: {e}", dump_path.display());
                        process::exit(1);
                    }
                    println!("dumped replay-save frame to {}", dump_path.display());
                    let gpu_rgba = gpu_readback.render_live_gpu_capture_rgba(&gpu_capture);
                    let gpu_path = {
                        let stem = dump_path.file_stem().unwrap_or_default().to_string_lossy();
                        let ext = dump_path
                            .extension()
                            .map(|e| e.to_string_lossy().into_owned())
                            .unwrap_or_default();
                        dump_path.with_file_name(if ext.is_empty() {
                            format!("{stem}.gpu")
                        } else {
                            format!("{stem}.gpu.{ext}")
                        })
                    };
                    if let Err(e) = write_rgba_frame_png(&gpu_path, &gpu_rgba, width, 224) {
                        eprintln!("failed to write {}: {e}", gpu_path.display());
                        process::exit(1);
                    }
                    println!("dumped GPU frame to {}", gpu_path.display());
                }
            }
            if should_log_render_hash {
                println!("{}", render_hash_frame_bgra_line(frames, frame));
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
                                    let dw01 = game.ppu.vram.get(dbase + dty).copied().unwrap_or(0);
                                    let dw23 =
                                        game.ppu.vram.get(dbase + 8 + dty).copied().unwrap_or(0);
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
                let gpu_frame = gpu_capture.gpu_frame();
                if frames == 8000 {
                    eprintln!(
                        "[gpu-dbg] math_enabled={:#04x} subtract={} half={} fixed_rgb=({},{},{}) add_sub={} clip_mode={} prevent_math={} windowsel_cm={:#04x} brightness={}",
                        gpu_frame.math_enabled,
                        gpu_frame.subtract_color,
                        gpu_frame.half_color,
                        gpu_frame.fixed_color_r,
                        gpu_frame.fixed_color_g,
                        gpu_frame.fixed_color_b,
                        gpu_frame.add_subscreen,
                        gpu_frame.clip_mode,
                        gpu_frame.prevent_math_mode,
                        gpu_frame.windowsel_cm,
                        gpu_frame.brightness
                    );
                    eprintln!(
                        "[gpu-dbg] ppu.math_enabled={:#04x} ppu.add_subscreen={} ppu.subtract={} ppu.prevent_math_mode={}",
                        game.ppu.math_enabled,
                        game.ppu.add_subscreen,
                        game.ppu.subtract_color,
                        game.ppu.prevent_math_mode
                    );
                }
                let gpu_rgba = gpu_readback.render_live_gpu_capture_rgba(&gpu_capture);
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
                    let gf = gpu_capture.gpu_frame();
                    eprintln!(
                        "[gpu-dbg] frame=332 math_enabled={:#04x} subtract={} half={} fixed=({},{},{}) clip_mode={} prevent_math={} windowsel_cm={:#04x} add_sub={}",
                        gf.math_enabled,
                        gf.subtract_color,
                        gf.half_color,
                        gf.fixed_color_r,
                        gf.fixed_color_g,
                        gf.fixed_color_b,
                        gf.clip_mode,
                        gf.prevent_math_mode,
                        gf.windowsel_cm,
                        gf.add_subscreen
                    );
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
                        "[gpu-dbg] frame=332 scanline[0]: w1l={} w1r={}",
                        gf.scanlines[0].window1_left, gf.scanlines[0].window1_right
                    );
                    // Find CGRAM entry = 0x014D (the mystery color R=13,G=10,B=0)
                    for (ci, &cv) in hdma_cgram.iter().enumerate() {
                        if cv == 0x014d {
                            eprintln!("[gpu-dbg] frame=332 hdma_cgram[{ci}]=0x014d");
                        }
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
                    let gf = gpu_capture.gpu_frame();
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
                        "[gpu-dbg] f{frames} math={:#04x} add_sub={} subtract={} half={} fixed_r={} fixed_g={} fixed_b={} bg1_hscroll={} irq_flag={}",
                        gf.math_enabled,
                        gf.add_subscreen,
                        gf.subtract_color,
                        gf.half_color,
                        gf.fixed_color_r,
                        gf.fixed_color_g,
                        gf.fixed_color_b,
                        game.ppu.bg_layer[0].h_scroll,
                        game.ram[0xf9]
                    );
                    if frames == 900 || frames == 1050 {
                        let (cx, cy) = match frames {
                            900 => (127i32, 56i32),
                            1050 => (40i32, 40i32),
                            _ => unreachable!(),
                        };
                        eprintln!("[gpu-dbg] f{frames} probe ({cx},{cy})");
                        eprintln!(
                            "[gpu-dbg] f{frames} scanline_tm row{}={:#04x} row{}={:#04x}",
                            cy,
                            gf.scanlines[cy as usize].screen_enabled_main,
                            cy + 1,
                            gf.scanlines[(cy + 1) as usize].screen_enabled_main
                        );
                        let ppu_x = (cx + PPU_EXTRA_LEFT_RIGHT as i32) as usize;
                        let main_z = game.ppu.bg_buffers[0].data.get(ppu_x).copied().unwrap_or(0);
                        let sub_z = game.ppu.bg_buffers[1].data.get(ppu_x).copied().unwrap_or(0);
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
                            let entry = game.ppu.vram.get(vram_idx as usize).copied().unwrap_or(0);
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
                                let pal_idx = ((w01 >> bit) & 1) | (((w01 >> (8 + bit)) & 1) << 1);
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
                                let pal_idx = ((w01 >> bit) & 1) | (((w01 >> (8 + bit)) & 1) << 1);
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
                                "[gpu-dbg] f{frames} BG{}@({cx},{cy}): enabled_main={} tilemap={} tile_adr={} entry={:#06x} tile={} pal_sub={} prio={} px={} py={} pal_idx={} cgram[{}]={:#06x}",
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
                                hdma_cgram.get(cgram_idx as usize).copied().unwrap_or(0)
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
                            let hi_word = game.ppu.oam.get(0x100 + idx / 16).copied().unwrap_or(0);
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
                                    .unwrap_or(0) as u32)
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
                                "[gpu-dbg] f{frames} sprite#{} covers ({cx},{cy}): x={} y_base={} size={} oam1={:#06x} prio={} pal_sub={} row={} col={} used_tile={:#04x} pixel={} cgram[{}]={:#06x}",
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
                                hdma_cgram.get(cgram_idx as usize).copied().unwrap_or(0)
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
                        let bg3_pal_idx =
                            ((bg3_w01 >> bg3_bit) & 1) | (((bg3_w01 >> (8 + bg3_bit)) & 1) << 1);
                        let bg3_cgram_idx = (bg3_pal_sub * 4 + bg3_pal_idx) as usize;
                        let bg3_cgram_val = hdma_cgram.get(bg3_cgram_idx).copied().unwrap_or(0);
                        eprintln!(
                            "[gpu-dbg] f800 BG3@(126,65): tilemap_adr={} tile={} pal_sub={} prio={} pal_idx={} cgram[{}]={:#06x}",
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
                        let gpu_pal_idx =
                            ((gpu_w_lo >> gpu_bit) & 1) | (((gpu_w_lo >> (8 + gpu_bit)) & 1) << 1);
                        let gpu_bg3_cgram = bg3_pal_sub * 4 + gpu_pal_idx; // 2bpp: 4 colors per sub-palette
                        eprintln!(
                            "[gpu-dbg] f800 GPU BG3@(126,65): atlas_slot={} sub={} gpu_pal_idx={} gpu_cgram_idx={}",
                            bg3_atlas_slot, bg3_atlas_sub, gpu_pal_idx, gpu_bg3_cgram
                        );
                        // Also log the specific GPU cgram value
                        let gpu_bg3_cgram_val =
                            hdma_cgram.get(gpu_bg3_cgram as usize).copied().unwrap_or(0);
                        eprintln!(
                            "[gpu-dbg] f800 GPU BG3 cgram[{}]={:#06x}",
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
                            let hi_word = game.ppu.oam.get(0x100 + idx / 16).copied().unwrap_or(0);
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
                    let hashes = gpu_rgba.hash_pair_with_cpu_bgra(frame);
                    eprintln!(
                        "[gpu-dbg] frame=332 cpu_hash={:#010x} gpu_hash={:#010x}",
                        hashes.cpu_hash, hashes.gpu_hash
                    );
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
                println!("{}", gpu_rgba.render_hash_line(frames));
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
            render_standard_play_frame_bgra(&mut game, frame);
            last_frame_had_fingerprint_render = true;
            fp_render_leaf = render_fingerprint_leaf_bgra(frame);
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
                render_standard_play_frame_bgra(&mut game, &mut scratch);
                last_frame_had_fingerprint_render = true;
            }
            write_checkpoint(&mut game, frames, path);
        }
    }

    modern_index_compare.emit_summary_line_if_enabled();
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
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!(
                    "failed to create coverage log directory {}: {e}",
                    parent.display()
                );
                process::exit(1);
            }
        }
        let json = serde_json::to_vec_pretty(coverage).unwrap_or_else(|e| {
            eprintln!("failed to encode coverage log: {e}");
            process::exit(1);
        });
        if let Err(e) = std::fs::write(path, json) {
            eprintln!("failed to write coverage log {}: {e}", path.display());
            process::exit(1);
        }
    }

    if let Some(path) = save_state_path.as_deref() {
        if !last_frame_had_fingerprint_render {
            let mut scratch = vec![0u8; 256 * 224 * 4];
            render_standard_play_frame_bgra(&mut game, &mut scratch);
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
        let mut frame = vec![0u8; width as usize * height as usize * 4];
        let mut render_game = game.clone();
        render_play_frame_bgra(
            &mut render_game,
            &mut frame,
            width as usize * 4,
            PpuRenderFlags::empty(),
        );
        let rgba = gpu_readback.render_cpu_bgra_frame_rgba(&frame);
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

fn load_replay_save_checkpoint(game: &mut ZeldaState, path: &Path) -> std::io::Result<()> {
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
    let mut file = fs::File::create(path)?;
    let mut state_recorder = std::mem::take(&mut game.state_recorder);
    game.state_recorder_save(&mut state_recorder, &mut file);
    game.state_recorder = state_recorder;
    file.write_all(&AUDIO_TRAILER_MAGIC)?;
    write_trailer_blob(&mut file, &audio_bytes)?;
    write_trailer_blob(&mut file, &hdma_dyn_bytes)?;
    write_trailer_blob(&mut file, &hdma_scratch_bytes)?;
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

fn replay_save_ancilla_dump(game: &ZeldaState) -> String {
    let mut out = String::from("ancilla");
    for k in 0..10 {
        if game.ram[0x0c4a + k] == 0 && game.ram[0x0c5e + k] == 0 && game.ram[0x03b1 + k] == 0 {
            continue;
        }
        out.push_str(&format!(
            " [{k}:t=0x{:02x} x=0x{:04x} y=0x{:04x} xv=0x{:02x} yv=0x{:02x} step=0x{:02x} aux=0x{:02x} item=0x{:02x} arr3=0x{:02x} floor=0x{:02x} floor2=0x{:02x}]",
            game.ram[0x0c4a + k],
            u16::from_le_bytes([game.ram[0x0c04 + k], game.ram[0x0c18 + k]]),
            u16::from_le_bytes([game.ram[0x0bfa + k], game.ram[0x0c0e + k]]),
            game.ram[0x0c2c + k],
            game.ram[0x0c22 + k],
            game.ram[0x0c54 + k],
            game.ram[0x03b1 + k],
            game.ram[0x0c5e + k],
            game.ram[0x039f + k],
            game.ram[0x0c7c + k],
            game.ram[0x03ca + k],
        ));
    }
    out
}

fn replay_save_ram_page_dump(game: &ZeldaState) -> String {
    let mut out = String::from("ram-pages");
    for page in 0..128usize {
        let start = page * 0x400;
        out.push_str(&format!(
            " [{start:05x}=0x{:08x}]",
            replay_checksum_ram_range(&game.ram, start, 0x400)
        ));
    }
    out
}

fn replay_save_ram0400_dump(game: &ZeldaState) -> String {
    let mut out = String::from("ram0400");
    for index in 0x400..0x800 {
        let byte = game.ram[index];
        if byte != 0 {
            out.push_str(&format!(" [{index:04x}=0x{byte:02x}]"));
        }
    }
    out
}

fn replay_save_ram0000_dump(game: &ZeldaState) -> String {
    let mut out = String::from("ram0000");
    for index in 0x000..0x400 {
        let byte = game.ram[index];
        if byte != 0 {
            out.push_str(&format!(" [{index:04x}=0x{byte:02x}]"));
        }
    }
    out
}

fn replay_save_requested_ram_page_dump(game: &ZeldaState) -> Option<String> {
    let raw = std::env::var("ZELDA3_REPLAY_RAM_DUMP_PAGE").ok()?;
    let parsed = raw
        .strip_prefix("0x")
        .or_else(|| raw.strip_prefix("0X"))
        .map_or_else(
            || raw.parse::<usize>().ok(),
            |hex| usize::from_str_radix(hex, 16).ok(),
        )?;
    let start = parsed & !0x3ff;
    if start >= game.ram.len() {
        return None;
    }
    let end = (start + 0x400).min(game.ram.len());
    let mut out = format!("ram-page-bytes page=0x{start:05x}");
    for index in start..end {
        let byte = game.ram[index];
        if byte != 0 {
            out.push_str(&format!(" [{index:05x}=0x{byte:02x}]"));
        }
    }
    Some(out)
}

fn replay_save_room_mask(game: &ZeldaState, room: u16) -> u16 {
    let offset = 0x1df80 + usize::from(room) * 2;
    u16::from_le_bytes([game.ram[offset], game.ram[offset + 1]])
}

fn replay_save_room_history(game: &ZeldaState) -> [u16; 4] {
    [
        u16::from_le_bytes([game.ram[0xb80], game.ram[0xb81]]),
        u16::from_le_bytes([game.ram[0xb82], game.ram[0xb83]]),
        u16::from_le_bytes([game.ram[0xb84], game.ram[0xb85]]),
        u16::from_le_bytes([game.ram[0xb86], game.ram[0xb87]]),
    ]
}

fn replay_save_garnish_dump(game: &ZeldaState) -> String {
    const GARNISH_TYPE: usize = 0x1f800;
    const GARNISH_Y_LO: usize = 0x1f81e;
    const GARNISH_X_LO: usize = 0x1f83c;
    const GARNISH_Y_HI: usize = 0x1f85a;
    const GARNISH_X_HI: usize = 0x1f878;
    const GARNISH_Y_VEL: usize = 0x1f896;
    const GARNISH_X_VEL: usize = 0x1f8b4;
    const GARNISH_COUNTDOWN: usize = 0x1f90e;
    const GARNISH_SPRITE: usize = 0x1f92c;
    const GARNISH_FLOOR: usize = 0x1f968;
    const GARNISH_OAM_FLAGS: usize = 0x1f9fe;

    let mut out = String::from("garnish");
    for k in 0..30 {
        if game.ram[GARNISH_TYPE + k] == 0 && game.ram[GARNISH_COUNTDOWN + k] == 0 {
            continue;
        }
        out.push_str(&format!(
            " [{k}:t=0x{:02x} cd=0x{:02x} x=0x{:04x} y=0x{:04x} xv=0x{:02x} yv=0x{:02x} spr=0x{:02x} floor=0x{:02x} oam=0x{:02x}]",
            game.ram[GARNISH_TYPE + k],
            game.ram[GARNISH_COUNTDOWN + k],
            u16::from_le_bytes([game.ram[GARNISH_X_LO + k], game.ram[GARNISH_X_HI + k]]),
            u16::from_le_bytes([game.ram[GARNISH_Y_LO + k], game.ram[GARNISH_Y_HI + k]]),
            game.ram[GARNISH_X_VEL + k],
            game.ram[GARNISH_Y_VEL + k],
            game.ram[GARNISH_SPRITE + k],
            game.ram[GARNISH_FLOOR + k],
            game.ram[GARNISH_OAM_FLAGS + k],
        ));
    }
    out
}

fn replay_save_room_history_dump(game: &ZeldaState) -> String {
    let mut out = String::from("room-history");
    for (k, room) in replay_save_room_history(game).into_iter().enumerate() {
        let mask = if room == 0xffff {
            0
        } else {
            replay_save_room_mask(game, room)
        };
        out.push_str(&format!(" [{k}:room=0x{room:04x} mask=0x{mask:04x}]"));
    }
    out
}

fn replay_save_room_mask_dump(game: &ZeldaState) -> String {
    let current_room = u16::from_le_bytes([game.ram[0x48e], game.ram[0x48f]]);
    let mut out = format!(
        "room-masks current=0x{:04x} current_room=0x{:04x}",
        replay_save_room_mask(game, current_room),
        current_room
    );
    for room in replay_save_room_history(game) {
        if room != 0xffff {
            out.push_str(&format!(
                " [room=0x{room:04x} mask=0x{:04x}]",
                replay_save_room_mask(game, room)
            ));
        }
    }
    out
}

fn replay_save_overlord_dump(game: &ZeldaState) -> String {
    let mut out = String::from("overlords");
    for k in 0..8 {
        if game.ram[0x0b00 + k] == 0 {
            continue;
        }
        out.push_str(&format!(
            " [{k}:t=0x{:02x} x=0x{:04x} y=0x{:04x} floor=0x{:02x} gen1=0x{:02x} gen2=0x{:02x}]",
            game.ram[0x0b00 + k],
            u16::from_le_bytes([game.ram[0x0b08 + k], game.ram[0x0b10 + k]]),
            u16::from_le_bytes([game.ram[0x0b18 + k], game.ram[0x0b20 + k]]),
            game.ram[0x0b40 + k],
            game.ram[0x0b28 + k],
            game.ram[0x0b30 + k],
        ));
    }
    out
}

fn replay_save_sprite_dump(game: &ZeldaState) -> String {
    let mut out = String::from("sprites");
    for k in 0..16 {
        if game.ram[0x0dd0 + k] == 0 && game.ram[0x0e20 + k] == 0 {
            continue;
        }
        out.push_str(&format!(
            " [{k}:t=0x{:02x} st=0x{:02x} ai=0x{:02x} head=0x{:02x} sub=0x{:02x} x=0x{:04x} y=0x{:04x} d=0x{:02x} c=0x{:02x} e=0x{:02x} f=0x{:02x} n=0x{:04x} delay=0x{:02x} bump=0x{:02x} hp=0x{:02x} hit=0x{:02x} give=0x{:02x}]",
            game.ram[0x0e20 + k],
            game.ram[0x0dd0 + k],
            game.ram[0x0d80 + k],
            game.ram[0x0eb0 + k],
            game.ram[0x0e80 + k],
            u16::from_le_bytes([game.ram[0x0d10 + k], game.ram[0x0d30 + k]]),
            u16::from_le_bytes([game.ram[0x0d00 + k], game.ram[0x0d20 + k]]),
            game.ram[0x0de0 + k],
            game.ram[0x0db0 + k],
            game.ram[0x0e90 + k],
            game.ram[0x0ea0 + k],
            u16::from_le_bytes([game.ram[0x0bc0 + k * 2], game.ram[0x0bc0 + k * 2 + 1]]),
            game.ram[0x0df0 + k],
            game.ram[0x0cd2 + k],
            game.ram[0x0e50 + k],
            game.ram[0x0ef0 + k],
            game.ram[0x0ce2 + k],
        ));
    }
    out
}

fn replay_save_door_dump(game: &ZeldaState) -> String {
    let mut out = format!(
        "doors opened=0x{:04x} opened_adj=0x{:04x} cur=0x{:04x} toggles={:04x}/{:04x} floor={:04x},{:04x} palace={:04x},{:04x} exit_count=0x{:04x} exits={:04x},{:04x},{:04x},{:04x}",
        u16::from_le_bytes([game.ram[0x400], game.ram[0x401]]),
        u16::from_le_bytes([game.ram[0x68c], game.ram[0x68d]]),
        u16::from_le_bytes([game.ram[0x68e], game.ram[0x68f]]),
        u16::from_le_bytes([game.ram[0x44e], game.ram[0x44f]]),
        u16::from_le_bytes([game.ram[0x450], game.ram[0x451]]),
        u16::from_le_bytes([game.ram[0x6c0], game.ram[0x6c1]]),
        u16::from_le_bytes([game.ram[0x6c2], game.ram[0x6c3]]),
        u16::from_le_bytes([game.ram[0x6d0], game.ram[0x6d1]]),
        u16::from_le_bytes([game.ram[0x6d2], game.ram[0x6d3]]),
        u16::from_le_bytes([game.ram[0x19e0], game.ram[0x19e1]]),
        u16::from_le_bytes([game.ram[0x19e2], game.ram[0x19e3]]),
        u16::from_le_bytes([game.ram[0x19e4], game.ram[0x19e5]]),
        u16::from_le_bytes([game.ram[0x19e6], game.ram[0x19e7]]),
        u16::from_le_bytes([game.ram[0x19e8], game.ram[0x19e9]]),
    );
    for k in 0..16 {
        let addr = u16::from_le_bytes([game.ram[0x19a0 + k * 2], game.ram[0x19a1 + k * 2]]);
        let kind = u16::from_le_bytes([game.ram[0x1980 + k * 2], game.ram[0x1981 + k * 2]]);
        let dir = u16::from_le_bytes([game.ram[0x19c0 + k * 2], game.ram[0x19c1 + k * 2]]);
        if addr != 0 || kind != 0 || dir != 0 {
            out.push_str(&format!(
                " [{k}:type=0x{kind:04x} addr=0x{addr:04x} dir=0x{dir:04x}]"
            ));
        }
    }
    out
}

fn replay_save_dungeon_attr_dump(game: &ZeldaState) -> String {
    const DUNG_BG2_ATTR_TABLE: usize = 0x12000;
    let target = std::env::var("ZELDA3_REPLAY_DUNGEON_ATTR_POS")
        .ok()
        .and_then(|value| {
            value
                .strip_prefix("0x")
                .or_else(|| value.strip_prefix("0X"))
                .and_then(|hex| usize::from_str_radix(hex, 16).ok())
                .or_else(|| value.parse::<usize>().ok())
        })
        .unwrap_or(0x05fb);
    let base = target.saturating_sub(2);
    let mut out = format!("dungeon-attrs target=0x{target:04x}");
    for pos in base..(base + 5).min(0x2000) {
        out.push_str(&format!(
            " [0x{pos:04x}=0x{:02x}]",
            game.ram[DUNG_BG2_ATTR_TABLE + pos]
        ));
    }
    out
}

fn replay_save_dungmap_dump(game: &ZeldaState) -> String {
    const DUNG_MAP_TAB5: [u16; 14] = [
        0x21, 0x23, 0x20, 0x21, 0x70, 0x12, 0x11, 0x212, 2, 0x217, 0x160, 0x12, 0x113, 0x171,
    ];
    const DUNG_MAP_TAB21: [u16; 3] = [137, 167, 79];
    const DUNG_MAP_TAB22: [u16; 3] = [169, 119, 190];

    let palace = read_le_u16(&game.ram, 0x040c);
    let raw_dung = usize::from(palace >> 1);
    let valid_dung = raw_dung < DUNG_MAP_TAB5.len();
    let dung = if valid_dung {
        raw_dung
    } else {
        DUNG_MAP_TAB5.len() - 1
    };
    let t5 = if valid_dung {
        (DUNG_MAP_TAB5[dung] & 0x0f) as u8
    } else {
        0
    };
    let floor1 = t5.wrapping_add(game.ram[0x00a4]);
    let mut room = read_le_u16(&game.ram, 0x00a0);
    for i in 0..3 {
        if room == DUNG_MAP_TAB21[i] {
            room = DUNG_MAP_TAB22[i];
        }
    }
    let layout = if valid_dung {
        game.replay_asset_memblk_bytes(97, raw_dung)
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let base = usize::from(floor1) * 25;
    let mut found = -1i32;
    let mut layout_bytes = String::new();
    for i in 0..25 {
        let value = layout.get(base + i).copied().unwrap_or(0x0f);
        if i != 0 {
            layout_bytes.push(',');
        }
        layout_bytes.push_str(&format!("{value:02x}"));
        if found < 0 && value == room as u8 {
            found = i as i32;
        }
    }
    format!(
        "dungmap state={} init={} floor=0x{:04x} idx=0x{:04x} palace=0x{:04x} room=0x{:04x} link=0x{:04x}/0x{:04x} dung={} t5=0x{:02x} floor1=0x{:02x} found={} vars=0x{:02x},0x{:04x},0x{:04x},0x{:04x},0x{:04x},0x{:04x},0x{:04x} layout={}",
        game.ram[0x0200],
        game.ram[0x020d],
        read_le_u16(&game.ram, 0x020e),
        read_le_u16(&game.ram, 0x0211),
        read_le_u16(&game.ram, 0x040c),
        read_le_u16(&game.ram, 0x00a0),
        read_le_u16(&game.ram, 0x0022),
        read_le_u16(&game.ram, 0x0020),
        raw_dung,
        t5,
        floor1,
        found,
        game.ram[0x0210],
        read_le_u16(&game.ram, 0x0215),
        read_le_u16(&game.ram, 0x0213),
        read_le_u16(&game.ram, 0x0217),
        read_le_u16(&game.ram, 0x0cf5),
        read_le_u16(&game.ram, 0x0fa8),
        read_le_u16(&game.ram, 0x0faa),
        layout_bytes,
    )
}

fn replay_save_message_dump(game: &ZeldaState) -> String {
    let read_pos = read_le_u16(&game.ram, 0x1cd9) as usize;
    let mut bytes = String::new();
    for k in 0..8 {
        if k != 0 {
            bytes.push(',');
        }
        let index = 0x11200 + read_pos + k;
        let byte = game.ram.get(index).copied().unwrap_or(0);
        bytes.push_str(&format!("{byte:02x}"));
    }
    format!(
        "message msgmod={} msg=0x{:04x} read=0x{:04x} state=0x{:02x} wait=0x{:04x}/0x{:02x} speed=0x{:02x}/0x{:02x} bytes={}",
        game.ram[0x1cd8],
        read_le_u16(&game.ram, 0x1cf0),
        read_pos,
        game.ram[0x1cd4],
        read_le_u16(&game.ram, 0x1ce0),
        game.ram[0x1ce9],
        game.ram[0x1cd5],
        game.ram[0x1cd6],
        bytes,
    )
}

fn replay_save_palette_dump(game: &ZeldaState) -> String {
    let mut words = String::new();
    for k in 0..8 {
        if k != 0 {
            words.push(',');
        }
        words.push_str(&format!(
            "{:04x}/{:04x}",
            read_le_u16(&game.ram, 0x0c300 + k * 2),
            read_le_u16(&game.ram, 0x0c500 + k * 2),
        ));
    }
    let armor = game.ram[0x0f35b];
    let gloves = game.ram[0x0f354];
    let armor_word = usize::from(armor) * 15 + 12;
    let armorfd = game.replay_asset_word(81, armor_word).unwrap_or(0xffff);
    let gloveclr0 = game.replay_gloves_color(0);
    let gloveclr1 = game.replay_gloves_color(1);
    format!(
        "palette aux=0x{:08x} main=0x{:08x} flag=0x{:02x} filter=0x{:04x} auxmain=0x{:04x} mainind=0x{:02x} sp0=0x{:02x} sp5=0x{:02x} sp6=0x{:02x} sp6r=0x{:02x} hud=0x{:02x} owmode=0x{:02x} sword=0x{:02x} shield=0x{:02x} armor=0x{:02x} gloves=0x{:02x} palfd={:04x}/{:04x} armorfd={:04x} gloveclr={:04x}/{:04x} words={}",
        replay_checksum_ram_range(&game.ram, 0x0c300, 0x200),
        replay_checksum_ram_range(&game.ram, 0x0c500, 0x200),
        game.ram[0x0015],
        read_le_u16(&game.ram, 0x0c007),
        read_le_u16(&game.ram, 0x0aa8),
        game.ram[0x0ab6],
        game.ram[0x0aac],
        game.ram[0x0aad],
        game.ram[0x0aae],
        game.ram[0x0ab1],
        game.ram[0x0ab2],
        game.ram[0x0ab3],
        game.ram[0x0f359],
        game.ram[0x0f35a],
        armor,
        gloves,
        read_le_u16(&game.ram, 0x0c300 + 0xfd * 2),
        read_le_u16(&game.ram, 0x0c500 + 0xfd * 2),
        armorfd,
        gloveclr0,
        gloveclr1,
        words,
    )
}

fn print_replay_save_panic_report(game: &ZeldaState, frames: u32, panic_info: &CapturedPanic) {
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
struct CapturedPanic {
    message: String,
    location: String,
    backtrace: String,
}

fn install_crash_panic_hook() -> Arc<Mutex<Option<CapturedPanic>>> {
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

fn captured_panic_from(
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

fn write_play_crash_report(
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

fn load_play_state(rom_path: &str) -> ZeldaState {
    load_game_state(rom_path, true)
}

fn load_embedded_play_state() -> ZeldaState {
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

fn load_translated_replay_state(rom_path: &str) -> ZeldaState {
    load_game_state(rom_path, false)
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

fn load_play_or_checkpoint(rom_path: &str, load_state: Option<&Path>) -> (ZeldaState, u32) {
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
    let mut frame = vec![0u8; 256 * 224 * 4];
    let render_flags = PpuRenderFlags::empty();
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
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            run_play_frame_with_run_what_bgra(&mut game, input, run_what, &mut frame, render_flags);
        }));
        if let Err(payload) = result {
            let panic_info = captured_panic_from(last_panic.clone(), payload);
            write_play_crash_report(
                &pre_frame_game,
                host_frame,
                input,
                run_what,
                "replay_run_frame",
                Some(&panic_info),
            );
            process::exit(101);
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
    let mut frame = vec![0u8; 256 * 224 * 4];
    let mut audio = vec![0i16; 735 * 2];
    let mut audio_nonzero = 0usize;
    let mut audio_peak = 0i16;
    let render_flags = PpuRenderFlags::empty();
    for _ in 0..frames {
        run_play_frame_bgra(&mut game, 0, &mut frame, render_flags);
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
    let mut frame = vec![0u8; 256 * 224 * 4];
    let mut audio = vec![0i16; 735 * 2];
    let render_flags = PpuRenderFlags::empty();
    let mut last_ports = [0u8; 4];
    let mut last_nonzero = false;
    for frame_index in 0..frames {
        run_play_frame_bgra(&mut game, 0, &mut frame, render_flags);
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
    let mut frame = vec![0u8; 256 * 224 * 4];
    let mut audio = vec![0i16; 735 * 2];
    let render_flags = PpuRenderFlags::empty();
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
        run_play_frame_bgra(&mut game, 0, &mut frame, render_flags);
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
    let mut frame = vec![0u8; 256 * 224 * 4];
    let mut audio = vec![0i16; 735 * 2];
    let render_flags = PpuRenderFlags::empty();
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
        run_play_frame_bgra(&mut game, 0, &mut frame, render_flags);
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
    let render_flags = PpuRenderFlags::empty();
    let mut rust_frame = vec![0u8; width as usize * height as usize * 4];
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
    for frame_index in 0..frames {
        let input = input_script.input_for_frame(frame_index);
        let pre_game = game.clone();
        run_play_frame_bgra(&mut game, input, &mut rust_frame, render_flags);
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
                dsp_writes =
                    game.zelda_render_audio_trace_dsp(&mut rust_audio, sample_frames as i32, 2);
            } else {
                game.zelda_render_audio(&mut rust_audio, sample_frames as i32, 2);
            }
        } else {
            rust_audio.clear();
            dsp_writes.clear();
            discard_audio.resize(last_sample_frames.saturating_mul(2), 0);
            if trace_dsp_writes {
                dsp_writes = game.zelda_render_audio_trace_dsp(
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
            let rust_pixel = rgba_pixel_at(&rust_frame, rust_offset).unwrap_or([0; 4]);
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
        let compare_this_frame = frame_index >= compare_from_frame;
        if compare_this_frame && compare_video {
            let mut video_diff = compare_libretro_video_frame(
                &rust_frame,
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
                    &rust_frame,
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
                        &rust_frame,
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
                    &rust_frame,
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
                let artifact_dir = write_bsnes_parity_failure_artifacts(
                    &pre_game,
                    &game,
                    &rust_frame,
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

fn format_dsp_writes(writes: &[(u8, u8, i32, u8)]) -> String {
    writes
        .iter()
        .map(|(addr, value, sample_offset, timer_cycles)| {
            format!("{addr:02x}:{value:02x}@{sample_offset}/{timer_cycles}")
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
    let width = 256u32;
    let height = 224u32;
    let render_flags = PpuRenderFlags::empty();
    let mut frame = vec![0u8; width as usize * height as usize * 4];
    let mut high_audio = vec![0i16; 735 * 2];
    let mut full_audio = vec![0i16; 735 * 2];
    let mut high_stats = Vec::with_capacity(frames as usize);
    let mut full_stats = Vec::with_capacity(frames as usize);
    let mut debug = Vec::with_capacity(frames as usize);

    for _ in 0..frames {
        run_play_frame_bgra(&mut game, 0, &mut frame, render_flags);
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

#[derive(Clone, Copy, Debug, Default)]
struct AudioFrameStats {
    samples_per_channel: usize,
    peak: i16,
    first_nonzero: Option<usize>,
    mean_abs: u32,
}

impl AudioFrameStats {
    fn from_interleaved_stereo(samples: &[i16]) -> Self {
        let mut sum = 0u64;
        let mut peak = 0i16;
        let mut first_nonzero = None;
        for (i, &sample) in samples.iter().enumerate() {
            let abs = sample.saturating_abs();
            if abs > peak {
                peak = abs;
            }
            if sample != 0 && first_nonzero.is_none() {
                first_nonzero = Some(i);
            }
            sum += abs as u64;
        }
        let mean_abs = if samples.is_empty() {
            0
        } else {
            (sum / samples.len() as u64) as u32
        };
        Self {
            samples_per_channel: samples.len() / 2,
            peak,
            first_nonzero,
            mean_abs,
        }
    }
}

fn print_replay_audio_trace(
    frame: u32,
    game: &ZeldaState,
    audio: &[i16],
    samples: usize,
    channels: usize,
    dsp_pre_hash: u32,
    dsp_writes: &[(u8, u8, i32, u8)],
) {
    let stats = AudioFrameStats::from_interleaved_stereo(audio);
    let mean_abs = if audio.is_empty() {
        0.0
    } else {
        audio
            .iter()
            .map(|sample| i64::from(sample.saturating_abs()))
            .sum::<i64>() as f64
            / audio.len() as f64
    };
    print!(
        "{{\"frame\":{frame},\"samples\":{samples},\"channels\":{channels},\"peak\":{},\"first_nonzero\":",
        stats.peak
    );
    if let Some(first_nonzero) = stats.first_nonzero {
        print!("{first_nonzero}");
    } else {
        print!("null");
    }
    println!(
        ",\"mean_abs\":{mean_abs:.6},\"hash\":\"0x{:08x}\",\"apui\":[{},{},{},{}],\"music\":[{},{},{}],\"main\":{},\"sub\":{},\"subsub\":{},\"inidisp\":{},\"dsp_pre\":\"0x{dsp_pre_hash:08x}\",\"dsp_post\":\"0x{:08x}\",\"dsp_writes\":{},\"dsp_write_hash\":\"0x{:08x}\",\"dsp_write_values_hash\":\"0x{:08x}\"{},{}{}",
        replay_checksum_samples(audio),
        game.ram[0x0648],
        game.ram[0x012c],
        game.ram[0x012d],
        game.ram[0x012e],
        game.ram[0x012f],
        game.ram[0x0132],
        game.ram[0x0133],
        game.ram[TRACE_MAIN_MODULE_INDEX],
        game.ram[TRACE_SUBMODULE_INDEX],
        game.ram[TRACE_SUBSUBMODULE_INDEX],
        game.ram[0x13],
        game.zelda_audio_dsp_hash(),
        dsp_writes.len(),
        replay_checksum_dsp_writes(dsp_writes),
        replay_checksum_dsp_write_values(dsp_writes),
        replay_dsp_write_events_json(frame, dsp_writes),
        game.zelda_audio_route_debug_json(),
        "}",
    );
}

fn replay_dsp_write_events_json(frame: u32, writes: &[(u8, u8, i32, u8)]) -> String {
    let target = env::var("ZELDA3_AUDIO_TRACE_DSP_WRITES_FRAME")
        .ok()
        .and_then(|value| value.parse::<u32>().ok());
    if target != Some(frame) {
        return String::new();
    }
    let events = writes
        .iter()
        .map(|(addr, val, sample_offset, timer)| format!("[{addr},{val},{sample_offset},{timer}]"))
        .collect::<Vec<_>>()
        .join(",");
    format!(",\"dsp_write_events\":[{events}]")
}

fn replay_checksum_dsp_writes(writes: &[(u8, u8, i32, u8)]) -> u32 {
    let mut hash = 2166136261u32;
    for &(addr, val, sample_offset, timer_cycles) in writes {
        hash = (hash ^ u32::from(addr)).wrapping_mul(16777619);
        hash = (hash ^ u32::from(val)).wrapping_mul(16777619);
        for byte in sample_offset.to_le_bytes() {
            hash = (hash ^ u32::from(byte)).wrapping_mul(16777619);
        }
        hash = (hash ^ u32::from(timer_cycles)).wrapping_mul(16777619);
    }
    hash
}

fn replay_checksum_dsp_write_values(writes: &[(u8, u8, i32, u8)]) -> u32 {
    let mut hash = 2166136261u32;
    for &(addr, val, _, _) in writes {
        hash = (hash ^ u32::from(addr)).wrapping_mul(16777619);
        hash = (hash ^ u32::from(val)).wrapping_mul(16777619);
    }
    hash
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
    let pitch = width as usize * 4;
    let mut rust_frame = vec![0u8; width as usize * height as usize * 4];
    let mut snes_state_rust_render_frame = vec![0u8; rust_frame.len()];
    let mut rust_state = post_oracle.game.clone();
    let mut oracle_state = post_oracle.game.clone();
    oracle_state.ppu = post_oracle.snes.ppu.clone();
    oracle_state.dma = post_oracle.snes.dma.clone();
    oracle_state.ram.copy_from_slice(&post_oracle.snes.ram);
    oracle_state
        .sram
        .copy_from_slice(&post_oracle.snes.cart.ram);
    render_play_frame_bgra(
        &mut rust_state,
        &mut rust_frame,
        pitch,
        PpuRenderFlags::empty(),
    );
    render_play_frame_bgra(
        &mut oracle_state,
        &mut snes_state_rust_render_frame,
        pitch,
        PpuRenderFlags::empty(),
    );
    write_argb_frame_png(&dir.join("rust_frame.png"), &rust_frame, width, height)?;
    write_argb_frame_png(
        &dir.join("snes_state_rust_render_frame.png"),
        &snes_state_rust_render_frame,
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
            "snes_state_rust_render_frame.png is the Rust renderer drawing the C/SNES oracle state; it is not a true C-rendered or bsnes-rendered frame".to_string(),
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
    rust_frame: &[u8],
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

    write_argb_frame_png(&dir.join("rust_frame.png"), rust_frame, 256, 224)?;
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

fn first_peak_frame(stats: &[AudioFrameStats], threshold: i16) -> Option<usize> {
    stats.iter().position(|stats| stats.peak >= threshold)
}

fn max_peak_frame(stats: &[AudioFrameStats]) -> Option<(usize, i16)> {
    stats
        .iter()
        .enumerate()
        .max_by_key(|(_, stats)| stats.peak)
        .map(|(i, stats)| (i, stats.peak))
}

fn print_audio_window(
    label: &str,
    stats: &[AudioFrameStats],
    debug: &[String],
    center: Option<usize>,
) {
    let Some(center) = center else {
        println!("{label}: no non-silent frames captured");
        return;
    };
    let start = center.saturating_sub(4);
    let end = (center + 12).min(stats.len().saturating_sub(1));
    println!("{label} window frames {start}..={end}:");
    for i in start..=end {
        let stats = stats[i];
        if let Some(debug) = debug.get(i) {
            println!(
                "  {i:>5}: peak={:>5} mean_abs={:>4} first={:?} samples={} {debug}",
                stats.peak, stats.mean_abs, stats.first_nonzero, stats.samples_per_channel,
            );
        } else {
            println!(
                "  {i:>5}: peak={:>5} mean_abs={:>4} first={:?} samples={}",
                stats.peak, stats.mean_abs, stats.first_nonzero, stats.samples_per_channel,
            );
        }
    }
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

fn read_file_or_exit(path: &Path, label: &str) -> Vec<u8> {
    match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("failed to read {label} {}: {e}", path.display());
            process::exit(1);
        }
    }
}

fn apply_sram_to_game_or_exit(game: &mut ZeldaState, path: &Path, sram: &[u8]) {
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

fn run_dump_frame(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --dump-frame <path-to-rom.sfc> <frames> <out.png> [--input-script <path>] [--load-sram <path>] [--load-state <path>]"
            );
            process::exit(2);
        }
    };
    let frames: u32 = match args.get(1).and_then(|s| s.parse().ok()) {
        Some(frames) => frames,
        None => {
            eprintln!(
                "usage: zelda3 --dump-frame <path-to-rom.sfc> <frames> <out.png> [--input-script <path>] [--load-sram <path>] [--load-state <path>]"
            );
            process::exit(2);
        }
    };
    let out_path = match args.get(2) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "usage: zelda3 --dump-frame <path-to-rom.sfc> <frames> <out.png> [--input-script <path>] [--load-sram <path>] [--load-state <path>]"
            );
            process::exit(2);
        }
    };
    let mut input_script = InputScript::default();
    let render_flags = PpuRenderFlags::empty();
    let mut load_sram = None;
    let mut load_state = None;
    let mut i = 3usize;
    while i < args.len() {
        match args[i].as_str() {
            "--input-script" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--input-script requires a path");
                    process::exit(2);
                });
                input_script = match InputScript::from_path(Path::new(path)) {
                    Ok(script) => script,
                    Err(e) => {
                        eprintln!("failed to parse input script {}: {e}", path);
                        process::exit(2);
                    }
                };
                i += 2;
            }
            "--load-state" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--load-state requires a path");
                    process::exit(2);
                });
                load_state = Some(PathBuf::from(path));
                i += 2;
            }
            "--load-sram" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--load-sram requires a path");
                    process::exit(2);
                });
                load_sram = Some(PathBuf::from(path));
                i += 2;
            }
            flag => {
                eprintln!("unknown dump-frame option: {flag}");
                process::exit(2);
            }
        }
    }
    if load_state.is_some() && load_sram.is_some() {
        eprintln!("--dump-frame cannot combine --load-sram with --load-state");
        process::exit(2);
    }

    let (mut game, start_frame) = load_play_or_checkpoint(rom_path, load_state.as_deref());
    if let Some(path) = load_sram.as_deref() {
        let sram = read_file_or_exit(path, "SRAM");
        apply_sram_to_game_or_exit(&mut game, path, &sram);
    }
    let width = 256u32;
    let height = 224u32;
    let mut frame = vec![0u8; width as usize * height as usize * 4];
    for frame_no in 0..frames {
        let input = input_script.input_for_frame(start_frame.wrapping_add(frame_no));
        run_play_frame_bgra(&mut game, input, &mut frame, render_flags);
    }
    if let Err(e) = write_argb_frame_png(&out_path, &frame, width, height) {
        eprintln!("failed to write {}: {e}", out_path.display());
        process::exit(1);
    }
    println!(
        "dumped frame {frames} to {}; main={:02x}; sub={:02x}; mode={}; screen={:02x}/{:02x}; cgram_nonzero={}; oam_nonzero={}",
        out_path.display(),
        game.ram[0x10],
        game.ram[0x11],
        game.ppu.mode,
        game.ppu.screen_enabled[0],
        game.ppu.screen_enabled[1],
        game.ppu.cgram.iter().filter(|&&v| v != 0).count(),
        game.ppu.oam.iter().filter(|&&v| v != 0).count(),
    );
}

fn run_dump_developer_destination(args: &[String]) {
    let id = match args.first() {
        Some(id) => id,
        None => {
            eprintln!(
                "usage: zelda3 --dump-developer-destination <destination-id> <frames> <cpu-out.png> [--gpu <gpu-out.png>]"
            );
            process::exit(2);
        }
    };
    let frames: u32 = match args.get(1).and_then(|s| s.parse().ok()) {
        Some(frames) => frames,
        None => {
            eprintln!(
                "usage: zelda3 --dump-developer-destination <destination-id> <frames> <cpu-out.png> [--gpu <gpu-out.png>]"
            );
            process::exit(2);
        }
    };
    let out_path = match args.get(2) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "usage: zelda3 --dump-developer-destination <destination-id> <frames> <cpu-out.png> [--gpu <gpu-out.png>]"
            );
            process::exit(2);
        }
    };
    let mut gpu_out_path = None::<PathBuf>;
    let mut i = 3usize;
    while i < args.len() {
        match args[i].as_str() {
            "--gpu" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--gpu requires a path");
                    process::exit(2);
                });
                gpu_out_path = Some(PathBuf::from(path));
                i += 2;
            }
            flag => {
                eprintln!("unknown dump-developer-destination option: {flag}");
                process::exit(2);
            }
        }
    }

    let (mut game, start_frame) = match load_developer_destination(id) {
        Ok(loaded) => loaded,
        Err(e) => {
            eprintln!("failed to load developer destination {id}: {e}");
            process::exit(1);
        }
    };
    let width = 256u32;
    let height = 224u32;
    let mut frame = vec![0u8; width as usize * height as usize * 4];
    for _ in 0..frames {
        run_play_frame_bgra(&mut game, 0, &mut frame, PpuRenderFlags::empty());
    }
    if let Err(e) = write_argb_frame_png(&out_path, &frame, width, height) {
        eprintln!("failed to write {}: {e}", out_path.display());
        process::exit(1);
    }

    if let Some(path) = gpu_out_path.as_deref() {
        let rgba = render_live_game_gpu_frame_rgba(&mut game, width, height);
        if let Err(e) = write_rgba_frame_png(path, &rgba, width, height) {
            eprintln!("failed to write {}: {e}", path.display());
            process::exit(1);
        }
    }

    println!(
        "dumped developer destination {id} frames={frames} start_frame={start_frame} to {}; gpu={}; main={:02x}; sub={:02x}; mode={}; screen={:02x}/{:02x}; bg1_tm={:04x}; bg1_chr={:04x}; cgram_nonzero={}; oam_nonzero={}",
        out_path.display(),
        gpu_out_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "none".to_string()),
        game.ram[0x10],
        game.ram[0x11],
        game.ppu.mode,
        game.ppu.screen_enabled[0],
        game.ppu.screen_enabled[1],
        game.ppu.bg_layer[0].tilemap_adr,
        game.ppu.bg_layer[0].tile_adr,
        game.ppu.cgram.iter().filter(|&&v| v != 0).count(),
        game.ppu.oam.iter().filter(|&&v| v != 0).count(),
    );
}

fn run_dump_overworld_screen(args: &[String]) {
    let rom_path = match args.first() {
        Some(path) => path,
        None => {
            eprintln!("usage: zelda3 --dump-overworld-screen <path-to-rom.sfc> <screen> <out.png>");
            process::exit(2);
        }
    };
    let screen = match args.get(1).and_then(|s| parse_u16_auto(s)) {
        Some(screen) => screen,
        None => {
            eprintln!("usage: zelda3 --dump-overworld-screen <path-to-rom.sfc> <screen> <out.png>");
            process::exit(2);
        }
    };
    let out_path = match args.get(2) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!("usage: zelda3 --dump-overworld-screen <path-to-rom.sfc> <screen> <out.png>");
            process::exit(2);
        }
    };

    let mut game = load_translated_replay_state(rom_path);
    let loaded = game.parity_probe_overworld_screen(screen);
    let width = 256u32;
    let height = 224u32;
    let mut frame = vec![0u8; width as usize * height as usize * 4];
    render_play_frame_bgra(
        &mut game,
        &mut frame,
        width as usize * 4,
        PpuRenderFlags::empty(),
    );
    if let Err(e) = write_argb_frame_png(&out_path, &frame, width, height) {
        eprintln!("failed to write {}: {e}", out_path.display());
        process::exit(1);
    }
    println!(
        "dumped overworld screen requested=0x{screen:04x} loaded=0x{loaded:04x} to {}; mode={}; screen={:02x}/{:02x}; bg1_tm={:04x}; bg1_chr={:04x}; bg2_tm={:04x}; bg2_chr={:04x}",
        out_path.display(),
        game.ppu.mode,
        game.ppu.screen_enabled[0],
        game.ppu.screen_enabled[1],
        game.ppu.bg_layer[0].tilemap_adr,
        game.ppu.bg_layer[0].tile_adr,
        game.ppu.bg_layer[1].tilemap_adr,
        game.ppu.bg_layer[1].tile_adr,
    );
}

fn run_scan_replay_checkpoints(args: &[String]) {
    let rom_path = match args.first() {
        Some(path) => path,
        None => {
            eprintln!(
                "usage: zelda3 --scan-replay-checkpoints <path-to-rom.sfc> <checkpoint-dir> [screen]"
            );
            process::exit(2);
        }
    };
    let checkpoint_dir = match args.get(1) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "usage: zelda3 --scan-replay-checkpoints <path-to-rom.sfc> <checkpoint-dir> [screen]"
            );
            process::exit(2);
        }
    };
    let target_screen = args.get(2).and_then(|value| parse_u16_auto(value));
    let mut checkpoints = match fs::read_dir(&checkpoint_dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sav"))
            .collect::<Vec<_>>(),
        Err(e) => {
            eprintln!("failed to read {}: {e}", checkpoint_dir.display());
            process::exit(1);
        }
    };
    checkpoints.sort();

    for path in checkpoints {
        let mut game = load_translated_replay_state(rom_path);
        if let Err(e) = load_replay_save_checkpoint(&mut game, &path) {
            eprintln!("failed to load {}: {e}", path.display());
            continue;
        }
        let indoors = game.ram[PLAYER_IS_INDOORS] != 0;
        let room = read_le_u16(&game.ram, 0x48e);
        let screen = read_le_u16(&game.ram, 0x8a);
        if target_screen.is_some_and(|target| indoors || screen != target) {
            continue;
        }
        println!(
            "{} indoors={} room=0x{room:04x} ow=0x{screen:04x} main={:02x} sub={:02x} mode={} screen={:02x}/{:02x} bg1_tm={:04x} bg1_chr={:04x} bg2_tm={:04x} bg2_chr={:04x} cgram_nonzero={} vram_nonzero={}",
            path.display(),
            indoors,
            game.ram[0x10],
            game.ram[0x11],
            game.ppu.mode,
            game.ppu.screen_enabled[0],
            game.ppu.screen_enabled[1],
            game.ppu.bg_layer[0].tilemap_adr,
            game.ppu.bg_layer[0].tile_adr,
            game.ppu.bg_layer[1].tilemap_adr,
            game.ppu.bg_layer[1].tile_adr,
            game.ppu.cgram.iter().filter(|&&v| v != 0).count(),
            game.ppu.vram.iter().filter(|&&v| v != 0).count(),
        );
    }
}

fn run_dump_replay_checkpoint_ppu(args: &[String]) {
    let rom_path = match args.first() {
        Some(path) => path,
        None => {
            eprintln!(
                "usage: zelda3 --dump-replay-checkpoint-ppu <path-to-rom.sfc> <checkpoint.sav> <out.png> [frames]"
            );
            process::exit(2);
        }
    };
    let checkpoint_path = match args.get(1) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "usage: zelda3 --dump-replay-checkpoint-ppu <path-to-rom.sfc> <checkpoint.sav> <out.png> [frames]"
            );
            process::exit(2);
        }
    };
    let out_path = match args.get(2) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "usage: zelda3 --dump-replay-checkpoint-ppu <path-to-rom.sfc> <checkpoint.sav> <out.png> [frames]"
            );
            process::exit(2);
        }
    };
    let frames = args
        .get(3)
        .and_then(|value| value.parse().ok())
        .unwrap_or(1);
    let mut game = load_translated_replay_state(rom_path);
    if let Err(e) = load_replay_save_checkpoint(&mut game, &checkpoint_path) {
        eprintln!("failed to load {}: {e}", checkpoint_path.display());
        process::exit(1);
    }
    let width = 256u32;
    let height = 224u32;
    let mut frame = vec![0u8; width as usize * height as usize * 4];
    for _ in 0..frames {
        run_play_frame_bgra(&mut game, 0, &mut frame, PpuRenderFlags::empty());
    }
    if let Err(e) = write_argb_frame_png(&out_path, &frame, width, height) {
        eprintln!("failed to write {}: {e}", out_path.display());
        process::exit(1);
    }
    println!(
        "dumped replay checkpoint {} frames={frames} to {}; indoors={} ow=0x{:04x} main={:02x} sub={:02x} mode={} screen={:02x}/{:02x} bg1_tm={:04x} bg1_chr={:04x}",
        checkpoint_path.display(),
        out_path.display(),
        game.ram[PLAYER_IS_INDOORS] != 0,
        read_le_u16(&game.ram, 0x8a),
        game.ram[0x10],
        game.ram[0x11],
        game.ppu.mode,
        game.ppu.screen_enabled[0],
        game.ppu.screen_enabled[1],
        game.ppu.bg_layer[0].tilemap_adr,
        game.ppu.bg_layer[0].tile_adr,
    );
}

fn run_dump_developer_tileset(args: &[String]) {
    let tileset_id = match args.first() {
        Some(id) => id,
        None => {
            eprintln!(
                "usage: zelda3 --dump-developer-tileset <tileset-id> <atlas.png> [manifest.json]"
            );
            process::exit(2);
        }
    };
    if tileset_id != "kakariko-town" && tileset_id != "kakariko_town" {
        eprintln!("unknown developer tileset '{tileset_id}'");
        process::exit(2);
    }
    let atlas_path = match args.get(1) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "usage: zelda3 --dump-developer-tileset <tileset-id> <atlas.png> [manifest.json]"
            );
            process::exit(2);
        }
    };
    let manifest_out_path = args.get(2).map(PathBuf::from);
    let room = developer_destinations::synthetic_room("preset-dev-sandbox")
        .expect("developer sandbox synthetic room should be registered");
    let source = match load_developer_room_theme_source(room) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };
    let manifest = match developer_kakariko_tileset_manifest() {
        Ok(manifest) => manifest,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };
    let scale = 2usize;
    let cell_px = 16usize;
    let grid_px = 1usize;
    let atlas_width =
        manifest.columns as usize * cell_px * scale + (manifest.columns as usize + 1) * grid_px;
    let atlas_height =
        manifest.rows as usize * cell_px * scale + (manifest.rows as usize + 1) * grid_px;
    let mut atlas = vec![0u8; atlas_width * atlas_height * 4];
    for px in atlas.chunks_exact_mut(4) {
        px.copy_from_slice(&[24, 24, 24, 0xff]);
    }
    let bg = source.ppu.bg_layer[DEVELOPER_ROOM_SOURCE_BG_LAYER];
    for entry in &manifest.entries {
        let (source_x, source_y) = developer_room_kakariko_sample_origin(&source, entry.id);
        let atlas_cell_x =
            grid_px + usize::from(entry.id % manifest.columns) * (cell_px * scale + grid_px);
        let atlas_cell_y =
            grid_px + usize::from(entry.id / manifest.columns) * (cell_px * scale + grid_px);
        for y_offset in 0..2usize {
            for x_offset in 0..2usize {
                let tilemap_index =
                    bg.tilemap_adr as usize + (source_y + y_offset) * 32 + source_x + x_offset;
                let tilemap_entry = source.ppu.vram.get(tilemap_index).copied().unwrap_or(0);
                draw_snes_4bpp_tilemap_entry_to_rgba(
                    &source.ppu.vram,
                    &source.ppu.cgram,
                    bg.tile_adr as usize,
                    tilemap_entry,
                    &mut atlas,
                    atlas_width,
                    atlas_cell_x + x_offset * 8 * scale,
                    atlas_cell_y + y_offset * 8 * scale,
                    scale,
                );
            }
        }
    }
    if let Err(e) =
        write_rgba_frame_png(&atlas_path, &atlas, atlas_width as u32, atlas_height as u32)
    {
        eprintln!("failed to write {}: {e}", atlas_path.display());
        process::exit(1);
    }
    if let Some(path) = manifest_out_path.as_deref() {
        let json = match serde_json::to_vec_pretty(&manifest) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("failed to serialize Kakariko developer tileset: {e}");
                process::exit(1);
            }
        };
        if let Err(e) = fs::write(path, json) {
            eprintln!("failed to write {}: {e}", path.display());
            process::exit(1);
        }
    }
    println!(
        "dumped developer tileset {} entries={} atlas={} manifest={}",
        manifest.id,
        manifest.entries.len(),
        atlas_path.display(),
        manifest_out_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "none".to_string()),
    );
}

fn run_dump_unique_overworld_cells(args: &[String]) {
    let atlas_path = match args.first() {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "usage: zelda3 --dump-unique-overworld-cells <atlas.png> <manifest.json> [max-screen]"
            );
            process::exit(2);
        }
    };
    let manifest_path = match args.get(1) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "usage: zelda3 --dump-unique-overworld-cells <atlas.png> <manifest.json> [max-screen]"
            );
            process::exit(2);
        }
    };
    let max_screen = args
        .get(2)
        .and_then(|value| parse_u16_auto(value))
        .unwrap_or(0x7f);

    let rom_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc");
    let mut collector = UniqueOverworldCellCollector::default();
    let mut loaded_count = 0u16;
    let mut skipped_count = 0u16;
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    for screen in 0..=max_screen {
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut game = load_translated_replay_state(rom_path);
            let loaded_screen = game.parity_probe_overworld_screen_and_build_map(screen);
            collect_unique_overworld_cells_from_built_bg2_map(
                &mut collector,
                &game,
                screen,
                loaded_screen,
            );
        }));
        if result.is_ok() {
            loaded_count = loaded_count.wrapping_add(1);
        } else {
            skipped_count = skipped_count.wrapping_add(1);
        }
    }
    panic::set_hook(original_hook);

    let columns = 64usize;
    let (atlas, width, height) = render_unique_overworld_cell_atlas(&collector, columns, 2);
    if let Err(e) = write_rgba_frame_png(&atlas_path, &atlas, width, height) {
        eprintln!("failed to write {}: {e}", atlas_path.display());
        process::exit(1);
    }
    let manifest = collector.manifest(columns as u16);
    let json = match serde_json::to_vec_pretty(&manifest) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("failed to serialize unique overworld cell manifest: {e}");
            process::exit(1);
        }
    };
    if let Err(e) = fs::write(&manifest_path, json) {
        eprintln!("failed to write {}: {e}", manifest_path.display());
        process::exit(1);
    }
    println!(
        "dumped unique overworld cells unique={} sources={} loaded_screens={} skipped_screens={} atlas={} manifest={}",
        collector.cells.len(),
        collector
            .cells
            .iter()
            .map(|cell| cell.sources.len())
            .sum::<usize>(),
        loaded_count,
        skipped_count,
        atlas_path.display(),
        manifest_path.display(),
    );
}

fn run_dump_unique_overworld_tiles(args: &[String]) {
    let atlas_path = match args.first() {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "usage: zelda3 --dump-unique-overworld-tiles <atlas.png> <manifest.json> [max-screen]"
            );
            process::exit(2);
        }
    };
    let manifest_path = match args.get(1) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "usage: zelda3 --dump-unique-overworld-tiles <atlas.png> <manifest.json> [max-screen]"
            );
            process::exit(2);
        }
    };
    let max_screen = args
        .get(2)
        .and_then(|value| parse_u16_auto(value))
        .unwrap_or(0x7f);

    let rom_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc");
    let mut collector = UniqueOverworldTileCollector::default();
    let mut index_collector = OverworldIndexTileCollector::default();
    let mut loaded_count = 0u16;
    let mut skipped_count = 0u16;
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    for screen in 0..=max_screen {
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut game = load_translated_replay_state(rom_path);
            let loaded_screen = game.parity_probe_overworld_screen_and_build_map(screen);
            collect_unique_overworld_tiles_from_built_bg2_map(
                &mut collector,
                &mut index_collector,
                &game,
                screen,
                loaded_screen,
            );
        }));
        if result.is_ok() {
            loaded_count = loaded_count.wrapping_add(1);
        } else {
            skipped_count = skipped_count.wrapping_add(1);
        }
    }
    panic::set_hook(original_hook);

    let columns = 64usize;
    let atlas_scale = 4u8;
    let atlas_grid_px = 1u8;
    let (atlas, width, height) =
        render_unique_overworld_tile_atlas(&collector, columns, usize::from(atlas_scale));
    if let Err(e) = write_rgba_frame_png(&atlas_path, &atlas, width, height) {
        eprintln!("failed to write {}: {e}", atlas_path.display());
        process::exit(1);
    }
    let manifest = collector.manifest(columns as u16, atlas_scale, atlas_grid_px);
    let json = match serde_json::to_vec_pretty(&manifest) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("failed to serialize unique overworld tile manifest: {e}");
            process::exit(1);
        }
    };
    if let Err(e) = fs::write(&manifest_path, json) {
        eprintln!("failed to write {}: {e}", manifest_path.display());
        process::exit(1);
    }
    println!(
        "dumped unique overworld tiles unique={} sources={} loaded_screens={} skipped_screens={} atlas={} manifest={}",
        collector.tiles.len(),
        collector
            .tiles
            .iter()
            .map(|tile| tile.sources.len())
            .sum::<usize>(),
        loaded_count,
        skipped_count,
        atlas_path.display(),
        manifest_path.display(),
    );

    // Write palette-index atlas (canonical paths, independent of the RGBA output args).
    const INDEX_BIN: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/overworld_index_tiles.bin"
    );
    const INDEX_JSON: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/overworld_index_tiles.json"
    );
    let cell_count = index_collector.tiles.len();
    let mut bin = Vec::with_capacity(cell_count * 64);
    for tile in &index_collector.tiles {
        bin.extend_from_slice(&tile.indices);
    }
    if let Err(e) = fs::write(INDEX_BIN, &bin) {
        eprintln!("failed to write index atlas bin {INDEX_BIN}: {e}");
        process::exit(1);
    }
    let index_manifest = OverworldIndexTileAtlasManifest {
        format: "zelda3_overworld_index_tiles_v1",
        tile_width_px: 8,
        tile_height_px: 8,
        cell_count: cell_count as u32,
        cells: index_collector
            .tiles
            .iter()
            .enumerate()
            .map(|(id, tile)| OverworldIndexTileCellManifest {
                id: id as u32,
                graphics_keys: tile.graphics_keys.clone(),
            })
            .collect(),
    };
    let index_json = match serde_json::to_vec_pretty(&index_manifest) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("failed to serialize overworld index tile manifest: {e}");
            process::exit(1);
        }
    };
    if let Err(e) = fs::write(INDEX_JSON, &index_json) {
        eprintln!("failed to write index atlas json {INDEX_JSON}: {e}");
        process::exit(1);
    }
    println!(
        "dumped index atlas cells={} bin={} json={}",
        cell_count, INDEX_BIN, INDEX_JSON
    );
}

#[derive(Serialize)]
struct DungeonIndexTileAtlasManifest {
    format: &'static str,
    tile_width_px: u8,
    tile_height_px: u8,
    cell_count: u32,
    cells: Vec<DungeonIndexTileCellManifest>,
}

#[derive(Serialize)]
struct DungeonIndexTileCellManifest {
    id: u32,
    /// Packed keys: `(theme as u32) << 16 | (tilemap_entry & 0xC3FF) as u32`.
    keys: Vec<u32>,
}

/// Manifest for the sprite palette-index tile atlas produced by `--dump-sprite-index-tiles`.
/// Format version: `zelda3_sprite_index_tiles_v1`.
#[derive(Serialize)]
struct SpriteIndexTileAtlasManifest {
    format: &'static str,
    tile_width_px: u8,
    tile_height_px: u8,
    cell_count: u32,
    cells: Vec<SpriteIndexTileCellManifest>,
}

#[derive(Serialize)]
struct SpriteIndexTileCellManifest {
    id: u32,
    keys: Vec<SpriteIndexKey>,
}

/// One lookup key for a sprite index cell: a `(context, tile)` pair where
/// `context = g0|(g1<<16)|(g2<<32)|(g3<<48)` (sprite graphics subsets 0..4)
/// and `tile` is the 8×8 cell offset in 0..512 from VRAM base 0x4000.
#[derive(Serialize)]
struct SpriteIndexKey {
    context: u64,
    tile: u16,
}

/// Walk all 0x128 dungeon entrance indices, dedup tiles by 64-byte pattern, and emit
/// `developer_tilesets/dungeon_index_tiles.{bin,json}`.
fn run_dump_dungeon_index_tiles(_args: &[String]) {
    use std::collections::BTreeSet;

    const INDEX_BIN: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/dungeon_index_tiles.bin"
    );
    const INDEX_JSON: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/dungeon_index_tiles.json"
    );

    let rom = concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc");
    let mut cells: Vec<[u8; 64]> = Vec::new();
    let mut index_by_pattern: HashMap<[u8; 64], usize> = HashMap::new();
    let mut keys_by_cell: Vec<BTreeSet<u32>> = Vec::new();

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    for room in 0u16..0x128 {
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut game = load_translated_replay_state(rom);
            dungeon_room_index_probe(&mut game, room)
        }));
        let (theme, tiles) = match result {
            Ok(v) => v,
            Err(_) => continue,
        };
        for (word, pattern) in tiles {
            let key = ((theme as u32) << 16) | ((word & 0xC3FF) as u32);
            let id = *index_by_pattern.entry(pattern).or_insert_with(|| {
                cells.push(pattern);
                keys_by_cell.push(BTreeSet::new());
                cells.len() - 1
            });
            keys_by_cell[id].insert(key);
        }
    }
    panic::set_hook(original_hook);

    let cell_count = cells.len();
    let mut bin = Vec::with_capacity(cell_count * 64);
    for pattern in &cells {
        bin.extend_from_slice(pattern);
    }
    if let Err(e) = fs::write(INDEX_BIN, &bin) {
        eprintln!("failed to write dungeon index atlas bin {INDEX_BIN}: {e}");
        process::exit(1);
    }

    let manifest = DungeonIndexTileAtlasManifest {
        format: "zelda3_dungeon_index_tiles_v1",
        tile_width_px: 8,
        tile_height_px: 8,
        cell_count: cell_count as u32,
        cells: keys_by_cell
            .iter()
            .enumerate()
            .map(|(id, keys)| DungeonIndexTileCellManifest {
                id: id as u32,
                keys: keys.iter().copied().collect(),
            })
            .collect(),
    };
    let index_json = match serde_json::to_vec_pretty(&manifest) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("failed to serialize dungeon index tile manifest: {e}");
            process::exit(1);
        }
    };
    if let Err(e) = fs::write(INDEX_JSON, &index_json) {
        eprintln!("failed to write dungeon index atlas json {INDEX_JSON}: {e}");
        process::exit(1);
    }

    println!("dumped dungeon index atlas cells={cell_count}");
}

/// Walk all dungeon entrances (0..0x128) and overworld screens (0..0x80), decode
/// every non-zero 8×8 sprite CHR tile (VRAM 0x4000, 512 tiles), dedup by 64-byte
/// pattern, and emit `developer_tilesets/sprite_index_tiles.{bin,json}`.
///
/// Context key: `g0|(g1<<16)|(g2<<32)|(g3<<48)` over the 4 sprite graphics subsets
/// populated by `InitializeTilesets`; one full decode per unique context.
fn run_dump_sprite_index_tiles(_args: &[String]) {
    use std::collections::{BTreeSet, HashSet};

    /// VRAM word base for OBJ (sprite) CHR, bank 1 (tile_adr1=0x4000).
    /// The 512-tile window covers 0x4000..0x5FFF: tiles 0..256 = bank1, 256..512 = bank2.
    const SPRITE_CHR_VRAM_BASE: usize = 0x4000;
    const SPRITE_CHR_TILE_COUNT: u16 = 512;

    const INDEX_BIN: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/sprite_index_tiles.bin"
    );
    const INDEX_JSON: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/sprite_index_tiles.json"
    );
    let rom = concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc");

    let mut cells: Vec<[u8; 64]> = Vec::new();
    let mut index_by_pattern: HashMap<[u8; 64], usize> = HashMap::new();
    let mut keys_by_cell: Vec<BTreeSet<(u64, u16)>> = Vec::new();
    let mut seen_contexts: HashSet<u64> = HashSet::new();

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    // Walk dungeon entrances 0..0x128.
    for room in 0u16..0x128 {
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut game = load_translated_replay_state(rom);
            game.parity_probe_dungeon_load_and_draw(room);
            let context = (game.parity_probe_sprite_graphics_subset(0) as u64)
                | ((game.parity_probe_sprite_graphics_subset(1) as u64) << 16)
                | ((game.parity_probe_sprite_graphics_subset(2) as u64) << 32)
                | ((game.parity_probe_sprite_graphics_subset(3) as u64) << 48);
            let tiles: Vec<(u16, [u8; 64])> = (0u16..SPRITE_CHR_TILE_COUNT)
                .filter_map(|tile| {
                    let pattern =
                        decode_snes_4bpp_tile_indices(&game.ppu.vram, SPRITE_CHR_VRAM_BASE, tile);
                    if pattern == [0u8; 64] {
                        None
                    } else {
                        Some((tile, pattern))
                    }
                })
                .collect();
            (context, tiles)
        }));
        if let Ok((context, tiles)) = result {
            if !seen_contexts.insert(context) {
                continue;
            }
            for (tile, pattern) in tiles {
                let id = *index_by_pattern.entry(pattern).or_insert_with(|| {
                    cells.push(pattern);
                    keys_by_cell.push(BTreeSet::new());
                    cells.len() - 1
                });
                keys_by_cell[id].insert((context, tile));
            }
        }
    }

    // Walk overworld screens 0..0x80.
    for screen in 0u16..0x80 {
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut game = load_translated_replay_state(rom);
            game.parity_probe_overworld_screen_and_build_map(screen);
            let context = (game.parity_probe_sprite_graphics_subset(0) as u64)
                | ((game.parity_probe_sprite_graphics_subset(1) as u64) << 16)
                | ((game.parity_probe_sprite_graphics_subset(2) as u64) << 32)
                | ((game.parity_probe_sprite_graphics_subset(3) as u64) << 48);
            let tiles: Vec<(u16, [u8; 64])> = (0u16..SPRITE_CHR_TILE_COUNT)
                .filter_map(|tile| {
                    let pattern =
                        decode_snes_4bpp_tile_indices(&game.ppu.vram, SPRITE_CHR_VRAM_BASE, tile);
                    if pattern == [0u8; 64] {
                        None
                    } else {
                        Some((tile, pattern))
                    }
                })
                .collect();
            (context, tiles)
        }));
        if let Ok((context, tiles)) = result {
            if !seen_contexts.insert(context) {
                continue;
            }
            for (tile, pattern) in tiles {
                let id = *index_by_pattern.entry(pattern).or_insert_with(|| {
                    cells.push(pattern);
                    keys_by_cell.push(BTreeSet::new());
                    cells.len() - 1
                });
                keys_by_cell[id].insert((context, tile));
            }
        }
    }

    panic::set_hook(original_hook);

    let cell_count = cells.len();
    let context_count = seen_contexts.len();

    let mut bin = Vec::with_capacity(cell_count * 64);
    for pattern in &cells {
        bin.extend_from_slice(pattern);
    }
    if let Err(e) = fs::write(INDEX_BIN, &bin) {
        eprintln!("failed to write sprite index atlas bin {INDEX_BIN}: {e}");
        process::exit(1);
    }

    let manifest = SpriteIndexTileAtlasManifest {
        format: "zelda3_sprite_index_tiles_v1",
        tile_width_px: 8,
        tile_height_px: 8,
        cell_count: cell_count as u32,
        cells: keys_by_cell
            .iter()
            .enumerate()
            .map(|(id, keys)| SpriteIndexTileCellManifest {
                id: id as u32,
                keys: keys
                    .iter()
                    .map(|&(context, tile)| SpriteIndexKey { context, tile })
                    .collect(),
            })
            .collect(),
    };
    let index_json = match serde_json::to_vec_pretty(&manifest) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("failed to serialize sprite index tile manifest: {e}");
            process::exit(1);
        }
    };
    if let Err(e) = fs::write(INDEX_JSON, &index_json) {
        eprintln!("failed to write sprite index atlas json {INDEX_JSON}: {e}");
        process::exit(1);
    }

    println!("dumped sprite index atlas cells={cell_count} contexts={context_count}");
}

#[derive(Debug, Serialize)]
struct SpriteSheetPngManifest {
    format: &'static str,
    tile_width_px: u8,
    tile_height_px: u8,
    cell_count: u32,
    columns: u32,
    cells: Vec<SpriteSheetPngCell>,
}

#[derive(Debug, Serialize)]
struct SpriteSheetPngCell {
    id: u32,
    atlas_x_px: u32,
    atlas_y_px: u32,
}

#[derive(Debug, Serialize)]
struct AssetsBySourceManifest {
    format: &'static str,
    cell_count: u32,
    cells: Vec<AssetsBySourceCell>,
}

#[derive(Debug, Serialize)]
struct AssetsBySourceCell {
    id: u32,
    key: u64,
    kind: u8,
    pack: u16,
    tile_off: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PaletteUsageKey {
    source_kind: &'static str,
    asset: &'static str,
    pack: u16,
    tile: u16,
    bpp: u8,
    preview_palette: &'static str,
    preview_palette_row: u8,
}

#[derive(Debug, Serialize)]
struct PaletteUsageManifest {
    format: &'static str,
    entries: Vec<PaletteUsageEntry>,
}

#[derive(Debug, Serialize)]
struct PaletteUsageEntry {
    source_kind: &'static str,
    asset: &'static str,
    pack: u16,
    tile: u16,
    bpp: u8,
    preview_palette: &'static str,
    preview_palette_row: u8,
    evidence_count: u32,
}

fn palette_usage_key_from_chr_source(
    src: zelda3::LogicalChrSrc,
    preview_palette: &'static str,
    preview_palette_row: u8,
) -> Option<PaletteUsageKey> {
    let (source_kind, asset) = match src.kind {
        zelda3::CHR_KIND_BG => ("bg", "kBgGfx"),
        zelda3::CHR_KIND_SPRITE => ("sprite", "kSprGfx"),
        _ => return None,
    };
    Some(PaletteUsageKey {
        source_kind,
        asset,
        pack: src.pack,
        tile: src.tile_off,
        bpp: 3,
        preview_palette,
        preview_palette_row,
    })
}

fn record_palette_usage_count(
    counts: &mut HashMap<PaletteUsageKey, u32>,
    src: zelda3::LogicalChrSrc,
    preview_palette: &'static str,
    preview_palette_row: u8,
) {
    if let Some(key) = palette_usage_key_from_chr_source(src, preview_palette, preview_palette_row)
    {
        *counts.entry(key).or_insert(0) += 1;
    }
}

fn palette_usage_entries_from_counts(
    counts: &HashMap<PaletteUsageKey, u32>,
) -> Vec<PaletteUsageEntry> {
    let mut best_by_tile: HashMap<
        (&'static str, &'static str, u16, u16, u8),
        (PaletteUsageKey, u32),
    > = HashMap::new();
    for (&key, &count) in counts {
        let tile_key = (key.source_kind, key.asset, key.pack, key.tile, key.bpp);
        match best_by_tile.get(&tile_key) {
            Some((best_key, best_count))
                if count < *best_count
                    || (count == *best_count
                        && (key.preview_palette, key.preview_palette_row)
                            >= (best_key.preview_palette, best_key.preview_palette_row)) => {}
            _ => {
                best_by_tile.insert(tile_key, (key, count));
            }
        }
    }

    let mut entries: Vec<_> = best_by_tile
        .into_values()
        .map(|(key, evidence_count)| PaletteUsageEntry {
            source_kind: key.source_kind,
            asset: key.asset,
            pack: key.pack,
            tile: key.tile,
            bpp: key.bpp,
            preview_palette: key.preview_palette,
            preview_palette_row: key.preview_palette_row,
            evidence_count,
        })
        .collect();
    entries.sort_by_key(|entry| {
        (
            entry.source_kind,
            entry.asset,
            entry.pack,
            entry.tile,
            entry.bpp,
            entry.preview_palette,
            entry.preview_palette_row,
        )
    });
    entries
}

#[cfg(test)]
mod palette_usage_tests {
    use super::*;

    #[test]
    fn raw_chr_sources_map_to_base_tile_usage_keys() {
        let bg = zelda3::LogicalChrSrc {
            kind: zelda3::CHR_KIND_BG,
            pack: 5,
            tile_off: 17,
        };
        let sprite = zelda3::LogicalChrSrc {
            kind: zelda3::CHR_KIND_SPRITE,
            pack: 12,
            tile_off: 3,
        };
        let streamed = zelda3::LogicalChrSrc {
            kind: zelda3::CHR_KIND_BG_STREAM,
            pack: 0x1234,
            tile_off: 0x5678,
        };

        self::assert_palette_usage_key(
            palette_usage_key_from_chr_source(bg, "palette_dung_bg_main", 2),
            "bg",
            "kBgGfx",
            5,
            17,
            "palette_dung_bg_main",
            2,
        );
        self::assert_palette_usage_key(
            palette_usage_key_from_chr_source(sprite, "palette_main_spr", 6),
            "sprite",
            "kSprGfx",
            12,
            3,
            "palette_main_spr",
            6,
        );
        assert!(palette_usage_key_from_chr_source(streamed, "palette_dung_bg_main", 1).is_none());
    }

    fn assert_palette_usage_key(
        got: Option<PaletteUsageKey>,
        source_kind: &str,
        asset: &str,
        pack: u16,
        tile: u16,
        palette: &str,
        palette_row: u8,
    ) {
        let got = got.expect("expected usage key");
        assert_eq!(got.source_kind, source_kind);
        assert_eq!(got.asset, asset);
        assert_eq!(got.pack, pack);
        assert_eq!(got.tile, tile);
        assert_eq!(got.bpp, 3);
        assert_eq!(got.preview_palette, palette);
        assert_eq!(got.preview_palette_row, palette_row);
    }
}

/// Walk the combined-route replay and dump an asset library keyed by the LOGICAL
/// CHR SOURCE (Milestone 2 of the animation-modeled asset renderer), NOT by VRAM
/// appearance.
///
/// At each frame the CHR tile slots actually USED that frame are enumerated by
/// walking the three BG tilemaps + OAM and mapping every referenced tile back to
/// its VRAM CHR slot (`tile_word_base / 16`). For each used slot the M1
/// bookkeeping table (`game.vram_chr_source()`) names the logical source that
/// filled it (`kind/pack/tile_off`); slots with no recorded source (`kind == 0`)
/// are skipped. Each unique logical source key
/// (`(kind<<24)|(pack<<8)|(tile_off&0xff)`) becomes one cell, whose 8×8 4bpp
/// palette-index pattern is decoded offline from live VRAM at that slot.
///
/// Emits `developer_tilesets/assets_by_source.{bin,json}`.
fn run_dump_assets_by_source(args: &[String]) {
    use renderer::modern_extract::decode_snes_2bpp_tile_indices;
    use renderer::modern_source_atlas::modern_source_key;
    use zelda3::{
        chr_content_hash32, CHR_KIND_BG, CHR_KIND_BG3, CHR_KIND_BG_STREAM, CHR_KIND_LINK,
        CHR_KIND_NONE, CHR_KIND_SPRITE,
    };

    // Content-hashed slots (CHR_KIND_BG_STREAM: streamed dungeon BG + all sprite CHR)
    // are keyed by a hash of their pixels. The source-table tag is computed at
    // DMA/rehash time; if the slot is rewritten in-place before frame-end (when this
    // dump decodes the cell), the tag and the captured pixels DESYNC — keep-first then
    // stores a cell under a key that does not match its pixels, and the render (which
    // trusts the live source-table tag) resolves the wrong cell. Re-derive the key from
    // the ACTUAL captured frame-end pixels so the atlas is self-consistent
    // (atlas[hash(W)] == decode(W) for the same 16 words W the render hashes live).
    let rekey_content_hash = |vram: &[u16], slot: usize, src: zelda3::LogicalChrSrc| -> u64 {
        if src.kind == CHR_KIND_BG_STREAM {
            let base = slot * 16;
            if base + 16 <= vram.len() {
                let h = chr_content_hash32(&vram[base..base + 16]);
                return modern_source_key(
                    CHR_KIND_BG_STREAM,
                    (h >> 16) as u16,
                    (h & 0xffff) as u16,
                );
            }
        }
        modern_source_key(src.kind, src.pack, src.tile_off)
    };

    /// Hardware OBJ sizes by `obj_size` (small, large) — mirrors SPRITE_SIZES.
    const SPRITE_SIZES: [[i32; 2]; 8] = [
        [8, 16],
        [8, 32],
        [8, 64],
        [16, 32],
        [16, 64],
        [32, 64],
        [16, 32],
        [16, 32],
    ];

    const OUT_PNG: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/assets_by_source.png"
    );
    const OUT_JSON: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/assets_by_source.json"
    );
    const PALETTE_USAGE_OUT_JSON: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../generated/zelda3_assets/atlas/palette_usage.json"
    );
    let rom = concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc");
    let replay = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../saves/zelda3-combined-route.sav"
    );

    let max_frames = args
        .first()
        .map(|s| {
            s.parse::<u32>().unwrap_or_else(|_| {
                eprintln!("invalid frame count: {s}");
                process::exit(2);
            })
        })
        .unwrap_or(60_000);

    // Optional single-key watcher (ZELDA3_DUMP_WATCH_KEY=0x<u64key>): logs every
    // distinct decoded pixel pattern seen under that source key across the route,
    // with the frame range each pattern appeared in. Distinguishes an AMBIGUOUS
    // key (>1 distinct pattern => genuine non-injectivity/collision) from a
    // stale-tag or gap issue (exactly 1 pattern => the key is injective and the
    // render mismatch lives elsewhere). Pattern id = FNV-1a of the 64 index bytes.
    let watch_key: Option<u64> = std::env::var("ZELDA3_DUMP_WATCH_KEY")
        .ok()
        .and_then(|s| u64::from_str_radix(s.trim().trim_start_matches("0x"), 16).ok());
    // pattern_hash -> (first_frame, last_frame, count, first_slot, ctx@first)
    // ctx = (main_module, submodule, indoor, animated_tile_pack) captured at the
    // FIRST frame the pattern appeared, to reveal what game state produced it.
    #[allow(clippy::type_complexity)]
    let mut watch_patterns: std::collections::BTreeMap<
        u32,
        (u32, u32, u32, usize, u8, u8, u8, u16),
    > = std::collections::BTreeMap::new();

    // key -> cell id, and the parallel cell pattern store (64 index bytes each).
    let mut cell_by_key: HashMap<u64, usize> = HashMap::new();
    let mut cells: Vec<[u8; 64]> = Vec::new();
    let mut collisions: usize = 0;
    let mut palette_usage_counts: HashMap<PaletteUsageKey, u32> = HashMap::new();
    // Keys whose decoded pattern was NOT stable across the route (the keep-first
    // representative differs from a later occurrence). For BG3 these are dropped
    // from the atlas: BG3 CHR is reused, so a non-injective (tile_number, palette)
    // key would otherwise overdraw the play area with a stale glyph. Stable HUD
    // glyphs survive; ambiguous BG3 tiles become transparent gaps (which is what
    // the classic renders for those play-area tiles).
    let mut ambiguous_keys: std::collections::HashSet<u64> = std::collections::HashSet::new();

    // Dedup one cell per logical-source key, keep-first on pattern collision.
    let mut record_keyed =
        |key: u64, pattern: [u8; 64], dbg_slot: usize| match cell_by_key.get(&key) {
            Some(&id) => {
                if cells[id] != pattern {
                    collisions += 1;
                    ambiguous_keys.insert(key);
                    if collisions <= 10 {
                        eprintln!(
                            "[warn] key 0x{key:016x} decoded to a different pattern at \
                             slot {dbg_slot:#x}; keeping first"
                        );
                    }
                }
            }
            None => {
                let id = cells.len();
                cell_by_key.insert(key, id);
                cells.push(pattern);
            }
        };

    let mut collect_used_slots = |game: &ZeldaState, cur_frame: u32| {
        let ppu = &game.ppu;

        // --- BG tilemaps (all 3 layers) ---
        for layer_index in 0..3usize {
            let bg = &ppu.bg_layer[layer_index];
            let base = bg.tilemap_adr as usize;
            let chr_base = bg.tile_adr as usize;
            if base == 0 && chr_base == 0 {
                continue;
            }
            // BG3 (mode 1) is the 2bpp HUD/font layer: its tiles are 8 words and
            // STATIC, so they are keyed directly by `(tile_number, palette)` (kind
            // BG3) rather than via the 16-word per-slot CHR source table, and decoded
            // as 2bpp with the classic BG3->CGRAM mapping (cgram = palette*4 + p)
            // baked in so the render path can emit palette 0. See `CHR_KIND_BG3`.
            let is_bg3 = layer_index == 2;
            let wide = bg.tilemap_wider;
            let tall = bg.tilemap_higher;
            let cols = if wide { 64usize } else { 32 };
            let rows = if tall { 64usize } else { 32 };
            for ty in 0..rows {
                for tx in 0..cols {
                    let q = (if wide && tx >= 32 { 1 } else { 0 })
                        + (if tall && ty >= 32 {
                            if wide {
                                2
                            } else {
                                1
                            }
                        } else {
                            0
                        });
                    let within = (ty % 32) * 32 + (tx % 32);
                    let addr = base + q * 0x400 + within;
                    let entry_word = ppu.vram.get(addr).copied().unwrap_or(0);
                    let tile_number = usize::from(entry_word & 0x03ff);
                    if is_bg3 {
                        if entry_word == 0 {
                            continue;
                        }
                        let palette = ((entry_word >> 10) & 7) as u16;
                        let pack = (tile_number as u16) | (palette << 10);
                        let key = modern_source_key(CHR_KIND_BG3, pack, 0);
                        // 2bpp decode UNFLIPPED (flip baked at render time), then bake
                        // the BG3->CGRAM palette mapping into the indices.
                        let raw =
                            decode_snes_2bpp_tile_indices(&ppu.vram, chr_base, entry_word & 0x03ff);
                        let mut baked = [0u8; 64];
                        for (b, &p) in baked.iter_mut().zip(raw.iter()) {
                            *b = if p == 0 { 0 } else { (palette as u8) * 4 + p };
                        }
                        record_keyed(key, baked, chr_base + tile_number * 8);
                        continue;
                    }
                    let slot = (chr_base + tile_number * 16) / 16;
                    let src = game.vram_chr_source().get(slot);
                    if src.kind == CHR_KIND_NONE {
                        continue;
                    }
                    let palette_row = ((entry_word >> 10) & 7) as u8;
                    let scene = renderer::ModernAssetFrameScene::from_player_indoors_flag(
                        game.ram.get(PLAYER_IS_INDOORS).copied().unwrap_or(0),
                    );
                    let preview_src = game.vram_chr_preview_source().get(slot);
                    let usage_src =
                        if src.kind == CHR_KIND_BG_STREAM && preview_src.kind == CHR_KIND_BG {
                            preview_src
                        } else {
                            src
                        };
                    record_palette_usage_count(
                        &mut palette_usage_counts,
                        usage_src,
                        scene.bg_palette_name(),
                        palette_row,
                    );
                    let key = rekey_content_hash(&ppu.vram, slot, src);
                    let pattern = decode_snes_4bpp_tile_indices(&ppu.vram, slot * 16, 0);
                    if watch_key == Some(key) {
                        let mut h: u32 = 0x811c_9dc5;
                        for &b in pattern.iter() {
                            h ^= b as u32;
                            h = h.wrapping_mul(0x0100_0193);
                        }
                        let module = game.ram.get(0x10).copied().unwrap_or(0);
                        let submodule = game.ram.get(0x11).copied().unwrap_or(0);
                        let indoor = game.ram.get(0x1b).copied().unwrap_or(0);
                        let anim_pack = game.animated_tile_pack;
                        let e = watch_patterns.entry(h).or_insert((
                            cur_frame, cur_frame, 0, slot, module, submodule, indoor, anim_pack,
                        ));
                        e.1 = cur_frame;
                        e.2 += 1;
                    }
                    record_keyed(key, pattern, slot);
                }
            }
        }

        // --- OAM (sprites, incl. Link) ---
        for sprite_num in 0..128usize {
            let idx = sprite_num * 2;
            let oam0 = ppu.oam.get(idx).copied().unwrap_or(0);
            let y_byte = ((oam0 >> 8) & 0xff) as i32;
            if y_byte == 0xf0 {
                continue; // off-screen sentinel
            }
            let hi_word = ppu.oam.get(0x100 + idx / 16).copied().unwrap_or(0);
            let hi_bits = (hi_word >> (idx % 16)) as i32;
            let size = SPRITE_SIZES[(ppu.obj_size & 7) as usize][((hi_bits >> 1) & 1) as usize];
            let object_x = (oam0 & 0xff) as i32 + (hi_bits & 1) * 256;
            if object_x > 256 && object_x + size - 1 < 512 {
                continue;
            }
            let mut x = object_x;
            if x >= 256 {
                x -= 512;
            }
            if x <= -size {
                continue;
            }
            let oam1 = ppu.oam.get(idx + 1).copied().unwrap_or(0);
            let obj_addr = if oam1 & 0x0100 != 0 {
                ppu.obj_tile_adr2
            } else {
                ppu.obj_tile_adr1
            };
            let tile_row_base = ((oam1 & 0xff) >> 4) as i32;
            let tile_col_base = (oam1 & 0x0f) as i32;
            let tiles_per_side = size / 8;
            for sty in 0..tiles_per_side {
                for stx in 0..tiles_per_side {
                    let used_tile =
                        (((tile_row_base + sty) << 4) | ((tile_col_base + stx) & 0x0f)) as u16;
                    let tile_word_base =
                        obj_addr.wrapping_add(used_tile.wrapping_mul(16)) as usize & 0x7fff;
                    let slot = tile_word_base / 16;
                    let src = game.vram_chr_source().get(slot);
                    if src.kind == CHR_KIND_NONE {
                        continue;
                    }
                    let preview_src = game.vram_chr_preview_source().get(slot);
                    let usage_src =
                        if src.kind == CHR_KIND_BG_STREAM && preview_src.kind == CHR_KIND_SPRITE {
                            preview_src
                        } else {
                            src
                        };
                    let palette_row = ((oam1 >> 9) & 7) as u8;
                    record_palette_usage_count(
                        &mut palette_usage_counts,
                        usage_src,
                        renderer::ModernAssetFrameScene::SPRITE_PALETTE_NAME,
                        palette_row,
                    );
                    let key = rekey_content_hash(&ppu.vram, slot, src);
                    let pattern = decode_snes_4bpp_tile_indices(&ppu.vram, slot * 16, 0);
                    record_keyed(key, pattern, slot);
                }
            }
        }
    };

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    let walk = panic::catch_unwind(AssertUnwindSafe(|| {
        let mut game = load_translated_replay_state(rom);
        if let Err(e) = game.replay_save_file(Path::new(replay)) {
            eprintln!("failed to load replay save {replay}: {e}");
            process::exit(1);
        }
        let mut frames = game.state_recorder.replay_frame_counter;
        while frames < max_frames && game.state_recorder.replay_mode {
            let step = panic::catch_unwind(AssertUnwindSafe(|| {
                game.zelda_run_frame_with_replay_input_override(0, None);
            }));
            if step.is_err() {
                eprintln!("[warn] replay frame {frames} panicked; stopping walk early");
                break;
            }
            frames = frames.wrapping_add(1);
            collect_used_slots(&game, frames);
        }
        frames
    }));

    panic::set_hook(original_hook);

    let frames_walked = match walk {
        Ok(f) => f,
        Err(_) => {
            eprintln!("assets-by-source walk aborted by panic");
            process::exit(1);
        }
    };

    if let Some(wk) = watch_key {
        eprintln!(
            "[WATCH] key 0x{wk:016x}: {} distinct pattern(s) over the route{}",
            watch_patterns.len(),
            if watch_patterns.len() > 1 {
                "  => AMBIGUOUS KEY (non-injective / collision)"
            } else {
                "  => injective (mismatch is elsewhere: stale tag or gap)"
            }
        );
        for (h, (first, last, count, slot, module, submodule, indoor, anim_pack)) in &watch_patterns
        {
            eprintln!(
                "[WATCH]   pattern 0x{h:08x}: frames {first}..{last} (x{count}), first slot 0x{slot:03x} \
                 | @first: module=0x{module:02x} submodule=0x{submodule:02x} indoor={indoor} anim_pack=0x{anim_pack:04x}"
            );
        }
    }

    // Build the canonical bin + json manifest. Ambiguous BG3 keys (reused CHR,
    // non-injective by tile_number) are DROPPED so they render as transparent gaps
    // instead of overdrawing the play area with a stale glyph; cells are then
    // re-indexed densely so each manifest `id` is its 64-byte slot in the bin.
    let mut bin = Vec::with_capacity(cells.len() * 64);
    let mut manifest_cells = Vec::with_capacity(cells.len());
    let mut count_bg = 0usize;
    let mut count_sprite = 0usize;
    let mut count_link = 0usize;
    let mut count_bg3 = 0usize;
    let mut dropped_bg3 = 0usize;
    // Reconstruct each cell's key (cell id order == insertion order); invert map.
    let mut key_by_id = vec![0u64; cells.len()];
    for (&key, &id) in &cell_by_key {
        key_by_id[id] = key;
    }
    for (id, pattern) in cells.iter().enumerate() {
        let key = key_by_id[id];
        // Link keys live in the `kind<<24` namespace (< 2^32, since the only Link
        // kind value is 3); every other kind uses `kind<<32` (full 16+16-bit
        // pack/tile_off content-hash payload) — see `modern_source_key`. The two
        // namespaces don't overlap, so the magnitude alone distinguishes them.
        let (kind, pack, tile_off) = if key < (1u64 << 32) {
            (
                CHR_KIND_LINK,
                ((key >> 14) & 0x3ff) as u16,
                (key & 0x3fff) as u16,
            )
        } else {
            (
                (key >> 32) as u8,
                ((key >> 16) & 0xffff) as u16,
                (key & 0xffff) as u16,
            )
        };
        if kind == CHR_KIND_BG3 && ambiguous_keys.contains(&key) {
            dropped_bg3 += 1;
            continue;
        }
        let new_id = manifest_cells.len() as u32;
        bin.extend_from_slice(&pattern[..]);
        match kind {
            CHR_KIND_BG => count_bg += 1,
            CHR_KIND_SPRITE => count_sprite += 1,
            CHR_KIND_LINK => count_link += 1,
            CHR_KIND_BG3 => count_bg3 += 1,
            _ => {}
        }
        manifest_cells.push(AssetsBySourceCell {
            id: new_id,
            key,
            kind,
            pack,
            tile_off,
        });
    }
    let cell_count = manifest_cells.len();

    // Diagnostic runs (partial frame ranges, key-watching) set ZELDA3_DUMP_NO_WRITE=1
    // to avoid overwriting the committed full-route atlas with partial coverage.
    let no_write = std::env::var("ZELDA3_DUMP_NO_WRITE").is_ok();

    if !no_write {
        if let Err(e) = write_assets_index_png(OUT_PNG, &bin, cell_count) {
            eprintln!("failed to write assets index PNG {OUT_PNG}: {e}");
            process::exit(1);
        }
    }

    let manifest = AssetsBySourceManifest {
        format: "zelda3_assets_by_source_v2_png",
        cell_count: cell_count as u32,
        cells: manifest_cells,
    };
    let json = match serde_json::to_vec_pretty(&manifest) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("failed to serialize assets manifest: {e}");
            process::exit(1);
        }
    };
    if !no_write {
        if let Err(e) = fs::write(OUT_JSON, &json) {
            eprintln!("failed to write assets manifest {OUT_JSON}: {e}");
            process::exit(1);
        }
        let usage_manifest = PaletteUsageManifest {
            format: "zelda3_palette_usage_v1",
            entries: palette_usage_entries_from_counts(&palette_usage_counts),
        };
        let usage_json = match serde_json::to_vec_pretty(&usage_manifest) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("failed to serialize palette usage manifest: {e}");
                process::exit(1);
            }
        };
        if let Some(parent) = Path::new(PALETTE_USAGE_OUT_JSON).parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!(
                    "failed to create palette usage dir {}: {e}",
                    parent.display()
                );
                process::exit(1);
            }
        }
        if let Err(e) = fs::write(PALETTE_USAGE_OUT_JSON, &usage_json) {
            eprintln!("failed to write palette usage manifest {PALETTE_USAGE_OUT_JSON}: {e}");
            process::exit(1);
        }
    }
    if no_write {
        eprintln!("[dump] ZELDA3_DUMP_NO_WRITE set — atlas files NOT written (diagnostic run)");
    }

    if collisions > 0 {
        eprintln!("[warn] {collisions} source->pattern collisions (kept first per key)");
    }
    println!(
        "dumped assets-by-source cells={cell_count} kind_counts(bg/sprite/link/bg3)={count_bg}/{count_sprite}/{count_link}/{count_bg3} dropped_bg3_ambiguous={dropped_bg3} palette_usage_entries={} frames={frames_walked}",
        palette_usage_entries_from_counts(&palette_usage_counts).len()
    );
}

/// Replay to a target frame and snapshot the live CGRAM as a 256×1 RGBA "reference
/// palette" PNG — the authoring palette for HD `ArtSidecar` overrides. An artist
/// renders base art under this palette (or upscales it) and ships this same PNG as the
/// sidecar manifest's `reference_palette`; the detail-modulate shader then re-lights HD
/// art through the LIVE CGRAM every frame. Because `detail = override / reference`, art
/// authored as `reference[idx]` gives `detail == 1` → exact parity at this frame's
/// palette, and graceful recolor as the runtime palette changes.
///
/// NOTE: the shader uses a SINGLE global reference palette (256 entries), so all HD art
/// should be authored under ONE canonical CGRAM state — pick a representative frame for
/// the area/module you are upscaling.
///
/// Usage: `zelda3 --dump-reference-palette <frame> [out.png]` (needs the 7 timing-hack
/// env vars, like every replay run). Emits `developer_tilesets/reference_palette.png`.
fn run_dump_reference_palette(args: &[String]) {
    use renderer::tile_atlas::expand_cgram_to_rgba8;

    let target_frame = match args.first().map(|s| s.parse::<u32>()) {
        Some(Ok(f)) => f,
        _ => {
            eprintln!("usage: zelda3 --dump-reference-palette <frame> [out.png]");
            process::exit(2);
        }
    };
    const DEFAULT_OUT: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/reference_palette.png"
    );
    let out_path = args.get(1).map(String::as_str).unwrap_or(DEFAULT_OUT);
    let rom = concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc");
    let replay = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../saves/zelda3-combined-route.sav"
    );

    let mut game = load_translated_replay_state(rom);
    if let Err(e) = game.replay_save_file(Path::new(replay)) {
        eprintln!("failed to load replay save {replay}: {e}");
        process::exit(1);
    }
    let mut frames = game.state_recorder.replay_frame_counter;
    while frames < target_frame && game.state_recorder.replay_mode {
        game.zelda_run_frame_with_replay_input_override(0, None);
        frames = frames.wrapping_add(1);
    }
    if frames < target_frame {
        eprintln!(
            "[warn] replay ended at frame {frames} before target {target_frame}; \
             capturing CGRAM at {frames}"
        );
    }

    let rgba = expand_cgram_to_rgba8(&game.ppu.cgram);
    if let Err(e) = write_rgba_frame_png(Path::new(out_path), &rgba, 256, 1) {
        eprintln!("failed to write reference palette PNG {out_path}: {e}");
        process::exit(1);
    }
    println!("dumped reference palette frame={frames} -> {out_path} (256x1 RGBA)");
}

/// Flatten a 256-entry CGRAM RGBA table to a 256x1 RGBA PNG (mirrors
/// `run_dump_reference_palette`'s encoding, but from an already-expanded
/// `ModernFrame::cgram_rgba` rather than raw CGRAM words).
fn write_reference_palette_png(
    path: &str,
    cgram_rgba: &[[u8; 4]; 256],
) -> Result<(), Box<dyn Error>> {
    let mut rgba = Vec::with_capacity(cgram_rgba.len() * 4);
    for px in cgram_rgba {
        rgba.extend_from_slice(px);
    }
    write_rgba_frame_png(Path::new(path), &rgba, 256, 1)
}

/// Capture native composited frames for offline HD-art authoring (Task 3 of the
/// HD-art-via-ML-super-resolution pipeline). At each requested replay frame, builds
/// the SAME `GpuFrame` -> sources-extract -> `ModernFrame` pipeline the live present
/// path uses, renders it at scale 1 with HD
/// overrides disabled (native RGBA — the frame the super-resolution step ingests),
/// and writes into `hd_art/capture/`:
///   - `frame_<n>.png`         native 256x224 RGBA
///   - `frame_<n>.map.json`    `Vec<HdPlacement>` (source key -> screen rect)
///   - `reference_palette.png` 256x1 RGBA CGRAM snapshot, from the FIRST captured frame
///
/// This does not touch the render/parity path — it's a standalone offline dump.
///
/// Usage: `zelda3 --dump-hd-capture <frame> [frame...]` (needs the 7 timing-hack env
/// vars, like every replay run). Mode-7 frames are skipped (the sources path, like
/// the live present path, doesn't cover Mode 7).
fn run_dump_hd_capture(args: &[String]) {
    let targets: Vec<u32> = args.iter().filter_map(|s| s.parse::<u32>().ok()).collect();
    if targets.is_empty() {
        eprintln!("usage: zelda3 --dump-hd-capture <frame> [frame...]");
        process::exit(2);
    }
    let max_frame = *targets.iter().max().unwrap();

    let atlas = match renderer::modern_source_atlas::load_modern_source_atlas(Path::new(".")) {
        Ok(atlas) => atlas,
        Err(e) => {
            eprintln!("failed to load source atlas: {e}");
            process::exit(1);
        }
    };

    const OUT_DIR: &str = "hd_art/capture";
    if let Err(e) = fs::create_dir_all(OUT_DIR) {
        eprintln!("failed to create {OUT_DIR}: {e}");
        process::exit(1);
    }

    let rom = concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc");
    let replay = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../saves/zelda3-combined-route.sav"
    );

    let mut game = load_translated_replay_state(rom);
    if let Err(e) = game.replay_save_file(Path::new(replay)) {
        eprintln!("failed to load replay save {replay}: {e}");
        process::exit(1);
    }

    let mut first_capture = true;
    let mut captured = 0usize;
    let mut completed = game.state_recorder.replay_frame_counter;
    while completed < max_frame && game.state_recorder.replay_mode {
        game.zelda_run_frame_with_replay_input_override(0, None);
        completed = completed.wrapping_add(1);

        if !targets.contains(&completed) {
            continue;
        }

        let Some(capture) = render_hd_capture_from_game(&mut game, &atlas) else {
            eprintln!("frame {completed}: Mode 7 not supported by the sources path; skipping");
            continue;
        };
        let png_path = format!("{OUT_DIR}/frame_{completed}.png");
        if let Err(e) = write_rgba_frame_png(Path::new(&png_path), &capture.rgba, 256, 224) {
            eprintln!("failed to write {png_path}: {e}");
            process::exit(1);
        }

        let map_path = format!("{OUT_DIR}/frame_{completed}.map.json");
        let json = match serde_json::to_vec_pretty(&capture.placements) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("failed to serialize placement map: {e}");
                process::exit(1);
            }
        };
        if let Err(e) = fs::write(&map_path, &json) {
            eprintln!("failed to write {map_path}: {e}");
            process::exit(1);
        }

        // Reference palette from the FIRST captured frame's CGRAM (256x1 RGBA).
        if first_capture {
            let pal_path = format!("{OUT_DIR}/reference_palette.png");
            if let Err(e) = write_reference_palette_png(&pal_path, &capture.cgram_rgba) {
                eprintln!("failed to write {pal_path}: {e}");
                process::exit(1);
            }
            first_capture = false;
        }
        captured += 1;
        eprintln!(
            "captured frame {completed}: {} placements",
            capture.placements.len()
        );
    }

    if captured < targets.len() {
        eprintln!(
            "[warn] replay ended at frame {completed} before capturing all {} requested frame(s) ({captured} captured)",
            targets.len()
        );
    }
    println!(
        "dumped hd capture: {captured}/{} frame(s) -> {OUT_DIR}/",
        targets.len()
    );
}

/// Assemble the HD-art override manifest from a Task-3 capture + a Task-4 super-
/// resolution pass (Task 5 of the HD-art-via-ML-super-resolution pipeline). For each
/// `hd_art/capture/frame_<n>.map.json` (ascending by `n`, so the first frame a source
/// key appears in wins — deterministic keep-first), crops that frame's matching
/// `hd_art/sr/frame_<n>.x<scale>.png` at each placement's `(x,y,w,h)` (scaled up by
/// `scale`, via [`renderer::hd_authoring::slice_hd_cell`]) and writes one PNG per
/// unique source key into `hd_art/cells/`. Finishes by writing `hd_art/manifest.json`
/// (`ModernHdOverrides::load_manifest`'s format) referencing the reference palette +
/// every cell written.
///
/// This does not touch the render/parity path — it's a standalone offline tool.
///
/// Usage: `zelda3 --slice-hd-cells [scale]` (default 4; must match the scale the SR
/// frames in `hd_art/sr/` were generated at).
fn run_slice_hd_cells(args: &[String]) {
    use renderer::hd_authoring::{slice_hd_cell, HdPlacement};
    use std::collections::HashSet;

    let scale: u32 = args.first().and_then(|s| s.parse().ok()).unwrap_or(4);

    const CAPTURE_DIR: &str = "hd_art/capture";
    const SR_DIR: &str = "hd_art/sr";
    const CELLS_DIR: &str = "hd_art/cells";
    if let Err(e) = fs::create_dir_all(CELLS_DIR) {
        eprintln!("failed to create {CELLS_DIR}: {e}");
        process::exit(1);
    }

    // Collect capture frame numbers from `frame_<n>.map.json`, sorted ascending so the
    // keep-first dedup below is deterministic regardless of directory-listing order.
    let mut frame_nums: Vec<u32> = match fs::read_dir(CAPTURE_DIR) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let name = entry.file_name();
                let name = name.to_str()?;
                let n = name.strip_prefix("frame_")?.strip_suffix(".map.json")?;
                n.parse::<u32>().ok()
            })
            .collect(),
        Err(e) => {
            eprintln!("failed to read {CAPTURE_DIR}: {e}");
            process::exit(1);
        }
    };
    frame_nums.sort_unstable();

    let mut written_keys: HashSet<String> = HashSet::new();
    let mut written: Vec<(String, String)> = Vec::new();

    for n in frame_nums {
        let map_path = format!("{CAPTURE_DIR}/frame_{n}.map.json");
        let placements: Vec<HdPlacement> = match fs::read(&map_path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        {
            Some(p) => p,
            None => {
                eprintln!("[warn] failed to read/parse {map_path}; skipping frame {n}");
                continue;
            }
        };

        let sr_path = format!("{SR_DIR}/frame_{n}.x{scale}.png");
        let (sr, sr_w, sr_h) = match decode_rgba_png(Path::new(&sr_path)) {
            Some(decoded) => decoded,
            None => {
                eprintln!("[warn] SR frame {sr_path} missing/unreadable; skipping frame {n}");
                continue;
            }
        };

        for p in &placements {
            if written_keys.contains(&p.key) {
                continue;
            }
            if u64::from_str_radix(p.key.trim_start_matches("0x"), 16).is_err() {
                eprintln!("[warn] bad placement key {:?}; skipping", p.key);
                continue;
            }
            let Some(cell) = slice_hd_cell(&sr, sr_w, sr_h, p.x, p.y, p.w, p.h, scale) else {
                continue;
            };
            let rel_path = format!("cells/{}.png", p.key);
            let cell_path = format!("{CELLS_DIR}/{}.png", p.key);
            if let Err(e) = write_rgba_frame_png(
                Path::new(&cell_path),
                &cell,
                p.w as u32 * scale,
                p.h as u32 * scale,
            ) {
                eprintln!("failed to write {cell_path}: {e}");
                process::exit(1);
            }
            written_keys.insert(p.key.clone());
            written.push((p.key.clone(), rel_path));
        }
    }

    #[derive(serde::Serialize)]
    struct OutManifest {
        reference_palette: String,
        overrides: Vec<OutOverride>,
    }
    #[derive(serde::Serialize)]
    struct OutOverride {
        key: String,
        rgba: String,
    }

    let overrides: Vec<OutOverride> = written
        .iter()
        .map(|(k, path)| OutOverride {
            key: k.clone(),
            rgba: path.clone(),
        })
        .collect();
    let manifest = OutManifest {
        reference_palette: "capture/reference_palette.png".into(),
        overrides,
    };
    if let Err(e) = fs::write(
        "hd_art/manifest.json",
        serde_json::to_vec_pretty(&manifest).unwrap(),
    ) {
        eprintln!("failed to write hd_art/manifest.json: {e}");
        process::exit(1);
    }
    println!("wrote {} cells + hd_art/manifest.json", written.len());
}

/// Decode any RGBA8 PNG to `(rgba, width, height)`; RGB is expanded to RGBA (alpha
/// 0xff). Used by [`run_slice_hd_cells`] to read the super-resolved source frames.
fn decode_rgba_png(path: &Path) -> Option<(Vec<u8>, u32, u32)> {
    let file = fs::File::open(path).ok()?;
    let decoder = png::Decoder::new(std::io::BufReader::new(file));
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let bytes = &buf[..info.buffer_size()];
    let rgba = match (info.color_type, info.bit_depth) {
        (png::ColorType::Rgba, png::BitDepth::Eight) => bytes.to_vec(),
        (png::ColorType::Rgb, png::BitDepth::Eight) => {
            let mut rgba = Vec::with_capacity((info.width * info.height * 4) as usize);
            for rgb in bytes.chunks_exact(3) {
                rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 0xff]);
            }
            rgba
        }
        (color, depth) => {
            eprintln!(
                "{}: unsupported PNG format {color:?}/{depth:?}",
                path.display()
            );
            return None;
        }
    };
    Some((rgba, info.width, info.height))
}

/// Walk the combined-route replay and extract a REAL colored sprite sheet: every
/// visible OAM 8x8 tile is decoded from live VRAM, colored with the live sprite
/// palette (CGRAM), and deduped by its 8x8 RGBA appearance so each unique colored
/// pose = one cell (captures all of Link's animation poses + every sprite seen).
///
/// Emits `developer_tilesets/sprite_sheet.{png,json}`.
fn run_dump_sprite_sheet_png(args: &[String]) {
    use renderer::modern_palette::snes_cgram_to_rgba;

    /// Hardware OBJ sizes by `obj_size` (small, large) — mirrors SPRITE_SIZES.
    const SPRITE_SIZES: [[i32; 2]; 8] = [
        [8, 16],
        [8, 32],
        [8, 64],
        [16, 32],
        [16, 64],
        [32, 64],
        [16, 32],
        [16, 32],
    ];

    const OUT_PNG: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/sprite_sheet.png"
    );
    const OUT_JSON: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/sprite_sheet.json"
    );
    let rom = concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc");
    let replay = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../saves/zelda3-combined-route.sav"
    );

    let max_frames = args
        .first()
        .map(|s| {
            s.parse::<u32>().unwrap_or_else(|_| {
                eprintln!("invalid frame count: {s}");
                process::exit(2);
            })
        })
        .unwrap_or(60_000);

    let mut cells: Vec<Vec<u8>> = Vec::new();
    let mut index_by_rgba: HashMap<Vec<u8>, usize> = HashMap::new();

    let mut collect_visible_sprite_tiles = |game: &ZeldaState| {
        let ppu = &game.ppu;
        for sprite_num in 0..128usize {
            let idx = sprite_num * 2;
            let oam0 = ppu.oam.get(idx).copied().unwrap_or(0);

            // Off-screen sentinel: hidden sprites are parked at y == 0xf0.
            let y_byte = ((oam0 >> 8) & 0xff) as i32;
            if y_byte == 0xf0 {
                continue;
            }
            let top_y = ((y_byte + 1) & 0xff) - 1;

            let hi_word = ppu.oam.get(0x100 + idx / 16).copied().unwrap_or(0);
            let hi_bits = (hi_word >> (idx % 16)) as i32;
            let size = SPRITE_SIZES[(ppu.obj_size & 7) as usize][((hi_bits >> 1) & 1) as usize];

            let object_x = (oam0 & 0xff) as i32 + (hi_bits & 1) * 256;
            if object_x > 256 && object_x + size - 1 < 512 {
                continue;
            }
            let mut x = object_x;
            if x >= 256 {
                x -= 512;
            }
            if x <= -size {
                continue;
            }

            let oam1 = ppu.oam.get(idx + 1).copied().unwrap_or(0);
            let hflip = oam1 & 0x4000 != 0;
            let vflip = oam1 & 0x8000 != 0;
            let palette = ((oam1 & 0x0e00) >> 9) as u32;
            let obj_addr = if oam1 & 0x0100 != 0 {
                ppu.obj_tile_adr2
            } else {
                ppu.obj_tile_adr1
            };
            let tile_row_base = ((oam1 & 0xff) >> 4) as i32;
            let tile_col_base = (oam1 & 0x0f) as i32;
            let _ = top_y; // screen position is irrelevant for an appearance-deduped sheet

            let tiles_per_side = size / 8;
            for sty in 0..tiles_per_side {
                for stx in 0..tiles_per_side {
                    let src_col_tile = if hflip { tiles_per_side - 1 - stx } else { stx };
                    let src_row_tile = if vflip { tiles_per_side - 1 - sty } else { sty };
                    let used_tile = (((tile_row_base + src_row_tile) << 4)
                        | ((tile_col_base + src_col_tile) & 0x0f))
                        as u16;
                    let tile_word_base =
                        obj_addr.wrapping_add(used_tile.wrapping_mul(16)) as usize & 0x7fff;
                    // Apply 8x8-level flips while decoding the 4bpp pattern.
                    let entry = (u16::from(hflip) * 0x4000) | (u16::from(vflip) * 0x8000);
                    let indices = decode_snes_4bpp_tile_indices(&ppu.vram, tile_word_base, entry);
                    if indices == [0u8; 64] {
                        continue;
                    }
                    let mut rgba = vec![0u8; 64 * 4];
                    for (px, &index) in indices.iter().enumerate() {
                        if index == 0 {
                            continue; // transparent
                        }
                        let cgram_idx = 0x80 + palette as usize * 16 + index as usize;
                        let color =
                            snes_cgram_to_rgba(ppu.cgram.get(cgram_idx).copied().unwrap_or(0));
                        rgba[px * 4..px * 4 + 4].copy_from_slice(&color);
                    }
                    if !index_by_rgba.contains_key(&rgba) {
                        let id = cells.len();
                        index_by_rgba.insert(rgba.clone(), id);
                        cells.push(rgba);
                    }
                }
            }
        }
    };

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    let walk = panic::catch_unwind(AssertUnwindSafe(|| {
        let mut game = load_translated_replay_state(rom);
        if let Err(e) = game.replay_save_file(Path::new(replay)) {
            eprintln!("failed to load replay save {replay}: {e}");
            process::exit(1);
        }
        let mut frames = game.state_recorder.replay_frame_counter;
        while frames < max_frames && game.state_recorder.replay_mode {
            let step = panic::catch_unwind(AssertUnwindSafe(|| {
                game.zelda_run_frame_with_replay_input_override(0, None);
            }));
            if step.is_err() {
                eprintln!("[warn] replay frame {frames} panicked; stopping walk early");
                break;
            }
            frames = frames.wrapping_add(1);
            collect_visible_sprite_tiles(&game);
        }
        frames
    }));

    panic::set_hook(original_hook);

    let frames_walked = match walk {
        Ok(f) => f,
        Err(_) => {
            eprintln!("sprite-sheet walk aborted by panic");
            process::exit(1);
        }
    };

    let cell_count = cells.len();
    let columns = 64usize;
    let rows = if cell_count == 0 {
        0
    } else {
        (cell_count + columns - 1) / columns
    };
    let tile_px = 8usize;
    let width = columns * tile_px;
    let height = rows * tile_px;
    let mut atlas = vec![0u8; width * height * 4]; // transparent background

    let mut manifest_cells = Vec::with_capacity(cell_count);
    for (id, rgba) in cells.iter().enumerate() {
        let col = id % columns;
        let row = id / columns;
        let dst_x = col * tile_px;
        let dst_y = row * tile_px;
        for y in 0..tile_px {
            for x in 0..tile_px {
                let src = (y * tile_px + x) * 4;
                let dst = ((dst_y + y) * width + dst_x + x) * 4;
                atlas[dst..dst + 4].copy_from_slice(&rgba[src..src + 4]);
            }
        }
        manifest_cells.push(SpriteSheetPngCell {
            id: id as u32,
            atlas_x_px: dst_x as u32,
            atlas_y_px: dst_y as u32,
        });
    }

    if let Err(e) = write_rgba_frame_png(Path::new(OUT_PNG), &atlas, width as u32, height as u32) {
        eprintln!("failed to write sprite sheet png {OUT_PNG}: {e}");
        process::exit(1);
    }

    let manifest = SpriteSheetPngManifest {
        format: "zelda3_sprite_sheet_png_v1",
        tile_width_px: 8,
        tile_height_px: 8,
        cell_count: cell_count as u32,
        columns: columns as u32,
        cells: manifest_cells,
    };
    let json = match serde_json::to_vec_pretty(&manifest) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("failed to serialize sprite sheet manifest: {e}");
            process::exit(1);
        }
    };
    if let Err(e) = fs::write(OUT_JSON, &json) {
        eprintln!("failed to write sprite sheet manifest {OUT_JSON}: {e}");
        process::exit(1);
    }

    println!(
        "dumped sprite sheet cells={cell_count} frames={frames_walked} png={width}x{height} columns={columns}"
    );
}

#[derive(Debug, Serialize)]
struct DungeonSheetPngManifest {
    format: &'static str,
    tile_width_px: u8,
    tile_height_px: u8,
    cell_count: u32,
    columns: u32,
    cells: Vec<DungeonSheetPngCell>,
}

#[derive(Debug, Serialize)]
struct DungeonSheetPngCell {
    id: u32,
    atlas_x_px: u32,
    atlas_y_px: u32,
}

/// Walk the combined-route replay and extract a colored dungeon BG tile sheet.
///
/// For every DUNGEON frame (main_module == 7 or 16) each BG tilemap entry is
/// decoded from live VRAM and colored with the live CGRAM:
///  - BG1 / BG2 (layer 0/1): 4bpp, palette base = ((word>>10)&7)*16.
///  - BG3 HUD (layer 2): 2bpp (8 words/tile), palette base = ((word>>10)&7)*4.
///
/// Tiles are deduped by their 8×8 RGBA appearance (not tile number); each unique
/// colored 8×8 appearance becomes one cell in the atlas.
///
/// Emits `developer_tilesets/dungeon_sheet.{png,json}`.
fn run_dump_dungeon_sheet_png(args: &[String]) {
    use renderer::modern_extract::decode_snes_2bpp_tile_indices;
    use renderer::modern_palette::snes_cgram_to_rgba;

    const OUT_PNG: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/dungeon_sheet.png"
    );
    const OUT_JSON: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/developer_tilesets/dungeon_sheet.json"
    );
    let rom = concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc");
    let replay = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../saves/zelda3-combined-route.sav"
    );

    let max_frames = args
        .first()
        .map(|s| {
            s.parse::<u32>().unwrap_or_else(|_| {
                eprintln!("invalid frame count: {s}");
                process::exit(2);
            })
        })
        .unwrap_or(60_000);

    let mut cells: Vec<Vec<u8>> = Vec::new();
    let mut index_by_rgba: HashMap<Vec<u8>, usize> = HashMap::new();

    let mut collect_dungeon_bg_tiles = |game: &ZeldaState| {
        let main_module = game.ram[TRACE_MAIN_MODULE_INDEX];
        // Only process dungeon frames.
        if main_module != 7 && main_module != 16 {
            return;
        }
        let ppu = &game.ppu;

        for layer_index in 0..3usize {
            // BG3 in PPU mode 1 is 2bpp; BG1/BG2 are 4bpp.
            let is_2bpp = layer_index == 2;
            let bg = &ppu.bg_layer[layer_index];
            let base = bg.tilemap_adr as usize;
            let chr_base = bg.tile_adr as usize;
            // Skip layers that haven't been set up yet (base==0 means the PPU
            // register was never written; VRAM[0] is usually zero/garbage).
            if base == 0 && chr_base == 0 {
                continue;
            }
            let wide = bg.tilemap_wider;
            let tall = bg.tilemap_higher;
            let cols = if wide { 64usize } else { 32 };
            let rows = if tall { 64usize } else { 32 };

            for ty in 0..rows {
                for tx in 0..cols {
                    // Quadrant layout (mirrors extract_modern_dungeon_frame_from_vram):
                    //   q0 = top-left, q1 = top-right (wide), q2 = bottom-left (tall),
                    //   q3 = bottom-right (wide+tall).
                    let q = (if wide && tx >= 32 { 1 } else { 0 })
                        + (if tall && ty >= 32 {
                            if wide {
                                2
                            } else {
                                1
                            }
                        } else {
                            0
                        });
                    let within = (ty % 32) * 32 + (tx % 32);
                    let addr = base + q * 0x400 + within;
                    let entry_word = ppu.vram.get(addr).copied().unwrap_or(0);
                    if entry_word == 0 {
                        continue;
                    }

                    // Decode 8×8 palette indices from live VRAM.
                    let indices: [u8; 64] = if is_2bpp {
                        decode_snes_2bpp_tile_indices(&ppu.vram, chr_base, entry_word)
                    } else {
                        decode_snes_4bpp_tile_indices(&ppu.vram, chr_base, entry_word)
                    };
                    if indices == [0u8; 64] {
                        continue; // fully transparent tile — skip
                    }

                    // Color with live CGRAM.
                    let palette = ((entry_word >> 10) & 7) as usize;
                    // 4bpp: 16 colors/palette (BG1/BG2 start at palette*16 in CGRAM).
                    // 2bpp: 4 colors/palette (BG3 starts at palette*4 in low CGRAM).
                    let colors_per_pal = if is_2bpp { 4usize } else { 16usize };
                    let palette_base = palette * colors_per_pal;
                    let mut rgba = vec![0u8; 64 * 4];
                    let mut all_transparent = true;
                    for (px, &index) in indices.iter().enumerate() {
                        if index == 0 {
                            continue; // palette index 0 is transparent
                        }
                        let cgram_idx = palette_base + index as usize;
                        let color =
                            snes_cgram_to_rgba(ppu.cgram.get(cgram_idx).copied().unwrap_or(0));
                        rgba[px * 4..px * 4 + 4].copy_from_slice(&color);
                        all_transparent = false;
                    }
                    if all_transparent {
                        continue;
                    }
                    if !index_by_rgba.contains_key(&rgba) {
                        let id = cells.len();
                        index_by_rgba.insert(rgba.clone(), id);
                        cells.push(rgba);
                    }
                }
            }
        }
    };

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    let walk = panic::catch_unwind(AssertUnwindSafe(|| {
        let mut game = load_translated_replay_state(rom);
        if let Err(e) = game.replay_save_file(Path::new(replay)) {
            eprintln!("failed to load replay save {replay}: {e}");
            process::exit(1);
        }
        let mut frames = game.state_recorder.replay_frame_counter;
        while frames < max_frames && game.state_recorder.replay_mode {
            let step = panic::catch_unwind(AssertUnwindSafe(|| {
                game.zelda_run_frame_with_replay_input_override(0, None);
            }));
            if step.is_err() {
                eprintln!("[warn] replay frame {frames} panicked; stopping walk early");
                break;
            }
            frames = frames.wrapping_add(1);
            collect_dungeon_bg_tiles(&game);
        }
        frames
    }));

    panic::set_hook(original_hook);

    let frames_walked = match walk {
        Ok(f) => f,
        Err(_) => {
            eprintln!("dungeon-sheet walk aborted by panic");
            process::exit(1);
        }
    };

    let cell_count = cells.len();
    let columns = 64usize;
    let rows = if cell_count == 0 {
        0
    } else {
        (cell_count + columns - 1) / columns
    };
    let tile_px = 8usize;
    let width = columns * tile_px;
    let height = rows * tile_px;
    let mut atlas = vec![0u8; width * height * 4]; // transparent background

    let mut manifest_cells = Vec::with_capacity(cell_count);
    for (id, rgba) in cells.iter().enumerate() {
        let col = id % columns;
        let row = id / columns;
        let dst_x = col * tile_px;
        let dst_y = row * tile_px;
        for y in 0..tile_px {
            for x in 0..tile_px {
                let src = (y * tile_px + x) * 4;
                let dst = ((dst_y + y) * width + dst_x + x) * 4;
                atlas[dst..dst + 4].copy_from_slice(&rgba[src..src + 4]);
            }
        }
        manifest_cells.push(DungeonSheetPngCell {
            id: id as u32,
            atlas_x_px: dst_x as u32,
            atlas_y_px: dst_y as u32,
        });
    }

    if let Err(e) = write_rgba_frame_png(Path::new(OUT_PNG), &atlas, width as u32, height as u32) {
        eprintln!("failed to write dungeon sheet png {OUT_PNG}: {e}");
        process::exit(1);
    }

    let manifest = DungeonSheetPngManifest {
        format: "zelda3_dungeon_sheet_v1",
        tile_width_px: 8,
        tile_height_px: 8,
        cell_count: cell_count as u32,
        columns: columns as u32,
        cells: manifest_cells,
    };
    let json = match serde_json::to_vec_pretty(&manifest) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("failed to serialize dungeon sheet manifest: {e}");
            process::exit(1);
        }
    };
    if let Err(e) = fs::write(OUT_JSON, &json) {
        eprintln!("failed to write dungeon sheet manifest {OUT_JSON}: {e}");
        process::exit(1);
    }

    println!(
        "dumped dungeon sheet cells={cell_count} frames={frames_walked} png={width}x{height} columns={columns}"
    );
}

/// Decode one SNES 4bpp tile into 64 palette indices (8×8, row-major, no color lookup).
/// Respects hflip (bit 14) and vflip (bit 15) in `tilemap_entry`; ignores palette/priority bits.
/// Each output byte is a raw 4-bit palette index (0..=15).
/// Task 1 spike deliverable: load + draw one real dungeon room and return its
/// blockset theme plus the decoded 4bpp index pattern for every nonzero BG1
/// tilemap entry.
///
/// Pinned facts (see `DUNGEON_BG_CHR_BASE` and `parity_probe_dungeon_load_and_draw`):
///   - Load+draw sequence: `parity_probe_dungeon_load_and_draw(room)` =
///     `Dungeon_LoadAndDrawEntranceRoom(room as u8)` (draws room_tilemaps + sets
///     palette_theme) then `InitializeTilesets()` (loads blockset CHR into VRAM).
///   - Theme/blockset key: `world.palette_theme.main_tile_theme_index()`.
///   - BG1 accessor: `parity_probe_dungeon_bg1_map8_entry(word_index)` over
///     `game_state.dungeon.room_tilemaps` (64x64 => word_index 0..0x1000).
///   - CHR base: `DUNGEON_BG_CHR_BASE` (= 0x2000 words), decoded via
///     `decode_snes_4bpp_tile_indices`.
///
/// NOTE: `room` is an ENTRANCE index (it is forwarded to
/// `Dungeon_LoadAndDrawEntranceRoom`), not a raw room-header index.
// (allow(dead_code): consumed by the Task 2 --dump-dungeon-index-tiles command.)
#[allow(dead_code)]
fn dungeon_room_index_probe(game: &mut ZeldaState, room: u16) -> (u16, Vec<(u16, [u8; 64])>) {
    let theme = game.parity_probe_dungeon_load_and_draw(room);
    // BG1 (walls/objects) and BG2 (floor) both decode from the same blockset CHR
    // loaded into VRAM at DUNGEON_BG_CHR_BASE (0x2000): at runtime the game writes
    // $210B = 0x22, so BG1 char base = 0x2<<12 = 0x2000 and BG2 char base =
    // 0x20<<8 = 0x2000 (shared). The synthetic dump load does not configure the PPU
    // char-base registers (bg_layer[*].tile_adr == 0 here), so decode against the
    // pinned base, not the register.
    let mut tiles = Vec::new();
    for word_index in 0..DUNGEON_BG1_TILEMAP_WORDS {
        let entry = game.parity_probe_dungeon_bg1_map8_entry(word_index);
        if entry != 0 {
            let pattern = decode_snes_4bpp_tile_indices(&game.ppu.vram, DUNGEON_BG_CHR_BASE, entry);
            tiles.push((entry, pattern));
        }
        let entry2 = game.parity_probe_dungeon_bg2_map8_entry(word_index);
        if entry2 != 0 {
            let pattern =
                decode_snes_4bpp_tile_indices(&game.ppu.vram, DUNGEON_BG_CHR_BASE, entry2);
            tiles.push((entry2, pattern));
        }
    }
    (theme, tiles)
}

/// SNES OBJ size table: `[small, large]` pixel dimensions per OBSEL `obj_size`
/// nibble (0..8). Mirrors `SPRITE_SIZES` in crates/renderer/src/sprite_renderer.rs
/// and the inline table in the gpu-dbg sprite scans above.
const OBJ_SIZE_TABLE: [[u32; 2]; 8] = [
    [8, 16],
    [8, 32],
    [8, 64],
    [16, 32],
    [16, 64],
    [32, 64],
    [16, 32],
    [16, 32],
];

/// One decoded 8x8 OBJ tile of a visible OAM sprite.
///
/// `bank_base` is the OBJ CHR base VRAM **word** address (`obj_tile_adr1` for
/// name-table 0, `obj_tile_adr2` for name-table 1), exactly as
/// `sprite_renderer::resolve_obj_pixels` uses it (`addr = obj_addr +
/// used_tile*16 + row`, masked to 0x7fff). `tile` is the resolved name
/// (`used_tile`, the 8x8 cell number within the bank), with NO flip bits set.
/// `indices` is the UNFLIPPED 4bpp pattern (`hflip`/`vflip` are recorded
/// separately so a cell can be deduped once and flipped at draw time).
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
struct SpriteTileProbe {
    bank_base: u16,
    tile: u16,
    hflip: bool,
    vflip: bool,
    indices: [u8; 64],
}

/// Task 1 spike deliverable: enumerate the visible OAM sprites of a *loaded*
/// area and decode each of their 8x8 OBJ tiles into 4bpp palette-index patterns.
///
/// Pinned facts (verified against `crates/renderer/src/sprite_renderer.rs`):
///   - OAM enumeration mirrors `resolve_obj_pixels` (sprite_renderer.rs:349):
///     per sprite_num 0..128, `yy = ((oam0>>8)+1)&0xff` (skip 0xf0); size from
///     `OBJ_SIZE_TABLE[obj_size&7][(hi_bits>>1)&1]`; `x = (oam0&0xff) +
///     (hi_bits&1)*256` with the same offscreen cull; per-sprite attrs come from
///     `oam1` (`tile_row_base=(oam1&0xff)>>4`, `tile_col_base=oam1&0x0f`,
///     hflip=`oam1&0x4000`, vflip=`oam1&0x8000`, bank=`oam1&0x100 ? tile_adr2 :
///     tile_adr1`). A `size`px sprite spans `size/8` tiles per axis;
///     `used_tile = ((tile_row_base+ty)<<4) | ((tile_col_base+tx)&0x0f)`.
///   - CHR addressing: `bank_base` (`obj_tile_adr1/2`) is a VRAM word address;
///     `decode_snes_4bpp_tile_indices(vram, bank_base as usize, used_tile)`
///     reads `bank_base + used_tile*16 + row` — identical to resolve_obj_pixels.
///   - Decode convention: decode the UNFLIPPED pattern (no flip bits in the
///     entry passed to the decoder) and record hflip/vflip on the probe, so a
///     cell is stored once and flipped at draw time. (The BG dump bakes flip
///     because it passes the live tilemap entry; sprites instead dedup by the
///     canonical unflipped pattern + a flip flag.)
///   - Context: a 64-bit fold of the 4 per-area sprite `graphics_subset` packs
///     (`parity_probe_sprite_graphics_subset(0..4)`), the per-area sprite CHR
///     identity used to key sprite cells: `g0 | g1<<16 | g2<<32 | g3<<48`.
///
/// The caller must have a loaded area with populated OAM + sprite CHR in VRAM
/// (e.g. a replay checkpoint advanced one frame); this fn does not load.
// (allow(dead_code): consumed by the Task 2 --dump-sprite-index-tiles command.)
#[allow(dead_code)]
fn sprite_index_probe(game: &mut ZeldaState) -> (u64, Vec<SpriteTileProbe>) {
    let context = (game.parity_probe_sprite_graphics_subset(0) as u64)
        | ((game.parity_probe_sprite_graphics_subset(1) as u64) << 16)
        | ((game.parity_probe_sprite_graphics_subset(2) as u64) << 32)
        | ((game.parity_probe_sprite_graphics_subset(3) as u64) << 48);

    let oam = &game.ppu.oam;
    let sizes = OBJ_SIZE_TABLE[(game.ppu.obj_size as usize) & 7];
    let tile_adr1 = game.ppu.obj_tile_adr1;
    let tile_adr2 = game.ppu.obj_tile_adr2;

    let mut probes = Vec::new();
    for sprite_num in 0..128usize {
        let idx = sprite_num * 2;
        let oam0 = oam.get(idx).copied().unwrap_or(0);
        let yy = (((oam0 >> 8) as i32) + 1) & 0xff;
        if yy == 0xf0 {
            continue;
        }
        let hi_word = oam.get(0x100 + idx / 16).copied().unwrap_or(0);
        let hi_bits = (hi_word >> (idx % 16)) as i32;
        let sprite_size = sizes[((hi_bits >> 1) & 1) as usize] as i32;

        // Offscreen cull, mirroring resolve_obj_pixels (extra_left_right=0 here).
        let object_x = (oam0 & 0xff) as i32 + (hi_bits & 1) * 256;
        if object_x > 256 && object_x + sprite_size - 1 < 512 {
            continue;
        }
        let mut x = object_x;
        if x >= 256 {
            x -= 512;
        }
        if x <= -sprite_size {
            continue;
        }

        let oam1 = oam.get(idx + 1).copied().unwrap_or(0);
        let tile_row_base = ((oam1 & 0xff) >> 4) as i32;
        let tile_col_base = (oam1 & 0x0f) as i32;
        let hflip = oam1 & 0x4000 != 0;
        let vflip = oam1 & 0x8000 != 0;
        let bank_base = if oam1 & 0x0100 != 0 {
            tile_adr2
        } else {
            tile_adr1
        };

        let tiles_per_axis = (sprite_size / 8).max(1);
        for ty in 0..tiles_per_axis {
            for tx in 0..tiles_per_axis {
                let used_tile =
                    (((tile_row_base + ty) << 4) | ((tile_col_base + tx) & 0x0f)) as u16;
                // Decode the UNFLIPPED pattern: no flip bits in the entry.
                let indices =
                    decode_snes_4bpp_tile_indices(&game.ppu.vram, bank_base as usize, used_tile);
                probes.push(SpriteTileProbe {
                    bank_base,
                    tile: used_tile,
                    hflip,
                    vflip,
                    indices,
                });
            }
        }
    }
    (context, probes)
}

fn decode_snes_4bpp_tile_indices(
    vram: &[u16],
    chr_base_words: usize,
    tilemap_entry: u16,
) -> [u8; 64] {
    let tile_number = usize::from(tilemap_entry & 0x03ff);
    let hflip = tilemap_entry & 0x4000 != 0;
    let vflip = tilemap_entry & 0x8000 != 0;
    let tile_base = chr_base_words + tile_number * 16;
    let mut out = [0u8; 64];
    for y in 0..8usize {
        let source_y = if vflip { 7 - y } else { y };
        let w01 = vram.get(tile_base + source_y).copied().unwrap_or(0);
        let w23 = vram.get(tile_base + 8 + source_y).copied().unwrap_or(0);
        let (bp0, bp1) = ((w01 & 0xff) as u8, (w01 >> 8) as u8);
        let (bp2, bp3) = ((w23 & 0xff) as u8, (w23 >> 8) as u8);
        for x in 0..8usize {
            let source_x = if hflip { x } else { 7 - x };
            let bit = 1u8 << source_x;
            out[y * 8 + x] = ((bp0 & bit != 0) as u8)
                | (((bp1 & bit != 0) as u8) << 1)
                | (((bp2 & bit != 0) as u8) << 2)
                | (((bp3 & bit != 0) as u8) << 3);
        }
    }
    out
}

fn draw_snes_4bpp_tilemap_entry_to_rgba(
    vram: &[u16],
    cgram: &[u16],
    chr_base_words: usize,
    tilemap_entry: u16,
    out: &mut [u8],
    out_width: usize,
    out_x: usize,
    out_y: usize,
    scale: usize,
) {
    let palette_base = usize::from((tilemap_entry >> 10) & 0x07) * 16;
    let indices = decode_snes_4bpp_tile_indices(vram, chr_base_words, tilemap_entry);
    for y in 0..8usize {
        for x in 0..8usize {
            let palette_index = usize::from(indices[y * 8 + x]);
            let color = snes_cgram_entry_to_rgba(cgram.get(palette_base + palette_index).copied());
            for yy in 0..scale {
                for xx in 0..scale {
                    let out_index =
                        ((out_y + y * scale + yy) * out_width + out_x + x * scale + xx) * 4;
                    if out_index + 4 <= out.len() {
                        out[out_index..out_index + 4].copy_from_slice(&color);
                    }
                }
            }
        }
    }
}

fn snes_cgram_entry_to_rgba(entry: Option<u16>) -> [u8; 4] {
    let entry = entry.unwrap_or(0);
    [
        ((entry & 0x1f) as u8) << 3,
        (((entry >> 5) & 0x1f) as u8) << 3,
        (((entry >> 10) & 0x1f) as u8) << 3,
        0xff,
    ]
}

fn run_play_gpu_render_compare(args: &[String]) {
    let rom_path = match args.first() {
        Some(p) => p,
        None => {
            eprintln!(
                "usage: zelda3 --play-gpu-render-compare <path-to-rom.sfc> [frames] [--input-script <path>] [--load-sram <path>] [--load-state <path>] [--stride <n>] [--modern-index-compare <n>] [--require-full-gpu-path] [--require-modern-index-parity]"
            );
            process::exit(2);
        }
    };
    let frames_to_run = args
        .get(1)
        .filter(|candidate| !candidate.starts_with("--"))
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(1);
    let mut i = if args.get(1).is_some_and(|arg| !arg.starts_with("--")) {
        2usize
    } else {
        1usize
    };
    let mut input_script = InputScript::default();
    let mut load_sram = None::<PathBuf>;
    let mut load_state = None::<PathBuf>;
    let mut stride = 1u32;
    let mut modern_render_compare = 0u32;
    let mut modern_index_compare = modern_index_compare_run_from_env();
    while i < args.len() {
        match args[i].as_str() {
            "--input-script" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--input-script requires a path");
                    process::exit(2);
                });
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
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--load-sram requires a path");
                    process::exit(2);
                });
                load_sram = Some(PathBuf::from(path));
                i += 2;
            }
            "--load-state" => {
                let path = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--load-state requires a path");
                    process::exit(2);
                });
                load_state = Some(PathBuf::from(path));
                i += 2;
            }
            "--stride" => {
                let value = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--stride requires a value");
                    process::exit(2);
                });
                stride = value.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid --stride value: {value}");
                    process::exit(2);
                });
                if stride == 0 {
                    eprintln!("--stride must be greater than zero");
                    process::exit(2);
                }
                i += 2;
            }
            "--modern-render-compare" => {
                let value = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--modern-render-compare requires a value");
                    process::exit(2);
                });
                modern_render_compare = value.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid --modern-render-compare value: {value}");
                    process::exit(2);
                });
                if modern_render_compare == 0 {
                    eprintln!("--modern-render-compare must be greater than zero");
                    process::exit(2);
                }
                i += 2;
            }
            "--modern-index-compare" => {
                let value = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("--modern-index-compare requires a value");
                    process::exit(2);
                });
                let value = value.parse::<u32>().unwrap_or_else(|_| {
                    eprintln!("invalid --modern-index-compare value: {value}");
                    process::exit(2);
                });
                if !modern_index_compare.set_stride(value) {
                    eprintln!("--modern-index-compare must be greater than zero");
                    process::exit(2);
                }
                i += 2;
            }
            "--require-full-gpu-path" => {
                modern_index_compare.set_require_full_gpu_path();
                i += 1;
            }
            "--require-modern-index-parity" => {
                modern_index_compare.set_require_modern_index_parity();
                i += 1;
            }
            flag => {
                eprintln!("unknown --play-gpu-render-compare option: {flag}");
                process::exit(2);
            }
        }
    }
    if let Err(e) = modern_index_compare.validate() {
        eprintln!("{e}");
        process::exit(2);
    }
    let modern_compare_defaults = modern_compare_mode_defaults_from_env();
    if modern_compare_defaults.enable_modern_render_compare {
        if let Some(note) = modern_compare_defaults.note {
            eprintln!("{note}");
        }
        if modern_render_compare == 0 {
            modern_render_compare = stride; // env var alone turns on the compare at the regular stride
        }
    }
    if load_state.is_some() && load_sram.is_some() {
        eprintln!("--play-gpu-render-compare cannot combine --load-sram with --load-state");
        process::exit(2);
    }
    if frames_to_run == 0 {
        println!(
            "play-gpu-render-compare completed compared=0 start_frame=0 last_frame=0 last_hash=0x00000000 mismatched_pixels=0"
        );
        return;
    }

    let (mut game, start_frame) = load_play_or_checkpoint(rom_path, load_state.as_deref());
    if let Some(path) = load_sram.as_deref() {
        let sram = read_file_or_exit(path, "SRAM");
        apply_sram_to_game_or_exit(&mut game, path, &sram);
    }

    let mut compare_session = play_gpu_render_compare_session(
        stride,
        modern_render_compare,
        modern_index_compare,
        Path::new("."),
    )
    .unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(2);
    });
    let last_panic = install_crash_panic_hook();
    for local_frame in 0..frames_to_run {
        let frame = start_frame.wrapping_add(local_frame);
        let input = input_script.input_for_frame(frame);
        let pre_frame_game = game.clone();
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            game.zelda_run_frame(input as i32);
        }));
        if let Err(payload) = result {
            let panic_info = captured_panic_from(last_panic.clone(), payload);
            print_replay_save_panic_report(&pre_frame_game, frame, &panic_info);
            process::exit(101);
        }
        let completed_frame = frame.wrapping_add(1);
        if !compare_session.compare_frame(&mut game, completed_frame) {
            process::exit(1);
        }
    }

    compare_session.emit_summaries(start_frame);
}

fn write_argb_frame_png(
    path: &Path,
    frame: &[u8],
    width: u32,
    height: u32,
) -> Result<(), Box<dyn Error>> {
    let mut rgba = Vec::with_capacity(frame.len());
    for pixel in frame.chunks_exact(4) {
        rgba.push(pixel[2]);
        rgba.push(pixel[1]);
        rgba.push(pixel[0]);
        rgba.push(0xff);
    }
    let file = fs::File::create(path)?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png = encoder.write_header()?;
    png.write_image_data(&rgba)?;
    Ok(())
}

/// Cells per row in the assets-by-source index PNG grid. The loader derives the
/// column count from the PNG width (`width / 8`), so this is a layout choice only.
const ASSETS_PNG_COLUMNS: usize = 128;

/// Encode the flat `bin` (`cell_count * 64` palette-slot indices) as a viewable
/// INDEXED PNG grid — the parity "index channel" that replaces
/// `assets_by_source.bin`. Each pixel is the 0..15 palette slot (0 = transparent);
/// the actual color comes from live CGRAM at render time, so the sheet stays
/// palette-agnostic and byte-exact. Cells are laid out `ASSETS_PNG_COLUMNS` per
/// row, 8x8 each; trailing grid slots past `cell_count` are index 0.
fn write_assets_index_png(path: &str, bin: &[u8], cell_count: usize) -> Result<(), Box<dyn Error>> {
    let cols = ASSETS_PNG_COLUMNS;
    let rows = cell_count.div_ceil(cols).max(1);
    let img_w = cols * 8;
    let img_h = rows * 8;
    let mut pixels = vec![0u8; img_w * img_h];
    for cell in 0..cell_count {
        let cx = (cell % cols) * 8;
        let cy = (cell / cols) * 8;
        for py in 0..8 {
            for px in 0..8 {
                pixels[(cy + py) * img_w + (cx + px)] = bin[cell * 64 + py * 8 + px];
            }
        }
    }
    // 32-entry viewing palette (index 0 transparent; 1..31 distinct hues). The atlas
    // stores palette SLOTS 0..31 — 0..15 for BG/sprite/Link, 0..31 for BG3 HUD cells
    // whose BG3->CGRAM mapping (palette*4 + pal_idx) is baked in — so the palette must
    // cover 0..31 for the PNG to be a valid indexed image in external viewers. These
    // colors are for human inspection only; the renderer reads the raw index.
    let mut palette = vec![0u8; 32 * 3];
    for i in 1..32usize {
        let t = (i as u8).wrapping_mul(37); // spread across 0..255 (37 is coprime to 256)
        palette[i * 3] = t;
        palette[i * 3 + 1] = t.wrapping_mul(2).wrapping_add(48);
        palette[i * 3 + 2] = 255u8.wrapping_sub(t);
    }
    let mut trns = vec![255u8; 32];
    trns[0] = 0; // index 0 → transparent

    let file = fs::File::create(path)?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, img_w as u32, img_h as u32);
    encoder.set_color(png::ColorType::Indexed);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_palette(palette);
    encoder.set_trns(trns);
    let mut png = encoder.write_header()?;
    png.write_image_data(&pixels)?;
    Ok(())
}

/// FNV-1a hash over R, G, B channels of an RGBA frame (pixel[0]=R, [1]=G, [2]=B).
///
/// Produces the same hash value as the C oracle's RGB hash for identical pixel
/// data, enabling direct comparison of render-hash lines in the parity gate.
/// Per-frame audio leaf hash: folds the same DSP/sample quantities the audio
/// trace prints, into one u32. Mirrored exactly in C (FingerprintAudioHash).
fn fingerprint_audio_hash(
    sample_checksum: u32,
    dsp_pre: u32,
    dsp_post: u32,
    dsp_write_count: u32,
    dsp_write_hash: u32,
    dsp_write_values_hash: u32,
) -> u32 {
    parity::fnv1a_u32s(&[
        sample_checksum,
        dsp_pre,
        dsp_post,
        dsp_write_count,
        dsp_write_hash,
        dsp_write_values_hash,
    ])
}

/// Write an RGBA frame to a PNG.
fn write_rgba_frame_png(
    path: &Path,
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Result<(), Box<dyn Error>> {
    let file = fs::File::create(path)?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png = encoder.write_header()?;
    png.write_image_data(rgba)?;
    Ok(())
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

        if let Some(render_diff) = compare_oracle_render_frame(
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
    let mut frontend = match NativeFrontend::new_with_options(
        width,
        height,
        NativeFrontendOptions::from_env(3, true),
    ) {
        Ok(frontend) => frontend,
        Err(e) => {
            eprintln!("failed to initialize native frontend: {e}");
            process::exit(1);
        }
    };
    let audio_samples = frontend.audio_samples_per_frame();
    let audio_channels = frontend.audio_channels();
    let mut audio = vec![0i16; audio_samples * audio_channels];
    let mut local_frame = 0u32;
    let mut input_history = Vec::new();
    let trace_live_input = env::var_os("ZELDA3_TRACE_LIVE_INPUT").is_some();
    let mut last_traced_live_input = u16::MAX;

    while !frontend.quit_requested() && frame_limit.is_none_or(|limit| local_frame < limit) {
        let frame = start_frame.wrapping_add(local_frame);
        let live_input = frontend.poll_input();
        if trace_live_input && live_input != last_traced_live_input {
            eprintln!(
                "live-input frame={frame} input=0x{live_input:04x} main={} sub={} subsub={}",
                oracle.game.ram[TRACE_MAIN_MODULE_INDEX],
                oracle.game.ram[TRACE_SUBMODULE_INDEX],
                oracle.game.ram[TRACE_SUBSUBMODULE_INDEX],
            );
            last_traced_live_input = live_input;
        }
        let input = if config.input_script.rules.is_empty() {
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
        if let Some(render_diff) = compare_oracle_render_frame(
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
        render_lockstep_frames_in_place(&mut oracle, &mut game_frame, &mut snes_frame, pitch);

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
        if let Some(capture) = &bsnes_capture {
            let compare_this_frame = frame >= config.compare_from_frame;
            if compare_this_frame && config.compare_bsnes_video {
                if let Some(video_diff) =
                    compare_bsnes_video_frame(&game_frame, width, height, capture)
                {
                    let artifact_dir = write_bsnes_parity_failure_artifacts(
                        &pre_oracle.game,
                        &oracle.game,
                        &game_frame,
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
                    let artifact_dir = write_bsnes_parity_failure_artifacts(
                        &pre_oracle.game,
                        &oracle.game,
                        &game_frame,
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
        frontend.push_audio(&audio);
        oracle.game.zelda_discard_unused_audio_frames();
        let pixels = unsafe {
            std::slice::from_raw_parts(game_frame.as_ptr().cast::<u32>(), game_frame.len() / 4)
        };
        frontend.present_frame(pixels, width, height);
        local_frame = local_frame.wrapping_add(1);
    }
}

struct RenderDiff {
    mismatched_pixels: usize,
    first_pixel: usize,
    mine_pixel: [u8; 4],
    theirs_pixel: [u8; 4],
    mine_ppu: String,
    theirs_ppu: String,
}

fn compare_oracle_render_frame(
    oracle: &LockstepOracle,
    game_frame: &mut [u8],
    snes_frame: &mut [u8],
    pitch: usize,
    width: usize,
) -> Option<RenderDiff> {
    let mut game_state = oracle.game.clone();
    let mut snes_state = snes_render_state(oracle);

    render_play_frame_bgra(&mut game_state, game_frame, pitch, PpuRenderFlags::empty());
    render_play_frame_bgra(&mut snes_state, snes_frame, pitch, PpuRenderFlags::empty());

    let mut mismatched_pixels = 0usize;
    let mut first_pixel = usize::MAX;
    let mut mine_pixel = [0; 4];
    let mut theirs_pixel = [0; 4];
    for (idx, (game_pixel, snes_pixel)) in game_frame
        .chunks_exact(4)
        .zip(snes_frame.chunks_exact(4))
        .take(width * 224)
        .enumerate()
    {
        if game_pixel != snes_pixel {
            mismatched_pixels += 1;
            if first_pixel == usize::MAX {
                first_pixel = idx;
                mine_pixel.copy_from_slice(game_pixel);
                theirs_pixel.copy_from_slice(snes_pixel);
            }
        }
    }

    (mismatched_pixels != 0).then_some(RenderDiff {
        mismatched_pixels,
        first_pixel,
        mine_pixel,
        theirs_pixel,
        mine_ppu: format_render_ppu_summary(&game_state),
        theirs_ppu: format_render_ppu_summary(&snes_state),
    })
}

fn snes_render_state(oracle: &LockstepOracle) -> ZeldaState {
    let mut snes_state = oracle.game.clone();
    snes_state.ppu = oracle.snes.ppu.clone();
    snes_state.dma = oracle.snes.dma.clone();
    snes_state.ram.copy_from_slice(&oracle.snes.ram);
    snes_state
        .ram
        .copy_within(0x1b00..0x1b00 + 224 * 2, 0x1dba0);
    snes_state.sram.copy_from_slice(&oracle.snes.cart.ram);
    snes_state
}

fn render_lockstep_frames_in_place(
    oracle: &mut LockstepOracle,
    game_frame: &mut [u8],
    snes_frame: &mut [u8],
    pitch: usize,
) {
    render_play_frame_bgra(&mut oracle.game, game_frame, pitch, PpuRenderFlags::empty());

    let mut snes_state = oracle.game.clone();
    snes_state.ppu = oracle.snes.ppu.clone();
    snes_state.dma = oracle.snes.dma.clone();
    snes_state.ram.copy_from_slice(&oracle.snes.ram);
    snes_state.sram.copy_from_slice(&oracle.snes.cart.ram);
    render_play_frame_bgra(&mut snes_state, snes_frame, pitch, PpuRenderFlags::empty());
    oracle.snes.ppu = snes_state.ppu;
    oracle.snes.dma = snes_state.dma;
    oracle.snes.ram.copy_from_slice(&snes_state.ram);
    oracle.snes.cart.ram.copy_from_slice(&snes_state.sram);
}

fn format_render_ppu_summary(state: &ZeldaState) -> String {
    let ppu = &state.ppu;
    format!(
        "mode={} forced_blank={} brightness={} screen={:02x}/{:02x} window={:02x}/{:02x} math={:02x} cg={:02x}/{:02x} fixed=({:02x},{:02x},{:02x}) m7={:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x} bg1=({:04x},{:04x},tm={:04x},chr={:04x}) bg2=({:04x},{:04x},tm={:04x},chr={:04x}) hdma={:02x} dma6={:02x}:{:04x}->{:02x} dma7={:02x}:{:04x}->{:02x} cgram0={:04x} cgram1={:04x} vram0000={:04x} vram1000={:04x}",
        ppu.mode,
        ppu.forced_blank,
        ppu.brightness,
        ppu.screen_enabled[0],
        ppu.screen_enabled[1],
        ppu.screen_windowed[0],
        ppu.screen_windowed[1],
        ppu.math_enabled,
        ppu.clip_mode,
        ppu.prevent_math_mode,
        ppu.fixed_color_r,
        ppu.fixed_color_g,
        ppu.fixed_color_b,
        ppu.m7_matrix[0] as u16,
        ppu.m7_matrix[1] as u16,
        ppu.m7_matrix[2] as u16,
        ppu.m7_matrix[3] as u16,
        ppu.m7_matrix[4] as u16,
        ppu.m7_matrix[5] as u16,
        ppu.m7_matrix[6] as u16,
        ppu.m7_matrix[7] as u16,
        ppu.bg_layer[0].h_scroll,
        ppu.bg_layer[0].v_scroll,
        ppu.bg_layer[0].tilemap_adr,
        ppu.bg_layer[0].tile_adr,
        ppu.bg_layer[1].h_scroll,
        ppu.bg_layer[1].v_scroll,
        ppu.bg_layer[1].tilemap_adr,
        ppu.bg_layer[1].tile_adr,
        state.ram[0x9b],
        state.dma.channel[6].a_bank,
        state.dma.channel[6].a_adr,
        state.dma.channel[6].b_adr,
        state.dma.channel[7].a_bank,
        state.dma.channel[7].a_adr,
        state.dma.channel[7].b_adr,
        ppu.cgram[0],
        ppu.cgram[1],
        ppu.vram[0],
        ppu.vram[0x1000],
    )
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

fn select_run_what(ram: &[u8]) -> u8 {
    let is_nmi_thread_active = ram[0x12a] != 0;
    let thread_other_stack = u16::from_le_bytes([ram[0x1f0a], ram[0x1f0b]]);
    if is_nmi_thread_active && thread_other_stack != 0x1f31 {
        RUN_POLY
    } else {
        RUN_MAIN
    }
}

#[derive(Debug, Default)]
struct InputScript {
    rules: Vec<InputRule>,
}

impl InputScript {
    fn from_path(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let mut stack = Vec::new();
        Self::from_path_inner(path.as_ref(), &mut stack)
    }

    fn from_path_inner(path: &Path, stack: &mut Vec<PathBuf>) -> Result<Self, Box<dyn Error>> {
        let path = fs::canonicalize(path)?;
        if stack.iter().any(|entry| entry == &path) {
            return Err(format!("recursive input script include: {}", path.display()).into());
        }
        stack.push(path.clone());
        let source = fs::read_to_string(&path)?;
        let script =
            Self::parse_with_base_dir(&source, path.parent().unwrap_or(Path::new(".")), stack)
                .map_err(|err| format!("{}: {err}", path.display()))?;
        stack.pop();
        Ok(script)
    }

    fn input_for_frame(&self, frame: u32) -> u16 {
        self.input_override_for_frame(frame).unwrap_or(0)
    }

    fn input_override_for_frame(&self, frame: u32) -> Option<u16> {
        self.rules
            .iter()
            .filter(|rule| rule.start <= frame && frame <= rule.end)
            .map(|rule| rule.input)
            .last()
    }

    #[cfg(test)]
    fn parse(source: &str) -> Result<Self, Box<dyn Error>> {
        Self::parse_with_base_dir(source, Path::new("."), &mut Vec::new())
    }

    fn parse_with_base_dir(
        source: &str,
        base_dir: &Path,
        stack: &mut Vec<PathBuf>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut rules = Vec::new();
        for (line_no, raw_line) in source.lines().enumerate() {
            let line = raw_line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }

            let mut parts = line.split_whitespace();
            let frame_spec = parts
                .next()
                .ok_or_else(|| format!("input script line {}: missing frame", line_no + 1))?;
            if frame_spec.eq_ignore_ascii_case("include") {
                let include_path = parts.next().ok_or_else(|| {
                    format!("input script line {}: include requires a path", line_no + 1)
                })?;
                if parts.next().is_some() {
                    return Err(format!(
                        "input script line {}: include takes one path",
                        line_no + 1
                    )
                    .into());
                }
                let include = Self::from_path_inner(&base_dir.join(include_path), stack)
                    .map_err(|err| format!("input script line {}: {err}", line_no + 1))?;
                rules.extend(include.rules);
                continue;
            }
            let buttons = parts.collect::<Vec<_>>().join("+");
            let (start, end) = parse_frame_spec(frame_spec)
                .map_err(|err| format!("input script line {}: {err}", line_no + 1))?;
            let input = parse_buttons(&buttons)
                .map_err(|err| format!("input script line {}: {err}", line_no + 1))?;
            rules.push(InputRule { start, end, input });
        }
        Ok(Self { rules })
    }
}

#[derive(Debug)]
struct InputRule {
    start: u32,
    end: u32,
    input: u16,
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

fn parse_frame_spec(spec: &str) -> Result<(u32, u32), String> {
    let parse_one = |s: &str| {
        s.parse::<u32>()
            .map_err(|_| format!("invalid frame number `{s}`"))
    };
    if let Some((start, end)) = spec.split_once("..") {
        let start = parse_one(start)?;
        let end = parse_one(end)?;
        if end < start {
            return Err(format!("invalid descending frame range `{spec}`"));
        }
        Ok((start, end))
    } else if let Some((start, end)) = spec.split_once('-') {
        let start = parse_one(start)?;
        let end = parse_one(end)?;
        if end < start {
            return Err(format!("invalid descending frame range `{spec}`"));
        }
        Ok((start, end))
    } else {
        let frame = parse_one(spec)?;
        Ok((frame, frame))
    }
}

fn parse_buttons(spec: &str) -> Result<u16, String> {
    if spec.is_empty() || spec.eq_ignore_ascii_case("none") {
        return Ok(0);
    }
    if let Some(hex) = spec.strip_prefix("0x").or_else(|| spec.strip_prefix("0X")) {
        return u16::from_str_radix(hex, 16)
            .map_err(|_| format!("invalid hex input word `{spec}`"));
    }

    let mut input = 0u16;
    for token in spec.split(['+', ',', '|']) {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        input |= match token.to_ascii_uppercase().as_str() {
            "B" => 1 << 0,
            "Y" => 1 << 1,
            "SELECT" => 1 << 2,
            "START" => 1 << 3,
            "UP" => 1 << 4,
            "DOWN" => 1 << 5,
            "LEFT" => 1 << 6,
            "RIGHT" => 1 << 7,
            "A" => 1 << 8,
            "X" => 1 << 9,
            "L" => 1 << 10,
            "R" => 1 << 11,
            "NONE" => 0,
            other => return Err(format!("unknown button `{other}`")),
        };
    }
    Ok(input)
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
    fn dungeon_room_index_probe_reads_a_real_room() {
        let mut game = load_translated_replay_state(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../saves/zelda3.sfc"
        ));
        let (theme, tiles) = dungeon_room_index_probe(&mut game, 0x0002);
        eprintln!(
            "dungeon probe: theme={theme} nonzero_bg1_entries={} first={:?}",
            tiles.len(),
            tiles.first().map(|(w, p)| (*w, &p[..8]))
        );
        assert!(theme != 0, "theme should be set for a dungeon room");
        assert!(!tiles.is_empty(), "room should have BG tiles");
        // patterns are real 4bpp indices (0..16) and not uniformly zero
        assert!(tiles.iter().all(|(_, p)| p.iter().all(|&i| i < 16)));
        assert!(tiles.iter().any(|(_, p)| p.iter().any(|&i| i != 0)));
    }

    #[test]
    fn sprite_index_probe_reads_visible_sprites() {
        // Real gameplay area with Link + NPCs/sprites: load the committed replay
        // checkpoint (sanctuary, frame 12000) which restores PPU OAM/VRAM/obj
        // regs, then advance one frame so the sprite engine rebuilds OAM.
        let mut game = load_translated_replay_state(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../saves/zelda3.sfc"
        ));
        load_replay_save_checkpoint(
            &mut game,
            Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../.cache/replay-bisect/rust-frame-12000.sav"
            )),
        )
        .expect("load sanctuary checkpoint");
        game.zelda_run_frame(0);

        let (context, probes) = sprite_index_probe(&mut game);
        let active_oam = (0..128usize)
            .filter(|&n| {
                let oam0 = game.ppu.oam.get(n * 2).copied().unwrap_or(0);
                (((oam0 >> 8) as i32) + 1) & 0xff != 0xf0
            })
            .count();
        eprintln!(
            "sprite probe: context={context:#018x} subsets={:?} obj_size={} tile_adr1={:#06x} tile_adr2={:#06x} active_oam={active_oam} probe_tiles={} first={:?}",
            [
                game.parity_probe_sprite_graphics_subset(0),
                game.parity_probe_sprite_graphics_subset(1),
                game.parity_probe_sprite_graphics_subset(2),
                game.parity_probe_sprite_graphics_subset(3),
            ],
            game.ppu.obj_size,
            game.ppu.obj_tile_adr1,
            game.ppu.obj_tile_adr2,
            probes.len(),
            probes.first().map(|p| (p.bank_base, p.tile, p.hflip, p.vflip, &p.indices[..8])),
        );

        assert!(context != 0, "sprite graphics context should be set");
        assert!(
            !probes.is_empty(),
            "area should have at least one visible sprite tile"
        );
        assert!(
            probes.iter().all(|p| p.indices.iter().all(|&i| i < 16)),
            "every index must be a 4bpp value (0..16)",
        );
        assert!(
            probes.iter().any(|p| p.indices.iter().any(|&i| i != 0)),
            "at least one sprite tile must be non-degenerate (a real sprite)",
        );
    }

    #[test]
    fn parses_named_buttons_to_snes_serial_bits() {
        assert_eq!(parse_buttons("START").unwrap(), 0x0008);
        assert_eq!(parse_buttons("A+RIGHT").unwrap(), 0x0180);
        assert_eq!(parse_buttons("B,Y,SELECT").unwrap(), 0x0007);
        assert_eq!(parse_buttons("none").unwrap(), 0);
    }

    #[test]
    fn parses_input_script_ranges_with_last_rule_winning() {
        let script = InputScript::parse(
            "
            # wake title
            10..12 START
            12 NONE
            20 A+RIGHT
            ",
        )
        .unwrap();

        assert_eq!(script.input_for_frame(9), 0);
        assert_eq!(script.input_for_frame(10), 0x0008);
        assert_eq!(script.input_for_frame(11), 0x0008);
        assert_eq!(script.input_for_frame(12), 0);
        assert_eq!(script.input_for_frame(20), 0x0180);
    }

    #[test]
    fn input_script_distinguishes_missing_frame_from_explicit_none_override() {
        let script = InputScript::parse(
            "
            10 NONE
            20 A+RIGHT
            ",
        )
        .unwrap();

        assert_eq!(script.input_override_for_frame(9), None);
        assert_eq!(script.input_override_for_frame(10), Some(0));
        assert_eq!(script.input_override_for_frame(20), Some(0x0180));
    }

    #[test]
    fn parses_input_script_includes_relative_to_parent_file() {
        let dir = env::temp_dir().join(format!("z3rs-input-test-{}", process::id()));
        fs::create_dir_all(&dir).unwrap();
        let base = dir.join("base.txt");
        let extended = dir.join("extended.txt");
        fs::write(&base, "10 START\n").unwrap();
        fs::write(&extended, "include base.txt\n20 A+RIGHT\n").unwrap();

        let script = InputScript::from_path(&extended).unwrap();

        assert_eq!(script.input_for_frame(10), 0x0008);
        assert_eq!(script.input_for_frame(20), 0x0180);
        fs::remove_file(base).unwrap();
        fs::remove_file(extended).unwrap();
        fs::remove_dir(dir).unwrap();
    }

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
    fn fingerprint_frame_filter_preserves_default_all_frames_behavior() {
        assert!(should_write_fingerprint(None, 41));
        assert!(should_write_fingerprint(Some(42), 42));
        assert!(!should_write_fingerprint(Some(42), 41));
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

#[cfg(test)]
mod decode_4bpp_tests {
    use super::*;

    #[test]
    fn decode_4bpp_indices_reads_planar_bits_and_flips() {
        // Build a 16-word tile (one SNES 4bpp tile = 16 words).
        // Words 0-7: rows 0-7, bp0=low byte, bp1=high byte.
        // Words 8-15: rows 0-7, bp2=low byte, bp3=high byte.
        // Set word 0 so bp0 has bit 7 set (0x80) → leftmost pixel of row 0 gets index 1.
        let mut vram = vec![0u16; 16];
        vram[0] = 0x0080; // bp0 = 0x80 (bit 7 set), bp1 = 0x00

        // No flip: tilemap_entry = 0 (tile 0, palette 0, hflip=false, vflip=false).
        // For pixel x=0: source_x = 7-0 = 7, bit = 0x80; bp0 & 0x80 != 0 → index bit 0 = 1 → index 1.
        let out_no_flip = decode_snes_4bpp_tile_indices(&vram, 0, 0x0000);
        assert_eq!(
            out_no_flip[0], 1,
            "no-flip: pixel (0,0) should be index 1 from bp0 bit 7"
        );
        assert!(
            out_no_flip[1..].iter().all(|&b| b == 0),
            "no-flip: all other pixels should be 0"
        );

        // H-flip: tilemap_entry = 0x4000.
        // With hflip, source_x = x (not 7-x), so pixel x=7 reads source_x=7 (bit 0x80).
        let out_hflip = decode_snes_4bpp_tile_indices(&vram, 0, 0x4000);
        assert_eq!(out_hflip[7], 1, "hflip: pixel (7,0) should be index 1");
        assert!(
            out_hflip
                .iter()
                .enumerate()
                .all(|(i, &b)| if i == 7 { b == 1 } else { b == 0 }),
            "hflip: only pixel index 7 should be 1"
        );

        // V-flip: tilemap_entry = 0x8000.
        // With vflip, display row 0 reads source row 7 and display row 7 reads source row 0.
        // Our data is at source row 0 → it appears at display row 7 → out[7*8+0] = out[56] = 1.
        let out_vflip = decode_snes_4bpp_tile_indices(&vram, 0, 0x8000);
        assert_eq!(out_vflip[56], 1, "vflip: pixel (0,7) should be index 1");
        assert!(
            out_vflip
                .iter()
                .enumerate()
                .all(|(i, &b)| if i == 56 { b == 1 } else { b == 0 }),
            "vflip: only pixel index 56 should be 1"
        );
    }
}
