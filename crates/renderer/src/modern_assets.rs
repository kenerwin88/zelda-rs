use serde::Deserialize;
use std::fs;
use std::io::BufReader;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct ModernTileAtlasAsset {
    pub tile_width_px: u16,
    pub tile_height_px: u16,
    pub width_px: u32,
    pub height_px: u32,
    pub rgba: Vec<u8>,
    pub entries: Vec<ModernTileAtlasEntry>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ModernTileAtlasEntry {
    pub id: u32,
    pub atlas_x_px: u16,
    pub atlas_y_px: u16,
    pub atlas_width_px: u16,
    pub atlas_height_px: u16,
    pub tilemap_entry: u16,
    pub tilemap_variants: Vec<u16>,
}

pub fn atlas_entry_for_tilemap_entry<'a>(
    asset: &'a ModernTileAtlasAsset,
    tilemap_entry: u16,
) -> Option<&'a ModernTileAtlasEntry> {
    asset
        .entries
        .iter()
        .find(|entry| entry.tilemap_entry == tilemap_entry || entry.tilemap_variants.contains(&tilemap_entry))
}

#[derive(Deserialize)]
struct Manifest {
    tile_width_px: u16,
    tile_height_px: u16,
    unique_tiles: Vec<ModernTileAtlasEntry>,
}

pub fn load_modern_overworld_tile_atlas(repo_root: &Path) -> Result<ModernTileAtlasAsset, String> {
    let base = repo_root.join("zelda3-bin/developer_tilesets");
    let manifest_path = base.join("overworld_unique_tiles.json");
    let png_path = base.join("overworld_unique_tiles.png");

    let manifest_bytes = fs::read(&manifest_path)
        .map_err(|e| format!("failed to read {}: {e}", manifest_path.display()))?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("failed to parse {}: {e}", manifest_path.display()))?;

    let file = std::fs::File::open(&png_path)
        .map_err(|e| format!("failed to open {}: {e}", png_path.display()))?;
    let decoder = png::Decoder::new(BufReader::new(file));
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("failed to decode PNG {}: {e}", png_path.display()))?;
    let mut buffer = vec![0u8; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|e| format!("failed to read PNG frame {}: {e}", png_path.display()))?;
    let width_px = info.width;
    let height_px = info.height;
    let bytes = &buffer[..info.buffer_size()];
    let rgba = match (info.color_type, info.bit_depth) {
        (png::ColorType::Rgba, png::BitDepth::Eight) => bytes.to_vec(),
        (png::ColorType::Rgb, png::BitDepth::Eight) => {
            let mut out = Vec::with_capacity((width_px * height_px * 4) as usize);
            for rgb in bytes.chunks_exact(3) {
                out.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 0xff]);
            }
            out
        }
        (png::ColorType::Grayscale, png::BitDepth::Eight) => {
            let mut out = Vec::with_capacity((width_px * height_px * 4) as usize);
            for &luma in bytes {
                out.extend_from_slice(&[luma, luma, luma, 0xff]);
            }
            out
        }
        (png::ColorType::GrayscaleAlpha, png::BitDepth::Eight) => {
            let mut out = Vec::with_capacity((width_px * height_px * 4) as usize);
            for ga in bytes.chunks_exact(2) {
                out.extend_from_slice(&[ga[0], ga[0], ga[0], ga[1]]);
            }
            out
        }
        (color, depth) => {
            return Err(format!(
                "unsupported PNG format {color:?}/{depth:?} in {}",
                png_path.display()
            ));
        }
    };
    let expected = (width_px * height_px * 4) as usize;
    if rgba.len() != expected {
        return Err(format!(
            "decoded RGBA length {} did not match expected {expected}",
            rgba.len()
        ));
    }

    Ok(ModernTileAtlasAsset {
        tile_width_px: manifest.tile_width_px,
        tile_height_px: manifest.tile_height_px,
        width_px,
        height_px,
        rgba,
        entries: manifest.unique_tiles,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn loads_repo_overworld_tile_atlas_manifest_and_png() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");

        let atlas = load_modern_overworld_tile_atlas(&root).expect("atlas should load");

        assert_eq!(atlas.tile_width_px, 8);
        assert_eq!(atlas.tile_height_px, 8);
        assert_eq!(atlas.entries.len(), 6140);
        assert_eq!(atlas.width_px, 2113);
        assert_eq!(atlas.height_px, 3169);
        assert!(!atlas.rgba.is_empty());
        assert_eq!(atlas.entries[0].atlas_x_px, 1);
    }

    #[test]
    fn atlas_lookup_finds_tilemap_variant() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let atlas = load_modern_overworld_tile_atlas(&root).expect("atlas should load");

        let entry = atlas_entry_for_tilemap_entry(&atlas, 2218).expect("tilemap entry should exist");

        assert_eq!(entry.id, 0);
        assert_eq!(entry.atlas_x_px, 1);
    }
}
