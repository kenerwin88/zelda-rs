//! Off-VRAM asset atlas keyed by LOGICAL CHR SOURCE (Milestone 3 of the
//! animation-modeled asset renderer).
//!
//! Loads `zelda3-bin/developer_tilesets/assets_by_source.{bin,json}` (produced by
//! Milestone 2's `--dump-assets-by-source`): one 8x8 palette-index cell per unique
//! logical CHR source `{kind, pack, tile_off}`. The render path looks a tile's
//! VRAM slot up in the M1 source table to obtain `{kind, pack, tile_off}`, then
//! resolves the cell here — never reading VRAM pixel content.
//!
//! The lookup key matches the M2 dump exactly. For most kinds it carries the
//! FULL `pack`/`tile_off` u16 fields losslessly (needed for content-hash kinds
//! — BG_STREAM/sprite — whose `tile_off` is hash payload, not a small index;
//! truncating to 8 bits collapsed the hash to 24 effective bits and produced
//! real birthday-paradox collisions once enough distinct patterns existed
//! route-wide):
//! `key = (kind << 32) | (pack << 16) | tile_off` (u64).
//!
//! Link (kind 3) is the exception: a Link pose's CHR tiles are not injective by
//! `(pack, relative-tile)` — several DMA pieces of the same pose each restart
//! their relative tile index at 0, colliding to the wrong cell. The dump keys a
//! Link tile by its *source identity* (the tile offset within the buffer it was
//! DMA'd from, plus a 1-bit asset-vs-WRAM buffer flag); see
//! `chr_source::CHR_LINK_SRC_RAM_FLAG`. That needs a wider `tile_off` field, so
//! the Link key carries 14 bits of `tile_off` and a 10-bit `pack`, in a lower
//! bit range than the non-Link `kind << 32` keys (no collision risk between the
//! two namespaces): `key = (3 << 24) | ((pack & 0x3ff) << 14) | (tile_off & 0x3fff)`.

use crate::modern_index_atlas::ModernIndexTile;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// `LogicalChrSrc::kind` for Link CHR tiles (mirrors `zelda3::CHR_KIND_LINK`;
/// the renderer crate must not depend on `zelda3`).
const CHR_KIND_LINK: u8 = 3;

/// Build the M2 lookup key from a logical CHR source triple.
pub fn modern_source_key(kind: u8, pack: u16, tile_off: u16) -> u64 {
    if kind == CHR_KIND_LINK {
        // Link: source-identity key — 10-bit pack + 14-bit `tile_off`
        // (buffer flag + per-buffer source tile index). See module docs.
        ((kind as u64) << 24) | (((pack as u64) & 0x3ff) << 14) | ((tile_off as u64) & 0x3fff)
    } else {
        ((kind as u64) << 32) | ((pack as u64) << 16) | (tile_off as u64)
    }
}

/// Atlas of unique palette-agnostic 8x8 cells keyed by logical CHR source.
pub struct ModernSourceAtlas {
    pub cells: Vec<ModernIndexTile>,
    key_to_cell: HashMap<u64, usize>,
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
    let png_path = base.join("assets_by_source.png");

    let json_bytes = std::fs::read(&json_path)
        .map_err(|e| format!("failed to read {}: {e}", json_path.display()))?;
    let manifest: Manifest = serde_json::from_slice(&json_bytes)
        .map_err(|e| format!("failed to parse {}: {e}", json_path.display()))?;
    if manifest.format != "zelda3_assets_by_source_v2_png" {
        return Err(format!(
            "{}: unsupported format {:?}",
            json_path.display(),
            manifest.format
        ));
    }
    if manifest.cell_count != manifest.cells.len() as u32 {
        return Err(format!(
            "{}: cell_count {} does not match {} cells",
            json_path.display(),
            manifest.cell_count,
            manifest.cells.len()
        ));
    }

    // Decode the INDEXED index-channel PNG: one byte per pixel = the 0..15 palette
    // slot. Cells are 8x8, laid out `width/8` per row in dump `id` order (see
    // `write_assets_index_png`). The color comes from live CGRAM at render time, so
    // this sheet is palette-agnostic — byte-exact with the retired `.bin`.
    let file = std::fs::File::open(&png_path)
        .map_err(|e| format!("failed to open {}: {e}", png_path.display()))?;
    let decoder = png::Decoder::new(std::io::BufReader::new(file));
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("failed to read PNG header {}: {e}", png_path.display()))?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| format!("failed to decode {}: {e}", png_path.display()))?;
    if info.bit_depth != png::BitDepth::Eight || info.color_type != png::ColorType::Indexed {
        return Err(format!(
            "{}: expected an 8-bit indexed PNG, got {:?}/{:?}",
            png_path.display(),
            info.color_type,
            info.bit_depth
        ));
    }
    let width = info.width as usize;
    let height = info.height as usize;
    if width % 8 != 0 || height % 8 != 0 {
        return Err(format!(
            "{}: PNG size {}x{} is not aligned to 8x8 cells",
            png_path.display(),
            info.width,
            info.height
        ));
    }
    let cols = width / 8;
    if cols == 0 {
        return Err(format!(
            "{}: PNG width {width} is less than one 8px cell",
            png_path.display()
        ));
    }
    let rows = height / 8;
    let grid_capacity = cols * rows;
    let data = &buf[..info.buffer_size()];

    let mut cells = Vec::with_capacity(manifest.cells.len());
    let mut key_to_cell: HashMap<u64, usize> = HashMap::new();

    for cell_json in &manifest.cells {
        let id = cell_json.id as usize;
        if id >= grid_capacity {
            return Err(format!(
                "{}: cell id {} is outside PNG grid capacity {} ({}x{} cells)",
                json_path.display(),
                cell_json.id,
                grid_capacity,
                cols,
                rows
            ));
        }
        let cx = (id % cols) * 8;
        let cy = (id / cols) * 8;
        let mut indices = [0u8; 64];
        for py in 0..8 {
            let row_start = (cy + py) * width + cx;
            indices[py * 8..py * 8 + 8].copy_from_slice(&data[row_start..row_start + 8]);
        }
        let cell_index = cells.len();
        cells.push(ModernIndexTile {
            id: cell_json.id,
            indices,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        });
        // Rebuild the key from {kind, pack, tile_off} so the loader is robust even
        // if the JSON `key` field were ever stale; this matches the dump.
        let key = modern_source_key(cell_json.kind, cell_json.pack, cell_json.tile_off);
        key_to_cell.insert(key, cell_index);
    }

    Ok(ModernSourceAtlas { cells, key_to_cell })
}

