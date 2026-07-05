use std::collections::HashMap;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::process;

use crate::image_output::write_rgba_frame_png;
use crate::{load_translated_replay_state, parse_u16_auto};
use renderer::modern_extract::decode_snes_4bpp_tile_indices;
use renderer::modern_palette::snes_cgram_to_rgba;
use serde::Serialize;
use zelda3::ZeldaState;

const OVERWORLD_BG_CHR_BASE: usize = 0x2000;
const OVERWORLD_BG_SOURCE_LAYER: u8 = 1;
const UNIQUE_OVERWORLD_MANIFEST_SOURCE_LIMIT: usize = 32;

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

/// One cell in the palette-index overworld atlas: 64 raw palette indices (0..=15) for an 8x8 tile,
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
                    layer: OVERWORLD_BG_SOURCE_LAYER,
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
                    layer: OVERWORLD_BG_SOURCE_LAYER,
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

pub(crate) fn run_dump_unique_overworld_cells(args: &[String]) {
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

pub(crate) fn run_dump_unique_overworld_tiles(args: &[String]) {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
