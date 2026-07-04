use crate::modern_frame::{ModernFrame, ModernIndexSpriteInstance, ModernIndexTileInstance};
use crate::modern_index_atlas::ModernIndexTile;
use crate::modern_software::VariantAtlasRenderStats;
use crate::modern_variant_atlas::{
    variant_key_for_index_tile, variant_key_for_source_key, ModernVariantAtlas, VariantAtlasDraw,
    VariantAtlasKey,
};

#[derive(Clone, Debug)]
pub struct VariantDrawPlan<'a> {
    pub bg: Vec<VariantBgDrawPacket<'a>>,
    pub sprites: Vec<VariantSpriteDrawPacket<'a>>,
    pub stats: VariantAtlasRenderStats,
}

#[derive(Clone, Debug)]
pub struct VariantBgDrawPacket<'a> {
    pub layer_index: usize,
    pub cell: &'a ModernIndexTile,
    pub inst: &'a ModernIndexTileInstance,
    pub key: Option<VariantAtlasKey>,
    pub draw: VariantAtlasDraw<'a>,
}

#[derive(Clone, Debug)]
pub struct VariantSpriteDrawPacket<'a> {
    pub cell: &'a ModernIndexTile,
    pub inst: &'a ModernIndexSpriteInstance,
    pub key: Option<VariantAtlasKey>,
    pub draw: VariantAtlasDraw<'a>,
}

