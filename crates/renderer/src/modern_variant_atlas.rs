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
    pub source_hflip: bool,
    pub source_vflip: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TileEffect {
    pub id: String,
    pub palette: String,
    pub palette_row: u8,
    pub colors_per_row: u8,
    pub index_to_rgba: Vec<[u8; 4]>,
    pub dynamic_policy: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModernVariantAtlas {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub entries: Vec<VariantAtlasEntry>,
    pub effects: Vec<TileEffect>,
}

pub fn variant_key_for_index_tile(
    cell: &crate::modern_index_atlas::ModernIndexTile,
    palette_name: &str,
    palette_row: u8,
) -> Option<VariantAtlasKey> {
    if cell.source_key == crate::modern_hd_overrides::NO_SOURCE_KEY {
        return None;
    }
    let kind = (cell.source_key >> 32) as u8;
    let pack = ((cell.source_key >> 16) & 0xffff) as u16;
    let tile = (cell.source_key & 0xffff) as u16;
    let (source_kind, asset) = match kind {
        1 | 5 | 6 => ("bg", "kBgGfx"),
        2 => ("sprite", "kSprGfx"),
        _ => return None,
    };
    Some(VariantAtlasKey {
        source_kind: source_kind.to_string(),
        asset: asset.to_string(),
        pack,
        tile,
        // The current ROM-derived source packs that feed this atlas are 3bpp.
        // Link/special live sources are deliberately unresolved above.
        bpp: 3,
        palette: palette_name.to_string(),
        palette_row,
    })
}

impl ModernVariantAtlas {
    pub fn entry_for_key(&self, key: &VariantAtlasKey) -> Option<&VariantAtlasEntry> {
        self.entries.iter().find(|entry| entry.key == *key)
    }

    pub fn effect_for_entry(&self, entry: &VariantAtlasEntry) -> Option<&TileEffect> {
        let colors_per_row = 1u8.checked_shl(u32::from(entry.key.bpp))?;
        self.effects.iter().find(|effect| {
            effect.palette == entry.key.palette
                && effect.palette_row == entry.key.palette_row
                && effect.colors_per_row == colors_per_row
        })
    }
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
    let entries = manifest
        .entries
        .into_iter()
        .map(VariantAtlasEntry::from)
        .collect();

    Ok(ModernVariantAtlas {
        width: info.width,
        height: info.height,
        rgba,
        entries,
        effects: load_tile_effects_from_dir(&atlas_dir)?,
    })
}

pub fn load_modern_base_art_atlas(root: &Path) -> Result<ModernVariantAtlas, String> {
    let root_atlas_dir = root.join("atlas");
    let atlas_dir = if root_atlas_dir.join("art_tiles.json").is_file()
        || root_atlas_dir.join("base_tiles.json").is_file()
    {
        root.join("atlas")
    } else {
        root.join("generated/zelda3_assets/atlas")
    };
    if atlas_dir.join("art_tiles.json").is_file() {
        return load_modern_canonical_art_atlas_from_dir(&atlas_dir);
    }
    let json_path = atlas_dir.join("base_tiles.json");
    let png_path = atlas_dir.join("base_tiles.png");

    let manifest_bytes = std::fs::read(&json_path)
        .map_err(|e| format!("failed to read {}: {e}", json_path.display()))?;
    let manifest: BaseManifestJson = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("failed to parse {}: {e}", json_path.display()))?;
    if manifest.format != "zelda3_base_art_atlas_v1" {
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
    let entries = manifest
        .entries
        .into_iter()
        .map(VariantAtlasEntry::from)
        .collect();

    Ok(ModernVariantAtlas {
        width: info.width,
        height: info.height,
        rgba,
        entries,
        effects: load_tile_effects_from_dir(&atlas_dir)?,
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
            source_hflip: false,
            source_vflip: false,
        }
    }
}

impl From<BaseEntryJson> for VariantAtlasEntry {
    fn from(entry: BaseEntryJson) -> Self {
        let key = VariantAtlasKey {
            source_kind: entry.source_kind,
            asset: entry.asset,
            pack: entry.pack,
            tile: entry.tile,
            bpp: entry.bpp,
            palette: entry.preview_palette,
            palette_row: entry.preview_palette_row,
        };
        Self {
            id: entry.id,
            key,
            rect: entry.rect,
            sha1: entry.sha1,
            duplicate_of: entry.duplicate_of,
            dynamic_policy: entry.dynamic_policy.unwrap_or_else(|| "stable".to_string()),
            source_hflip: false,
            source_vflip: false,
        }
    }
}

fn load_modern_canonical_art_atlas_from_dir(
    atlas_dir: &Path,
) -> Result<ModernVariantAtlas, String> {
    let json_path = atlas_dir.join("art_tiles.json");
    let png_path = atlas_dir.join("art_tiles.png");

    let manifest_bytes = std::fs::read(&json_path)
        .map_err(|e| format!("failed to read {}: {e}", json_path.display()))?;
    let manifest: ArtManifestJson = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("failed to parse {}: {e}", json_path.display()))?;
    if manifest.format != "zelda3_canonical_art_atlas_v1" {
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

    let mut entries = Vec::new();
    for art in manifest.arts {
        for source_ref in art.source_refs {
            let dynamic_policy = if source_ref.preview_source == "palette_usage" {
                "stable"
            } else {
                "requires_live_palette"
            };
            let id = format!(
                "{}:{}:pack{}:tile{}:{}bpp",
                source_ref.source_kind,
                source_ref.asset,
                source_ref.pack,
                source_ref.tile,
                source_ref.bpp
            );
            entries.push(VariantAtlasEntry {
                id,
                key: VariantAtlasKey {
                    source_kind: source_ref.source_kind,
                    asset: source_ref.asset,
                    pack: source_ref.pack,
                    tile: source_ref.tile,
                    bpp: source_ref.bpp,
                    palette: source_ref.preview_palette,
                    palette_row: source_ref.preview_palette_row,
                },
                rect: art.rect,
                sha1: art.sha1_indices.clone(),
                duplicate_of: None,
                dynamic_policy: dynamic_policy.to_string(),
                source_hflip: source_ref.hflip,
                source_vflip: source_ref.vflip,
            });
        }
    }

    Ok(ModernVariantAtlas {
        width: info.width,
        height: info.height,
        rgba: buf[..info.buffer_size()].to_vec(),
        entries,
        effects: load_tile_effects_from_dir(atlas_dir)?,
    })
}

fn load_tile_effects_from_dir(atlas_dir: &Path) -> Result<Vec<TileEffect>, String> {
    let json_path = atlas_dir.join("tile_effects.json");
    if !json_path.is_file() {
        return Ok(Vec::new());
    }
    let manifest_bytes = std::fs::read(&json_path)
        .map_err(|e| format!("failed to read {}: {e}", json_path.display()))?;
    let manifest: EffectsManifestJson = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("failed to parse {}: {e}", json_path.display()))?;
    if manifest.format != "zelda3_tile_effects_v1" {
        return Err(format!(
            "{}: unsupported format {:?}",
            json_path.display(),
            manifest.format
        ));
    }
    manifest
        .effects
        .into_iter()
        .map(|effect| {
            if effect.effect_type != "palette_lut" {
                return Err(format!(
                    "{}: unsupported effect type {:?}",
                    json_path.display(),
                    effect.effect_type
                ));
            }
            let mut index_to_rgba = Vec::with_capacity(effect.index_to_rgb.len());
            for rgb in effect.index_to_rgb {
                index_to_rgba.push([rgb[0], rgb[1], rgb[2], 0xff]);
            }
            Ok(TileEffect {
                id: effect.id,
                palette: effect.palette,
                palette_row: effect.palette_row,
                colors_per_row: effect.colors_per_row,
                index_to_rgba,
                dynamic_policy: effect.dynamic_policy,
            })
        })
        .collect()
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

#[derive(Deserialize)]
struct BaseManifestJson {
    format: String,
    width: u32,
    height: u32,
    entries: Vec<BaseEntryJson>,
}

#[derive(Deserialize)]
struct BaseEntryJson {
    id: String,
    source_kind: String,
    asset: String,
    pack: u16,
    tile: u16,
    bpp: u8,
    preview_palette: String,
    preview_palette_row: u8,
    rect: [u32; 4],
    sha1: String,
    duplicate_of: Option<String>,
    dynamic_policy: Option<String>,
}

#[derive(Deserialize)]
struct ArtManifestJson {
    format: String,
    width: u32,
    height: u32,
    arts: Vec<ArtEntryJson>,
}

#[derive(Deserialize)]
struct ArtEntryJson {
    rect: [u32; 4],
    sha1_indices: String,
    source_refs: Vec<ArtSourceRefJson>,
}

#[derive(Deserialize)]
struct ArtSourceRefJson {
    source_kind: String,
    asset: String,
    pack: u16,
    tile: u16,
    bpp: u8,
    hflip: bool,
    vflip: bool,
    preview_palette: String,
    preview_palette_row: u8,
    preview_source: String,
}

#[derive(Deserialize)]
struct EffectsManifestJson {
    format: String,
    effects: Vec<EffectJson>,
}

#[derive(Deserialize)]
struct EffectJson {
    id: String,
    #[serde(rename = "type")]
    effect_type: String,
    palette: String,
    palette_row: u8,
    colors_per_row: u8,
    index_to_rgb: Vec<[u8; 3]>,
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
        let rgba = vec![1, 2, 3, 255, 4, 5, 6, 255, 7, 8, 9, 255, 10, 11, 12, 255];
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
        assert_eq!(
            atlas.entries[0].id,
            "sprite:kSprGfx:pack12:tile37:3bpp:palette_main_spr:row3"
        );
        assert_eq!(atlas.entries[0].key.source_kind, "sprite");
        assert_eq!(atlas.entries[0].key.pack, 12);
        assert_eq!(atlas.entries[0].key.palette_row, 3);
        assert_eq!(atlas.entries[0].dynamic_policy, "stable");

        std::fs::remove_dir_all(root).expect("remove temp root");
    }

    #[test]
    fn modern_base_art_atlas_loads_preview_keys_from_base_tiles_manifest() {
        let root = unique_temp_root();
        let atlas_dir = root.join("atlas");
        std::fs::create_dir_all(&atlas_dir).expect("create atlas dir");
        let rgba = vec![1, 2, 3, 255, 4, 5, 6, 255, 7, 8, 9, 255, 10, 11, 12, 255];
        write_rgba_png(&atlas_dir.join("base_tiles.png"), 2, 2, &rgba);
        std::fs::write(
            atlas_dir.join("base_tiles.json"),
            r#"{
              "format": "zelda3_base_art_atlas_v1",
              "tile_width": 8,
              "tile_height": 8,
              "width": 2,
              "height": 2,
              "entry_count": 1,
              "entries": [{
                "id": "bg:kBgGfx:pack5:tile17:3bpp",
                "source_kind": "bg",
                "asset": "kBgGfx",
                "pack": 5,
                "tile": 17,
                "bpp": 3,
                "preview_palette": "palette_dung_bg_main",
                "preview_palette_row": 2,
                "preview_source": "palette_usage",
                "rect": [0, 0, 8, 8],
                "sha1": "abc",
                "duplicate_of": null
              }]
            }"#,
        )
        .expect("write manifest");
        std::fs::write(
            atlas_dir.join("tile_effects.json"),
            r#"{
              "format": "zelda3_tile_effects_v1",
              "strategy": "base_art_plus_shader_effects",
              "effects": [{
                "id": "palette_dung_bg_main:8color:row2",
                "type": "palette_lut",
                "palette": "palette_dung_bg_main",
                "palette_row": 2,
                "colors_per_row": 8,
                "index_to_rgb": [
                  [0, 0, 0],
                  [10, 20, 30],
                  [40, 50, 60],
                  [70, 80, 90],
                  [100, 110, 120],
                  [130, 140, 150],
                  [160, 170, 180],
                  [190, 200, 210]
                ],
                "dynamic_policy": "stable",
                "runtime": "shader_effect"
              }]
            }"#,
        )
        .expect("write effects");

        let atlas = load_modern_base_art_atlas(&root).expect("load base atlas");

        assert_eq!(atlas.width, 2);
        assert_eq!(atlas.height, 2);
        assert_eq!(atlas.rgba, rgba);
        assert_eq!(atlas.entries.len(), 1);
        assert_eq!(atlas.entries[0].id, "bg:kBgGfx:pack5:tile17:3bpp");
        assert_eq!(atlas.entries[0].key.source_kind, "bg");
        assert_eq!(atlas.entries[0].key.pack, 5);
        assert_eq!(atlas.entries[0].key.tile, 17);
        assert_eq!(atlas.entries[0].key.palette, "palette_dung_bg_main");
        assert_eq!(atlas.entries[0].key.palette_row, 2);
        assert_eq!(atlas.entries[0].dynamic_policy, "stable");
        let effect = atlas
            .effect_for_entry(&atlas.entries[0])
            .expect("resolve palette effect");
        assert_eq!(effect.id, "palette_dung_bg_main:8color:row2");
        assert_eq!(effect.index_to_rgba[2], [40, 50, 60, 0xff]);

        std::fs::remove_dir_all(root).expect("remove temp root");
    }

    #[test]
    fn modern_base_art_atlas_prefers_canonical_art_source_refs() {
        let root = unique_temp_root();
        let atlas_dir = root.join("atlas");
        std::fs::create_dir_all(&atlas_dir).expect("create atlas dir");
        let rgba = vec![1, 2, 3, 255, 4, 5, 6, 255, 7, 8, 9, 255, 10, 11, 12, 255];
        write_rgba_png(&atlas_dir.join("art_tiles.png"), 2, 2, &rgba);
        std::fs::write(
            atlas_dir.join("art_tiles.json"),
            r#"{
              "format": "zelda3_canonical_art_atlas_v1",
              "tile_width": 8,
              "tile_height": 8,
              "width": 2,
              "height": 2,
              "art_count": 1,
              "source_ref_count": 1,
              "arts": [{
                "art_id": "art:abc",
                "bpp": 3,
                "rect": [0, 0, 8, 8],
                "sha1_indices": "abc",
                "preview_palette": "palette_dung_bg_main",
                "preview_palette_row": 2,
                "preview_source": "palette_usage",
                "source_refs": [{
                  "source_kind": "bg",
                  "asset": "kBgGfx",
                  "pack": 5,
                  "tile": 17,
                  "bpp": 3,
                  "hflip": true,
                  "vflip": false,
                  "preview_palette": "palette_dung_bg_main",
                  "preview_palette_row": 2,
                  "preview_source": "palette_usage"
                }]
              }]
            }"#,
        )
        .expect("write manifest");

        let atlas = load_modern_base_art_atlas(&root).expect("load art atlas");

        assert_eq!(atlas.width, 2);
        assert_eq!(atlas.height, 2);
        assert_eq!(atlas.rgba, rgba);
        assert_eq!(atlas.entries.len(), 1);
        assert_eq!(atlas.entries[0].id, "bg:kBgGfx:pack5:tile17:3bpp");
        assert_eq!(atlas.entries[0].key.source_kind, "bg");
        assert_eq!(atlas.entries[0].key.pack, 5);
        assert_eq!(atlas.entries[0].key.tile, 17);
        assert_eq!(atlas.entries[0].key.palette, "palette_dung_bg_main");
        assert_eq!(atlas.entries[0].key.palette_row, 2);
        assert_eq!(atlas.entries[0].rect, [0, 0, 8, 8]);
        assert!(atlas.entries[0].source_hflip);
        assert!(!atlas.entries[0].source_vflip);
        assert_eq!(atlas.entries[0].dynamic_policy, "stable");

        std::fs::remove_dir_all(root).expect("remove temp root");
    }
}
