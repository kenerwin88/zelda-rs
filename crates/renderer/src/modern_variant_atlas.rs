use serde::Deserialize;
use std::collections::HashMap;
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
    pub mode7_source_chars: Option<Vec<u8>>,
    pub dialogue_glyph_atlas: Option<DialogueGlyphAtlas>,
    pub dialogue_vwf_font: Option<DialogueVwfFont>,
    pub dialogue_vwf_glyph_atlas: Option<DialogueVwfGlyphAtlas>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogueGlyphAtlas {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub tiles: Vec<DialogueGlyphTile>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct DialogueGlyphTile {
    pub id: String,
    pub rect: [u32; 4],
    pub relative_tile: u16,
    pub source_block: String,
    pub source_bpp: u8,
    pub source_kind: String,
    pub source_pack: u16,
    pub source_sheet: String,
    pub source_tile: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogueVwfFont {
    pub glyph_count: u32,
    pub width_table_size: u32,
    pub glyphs: Vec<DialogueVwfGlyph>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogueVwfGlyph {
    pub code: u8,
    pub hex: String,
    pub width: u8,
    pub top_row_offset: u16,
    pub bottom_row_offset: u16,
    pub top_row_bytes: [u8; 16],
    pub bottom_row_bytes: [u8; 16],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogueVwfGlyphAtlas {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub glyphs: Vec<DialogueVwfGlyphCell>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogueVwfGlyphCell {
    pub code: u8,
    pub hex: String,
    pub width: u8,
    pub palette: String,
    pub rect: [u32; 4],
    pub indices: Vec<u8>,
}

pub const DIALOGUE_TEXT_MAIN_PALETTE: &str = "palette_bg3_text_main";

pub fn dialogue_text_color_palette_name(color: u8) -> String {
    format!("palette_bg3_text_color_{:02x}", color & 0x0f)
}

pub fn dialogue_vwf_variant_key(
    glyph_code: u16,
    quadrant: u16,
    palette: impl Into<String>,
) -> VariantAtlasKey {
    VariantAtlasKey {
        source_kind: "dialogue_vwf".to_string(),
        asset: "kDialogueVwfGlyphs".to_string(),
        pack: glyph_code,
        tile: quadrant,
        bpp: 2,
        palette: palette.into(),
        palette_row: 0,
    }
}

pub fn dialogue_vwf_variant_entry(
    atlas: &ModernVariantAtlas,
    glyph_code: u16,
    quadrant: u16,
    dialogue_color: Option<u8>,
) -> Option<&VariantAtlasEntry> {
    if let Some(color) = dialogue_color {
        let color_key = dialogue_vwf_variant_key(
            glyph_code,
            quadrant,
            dialogue_text_color_palette_name(color),
        );
        if let Some(entry) = atlas.entry_for_key(&color_key) {
            return Some(entry);
        }
    }
    let main_key = dialogue_vwf_variant_key(glyph_code, quadrant, DIALOGUE_TEXT_MAIN_PALETTE);
    atlas
        .entry_for_key(&main_key)
        .or_else(|| atlas.entry_for_source_key(&main_key))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DialogueFontTileAtlas {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    tiles: Vec<DialogueFontTile>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct DialogueFontTile {
    id: String,
    rect: [u32; 4],
    source_pack: u16,
    source_tile: u16,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) struct VariantSourceLookupKey {
    source_kind: u8,
    asset: u8,
    pack: u16,
    tile: u16,
    bpp: u8,
}

impl VariantSourceLookupKey {
    pub(crate) fn from_key(key: &VariantAtlasKey) -> Self {
        Self {
            source_kind: source_kind_lookup_id(&key.source_kind),
            asset: asset_lookup_id(&key.asset),
            pack: key.pack,
            tile: key.tile,
            bpp: key.bpp,
        }
    }

    pub(crate) fn from_entry(entry: &VariantAtlasEntry) -> Self {
        Self {
            source_kind: source_kind_lookup_id(&entry.key.source_kind),
            asset: asset_lookup_id(&entry.key.asset),
            pack: entry.key.pack,
            tile: entry.key.tile,
            bpp: entry.key.bpp,
        }
    }
}

impl<'a> From<&'a VariantAtlasKey> for VariantRuntimeKey<'a> {
    fn from(key: &'a VariantAtlasKey) -> Self {
        Self {
            lookup: VariantSourceLookupKey::from_key(key),
            source_kind: source_kind_lookup_id(&key.source_kind),
            bpp: key.bpp,
            palette: &key.palette,
            palette_row: key.palette_row,
        }
    }
}

const SOURCE_KIND_BG_LOOKUP_ID: u8 = 1;
const SOURCE_KIND_SPRITE_LOOKUP_ID: u8 = 2;

fn source_kind_lookup_id(source_kind: &str) -> u8 {
    match source_kind {
        "bg" => SOURCE_KIND_BG_LOOKUP_ID,
        "sprite" => SOURCE_KIND_SPRITE_LOOKUP_ID,
        "mode7" => 3,
        "bg3" => 4,
        "bg3_dynamic" => 7,
        "link" => 8,
        "dialogue_glyph" => 9,
        "dialogue_vwf" => 10,
        "dialogue_font" => 11,
        _ => 0xff,
    }
}

fn asset_lookup_id(asset: &str) -> u8 {
    match asset {
        "kBgGfx" => 1,
        "kSprGfx" => 2,
        "kOverworldMapGfx" => 3,
        "kBg3Gfx" => 4,
        "kBg3TextGfx" => 7,
        "kLinkGfx" => 8,
        "kDialogueGlyphTiles" => 9,
        "kDialogueVwfGlyphs" => 10,
        "kDialogueFontTiles" => 11,
        _ => 0xff,
    }
}

pub(crate) type SourceEntryIndex = HashMap<VariantSourceLookupKey, usize>;

#[derive(Clone, Copy, Debug)]
pub(crate) struct VariantRuntimeKey<'a> {
    lookup: VariantSourceLookupKey,
    source_kind: u8,
    bpp: u8,
    palette: &'a str,
    palette_row: u8,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) struct EffectLookupKey<'a> {
    palette: &'a str,
    palette_row: u8,
    colors_per_row: u8,
}

pub(crate) type EffectLookupIndex<'a> = HashMap<EffectLookupKey<'a>, &'a TileEffect>;

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
        4 => ("bg3", "kBg3Gfx", 5),
        7 => ("bg3_dynamic", "kBg3TextGfx", 5),
        8 => ("link", "kLinkGfx", 3),
        9 => ("dialogue_glyph", "kDialogueGlyphTiles", 2),
        10 => ("dialogue_vwf", "kDialogueVwfGlyphs", 2),
        11 => ("dialogue_font", "kDialogueFontTiles", 2),
        _ => return None,
    };
    let is_dialogue_source = matches!(
        source_kind,
        "dialogue_glyph" | "dialogue_vwf" | "dialogue_font"
    );
    let palette = if is_dialogue_source {
        DIALOGUE_TEXT_MAIN_PALETTE
    } else {
        palette_name
    };
    let palette_row = if is_dialogue_source { 0 } else { palette_row };
    Some(VariantAtlasKey {
        source_kind: source_kind.to_string(),
        asset: asset.to_string(),
        pack,
        tile,
        bpp,
        palette: palette.to_string(),
        palette_row,
    })
}

pub(crate) fn variant_runtime_key_for_source_key<'a>(
    source_key: u64,
    palette_name: &'a str,
    palette_row: u8,
) -> Option<VariantRuntimeKey<'a>> {
    if source_key == crate::modern_hd_overrides::NO_SOURCE_KEY {
        return None;
    }
    let kind = (source_key >> 32) as u8;
    let pack = ((source_key >> 16) & 0xffff) as u16;
    let tile = (source_key & 0xffff) as u16;
    let (source_kind, asset, bpp) = match kind {
        1 | 5 | 6 => (source_kind_lookup_id("bg"), asset_lookup_id("kBgGfx"), 3),
        2 => (
            source_kind_lookup_id("sprite"),
            asset_lookup_id("kSprGfx"),
            3,
        ),
        4 => (source_kind_lookup_id("bg3"), asset_lookup_id("kBg3Gfx"), 5),
        7 => (
            source_kind_lookup_id("bg3_dynamic"),
            asset_lookup_id("kBg3TextGfx"),
            5,
        ),
        8 => (
            source_kind_lookup_id("link"),
            asset_lookup_id("kLinkGfx"),
            3,
        ),
        9 => (
            source_kind_lookup_id("dialogue_glyph"),
            asset_lookup_id("kDialogueGlyphTiles"),
            2,
        ),
        10 => (
            source_kind_lookup_id("dialogue_vwf"),
            asset_lookup_id("kDialogueVwfGlyphs"),
            2,
        ),
        11 => (
            source_kind_lookup_id("dialogue_font"),
            asset_lookup_id("kDialogueFontTiles"),
            2,
        ),
        _ => return None,
    };
    let is_dialogue_source = matches!(kind, 9..=11);
    Some(VariantRuntimeKey {
        lookup: VariantSourceLookupKey {
            source_kind,
            asset,
            pack,
            tile,
            bpp,
        },
        source_kind,
        bpp,
        palette: if is_dialogue_source {
            DIALOGUE_TEXT_MAIN_PALETTE
        } else {
            palette_name
        },
        palette_row: if is_dialogue_source { 0 } else { palette_row },
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

    pub(crate) fn build_source_entry_index(&self) -> SourceEntryIndex {
        let mut index = SourceEntryIndex::with_capacity(self.entries.len());
        for (entry_index, entry) in self.entries.iter().enumerate() {
            index
                .entry(VariantSourceLookupKey::from_entry(entry))
                .or_insert(entry_index);
        }
        index
    }

    pub(crate) fn build_effect_lookup_index(&self) -> EffectLookupIndex<'_> {
        let mut index = EffectLookupIndex::with_capacity(self.effects.len());
        for effect in &self.effects {
            index
                .entry(EffectLookupKey {
                    palette: &effect.palette,
                    palette_row: effect.palette_row,
                    colors_per_row: effect.colors_per_row,
                })
                .or_insert(effect);
        }
        index
    }

    pub(crate) fn entry_for_source_key_in_index(
        &self,
        key: &VariantAtlasKey,
        index: &SourceEntryIndex,
    ) -> Option<&VariantAtlasEntry> {
        index
            .get(&VariantSourceLookupKey::from_key(key))
            .and_then(|entry_index| self.entries.get(*entry_index))
    }

    pub(crate) fn resolve_draw_for_runtime_key_in_indexes<'a>(
        &'a self,
        key: Option<VariantRuntimeKey<'_>>,
        index: &SourceEntryIndex,
        effect_index: Option<&EffectLookupIndex<'a>>,
    ) -> VariantAtlasDraw<'a> {
        let Some(key) = key else {
            return VariantAtlasDraw::Unkeyed;
        };
        let Some(entry) = index
            .get(&key.lookup)
            .and_then(|entry_index| self.entries.get(*entry_index))
        else {
            return VariantAtlasDraw::MissingArt;
        };
        self.resolve_draw_for_runtime_entry(&key, entry, effect_index)
    }

    pub(crate) fn resolve_dynamic_draw_for_runtime_key_in_index<'a>(
        &'a self,
        key: Option<VariantRuntimeKey<'_>>,
        reason: DynamicFallbackReason,
        index: &SourceEntryIndex,
    ) -> VariantAtlasDraw<'a> {
        let Some(key) = key else {
            return VariantAtlasDraw::Unkeyed;
        };
        let Some(entry) = index
            .get(&key.lookup)
            .and_then(|entry_index| self.entries.get(*entry_index))
        else {
            return VariantAtlasDraw::MissingArt;
        };
        VariantAtlasDraw::DynamicPalette { entry, reason }
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

    pub fn mode7_source_chars(&self) -> Option<&[u8]> {
        self.mode7_source_chars.as_deref()
    }

    pub fn effect_for_entry(&self, entry: &VariantAtlasEntry) -> Option<&TileEffect> {
        self.effect_for_entry_and_key(entry, &entry.key)
    }

    pub fn effect_for_key(&self, key: &VariantAtlasKey) -> Option<&TileEffect> {
        self.effect_for_key_with_color_rows(key, None)
    }

    fn effect_for_entry_and_key(
        &self,
        entry: &VariantAtlasEntry,
        key: &VariantAtlasKey,
    ) -> Option<&TileEffect> {
        self.effect_for_key_with_color_rows(key, entry.runtime_colors_per_row)
    }

    fn effect_for_runtime_entry_and_key<'a>(
        &'a self,
        entry: &VariantAtlasEntry,
        key: &VariantRuntimeKey<'_>,
        effect_index: Option<&EffectLookupIndex<'a>>,
    ) -> Option<&'a TileEffect> {
        self.effect_for_runtime_key_with_color_rows(key, entry.runtime_colors_per_row, effect_index)
    }

    fn effect_for_key_with_color_rows(
        &self,
        key: &VariantAtlasKey,
        runtime_colors_per_row: Option<u8>,
    ) -> Option<&TileEffect> {
        self.effect_for_runtime_key_with_color_rows(
            &VariantRuntimeKey::from(key),
            runtime_colors_per_row,
            None,
        )
    }

    fn effect_for_runtime_key_with_color_rows<'a>(
        &'a self,
        key: &VariantRuntimeKey<'_>,
        runtime_colors_per_row: Option<u8>,
        effect_index: Option<&EffectLookupIndex<'a>>,
    ) -> Option<&'a TileEffect> {
        if let Some(colors_per_row) = runtime_colors_per_row {
            return self.effect_for_color_row(key, colors_per_row, effect_index);
        }

        let source_stride = 1u8.checked_shl(u32::from(key.bpp))?;
        if matches!(
            key.source_kind,
            SOURCE_KIND_BG_LOOKUP_ID | SOURCE_KIND_SPRITE_LOOKUP_ID
        ) {
            if let Some(effect) = self.effect_for_color_row(key, 16, effect_index) {
                return Some(effect);
            }
            if source_stride == 16 {
                return None;
            }
        }
        self.effect_for_color_row(key, source_stride, effect_index)
    }

    fn effect_for_color_row<'a>(
        &'a self,
        key: &VariantRuntimeKey<'_>,
        colors_per_row: u8,
        effect_index: Option<&EffectLookupIndex<'a>>,
    ) -> Option<&'a TileEffect> {
        if let Some(index) = effect_index {
            return index
                .get(&EffectLookupKey {
                    palette: key.palette,
                    palette_row: key.palette_row,
                    colors_per_row,
                })
                .copied();
        }
        self.effects.iter().find(|effect| {
            effect.palette == key.palette
                && effect.palette_row == key.palette_row
                && effect.colors_per_row == colors_per_row
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
        self.resolve_draw_for_entry(key, entry)
    }

    pub(crate) fn resolve_draw_in_index<'a>(
        &'a self,
        key: Option<&VariantAtlasKey>,
        index: &SourceEntryIndex,
    ) -> VariantAtlasDraw<'a> {
        let Some(key) = key else {
            return VariantAtlasDraw::Unkeyed;
        };
        let Some(entry) = self.entry_for_source_key_in_index(key, index) else {
            return VariantAtlasDraw::MissingArt;
        };
        self.resolve_draw_for_entry(key, entry)
    }

    fn resolve_draw_for_entry<'a>(
        &'a self,
        key: &VariantAtlasKey,
        entry: &'a VariantAtlasEntry,
    ) -> VariantAtlasDraw<'a> {
        self.resolve_draw_for_runtime_entry(&VariantRuntimeKey::from(key), entry, None)
    }

    fn resolve_draw_for_runtime_entry<'a>(
        &'a self,
        key: &VariantRuntimeKey<'_>,
        entry: &'a VariantAtlasEntry,
        effect_index: Option<&EffectLookupIndex<'a>>,
    ) -> VariantAtlasDraw<'a> {
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
                    .effect_for_runtime_entry_and_key(entry, key, effect_index)
                    .filter(|effect| effect_dynamic_policy(effect) == DynamicPolicy::Stable);
                if let Some(effect) = stable_effect {
                    return VariantAtlasDraw::MaterialEffect { entry, effect };
                }
                if entry_matches_runtime_material(entry, key) {
                    return VariantAtlasDraw::Stable { entry };
                }
            }
            RuntimeMaterial::LegacyPreview if !entry_matches_runtime_material(entry, key) => {
                let stable_effect = self
                    .effect_for_runtime_entry_and_key(entry, key, effect_index)
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

    pub(crate) fn resolve_dynamic_draw_in_index<'a>(
        &'a self,
        key: Option<&VariantAtlasKey>,
        reason: DynamicFallbackReason,
        index: &SourceEntryIndex,
    ) -> VariantAtlasDraw<'a> {
        let Some(key) = key else {
            return VariantAtlasDraw::Unkeyed;
        };
        let Some(entry) = self.entry_for_source_key_in_index(key, index) else {
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

fn entry_matches_runtime_material(entry: &VariantAtlasEntry, key: &VariantRuntimeKey<'_>) -> bool {
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
        mode7_source_chars: None,
        dialogue_glyph_atlas: None,
        dialogue_vwf_font: None,
        dialogue_vwf_glyph_atlas: None,
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

    let manifest = load_art_manifest(&json_path, "zelda3_canonical_art_atlas_v1")?;
    let png = load_rgba_png(&png_path)?;
    if png.width != manifest.width || png.height != manifest.height {
        return Err(format!(
            "{}: PNG size {}x{} does not match manifest {}x{}",
            png_path.display(),
            png.width,
            png.height,
            manifest.width,
            manifest.height
        ));
    }
    validate_art_manifest_counts_and_rects(&json_path, &manifest, png.width, png.height)?;

    let dynamic_json_path = atlas_dir.join("dynamic_bg3_tiles.json");
    let dynamic_png_path = atlas_dir.join("dynamic_bg3_tiles.png");
    let dynamic_manifest = if dynamic_json_path.is_file() {
        let manifest = load_art_manifest(&dynamic_json_path, "zelda3_dynamic_bg3_art_atlas_v1")?;
        validate_art_manifest_counts_and_rects(
            &dynamic_json_path,
            &manifest,
            manifest.width,
            manifest.height,
        )?;
        Some(manifest)
    } else if dynamic_png_path.is_file() {
        return Err(format!(
            "dynamic BG3 manifest missing: expected {} for {}",
            dynamic_json_path.display(),
            dynamic_png_path.display()
        ));
    } else {
        None
    };

    let dialogue_glyph_atlas = load_dialogue_glyph_atlas_from_dir(atlas_dir)?;
    let dialogue_vwf_font = load_dialogue_vwf_font_from_dir(atlas_dir)?;
    let dialogue_vwf_glyph_atlas = load_dialogue_vwf_glyph_atlas_from_dir(atlas_dir)?;
    let dialogue_font_tile_atlas = load_dialogue_font_tile_atlas_from_dir(atlas_dir)?;

    let atlas_width = [
        png.width,
        dialogue_glyph_atlas
            .as_ref()
            .map_or(0, |glyphs| glyphs.width),
        dialogue_vwf_glyph_atlas
            .as_ref()
            .map_or(0, |glyphs| glyphs.width),
        dialogue_font_tile_atlas
            .as_ref()
            .map_or(0, |tiles| tiles.width),
    ]
    .into_iter()
    .max()
    .unwrap_or(png.width);
    let glyph_y_offset = png.height;
    let vwf_glyph_y_offset = glyph_y_offset
        + dialogue_glyph_atlas
            .as_ref()
            .map_or(0, |glyphs| glyphs.height);
    let font_tile_y_offset = vwf_glyph_y_offset
        + dialogue_vwf_glyph_atlas
            .as_ref()
            .map_or(0, |glyphs| glyphs.height);
    let atlas_height = font_tile_y_offset
        + dialogue_font_tile_atlas
            .as_ref()
            .map_or(0, |tiles| tiles.height);
    let mut combined_rgba = vec![0; (atlas_width as usize) * (atlas_height as usize) * 4];
    blit_rgba_sheet(
        &mut combined_rgba,
        atlas_width,
        &png.rgba,
        png.width,
        png.height,
        0,
    );

    let effects = load_tile_effects_from_dir(atlas_dir)?;
    let mut entries = Vec::new();
    let mut mode7_source_chars = vec![0u8; 0x4000];
    let mut has_mode7_source_chars = false;
    append_art_manifest_entries(
        &json_path,
        manifest.arts,
        &effects,
        0,
        &mut entries,
        &mut mode7_source_chars,
        &mut has_mode7_source_chars,
    )?;
    if let Some(dynamic_manifest) = dynamic_manifest {
        append_art_manifest_entries(
            &dynamic_json_path,
            dynamic_manifest.arts,
            &effects,
            0,
            &mut entries,
            &mut mode7_source_chars,
            &mut has_mode7_source_chars,
        )?;
    }
    if let Some(glyph_atlas) = &dialogue_glyph_atlas {
        blit_rgba_sheet(
            &mut combined_rgba,
            atlas_width,
            &glyph_atlas.rgba,
            glyph_atlas.width,
            glyph_atlas.height,
            glyph_y_offset,
        );
        append_dialogue_glyph_entries(glyph_atlas, glyph_y_offset, &mut entries)?;
    }
    if let Some(vwf_glyph_atlas) = &dialogue_vwf_glyph_atlas {
        blit_rgba_sheet(
            &mut combined_rgba,
            atlas_width,
            &vwf_glyph_atlas.rgba,
            vwf_glyph_atlas.width,
            vwf_glyph_atlas.height,
            vwf_glyph_y_offset,
        );
        append_dialogue_vwf_glyph_entries(vwf_glyph_atlas, vwf_glyph_y_offset, &mut entries)?;
    }
    if let Some(font_tile_atlas) = &dialogue_font_tile_atlas {
        blit_rgba_sheet(
            &mut combined_rgba,
            atlas_width,
            &font_tile_atlas.rgba,
            font_tile_atlas.width,
            font_tile_atlas.height,
            font_tile_y_offset,
        );
        append_dialogue_font_tile_entries(font_tile_atlas, font_tile_y_offset, &mut entries)?;
    }

    Ok(ModernVariantAtlas {
        width: atlas_width,
        height: atlas_height,
        rgba: combined_rgba,
        entries,
        effects,
        mode7_source_chars: has_mode7_source_chars.then_some(mode7_source_chars),
        dialogue_glyph_atlas,
        dialogue_vwf_font,
        dialogue_vwf_glyph_atlas,
    })
}

fn load_dialogue_glyph_atlas_from_dir(
    atlas_dir: &Path,
) -> Result<Option<DialogueGlyphAtlas>, String> {
    let json_path = atlas_dir.join("dialogue_glyph_tiles.json");
    let png_path = atlas_dir.join("dialogue_glyph_tiles.png");
    if !json_path.is_file() && !png_path.is_file() {
        return Ok(None);
    }

    let manifest_bytes = std::fs::read(&json_path)
        .map_err(|e| format!("failed to read {}: {e}", json_path.display()))?;
    let manifest: DialogueGlyphManifestJson = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("failed to parse {}: {e}", json_path.display()))?;
    if manifest.format != "zelda3_dialogue_glyph_source_atlas_v1" {
        return Err(format!(
            "{}: unsupported format {:?}",
            json_path.display(),
            manifest.format
        ));
    }
    if manifest.tile_count != manifest.tiles.len() as u32 {
        return Err(format!(
            "{}: tile_count {} does not match {} tiles",
            json_path.display(),
            manifest.tile_count,
            manifest.tiles.len()
        ));
    }

    let png = load_rgba_png(&png_path)?;
    if png.width != manifest.width || png.height != manifest.height {
        return Err(format!(
            "{}: PNG size {}x{} does not match manifest {}x{}",
            png_path.display(),
            png.width,
            png.height,
            manifest.width,
            manifest.height
        ));
    }
    for tile in &manifest.tiles {
        if !rect_within_atlas(tile.rect, png.width, png.height) {
            return Err(format!(
                "{}: glyph rect {:?} for {} is outside PNG bounds {}x{}",
                json_path.display(),
                tile.rect,
                tile.id,
                png.width,
                png.height
            ));
        }
    }

    Ok(Some(DialogueGlyphAtlas {
        width: png.width,
        height: png.height,
        rgba: png.rgba,
        tiles: manifest.tiles,
    }))
}