// ── JSON manifest types ───────────────────────────────────────────────────────

#[derive(Deserialize)]
struct Manifest {
    format: String,
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
        let mut key_to_cell: HashMap<u64, usize> = HashMap::new();
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
    use std::fs::File;
    use std::io::BufWriter;
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEMP_ROOT_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_temp_root() -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        let pid = std::process::id();
        let counter = TEMP_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("zelda3-source-atlas-test-{pid}-{suffix}-{counter}"))
    }

    fn write_index_png(path: &Path, width: u32, height: u32, pixels: &[u8]) {
        let file = File::create(path).expect("create png");
        let writer = BufWriter::new(file);
        let mut encoder = png::Encoder::new(writer, width, height);
        encoder.set_color(png::ColorType::Indexed);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_palette(vec![0, 0, 0, 255, 255, 255]);
        let mut writer = encoder.write_header().expect("write png header");
        writer.write_image_data(pixels).expect("write png data");
    }

    #[test]
    fn source_key_matches_m2_layout() {
        // Non-Link: key = (kind<<32)|(pack<<16)|tile_off; FULL 16 bits of tile_off
        // are kept (lossless — needed for content-hash kinds' hash payload).
        assert_eq!(modern_source_key(1, 30, 44), (1u64 << 32) | (30 << 16) | 44);
        assert_eq!(modern_source_key(2, 8, 41), (2u64 << 32) | (8 << 16) | 41);
        // Non-Link tile_off high bits are NOT dropped (full 16 bits keyed).
        assert_ne!(
            modern_source_key(1, 5, 0x107),
            modern_source_key(1, 5, 0x07)
        );
        // Link (kind 3): 10-bit pack (bits 14-23) + 14-bit tile_off (bits 0-13),
        // in a lower bit range than non-Link's `kind<<32` keys.
        assert_eq!(
            modern_source_key(3, 5, 0x2103),
            (3u64 << 24) | (5 << 14) | 0x2103
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
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
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
        // A known content-hash BG stream source from the committed manifest resolves.
        assert!(source_cell(&atlas, 6, 53219, 29765).is_some());
    }

    #[test]
    fn rejects_assets_by_source_manifest_count_drift() {
        let root = unique_temp_root();
        let atlas_dir = root.join("zelda3-bin/developer_tilesets");
        std::fs::create_dir_all(&atlas_dir).expect("create atlas dir");
        write_index_png(&atlas_dir.join("assets_by_source.png"), 8, 8, &[0u8; 64]);
        std::fs::write(
            atlas_dir.join("assets_by_source.json"),
            r#"{
              "format": "zelda3_assets_by_source_v2_png",
              "cell_count": 2,
              "cells": [{
                "id": 0,
                "key": 4294967296,
                "kind": 1,
                "pack": 0,
                "tile_off": 0
              }]
            }"#,
        )
        .expect("write manifest");

        let err = match load_modern_source_atlas(&root) {
            Ok(_) => panic!("source atlas count drift should be rejected"),
            Err(err) => err,
        };

        assert!(err.contains("cell_count 2 does not match 1 cells"), "{err}");

        std::fs::remove_dir_all(root).expect("remove temp root");
    }

    #[test]
    fn rejects_assets_by_source_cells_outside_png_grid() {
        let root = unique_temp_root();
        let atlas_dir = root.join("zelda3-bin/developer_tilesets");
        std::fs::create_dir_all(&atlas_dir).expect("create atlas dir");
        write_index_png(&atlas_dir.join("assets_by_source.png"), 8, 8, &[0u8; 64]);
        std::fs::write(
            atlas_dir.join("assets_by_source.json"),
            r#"{
              "format": "zelda3_assets_by_source_v2_png",
              "cell_count": 1,
              "cells": [{
                "id": 1,
                "key": 4294967296,
                "kind": 1,
                "pack": 0,
                "tile_off": 0
              }]
            }"#,
        )
        .expect("write manifest");

        let err = match load_modern_source_atlas(&root) {
            Ok(_) => panic!("source atlas out-of-bounds cell should be rejected"),
            Err(err) => err,
        };

        assert!(
            err.contains("cell id 1 is outside PNG grid capacity 1"),
            "{err}"
        );

        std::fs::remove_dir_all(root).expect("remove temp root");
    }
}
