use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::process;

use crate::index_source_keys::{IndexSourceKey, IndexSourceKeyMap};
use crate::load_translated_replay_state;
use renderer::modern_extract::decode_snes_4bpp_tile_indices;
use serde::Serialize;
use zelda3::ZeldaState;

// Dungeon BG CHR base (VRAM word address), pinned by the Task 1 spike. The dungeon
// blockset loads the same way the overworld does: `InitializeTilesets` writes the
// main blockset's first BG graphics pack to VRAM word 0x2000 via
// `load_background_graphics(0x2000, main_tile_set[0], ..)` (crates/zelda3/src/load_gfx.rs).
// Tile numbers (tilemap_entry & 0x3ff) index 16-word tiles from this base, so the
// main blockset spans words 0x2000..0x4000 - identical to OVERWORLD_BG_CHR_BASE.
const DUNGEON_BG_CHR_BASE: usize = 0x2000;

// BG1 dungeon tilemap is 64x64 tiles8 => 0x1000 word entries. Word index
// 0..0x1000 into game_state.dungeon.room_tilemaps BG1.
const DUNGEON_BG1_TILEMAP_WORDS: usize = 0x1000;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    source_key: Option<IndexSourceKey>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    source_key: Option<IndexSourceKey>,
}

/// One lookup key for a sprite index cell: a `(context, tile)` pair where
/// `context = g0|(g1<<16)|(g2<<32)|(g3<<48)` (sprite graphics subsets 0..4)
/// and `tile` is the 8x8 cell offset in 0..512 from VRAM base 0x4000.
#[derive(Serialize)]
struct SpriteIndexKey {
    context: u64,
    tile: u16,
}

/// Walk all 0x128 dungeon entrance indices, dedup tiles by 64-byte pattern, and emit
/// `developer_tilesets/dungeon_index_tiles.{bin,json}`.
pub(crate) fn run_dump_dungeon_index_tiles(_args: &[String]) {
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
    let source_keys = IndexSourceKeyMap::load_from_developer_tilesets(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("developer_tilesets"),
    )
    .unwrap_or_default();
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
                source_key: source_keys.get(&cells[id]),
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
/// every non-zero 8x8 sprite CHR tile (VRAM 0x4000, 512 tiles), dedup by 64-byte
/// pattern, and emit `developer_tilesets/sprite_index_tiles.{bin,json}`.
///
/// Context key: `g0|(g1<<16)|(g2<<32)|(g3<<48)` over the 4 sprite graphics subsets
/// populated by `InitializeTilesets`; one full decode per unique context.
pub(crate) fn run_dump_sprite_index_tiles(_args: &[String]) {
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
    let source_keys = IndexSourceKeyMap::load_from_developer_tilesets(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("developer_tilesets"),
    )
    .unwrap_or_default();

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
                source_key: source_keys.get(&cells[id]),
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
        assert!(tiles.iter().all(|(_, p)| p.iter().all(|&i| i < 16)));
        assert!(tiles.iter().any(|(_, p)| p.iter().any(|&i| i != 0)));
    }
}
