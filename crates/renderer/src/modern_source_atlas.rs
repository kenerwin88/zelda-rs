//! Off-VRAM asset atlas keyed by LOGICAL CHR SOURCE (Milestone 3 of the
//! animation-modeled asset renderer).
//!
//! Loads `zelda3-bin/developer_tilesets/assets_by_source.{bin,json}` (produced by
//! Milestone 2's `--dump-assets-by-source`): one 8x8 palette-index cell per unique
//! logical CHR source `{kind, pack, tile_off}`. The render path looks a tile's
//! VRAM slot up in the M1 source table to obtain `{kind, pack, tile_off}`, then
//! resolves the cell here — never reading VRAM pixel content.
//!
//! The lookup key matches the M2 dump exactly. For most kinds:
//! `key = (kind << 24) | (pack << 8) | (tile_off & 0xff)`.
//!
//! Link (kind 3) is the exception: a Link pose's CHR tiles are not injective by
//! `(pack, relative-tile)` — several DMA pieces of the same pose each restart
//! their relative tile index at 0, colliding to the wrong cell. The dump keys a
//! Link tile by its *source identity* (the tile offset within the buffer it was
//! DMA'd from, plus a 1-bit asset-vs-WRAM buffer flag); see
//! `chr_source::CHR_LINK_SRC_RAM_FLAG`. That needs a wider `tile_off` field, so
//! the Link key carries 14 bits of `tile_off` and a 10-bit `pack`:
//! `key = (3 << 24) | ((pack & 0x3ff) << 14) | (tile_off & 0x3fff)`.

use crate::modern_index_atlas::ModernIndexTile;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// `LogicalChrSrc::kind` for Link CHR tiles (mirrors `zelda3::CHR_KIND_LINK`;
/// the renderer crate must not depend on `zelda3`).
const CHR_KIND_LINK: u8 = 3;

/// Build the M2 lookup key from a logical CHR source triple.
pub fn modern_source_key(kind: u8, pack: u16, tile_off: u16) -> u32 {
    if kind == CHR_KIND_LINK {
        // Link: source-identity key — 10-bit pack + 14-bit `tile_off`
        // (buffer flag + per-buffer source tile index). See module docs.
        ((kind as u32) << 24) | (((pack as u32) & 0x3ff) << 14) | ((tile_off as u32) & 0x3fff)
    } else {
        ((kind as u32) << 24) | ((pack as u32) << 8) | ((tile_off as u32) & 0xff)
    }
}

/// Atlas of unique palette-agnostic 8x8 cells keyed by logical CHR source.
pub struct ModernSourceAtlas {
    pub cells: Vec<ModernIndexTile>,
    key_to_cell: HashMap<u32, usize>,
}

/// Resolve the cell for a logical CHR source `{kind, pack, tile_off}`.
/// Returns `None` if no cell was recorded for that source (render path skips it).
pub fn source_cell<'a>(
    atlas: &'a ModernSourceAtlas,
    kind: u8,
    pack: u16,
    tile_off: u16,
) -> Option<&'a ModernIndexTile> {
    let key = modern_source_key(kind, pack, tile_off);
    atlas.key_to_cell.get(&key).map(|&idx| &atlas.cells[idx])
}

/// Load the assets-by-source atlas from the committed assets under
/// `repo_root/zelda3-bin/developer_tilesets/`.
pub fn load_modern_source_atlas(repo_root: &Path) -> Result<ModernSourceAtlas, String> {
    let base = repo_root.join("zelda3-bin/developer_tilesets");
    let json_path = base.join("assets_by_source.json");
    let bin_path = base.join("assets_by_source.bin");

    let json_bytes = std::fs::read(&json_path)
        .map_err(|e| format!("failed to read {}: {e}", json_path.display()))?;
    let manifest: Manifest = serde_json::from_slice(&json_bytes)
        .map_err(|e| format!("failed to parse {}: {e}", json_path.display()))?;

    let bin_bytes = std::fs::read(&bin_path)
        .map_err(|e| format!("failed to read {}: {e}", bin_path.display()))?;

    let expected_len = manifest.cell_count as usize * 64;
    if bin_bytes.len() != expected_len {
        return Err(format!(
            "assets-by-source bin length {} does not match expected {} ({} cells * 64)",
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
        });
        // Rebuild the key from {kind, pack, tile_off} so the loader is robust even
        // if the JSON `key` field were ever stale; this matches the M2 dump.
        let key = modern_source_key(cell_json.kind, cell_json.pack, cell_json.tile_off);
        key_to_cell.insert(key, cell_index);
    }

    Ok(ModernSourceAtlas { cells, key_to_cell })
}

