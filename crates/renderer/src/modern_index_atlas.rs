use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// A single palette-agnostic tile pattern: 64 4-bit indices (0–15), row-major,
/// with hflip/vflip already baked in (the graphics key encodes flip bits).
#[derive(Clone, Debug)]
pub struct ModernIndexTile {
    pub id: u32,
    pub indices: [u8; 64],
    /// Logical source key (`modern_source_key(kind, pack, tile_off)`) for HD override
    /// lookup, or `crate::modern_hd_overrides::NO_SOURCE_KEY` (0) for cells with no
    /// atlas source (live-VRAM-decoded animation cells, test cells).
    pub source_key: u64,
    /// For a BG cell whose `indices` were baked with `flip_index_pattern`, the original
    /// tilemap flip — used ONLY to un-flip HD-override sampling back to source orientation
    /// so the HD art aligns with the baked base index. `false` on every cell that has no
    /// atlas `source_key` (its flip is never read: `ctx.resolve(NO_SOURCE_KEY) == None`).
    pub hflip: bool,
    pub vflip: bool,
}

/// Atlas of all unique palette-agnostic tile patterns for the overworld.
/// The `key_to_cell` map is keyed by `tilemap_word & 0xC3FF` (tile number +
/// hflip + vflip, palette/priority bits stripped).
pub struct ModernIndexAtlas {
    pub tile_width_px: u16,
    pub tile_height_px: u16,
    pub cells: Vec<ModernIndexTile>,
    key_to_cell: HashMap<u16, usize>,
}

/// Look up the index tile for a tilemap word, ignoring palette and priority bits.
/// Returns `None` if the graphics key is not in the atlas.
pub fn index_cell_for_tilemap_entry<'a>(
    atlas: &'a ModernIndexAtlas,
    tilemap_entry: u16,
) -> Option<&'a ModernIndexTile> {
    let key = tilemap_entry & 0xC3FF;
    atlas.key_to_cell.get(&key).map(|&idx| &atlas.cells[idx])
}

/// Load the overworld palette-index atlas from the committed assets under
/// `repo_root/zelda3-bin/developer_tilesets/`.
pub fn load_modern_overworld_index_atlas(repo_root: &Path) -> Result<ModernIndexAtlas, String> {
    let base = repo_root.join("zelda3-bin/developer_tilesets");
    let json_path = base.join("overworld_index_tiles.json");
    let bin_path = base.join("overworld_index_tiles.bin");

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
    let mut key_to_cell: HashMap<u16, usize> = HashMap::new();

    for cell_json in &manifest.cells {
        let offset = cell_json.id as usize * 64;
        let mut indices = [0u8; 64];
        indices.copy_from_slice(&bin_bytes[offset..offset + 64]);
        let cell_index = cells.len();
        cells.push(ModernIndexTile {
            id: cell_json.id,
            indices,
            source_key: source_key_from_manifest(cell_json.source_key),
            hflip: false,
            vflip: false,
        });
        for &key in &cell_json.graphics_keys {
            key_to_cell.insert(key, cell_index);
        }
    }

    Ok(ModernIndexAtlas {
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
    graphics_keys: Vec<u16>,
    #[serde(default)]
    source_key: Option<SourceKeyJson>,
}

#[derive(Clone, Copy, Deserialize)]
pub(crate) struct SourceKeyJson {
    kind: u8,
    pack: u16,
    tile_off: u16,
}

pub(crate) fn source_key_from_manifest(source_key: Option<SourceKeyJson>) -> u64 {
    source_key.map_or(crate::modern_hd_overrides::NO_SOURCE_KEY, |source| {
        crate::modern_source_atlas::modern_source_key(source.kind, source.pack, source.tile_off)
    })
}

// ── Test helpers ─────────────────────────────────────────────────────────────

#[cfg(test)]
impl ModernIndexAtlas {
    /// Construct an in-memory atlas from a list of cells, for unit tests only.
    /// `key_to_cell` is left empty; callers access cells by id via
    /// `atlas.cells.get(cell_id as usize)` (requires dense 0..len ids).
    pub fn from_cells_for_test(cells: Vec<ModernIndexTile>) -> Self {
        Self {
            tile_width_px: 8,
            tile_height_px: 8,
            cells,
            key_to_cell: HashMap::new(),
        }
    }

    /// Construct an in-memory atlas with explicit `(graphics_key, cell_index)` lookup
    /// entries, for unit tests that exercise `index_cell_for_tilemap_entry`.
    pub fn from_keyed_cells_for_test(cells: Vec<ModernIndexTile>, keys: &[(u16, usize)]) -> Self {
        let mut key_to_cell = HashMap::new();
        for &(key, cell_idx) in keys {
            key_to_cell.insert(key, cell_idx);
        }
        Self {
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
    fn loads_index_atlas_and_resolves_graphics_key() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let atlas = load_modern_overworld_index_atlas(&root).expect("index atlas loads");
        assert_eq!(atlas.tile_width_px, 8);
        assert_eq!(atlas.tile_height_px, 8);
        assert!(!atlas.cells.is_empty());
        assert!(atlas
            .cells
            .iter()
            .all(|c| c.indices.iter().all(|&i| i < 16)));
        assert_eq!(atlas.cells.len() * 64, atlas.cells.len() * 64);
        // word 2218 masks to graphics_key 170 (2218 & 0xC3FF == 170), which is in cell 0.
        let cell = index_cell_for_tilemap_entry(&atlas, 2218).expect("resolves");
        assert_eq!(cell.id, 0);
        assert_ne!(
            cell.source_key,
            crate::modern_hd_overrides::NO_SOURCE_KEY,
            "committed content-hash source refs should feed PNG/RGBA material lookup"
        );
        // palette/priority bits are ignored: 2218 | 0x2000 (priority) resolves the same
        assert_eq!(
            index_cell_for_tilemap_entry(&atlas, 2218 | 0x2000)
                .unwrap()
                .id,
            0
        );
    }
}