fn load_dialogue_font_tile_atlas_from_dir(
    atlas_dir: &Path,
) -> Result<Option<DialogueFontTileAtlas>, String> {
    let json_path = atlas_dir.join("dialogue_font_tiles.json");
    let png_path = atlas_dir.join("dialogue_font_tiles.png");
    if !json_path.is_file() && !png_path.is_file() {
        return Ok(None);
    }

    let manifest_bytes = std::fs::read(&json_path)
        .map_err(|e| format!("failed to read {}: {e}", json_path.display()))?;
    let manifest: DialogueFontTileManifestJson = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("failed to parse {}: {e}", json_path.display()))?;
    if manifest.format != "zelda3_dialogue_font_tile_atlas_v1" {
        return Err(format!(
            "{}: unsupported format {:?}",
            json_path.display(),
            manifest.format
        ));
    }
    if manifest.tile_count != manifest.tiles.len() as u32 {
        return Err(format!(
            "{}: tile_count {} does not match {} tiles",
            json_path.display(),
            manifest.tile_count,
            manifest.tiles.len()
        ));
    }

    let png = load_rgba_png(&png_path)?;
    if png.width != manifest.width || png.height != manifest.height {
        return Err(format!(
            "{}: PNG size {}x{} does not match manifest {}x{}",
            png_path.display(),
            png.width,
            png.height,
            manifest.width,
            manifest.height
        ));
    }
    for tile in &manifest.tiles {
        if !rect_within_atlas(tile.rect, png.width, png.height) {
            return Err(format!(
                "{}: dialogue font rect {:?} for {} is outside PNG bounds {}x{}",
                json_path.display(),
                tile.rect,
                tile.id,
                png.width,
                png.height
            ));
        }
    }

    Ok(Some(DialogueFontTileAtlas {
        width: png.width,
        height: png.height,
        rgba: png.rgba,
        tiles: manifest.tiles,
    }))
}

