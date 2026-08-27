use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use crate::developer_destinations;
use crate::gpu_capture::render_live_game_gpu_frame_rgba;
use crate::image_output::write_rgba_frame_png;
use crate::{
    load_embedded_asset_replay_state, load_replay_save_checkpoint, load_translated_replay_state,
    read_le_u16, write_le_u16,
};
use platform::{DeveloperCurrentLocation, DeveloperThumbnail};
use renderer::modern_extract::decode_snes_4bpp_tile_indices;
use renderer::modern_palette::snes_cgram_to_rgba;
use serde::{Deserialize, Serialize};
use zelda3::ZeldaState;

const PLAYER_IS_INDOORS: usize = 0x001b;
const TM_COPY: usize = 0x001c;
const TS_COPY: usize = 0x001d;
const BGMODE_COPY: usize = 0x0094;
const FLAG_UPDATE_CGRAM_IN_NMI: usize = 0x0015;
#[cfg(test)]
const TRACE_MAIN_MODULE_INDEX: usize = 0x10;
#[cfg(test)]
const TRACE_SUBMODULE_INDEX: usize = 0x11;
const MAIN_PALETTE_BUFFER: usize = 0x0c500;
const DEVELOPER_ROOM_BG_TILE_BASE: u16 = 0x2000;
const DEVELOPER_ROOM_SOURCE_BG_LAYER: usize = 1;
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
#[cfg(test)]
const MUSIC_CONTROL: usize = 0x012c;
#[cfg(test)]
const CURRENT_MUSIC_CONTROL: usize = 0x0130;
#[cfg(test)]
const LAST_MUSIC_CONTROL: usize = 0x0133;

pub(crate) fn current_developer_location_from_ram(
    ram: &[u8],
    host_frame: u32,
) -> DeveloperCurrentLocation {
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

pub(crate) fn load_developer_destination(id: &str) -> Result<(ZeldaState, u32), String> {
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
    let mut game = load_embedded_asset_replay_state(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../saves/zelda3.sfc"
    ))?;
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
    let mut source = load_embedded_asset_replay_state(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../saves/zelda3.sfc"
    ))?;
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
            let color = snes_cgram_to_rgba(
                cgram
                    .get(palette_base + palette_index)
                    .copied()
                    .unwrap_or(0),
            );
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

pub(crate) fn run_dump_developer_destination(args: &[String]) {
    let id = match args.first() {
        Some(id) => id,
        None => {
            eprintln!(
                "usage: zelda3 --dump-developer-destination <destination-id> <frames> <gpu-out.png>"
            );
            process::exit(2);
        }
    };
    let frames: u32 = match args.get(1).and_then(|s| s.parse().ok()) {
        Some(frames) => frames,
        None => {
            eprintln!(
                "usage: zelda3 --dump-developer-destination <destination-id> <frames> <gpu-out.png>"
            );
            process::exit(2);
        }
    };
    let out_path = match args.get(2) {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "usage: zelda3 --dump-developer-destination <destination-id> <frames> <gpu-out.png>"
            );
            process::exit(2);
        }
    };
    if let Some(flag) = args.get(3) {
        eprintln!("unknown dump-developer-destination option: {flag}");
        process::exit(2);
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
    for _ in 0..frames {
        game.zelda_run_frame(0);
    }
    let rgba = match render_live_game_gpu_frame_rgba(&mut game, width, height) {
        Ok(rgba) => rgba,
        Err(e) => {
            eprintln!("failed to render developer destination via modern asset GPU path: {e}");
            process::exit(1);
        }
    };
    if let Err(e) = write_rgba_frame_png(&out_path, &rgba, width, height) {
        eprintln!("failed to write {}: {e}", out_path.display());
        process::exit(1);
    }

    println!(
        "dumped developer destination {id} frames={frames} start_frame={start_frame} to {}; main={:02x}; sub={:02x}; mode={}; screen={:02x}/{:02x}; bg1_tm={:04x}; bg1_chr={:04x}; cgram_nonzero={}; oam_nonzero={}",
        out_path.display(),
        game.ram[0x10],
        game.ram[0x11],
        game.ppu.bg_mode(),
        game.ppu.screen_enabled[0],
        game.ppu.screen_enabled[1],
        game.ppu.bg_layer[0].tilemap_adr,
        game.ppu.bg_layer[0].tile_adr,
        game.ppu.cgram.iter().filter(|&&v| v != 0).count(),
        game.ppu.oam.iter().filter(|&&v| v != 0).count(),
    );
}

pub(crate) fn run_dump_developer_tileset(args: &[String]) {
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

#[cfg(test)]
mod tests {
    use super::*;

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
