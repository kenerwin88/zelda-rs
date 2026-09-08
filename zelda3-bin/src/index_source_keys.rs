use crate::indexed_tile_sheet::IndexedTileSheet;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct IndexSourceKey {
    pub kind: u8,
    pub pack: u16,
    pub tile_off: u16,
}

#[derive(Default)]
pub(crate) struct IndexSourceKeyMap {
    by_pattern: HashMap<[u8; 64], IndexSourceKey>,
}

impl IndexSourceKeyMap {
    pub(crate) fn load_from_developer_tilesets(base: &Path) -> Result<Self, String> {
        let json_path = base.join("assets_by_source.json");
        let png_path = base.join("assets_by_source.png");
        let manifest_bytes = fs::read(&json_path)
            .map_err(|e| format!("failed to read {}: {e}", json_path.display()))?;
        let manifest: AssetsBySourceManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|e| format!("failed to parse {}: {e}", json_path.display()))?;

        let file = fs::File::open(&png_path)
            .map_err(|e| format!("failed to open {}: {e}", png_path.display()))?;
        let sheet = IndexedTileSheet::decode(std::io::BufReader::new(file), &png_path)?;
        let mut by_pattern = HashMap::new();
        for cell in manifest.cells {
            if cell.kind != 6 {
                continue;
            }
            let Some(pattern) = sheet.cell(cell.id as usize) else {
                continue;
            };
            by_pattern.insert(
                pattern,
                IndexSourceKey {
                    kind: cell.kind,
                    pack: cell.pack,
                    tile_off: cell.tile_off,
                },
            );
        }

        Ok(Self { by_pattern })
    }

    pub(crate) fn get(&self, indices: &[u8; 64]) -> Option<IndexSourceKey> {
        self.by_pattern.get(indices).copied()
    }
}

#[derive(Deserialize)]
struct AssetsBySourceManifest {
    cells: Vec<AssetsBySourceCell>,
}

#[derive(Deserialize)]
struct AssetsBySourceCell {
    id: u32,
    kind: u8,
    pack: u16,
    tile_off: u16,
}