fn load_dialogue_vwf_font_from_dir(atlas_dir: &Path) -> Result<Option<DialogueVwfFont>, String> {
    let json_path = atlas_dir.join("dialogue_vwf_font.json");
    if !json_path.is_file() {
        return Ok(None);
    }

    let manifest_bytes = std::fs::read(&json_path)
        .map_err(|e| format!("failed to read {}: {e}", json_path.display()))?;
    let manifest: DialogueVwfFontJson = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("failed to parse {}: {e}", json_path.display()))?;
    if manifest.format != "zelda3_dialogue_vwf_font_v1" {
        return Err(format!(
            "{}: unsupported format {:?}",
            json_path.display(),
            manifest.format
        ));
    }
    if manifest.glyph_count != manifest.glyphs.len() as u32 {
        return Err(format!(
            "{}: glyph_count {} does not match {} glyphs",
            json_path.display(),
            manifest.glyph_count,
            manifest.glyphs.len()
        ));
    }
    if manifest.width_table_size != manifest.glyphs.len() as u32 {
        return Err(format!(
            "{}: width_table_size {} does not match {} glyphs",
            json_path.display(),
            manifest.width_table_size,
            manifest.glyphs.len()
        ));
    }

    let glyphs = manifest
        .glyphs
        .into_iter()
        .map(|glyph| {
            Ok(DialogueVwfGlyph {
                code: glyph.code,
                hex: glyph.hex,
                width: glyph.width,
                top_row_offset: glyph.top_row_offset,
                bottom_row_offset: glyph.bottom_row_offset,
                top_row_bytes: parse_fixed_hex_16(
                    &json_path,
                    "top_row_bytes",
                    &glyph.top_row_bytes,
                )?,
                bottom_row_bytes: parse_fixed_hex_16(
                    &json_path,
                    "bottom_row_bytes",
                    &glyph.bottom_row_bytes,
                )?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(Some(DialogueVwfFont {
        glyph_count: manifest.glyph_count,
        width_table_size: manifest.width_table_size,
        glyphs,
    }))
}

fn load_dialogue_vwf_glyph_atlas_from_dir(
    atlas_dir: &Path,
) -> Result<Option<DialogueVwfGlyphAtlas>, String> {
    let json_path = atlas_dir.join("dialogue_vwf_glyphs.json");
    let png_path = atlas_dir.join("dialogue_vwf_glyphs.png");
    if !json_path.is_file() && !png_path.is_file() {
        return Ok(None);
    }

    let manifest_bytes = std::fs::read(&json_path)
        .map_err(|e| format!("failed to read {}: {e}", json_path.display()))?;
    let manifest: DialogueVwfGlyphAtlasJson = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("failed to parse {}: {e}", json_path.display()))?;
    if manifest.format != "zelda3_dialogue_vwf_glyph_atlas_v1" {
        return Err(format!(
            "{}: unsupported format {:?}",
            json_path.display(),
            manifest.format
        ));
    }
    if manifest.glyph_count != manifest.glyphs.len() as u32 {
        return Err(format!(
            "{}: glyph_count {} does not match {} glyphs",
            json_path.display(),
            manifest.glyph_count,
            manifest.glyphs.len()
        ));
    }

    let png = load_rgba_png(&png_path)?;
    if png.width != manifest.width || png.height != manifest.height {
        return Err(format!(
            "{}: PNG size {}x{} does not match manifest {}x{}",
            png_path.display(),
            png.width,
            png.height,
            manifest.width,
            manifest.height
        ));
    }

    let mut glyphs = Vec::with_capacity(manifest.glyphs.len());
    for glyph in manifest.glyphs {
        if !rect_within_atlas(glyph.rect, png.width, png.height) {
            return Err(format!(
                "{}: VWF glyph rect {:?} for {} is outside PNG bounds {}x{}",
                json_path.display(),
                glyph.rect,
                glyph.hex,
                png.width,
                png.height
            ));
        }
        let indices = parse_hex_bytes_exact(
            &json_path,
            "indices_hex",
            &glyph.indices_hex,
            (manifest.cell_width as usize) * (manifest.cell_height as usize),
        )?;
        glyphs.push(DialogueVwfGlyphCell {
            code: glyph.code,
            hex: glyph.hex,
            width: glyph.width,
            palette: glyph.palette,
            rect: glyph.rect,
            indices,
        });
    }

    Ok(Some(DialogueVwfGlyphAtlas {
        width: png.width,
        height: png.height,
        rgba: png.rgba,
        glyphs,
    }))
}

fn decode_art_indices_hex(
    json_path: &Path,
    art: &ArtEntryJson,
) -> Result<Option<[u8; 64]>, String> {
    let Some(indices_hex) = art.indices_hex.as_deref() else {
        return Ok(None);
    };
    if indices_hex.len() != 128 {
        return Err(format!(
            "{}: indices_hex for {} must be 128 hex characters",
            json_path.display(),
            art.art_id
        ));
    }

    let mut indices = [0u8; 64];
    for (idx, chunk) in indices_hex.as_bytes().chunks_exact(2).enumerate() {
        let text = std::str::from_utf8(chunk).map_err(|e| {
            format!(
                "{}: invalid UTF-8 in indices_hex for {}: {e}",
                json_path.display(),
                art.art_id
            )
        })?;
        indices[idx] = u8::from_str_radix(text, 16).map_err(|e| {
            format!(
                "{}: invalid hex byte {:?} in indices_hex for {}: {e}",
                json_path.display(),
                text,
                art.art_id
            )
        })?;
    }
    Ok(Some(indices))
}

fn parse_fixed_hex_16(json_path: &Path, field: &str, value: &str) -> Result<[u8; 16], String> {
    let parsed = parse_hex_bytes_exact(json_path, field, value, 16)?;
    parsed.try_into().map_err(|_| {
        format!(
            "{}: {field} expected 16 bytes after parsing",
            json_path.display()
        )
    })
}

fn parse_hex_bytes_exact(
    json_path: &Path,
    field: &str,
    value: &str,
    expected_bytes: usize,
) -> Result<Vec<u8>, String> {
    let expected_hex_chars = expected_bytes * 2;
    if value.len() != expected_hex_chars {
        return Err(format!(
            "{}: {field} must be {expected_hex_chars} hex characters, got {}",
            json_path.display(),
            value.len()
        ));
    }
    let mut bytes = vec![0u8; expected_bytes];
    for (idx, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        let text = std::str::from_utf8(chunk).map_err(|e| {
            format!(
                "{}: invalid UTF-8 in {field} at byte {idx}: {e}",
                json_path.display()
            )
        })?;
        bytes[idx] = u8::from_str_radix(text, 16).map_err(|e| {
            format!(
                "{}: invalid hex byte {:?} in {field}: {e}",
                json_path.display(),
                text
            )
        })?;
    }
    Ok(bytes)
}

struct RgbaSheet {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

fn load_art_manifest(json_path: &Path, expected_format: &str) -> Result<ArtManifestJson, String> {
    let manifest_bytes = std::fs::read(json_path)
        .map_err(|e| format!("failed to read {}: {e}", json_path.display()))?;
    let manifest: ArtManifestJson = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("failed to parse {}: {e}", json_path.display()))?;
    if manifest.format != expected_format {
        return Err(format!(
            "{}: unsupported format {:?}",
            json_path.display(),
            manifest.format
        ));
    }
    Ok(manifest)
}

fn load_rgba_png(png_path: &Path) -> Result<RgbaSheet, String> {
    let file = std::fs::File::open(png_path)
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
    Ok(RgbaSheet {
        width: info.width,
        height: info.height,
        rgba: buf[..info.buffer_size()].to_vec(),
    })
}

fn validate_art_manifest_counts_and_rects(
    json_path: &Path,
    manifest: &ArtManifestJson,
    width: u32,
    height: u32,
) -> Result<(), String> {
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
        if !rect_within_atlas(art.rect, width, height) {
            return Err(format!(
                "{}: art rect {:?} for {} is outside PNG bounds {}x{}",
                json_path.display(),
                art.rect,
                art.art_id,
                width,
                height
            ));
        }
    }
    Ok(())
}

fn blit_rgba_sheet(
    destination: &mut [u8],
    destination_width: u32,
    source: &[u8],
    source_width: u32,
    source_height: u32,
    destination_y: u32,
) {
    let destination_width = destination_width as usize;
    let source_width = source_width as usize;
    for row in 0..source_height as usize {
        let src = row * source_width * 4;
        let dst = ((destination_y as usize + row) * destination_width) * 4;
        destination[dst..dst + source_width * 4]
            .copy_from_slice(&source[src..src + source_width * 4]);
    }
}

fn append_art_manifest_entries(
    json_path: &Path,
    arts: Vec<ArtEntryJson>,
    effects: &[TileEffect],
    y_offset: u32,
    entries: &mut Vec<VariantAtlasEntry>,
    mode7_source_chars: &mut [u8],
    has_mode7_source_chars: &mut bool,
) -> Result<(), String> {
    for art in arts {
        let art_indices = decode_art_indices_hex(json_path, &art)?;
        let rect = [
            art.rect[0],
            art.rect[1].checked_add(y_offset).ok_or_else(|| {
                format!(
                    "{}: art rect y offset overflow for {}",
                    json_path.display(),
                    art.art_id
                )
            })?,
            art.rect[2],
            art.rect[3],
        ];
        for source_ref in art.source_refs {
            let dynamic_policy = dynamic_policy_for_source_ref(effects, &source_ref);
            let runtime_material = runtime_material_for_source_ref(effects, &source_ref);
            let is_mode7_source = source_ref.source_kind == "mode7"
                && source_ref.asset == "kOverworldMapGfx"
                && source_ref.pack == 0
                && source_ref.bpp == 8
                && usize::from(source_ref.tile) < 256;
            let source_hflip = source_ref.hflip;
            let source_vflip = source_ref.vflip;
            let source_tile = source_ref.tile;
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
                rect,
                sha1: art.sha1_indices.clone(),
                duplicate_of: None,
                dynamic_policy,
                runtime_material,
                runtime_colors_per_row: source_ref.runtime_colors_per_row,
                source_hflip,
                source_vflip,
            });
            if is_mode7_source {
                if let Some(indices) = &art_indices {
                    let source_indices = transform_art_indices(indices, source_hflip, source_vflip);
                    let start = usize::from(source_tile) * 64;
                    mode7_source_chars[start..start + 64].copy_from_slice(&source_indices);
                    *has_mode7_source_chars = true;
                }
            }
        }
    }
    Ok(())
}