pub fn compile_variant_draws<'a>(
    frame: &'a ModernFrame,
    bg_cells: &'a [ModernIndexTile],
    sprite_cells: &'a [ModernIndexTile],
    atlas: &'a ModernVariantAtlas,
    bg_palette_name: &str,
    sprite_palette_name: &str,
) -> VariantDrawPlan<'a> {
    let mut plan = VariantDrawPlan {
        bg: Vec::new(),
        sprites: Vec::new(),
        stats: VariantAtlasRenderStats::default(),
    };

    if frame.forced_blank {
        return plan;
    }

    for (layer_index, layer) in frame.bg_layers.iter().enumerate() {
        if !layer.enabled_main {
            continue;
        }
        for inst in &layer.index_tiles {
            let Some(cell) = bg_cells.get(inst.cell_id as usize) else {
                continue;
            };
            let source_key = if inst.source_key != crate::modern_hd_overrides::NO_SOURCE_KEY {
                inst.source_key
            } else {
                cell.source_key
            };
            let key = variant_key_for_source_key(source_key, bg_palette_name, inst.palette);
            let draw = atlas.resolve_draw(key.as_ref());
            plan.stats.record_bg_draw(&draw);
            plan.bg.push(VariantBgDrawPacket {
                layer_index,
                cell,
                inst,
                key,
                draw,
            });
        }
    }

    for inst in frame.index_sprites.iter().rev() {
        let Some(cell) = sprite_cells.get(inst.cell_id as usize) else {
            continue;
        };
        let key = variant_key_for_index_tile(cell, sprite_palette_name, inst.palette);
        let draw = atlas.resolve_draw(key.as_ref());
        plan.stats.record_sprite_draw(&draw);
        plan.sprites.push(VariantSpriteDrawPacket {
            cell,
            inst,
            key,
            draw,
        });
    }

    plan
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_frame::{
        ModernBgLayer, ModernFrame, ModernIndexSpriteInstance, ModernIndexTileInstance,
    };
    use crate::modern_hd_overrides::NO_SOURCE_KEY;
    use crate::modern_index_atlas::ModernIndexTile;
    use crate::modern_source_atlas::modern_source_key;
    use crate::modern_variant_atlas::{
        ModernVariantAtlas, TileEffect, VariantAtlasDraw, VariantAtlasEntry, VariantAtlasKey,
    };

    fn bg_key(pack: u16, tile: u16, palette_row: u8) -> VariantAtlasKey {
        VariantAtlasKey {
            source_kind: "bg".to_string(),
            asset: "kBgGfx".to_string(),
            pack,
            tile,
            bpp: 3,
            palette: "palette_dung_bg_main".to_string(),
            palette_row,
        }
    }

    fn bg_entry(pack: u16, tile: u16, palette_row: u8) -> VariantAtlasEntry {
        VariantAtlasEntry {
            id: format!("bg:kBgGfx:pack{pack}:tile{tile}:3bpp"),
            key: bg_key(pack, tile, palette_row),
            rect: [0, 0, 8, 8],
            sha1: "test".to_string(),
            duplicate_of: None,
            dynamic_policy: "stable".to_string(),
            source_hflip: false,
            source_vflip: false,
        }
    }

    fn effect(palette_row: u8) -> TileEffect {
        TileEffect {
            id: format!("palette_dung_bg_main:8color:row{palette_row}"),
            palette: "palette_dung_bg_main".to_string(),
            palette_row,
            colors_per_row: 8,
            index_to_rgba: vec![[0, 0, 0, 0xff]; 8],
            dynamic_policy: "stable".to_string(),
        }
    }

    fn index_cell(id: u32, source_key: u64) -> ModernIndexTile {
        ModernIndexTile {
            id,
            indices: [0u8; 64],
            source_key,
            hflip: false,
            vflip: false,
        }
    }

    #[test]
    fn compiles_bg_and_sprite_packets_in_render_order_with_stats() {
        let mut frame = ModernFrame::empty();
        let mut bg0 = ModernBgLayer::new(0);
        bg0.enabled_main = true;
        bg0.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 4,
            screen_y: 8,
            palette: 2,
            hflip: true,
            vflip: false,
            priority: false,
        });
        bg0.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 12,
            screen_y: 8,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = bg0;
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 20,
            screen_y: 24,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: true,
            row_mask: 0xff,
        });
        let bg_cells = vec![
            index_cell(0, modern_source_key(1, 0, 0)),
            index_cell(1, modern_source_key(1, 9, 9)),
        ];
        let sprite_cells = vec![index_cell(0, NO_SOURCE_KEY)];
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![bg_entry(0, 0, 0)],
            effects: vec![effect(2)],
        };

        let plan = compile_variant_draws(
            &frame,
            &bg_cells,
            &sprite_cells,
            &atlas,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(plan.bg.len(), 2);
        assert_eq!(plan.bg[0].cell.id, 0);
        assert_eq!(plan.bg[0].inst.screen_x, 4);
        assert!(matches!(
            plan.bg[0].draw,
            VariantAtlasDraw::Stable {
                effect: Some(_),
                ..
            }
        ));
        assert_eq!(plan.bg[1].cell.id, 1);
        assert!(matches!(plan.bg[1].draw, VariantAtlasDraw::MissingArt));
        assert_eq!(plan.sprites.len(), 1);
        assert_eq!(plan.sprites[0].inst.screen_y, 24);
        assert!(matches!(plan.sprites[0].draw, VariantAtlasDraw::Unkeyed));
        assert_eq!(plan.stats.stable_effect_draws, 1);
        assert_eq!(plan.stats.missing_art_draws, 1);
        assert_eq!(plan.stats.unkeyed_fallback_draws, 1);
        assert_eq!(plan.stats.unkeyed_bg_fallback_draws, 0);
        assert_eq!(plan.stats.unkeyed_sprite_fallback_draws, 1);
    }

    #[test]
    fn bg_instance_source_key_resolves_deduped_unkeyed_cell() {
        let mut frame = ModernFrame::empty();
        let mut bg0 = ModernBgLayer::new(0);
        bg0.enabled_main = true;
        bg0.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: modern_source_key(1, 3, 5),
            screen_x: 4,
            screen_y: 8,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = bg0;
        let bg_cells = vec![index_cell(0, NO_SOURCE_KEY)];
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![bg_entry(3, 5, 2)],
            effects: vec![effect(2)],
        };

        let plan = compile_variant_draws(
            &frame,
            &bg_cells,
            &[],
            &atlas,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(plan.bg.len(), 1);
        assert!(matches!(
            plan.bg[0].draw,
            VariantAtlasDraw::Stable {
                effect: Some(_),
                ..
            }
        ));
        assert_eq!(plan.stats.stable_effect_draws, 1);
        assert_eq!(plan.stats.unkeyed_bg_fallback_draws, 0);
    }

    #[test]
    fn skips_disabled_layers_and_out_of_range_cells() {
        let mut frame = ModernFrame::empty();
        let mut disabled = ModernBgLayer::new(0);
        disabled.enabled_main = false;
        disabled.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = disabled;
        let mut enabled = ModernBgLayer::new(1);
        enabled.enabled_main = true;
        enabled.index_tiles.push(ModernIndexTileInstance {
            cell_id: 99,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 8,
            screen_y: 8,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[1] = enabled;
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 7,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: Vec::new(),
            effects: Vec::new(),
        };

        let plan = compile_variant_draws(
            &frame,
            &[],
            &[],
            &atlas,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert!(plan.bg.is_empty());
        assert!(plan.sprites.is_empty());
        assert_eq!(plan.stats.fallback_draws, 0);
    }
}
