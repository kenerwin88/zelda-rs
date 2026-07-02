use serde::Deserialize;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VariantAtlasKey {
    pub source_kind: String,
    pub asset: String,
    pub pack: u16,
    pub tile: u16,
    pub bpp: u8,
    pub palette: String,
    pub palette_row: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VariantAtlasEntry {
    pub id: String,
    pub key: VariantAtlasKey,
    pub rect: [u32; 4],
    pub sha1: String,
    pub duplicate_of: Option<String>,
    pub dynamic_policy: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModernVariantAtlas {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub entries: Vec<VariantAtlasEntry>,
}

pub fn load_modern_variant_atlas(root: &Path) -> Result<ModernVariantAtlas, String> {
    let atlas_dir = if root.join("atlas/tile_variants.json").is_file() {
        root.join("atlas")
    } else {
        root.join("generated/zelda3_assets/atlas")
    };
    let json_path = atlas_dir.join("tile_variants.json");
    let png_path = atlas_dir.join("tile_variants.png");

    let manifest_bytes = std::fs::read(&json_path)
        .map_err(|e| format!("failed to read {}: {e}", json_path.display()))?;
    let manifest: ManifestJson = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("failed to parse {}: {e}", json_path.display()))?;
    if manifest.format != "zelda3_rgba_variant_atlas_v1" {
        return Err(format!(
            "{}: unsupported format {:?}",
            json_path.display(),
            manifest.format
        ));
    }

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
    if info.bit_depth != png::BitDepth::Eight || info.color_type != png::ColorType::Rgba {
        return Err(format!(
            "{}: expected an 8-bit RGBA PNG, got {:?}/{:?}",
            png_path.display(),
            info.color_type,
            info.bit_depth
        ));
    }
    if info.width != manifest.width || info.height != manifest.height {
        return Err(format!(
            "{}: PNG size {}x{} does not match manifest {}x{}",
            png_path.display(),
            info.width,
            info.height,
            manifest.width,
            manifest.height
        ));
    }
    let rgba = buf[..info.buffer_size()].to_vec();
    let entries = manifest.entries.into_iter().map(VariantAtlasEntry::from).collect();

    Ok(ModernVariantAtlas {
        width: info.width,
        height: info.height,
        rgba,
        entries,
    })
}

impl From<EntryJson> for VariantAtlasEntry {
    fn from(entry: EntryJson) -> Self {
        let key = VariantAtlasKey {
            source_kind: entry.source_kind,
            asset: entry.asset,
            pack: entry.pack,
            tile: entry.tile,
            bpp: entry.bpp,
            palette: entry.palette,
            palette_row: entry.palette_row,
        };
        Self {
            id: entry.id,
            key,
            rect: entry.rect,
            sha1: entry.sha1,
            duplicate_of: entry.duplicate_of,
            dynamic_policy: entry.dynamic_policy,
        }
    }
}

#[derive(Deserialize)]
struct ManifestJson {
    format: String,
    width: u32,
    height: u32,
    entries: Vec<EntryJson>,
}

#[derive(Deserialize)]
struct EntryJson {
    id: String,
    source_kind: String,
    asset: String,
    pack: u16,
    tile: u16,
    bpp: u8,
    palette: String,
    palette_row: u8,
    rect: [u32; 4],
    sha1: String,
    duplicate_of: Option<String>,
    dynamic_policy: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufWriter;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_root() -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("zelda3-variant-atlas-test-{suffix}"))
    }

    fn write_rgba_png(path: &Path, width: u32, height: u32, rgba: &[u8]) {
        let file = File::create(path).expect("create png");
        let writer = BufWriter::new(file);
        let mut encoder = png::Encoder::new(writer, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("write png header");
        writer.write_image_data(rgba).expect("write png data");
    }

    #[test]
    fn modern_variant_atlas_loads_rgba_png_and_manifest_entries() {
        let root = unique_temp_root();
        let atlas_dir = root.join("atlas");
        std::fs::create_dir_all(&atlas_dir).expect("create atlas dir");
        let rgba = vec![
            1, 2, 3, 255, 4, 5, 6, 255, 7, 8, 9, 255, 10, 11, 12, 255,
        ];
        write_rgba_png(&atlas_dir.join("tile_variants.png"), 2, 2, &rgba);
        std::fs::write(
            atlas_dir.join("tile_variants.json"),
            r#"{
              "format": "zelda3_rgba_variant_atlas_v1",
              "tile_width": 8,
              "tile_height": 8,
              "width": 2,
              "height": 2,
              "entry_count": 1,
              "entries": [{
                "id": "sprite:kSprGfx:pack12:tile37:3bpp:palette_main_spr:row3",
                "source_kind": "sprite",
                "asset": "kSprGfx",
                "pack": 12,
                "tile": 37,
                "bpp": 3,
                "palette": "palette_main_spr",
                "palette_row": 3,
                "rect": [0, 0, 8, 8],
                "sha1": "abc",
                "duplicate_of": null,
                "dynamic_policy": "stable"
              }]
            }"#,
        )
        .expect("write manifest");

        let atlas = load_modern_variant_atlas(&root).expect("load variant atlas");

        assert_eq!(atlas.width, 2);
        assert_eq!(atlas.height, 2);
        assert_eq!(atlas.rgba, rgba);
        assert_eq!(atlas.entries.len(), 1);
        assert_eq!(atlas.entries[0].id, "sprite:kSprGfx:pack12:tile37:3bpp:palette_main_spr:row3");
        assert_eq!(atlas.entries[0].key.source_kind, "sprite");
        assert_eq!(atlas.entries[0].key.pack, 12);
        assert_eq!(atlas.entries[0].key.palette_row, 3);
        assert_eq!(atlas.entries[0].dynamic_policy, "stable");

        std::fs::remove_dir_all(root).expect("remove temp root");
    }
}