fn append_dialogue_glyph_entries(
    atlas: &DialogueGlyphAtlas,
    y_offset: u32,
    entries: &mut Vec<VariantAtlasEntry>,
) -> Result<(), String> {
    for tile in &atlas.tiles {
        entries.push(VariantAtlasEntry {
            id: format!(
                "dialogue_glyph:kDialogueGlyphTiles:pack{}:tile{}:2bpp",
                tile.source_pack, tile.source_tile
            ),
            key: VariantAtlasKey {
                source_kind: "dialogue_glyph".to_string(),
                asset: "kDialogueGlyphTiles".to_string(),
                pack: tile.source_pack,
                tile: tile.source_tile,
                bpp: 2,
                palette: "palette_bg3_text_main".to_string(),
                palette_row: 0,
            },
            rect: [
                tile.rect[0],
                tile.rect[1].checked_add(y_offset).ok_or_else(|| {
                    format!("dialogue glyph rect y offset overflow for {}", tile.id)
                })?,
                tile.rect[2],
                tile.rect[3],
            ],
            sha1: tile.id.clone(),
            duplicate_of: None,
            dynamic_policy: "stable".to_string(),
            runtime_material: None,
            runtime_colors_per_row: None,
            source_hflip: false,
            source_vflip: false,
        });
    }
    Ok(())
}

