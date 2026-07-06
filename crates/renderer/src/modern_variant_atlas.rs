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
    pub runtime_material: Option<String>,
    pub runtime_colors_per_row: Option<u8>,
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

#[derive(Clone, Copy, Debug)]
pub enum VariantAtlasDraw<'a> {
    Stable {
        entry: &'a VariantAtlasEntry,
    },
    MaterialEffect {
        entry: &'a VariantAtlasEntry,
        effect: &'a TileEffect,
    },
    DynamicPalette {
        entry: &'a VariantAtlasEntry,
        reason: DynamicFallbackReason,
    },
    MissingArt,
    Unkeyed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DynamicFallbackReason {
    InstanceSourceKey,
    Brightness,
    EntryRequiresLivePalette,
    UnsupportedMaterial,
    MissingStableEffect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DynamicPolicy {
    Stable,
    RequiresLivePalette,
}

impl DynamicPolicy {
    fn from_manifest(value: &str) -> Self {
        match value {
            "stable" => Self::Stable,
            _ => Self::RequiresLivePalette,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuntimeMaterial {
    LegacyPreview,
    PaletteLut,
    Unsupported,
}

impl RuntimeMaterial {
    fn from_manifest(value: Option<&str>) -> Self {
        match value {
            None => Self::LegacyPreview,
            Some("palette_lut") => Self::PaletteLut,
            Some(_) => Self::Unsupported,
        }
    }
}

impl<'a> VariantAtlasDraw<'a> {
    pub fn entry(self) -> Option<&'a VariantAtlasEntry> {
        match self {
            Self::Stable { entry }
            | Self::MaterialEffect { entry, .. }
            | Self::DynamicPalette { entry, .. } => Some(entry),
            Self::MissingArt | Self::Unkeyed => None,
        }
    }

    pub fn material_effect(self) -> Option<(&'a VariantAtlasEntry, &'a TileEffect)> {
        match self {
            Self::MaterialEffect { entry, effect } => Some((entry, effect)),
            Self::Stable { .. }
            | Self::DynamicPalette { .. }
            | Self::MissingArt
            | Self::Unkeyed => None,
        }
    }

    pub fn is_unsupported_material_fallback(self) -> bool {
        matches!(
            self,
            Self::DynamicPalette {
                reason: DynamicFallbackReason::UnsupportedMaterial,
                ..
            }
        )
    }
}

pub fn variant_key_for_index_tile(
    cell: &crate::modern_index_atlas::ModernIndexTile,
    palette_name: &str,
    palette_row: u8,
) -> Option<VariantAtlasKey> {
    variant_key_for_source_key(cell.source_key, palette_name, palette_row)
}

pub fn variant_key_for_source_key(
    source_key: u64,
    palette_name: &str,
    palette_row: u8,
) -> Option<VariantAtlasKey> {
    if source_key == crate::modern_hd_overrides::NO_SOURCE_KEY {
        return None;
    }
    let kind = (source_key >> 32) as u8;
    let pack = ((source_key >> 16) & 0xffff) as u16;
    let tile = (source_key & 0xffff) as u16;
    let (source_kind, asset, bpp) = match kind {
        1 | 5 | 6 => ("bg", "kBgGfx", 3),
        2 => ("sprite", "kSprGfx", 3),
        4 | 7 => ("bg3", "kBg3Gfx", 5),
        8 => ("link", "kLinkGfx", 3),
        _ => return None,
    };
    Some(VariantAtlasKey {
        source_kind: source_kind.to_string(),
        asset: asset.to_string(),
        pack,
        tile,
        bpp,
        palette: palette_name.to_string(),
        palette_row,
    })
}

impl ModernVariantAtlas {
    pub fn entry_for_key(&self, key: &VariantAtlasKey) -> Option<&VariantAtlasEntry> {
        self.entries.iter().find(|entry| entry.key == *key)
    }

    pub fn entry_for_source_key(&self, key: &VariantAtlasKey) -> Option<&VariantAtlasEntry> {
        self.entries.iter().find(|entry| {
            entry.key.source_kind == key.source_kind
                && entry.key.asset == key.asset
                && entry.key.pack == key.pack
                && entry.key.tile == key.tile
                && entry.key.bpp == key.bpp
        })
    }

    pub fn has_mode7_source_art(&self) -> bool {
        self.entries.iter().any(|entry| {
            entry.key.source_kind == "mode7"
                && entry.key.asset == "kOverworldMapGfx"
                && entry.key.pack == 0
                && entry.key.bpp == 8
                && entry.dynamic_policy == "stable"
                && entry.runtime_material.as_deref() == Some("palette_lut")
                && entry.runtime_colors_per_row == Some(128)
        })
    }

    pub fn effect_for_entry(&self, entry: &VariantAtlasEntry) -> Option<&TileEffect> {
        self.effect_for_entry_and_key(entry, &entry.key)
    }

    pub fn effect_for_key(&self, key: &VariantAtlasKey) -> Option<&TileEffect> {
        let colors_per_row = runtime_effect_color_rows(key);
        self.effect_for_key_with_color_rows(key, colors_per_row)
    }

    fn effect_for_entry_and_key(
        &self,
        entry: &VariantAtlasEntry,
        key: &VariantAtlasKey,
    ) -> Option<&TileEffect> {
        let colors_per_row = entry_runtime_effect_color_rows(entry, key);
        self.effect_for_key_with_color_rows(key, colors_per_row)
    }

    fn effect_for_key_with_color_rows(
        &self,
        key: &VariantAtlasKey,
        colors_per_row: Option<Vec<u8>>,
    ) -> Option<&TileEffect> {
        let colors_per_row = colors_per_row?;
        colors_per_row.into_iter().find_map(|colors_per_row| {
            self.effects.iter().find(|effect| {
                effect.palette == key.palette
                    && effect.palette_row == key.palette_row
                    && effect.colors_per_row == colors_per_row
            })
        })
    }

    pub fn effect_row_for_effect(&self, effect: &TileEffect) -> Option<u32> {
        self.effects
            .iter()
            .position(|candidate| candidate == effect)
            .map(|row| row as u32)
    }

    pub fn resolve_draw<'a>(&'a self, key: Option<&VariantAtlasKey>) -> VariantAtlasDraw<'a> {
        let Some(key) = key else {
            return VariantAtlasDraw::Unkeyed;
        };
        let Some(entry) = self.entry_for_source_key(key) else {
            return VariantAtlasDraw::MissingArt;
        };
        if entry_dynamic_policy(entry) != DynamicPolicy::Stable {
            return VariantAtlasDraw::DynamicPalette {
                entry,
                reason: DynamicFallbackReason::EntryRequiresLivePalette,
            };
        }
        match entry_runtime_material(entry) {
            RuntimeMaterial::Unsupported => {
                return VariantAtlasDraw::DynamicPalette {
                    entry,
                    reason: DynamicFallbackReason::UnsupportedMaterial,
                };
            }
            RuntimeMaterial::PaletteLut => {
                let stable_effect = self
                    .effect_for_entry_and_key(entry, key)
                    .filter(|effect| effect_dynamic_policy(effect) == DynamicPolicy::Stable);
                if let Some(effect) = stable_effect {
                    return VariantAtlasDraw::MaterialEffect { entry, effect };
                }
                if entry_matches_material(entry, key) {
                    return VariantAtlasDraw::Stable { entry };
                }
            }
            RuntimeMaterial::LegacyPreview if !entry_matches_material(entry, key) => {
                let stable_effect = self
                    .effect_for_entry_and_key(entry, key)
                    .filter(|effect| effect_dynamic_policy(effect) == DynamicPolicy::Stable);
                if let Some(effect) = stable_effect {
                    return VariantAtlasDraw::MaterialEffect { entry, effect };
                }
            }
            RuntimeMaterial::LegacyPreview => {
                return VariantAtlasDraw::Stable { entry };
            }
        }
        VariantAtlasDraw::DynamicPalette {
            entry,
            reason: DynamicFallbackReason::MissingStableEffect,
        }
    }

    pub fn resolve_dynamic_draw<'a>(
        &'a self,
        key: Option<&VariantAtlasKey>,
        reason: DynamicFallbackReason,
    ) -> VariantAtlasDraw<'a> {
        let Some(key) = key else {
            return VariantAtlasDraw::Unkeyed;
        };
        let Some(entry) = self.entry_for_source_key(key) else {
            return VariantAtlasDraw::MissingArt;
        };
        VariantAtlasDraw::DynamicPalette { entry, reason }
    }
}

