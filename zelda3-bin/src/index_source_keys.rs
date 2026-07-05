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
                "{}: expected 8-bit indexed PNG, got {:?}/{:?}",
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
        let data = &buf[..info.buffer_size()];
        let mut by_pattern = HashMap::new();
        for cell in manifest.cells {
            if cell.kind != 6 {
                continue;
            }
            let id = cell.id as usize;
            let cx = (id % cols) * 8;
            let cy = (id / cols) * 8;
            if cy + 8 > height {
                continue;
            }
            let mut pattern = [0u8; 64];
            for row in 0..8usize {
                let src = (cy + row) * width + cx;
                pattern[row * 8..row * 8 + 8].copy_from_slice(&data[src..src + 8]);
            }
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