fn append_dialogue_vwf_glyph_entries(
    atlas: &DialogueVwfGlyphAtlas,
    y_offset: u32,
    entries: &mut Vec<VariantAtlasEntry>,
) -> Result<(), String> {
    for glyph in &atlas.glyphs {
        for quadrant in 0..4u16 {
            let qx = u32::from(quadrant & 1) * 8;
            let qy = u32::from(quadrant >> 1) * 8;
            entries.push(VariantAtlasEntry {
                id: format!(
                    "dialogue_vwf:kDialogueVwfGlyphs:pack{}:tile{}:2bpp:{}",
                    glyph.code, quadrant, glyph.palette
                ),
                key: VariantAtlasKey {
                    source_kind: "dialogue_vwf".to_string(),
                    asset: "kDialogueVwfGlyphs".to_string(),
                    pack: u16::from(glyph.code),
                    tile: quadrant,
                    bpp: 2,
                    palette: glyph.palette.clone(),
                    palette_row: 0,
                },
                rect: [
                    glyph.rect[0].checked_add(qx).ok_or_else(|| {
                        format!("dialogue VWF rect x offset overflow for {}", glyph.hex)
                    })?,
                    glyph.rect[1]
                        .checked_add(y_offset)
                        .and_then(|y| y.checked_add(qy))
                        .ok_or_else(|| {
                            format!("dialogue VWF rect y offset overflow for {}", glyph.hex)
                        })?,
                    8,
                    8,
                ],
                sha1: format!("{}:{}:{quadrant}", glyph.hex, glyph.palette),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: None,
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            });
        }
    }
    Ok(())
}