fn entry_dynamic_policy(entry: &VariantAtlasEntry) -> DynamicPolicy {
    DynamicPolicy::from_manifest(&entry.dynamic_policy)
}

fn effect_dynamic_policy(effect: &TileEffect) -> DynamicPolicy {
    DynamicPolicy::from_manifest(&effect.dynamic_policy)
}

fn entry_runtime_material(entry: &VariantAtlasEntry) -> RuntimeMaterial {
    RuntimeMaterial::from_manifest(entry.runtime_material.as_deref())
}

fn entry_matches_material(entry: &VariantAtlasEntry, key: &VariantAtlasKey) -> bool {
    entry.key.palette == key.palette && entry.key.palette_row == key.palette_row
}

fn runtime_effect_color_rows(key: &VariantAtlasKey) -> Option<Vec<u8>> {
    let source_stride = 1u8.checked_shl(u32::from(key.bpp))?;
    let mut rows = Vec::new();
    if matches!(key.source_kind.as_str(), "bg" | "sprite") {
        rows.push(16);
    }
    if !rows.contains(&source_stride) {
        rows.push(source_stride);
    }
    Some(rows)
}

fn entry_runtime_effect_color_rows(
    entry: &VariantAtlasEntry,
    key: &VariantAtlasKey,
) -> Option<Vec<u8>> {
    if let Some(colors_per_row) = entry.runtime_colors_per_row {
        return Some(vec![colors_per_row]);
    }
    runtime_effect_color_rows(key)
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

pub fn load_modern_canonical_art_atlas(root: &Path) -> Result<ModernVariantAtlas, String> {
    let root_atlas_dir = root.join("atlas");
    let atlas_dir = if root_atlas_dir.join("art_tiles.json").is_file() {
        root.join("atlas")
    } else {
        root.join("generated/zelda3_assets/atlas")
    };
    if !atlas_dir.join("art_tiles.json").is_file() {
        return Err(format!(
            "canonical art atlas missing: expected {} and {}; regenerate assets with scripts/extract_assets.py",
            atlas_dir.join("art_tiles.json").display(),
            atlas_dir.join("art_tiles.png").display()
        ));
    }
    load_modern_canonical_art_atlas_from_dir(&atlas_dir)
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
            runtime_material: entry.runtime_material,
            runtime_colors_per_row: entry.runtime_colors_per_row,
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
    if manifest.art_count != manifest.arts.len() as u32 {
        return Err(format!(
            "{}: art_count {} does not match {} arts",
            json_path.display(),
            manifest.art_count,
            manifest.arts.len()
        ));
    }
    let source_ref_count: u32 = manifest
        .arts
        .iter()
        .map(|art| art.source_refs.len() as u32)
        .sum();
    if manifest.source_ref_count != source_ref_count {
        return Err(format!(
            "{}: source_ref_count {} does not match {} source refs",
            json_path.display(),
            manifest.source_ref_count,
            source_ref_count
        ));
    }
    for art in &manifest.arts {
        if !rect_within_atlas(art.rect, info.width, info.height) {
            return Err(format!(
                "{}: art rect {:?} for {} is outside PNG bounds {}x{}",
                json_path.display(),
                art.rect,
                art.art_id,
                info.width,
                info.height
            ));
        }
    }

    let effects = load_tile_effects_from_dir(atlas_dir)?;
    let mut entries = Vec::new();
    for art in manifest.arts {
        for source_ref in art.source_refs {
            let dynamic_policy = dynamic_policy_for_source_ref(&effects, &source_ref);
            let runtime_material = runtime_material_for_source_ref(&effects, &source_ref);
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
                dynamic_policy,
                runtime_material,
                runtime_colors_per_row: source_ref.runtime_colors_per_row,
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
        effects,
    })
}

fn dynamic_policy_for_source_ref(effects: &[TileEffect], source_ref: &ArtSourceRefJson) -> String {
    if source_ref.runtime_material.as_deref() == Some("palette_lut") {
        match source_ref.runtime_material_policy.as_deref() {
            Some("stable") => return "stable".to_string(),
            Some("requires_live_palette") => return "requires_live_palette".to_string(),
            _ => {}
        }
    }
    if source_ref.preview_source == "palette_usage"
        || has_stable_effect_for_source_ref(effects, source_ref)
    {
        "stable".to_string()
    } else {
        "requires_live_palette".to_string()
    }
}

fn runtime_material_for_source_ref(
    effects: &[TileEffect],
    source_ref: &ArtSourceRefJson,
) -> Option<String> {
    if let Some(runtime_material) = &source_ref.runtime_material {
        return Some(runtime_material.clone());
    }
    if has_stable_effect_for_source_ref(effects, source_ref) {
        return Some("palette_lut".to_string());
    }
    None
}

fn has_stable_effect_for_source_ref(effects: &[TileEffect], source_ref: &ArtSourceRefJson) -> bool {
    let key = VariantAtlasKey {
        source_kind: source_ref.source_kind.clone(),
        asset: source_ref.asset.clone(),
        pack: source_ref.pack,
        tile: source_ref.tile,
        bpp: source_ref.bpp,
        palette: source_ref.preview_palette.clone(),
        palette_row: source_ref.preview_palette_row,
    };
    let colors_per_rows = source_ref_effect_color_rows(source_ref, &key);
    colors_per_rows.into_iter().any(|colors_per_row| {
        effects.iter().any(|effect| {
            effect.dynamic_policy == "stable"
                && effect.palette == source_ref.preview_palette
                && effect.palette_row == source_ref.preview_palette_row
                && effect.colors_per_row == colors_per_row
        })
    })
}

fn source_ref_effect_color_rows(source_ref: &ArtSourceRefJson, key: &VariantAtlasKey) -> Vec<u8> {
    if let Some(colors_per_row) = source_ref.runtime_colors_per_row {
        return vec![colors_per_row];
    }
    runtime_effect_color_rows(key).unwrap_or_default()
}

fn rect_within_atlas(rect: [u32; 4], width: u32, height: u32) -> bool {
    let [x, y, w, h] = rect;
    if w == 0 || h == 0 {
        return false;
    }
    let Some(right) = x.checked_add(w) else {
        return false;
    };
    let Some(bottom) = y.checked_add(h) else {
        return false;
    };
    right <= width && bottom <= height
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
    if !matches!(
        manifest.format.as_str(),
        "zelda3_tile_effects_v1" | "zelda3_tile_effect_table_v1"
    ) {
        return Err(format!(
            "{}: unsupported format {:?}",
            json_path.display(),
            manifest.format
        ));
    }
    let mut effects = Vec::with_capacity(manifest.effects.len());
    for effect in manifest.effects {
        if effect.effect_type != "palette_lut" {
            return Err(format!(
                "{}: unsupported effect type {:?}",
                json_path.display(),
                effect.effect_type
            ));
        }
        let Ok(palette_row) = u8::try_from(effect.palette_row) else {
            continue;
        };
        let mut index_to_rgba = Vec::with_capacity(effect.index_to_rgb.len());
        for rgb in effect.index_to_rgb {
            index_to_rgba.push([
                rgb[0].min(u8::MAX.into()) as u8,
                rgb[1].min(u8::MAX.into()) as u8,
                rgb[2].min(u8::MAX.into()) as u8,
                0xff,
            ]);
        }
        effects.push(TileEffect {
            id: effect.id,
            palette: effect.palette,
            palette_row,
            colors_per_row: effect.colors_per_row,
            index_to_rgba,
            dynamic_policy: effect.dynamic_policy,
        });
    }
    Ok(effects)
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
    #[serde(default)]
    runtime_material: Option<String>,
    #[serde(default)]
    runtime_colors_per_row: Option<u8>,
}

#[derive(Deserialize)]
struct ArtManifestJson {
    format: String,
    width: u32,
    height: u32,
    art_count: u32,
    source_ref_count: u32,
    arts: Vec<ArtEntryJson>,
}

#[derive(Deserialize)]
struct ArtEntryJson {
    art_id: String,
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
    #[serde(default)]
    runtime_material: Option<String>,
    #[serde(default)]
    runtime_material_policy: Option<String>,
    #[serde(default)]
    runtime_colors_per_row: Option<u8>,
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
    palette_row: u16,
    colors_per_row: u8,
    index_to_rgb: Vec<[u16; 3]>,
    dynamic_policy: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufWriter;
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
        std::env::temp_dir().join(format!(
            "zelda3-variant-atlas-test-{pid}-{suffix}-{counter}"
        ))
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

    fn solid_rgba(width: u32, height: u32, rgba: [u8; 4]) -> Vec<u8> {
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        for _ in 0..(width * height) {
            pixels.extend_from_slice(&rgba);
        }
        pixels
    }

    fn bg_test_key_with_palette_row(palette_row: u8) -> VariantAtlasKey {
        VariantAtlasKey {
            source_kind: "bg".to_string(),
            asset: "kBgGfx".to_string(),
            pack: 0,
            tile: 0,
            bpp: 3,
            palette: "palette_dung_bg_main".to_string(),
            palette_row,
        }
    }

    fn bg_test_entry_with_palette_row(palette_row: u8) -> VariantAtlasEntry {
        VariantAtlasEntry {
            id: "bg:kBgGfx:pack0:tile0:3bpp".to_string(),
            key: bg_test_key_with_palette_row(palette_row),
            rect: [0, 0, 8, 8],
            sha1: "test".to_string(),
            duplicate_of: None,
            dynamic_policy: "stable".to_string(),
            runtime_material: Some("palette_lut".to_string()),
            runtime_colors_per_row: None,
            source_hflip: false,
            source_vflip: false,
        }
    }

    #[test]
    fn bg3_source_key_maps_to_32_color_variant_key() {
        let key = variant_key_for_source_key(
            crate::modern_source_atlas::modern_source_key(4, 0x0407, 0),
            "palette_overworld_bg_main",
            0,
        )
        .expect("BG3 source key should resolve");

        assert_eq!(key.source_kind, "bg3");
        assert_eq!(key.asset, "kBg3Gfx");
        assert_eq!(key.pack, 0x0407);
        assert_eq!(key.tile, 0);
        assert_eq!(key.bpp, 5);
        assert_eq!(key.palette, "palette_overworld_bg_main");
        assert_eq!(key.palette_row, 0);

        let content_key = variant_key_for_source_key(
            crate::modern_source_atlas::modern_source_key(7, 0x1234, 0x5678),
            "palette_overworld_bg_main",
            0,
        )
        .expect("BG3 content source key should resolve");
        assert_eq!(content_key.source_kind, "bg3");
        assert_eq!(content_key.asset, "kBg3Gfx");
        assert_eq!(content_key.pack, 0x1234);
        assert_eq!(content_key.tile, 0x5678);
        assert_eq!(content_key.bpp, 5);
    }

    #[test]
    fn link_content_source_key_maps_to_link_variant_key() {
        let key = variant_key_for_source_key(
            crate::modern_source_atlas::modern_source_key(8, 0x1234, 0x5678),
            "palette_main_spr",
            4,
        )
        .expect("Link content source key should resolve");

        assert_eq!(key.source_kind, "link");
        assert_eq!(key.asset, "kLinkGfx");
        assert_eq!(key.pack, 0x1234);
        assert_eq!(key.tile, 0x5678);
        assert_eq!(key.bpp, 3);
        assert_eq!(key.palette, "palette_main_spr");
        assert_eq!(key.palette_row, 4);
    }

    fn bg_test_effect_with_palette_row(palette_row: u8) -> TileEffect {
        TileEffect {
            id: format!("palette_dung_bg_main:8color:row{palette_row}"),
            palette: "palette_dung_bg_main".to_string(),
            palette_row,
            colors_per_row: 8,
            index_to_rgba: vec![[0, 0, 0, 255]; 8],
            dynamic_policy: "stable".to_string(),
        }
    }

    fn bg_runtime_effect_with_palette_row(palette_row: u8) -> TileEffect {
        TileEffect {
            id: format!("palette_dung_bg_main:16color:row{palette_row}"),
            palette: "palette_dung_bg_main".to_string(),
            palette_row,
            colors_per_row: 16,
            index_to_rgba: vec![[0, 0, 0, 255]; 16],
            dynamic_policy: "stable".to_string(),
        }
    }

    fn bg_test_atlas(entry_row: u8, effects: Vec<TileEffect>) -> ModernVariantAtlas {
        ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: solid_rgba(8, 8, [1, 2, 3, 255]),
            entries: vec![bg_test_entry_with_palette_row(entry_row)],
            effects,
        }
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
    fn modern_canonical_art_atlas_requires_art_tiles() {
        let root = unique_temp_root();
        let atlas_dir = root.join("atlas");
        std::fs::create_dir_all(&atlas_dir).expect("create atlas dir");
        let rgba = solid_rgba(8, 8, [1, 2, 3, 255]);
        write_rgba_png(&atlas_dir.join("base_tiles.png"), 8, 8, &rgba);
        std::fs::write(
            atlas_dir.join("base_tiles.json"),
            r#"{
              "format": "zelda3_base_art_atlas_v1",
              "tile_width": 8,
              "tile_height": 8,
              "width": 8,
              "height": 8,
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

        let err = load_modern_canonical_art_atlas(&root).expect_err("reject base fallback");

        assert!(
            err.contains("canonical art atlas missing: expected"),
            "{err}"
        );

        std::fs::remove_dir_all(root).expect("remove temp root");
    }

    #[test]
    fn modern_canonical_art_atlas_loads_source_refs() {
        let root = unique_temp_root();
        let atlas_dir = root.join("atlas");
        std::fs::create_dir_all(&atlas_dir).expect("create atlas dir");
        let rgba = solid_rgba(8, 8, [1, 2, 3, 255]);
        write_rgba_png(&atlas_dir.join("art_tiles.png"), 8, 8, &rgba);
        std::fs::write(
            atlas_dir.join("art_tiles.json"),
            r#"{
              "format": "zelda3_canonical_art_atlas_v1",
              "tile_width": 8,
              "tile_height": 8,
              "width": 8,
              "height": 8,
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

        let atlas = load_modern_canonical_art_atlas(&root).expect("load art atlas");

        assert_eq!(atlas.width, 8);
        assert_eq!(atlas.height, 8);
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

    #[test]
    fn modern_canonical_art_atlas_loads_mode7_source_refs() {
        let root = unique_temp_root();
        let atlas_dir = root.join("atlas");
        std::fs::create_dir_all(&atlas_dir).expect("create atlas dir");
        let rgba = solid_rgba(8, 8, [110, 110, 110, 255]);
        write_rgba_png(&atlas_dir.join("art_tiles.png"), 8, 8, &rgba);
        std::fs::write(
            atlas_dir.join("art_tiles.json"),
            r#"{
              "format": "zelda3_canonical_art_atlas_v1",
              "tile_width": 8,
              "tile_height": 8,
              "width": 8,
              "height": 8,
              "art_count": 1,
              "source_ref_count": 1,
              "arts": [{
                "art_id": "art:mode7",
                "bpp": 8,
                "rect": [0, 0, 8, 8],
                "sha1_indices": "mode7",
                "preview_palette": "palette_overworld_bg_main",
                "preview_palette_row": 0,
                "preview_source": "source_kind_default",
                "source_refs": [{
                  "source_kind": "mode7",
                  "asset": "kOverworldMapGfx",
                  "pack": 0,
                  "tile": 42,
                  "bpp": 8,
                  "hflip": false,
                  "vflip": false,
                  "preview_palette": "palette_overworld_bg_main",
                  "preview_palette_row": 0,
                  "preview_source": "source_kind_default",
                  "runtime_material": "palette_lut",
                  "runtime_material_policy": "stable",
                  "runtime_colors_per_row": 128
                }]
              }]
            }"#,
        )
        .expect("write manifest");

        let atlas = load_modern_canonical_art_atlas(&root).expect("load art atlas");

        assert_eq!(atlas.entries.len(), 1);
        let entry = &atlas.entries[0];
        assert_eq!(entry.id, "mode7:kOverworldMapGfx:pack0:tile42:8bpp");
        assert_eq!(entry.key.source_kind, "mode7");
        assert_eq!(entry.key.asset, "kOverworldMapGfx");
        assert_eq!(entry.key.bpp, 8);
        assert_eq!(entry.runtime_material.as_deref(), Some("palette_lut"));
        assert_eq!(entry.runtime_colors_per_row, Some(128));
        assert_eq!(entry.dynamic_policy, "stable");
        assert!(atlas.has_mode7_source_art());

        std::fs::remove_dir_all(root).expect("remove temp root");
    }

    #[test]
    fn mode7_source_art_requires_palette_lut_metadata() {
        let mut atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: solid_rgba(8, 8, [1, 2, 3, 255]),
            entries: vec![VariantAtlasEntry {
                id: "mode7:kOverworldMapGfx:pack0:tile1:8bpp".to_string(),
                key: VariantAtlasKey {
                    source_kind: "mode7".to_string(),
                    asset: "kOverworldMapGfx".to_string(),
                    pack: 0,
                    tile: 1,
                    bpp: 8,
                    palette: String::new(),
                    palette_row: 0,
                },
                rect: [0, 0, 8, 8],
                sha1: "abc".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: None,
                runtime_colors_per_row: Some(128),
                source_hflip: false,
                source_vflip: false,
            }],
            effects: Vec::new(),
        };

        assert!(!atlas.has_mode7_source_art());

        atlas.entries[0].runtime_material = Some("palette_lut".to_string());
        assert!(atlas.has_mode7_source_art());
    }

    #[test]
    fn source_entry_lookup_ignores_live_palette_material() {
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: solid_rgba(8, 8, [1, 2, 3, 255]),
            entries: vec![VariantAtlasEntry {
                id: "bg:kBgGfx:pack5:tile17:3bpp".to_string(),
                key: VariantAtlasKey {
                    source_kind: "bg".to_string(),
                    asset: "kBgGfx".to_string(),
                    pack: 5,
                    tile: 17,
                    bpp: 3,
                    palette: "palette_dung_bg_main".to_string(),
                    palette_row: 2,
                },
                rect: [0, 0, 8, 8],
                sha1: "abc".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: None,
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: Vec::new(),
        };
        let live_key = VariantAtlasKey {
            source_kind: "bg".to_string(),
            asset: "kBgGfx".to_string(),
            pack: 5,
            tile: 17,
            bpp: 3,
            palette: "palette_dung_bg_main".to_string(),
            palette_row: 6,
        };

        assert!(
            atlas.entry_for_key(&live_key).is_none(),
            "strict variant lookup still includes palette material"
        );
        assert_eq!(
            atlas
                .entry_for_source_key(&live_key)
                .expect("source art should ignore palette row")
                .id,
            "bg:kBgGfx:pack5:tile17:3bpp"
        );
    }

    #[test]
    fn effect_lookup_uses_live_palette_material() {
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: solid_rgba(8, 8, [1, 2, 3, 255]),
            entries: vec![VariantAtlasEntry {
                id: "sprite:kSprGfx:pack0:tile7:3bpp".to_string(),
                key: VariantAtlasKey {
                    source_kind: "sprite".to_string(),
                    asset: "kSprGfx".to_string(),
                    pack: 0,
                    tile: 7,
                    bpp: 3,
                    palette: "palette_main_spr".to_string(),
                    palette_row: 0,
                },
                rect: [0, 0, 8, 8],
                sha1: "abc".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: None,
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_main_spr:8color:row4".to_string(),
                palette: "palette_main_spr".to_string(),
                palette_row: 4,
                colors_per_row: 8,
                index_to_rgba: vec![[0, 0, 0, 255]; 8],
                dynamic_policy: "stable".to_string(),
            }],
        };
        let live_key = VariantAtlasKey {
            source_kind: "sprite".to_string(),
            asset: "kSprGfx".to_string(),
            pack: 0,
            tile: 7,
            bpp: 3,
            palette: "palette_main_spr".to_string(),
            palette_row: 4,
        };

        assert!(
            atlas.effect_for_entry(&atlas.entries[0]).is_none(),
            "entry effect lookup still uses preview material"
        );
        assert_eq!(
            atlas
                .effect_for_key(&live_key)
                .expect("effect should use live palette row")
                .id,
            "palette_main_spr:8color:row4"
        );
    }

    #[test]
    fn effect_lookup_uses_live_cgram_stride_for_runtime_bg_draws() {
        let atlas = bg_test_atlas(1, vec![bg_runtime_effect_with_palette_row(3)]);
        let live_key = bg_test_key_with_palette_row(3);

        let effect = atlas
            .effect_for_key(&live_key)
            .expect("runtime effect should use live CGRAM row stride");

        assert_eq!(effect.id, "palette_dung_bg_main:16color:row3");
        assert_eq!(effect.colors_per_row, 16);
    }

    #[test]
    fn resolve_draw_returns_live_effect_for_source_art() {
        let atlas = bg_test_atlas(0, vec![bg_test_effect_with_palette_row(3)]);
        let live_key = bg_test_key_with_palette_row(3);

        match atlas.resolve_draw(Some(&live_key)) {
            VariantAtlasDraw::MaterialEffect { entry, effect } => {
                assert_eq!(entry.id, "bg:kBgGfx:pack0:tile0:3bpp");
                assert_eq!(effect.id, "palette_dung_bg_main:8color:row3");
                assert_eq!(
                    atlas
                        .effect_row_for_effect(effect)
                        .expect("effect row should be resolvable"),
                    0
                );
            }
            other => panic!("expected material effect draw, got {other:?}"),
        }
    }

    #[test]
    fn resolve_draw_keeps_unmodeled_material_on_dynamic_fallback() {
        let atlas = bg_test_atlas(0, Vec::new());
        let live_key = bg_test_key_with_palette_row(3);

        match atlas.resolve_draw(Some(&live_key)) {
            VariantAtlasDraw::DynamicPalette { entry, reason } => {
                assert_eq!(entry.id, "bg:kBgGfx:pack0:tile0:3bpp");
                assert_eq!(reason, DynamicFallbackReason::MissingStableEffect);
            }
            other => panic!("expected dynamic fallback, got {other:?}"),
        }
    }

    #[test]
    fn resolve_draw_keeps_requires_live_policy_dynamic() {
        let mut entry = bg_test_entry_with_palette_row(3);
        entry.dynamic_policy = "requires_live_palette".to_string();
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: solid_rgba(8, 8, [1, 2, 3, 255]),
            entries: vec![entry],
            effects: Vec::new(),
        };
        let live_key = bg_test_key_with_palette_row(3);

        match atlas.resolve_draw(Some(&live_key)) {
            VariantAtlasDraw::DynamicPalette { reason, .. } => {
                assert_eq!(reason, DynamicFallbackReason::EntryRequiresLivePalette);
            }
            other => panic!("expected requires-live entry to stay dynamic, got {other:?}"),
        }
    }

    #[test]
    fn resolve_draw_preserves_preview_material_fast_path() {
        let atlas = bg_test_atlas(3, Vec::new());
        let live_key = bg_test_key_with_palette_row(3);

        match atlas.resolve_draw(Some(&live_key)) {
            VariantAtlasDraw::Stable { entry } => {
                assert_eq!(entry.id, "bg:kBgGfx:pack0:tile0:3bpp");
            }
            other => panic!("expected preview-backed stable draw, got {other:?}"),
        }
    }

    #[test]
    fn resolve_draw_prefers_matching_stable_art_over_untyped_effect() {
        let mut entry = bg_test_entry_with_palette_row(3);
        entry.runtime_material = None;
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: solid_rgba(8, 8, [1, 2, 3, 255]),
            entries: vec![entry],
            effects: vec![bg_test_effect_with_palette_row(3)],
        };
        let live_key = bg_test_key_with_palette_row(3);

        match atlas.resolve_draw(Some(&live_key)) {
            VariantAtlasDraw::Stable { entry } => {
                assert_eq!(entry.id, "bg:kBgGfx:pack0:tile0:3bpp");
            }
            other => panic!("expected matching stable art draw, got {other:?}"),
        }
    }

    #[test]
    fn resolve_draw_keeps_unknown_runtime_material_dynamic() {
        let mut entry = bg_test_entry_with_palette_row(3);
        entry.runtime_material = Some("shader_magic".to_string());
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: solid_rgba(8, 8, [1, 2, 3, 255]),
            entries: vec![entry],
            effects: Vec::new(),
        };
        let live_key = bg_test_key_with_palette_row(3);

        match atlas.resolve_draw(Some(&live_key)) {
            VariantAtlasDraw::DynamicPalette { entry, reason } => {
                assert_eq!(entry.id, "bg:kBgGfx:pack0:tile0:3bpp");
                assert_eq!(reason, DynamicFallbackReason::UnsupportedMaterial);
            }
            other => panic!("expected unknown material to stay dynamic, got {other:?}"),
        }
    }

    #[test]
    fn resolve_draw_reports_missing_and_unkeyed_separately() {
        let atlas = bg_test_atlas(0, Vec::new());
        let mut missing_key = bg_test_key_with_palette_row(0);
        missing_key.tile = 999;

        assert!(matches!(
            atlas.resolve_draw(Some(&missing_key)),
            VariantAtlasDraw::MissingArt
        ));
        assert!(matches!(
            atlas.resolve_draw(None),
            VariantAtlasDraw::Unkeyed
        ));
    }

    #[test]
    fn canonical_art_source_kind_default_is_stable_when_effect_backed() {
        let root = unique_temp_root();
        let atlas_dir = root.join("atlas");
        std::fs::create_dir_all(&atlas_dir).expect("create atlas dir");
        let rgba = solid_rgba(8, 8, [1, 2, 3, 255]);
        write_rgba_png(&atlas_dir.join("art_tiles.png"), 8, 8, &rgba);
        std::fs::write(
            atlas_dir.join("art_tiles.json"),
            r#"{
              "format": "zelda3_canonical_art_atlas_v1",
              "tile_width": 8,
              "tile_height": 8,
              "width": 8,
              "height": 8,
              "art_count": 1,
              "source_ref_count": 1,
              "arts": [{
                "art_id": "art:abc",
                "bpp": 3,
                "rect": [0, 0, 8, 8],
                "sha1_indices": "abc",
                "preview_palette": "palette_main_spr",
                "preview_palette_row": 0,
                "preview_source": "source_kind_default",
                "source_refs": [{
                  "source_kind": "sprite",
                  "asset": "kSprGfx",
                  "pack": 0,
                  "tile": 7,
                  "bpp": 3,
                  "hflip": false,
                  "vflip": false,
                  "preview_palette": "palette_main_spr",
                  "preview_palette_row": 0,
                  "preview_source": "source_kind_default"
                }]
              }]
            }"#,
        )
        .expect("write manifest");
        std::fs::write(
            atlas_dir.join("tile_effects.json"),
            r#"{
              "format": "zelda3_tile_effect_table_v1",
              "strategy": "base_art_plus_shader_effects",
              "effects": [{
                "id": "palette_main_spr:8color:row0",
                "type": "palette_lut",
                "palette": "palette_main_spr",
                "palette_row": 0,
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

        let atlas = load_modern_canonical_art_atlas(&root).expect("load art atlas");

        assert_eq!(atlas.entries.len(), 1);
        assert_eq!(atlas.entries[0].dynamic_policy, "stable");
        assert_eq!(
            atlas
                .effect_for_entry(&atlas.entries[0])
                .expect("resolve source-kind default effect")
                .id,
            "palette_main_spr:8color:row0"
        );

        std::fs::remove_dir_all(root).expect("remove temp root");
    }

    #[test]
    fn canonical_art_runtime_material_policy_can_force_live_palette() {
        let root = unique_temp_root();
        let atlas_dir = root.join("atlas");
        std::fs::create_dir_all(&atlas_dir).expect("create atlas dir");
        let rgba = solid_rgba(8, 8, [1, 2, 3, 255]);
        write_rgba_png(&atlas_dir.join("art_tiles.png"), 8, 8, &rgba);
        std::fs::write(
            atlas_dir.join("art_tiles.json"),
            r#"{
              "format": "zelda3_canonical_art_atlas_v1",
              "tile_width": 8,
              "tile_height": 8,
              "width": 8,
              "height": 8,
              "art_count": 1,
              "source_ref_count": 1,
              "arts": [{
                "art_id": "art:abc",
                "bpp": 3,
                "rect": [0, 0, 8, 8],
                "sha1_indices": "abc",
                "preview_palette": "palette_main_spr",
                "preview_palette_row": 0,
                "preview_source": "source_kind_default",
                "source_refs": [{
                  "source_kind": "sprite",
                  "asset": "kSprGfx",
                  "pack": 0,
                  "tile": 7,
                  "bpp": 3,
                  "hflip": false,
                  "vflip": false,
                  "preview_palette": "palette_main_spr",
                  "preview_palette_row": 0,
                  "preview_source": "source_kind_default",
                  "runtime_material": "palette_lut",
                  "runtime_material_policy": "requires_live_palette",
                  "runtime_colors_per_row": 8
                }]
              }]
            }"#,
        )
        .expect("write manifest");
        std::fs::write(
            atlas_dir.join("tile_effects.json"),
            r#"{
              "format": "zelda3_tile_effect_table_v1",
              "strategy": "base_art_plus_shader_effects",
              "effects": [{
                "id": "palette_main_spr:8color:row0",
                "type": "palette_lut",
                "palette": "palette_main_spr",
                "palette_row": 0,
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

        let atlas = load_modern_canonical_art_atlas(&root).expect("load art atlas");
        let live_key = VariantAtlasKey {
            source_kind: "sprite".to_string(),
            asset: "kSprGfx".to_string(),
            pack: 0,
            tile: 7,
            bpp: 3,
            palette: "palette_main_spr".to_string(),
            palette_row: 0,
        };

        assert_eq!(atlas.entries.len(), 1);
        assert_eq!(atlas.entries[0].dynamic_policy, "requires_live_palette");
        match atlas.resolve_draw(Some(&live_key)) {
            VariantAtlasDraw::DynamicPalette { entry, reason } => {
                assert_eq!(entry.id, "sprite:kSprGfx:pack0:tile7:3bpp");
                assert_eq!(reason, DynamicFallbackReason::EntryRequiresLivePalette);
            }
            other => panic!("expected runtime policy to force dynamic draw, got {other:?}"),
        }

        std::fs::remove_dir_all(root).expect("remove temp root");
    }

    #[test]
    fn canonical_art_runtime_colors_per_row_drives_draw_effect_lookup() {
        let root = unique_temp_root();
        let atlas_dir = root.join("atlas");
        std::fs::create_dir_all(&atlas_dir).expect("create atlas dir");
        let rgba = solid_rgba(8, 8, [1, 2, 3, 255]);
        write_rgba_png(&atlas_dir.join("art_tiles.png"), 8, 8, &rgba);
        std::fs::write(
            atlas_dir.join("art_tiles.json"),
            r#"{
              "format": "zelda3_canonical_art_atlas_v1",
              "tile_width": 8,
              "tile_height": 8,
              "width": 8,
              "height": 8,
              "art_count": 1,
              "source_ref_count": 1,
              "arts": [{
                "art_id": "art:abc",
                "bpp": 3,
                "rect": [0, 0, 8, 8],
                "sha1_indices": "abc",
                "preview_palette": "palette_dung_bg_main",
                "preview_palette_row": 3,
                "preview_source": "source_kind_default",
                "source_refs": [{
                  "source_kind": "bg",
                  "asset": "kBgGfx",
                  "pack": 0,
                  "tile": 0,
                  "bpp": 3,
                  "hflip": false,
                  "vflip": false,
                  "preview_palette": "palette_dung_bg_main",
                  "preview_palette_row": 3,
                  "preview_source": "source_kind_default",
                  "runtime_material": "palette_lut",
                  "runtime_material_policy": "stable",
                  "runtime_colors_per_row": 8
                }]
              }]
            }"#,
        )
        .expect("write manifest");
        std::fs::write(
            atlas_dir.join("tile_effects.json"),
            r#"{
              "format": "zelda3_tile_effect_table_v1",
              "strategy": "base_art_plus_shader_effects",
              "effects": [{
                "id": "palette_dung_bg_main:16color:row3",
                "type": "palette_lut",
                "palette": "palette_dung_bg_main",
                "palette_row": 3,
                "colors_per_row": 16,
                "index_to_rgb": [
                  [0, 0, 0],
                  [1, 1, 1],
                  [2, 2, 2],
                  [3, 3, 3],
                  [4, 4, 4],
                  [5, 5, 5],
                  [6, 6, 6],
                  [7, 7, 7],
                  [8, 8, 8],
                  [9, 9, 9],
                  [10, 10, 10],
                  [11, 11, 11],
                  [12, 12, 12],
                  [13, 13, 13],
                  [14, 14, 14],
                  [15, 15, 15]
                ],
                "dynamic_policy": "stable",
                "runtime": "shader_effect"
              }, {
                "id": "palette_dung_bg_main:8color:row3",
                "type": "palette_lut",
                "palette": "palette_dung_bg_main",
                "palette_row": 3,
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

        let atlas = load_modern_canonical_art_atlas(&root).expect("load art atlas");
        let live_key = bg_test_key_with_palette_row(3);

        match atlas.resolve_draw(Some(&live_key)) {
            VariantAtlasDraw::MaterialEffect { effect, .. } => {
                assert_eq!(effect.id, "palette_dung_bg_main:8color:row3");
            }
            other => panic!("expected declared material stride effect, got {other:?}"),
        }

        std::fs::remove_dir_all(root).expect("remove temp root");
    }

    #[test]
    fn canonical_art_loader_rejects_manifest_count_drift() {
        let root = unique_temp_root();
        let atlas_dir = root.join("atlas");
        std::fs::create_dir_all(&atlas_dir).expect("create atlas dir");
        let rgba = solid_rgba(8, 8, [1, 2, 3, 255]);
        write_rgba_png(&atlas_dir.join("art_tiles.png"), 8, 8, &rgba);
        std::fs::write(
            atlas_dir.join("art_tiles.json"),
            r#"{
              "format": "zelda3_canonical_art_atlas_v1",
              "tile_width": 8,
              "tile_height": 8,
              "width": 8,
              "height": 8,
              "art_count": 2,
              "source_ref_count": 1,
              "arts": [{
                "art_id": "art:abc",
                "bpp": 3,
                "rect": [0, 0, 8, 8],
                "sha1_indices": "abc",
                "preview_palette": "palette_main_spr",
                "preview_palette_row": 0,
                "preview_source": "palette_usage",
                "source_refs": [{
                  "source_kind": "sprite",
                  "asset": "kSprGfx",
                  "pack": 0,
                  "tile": 7,
                  "bpp": 3,
                  "hflip": false,
                  "vflip": false,
                  "preview_palette": "palette_main_spr",
                  "preview_palette_row": 0,
                  "preview_source": "palette_usage"
                }]
              }]
            }"#,
        )
        .expect("write manifest");

        let err = load_modern_canonical_art_atlas(&root).expect_err("reject count drift");

        assert!(err.contains("art_count 2 does not match 1 arts"), "{err}");

        std::fs::remove_dir_all(root).expect("remove temp root");
    }

    #[test]
    fn canonical_art_loader_rejects_out_of_bounds_rects() {
        let root = unique_temp_root();
        let atlas_dir = root.join("atlas");
        std::fs::create_dir_all(&atlas_dir).expect("create atlas dir");
        let rgba = solid_rgba(8, 8, [1, 2, 3, 255]);
        write_rgba_png(&atlas_dir.join("art_tiles.png"), 8, 8, &rgba);
        std::fs::write(
            atlas_dir.join("art_tiles.json"),
            r#"{
              "format": "zelda3_canonical_art_atlas_v1",
              "tile_width": 8,
              "tile_height": 8,
              "width": 8,
              "height": 8,
              "art_count": 1,
              "source_ref_count": 1,
              "arts": [{
                "art_id": "art:abc",
                "bpp": 3,
                "rect": [4, 0, 8, 8],
                "sha1_indices": "abc",
                "preview_palette": "palette_main_spr",
                "preview_palette_row": 0,
                "preview_source": "palette_usage",
                "source_refs": [{
                  "source_kind": "sprite",
                  "asset": "kSprGfx",
                  "pack": 0,
                  "tile": 7,
                  "bpp": 3,
                  "hflip": false,
                  "vflip": false,
                  "preview_palette": "palette_main_spr",
                  "preview_palette_row": 0,
                  "preview_source": "palette_usage"
                }]
              }]
            }"#,
        )
        .expect("write manifest");

        let err = load_modern_canonical_art_atlas(&root).expect_err("reject bad rect");

        assert!(
            err.contains("art rect [4, 0, 8, 8] for art:abc is outside PNG bounds 8x8"),
            "{err}"
        );

        std::fs::remove_dir_all(root).expect("remove temp root");
    }
}
