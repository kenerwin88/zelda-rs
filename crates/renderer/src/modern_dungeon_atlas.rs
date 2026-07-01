use crate::modern_index_atlas::ModernIndexTile;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Atlas of all unique palette-agnostic tile patterns for dungeons.
/// The `key_to_cell` map is keyed by `((theme as u32) << 16) | (tilemap_word & 0xC3FF) as u32`
/// (blockset theme + tile number + hflip + vflip; palette and priority bits stripped).
pub struct ModernDungeonIndexAtlas {
    pub tile_width_px: u16,
    pub tile_height_px: u16,
    pub cells: Vec<ModernIndexTile>,
    key_to_cell: HashMap<u32, usize>,
}

/// Look up the index tile for a dungeon tilemap word + blockset theme,
/// ignoring palette and priority bits.
/// Returns `None` if the `(theme, graphics_key)` pair is not in the atlas.
pub fn dungeon_index_cell<'a>(
    atlas: &'a ModernDungeonIndexAtlas,
    theme: u16,
    tilemap_entry: u16,
) -> Option<&'a ModernIndexTile> {
    let key = ((theme as u32) << 16) | ((tilemap_entry & 0xC3FF) as u32);
    atlas.key_to_cell.get(&key).map(|&idx| &atlas.cells[idx])
}

/// Load the dungeon palette-index atlas from the committed assets under
/// `repo_root/zelda3-bin/developer_tilesets/`.
pub fn load_modern_dungeon_index_atlas(
    repo_root: &Path,
) -> Result<ModernDungeonIndexAtlas, String> {
    let base = repo_root.join("zelda3-bin/developer_tilesets");
    let json_path = base.join("dungeon_index_tiles.json");
    let bin_path = base.join("dungeon_index_tiles.bin");

    let json_bytes = std::fs::read(&json_path)
        .map_err(|e| format!("failed to read {}: {e}", json_path.display()))?;
    let manifest: Manifest = serde_json::from_slice(&json_bytes)
        .map_err(|e| format!("failed to parse {}: {e}", json_path.display()))?;

    let bin_bytes = std::fs::read(&bin_path)
        .map_err(|e| format!("failed to read {}: {e}", bin_path.display()))?;

    let expected_len = manifest.cell_count as usize * 64;
    if bin_bytes.len() != expected_len {
        return Err(format!(
            "index bin length {} does not match expected {} ({} cells * 64)",
            bin_bytes.len(),
            expected_len,
            manifest.cell_count,
        ));
    }

    let mut cells = Vec::with_capacity(manifest.cells.len());
    let mut key_to_cell: HashMap<u32, usize> = HashMap::new();

    for cell_json in &manifest.cells {
        let offset = cell_json.id as usize * 64;
        let mut indices = [0u8; 64];
        indices.copy_from_slice(&bin_bytes[offset..offset + 64]);
        let cell_index = cells.len();
        cells.push(ModernIndexTile {
            id: cell_json.id,
            indices,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        });
        for &key in &cell_json.keys {
            key_to_cell.insert(key, cell_index);
        }
    }

    Ok(ModernDungeonIndexAtlas {
        tile_width_px: manifest.tile_width_px,
        tile_height_px: manifest.tile_height_px,
        cells,
        key_to_cell,
    })
}

// ── JSON manifest types ───────────────────────────────────────────────────────

#[derive(Deserialize)]
struct Manifest {
    tile_width_px: u16,
    tile_height_px: u16,
    cell_count: u32,
    cells: Vec<CellJson>,
}

#[derive(Deserialize)]
struct CellJson {
    id: u32,
    keys: Vec<u32>,
}

// ── Test helpers ─────────────────────────────────────────────────────────────

#[cfg(test)]
impl ModernDungeonIndexAtlas {
    /// Construct an in-memory atlas with explicit `(packed_key, cell_index)` lookup
    /// entries, for unit tests that exercise `dungeon_index_cell`.
    pub fn from_keyed_cells_for_test(
        cells: Vec<ModernIndexTile>,
        keys: Vec<(u32, usize)>,
    ) -> ModernDungeonIndexAtlas {
        let mut key_to_cell = HashMap::new();
        for (key, cell_idx) in keys {
            key_to_cell.insert(key, cell_idx);
        }
        ModernDungeonIndexAtlas {
            tile_width_px: 8,
            tile_height_px: 8,
            cells,
            key_to_cell,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn loads_dungeon_atlas_and_resolves_by_theme() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let atlas = load_modern_dungeon_index_atlas(&root).expect("dungeon atlas loads");
        assert_eq!(atlas.tile_width_px, 8);
        assert!(!atlas.cells.is_empty());
        assert!(atlas
            .cells
            .iter()
            .all(|c| c.indices.iter().all(|&i| i < 16)));
        assert_eq!(
            dungeon_index_cell(&atlas, 0, 0x01ec).expect("resolves").id,
            0
        );
        assert_eq!(
            dungeon_index_cell(&atlas, 0, 0x01ec | 0x2000)
                .expect("priority-ignored")
                .id,
            0
        );
    }
}