// ── JSON manifest types ───────────────────────────────────────────────────────

#[derive(Deserialize)]
struct Manifest {
    cell_count: u32,
    cells: Vec<CellJson>,
}

#[derive(Deserialize)]
struct CellJson {
    id: u32,
    kind: u8,
    pack: u16,
    tile_off: u16,
}

// ── Test helpers ─────────────────────────────────────────────────────────────

#[cfg(test)]
impl ModernSourceAtlas {
    /// Construct an in-memory atlas from `(kind, pack, tile_off)` keyed cells.
    pub fn from_keyed_cells_for_test(
        cells: Vec<ModernIndexTile>,
        keys: &[(u8, u16, u16, usize)],
    ) -> Self {
        let mut key_to_cell = HashMap::new();
        for &(kind, pack, tile_off, cell_idx) in keys {
            key_to_cell.insert(modern_source_key(kind, pack, tile_off), cell_idx);
        }
        Self { cells, key_to_cell }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn source_key_matches_m2_layout() {
        // Non-Link: key = (kind<<24)|(pack<<8)|(tile_off&0xff); tile_off low 8 bits.
        assert_eq!(modern_source_key(1, 30, 44), 16784940);
        assert_eq!(modern_source_key(2, 8, 41), 33556521);
        // Non-Link tile_off high bits are dropped (only low 8 bits keyed).
        assert_eq!(
            modern_source_key(1, 5, 0x107),
            modern_source_key(1, 5, 0x07)
        );
        // Link (kind 3): 10-bit pack (bits 14-23) + 14-bit tile_off (bits 0-13).
        assert_eq!(
            modern_source_key(3, 5, 0x2103),
            (3 << 24) | (5 << 14) | 0x2103
        );
        // Link tile_off keeps 14 bits (buffer flag + index), so 0x107 != 0x07.
        assert_ne!(
            modern_source_key(3, 5, 0x107),
            modern_source_key(3, 5, 0x07)
        );
        // Link pack is masked to 10 bits; tile_off to 14.
        assert_eq!(
            modern_source_key(3, 0x405, 0x4103),
            modern_source_key(3, 0x5, 0x103)
        );
    }

    #[test]
    fn source_cell_resolves_known_key_and_misses_unknown() {
        let cell = ModernIndexTile {
            id: 7,
            indices: [9u8; 64],
        };
        let atlas = ModernSourceAtlas::from_keyed_cells_for_test(vec![cell], &[(1, 30, 44, 0)]);
        let got = source_cell(&atlas, 1, 30, 44).expect("known source resolves");
        assert_eq!(got.id, 7);
        assert_eq!(got.indices[0], 9);
        // Unknown source → None.
        assert!(source_cell(&atlas, 2, 99, 99).is_none());
        // kind=0 (none) is never recorded → None.
        assert!(source_cell(&atlas, 0, 0, 0).is_none());
    }

    #[test]
    fn loads_committed_assets_by_source_atlas() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let atlas = load_modern_source_atlas(&root).expect("source atlas loads");
        assert!(!atlas.cells.is_empty());
        // BG/sprite/link cells are 4bpp palette indices (0..15); BG3 (kind=4) HUD
        // cells bake the BG3->CGRAM mapping (palette*4 + pal_idx) into low CGRAM,
        // so their indices span 0..31. All cells stay within a 256-color CGRAM.
        assert!(atlas
            .cells
            .iter()
            .all(|c| c.indices.iter().all(|&i| i < 32)));
        // A known BG (kind=1) source from the committed manifest resolves.
        assert!(source_cell(&atlas, 1, 30, 44).is_some());
    }
}