fn append_dialogue_font_tile_entries(
    atlas: &DialogueFontTileAtlas,
    y_offset: u32,
    entries: &mut Vec<VariantAtlasEntry>,
) -> Result<(), String> {
    for tile in &atlas.tiles {
        entries.push(VariantAtlasEntry {
            id: format!(
                "dialogue_font:kDialogueFontTiles:pack{}:tile{}:2bpp",
                tile.source_pack, tile.source_tile
            ),
            key: VariantAtlasKey {
                source_kind: "dialogue_font".to_string(),
                asset: "kDialogueFontTiles".to_string(),
                pack: tile.source_pack,
                tile: tile.source_tile,
                bpp: 2,
                palette: "palette_bg3_text_main".to_string(),
                palette_row: 0,
            },
            rect: [
                tile.rect[0],
                tile.rect[1].checked_add(y_offset).ok_or_else(|| {
                    format!("dialogue font rect y offset overflow for {}", tile.id)
                })?,
                tile.rect[2],
                tile.rect[3],
            ],
            sha1: tile.id.clone(),
            duplicate_of: None,
            dynamic_policy: "stable".to_string(),
            runtime_material: None,
            runtime_colors_per_row: None,
            source_hflip: false,
            source_vflip: false,
        });
    }
    Ok(())
}

fn transform_art_indices(indices: &[u8; 64], hflip: bool, vflip: bool) -> [u8; 64] {
    let mut out = [0u8; 64];
    for y in 0..8 {
        for x in 0..8 {
            let source_x = if hflip { 7 - x } else { x };
            let source_y = if vflip { 7 - y } else { y };
            out[y * 8 + x] = indices[source_y * 8 + source_x];
        }
    }
    out
}

fn dynamic_policy_for_source_ref(effects: &[TileEffect], source_ref: &ArtSourceRefJson) -> String {
    if source_ref.source_kind == "bg3_dynamic" {
        return "requires_live_palette".to_string();
    }
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
    #[serde(default)]
    indices_hex: Option<String>,
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
struct DialogueGlyphManifestJson {
    format: String,
    width: u32,
    height: u32,
    tile_count: u32,
    tiles: Vec<DialogueGlyphTile>,
}

#[derive(Deserialize)]
struct DialogueVwfFontJson {
    format: String,
    glyph_count: u32,
    width_table_size: u32,
    glyphs: Vec<DialogueVwfGlyphJson>,
}

#[derive(Deserialize)]
struct DialogueVwfGlyphJson {
    code: u8,
    hex: String,
    width: u8,
    top_row_offset: u16,
    bottom_row_offset: u16,
    top_row_bytes: String,
    bottom_row_bytes: String,
}

#[derive(Deserialize)]
struct DialogueVwfGlyphAtlasJson {
    format: String,
    width: u32,
    height: u32,
    cell_width: u32,
    cell_height: u32,
    glyph_count: u32,
    glyphs: Vec<DialogueVwfGlyphCellJson>,
}

#[derive(Deserialize)]
struct DialogueVwfGlyphCellJson {
    code: u8,
    hex: String,
    width: u8,
    #[serde(default = "default_dialogue_vwf_palette")]
    palette: String,
    rect: [u32; 4],
    indices_hex: String,
}

fn default_dialogue_vwf_palette() -> String {
    DIALOGUE_TEXT_MAIN_PALETTE.to_string()
}

#[derive(Deserialize)]
struct DialogueFontTileManifestJson {
    format: String,
    width: u32,
    height: u32,
    tile_count: u32,
    tiles: Vec<DialogueFontTile>,
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

    fn dialogue_vwf_test_entry(code: u16, quadrant: u16, palette: &str) -> VariantAtlasEntry {
        VariantAtlasEntry {
            id: format!("dialogue_vwf:kDialogueVwfGlyphs:pack{code}:tile{quadrant}:2bpp"),
            key: dialogue_vwf_variant_key(code, quadrant, palette),
            rect: if palette == DIALOGUE_TEXT_MAIN_PALETTE {
                [0, 0, 8, 8]
            } else {
                [8, 0, 8, 8]
            },
            sha1: palette.to_string(),
            duplicate_of: None,
            dynamic_policy: "stable".to_string(),
            runtime_material: None,
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
        assert_eq!(content_key.source_kind, "bg3_dynamic");
        assert_eq!(content_key.asset, "kBg3TextGfx");
        assert_eq!(content_key.pack, 0x1234);
        assert_eq!(content_key.tile, 0x5678);
        assert_eq!(content_key.bpp, 5);
    }

    #[test]
    fn dialogue_source_keys_map_to_fixed_text_palette_variant_keys() {
        let glyph_key = variant_key_for_source_key(
            crate::modern_source_atlas::modern_source_key(9, 103, 128),
            "palette_overworld_bg_main",
            3,
        )
        .expect("dialogue glyph source key should resolve");
        assert_eq!(glyph_key.source_kind, "dialogue_glyph");
        assert_eq!(glyph_key.asset, "kDialogueGlyphTiles");
        assert_eq!(glyph_key.pack, 103);
        assert_eq!(glyph_key.tile, 128);
        assert_eq!(glyph_key.bpp, 2);
        assert_eq!(glyph_key.palette, "palette_bg3_text_main");
        assert_eq!(glyph_key.palette_row, 0);

        let vwf_key = variant_key_for_source_key(
            crate::modern_source_atlas::modern_source_key(10, 0x41, 2),
            "palette_dung_bg_main",
            7,
        )
        .expect("dialogue VWF source key should resolve");
        assert_eq!(vwf_key.source_kind, "dialogue_vwf");
        assert_eq!(vwf_key.asset, "kDialogueVwfGlyphs");
        assert_eq!(vwf_key.pack, 0x41);
        assert_eq!(vwf_key.tile, 2);
        assert_eq!(vwf_key.bpp, 2);
        assert_eq!(vwf_key.palette, "palette_bg3_text_main");
        assert_eq!(vwf_key.palette_row, 0);
    }

    #[test]
    fn dialogue_vwf_entry_prefers_semantic_color_palette_when_available() {
        let color_palette = dialogue_text_color_palette_name(7);
        let atlas = ModernVariantAtlas {
            width: 16,
            height: 8,
            rgba: vec![0; 16 * 8 * 4],
            entries: vec![
                dialogue_vwf_test_entry(0x41, 0, DIALOGUE_TEXT_MAIN_PALETTE),
                dialogue_vwf_test_entry(0x41, 0, &color_palette),
            ],
            effects: Vec::new(),
            mode7_source_chars: None,
            dialogue_glyph_atlas: None,
            dialogue_vwf_font: None,
            dialogue_vwf_glyph_atlas: None,
        };

        let entry = dialogue_vwf_variant_entry(&atlas, 0x41, 0, Some(7))
            .expect("colored VWF entry should resolve");

        assert_eq!(entry.key.palette, "palette_bg3_text_color_07");
        assert_eq!(entry.rect, [8, 0, 8, 8]);
    }

    #[test]
    fn dialogue_vwf_entry_falls_back_to_main_palette_when_color_variant_is_missing() {
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0; 8 * 8 * 4],
            entries: vec![dialogue_vwf_test_entry(0x41, 0, DIALOGUE_TEXT_MAIN_PALETTE)],
            effects: Vec::new(),
            mode7_source_chars: None,
            dialogue_glyph_atlas: None,
            dialogue_vwf_font: None,
            dialogue_vwf_glyph_atlas: None,
        };

        let entry = dialogue_vwf_variant_entry(&atlas, 0x41, 0, Some(7))
            .expect("main VWF entry should resolve as fallback");

        assert_eq!(entry.key.palette, DIALOGUE_TEXT_MAIN_PALETTE);
        assert_eq!(entry.rect, [0, 0, 8, 8]);
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
            mode7_source_chars: None,
            dialogue_glyph_atlas: None,
            dialogue_vwf_font: None,
            dialogue_vwf_glyph_atlas: None,
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
    fn modern_canonical_art_atlas_loads_dynamic_bg3_source_refs_without_runtime_png() {
        let root = unique_temp_root();
        let atlas_dir = root.join("atlas");
        std::fs::create_dir_all(&atlas_dir).expect("create atlas dir");
        let main_rgba = solid_rgba(8, 8, [1, 2, 3, 255]);
        write_rgba_png(&atlas_dir.join("art_tiles.png"), 8, 8, &main_rgba);
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
                "art_id": "art:main",
                "bpp": 3,
                "rect": [0, 0, 8, 8],
                "sha1_indices": "main",
                "preview_palette": "palette_dung_bg_main",
                "preview_palette_row": 0,
                "preview_source": "source_kind_default",
                "source_refs": [{
                  "source_kind": "bg",
                  "asset": "kBgGfx",
                  "pack": 1,
                  "tile": 2,
                  "bpp": 3,
                  "hflip": false,
                  "vflip": false,
                  "preview_palette": "palette_dung_bg_main",
                  "preview_palette_row": 0,
                  "preview_source": "source_kind_default"
                }]
              }]
            }"#,
        )
        .expect("write main manifest");
        std::fs::write(
            atlas_dir.join("dynamic_bg3_tiles.json"),
            r#"{
              "format": "zelda3_dynamic_bg3_art_atlas_v1",
              "tile_width": 8,
              "tile_height": 8,
              "width": 8,
              "height": 8,
              "art_count": 1,
              "source_ref_count": 1,
              "arts": [{
                "art_id": "art:dynamic",
                "bpp": 5,
                "rect": [0, 0, 8, 8],
                "sha1_indices": "dynamic",
                "preview_palette": "palette_overworld_bg_main",
                "preview_palette_row": 0,
                "preview_source": "source_kind_default",
                "source_refs": [{
                  "source_kind": "bg3_dynamic",
                  "asset": "kBg3TextGfx",
                  "pack": 4660,
                  "tile": 22136,
                  "bpp": 5,
                  "hflip": false,
                  "vflip": false,
                  "preview_palette": "palette_overworld_bg_main",
                  "preview_palette_row": 0,
                  "preview_source": "source_kind_default",
                  "runtime_material": "palette_lut",
                  "runtime_material_policy": "stable",
                  "runtime_colors_per_row": 32
                }]
              }]
            }"#,
        )
        .expect("write dynamic manifest");

        let atlas = load_modern_canonical_art_atlas(&root).expect("load art atlas");

        assert_eq!(atlas.width, 8);
        assert_eq!(atlas.height, 8);
        assert_eq!(atlas.rgba, main_rgba);
        assert_eq!(atlas.entries.len(), 2);
        assert_eq!(atlas.entries[1].key.source_kind, "bg3_dynamic");
        assert_eq!(atlas.entries[1].key.asset, "kBg3TextGfx");
        assert_eq!(atlas.entries[1].key.pack, 0x1234);
        assert_eq!(atlas.entries[1].key.tile, 0x5678);
        assert_eq!(atlas.entries[1].rect, [0, 0, 8, 8]);
        assert_eq!(atlas.entries[1].dynamic_policy, "requires_live_palette");

        std::fs::remove_dir_all(root).expect("remove temp root");
    }

    #[test]
    fn modern_canonical_art_atlas_loads_dialogue_source_assets() {
        let root = unique_temp_root();
        let atlas_dir = root.join("atlas");
        std::fs::create_dir_all(&atlas_dir).expect("create atlas dir");
        let art_rgba = solid_rgba(8, 8, [1, 2, 3, 255]);
        let glyph_rgba = solid_rgba(8, 8, [9, 8, 7, 255]);
        let vwf_glyph_rgba = solid_rgba(16, 16, [240, 240, 240, 255]);
        write_rgba_png(&atlas_dir.join("art_tiles.png"), 8, 8, &art_rgba);
        write_rgba_png(
            &atlas_dir.join("dialogue_glyph_tiles.png"),
            8,
            8,
            &glyph_rgba,
        );
        write_rgba_png(
            &atlas_dir.join("dialogue_vwf_glyphs.png"),
            16,
            32,
            &[vwf_glyph_rgba.clone(), vwf_glyph_rgba.clone()].concat(),
        );
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
                "art_id": "art:main",
                "bpp": 3,
                "rect": [0, 0, 8, 8],
                "sha1_indices": "main",
                "preview_palette": "palette_dung_bg_main",
                "preview_palette_row": 0,
                "preview_source": "source_kind_default",
                "source_refs": [{
                  "source_kind": "bg",
                  "asset": "kBgGfx",
                  "pack": 1,
                  "tile": 2,
                  "bpp": 3,
                  "hflip": false,
                  "vflip": false,
                  "preview_palette": "palette_dung_bg_main",
                  "preview_palette_row": 0,
                  "preview_source": "source_kind_default"
                }]
              }]
            }"#,
        )
        .expect("write art manifest");
        std::fs::write(
            atlas_dir.join("dialogue_glyph_tiles.json"),
            r#"{
              "format": "zelda3_dialogue_glyph_source_atlas_v1",
              "width": 8,
              "height": 8,
              "tile_count": 1,
              "tiles": [{
                "id": "1w-2d.DAT3N:tile0",
                "rect": [0, 0, 8, 8],
                "relative_tile": 0,
                "source_block": "1w-2d.DAT3N",
                "source_bpp": 2,
                "source_kind": "sprite",
                "source_pack": 103,
                "source_sheet": "1w-2d",
                "source_tile": 128
              }]
            }"#,
        )
        .expect("write glyph manifest");
        std::fs::write(
            atlas_dir.join("dialogue_vwf_font.json"),
            r#"{
              "format": "zelda3_dialogue_vwf_font_v1",
              "glyph_count": 1,
              "width_table_size": 1,
              "glyphs": [{
                "code": 0,
                "hex": "0x00",
                "width": 5,
                "top_row_offset": 0,
                "bottom_row_offset": 256,
                "top_row_bytes": "000102030405060708090a0b0c0d0e0f",
                "bottom_row_bytes": "101112131415161718191a1b1c1d1e1f"
              }]
            }"#,
        )
        .expect("write vwf manifest");
        let vwf_indices_hex = "01".repeat(16 * 16);
        std::fs::write(
            atlas_dir.join("dialogue_vwf_glyphs.json"),
            format!(
                r#"{{
              "format": "zelda3_dialogue_vwf_glyph_atlas_v1",
              "width": 16,
              "height": 32,
              "cell_width": 16,
              "cell_height": 16,
              "glyph_count": 2,
              "source_glyph_count": 1,
              "width_table_size": 1,
              "palette_variants": ["palette_bg3_text_main", "palette_bg3_text_color_07"],
              "glyphs": [{{
                "code": 0,
                "hex": "0x00",
                "width": 5,
                "palette": "palette_bg3_text_main",
                "rect": [0, 0, 16, 16],
                "indices_hex": "{vwf_indices_hex}"
              }}, {{
                "code": 0,
                "hex": "0x00",
                "width": 5,
                "palette": "palette_bg3_text_color_07",
                "rect": [0, 16, 16, 16],
                "indices_hex": "{vwf_indices_hex}"
              }}]
            }}"#
            ),
        )
        .expect("write VWF glyph atlas manifest");

        let atlas = load_modern_canonical_art_atlas(&root).expect("load art atlas");
        let glyph_atlas = atlas
            .dialogue_glyph_atlas
            .as_ref()
            .expect("dialogue glyph source atlas should load");
        let vwf_font = atlas
            .dialogue_vwf_font
            .as_ref()
            .expect("dialogue VWF metadata should load");
        let vwf_glyph_atlas = atlas
            .dialogue_vwf_glyph_atlas
            .as_ref()
            .expect("dialogue VWF glyph source atlas should load");

        assert_eq!(glyph_atlas.width, 8);
        assert_eq!(glyph_atlas.height, 8);
        assert_eq!(glyph_atlas.rgba, glyph_rgba);
        assert_eq!(glyph_atlas.tiles.len(), 1);
        assert_eq!(glyph_atlas.tiles[0].source_sheet, "1w-2d");
        assert_eq!(glyph_atlas.tiles[0].source_block, "1w-2d.DAT3N");
        assert_eq!(glyph_atlas.tiles[0].source_tile, 128);
        assert_eq!(vwf_font.glyph_count, 1);
        assert_eq!(vwf_font.width_table_size, 1);
        assert_eq!(vwf_font.glyphs[0].width, 5);
        assert_eq!(vwf_font.glyphs[0].top_row_bytes[15], 0x0f);
        assert_eq!(vwf_font.glyphs[0].bottom_row_bytes[0], 0x10);
        assert_eq!(vwf_glyph_atlas.width, 16);
        assert_eq!(vwf_glyph_atlas.height, 32);
        assert_eq!(
            vwf_glyph_atlas.rgba,
            [vwf_glyph_rgba.clone(), vwf_glyph_rgba.clone()].concat()
        );
        assert_eq!(vwf_glyph_atlas.glyphs.len(), 2);
        assert_eq!(vwf_glyph_atlas.glyphs[0].code, 0);
        assert_eq!(vwf_glyph_atlas.glyphs[0].width, 5);
        assert_eq!(
            vwf_glyph_atlas.glyphs[0].palette,
            DIALOGUE_TEXT_MAIN_PALETTE
        );
        assert_eq!(vwf_glyph_atlas.glyphs[0].rect, [0, 0, 16, 16]);
        assert_eq!(
            vwf_glyph_atlas.glyphs[1].palette,
            "palette_bg3_text_color_07"
        );
        assert_eq!(vwf_glyph_atlas.glyphs[1].rect, [0, 16, 16, 16]);
        assert_eq!(vwf_glyph_atlas.glyphs[0].indices.len(), 16 * 16);
        assert_eq!(vwf_glyph_atlas.glyphs[0].indices[0], 1);
        assert_eq!(atlas.width, 16);
        assert_eq!(atlas.height, 48);
        assert_eq!(
            atlas
                .entry_for_source_key(&VariantAtlasKey {
                    source_kind: "dialogue_glyph".to_string(),
                    asset: "kDialogueGlyphTiles".to_string(),
                    pack: 103,
                    tile: 128,
                    bpp: 2,
                    palette: "palette_bg3_text_main".to_string(),
                    palette_row: 0,
                })
                .expect("dialogue source glyph tile should be in main atlas")
                .rect,
            [0, 8, 8, 8]
        );
        assert_eq!(
            atlas
                .entry_for_key(&dialogue_vwf_variant_key(0, 3, DIALOGUE_TEXT_MAIN_PALETTE))
                .expect("dialogue VWF glyph quadrant should be in main atlas")
                .rect,
            [8, 24, 8, 8]
        );
        assert_eq!(
            atlas
                .entry_for_key(&dialogue_vwf_variant_key(0, 3, "palette_bg3_text_color_07"))
                .expect("colored dialogue VWF glyph quadrant should be in main atlas")
                .rect,
            [8, 40, 8, 8]
        );

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
                "indices_hex": "2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a",
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
        let chars = atlas
            .mode7_source_chars()
            .expect("Mode 7 art indices should populate source chars");
        assert_eq!(chars.len(), 0x4000);
        assert!(chars[42 * 64..42 * 64 + 64]
            .iter()
            .all(|&index| index == 0x2a));

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
            mode7_source_chars: None,
            dialogue_glyph_atlas: None,
            dialogue_vwf_font: None,
            dialogue_vwf_glyph_atlas: None,
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
            mode7_source_chars: None,
            dialogue_glyph_atlas: None,
            dialogue_vwf_font: None,
            dialogue_vwf_glyph_atlas: None,
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
            mode7_source_chars: None,
            dialogue_glyph_atlas: None,
            dialogue_vwf_font: None,
            dialogue_vwf_glyph_atlas: None,
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
            mode7_source_chars: None,
            dialogue_glyph_atlas: None,
            dialogue_vwf_font: None,
            dialogue_vwf_glyph_atlas: None,
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
            mode7_source_chars: None,
            dialogue_glyph_atlas: None,
            dialogue_vwf_font: None,
            dialogue_vwf_glyph_atlas: None,
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
            mode7_source_chars: None,
            dialogue_glyph_atlas: None,
            dialogue_vwf_font: None,
            dialogue_vwf_glyph_atlas: None,
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
