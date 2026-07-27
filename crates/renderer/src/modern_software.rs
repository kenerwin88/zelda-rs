use crate::modern_frame::{ModernFrame, MODERN_FRAME_HEIGHT, MODERN_FRAME_WIDTH};
use crate::modern_index_atlas::ModernIndexTile;

/// Composite the frame's palette-index OAM sprites onto an existing 256x224 RGBA
/// buffer (`out`, typically the BG render from
/// [`render_modern_frame_software_indexed`]).
///
/// Unlike BG cells, sprite cells in `sprite_cells` are stored UNFLIPPED, so this
/// applies each instance's `hflip`/`vflip` while sampling:
/// `src_x = if hflip {7-x}`, `src_y = if vflip {7-y}`,
/// `index = cell.indices[src_y*8 + src_x]`. Index 0 is transparent. The final
/// color is `frame.cgram_rgba[0x80 + palette*16 + index]` (OBJ CGRAM is 128..255).
///
/// Z-order matches the classic resolver's "first non-transparent pixel in OAM
/// order wins": sprites are drawn in REVERSE `index_sprites` order with REPLACE,
/// so the earliest OAM sprite is painted last and ends up on top. Sprites always
/// draw over the BG (sprite-vs-BG priority is out of scope here).
///
/// No-op when `frame.forced_blank` (the BG buffer is already solid black).
pub fn draw_modern_sprites_indexed(
    out: &mut [u8],
    frame: &ModernFrame,
    sprite_cells: &[ModernIndexTile],
) {
    if frame.forced_blank {
        return;
    }
    let width = usize::from(MODERN_FRAME_WIDTH);

    // Reverse OAM order → earliest OAM sprite drawn last (wins) under REPLACE.
    for inst in frame.index_sprites.iter().rev() {
        let cell = match sprite_cells.get(inst.cell_id as usize) {
            Some(c) => c,
            None => continue,
        };
        for y in 0..8usize {
            if inst.row_mask & (1 << y) == 0 {
                continue; // dropped by the per-scanline OBJ budget
            }
            for x in 0..8usize {
                // Sprite cells are UNFLIPPED → apply flip when sampling.
                let src_x = if inst.hflip { 7 - x } else { x };
                let src_y = if inst.vflip { 7 - y } else { y };
                let index = cell.indices[src_y * 8 + src_x];
                if index == 0 {
                    continue; // transparent
                }
                let dst_x = inst.screen_x + x as i16;
                let dst_y = inst.screen_y + y as i16;
                if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                    continue; // clip to screen
                }
                let color = frame.cgram_rgba[0x80 + inst.palette as usize * 16 + index as usize];
                let dst = (dst_y as usize * width + dst_x as usize) * 4;
                out[dst..dst + 4].copy_from_slice(&color);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VariantAtlasRenderStats {
    pub stable_draws: u32,
    pub effect_draws: u32,
    pub fallback_draws: u32,
    pub live_index_draws: u32,
    pub live_index_bg_draws: u32,
    pub live_index_bg12_draws: u32,
    pub live_index_bg3_draws: u32,
    pub live_index_sprite_draws: u32,
    pub gpu_prefinal_base_frames: u32,
    pub gpu_screen_builder_frames: u32,
    pub cpu_prefinal_composite_frames: u32,
    pub cpu_prefinal_overlay_frames: u32,
    pub dynamic_palette_draws: u32,
    pub missing_variant_draws: u32,
    pub stable_preview_draws: u32,
    pub stable_effect_draws: u32,
    pub dynamic_material_draws: u32,
    pub effect_material_draws: u32,
    pub dynamic_material_fallback_draws: u32,
    pub dynamic_material_fallback_instance_source_draws: u32,
    pub dynamic_material_fallback_brightness_draws: u32,
    pub dynamic_material_fallback_policy_draws: u32,
    pub dynamic_material_fallback_missing_effect_draws: u32,
    pub dynamic_material_fallback_unsupported_draws: u32,
    pub unsupported_material_draws: u32,
    pub missing_art_draws: u32,
    pub unkeyed_fallback_draws: u32,
    pub unkeyed_bg_fallback_draws: u32,
    pub unkeyed_bg12_fallback_draws: u32,
    pub unkeyed_bg3_fallback_draws: u32,
    pub unkeyed_sprite_fallback_draws: u32,
    pub mixed_overlay_bg_effect_draws: u32,
    pub mixed_overlay_bg_effect_candidates: u32,
    pub mixed_overlay_bg_effect_culled_invisible_main: u32,
    pub mixed_overlay_bg_effect_reject_complex_frame: u32,
    pub mixed_overlay_bg_effect_reject_complex_brightness: u32,
    pub mixed_overlay_bg_effect_reject_complex_invalid_layer: u32,
    pub mixed_overlay_bg_effect_reject_complex_mosaic: u32,
    pub mixed_overlay_bg_effect_reject_complex_sub_window: u32,
    pub mixed_overlay_bg_effect_reject_complex_effect_bounds: u32,
    pub mixed_overlay_bg_effect_reject_complex_scanline_main: u32,
    pub mixed_overlay_bg_effect_reject_complex_layer_window: u32,
    pub mixed_overlay_bg_effect_reject_complex_color_math: u32,
    pub mixed_overlay_bg_effect_reject_complex_color_math_clip: u32,
    pub mixed_overlay_bg_effect_reject_complex_color_math_subscreen: u32,
    pub mixed_overlay_bg_effect_reject_complex_color_math_fixed_color: u32,
    pub mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch: u32,
    pub mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap: u32,
    pub mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg: u32,
    pub mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj: u32,
    pub mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain: u32,
    pub mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front:
        u32,
    pub mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order:
        u32,
    pub mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect:
        u32,
    pub mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex:
        u32,
    pub mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch:
        u32,
    pub mixed_overlay_bg_effect_reject_cgram_mismatch: u32,
    pub mixed_overlay_bg_effect_reject_overlap: u32,
}

impl VariantAtlasRenderStats {
    pub fn needs_live_index_base(&self) -> bool {
        self.fallback_draws != 0 || self.live_index_draws != 0
    }

    pub fn needs_unresolved_live_index_base(&self) -> bool {
        self.missing_art_draws != 0 || self.unkeyed_fallback_draws != 0
    }

    pub fn live_gpu_material_draws(&self) -> u32 {
        self.effect_material_draws + self.dynamic_material_fallback_draws
    }

    pub fn record_draw(&mut self, draw: &crate::modern_variant_atlas::VariantAtlasDraw<'_>) {
        match draw {
            crate::modern_variant_atlas::VariantAtlasDraw::Stable { .. } => {
                self.stable_draws += 1;
                self.stable_preview_draws += 1;
            }
            crate::modern_variant_atlas::VariantAtlasDraw::MaterialEffect { .. } => {
                self.effect_draws += 1;
                self.dynamic_material_draws += 1;
                self.effect_material_draws += 1;
            }
            crate::modern_variant_atlas::VariantAtlasDraw::DynamicPalette { reason, .. } => {
                self.fallback_draws += 1;
                self.dynamic_palette_draws += 1;
                self.dynamic_material_draws += 1;
                self.dynamic_material_fallback_draws += 1;
                match reason {
                    crate::modern_variant_atlas::DynamicFallbackReason::InstanceSourceKey => {
                        self.dynamic_material_fallback_instance_source_draws += 1;
                    }
                    crate::modern_variant_atlas::DynamicFallbackReason::Brightness => {
                        self.dynamic_material_fallback_brightness_draws += 1;
                    }
                    crate::modern_variant_atlas::DynamicFallbackReason::EntryRequiresLivePalette => {
                        self.dynamic_material_fallback_policy_draws += 1;
                    }
                    crate::modern_variant_atlas::DynamicFallbackReason::UnsupportedMaterial => {
                        self.dynamic_material_fallback_unsupported_draws += 1;
                    }
                    crate::modern_variant_atlas::DynamicFallbackReason::MissingStableEffect => {
                        self.dynamic_material_fallback_missing_effect_draws += 1;
                    }
                }
                if draw.is_unsupported_material_fallback() {
                    self.unsupported_material_draws += 1;
                }
            }
            crate::modern_variant_atlas::VariantAtlasDraw::MissingArt => {
                self.fallback_draws += 1;
                self.missing_variant_draws += 1;
                self.missing_art_draws += 1;
            }
            crate::modern_variant_atlas::VariantAtlasDraw::Unkeyed => {
                self.live_index_draws += 1;
                self.unkeyed_fallback_draws += 1;
            }
        }
    }

    pub fn record_bg_draw(
        &mut self,
        layer_index: usize,
        draw: &crate::modern_variant_atlas::VariantAtlasDraw<'_>,
    ) {
        self.record_draw(draw);
        if matches!(draw, crate::modern_variant_atlas::VariantAtlasDraw::Unkeyed) {
            self.live_index_bg_draws += 1;
            self.unkeyed_bg_fallback_draws += 1;
            if layer_index == 2 {
                self.live_index_bg3_draws += 1;
                self.unkeyed_bg3_fallback_draws += 1;
            } else {
                self.live_index_bg12_draws += 1;
                self.unkeyed_bg12_fallback_draws += 1;
            }
        }
    }

    pub fn record_sprite_draw(&mut self, draw: &crate::modern_variant_atlas::VariantAtlasDraw<'_>) {
        self.record_draw(draw);
        if matches!(draw, crate::modern_variant_atlas::VariantAtlasDraw::Unkeyed) {
            self.live_index_sprite_draws += 1;
            self.unkeyed_sprite_fallback_draws += 1;
        }
    }
}

pub fn render_modern_frame_software_variant_atlas(
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
    atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
    bg_palette_name: &str,
    sprite_palette_name: &str,
) -> (Vec<u8>, VariantAtlasRenderStats) {
    let width = usize::from(MODERN_FRAME_WIDTH);
    let height = usize::from(MODERN_FRAME_HEIGHT);
    let mut out = vec![0u8; width * height * 4];
    for px in out.chunks_exact_mut(4) {
        px.copy_from_slice(&frame.backdrop_color_rgba);
    }

    if frame.forced_blank {
        for px in out.chunks_exact_mut(4) {
            px.copy_from_slice(&[0, 0, 0, 0xff]);
        }
        return (out, VariantAtlasRenderStats::default());
    }

    let plan = crate::modern_variant_draw::compile_variant_draws(
        frame,
        bg_cells,
        sprite_cells,
        atlas,
        bg_palette_name,
        sprite_palette_name,
    );
    for packet in &plan.bg {
        match packet.draw {
            crate::modern_variant_atlas::VariantAtlasDraw::Stable { entry } => {
                draw_variant_bg_instance(
                    &mut out,
                    frame,
                    atlas,
                    entry,
                    None,
                    packet.cell,
                    packet.inst,
                );
            }
            crate::modern_variant_atlas::VariantAtlasDraw::MaterialEffect { entry, effect } => {
                draw_variant_bg_instance(
                    &mut out,
                    frame,
                    atlas,
                    entry,
                    Some(effect),
                    packet.cell,
                    packet.inst,
                );
            }
            crate::modern_variant_atlas::VariantAtlasDraw::DynamicPalette { .. }
            | crate::modern_variant_atlas::VariantAtlasDraw::MissingArt
            | crate::modern_variant_atlas::VariantAtlasDraw::Unkeyed => {
                draw_indexed_bg_instance(&mut out, frame, packet.cell, packet.inst);
            }
        }
    }

    for packet in &plan.sprites {
        match packet.draw {
            crate::modern_variant_atlas::VariantAtlasDraw::Stable { entry } => {
                draw_variant_sprite_instance(
                    &mut out,
                    atlas,
                    entry,
                    None,
                    packet.cell,
                    packet.inst,
                );
            }
            crate::modern_variant_atlas::VariantAtlasDraw::MaterialEffect { entry, effect } => {
                draw_variant_sprite_instance(
                    &mut out,
                    atlas,
                    entry,
                    Some(effect),
                    packet.cell,
                    packet.inst,
                );
            }
            crate::modern_variant_atlas::VariantAtlasDraw::DynamicPalette { .. }
            | crate::modern_variant_atlas::VariantAtlasDraw::MissingArt
            | crate::modern_variant_atlas::VariantAtlasDraw::Unkeyed => {
                draw_indexed_sprite_instance(&mut out, frame, packet.cell, packet.inst);
            }
        }
    }
    draw_vwf_glyph_runs(&mut out, frame, atlas);

    (out, plan.stats)
}

fn draw_vwf_glyph_runs(
    out: &mut [u8],
    frame: &ModernFrame,
    atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
) {
    for run in frame.vwf_glyph_runs_for_draw() {
        let glyph_code = run.source_glyph_code();
        for quadrant in 0..4u16 {
            let Some(entry) = crate::modern_variant_atlas::dialogue_vwf_variant_entry(
                atlas,
                glyph_code,
                quadrant,
                run.dialogue_color,
            ) else {
                continue;
            };
            let qx = i16::from((quadrant & 1) as u8) * 8;
            let qy = i16::from((quadrant >> 1) as u8) * 8;
            draw_stable_variant_rect(out, atlas, entry, run.screen_x + qx, run.screen_y + qy);
        }
    }
}

fn draw_stable_variant_rect(
    out: &mut [u8],
    atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
    entry: &crate::modern_variant_atlas::VariantAtlasEntry,
    screen_x: i16,
    screen_y: i16,
) {
    let width = usize::from(MODERN_FRAME_WIDTH);
    for sy in 0..entry.rect[3] as usize {
        for sx in 0..entry.rect[2] as usize {
            let atlas_x = entry.rect[0] as usize + sx;
            let atlas_y = entry.rect[1] as usize + sy;
            if atlas_x >= atlas.width as usize || atlas_y >= atlas.height as usize {
                continue;
            }
            let src = (atlas_y * atlas.width as usize + atlas_x) * 4;
            if atlas.rgba[src + 3] == 0 {
                continue;
            }
            let dst_x = screen_x + sx as i16;
            let dst_y = screen_y + sy as i16;
            if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                continue;
            }
            let dst = (dst_y as usize * width + dst_x as usize) * 4;
            out[dst..dst + 4].copy_from_slice(&atlas.rgba[src..src + 4]);
        }
    }
}

fn draw_variant_bg_instance(
    out: &mut [u8],
    _frame: &ModernFrame,
    atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
    entry: &crate::modern_variant_atlas::VariantAtlasEntry,
    stable_effect: Option<&crate::modern_variant_atlas::TileEffect>,
    cell: &ModernIndexTile,
    inst: &crate::modern_frame::ModernIndexTileInstance,
) {
    let width = usize::from(MODERN_FRAME_WIDTH);
    for sy in 0..8usize {
        for sx in 0..8usize {
            let src_x = if cell.hflip ^ entry.source_hflip {
                7 - sx
            } else {
                sx
            };
            let src_y = if cell.vflip ^ entry.source_vflip {
                7 - sy
            } else {
                sy
            };
            let atlas_x = entry.rect[0] as usize + src_x;
            let atlas_y = entry.rect[1] as usize + src_y;
            if let Some(effect) = stable_effect {
                if let Some(color) = effect_color_for_index(effect, cell.indices[src_y * 8 + src_x])
                {
                    let dst_x = inst.screen_x + sx as i16;
                    let dst_y = inst.screen_y + sy as i16;
                    if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                        continue;
                    }
                    let dst = (dst_y as usize * width + dst_x as usize) * 4;
                    out[dst..dst + 4].copy_from_slice(&color);
                }
                continue;
            }
            if atlas_x >= atlas.width as usize || atlas_y >= atlas.height as usize {
                continue;
            }
            let src = (atlas_y * atlas.width as usize + atlas_x) * 4;
            if atlas.rgba[src + 3] == 0 {
                continue;
            }
            let dst_x = inst.screen_x + sx as i16;
            let dst_y = inst.screen_y + sy as i16;
            if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                continue;
            }
            let dst = (dst_y as usize * width + dst_x as usize) * 4;
            out[dst..dst + 4].copy_from_slice(&atlas.rgba[src..src + 4]);
        }
    }
}

fn draw_indexed_bg_instance(
    out: &mut [u8],
    frame: &ModernFrame,
    cell: &ModernIndexTile,
    inst: &crate::modern_frame::ModernIndexTileInstance,
) {
    let width = usize::from(MODERN_FRAME_WIDTH);
    for sy in 0..8usize {
        for sx in 0..8usize {
            let index = cell.indices[sy * 8 + sx];
            if index == 0 {
                continue;
            }
            let dst_x = inst.screen_x + sx as i16;
            let dst_y = inst.screen_y + sy as i16;
            if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                continue;
            }
            let color = frame.cgram_rgba[inst.palette as usize * 16 + index as usize];
            let dst = (dst_y as usize * width + dst_x as usize) * 4;
            out[dst..dst + 4].copy_from_slice(&color);
        }
    }
}

fn draw_variant_sprite_instance(
    out: &mut [u8],
    atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
    entry: &crate::modern_variant_atlas::VariantAtlasEntry,
    stable_effect: Option<&crate::modern_variant_atlas::TileEffect>,
    cell: &ModernIndexTile,
    inst: &crate::modern_frame::ModernIndexSpriteInstance,
) {
    let width = usize::from(MODERN_FRAME_WIDTH);
    for y in 0..8usize {
        if inst.row_mask & (1 << y) == 0 {
            continue;
        }
        for x in 0..8usize {
            let src_x = if inst.hflip ^ entry.source_hflip {
                7 - x
            } else {
                x
            };
            let src_y = if inst.vflip ^ entry.source_vflip {
                7 - y
            } else {
                y
            };
            let atlas_x = entry.rect[0] as usize + src_x;
            let atlas_y = entry.rect[1] as usize + src_y;
            if let Some(effect) = stable_effect {
                if let Some(color) = effect_color_for_index(effect, cell.indices[src_y * 8 + src_x])
                {
                    let dst_x = inst.screen_x + x as i16;
                    let dst_y = inst.screen_y + y as i16;
                    if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                        continue;
                    }
                    let dst = (dst_y as usize * width + dst_x as usize) * 4;
                    out[dst..dst + 4].copy_from_slice(&color);
                }
                continue;
            }
            if atlas_x >= atlas.width as usize || atlas_y >= atlas.height as usize {
                continue;
            }
            let src = (atlas_y * atlas.width as usize + atlas_x) * 4;
            if atlas.rgba[src + 3] == 0 {
                continue;
            }
            let dst_x = inst.screen_x + x as i16;
            let dst_y = inst.screen_y + y as i16;
            if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                continue;
            }
            let dst = (dst_y as usize * width + dst_x as usize) * 4;
            out[dst..dst + 4].copy_from_slice(&atlas.rgba[src..src + 4]);
        }
    }
}

fn effect_color_for_index(
    effect: &crate::modern_variant_atlas::TileEffect,
    index: u8,
) -> Option<[u8; 4]> {
    if index == 0 {
        return None;
    }
    effect.index_to_rgba.get(index as usize).copied()
}

fn draw_indexed_sprite_instance(
    out: &mut [u8],
    frame: &ModernFrame,
    cell: &ModernIndexTile,
    inst: &crate::modern_frame::ModernIndexSpriteInstance,
) {
    let width = usize::from(MODERN_FRAME_WIDTH);
    for y in 0..8usize {
        if inst.row_mask & (1 << y) == 0 {
            continue;
        }
        for x in 0..8usize {
            let src_x = if inst.hflip { 7 - x } else { x };
            let src_y = if inst.vflip { 7 - y } else { y };
            let index = cell.indices[src_y * 8 + src_x];
            if index == 0 {
                continue;
            }
            let dst_x = inst.screen_x + x as i16;
            let dst_y = inst.screen_y + y as i16;
            if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                continue;
            }
            let color = frame.cgram_rgba[0x80 + inst.palette as usize * 16 + index as usize];
            let dst = (dst_y as usize * width + dst_x as usize) * 4;
            out[dst..dst + 4].copy_from_slice(&color);
        }
    }
}

pub fn render_modern_frame_software(
    frame: &ModernFrame,
    atlas_rgba: &[u8],
    atlas_width: u16,
    atlas_height: u16,
) -> Vec<u8> {
    let mut out = vec![0u8; usize::from(MODERN_FRAME_WIDTH) * usize::from(MODERN_FRAME_HEIGHT) * 4];
    for px in out.chunks_exact_mut(4) {
        px.copy_from_slice(&frame.backdrop_color_rgba);
    }
    if frame.forced_blank {
        for px in out.chunks_exact_mut(4) {
            px.copy_from_slice(&[0, 0, 0, 0xff]);
        }
        return out;
    }
    for layer in &frame.bg_layers {
        if !layer.enabled_main {
            continue;
        }
        for tile in &layer.tiles {
            if tile.screen_width_px == 0 || tile.screen_height_px == 0 {
                continue; // degenerate footprint — nothing to draw
            }
            // Downsample factor from the (upscaled) atlas source rect to the
            // on-screen footprint. Nearest: sample the top-left of each block.
            let scale_x = tile.atlas_width_px / tile.screen_width_px;
            let scale_y = tile.atlas_height_px / tile.screen_height_px;
            for y in 0..tile.screen_height_px {
                for x in 0..tile.screen_width_px {
                    // Mirror on the SCREEN coordinate, then scale up into the source.
                    let src_x = if tile.hflip {
                        tile.screen_width_px - 1 - x
                    } else {
                        x
                    };
                    let src_y = if tile.vflip {
                        tile.screen_height_px - 1 - y
                    } else {
                        y
                    };
                    let atlas_x =
                        u32::from(tile.atlas_x_px) + u32::from(src_x) * u32::from(scale_x);
                    let atlas_y =
                        u32::from(tile.atlas_y_px) + u32::from(src_y) * u32::from(scale_y);
                    if atlas_x >= u32::from(atlas_width) || atlas_y >= u32::from(atlas_height) {
                        continue;
                    }
                    let dst_x = tile.screen_x + x as i16;
                    let dst_y = tile.screen_y + y as i16;
                    if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                        continue;
                    }
                    let src = (atlas_y as usize * usize::from(atlas_width) + atlas_x as usize) * 4;
                    let dst = (dst_y as usize * 256 + dst_x as usize) * 4;
                    if tile.transparent_color_zero && atlas_rgba[src + 3] == 0 {
                        continue;
                    }
                    out[dst..dst + 4].copy_from_slice(&atlas_rgba[src..src + 4]);
                }
            }
        }
    }
    out
}

/// Composite direct-color power-on residue after normal composition. This is
/// intentionally not a palette, sprite, or PPU path: it is the semantic
/// representation of Snes9x's visible startup hardware state.
fn apply_hardware_startup_transient(
    out: &mut [u8],
    frame: &ModernFrame,
    out_width: usize,
    scale: usize,
) {
    let Some(transient) = frame.hardware_startup_transient.as_ref() else {
        return;
    };
    for (origin_x, origin_y) in transient.origins {
        for (cell_index, rgba) in transient.rgba.iter().enumerate() {
            if rgba[3] == 0 {
                continue;
            }
            let x = isize::from(origin_x) + (cell_index % 8) as isize;
            let y = isize::from(origin_y) + (cell_index / 8) as isize;
            if x < 0 || y < 0 || x >= frame.width as isize || y >= frame.height as isize {
                continue;
            }
            for dy in 0..scale {
                for dx in 0..scale {
                    let offset =
                        ((y as usize * scale + dy) * out_width + x as usize * scale + dx) * 4;
                    out[offset..offset + 4].copy_from_slice(rgba);
                }
            }
        }
    }
    for pixel in &transient.direct_pixels {
        if pixel.rgba[3] == 0
            || pixel.screen_x < 0
            || pixel.screen_y < 0
            || pixel.screen_x >= frame.width as i16
            || pixel.screen_y >= frame.height as i16
        {
            continue;
        }
        for dy in 0..scale {
            for dx in 0..scale {
                let offset = ((pixel.screen_y as usize * scale + dy) * out_width
                    + pixel.screen_x as usize * scale
                    + dx)
                    * 4;
                out[offset..offset + 4].copy_from_slice(&pixel.rgba);
            }
        }
    }
}

/// Render a `ModernFrame` using the palette-index atlas + live CGRAM.
///
/// For each enabled BG layer, each `index_tiles` instance is drawn: for each
/// 8×8 pixel in the tile, `index = cell.indices[sy*8+sx]`; if `index == 0`
/// the pixel is transparent (skip); otherwise `color = frame.cgram_rgba[palette*16 + index]`.
///
/// Backdrop and forced_blank behaviour match `render_modern_frame_software`.
///
/// Note: `hflip`/`vflip` on the instance are intentionally ignored here.
/// The index pattern in the atlas already baked flip via `graphics_key` during
/// Task 2 atlas generation, so re-applying flip would double-mirror the pixels.
pub fn render_modern_frame_software_indexed(
    frame: &ModernFrame,
    cells: &[ModernIndexTile],
) -> Vec<u8> {
    let width = usize::from(MODERN_FRAME_WIDTH);
    let height = usize::from(MODERN_FRAME_HEIGHT);
    let mut out = vec![0u8; width * height * 4];

    // Fill backdrop.
    for px in out.chunks_exact_mut(4) {
        px.copy_from_slice(&frame.backdrop_color_rgba);
    }

    // Forced blank: solid black.
    if frame.forced_blank {
        for px in out.chunks_exact_mut(4) {
            px.copy_from_slice(&[0, 0, 0, 0xff]);
        }
        return out;
    }

    for layer in &frame.bg_layers {
        if !layer.enabled_main {
            continue;
        }
        for inst in &layer.index_tiles {
            // Cells are stored densely 0..len; guard against a bad id.
            let cell = match cells.get(inst.cell_id as usize) {
                Some(c) => c,
                None => continue,
            };
            for sy in 0..8usize {
                for sx in 0..8usize {
                    let index = cell.indices[sy * 8 + sx];
                    if index == 0 {
                        continue; // transparent
                    }
                    let dst_x = inst.screen_x + sx as i16;
                    let dst_y = inst.screen_y + sy as i16;
                    if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                        continue; // clip to screen
                    }
                    let color = frame.cgram_rgba[inst.palette as usize * 16 + index as usize];
                    let dst = (dst_y as usize * width + dst_x as usize) * 4;
                    out[dst..dst + 4].copy_from_slice(&color);
                }
            }
        }
    }

    out
}

/// One composited SNES screen (main or sub) as 5-bit-per-channel pixels plus the
/// per-pixel winning-layer math bit and whether a non-backdrop source drew there.
struct Screen {
    /// Per-pixel 5-bit channels (0..31).
    c5: Vec<[u8; 3]>,
    /// Winning layer's color-math bit: BG1=0..BG4=3, OBJ=4, BACKDROP=5.
    bit: Vec<u8>,
    /// True where a BG/sprite (not the backdrop) covered the pixel.
    real: Vec<bool>,
}

impl Screen {
    fn new(backdrop_c5: [u8; 3], len: usize) -> Self {
        Self {
            c5: vec![backdrop_c5; len],
            bit: vec![5u8; len],
            real: vec![false; len],
        }
    }
}

/// Packed MAIN/SUB compositing buffers for the GPU finalizer.
///
/// Each `u32` stores the 5-bit color channels, winning color-math layer bit, and
/// whether a real BG/OBJ pixel covered the backdrop:
/// bits 0..4 = R, 5..9 = G, 10..14 = B, 15..17 = math bit, 18 = real.
#[cfg(test)]
pub(crate) struct ModernCompositedScreens {
    pub(crate) width: usize,
    pub(crate) scale: usize,
    pub(crate) main: Vec<u32>,
}

/// Composite one screen (main or sub) in SNES Mode 1 z-order, back → front:
/// `BG3-lo, OBJ0, OBJ1, BG2-lo, BG1-lo, OBJ2, BG2-hi, BG1-hi, OBJ3, BG3-hi`.
/// `enabled` is the screen's layer-enable mask (bits 0-2 = BG1-3, bit 4 = OBJ).
/// Byte-for-byte the same order the GPU reference renderer uses, so the modern
/// software output matches the classic for high-priority BG tiles (e.g. HUD digits
/// over the play field) and OBJ-vs-BG interleaving.
#[allow(clippy::too_many_arguments)]
fn composite_mode1(
    screen: &mut Screen,
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
    enabled: u8,
    // Per-scanline main-screen TM (HDMA layer-enable). `Some` for the main screen:
    // a pixel on scanline y only shows if `tm[y]` has the layer bit (BG L = 1<<L,
    // OBJ = 0x10), matching the classic per-scanline TM check. `None` for the sub
    // screen (classic skips the per-scanline TM check there).
    main_tm: Option<&[u8]>,
    // This screen's window-enable mask (TMW/$212E for main, TSW/$212F for sub). Bit L
    // masks layer L's pixels where the layer's window region is active. The classic
    // window-masks BOTH screens (unlike the per-scanline TM check, which is main-only),
    // so this is passed for the sub composite too.
    windowed: u8,
    ctx: &crate::modern_hd_overrides::HdOverrideCtx,
    // Output-buffer width (`256 * scale`) and integer render `scale`. At `scale == 1`
    // this is the native 256-wide path, byte-identical to before; at `scale > 1` the
    // simple (non-mosaic, non-scanline-scroll) branch renders at N× — the mosaic and
    // scanline branches are only reached at `scale == 1` (the N× entry routes complex
    // frames to a native render + nearest-upscale), so they pass native dims onward.
    out_width: usize,
    scale: usize,
) {
    // SNES mosaic ($2106): active only when the block size is >1 AND at least one
    // ENABLED BG layer (1-3) has its mosaic bit set. When inactive, run the existing
    // (pixel-exact) path UNCHANGED so normal frames are byte-for-byte identical.
    let mosaic_active = frame.mosaic_size > 1 && (frame.mosaic_enabled & enabled & 0x07) != 0;
    if mosaic_active {
        // Native-only path (reached only at scale == 1).
        composite_mode1_mosaic(
            screen,
            frame,
            bg_cells,
            sprite_cells,
            enabled,
            main_tm,
            windowed,
            ctx,
        );
        return;
    }
    let bg_on = |i: usize| (enabled >> i) & 1 != 0;
    let obj_on = (enabled >> 4) & 1 != 0;
    let bg = &frame.bg_layers;
    // BG scroll wrapping: varying HDMA scroll must be re-sampled per output row,
    // and uniform non-zero scroll still needs torus sampling so edge-crossing tiles
    // wrap across the screen boundary. Zero-scroll layers stay on the fast path.
    let scroll_layers = [
        bg_on(0) && bg_layer_needs_torus_sampling(frame, 0),
        bg_on(1) && bg_layer_needs_torus_sampling(frame, 1),
        bg_on(2) && bg_layer_needs_torus_sampling(frame, 2),
    ];
    if scroll_layers.iter().any(|&v| v) {
        composite_mode1_scanline_scroll(
            screen,
            frame,
            bg_cells,
            sprite_cells,
            enabled,
            main_tm,
            windowed,
            scroll_layers,
            ctx,
        );
        return;
    }
    // Resolve OBJ-vs-OBJ FIRST, by OAM index (lowest index wins) — NOT by the
    // priority attribute. The SNES priority attribute (0-3) only sets where the
    // winning OBJ pixel sits relative to BG layers; it does NOT reorder sprites
    // against each other. Compositing in priority passes (as before) wrongly let a
    // higher-priority-attr but higher-index sprite overwrite a lower-index one.
    let obj = if obj_on {
        Some(resolve_obj_layer(
            frame,
            sprite_cells,
            screen.c5.len(),
            ctx,
            out_width,
            scale,
        ))
    } else {
        None
    };
    // BG3-lo
    if bg_on(2) {
        composite_index_tiles_c5(
            screen, &bg[2], bg_cells, frame, false, main_tm, windowed, ctx, out_width, scale,
        );
    }
    // OBJ priority 0, 1 (below BG2/BG1 low)
    if let Some(o) = &obj {
        paint_obj_priority(screen, o, 0, main_tm, frame, windowed, out_width, scale);
        paint_obj_priority(screen, o, 1, main_tm, frame, windowed, out_width, scale);
    }
    // BG2-lo, BG1-lo
    if bg_on(1) {
        composite_index_tiles_c5(
            screen, &bg[1], bg_cells, frame, false, main_tm, windowed, ctx, out_width, scale,
        );
    }
    if bg_on(0) {
        composite_index_tiles_c5(
            screen, &bg[0], bg_cells, frame, false, main_tm, windowed, ctx, out_width, scale,
        );
    }
    // OBJ priority 2 (above BG2/BG1 low)
    if let Some(o) = &obj {
        paint_obj_priority(screen, o, 2, main_tm, frame, windowed, out_width, scale);
    }
    // BG2-hi, BG1-hi
    if bg_on(1) {
        composite_index_tiles_c5(
            screen, &bg[1], bg_cells, frame, true, main_tm, windowed, ctx, out_width, scale,
        );
    }
    if bg_on(0) {
        composite_index_tiles_c5(
            screen, &bg[0], bg_cells, frame, true, main_tm, windowed, ctx, out_width, scale,
        );
    }
    // OBJ priority 3 (above BG2/BG1 high)
    if let Some(o) = &obj {
        paint_obj_priority(screen, o, 3, main_tm, frame, windowed, out_width, scale);
    }
    // BG3-hi
    if bg_on(2) {
        composite_index_tiles_c5(
            screen, &bg[2], bg_cells, frame, true, main_tm, windowed, ctx, out_width, scale,
        );
    }
}

/// One BG layer rendered into a full-screen buffer for the mosaic path: per-pixel
/// 5-bit color, whether a non-transparent tile pixel covered it (`real`), and the
/// tile's priority bit (`hipri`). Built WITHOUT the per-scanline TM gate (that is a
/// function of the OUTPUT scanline, applied after the mosaic source-snap), so the
/// snap reads pure tile color/opacity.
struct BgBuf {
    c5: Vec<[u8; 3]>,
    real: Vec<bool>,
    hipri: Vec<bool>,
}

/// Render every tile of one BG layer into a `BgBuf` (no TM gate, no priority
/// filtering — both priorities recorded with their per-pixel `hipri` bit). Mirrors
/// `composite_index_tiles_c5`'s per-pixel decode. Each visible screen pixel of a BG
/// layer is covered by at most one tile, so there is no within-layer overwrite.
fn render_bg_layer_buf(
    layer: &crate::modern_frame::ModernBgLayer,
    cells: &[ModernIndexTile],
    frame: &ModernFrame,
    len: usize,
    ctx: &crate::modern_hd_overrides::HdOverrideCtx,
) -> BgBuf {
    let width = usize::from(MODERN_FRAME_WIDTH);
    let mut buf = BgBuf {
        c5: vec![[0u8; 3]; len],
        real: vec![false; len],
        hipri: vec![false; len],
    };
    for inst in &layer.index_tiles {
        let cell = match cells.get(inst.cell_id as usize) {
            Some(c) => c,
            None => continue,
        };
        let ov = ctx.resolve(cell.source_key);
        for sy in 0..8usize {
            for sx in 0..8usize {
                let index = cell.indices[sy * 8 + sx];
                if index == 0 {
                    continue;
                }
                let dst_x = inst.screen_x + sx as i16;
                let dst_y = inst.screen_y + sy as i16;
                if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                    continue;
                }
                // un-flip HD sampling back to source orientation (base index is baked-flipped)
                let hx = if cell.hflip { 7 - sx } else { sx };
                let hy = if cell.vflip { 7 - sy } else { sy };
                let cgram_idx = inst.palette as usize * 16 + index as usize;
                let color = match crate::modern_hd_overrides::resolve_pixel_color(
                    index,
                    cgram_idx,
                    frame.cgram_rgba[cgram_idx],
                    ov,
                    ctx.reference(),
                    hx as u32,
                    hy as u32,
                    8,
                ) {
                    Some(c) => c,
                    None => continue,
                };
                let i = dst_y as usize * width + dst_x as usize;
                buf.c5[i] = [color[0] >> 3, color[1] >> 3, color[2] >> 3];
                buf.real[i] = true;
                buf.hipri[i] = inst.priority;
            }
        }
    }
    buf
}

/// Apply the SNES mosaic source-snap to a BG buffer, IN SCREEN SPACE.
///
/// The classic shader (`bg_layer.wgsl`) snaps the pre-scroll source coords
/// `source_x = sx`, `source_y = sy + 1` (the +1 is the SNES vertical fetch) to the
/// mosaic grid, THEN adds scroll. Because the modern per-layer buffer already maps
/// screen pixel `(sx, sy)` to BG sample `(sx + h_scroll, sy + v_scroll + 1)`, the
/// scroll and the +1 cancel out of the origin lookup, leaving a pure screen-space
/// snap to the N-grid: `origin = (sx - sx%N, sy - sy%N)`. (Derivation: classic
/// snapped source row = `(sy+1) - (sy % N)`; the origin screen row `osy` satisfying
/// `osy + 1 == (sy+1) - (sy%N)` is `osy = sy - sy%N`; likewise for x.) Origin is
/// always within `[0, sx] x [0, sy]`, so no bounds check is needed.
fn mosaic_snap_bg_buf(buf: &mut BgBuf, n: usize) {
    let width = usize::from(MODERN_FRAME_WIDTH);
    let height = usize::from(MODERN_FRAME_HEIGHT);
    let src_c5 = buf.c5.clone();
    let src_real = buf.real.clone();
    let src_hipri = buf.hipri.clone();
    for sy in 0..height {
        let osy = sy - sy % n;
        for sx in 0..width {
            let osx = sx - sx % n;
            let i = sy * width + sx;
            let j = osy * width + osx;
            buf.c5[i] = src_c5[j];
            buf.real[i] = src_real[j];
            buf.hipri[i] = src_hipri[j];
        }
    }
}

/// Paint one BG layer's (possibly mosaiced) buffer into the screen at the given
/// priority pass (REPLACE), applying the per-scanline main-screen TM gate at the
/// OUTPUT row (matching the classic, which TM-gates by output scanline regardless of
/// the mosaic source-snap). `lo` pixels = `real && !hipri`; `hi` = `real && hipri`.
fn paint_bg_buf(
    screen: &mut Screen,
    buf: &BgBuf,
    layer_index: u8,
    hi_priority: bool,
    main_tm: Option<&[u8]>,
    frame: &ModernFrame,
    windowed: u8,
) {
    let width = usize::from(MODERN_FRAME_WIDTH);
    let tm_bit = 1u8 << layer_index;
    for i in 0..buf.real.len() {
        if buf.real[i] && buf.hipri[i] == hi_priority {
            if let Some(tm) = main_tm {
                if tm[i / width] & tm_bit == 0 {
                    continue;
                }
            }
            // Main-screen + sub-screen window mask (TMW/TSW), gated by `windowed`.
            if layer_window_masks(
                layer_index,
                frame.windowsel,
                windowed,
                (i % width) as u32,
                i / width,
                &frame.window_scanlines,
            ) {
                continue;
            }
            screen.c5[i] = buf.c5[i];
            screen.bit[i] = layer_index;
            screen.real[i] = true;
        }
    }
}

/// Mosaic variant of [`composite_mode1`]: render each enabled BG layer (1-3) into
/// its own buffer, apply the source-snap to mosaiced layers, then composite in the
/// SAME Mode-1 z-order. Sprites are NOT mosaiced. Only reached when `mosaic_active`.
#[allow(clippy::too_many_arguments)]
fn composite_mode1_mosaic(
    screen: &mut Screen,
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
    enabled: u8,
    main_tm: Option<&[u8]>,
    windowed: u8,
    ctx: &crate::modern_hd_overrides::HdOverrideCtx,
) {
    let bg_on = |i: usize| (enabled >> i) & 1 != 0;
    let obj_on = (enabled >> 4) & 1 != 0;
    let bg = &frame.bg_layers;
    let len = screen.c5.len();
    let n = usize::from(frame.mosaic_size);

    // Build one buffer per enabled BG layer (1-3), source-snapping mosaiced layers.
    let mut bufs: [Option<BgBuf>; 3] = [None, None, None];
    for layer in 0..3usize {
        if !bg_on(layer) {
            continue;
        }
        let mut buf = render_bg_layer_buf(&bg[layer], bg_cells, frame, len, ctx);
        if frame.mosaic_enabled & (1u8 << layer) != 0 {
            mosaic_snap_bg_buf(&mut buf, n);
        }
        bufs[layer] = Some(buf);
    }

    // OBJ resolved once (sprites are not mosaiced), same as the non-mosaic path.
    // Mosaic is a native-only path (scale == 1), so OBJ uses native dims.
    let native_w = usize::from(MODERN_FRAME_WIDTH);
    let obj = if obj_on {
        Some(resolve_obj_layer(
            frame,
            sprite_cells,
            len,
            ctx,
            native_w,
            1,
        ))
    } else {
        None
    };

    // Mode-1 z-order: BG3-lo, OBJ0, OBJ1, BG2-lo, BG1-lo, OBJ2, BG2-hi, BG1-hi,
    // OBJ3, BG3-hi.
    if let Some(b) = &bufs[2] {
        paint_bg_buf(screen, b, 2, false, main_tm, frame, windowed);
    }
    if let Some(o) = &obj {
        paint_obj_priority(screen, o, 0, main_tm, frame, windowed, native_w, 1);
        paint_obj_priority(screen, o, 1, main_tm, frame, windowed, native_w, 1);
    }
    if let Some(b) = &bufs[1] {
        paint_bg_buf(screen, b, 1, false, main_tm, frame, windowed);
    }
    if let Some(b) = &bufs[0] {
        paint_bg_buf(screen, b, 0, false, main_tm, frame, windowed);
    }
    if let Some(o) = &obj {
        paint_obj_priority(screen, o, 2, main_tm, frame, windowed, native_w, 1);
    }
    if let Some(b) = &bufs[1] {
        paint_bg_buf(screen, b, 1, true, main_tm, frame, windowed);
    }
    if let Some(b) = &bufs[0] {
        paint_bg_buf(screen, b, 0, true, main_tm, frame, windowed);
    }
    if let Some(o) = &obj {
        paint_obj_priority(screen, o, 3, main_tm, frame, windowed, native_w, 1);
    }
    if let Some(b) = &bufs[2] {
        paint_bg_buf(screen, b, 2, true, main_tm, frame, windowed);
    }
}

/// True when BG `layer`'s per-scanline scroll (HDMA-captured `bg_scroll_scanlines`)
/// differs from its baked base `scroll_x`/`scroll_y` on any scanline. When false the
/// layer is uniform and the fast (single-baked-scroll) compositor path is exact.
fn bg_layer_scroll_varies(frame: &ModernFrame, layer: usize) -> bool {
    if frame.bg_scroll_scanlines.is_empty() {
        return false;
    }
    let base_h = frame.bg_layers[layer].scroll_x;
    let base_v = frame.bg_layers[layer].scroll_y;
    frame
        .bg_scroll_scanlines
        .iter()
        .any(|sl| sl[layer][0] != base_h || sl[layer][1] != base_v)
}

/// True when a BG layer must be sampled through the full tilemap torus instead
/// of the fast screen-space buffer. Varying HDMA scroll needs per-row deltas;
/// uniform non-zero scroll still needs torus sampling so edge-crossing tiles wrap
/// across the screen boundary like the SNES tilemap.
fn bg_layer_needs_torus_sampling(frame: &ModernFrame, layer: usize) -> bool {
    bg_layer_scroll_varies(frame, layer)
        || frame.bg_layers[layer].scroll_x != 0
        || frame.bg_layers[layer].scroll_y != 0
}

/// Render one BG layer into a FULL-TORUS buffer (`bg_w` x `bg_h`, the SNES scroll
/// wrap period), so it can be re-sampled per scanline with an arbitrary scroll. The
/// baked tile `screen_x`/`screen_y` are the canonical wrapped positions for the base
/// scroll (range `[-(bg_w-256), 256)` x `[-(bg_h-224), 224)`); they map to torus
/// index `(screen + off)` with `off = (bg_w-256, bg_h-224)`, and per-pixel wrap
/// (`rem_euclid`) tiles the torus seamlessly. A torus column `bx` represents tilemap
/// world pixel `(bx - off_x + base_h) mod bg_w`; row `by` likewise (incl. the SNES
/// +1 vertical fetch already baked into `screen_y`).
fn render_bg_layer_torus(
    layer: &crate::modern_frame::ModernBgLayer,
    cells: &[ModernIndexTile],
    frame: &ModernFrame,
    bg_w: usize,
    bg_h: usize,
    ctx: &crate::modern_hd_overrides::HdOverrideCtx,
) -> BgBuf {
    let len = bg_w * bg_h;
    let mut buf = BgBuf {
        c5: vec![[0u8; 3]; len],
        real: vec![false; len],
        hipri: vec![false; len],
    };
    let off_x = (bg_w as i32) - 256;
    let off_y = (bg_h as i32) - 224;
    for inst in &layer.index_tiles {
        let cell = match cells.get(inst.cell_id as usize) {
            Some(c) => c,
            None => continue,
        };
        let ov = ctx.resolve(cell.source_key);
        let bx0 = i32::from(inst.screen_x) + off_x;
        let by0 = i32::from(inst.screen_y) + off_y;
        for sy in 0..8i32 {
            for sx in 0..8i32 {
                let index = cell.indices[(sy * 8 + sx) as usize];
                if index == 0 {
                    continue;
                }
                let bx = (bx0 + sx).rem_euclid(bg_w as i32) as usize;
                let by = (by0 + sy).rem_euclid(bg_h as i32) as usize;
                // un-flip HD sampling back to source orientation (base index is baked-flipped)
                let hx = if cell.hflip { 7 - sx } else { sx };
                let hy = if cell.vflip { 7 - sy } else { sy };
                let cgram_idx = inst.palette as usize * 16 + index as usize;
                let color = match crate::modern_hd_overrides::resolve_pixel_color(
                    index,
                    cgram_idx,
                    frame.cgram_rgba[cgram_idx],
                    ov,
                    ctx.reference(),
                    hx as u32,
                    hy as u32,
                    8,
                ) {
                    Some(c) => c,
                    None => continue,
                };
                let i = by * bg_w + bx;
                buf.c5[i] = [color[0] >> 3, color[1] >> 3, color[2] >> 3];
                buf.real[i] = true;
                buf.hipri[i] = inst.priority;
            }
        }
    }
    buf
}

/// Re-sample a full-torus BG buffer into a 256x224 screen-space [`BgBuf`], applying
/// each output row's per-scanline scroll DELTA relative to the layer's base scroll.
/// This is the modern equivalent of the classic `bg_layer.wgsl`, which samples each
/// output scanline at `(x + h_scroll[sy], sy + 1 + v_scroll[sy])`: with the base
/// scroll already baked into the torus, output `(x, sy)` reads torus index
/// `((x + dh + off_x) mod bg_w, (sy + dv + off_y) mod bg_h)`, `dh/dv` the per-row
/// scroll minus base.
fn sample_scanline_scroll(
    torus: &BgBuf,
    frame: &ModernFrame,
    layer: usize,
    bg_w: usize,
    bg_h: usize,
) -> BgBuf {
    let width = usize::from(MODERN_FRAME_WIDTH);
    let height = usize::from(MODERN_FRAME_HEIGHT);
    let len = width * height;
    let mut out = BgBuf {
        c5: vec![[0u8; 3]; len],
        real: vec![false; len],
        hipri: vec![false; len],
    };
    let base_h = i32::from(frame.bg_layers[layer].scroll_x);
    let base_v = i32::from(frame.bg_layers[layer].scroll_y);
    let off_x = (bg_w as i32) - 256;
    let off_y = (bg_h as i32) - 224;
    for sy in 0..height {
        let (dh, dv) = match frame.bg_scroll_scanlines.get(sy) {
            Some(sl) => (
                i32::from(sl[layer][0]) - base_h,
                i32::from(sl[layer][1]) - base_v,
            ),
            None => (0, 0),
        };
        let by = (sy as i32 + dv + off_y).rem_euclid(bg_h as i32) as usize;
        for x in 0..width {
            let bx = (x as i32 + dh + off_x).rem_euclid(bg_w as i32) as usize;
            let ti = by * bg_w + bx;
            let oi = sy * width + x;
            out.c5[oi] = torus.c5[ti];
            out.real[oi] = torus.real[ti];
            out.hipri[oi] = torus.hipri[ti];
        }
    }
    out
}

/// Per-scanline-scroll variant of [`composite_mode1`]: BG layers flagged in
/// `scroll_layers` are wrap-sampled per output row (HDMA scroll); the rest use the
/// plain baked-scroll per-layer buffer (identical to the direct fast path). Composited
/// in the SAME Mode-1 z-order as [`composite_mode1_mosaic`], with the same TM/window
/// gates via [`paint_bg_buf`]/[`paint_obj_priority`]. Mosaic is handled by its own
/// path; this runs only when mosaic is inactive.
#[allow(clippy::too_many_arguments)]
fn composite_mode1_scanline_scroll(
    screen: &mut Screen,
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
    enabled: u8,
    main_tm: Option<&[u8]>,
    windowed: u8,
    scroll_layers: [bool; 3],
    ctx: &crate::modern_hd_overrides::HdOverrideCtx,
) {
    let bg_on = |i: usize| (enabled >> i) & 1 != 0;
    let obj_on = (enabled >> 4) & 1 != 0;
    let bg = &frame.bg_layers;
    let len = screen.c5.len();

    let mut bufs: [Option<BgBuf>; 3] = [None, None, None];
    for l in 0..3usize {
        if !bg_on(l) {
            continue;
        }
        let buf = if scroll_layers[l] {
            let bg_w = usize::from(bg[l].wrap_w).max(256);
            let bg_h = usize::from(bg[l].wrap_h).max(224);
            let torus = render_bg_layer_torus(&bg[l], bg_cells, frame, bg_w, bg_h, ctx);
            sample_scanline_scroll(&torus, frame, l, bg_w, bg_h)
        } else {
            render_bg_layer_buf(&bg[l], bg_cells, frame, len, ctx)
        };
        bufs[l] = Some(buf);
    }

    // Per-scanline-scroll is a native-only path (scale == 1); OBJ uses native dims.
    let native_w = usize::from(MODERN_FRAME_WIDTH);
    let obj = if obj_on {
        Some(resolve_obj_layer(
            frame,
            sprite_cells,
            len,
            ctx,
            native_w,
            1,
        ))
    } else {
        None
    };

    // Mode-1 z-order: BG3-lo, OBJ0, OBJ1, BG2-lo, BG1-lo, OBJ2, BG2-hi, BG1-hi,
    // OBJ3, BG3-hi.
    if let Some(b) = &bufs[2] {
        paint_bg_buf(screen, b, 2, false, main_tm, frame, windowed);
    }
    if let Some(o) = &obj {
        paint_obj_priority(screen, o, 0, main_tm, frame, windowed, native_w, 1);
        paint_obj_priority(screen, o, 1, main_tm, frame, windowed, native_w, 1);
    }
    if let Some(b) = &bufs[1] {
        paint_bg_buf(screen, b, 1, false, main_tm, frame, windowed);
    }
    if let Some(b) = &bufs[0] {
        paint_bg_buf(screen, b, 0, false, main_tm, frame, windowed);
    }
    if let Some(o) = &obj {
        paint_obj_priority(screen, o, 2, main_tm, frame, windowed, native_w, 1);
    }
    if let Some(b) = &bufs[1] {
        paint_bg_buf(screen, b, 1, true, main_tm, frame, windowed);
    }
    if let Some(b) = &bufs[0] {
        paint_bg_buf(screen, b, 0, true, main_tm, frame, windowed);
    }
    if let Some(o) = &obj {
        paint_obj_priority(screen, o, 3, main_tm, frame, windowed, native_w, 1);
    }
    if let Some(b) = &bufs[2] {
        paint_bg_buf(screen, b, 2, true, main_tm, frame, windowed);
    }
}

/// The resolved OBJ layer: one winning sprite pixel per screen location (lowest
/// OAM index), carrying its 5-bit color, math `bit` (palette-gated), and the OBJ
/// priority attribute used to slot it against the BG layers.
struct ObjLayer {
    c5: Vec<[u8; 3]>,
    bit: Vec<u8>,
    prio: Vec<u8>,
    real: Vec<bool>,
}

/// Resolve OBJ-vs-OBJ by OAM index: paint sprites in REVERSE `index_sprites`
/// order with REPLACE so the earliest (lowest-index) opaque sprite wins each
/// pixel, recording that pixel's priority attribute for later BG slotting.
fn resolve_obj_layer(
    frame: &ModernFrame,
    sprite_cells: &[ModernIndexTile],
    len: usize,
    ctx: &crate::modern_hd_overrides::HdOverrideCtx,
    out_width: usize,
    scale: usize,
) -> ObjLayer {
    let mut o = ObjLayer {
        c5: vec![[0u8; 3]; len],
        bit: vec![0u8; len],
        prio: vec![0u8; len],
        real: vec![false; len],
    };
    // Per-tile output footprint in pixels (`8 * scale`); sub-pixel HD sampling maps each
    // output pixel to an HD texel within this footprint. At scale == 1 this is 8.
    let fp = (8 * scale) as u32;
    for inst in frame.index_sprites.iter().rev() {
        let cell = match sprite_cells.get(inst.cell_id as usize) {
            Some(c) => c,
            None => continue,
        };
        let ov = ctx.resolve(cell.source_key);
        for oy in 0..(8 * scale) {
            let ny = oy / scale; // native texel row 0..8
            if inst.row_mask & (1 << ny) == 0 {
                continue; // dropped by the per-scanline OBJ budget
            }
            for ox in 0..(8 * scale) {
                let nx = ox / scale; // native texel column 0..8
                                     // Sprites are NOT flip-baked: read the index at the un-flipped source
                                     // texel, exactly as the native path did.
                let src_x = if inst.hflip { 7 - nx } else { nx };
                let src_y = if inst.vflip { 7 - ny } else { ny };
                let index = cell.indices[src_y * 8 + src_x];
                if index == 0 {
                    continue;
                }
                let dst_x = inst.screen_x as isize * scale as isize + ox as isize;
                let dst_y = inst.screen_y as isize * scale as isize + oy as isize;
                if dst_x < 0
                    || dst_y < 0
                    || dst_x >= (256 * scale) as isize
                    || dst_y >= (224 * scale) as isize
                {
                    continue;
                }
                let (dst_x, dst_y) = (dst_x as usize, dst_y as usize);
                let cgram_idx = 0x80 + inst.palette as usize * 16 + index as usize;
                // HD sample coords in SOURCE orientation over the `fp` footprint: the
                // un-flip mirrors the whole footprint (`fp-1-ox`), so at scale == 1 this
                // collapses to `src_x`/`src_y` (fp == 8) — the native sampling.
                let hx = if inst.hflip { fp as usize - 1 - ox } else { ox };
                let hy = if inst.vflip { fp as usize - 1 - oy } else { oy };
                let color = match crate::modern_hd_overrides::resolve_pixel_color(
                    index,
                    cgram_idx,
                    frame.cgram_rgba[cgram_idx],
                    ov,
                    ctx.reference(),
                    hx as u32,
                    hy as u32,
                    fp,
                ) {
                    Some(c) => c,
                    None => continue,
                };
                let i = dst_y * out_width + dst_x;
                o.c5[i] = [color[0] >> 3, color[1] >> 3, color[2] >> 3];
                // OBJ color-math only for palettes 4-7 (bit 4); 0-3 use bit 6 (never
                // in `math_enabled`). See the obj_color_math test.
                o.bit[i] = if inst.palette < 4 { 6 } else { 4 };
                o.prio[i] = inst.priority;
                o.real[i] = true;
            }
        }
    }
    o
}

/// Paint the resolved OBJ pixels whose priority attribute equals `prio` into the
/// screen (REPLACE). Called at the matching Mode-1 z-slot so OBJ-vs-BG layering
/// follows the priority attribute while OBJ-vs-OBJ stays index-ordered.
fn paint_obj_priority(
    screen: &mut Screen,
    obj: &ObjLayer,
    prio: u8,
    main_tm: Option<&[u8]>,
    frame: &ModernFrame,
    windowed: u8,
    out_width: usize,
    scale: usize,
) {
    for i in 0..obj.real.len() {
        if obj.real[i] && obj.prio[i] == prio {
            // Per-scanline TM and window data are NATIVE-length: index them by the
            // native row/col (`out_y / scale`, `out_x / scale`). At scale == 1 these
            // collapse to `i / width` / `i % width` — the native indexing.
            let nrow = (i / out_width) / scale;
            let ncol = (i % out_width) / scale;
            // Per-scanline OBJ enable (TM bit 4) for the main screen only.
            if let Some(tm) = main_tm {
                if tm[nrow] & 0x10 == 0 {
                    continue;
                }
            }
            // OBJ window mask (layer 4: windowsel >> 16, TMW/TSW bit 4) on both screens.
            if layer_window_masks(
                4,
                frame.windowsel,
                windowed,
                ncol as u32,
                nrow,
                &frame.window_scanlines,
            ) {
                continue;
            }
            screen.c5[i] = obj.c5[i];
            screen.bit[i] = obj.bit[i];
            screen.real[i] = true;
        }
    }
}

/// Main-screen window mask for a layer pixel, byte-exact with the classic shaders
/// (`bg_layer.wgsl::layer_window_active` / `sprite_pixels.wgsl::obj_window_active`).
///
/// `layer` is the SNES layer index (BG1-3 = 0-2, OBJ = 4); it selects both the TMW
/// main-screen window-enable bit (`screen_windowed_main & (1<<layer)`) and the
/// window-select nibble (`windowsel >> (layer*4)`: bit0=W1inv, bit1=W1en, bit2=W2inv,
/// bit3=W2en). Returns `true` where the pixel is INSIDE the active window region and
/// must therefore be MASKED (not drawn → backdrop shows), matching the shaders which
/// `discard` where the corresponding `*_window_active` is true. Only the main screen
/// is gated (the classic never window-masks the sub screen).
fn layer_window_masks(
    layer: u8,
    windowsel: u32,
    screen_windowed_main: u8,
    sx: u32,
    sy: usize,
    window_scanlines: &[[u8; 4]],
) -> bool {
    if screen_windowed_main & (1u8 << layer) == 0 {
        return false;
    }
    let window_flags = (windowsel >> (u32::from(layer) * 4)) & 0x0f;
    let w1_enabled = window_flags & 0x2 != 0;
    let w2_enabled = window_flags & 0x8 != 0;
    if !w1_enabled && !w2_enabled {
        return false;
    }
    let [w1l, w1r, w2l, w2r] = window_scanlines
        .get(sy)
        .copied()
        .unwrap_or([0u8; 4])
        .map(u32::from);
    let mut test1 = sx >= w1l && sx <= w1r;
    let mut test2 = sx >= w2l && sx <= w2r;
    if window_flags & 0x1 != 0 {
        test1 = !test1;
    }
    if window_flags & 0x4 != 0 {
        test2 = !test2;
    }
    if w1_enabled && !w2_enabled {
        return test1;
    }
    if !w1_enabled && w2_enabled {
        return test2;
    }
    test1 || test2
}

/// Composite one BG layer's indexed tiles into `screen` (painter-style, REPLACE),
/// stamping the layer's math `bit` and marking pixels real. Only tiles whose
/// per-tile priority matches `hi_priority` are painted, so the caller can run the
/// SNES Mode 1 lo-pass and hi-pass with the other layers / OBJ priorities
/// interleaved between them.
#[allow(clippy::too_many_arguments)]
fn composite_index_tiles_c5(
    screen: &mut Screen,
    layer: &crate::modern_frame::ModernBgLayer,
    cells: &[ModernIndexTile],
    frame: &ModernFrame,
    hi_priority: bool,
    main_tm: Option<&[u8]>,
    windowed: u8,
    ctx: &crate::modern_hd_overrides::HdOverrideCtx,
    out_width: usize,
    scale: usize,
) {
    let bit = layer.index;
    let tm_bit = 1u8 << bit; // TM enable bit for this BG layer
                             // Per-tile output footprint in pixels (`8 * scale`); at scale == 1 this is 8.
    let fp = (8 * scale) as u32;
    for inst in &layer.index_tiles {
        if inst.priority != hi_priority {
            continue;
        }
        let cell = match cells.get(inst.cell_id as usize) {
            Some(c) => c,
            None => continue,
        };
        let ov = ctx.resolve(cell.source_key);
        for oy in 0..(8 * scale) {
            let nsy = oy / scale; // native texel row 0..8
            for ox in 0..(8 * scale) {
                let nsx = ox / scale; // native texel column 0..8
                let index = cell.indices[nsy * 8 + nsx];
                if index == 0 {
                    continue; // transparent
                }
                let dst_x = inst.screen_x as isize * scale as isize + ox as isize;
                let dst_y = inst.screen_y as isize * scale as isize + oy as isize;
                if dst_x < 0
                    || dst_y < 0
                    || dst_x >= (256 * scale) as isize
                    || dst_y >= (224 * scale) as isize
                {
                    continue;
                }
                let (dst_x, dst_y) = (dst_x as usize, dst_y as usize);
                // Per-scanline TM and window data are NATIVE-length: index by the native
                // row/col. At scale == 1 these equal `dst_y`/`dst_x`.
                let nrow = dst_y / scale;
                let ncol = dst_x / scale;
                // Per-scanline BG-layer enable (TM) for the main screen only.
                if let Some(tm) = main_tm {
                    if tm[nrow] & tm_bit == 0 {
                        continue;
                    }
                }
                // BG window mask (TMW/TSW), applied on both screens via `windowed`.
                if layer_window_masks(
                    bit,
                    frame.windowsel,
                    windowed,
                    ncol as u32,
                    nrow,
                    &frame.window_scanlines,
                ) {
                    continue;
                }
                // un-flip HD sampling back to source orientation (base index is
                // baked-flipped) over the `fp` footprint: at scale == 1 this collapses
                // to `7 - sx` / `sx` (fp == 8) — the native sampling.
                let hx = if cell.hflip { fp as usize - 1 - ox } else { ox };
                let hy = if cell.vflip { fp as usize - 1 - oy } else { oy };
                let cgram_idx = inst.palette as usize * 16 + index as usize;
                let color = match crate::modern_hd_overrides::resolve_pixel_color(
                    index,
                    cgram_idx,
                    frame.cgram_rgba[cgram_idx],
                    ov,
                    ctx.reference(),
                    hx as u32,
                    hy as u32,
                    fp,
                ) {
                    Some(c) => c,
                    None => continue,
                };
                let i = dst_y * out_width + dst_x;
                screen.c5[i] = [color[0] >> 3, color[1] >> 3, color[2] >> 3];
                screen.bit[i] = bit;
                screen.real[i] = true;
            }
        }
    }
}

/// Master-brightness scale on a 5-bit component (Snes9x's `mul_brightness`).
#[inline]
fn scale_brightness5(c5: i32, brightness: u8) -> i32 {
    (c5.clamp(0, 31) * i32::from(brightness.min(15)) + 7) / 15
}

/// Snes9x performs half-add in packed RGB565. Its green channel is six bits,
/// with bit 4 of the SNES five-bit component duplicated into the low bit.
/// The oracle comparator then collapses that six-bit value back to five bits.
#[inline]
fn snes9x_rgb565_half_add_green(primary_g5: i32, second_g5: i32) -> i32 {
    let expand_green6 = |g5: i32| {
        let g5 = g5.clamp(0, 31);
        (g5 << 1) | (g5 >> 4)
    };
    (expand_green6(primary_g5) + expand_green6(second_g5)) >> 2
}

/// Expand a brightness-scaled 5-bit component to 8 bits.
#[inline]
fn expand_5bit(c5: i32) -> u8 {
    let c = c5.clamp(0, 31) as u8;
    (c << 3) | (c >> 2)
}

#[cfg(test)]
fn expand_brightness(c5: i32, brightness: u8) -> u8 {
    expand_5bit(scale_brightness5(c5, brightness))
}

/// Decode a CGWSEL clip/math-mode bit, byte-exact with `post_process.wgsl::cw_bit`.
///
/// `mode` 0=always 1 (never clip / always math), 1=follows window, 2=inverted,
/// 3=always 0 (always clip / never math). Mirrors the SNES CW_BITS_MOD table:
/// `((w & masks[mode]) ^ masks[mode + 4]) != 0`, where `w = 0xff` inside the window.
#[inline]
fn cw_bit(in_window: bool, mode: u8) -> bool {
    const MASKS: [u32; 8] = [0x00, 0xff, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00];
    let w = if in_window { 0xffu32 } else { 0 };
    let m = mode as usize & 7;
    ((w & MASKS[m]) ^ MASKS[m + 4]) != 0
}

/// Color-math window membership for screen column `sx`, byte-exact with the GPU
/// post-process shader: enabled W1/W2 masks (each with its inversion flag) are
/// OR-combined. `windowsel_cm` bits: 0=W1inv, 1=W1en, 2=W2inv, 3=W2en.
#[inline]
fn in_cm_window(sx: u32, win: [u8; 4], windowsel_cm: u8) -> bool {
    let [w1l, w1r, w2l, w2r] = win.map(u32::from);
    let mut inside = false;
    if windowsel_cm & 0x2 != 0 {
        let mut in_w1 = w1l <= w1r && sx >= w1l && sx <= w1r;
        if windowsel_cm & 0x1 != 0 {
            in_w1 = !in_w1;
        }
        inside |= in_w1;
    }
    if windowsel_cm & 0x8 != 0 {
        let mut in_w2 = w2l <= w2r && sx >= w2l && sx <= w2r;
        if windowsel_cm & 0x4 != 0 {
            in_w2 = !in_w2;
        }
        inside |= in_w2;
    }
    inside
}

/// Full modern software render: BG + sprites + SNES color-math + master brightness
/// in one call, producing the FINAL 8-bit RGBA (256×224) that mirrors the classic
/// post-process pipeline (`post_process.wgsl`), including the color-math/clip
/// window gating (`cw_bit` + per-scanline `window_scanlines`).
///
/// `bg_cells` are the indexed BG tile patterns; `sprite_cells` the live-VRAM sprite
/// patterns (from `extract_modern_sprites_from_vram`). A MAIN and a SUB composite
/// are built from the raw `screen_enabled_main`/`screen_enabled_sub` bits (BG bits
/// 0-3, OBJ bit 4), then color math + brightness are applied per pixel.
pub fn render_modern_frame_full(
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
) -> Vec<u8> {
    render_modern_frame_full_with_overrides(
        frame,
        bg_cells,
        sprite_cells,
        &crate::modern_hd_overrides::HdOverrideCtx::disabled(),
    )
}

/// As `render_modern_frame_full`, but applies source-keyed HD overrides via `ctx`
/// (detail-modulated recolor). `HdOverrideCtx::disabled()` → byte-identical to the
/// plain entry.
pub fn render_modern_frame_full_with_overrides(
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
    ctx: &crate::modern_hd_overrides::HdOverrideCtx,
) -> Vec<u8> {
    let (main, sub) = build_modern_screens_with_overrides(frame, bg_cells, sprite_cells, ctx);
    finalize_frame(&main, &sub, frame, usize::from(MODERN_FRAME_WIDTH), 1)
}

fn build_modern_screens_with_overrides(
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
    ctx: &crate::modern_hd_overrides::HdOverrideCtx,
) -> (Screen, Screen) {
    let width = usize::from(MODERN_FRAME_WIDTH);
    let height = usize::from(MODERN_FRAME_HEIGHT);
    let len = width * height;

    if frame.forced_blank {
        let black = [0, 0, 0];
        return (Screen::new(black, len), Screen::new(black, len));
    }

    let bd = &frame.backdrop_color_rgba;
    let backdrop_c5 = [bd[0] >> 3, bd[1] >> 3, bd[2] >> 3];

    // MAIN composite in SNES Mode 1 z-order (back → front), mirroring the GPU
    // reference (`gpu_renderer.rs`): BG3-lo, OBJ0, OBJ1, BG2-lo, BG1-lo, OBJ2,
    // BG2-hi, BG1-hi, OBJ3, BG3-hi. Painter-style REPLACE, so a later (more-front)
    // source overwrites earlier ones where opaque.
    let mut main = Screen::new(backdrop_c5, len);
    composite_mode1(
        &mut main,
        frame,
        bg_cells,
        sprite_cells,
        frame.screen_enabled_main,
        Some(&frame.main_tm_scanlines),
        frame.screen_windowed_main,
        ctx,
        width,
        1,
    );

    // SUB composite: same Mode 1 z-order over the sub-screen enable mask. The sub
    // screen skips the per-scanline TM check (matching the classic renderer).
    let mut sub = Screen::new(backdrop_c5, len);
    composite_mode1(
        &mut sub,
        frame,
        bg_cells,
        sprite_cells,
        frame.screen_enabled_sub,
        None,
        frame.screen_windowed_sub,
        ctx,
        width,
        1,
    );

    (main, sub)
}

/// True if the frame would take the mosaic OR torus-scroll composite path on
/// EITHER screen — those paths are not N×-parameterized (Phase 2), so the entry renders
/// them natively and nearest-upscales. Detection mirrors `composite_mode1` verbatim.
fn frame_uses_complex_bg_path(frame: &ModernFrame) -> bool {
    for enabled in [frame.screen_enabled_main, frame.screen_enabled_sub] {
        let mosaic = frame.mosaic_size > 1 && (frame.mosaic_enabled & enabled & 0x07) != 0;
        let bg_on = |i: usize| (enabled >> i) & 1 != 0;
        let torus_scroll = (0..3).any(|i| bg_on(i) && bg_layer_needs_torus_sampling(frame, i));
        if mosaic || torus_scroll {
            return true;
        }
    }
    false
}

/// Modern render at integer `scale` (1..=4). `scale<=1` → the native entry unchanged.
/// A mosaic / torus-scroll frame → native render nearest-upscaled to N× (those
/// composite paths are not N×-parameterized). Otherwise the parameterized common path
/// runs directly at N·256 × N·224, sampling HD overrides sub-pixel.
pub fn render_modern_frame_full_scaled(
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
    ctx: &crate::modern_hd_overrides::HdOverrideCtx,
    scale: u32,
) -> Vec<u8> {
    let scale = scale.clamp(1, 4) as usize;
    if scale == 1 {
        return render_modern_frame_full_with_overrides(frame, bg_cells, sprite_cells, ctx);
    }
    if frame_uses_complex_bg_path(frame) {
        let native = render_modern_frame_full_with_overrides(frame, bg_cells, sprite_cells, ctx);
        return upscale_rgba_nearest(
            &native,
            usize::from(MODERN_FRAME_WIDTH),
            usize::from(MODERN_FRAME_HEIGHT),
            scale,
        );
    }
    let out_width = 256 * scale;
    let len = out_width * 224 * scale;
    if frame.forced_blank {
        let mut out = vec![0u8; len * 4];
        for px in out.chunks_exact_mut(4) {
            px.copy_from_slice(&[0, 0, 0, 0xff]);
        }
        return out;
    }
    let bd = &frame.backdrop_color_rgba;
    let backdrop_c5 = [bd[0] >> 3, bd[1] >> 3, bd[2] >> 3];
    let mut main = Screen::new(backdrop_c5, len);
    let mut sub = Screen::new(backdrop_c5, len);
    // The main and sub composites write disjoint buffers and share only immutable
    // inputs, so build them concurrently (scoped threads → no borrow escape). At scale 1
    // this path isn't taken (the native entry handles it), so parity is unaffected.
    std::thread::scope(|s| {
        s.spawn(|| {
            composite_mode1(
                &mut main,
                frame,
                bg_cells,
                sprite_cells,
                frame.screen_enabled_main,
                Some(&frame.main_tm_scanlines),
                frame.screen_windowed_main,
                ctx,
                out_width,
                scale,
            );
        });
        composite_mode1(
            &mut sub,
            frame,
            bg_cells,
            sprite_cells,
            frame.screen_enabled_sub,
            None,
            frame.screen_windowed_sub,
            ctx,
            out_width,
            scale,
        );
    });
    finalize_frame(&main, &sub, frame, out_width, scale)
}

/// Block-replicate an RGBA frame to `scale`× (nearest upscale): each source pixel
/// becomes a `scale×scale` block. Used for the mosaic/per-scanline-scroll fallback,
/// which renders natively then upscales to match the HD frame size.
pub fn upscale_rgba_nearest(rgba: &[u8], width: usize, height: usize, scale: usize) -> Vec<u8> {
    if scale <= 1 {
        return rgba.to_vec();
    }
    let out_w = width * scale;
    let mut out = vec![0u8; out_w * height * scale * 4];
    for sy in 0..height {
        for sx in 0..width {
            let src = (sy * width + sx) * 4;
            let px: [u8; 4] = [rgba[src], rgba[src + 1], rgba[src + 2], rgba[src + 3]];
            for dy in 0..scale {
                let oy = sy * scale + dy;
                for dx in 0..scale {
                    let ox = sx * scale + dx;
                    let o = (oy * out_w + ox) * 4;
                    out[o..o + 4].copy_from_slice(&px);
                }
            }
        }
    }
    out
}

/// Shared color-math + master-brightness resolve for a composited MAIN/SUB screen
/// pair, byte-identical to the classic post-process shader. Also emits the
/// `ZELDA3_BITMAP` winning-layer diagnostic for the main screen. Used by both the
/// Mode-1 (`render_modern_frame_full`) and Mode-7 (`render_modern_mode7_frame`)
/// compositors so they share one finalize path.
/// Resolve one finalized output pixel `i` to RGBA — the per-pixel color-math +
/// brightness body of `finalize_frame`, factored out so the finalize pass can run it
/// serially or across parallel row stripes with identical results. `fixed` and
/// `no_effect_math` are the frame-constant terms hoisted by the caller.
#[allow(clippy::too_many_arguments)]
#[inline]
fn finalize_pixel(
    i: usize,
    main: &Screen,
    sub: &Screen,
    frame: &ModernFrame,
    width: usize,
    scale: usize,
    fixed: [i32; 3],
    no_effect_math: bool,
) -> [u8; 4] {
    // Per-output pixel color math, but the window data is NATIVE-length, so index
    // it by the native row/col (`out_y / scale`, `out_x / scale`). At scale == 1
    // these equal the output coords — byte-identical to before.
    let out_x = i % width;
    let out_y = i / width;
    let nrow = out_y / scale;
    let ncol = out_x / scale;
    if nrow < usize::from(frame.forced_blank_scanlines)
        || frame
            .forced_blank_from_scanline
            .is_some_and(|line| nrow >= usize::from(line))
    {
        return [0, 0, 0, 0xff];
    }
    let mut c = [
        i32::from(main.c5[i][0]),
        i32::from(main.c5[i][1]),
        i32::from(main.c5[i][2]),
    ];
    let layer_math_on = (frame.math_enabled >> main.bit[i]) & 1 != 0;
    // Color-window membership for this pixel, then gate clip + color-math exactly
    // like the classic post-process shader.
    let win = frame
        .window_scanlines
        .get(nrow)
        .copied()
        .unwrap_or([0u8; 4]);
    let cm_window = in_cm_window(ncol as u32, win, frame.windowsel_cm);
    let not_clipped = cw_bit(cm_window, frame.clip_mode);
    let math_window_ok = cw_bit(cm_window, frame.prevent_math_mode);
    let do_math = !no_effect_math && layer_math_on && math_window_ok;
    if !not_clipped {
        return [0, 0, 0, 0xff];
    }
    // Snes9x (the parity oracle) pre-scales palette colors by master
    // brightness before color math, fixed color included; mirror that order
    // (identical to math-then-brightness at full brightness). See the same
    // transform in modern_finalize.wgsl.
    // Snes9x derives its palette map, fixed-color map, and full-add cap from
    // the same INIDISP generation. CGRAM may be deferred independently, but
    // brightness cannot be split between the color-math operands.
    let scanout_brightness = frame.scanout_brightness();
    for ch in 0..3 {
        c[ch] = scale_brightness5(c[ch], scanout_brightness);
    }
    if do_math {
        let primary_green = c[1];
        let (operand, second_real) = if frame.add_subscreen {
            if sub.real[i] {
                (
                    [
                        i32::from(sub.c5[i][0]),
                        i32::from(sub.c5[i][1]),
                        i32::from(sub.c5[i][2]),
                    ],
                    true,
                )
            } else {
                (fixed, false)
            }
        } else {
            (fixed, false)
        };
        let mut second_green = 0;
        for ch in 0..3 {
            let operand = scale_brightness5(operand[ch], scanout_brightness);
            if ch == 1 {
                second_green = operand;
            }
            if frame.subtract_color {
                c[ch] -= operand;
            } else {
                c[ch] += operand;
            }
        }
        let half_color_applies = frame.half_color && (second_real || !frame.add_subscreen);
        // When the frame never reached full brightness, Snes9x selects
        // `COLOR_ADD_BRIGHTNESS` (tile.cpp renderer index 7) for every full-add
        // blend, including subscreen addition. Its inputs are already
        // brightness-scaled, then the sum is capped at brightness-mapped white.
        // Half-add remains the packed RGB565 average; the add-subsreen fallback
        // to fixed color is a full add and therefore still takes this cap.
        if scanout_brightness < 15 && !frame.subtract_color && !half_color_applies {
            let brightness_white = scale_brightness5(31, scanout_brightness);
            for channel in &mut c {
                *channel = (*channel).min(brightness_white);
            }
        }
        if half_color_applies {
            c[0] >>= 1;
            if frame.subtract_color {
                c[1] >>= 1;
            } else {
                c[1] = snes9x_rgb565_half_add_green(primary_green, second_green);
            }
            c[2] >>= 1;
        }
    }
    [
        expand_5bit(c[0]),
        expand_5bit(c[1]),
        expand_5bit(c[2]),
        0xff,
    ]
}

fn finalize_frame(
    main: &Screen,
    sub: &Screen,
    frame: &ModernFrame,
    out_width: usize,
    scale: usize,
) -> Vec<u8> {
    let width = out_width;
    let len = main.c5.len();
    let mut out = vec![0u8; len * 4];

    // `rendered_subscreen`: is there ANY sub composite (BG1-4 or OBJ on sub)?
    let rendered_subscreen = (frame.screen_enabled_sub & 0x1f) != 0;
    let fixed = [
        i32::from(frame.fixed_color_r),
        i32::from(frame.fixed_color_g),
        i32::from(frame.fixed_color_b),
    ];
    let no_effect_math = frame.fixed_color_r == 0
        && frame.fixed_color_g == 0
        && frame.fixed_color_b == 0
        && !frame.half_color
        && !rendered_subscreen;

    // DIAGNOSTIC (ZELDA3_BITMAP=<path>): write the per-pixel WINNING-LAYER map of
    // the MAIN screen as raw bytes (one u8 per pixel: 0=BG1 1=BG2 2=BG3 4=OBJ(pal4-7)
    // 5=backdrop 6=OBJ(pal0-3), 0xff=no real pixel). Lets a viewer attribute every
    // diff pixel to the layer/source that produced it — ground truth, vs guessing.
    if let Ok(path) = std::env::var("ZELDA3_BITMAP") {
        let map: Vec<u8> = (0..len)
            .map(|i| if main.real[i] { main.bit[i] } else { 0xff })
            .collect();
        let _ = std::fs::write(&path, &map);
    }

    // Per-pixel color math is independent per output pixel. For large (scaled) frames,
    // stripe the rows across CPU threads; for small/native frames the thread overhead
    // isn't worth it, so run serial. Both paths call `finalize_pixel`, so the output is
    // byte-identical either way (the parallel test-suite gate proves it).
    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    const PARALLEL_MIN_PIXELS: usize = 200_000; // native 256×224 stays serial; scale≥2 parallelizes
    if n_threads <= 1 || len < PARALLEL_MIN_PIXELS {
        for i in 0..len {
            let px = finalize_pixel(i, main, sub, frame, width, scale, fixed, no_effect_math);
            out[i * 4..i * 4 + 4].copy_from_slice(&px);
        }
    } else {
        let rows = len / width;
        let rows_per = rows.div_ceil(n_threads);
        std::thread::scope(|s| {
            for (t, chunk) in out.chunks_mut(rows_per * width * 4).enumerate() {
                let row0 = t * rows_per;
                s.spawn(move || {
                    let chunk_rows = chunk.len() / (width * 4);
                    for lr in 0..chunk_rows {
                        let y = row0 + lr;
                        let base = y * width;
                        for x in 0..width {
                            let px = finalize_pixel(
                                base + x,
                                main,
                                sub,
                                frame,
                                width,
                                scale,
                                fixed,
                                no_effect_math,
                            );
                            let o = lr * width * 4 + x * 4;
                            chunk[o..o + 4].copy_from_slice(&px);
                        }
                    }
                });
            }
        });
    }

    apply_hardware_startup_transient(&mut out, frame, out_width, scale);
    out
}

// --- Mode 7 (affine BG) software path ---------------------------------------
// CPU port of `mode7.wgsl`, sharing the modern OBJ + color-math + brightness
// machinery. Used for the map screen (module 14, PPU mode 7), which the Mode-1
// compositor cannot render. The Mode-7 BG decodes from live VRAM (its CHR is a
// single affine field, not a reusable atlas asset — same VRAM-decode precedent as
// BG3/Link in the Mode-1 path).

fn expand_m7_13(value: i32) -> i32 {
    let masked = value & 0x1fff;
    if masked & 0x1000 != 0 {
        masked | -8192
    } else {
        masked
    }
}

fn clip_m7_offset(value: i32) -> i32 {
    if value & 0x2000 != 0 {
        value | -1024
    } else {
        value & 1023
    }
}

/// Resolve one Mode-7 BG pixel to a CGRAM index (0..255), or `None` if transparent
/// (index 0) — byte-exact with `mode7.wgsl::fs_main`'s affine sample.
fn mode7_bg_index(frame: &crate::gpu_frame::GpuFrame<'_>, sx: usize, sy: usize) -> Option<u8> {
    let m7 = &frame.mode7;
    let m = frame.scanlines[sy].mode7_matrix;
    let (m0, m1, m2, m3) = (m[0] as i32, m[1] as i32, m[2] as i32, m[3] as i32);
    let x_center = expand_m7_13(m[4] as i32);
    let y_center = expand_m7_13(m[5] as i32);
    let h_scroll = expand_m7_13(m[6] as i32);
    let v_scroll = expand_m7_13(m[7] as i32);
    let clipped_h = clip_m7_offset(h_scroll - x_center);
    let clipped_v = clip_m7_offset(v_scroll - y_center);

    let y = sy as i32 + 1;
    let ry = if m7.y_flip { 255 - y } else { y };
    let start_x =
        (m0 * clipped_h & -64) + (m1 * ry & -64) + (m1 * clipped_v & -64) + (x_center << 8);
    let start_y =
        (m2 * clipped_h & -64) + (m3 * ry & -64) + (m3 * clipped_v & -64) + (y_center << 8);
    let rx = if m7.x_flip {
        255 - sx as i32
    } else {
        sx as i32
    };

    let mut x_pos = (start_x + m0 * rx) >> 8;
    let mut y_pos = (start_y + m2 * rx) >> 8;
    let mut outside = x_pos < 0 || x_pos >= 1024 || y_pos < 0 || y_pos >= 1024;
    x_pos &= 0x3ff;
    y_pos &= 0x3ff;
    if !m7.large_field {
        outside = false;
    }

    let tile = if outside {
        0u32
    } else {
        (frame
            .vram
            .get(((y_pos >> 3) * 128 + (x_pos >> 3)) as usize)
            .copied()
            .unwrap_or(0)
            & 0xff) as u32
    };
    let pixel = if outside && !m7.char_fill {
        0u32
    } else {
        ((frame
            .vram
            .get((tile * 64 + ((y_pos & 7) * 8 + (x_pos & 7)) as u32) as usize)
            .copied()
            .unwrap_or(0)
            >> 8)
            & 0xff) as u32
    };
    if pixel == 0 {
        None
    } else {
        Some(pixel as u8)
    }
}

/// Paint the Mode-7 BG (BG1) into `screen` (REPLACE). `main_tm`=Some enforces the
/// per-scanline BG1 enable (main call, `layer_bit=1`); `None` skips it (sub call,
/// `layer_bit=0`). Stamps math bit 0 (BG1).
fn paint_mode7_bg(
    screen: &mut Screen,
    frame: &crate::gpu_frame::GpuFrame<'_>,
    modern: &ModernFrame,
    main_tm: Option<&[u8]>,
    windowed: u8,
) {
    let width = usize::from(MODERN_FRAME_WIDTH);
    let height = usize::from(MODERN_FRAME_HEIGHT);
    for sy in 0..height {
        if let Some(tm) = main_tm {
            if tm[sy] & 0x01 == 0 {
                continue; // BG1 disabled on this scanline
            }
        }
        for sx in 0..width {
            if layer_window_masks(
                0,
                modern.windowsel,
                windowed,
                sx as u32,
                sy,
                &modern.window_scanlines,
            ) {
                continue;
            }
            if let Some(pixel) = mode7_bg_index(frame, sx, sy) {
                let color = modern.cgram_rgba[pixel as usize];
                let i = sy * width + sx;
                screen.c5[i] = [color[0] >> 3, color[1] >> 3, color[2] >> 3];
                screen.bit[i] = 0; // BG1
                screen.real[i] = true;
            }
        }
    }
}

/// Full Mode-7 frame render: composites in the classic Mode-7 z-order
/// (OBJ0 -> Mode7 BG -> OBJ1/2/3), then the shared color-math + brightness finalize.
/// BG decodes from live VRAM; sprites via the live-VRAM OAM path.
pub fn render_modern_mode7_frame(frame: &crate::gpu_frame::GpuFrame<'_>) -> Vec<u8> {
    let width = usize::from(MODERN_FRAME_WIDTH);
    let height = usize::from(MODERN_FRAME_HEIGHT);
    let len = width * height;

    let mut modern = crate::modern_extract::extract_modern_frame(frame);
    crate::modern_extract::fill_modern_cgram_colors(&mut modern, frame, true);

    if let Ok(value) = std::env::var("ZELDA3_DEBUG_MODE7_PIXEL") {
        let mut parts = value.split(',');
        if let (Some(Ok(sx)), Some(Ok(sy))) = (
            parts.next().map(str::parse::<usize>),
            parts.next().map(str::parse::<usize>),
        ) {
            if sx < usize::from(MODERN_FRAME_WIDTH) && sy < usize::from(MODERN_FRAME_HEIGHT) {
                let index = mode7_bg_index(frame, sx, sy).unwrap_or(0) as usize;
                let m = frame.scanlines[sy].mode7_matrix;
                let (m0, m1, m2, m3) = (m[0] as i32, m[1] as i32, m[2] as i32, m[3] as i32);
                let x_center = expand_m7_13(m[4] as i32);
                let y_center = expand_m7_13(m[5] as i32);
                let h_scroll = expand_m7_13(m[6] as i32);
                let v_scroll = expand_m7_13(m[7] as i32);
                let clipped_h = clip_m7_offset(h_scroll - x_center);
                let clipped_v = clip_m7_offset(v_scroll - y_center);
                let ry = sy as i32 + 1;
                let start_x = (m0 * clipped_h & -64)
                    + (m1 * ry & -64)
                    + (m1 * clipped_v & -64)
                    + (x_center << 8);
                let start_y = (m2 * clipped_h & -64)
                    + (m3 * ry & -64)
                    + (m3 * clipped_v & -64)
                    + (y_center << 8);
                let sample_x = ((start_x + m0 * sx as i32) >> 8) & 0x3ff;
                let sample_y = ((start_y + m2 * sx as i32) >> 8) & 0x3ff;
                let window = modern.window_scanlines.get(sy).copied().unwrap_or([0; 4]);
                let cm_window = in_cm_window(sx as u32, window, modern.windowsel_cm);
                let trace = format!(
                    "mode7_pixel xy=({sx},{sy}) src=({sample_x},{sample_y}) matrix={m:04x?} scroll=({h_scroll},{v_scroll}) center=({x_center},{y_center}) index={index:02x} cgram={:04x} rgba={:02x?} brightness={} math={:02x} sub={} subtract={} half={} fixed=({:02x},{:02x},{:02x}) window={window:02x?} cm_window={cm_window} prevent_math={} math_ok={}",
                    frame.cgram.get(index).copied().unwrap_or(0),
                    modern.cgram_rgba[index],
                    modern.brightness,
                    modern.math_enabled,
                    modern.add_subscreen,
                    modern.subtract_color,
                    modern.half_color,
                    modern.fixed_color_r,
                    modern.fixed_color_g,
                    modern.fixed_color_b,
                    modern.prevent_math_mode,
                    cw_bit(cm_window, modern.prevent_math_mode),
                );
                eprintln!("{trace}");
                use std::io::Write;
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open("/tmp/zelda3-mode7-pixel.trace")
                {
                    let _ = writeln!(file, "{trace}");
                }
            }
        }
    }

    if modern.forced_blank && modern.mode7_scanout_brightness_override.is_none() {
        let mut out = vec![0u8; len * 4];
        for px in out.chunks_exact_mut(4) {
            px.copy_from_slice(&[0, 0, 0, 0xff]);
        }
        return out;
    }

    let (sprite_cells, sprites) = crate::modern_extract::extract_modern_sprites_from_vram(frame);
    modern.index_sprites = sprites;

    let bd = &modern.backdrop_color_rgba;
    let backdrop_c5 = [bd[0] >> 3, bd[1] >> 3, bd[2] >> 3];
    // Mode-7 wiring of HD overrides is deferred (Phase 2); always disabled here so
    // Mode-7 output is unaffected.
    let obj = resolve_obj_layer(
        &modern,
        &sprite_cells,
        len,
        &crate::modern_hd_overrides::HdOverrideCtx::disabled(),
        width,
        1,
    );

    // MAIN: OBJ0 (behind BG) -> Mode7 BG -> OBJ 1..3 (matches render_mode7_frame).
    let mut main = Screen::new(backdrop_c5, len);
    paint_obj_priority(
        &mut main,
        &obj,
        0,
        Some(&modern.main_tm_scanlines),
        &modern,
        modern.screen_windowed_main,
        width,
        1,
    );
    paint_mode7_bg(
        &mut main,
        frame,
        &modern,
        Some(&modern.main_tm_scanlines),
        modern.screen_windowed_main,
    );
    for prio in 1..=3 {
        paint_obj_priority(
            &mut main,
            &obj,
            prio,
            Some(&modern.main_tm_scanlines),
            &modern,
            modern.screen_windowed_main,
            width,
            1,
        );
    }

    // SUB: Mode7 BG (only if BG1 sub-enabled; no per-scanline TM) then OBJ 0..3.
    let mut sub = Screen::new(backdrop_c5, len);
    if modern.screen_enabled_sub & 0x01 != 0 {
        paint_mode7_bg(&mut sub, frame, &modern, None, modern.screen_windowed_sub);
    }
    for prio in 0..=3 {
        paint_obj_priority(
            &mut sub,
            &obj,
            prio,
            None,
            &modern,
            modern.screen_windowed_sub,
            width,
            1,
        );
    }

    finalize_frame(&main, &sub, &modern, width, 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_frame::{
        ModernBgLayer, ModernBlendMode, ModernFrame, ModernIndexSpriteInstance,
        ModernIndexTileInstance, ModernTileInstance,
    };
    use crate::modern_index_atlas::ModernIndexTile;

    /// `layer_window_masks` mirrors the classic `bg_layer.wgsl::layer_window_active`:
    /// a windowed layer with W1 enabled and boundary [10,20] masks pixels INSIDE the
    /// window; not-windowed or W1-disabled never masks; the W1inv flag inverts.
    #[test]
    fn layer_window_masks_w1_and_inversion() {
        // 224 scanlines, window 1 = [10, 20] on every row; window 2 = [0,0].
        let scan = vec![[10u8, 20u8, 0u8, 0u8]; 224];
        // BG1 (layer 0), W1 enabled (bit1), no inversion: nibble 0 = 0b0010 = 0x2.
        let windowsel_w1 = 0x0000_0002u32;
        let windowed = 0x01u8; // TMW bit 0 (BG1)

        // Not windowed (TMW bit clear) → never masked, even inside the window.
        assert!(!layer_window_masks(0, windowsel_w1, 0x00, 15, 5, &scan));

        // Windowed + W1en, no inversion: inside [10,20] masked, outside not.
        assert!(layer_window_masks(0, windowsel_w1, windowed, 15, 5, &scan));
        assert!(layer_window_masks(0, windowsel_w1, windowed, 10, 5, &scan)); // left edge inclusive
        assert!(layer_window_masks(0, windowsel_w1, windowed, 20, 5, &scan)); // right edge inclusive
        assert!(!layer_window_masks(0, windowsel_w1, windowed, 9, 5, &scan));
        assert!(!layer_window_masks(0, windowsel_w1, windowed, 21, 5, &scan));

        // W1 enabled but W1inv set (nibble 0 = 0b0011 = 0x3): masking INVERTED.
        let windowsel_w1_inv = 0x0000_0003u32;
        assert!(!layer_window_masks(
            0,
            windowsel_w1_inv,
            windowed,
            15,
            5,
            &scan
        ));
        assert!(layer_window_masks(
            0,
            windowsel_w1_inv,
            windowed,
            9,
            5,
            &scan
        ));

        // W1 region set but neither W1en nor W2en → no masking (flags 0x0).
        assert!(!layer_window_masks(0, 0x0000_0000, windowed, 15, 5, &scan));

        // OBJ (layer 4) uses windowsel >> 16 and TMW bit 4 (0x10).
        let windowsel_obj = 0x0002_0000u32; // OBJ nibble (>>16) = W1en
        assert!(layer_window_masks(4, windowsel_obj, 0x10, 15, 5, &scan));
        assert!(!layer_window_masks(4, windowsel_obj, 0x00, 15, 5, &scan));
    }

    /// Build a frame with one BG layer (`layer_index`) carrying a single index-1
    /// tile at (0,0) whose color resolves to the given 5-bit channel triple.
    fn frame_with_single_bg_pixel(
        layer_index: u8,
        c5: [u8; 3],
    ) -> (ModernFrame, Vec<ModernIndexTile>) {
        let mut indices = [0u8; 64];
        indices[0] = 1; // pixel (0,0)
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];
        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        // Use palette `layer_index` so the two layers in the half-add test don't share a slot.
        let pal = layer_index as usize;
        frame.cgram_rgba[pal * 16 + 1] = [c5[0] << 3, c5[1] << 3, c5[2] << 3, 0xff];
        frame.bg_layers[layer_index as usize]
            .index_tiles
            .push(ModernIndexTileInstance {
                cell_id: 0,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                screen_x: 0,
                screen_y: 0,
                palette: layer_index,
                hflip: false,
                vflip: false,
                priority: false,
            });
        (frame, cells)
    }

    /// Per-scanline HDMA BG scroll: a layer whose `bg_scroll_scanlines` differs from
    /// its base scroll is re-sampled PER OUTPUT ROW. Two adjacent BG1 tiles (A at
    /// x0-7, B at x8-15); row 0 carries h-scroll +8 (so it samples 8px right → B),
    /// row 1 carries the base scroll (+0 → A). The same scene must therefore render a
    /// DIFFERENT pixel at column 0 on the two rows, proving the shift is per-scanline
    /// and that uniform rows still match the fast path.
    #[test]
    fn per_scanline_scroll_shifts_each_row_independently() {
        let mut indices = [0u8; 64];
        for v in indices.iter_mut() {
            *v = 1; // fully opaque 8x8 tile
        }
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];
        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.brightness = 15;
        frame.screen_enabled_main = 0x01; // BG1 on main, no color math
                                          // A = red (palette 0), B = green (palette 1).
        frame.cgram_rgba[1] = [20 << 3, 0, 0, 0xff];
        frame.cgram_rgba[16 + 1] = [0, 20 << 3, 0, 0xff];
        let bg1 = &mut frame.bg_layers[0];
        bg1.scroll_x = 0;
        bg1.scroll_y = 0;
        bg1.wrap_w = 256;
        bg1.wrap_h = 256;
        bg1.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        bg1.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 8,
            screen_y: 0,
            palette: 1,
            hflip: false,
            vflip: false,
            priority: false,
        });
        // Per-scanline scroll: row 0 = h+8 (samples B at column 0), all others = base.
        let mut scan = vec![[[0u16; 2]; 4]; usize::from(MODERN_FRAME_HEIGHT)];
        scan[0][0] = [8, 0];
        frame.bg_scroll_scanlines = scan;

        let out = render_modern_frame_full(&frame, &cells, &[]);
        // c5 20, brightness 15 → (20<<3)|(20>>2) = 165.
        let row0 = (0 * 256 + 0) * 4;
        let row1 = (1 * 256 + 0) * 4;
        assert_eq!(
            &out[row0..row0 + 3],
            &[0, 165, 0],
            "row 0 (+8 h-scroll) samples tile B (green) at column 0"
        );
        assert_eq!(
            &out[row1..row1 + 3],
            &[165, 0, 0],
            "row 1 (base scroll) samples tile A (red) at column 0"
        );
    }

    #[test]
    fn uniform_scroll_wraps_tiles_past_screen_edge() {
        let mut indices = [0u8; 64];
        for v in indices.iter_mut() {
            *v = 1;
        }
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];
        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.brightness = 15;
        frame.screen_enabled_main = 0x01;
        frame.cgram_rgba[1] = [20 << 3, 0, 0, 0xff];

        let bg1 = &mut frame.bg_layers[0];
        bg1.scroll_x = 6;
        bg1.scroll_y = 0;
        bg1.wrap_w = 256;
        bg1.wrap_h = 256;
        bg1.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 250,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });

        let out = render_modern_frame_full(&frame, &cells, &[]);
        let left_wrapped = (0usize * 256 + 0) * 4;

        assert_eq!(
            &out[left_wrapped..left_wrapped + 3],
            &[165, 0, 0],
            "uniformly scrolled BG tiles that cross the right edge must wrap to the left"
        );
    }

    /// OBJ-vs-OBJ priority is by OAM INDEX (lowest wins), NOT the priority attribute.
    /// A lower-index sprite with a LOWER priority attribute must still win over a
    /// higher-index sprite with a HIGHER priority attribute where they overlap.
    #[test]
    fn obj_vs_obj_priority_is_by_index_not_attribute() {
        let mut indices = [0u8; 64];
        indices[0] = 1; // pixel (0,0) opaque for both
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];
        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        // A = palette 4 (red), B = palette 5 (green). Distinct colors.
        frame.cgram_rgba[0x80 + 4 * 16 + 1] = [31 << 3, 0, 0, 0xff];
        frame.cgram_rgba[0x80 + 5 * 16 + 1] = [0, 31 << 3, 0, 0xff];
        let spr = |pal: u8, prio: u8| ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: pal,
            priority: prio,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        };
        // A first => lower OAM index, priority attr 0 (lowest). B second => higher
        // index, priority attr 3 (highest). A must win the overlap.
        frame.index_sprites.push(spr(4, 0)); // A
        frame.index_sprites.push(spr(5, 3)); // B
        frame.screen_enabled_main = 0x10; // OBJ on main
        frame.brightness = 15;
        let out = render_modern_frame_full(&frame, &[], &cells);
        assert_eq!(
            &out[0..3],
            &[255, 0, 0],
            "lower-index A (red) wins, not higher-prio B"
        );
    }

    /// OBJ color-math is gated by palette: OBJ palettes 0-3 use a non-math layer
    /// designation (bit 6, never in `math_enabled`) and are NEVER subtracted/added;
    /// palettes 4-7 (bit 4) participate. Guards the glow-sprite over-subtract bug.
    #[test]
    fn obj_color_math_only_applies_to_palettes_4_to_7() {
        let mut indices = [0u8; 64];
        indices[0] = 1;
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];
        let make = |pal: u8| {
            let mut frame = ModernFrame::empty();
            frame.backdrop_color_rgba = [0, 0, 0, 0xff];
            frame.cgram_rgba[0x80 + pal as usize * 16 + 1] = [20 << 3, 20 << 3, 20 << 3, 0xff];
            frame.index_sprites.push(ModernIndexSpriteInstance {
                cell_id: 0,
                screen_x: 0,
                screen_y: 0,
                palette: pal,
                priority: 0,
                hflip: false,
                vflip: false,
                row_mask: 0xff,
            });
            frame.screen_enabled_main = 0x10; // OBJ on main
            frame.math_enabled = 0x10; // OBJ math layer (bit 4) enabled
            frame.subtract_color = true;
            frame.fixed_color_r = 4;
            frame.fixed_color_g = 4;
            frame.fixed_color_b = 4;
            frame.brightness = 15;
            frame
        };
        // palette 2 (<4): unmathed → 5-bit 20 → (20<<3)|(20>>2) = 165.
        let out_lo = render_modern_frame_full(&make(2), &[], &cells);
        assert_eq!(
            &out_lo[0..3],
            &[165, 165, 165],
            "palette<4 sprite must not be mathed"
        );
        // palette 5 (>=4): subtracted by 4 → 5-bit 16 → (16<<3)|(16>>2) = 132.
        let out_hi = render_modern_frame_full(&make(5), &[], &cells);
        assert_eq!(
            &out_hi[0..3],
            &[132, 132, 132],
            "palette>=4 sprite must be subtracted"
        );
    }

    #[test]
    fn color_math_half_add_matches_formula() {
        // Main BG (layer 0) = (20,20,20); sub BG (layer 1) = (10,10,10).
        let (mut frame, cells) = frame_with_single_bg_pixel(0, [20, 20, 20]);
        let (sub_frame, _) = frame_with_single_bg_pixel(1, [10, 10, 10]);
        // Merge the sub layer's tile + palette into the frame.
        frame.bg_layers[1].index_tiles = sub_frame.bg_layers[1].index_tiles.clone();
        frame.cgram_rgba[1 * 16 + 1] = sub_frame.cgram_rgba[1 * 16 + 1];

        frame.screen_enabled_main = 0x01; // BG1 on main
        frame.screen_enabled_sub = 0x02; // BG2 on sub
        frame.math_enabled = 0x01; // math on for BG1 (the winning main layer, bit 0)
        frame.add_subscreen = true;
        frame.half_color = true;
        frame.subtract_color = false;
        frame.fixed_color_r = 0;
        frame.fixed_color_g = 0;
        frame.fixed_color_b = 0;
        frame.brightness = 15;

        let out = render_modern_frame_full(&frame, &cells, &[]);
        // r5 = (20 + 10) >> 1 = 15. Brightness 15 → v8 = (15<<3)|(15>>2) = 123,
        // out8 = 123*15/15 = 123. (The classic shader's low-bit fill makes this 123,
        // not the 120 of the task's no-fill paraphrase.)
        assert_eq!(&out[0..4], &[123, 123, 123, 0xff], "half-add pixel (0,0)");
    }

    #[test]
    fn color_math_half_add_matches_snes9x_rgb565_green_rounding() {
        assert_eq!((20 + 21) >> 1, 20);
        assert_eq!(snes9x_rgb565_half_add_green(20, 21), 21);
    }

    #[test]
    fn color_math_adds_subscreen_to_backdrop_main_pixel() {
        // Regression for the BLACK dungeon room: the MAIN pixel is the BACKDROP
        // (no BG/sprite on main → math bit 5), the room BG1 is on the SUB screen,
        // and math_enabled bit 5 + add_subscreen composites the sub room onto the
        // backdrop. Mirrors post_process.wgsl, which encodes backdrop as alpha=5/255
        // and checks `(math_enabled >> 5) & 1`.
        let (mut frame, cells) = frame_with_single_bg_pixel(1, [10, 10, 10]);
        // Backdrop = cgram[0] = (4,4,4) in 5-bit → [32,32,32] rgba.
        frame.backdrop_color_rgba = [4 << 3, 4 << 3, 4 << 3, 0xff];
        frame.screen_enabled_main = 0x00; // nothing on main → backdrop everywhere
        frame.screen_enabled_sub = 0x02; // BG2 (the room) on sub
        frame.math_enabled = 0x20; // bit 5 = backdrop participates in math
        frame.add_subscreen = true;
        frame.half_color = false;
        frame.subtract_color = false;
        frame.fixed_color_r = 0;
        frame.fixed_color_g = 0;
        frame.fixed_color_b = 0;
        frame.brightness = 15;

        let out = render_modern_frame_full(&frame, &cells, &[]);
        // Pixel (0,0): main = backdrop (4), sub = room (10) → 4 + 10 = 14.
        // brightness 15 → v8 = (14<<3)|(14>>2) = 115.
        assert_eq!(
            &out[0..4],
            &[115, 115, 115, 0xff],
            "backdrop main + sub room composited via color math"
        );
        // A pixel with NO sub room pixel stays pure backdrop: 4 → v8 = (4<<3)|(4>>2)=33.
        let nb = (0 * 256 + 1) * 4;
        assert_eq!(
            &out[nb..nb + 4],
            &[33, 33, 33, 0xff],
            "backdrop pixel with no sub stays backdrop"
        );
    }

    #[test]
    fn mode7_scanout_brightness_owns_color_math_generation() {
        let (mut frame, cells) = frame_with_single_bg_pixel(0, [27, 28, 30]);
        frame.screen_enabled_main = 0x01;
        frame.math_enabled = 0x01;
        frame.add_subscreen = false;
        frame.fixed_color_r = 5;
        frame.fixed_color_g = 5;
        frame.fixed_color_b = 5;
        frame.brightness = 0;
        frame.mode7_scanout_brightness_override = Some(1);

        let out = render_modern_frame_full(&frame, &cells, &[]);

        assert_eq!(&out[0..4], &[16, 16, 16, 0xff]);
    }

    #[test]
    fn low_brightness_full_add_caps_at_brightness_mapped_white() {
        // Snes9x's `COLOR_ADD_BRIGHTNESS` inputs measured at the world-map
        // transition: main [5,11,6], subscreen [14,22,30], brightness 8.
        let (mut frame, cells) = frame_with_single_bg_pixel(0, [5, 11, 6]);
        let (sub_frame, _) = frame_with_single_bg_pixel(1, [14, 22, 30]);
        frame.bg_layers[1].index_tiles = sub_frame.bg_layers[1].index_tiles.clone();
        frame.cgram_rgba[1 * 16 + 1] = sub_frame.cgram_rgba[1 * 16 + 1];

        frame.screen_enabled_main = 0x01;
        frame.screen_enabled_sub = 0x02;
        frame.math_enabled = 0x01;
        frame.add_subscreen = true;
        frame.half_color = false;
        frame.subtract_color = false;
        frame.brightness = 8;

        let out = render_modern_frame_full(&frame, &cells, &[]);
        // Scaled operands sum to [10,18,19], while brightness-mapped white is
        // 17. Snes9x therefore emits [10,17,17] before five-bit expansion.
        assert_eq!(&out[0..4], &[82, 140, 140, 0xff]);
    }

    #[test]
    fn brightness_only_scales_channels() {
        let (mut frame, cells) = frame_with_single_bg_pixel(0, [31, 31, 31]);
        frame.screen_enabled_main = 0x01;
        frame.math_enabled = 0; // no math
        frame.brightness = 8;

        let out = render_modern_frame_full(&frame, &cells, &[]);
        // Brightness is quantized in the 5-bit DAC domain before RGB expansion:
        // floor(31*9/16) = 17, and 17 expands to 140.
        assert_eq!(
            &out[0..4],
            &[140, 140, 140, 0xff],
            "brightness-only pixel (0,0)"
        );
    }

    #[test]
    fn disabled_overrides_match_plain_full_render() {
        use crate::modern_hd_overrides::HdOverrideCtx;
        let (mut frame, cells) = frame_with_single_bg_pixel(0, [31, 31, 31]);
        frame.screen_enabled_main = 0x01;
        frame.math_enabled = 0;
        frame.brightness = 8;

        let plain = render_modern_frame_full(&frame, &cells, &[]);
        let disabled = render_modern_frame_full_with_overrides(
            &frame,
            &cells,
            &[],
            &HdOverrideCtx::disabled(),
        );
        assert_eq!(plain, disabled);
    }

    #[test]
    fn no_effect_math_leaves_channel_unchanged_except_brightness() {
        // Math bit set for the winning layer, but fixed=0, half=0, no sub →
        // no_effect_math = true → channel untouched, only brightness applied.
        let (mut frame, cells) = frame_with_single_bg_pixel(0, [20, 20, 20]);
        frame.screen_enabled_main = 0x01;
        frame.screen_enabled_sub = 0; // no sub composite
        frame.math_enabled = 0x01; // math "on" for BG1 but ineffective
        frame.half_color = false;
        frame.add_subscreen = false;
        frame.fixed_color_r = 0;
        frame.fixed_color_g = 0;
        frame.fixed_color_b = 0;
        frame.brightness = 15;

        let out = render_modern_frame_full(&frame, &cells, &[]);
        // Unchanged channel: v8 = (20<<3)|(20>>2) = 165; out8 = 165*15/15 = 165.
        assert_eq!(
            &out[0..4],
            &[165, 165, 165, 0xff],
            "no-effect-math pixel (0,0)"
        );
    }

    #[test]
    fn color_math_gated_off_outside_color_window() {
        // Math WOULD add the fixed color, but prevent_math_mode=1 (follows window)
        // and the color window (x in 10..=20) excludes pixel (0,0) → math is gated
        // OFF there, so the main channel is unchanged (only brightness applies).
        let (mut frame, cells) = frame_with_single_bg_pixel(0, [20, 20, 20]);
        frame.screen_enabled_main = 0x01;
        frame.screen_enabled_sub = 0;
        frame.math_enabled = 0x01; // BG1 participates in math
        frame.add_subscreen = false;
        frame.half_color = false;
        frame.subtract_color = false;
        frame.fixed_color_r = 5; // non-zero → math is "effective" if allowed
        frame.fixed_color_g = 5;
        frame.fixed_color_b = 5;
        frame.brightness = 15;
        // CGWSEL: prevent math OUTSIDE the window (mode 1, follows window).
        frame.prevent_math_mode = 1;
        frame.clip_mode = 0; // never clip
        frame.windowsel_cm = 0x02; // W1 enabled, not inverted
        frame.window_scanlines[0] = [10, 20, 0, 0]; // window covers x 10..=20

        let out = render_modern_frame_full(&frame, &cells, &[]);
        // (0,0) is outside the window → no math → unchanged 20 → v8=165, out=165.
        assert_eq!(
            &out[0..4],
            &[165, 165, 165, 0xff],
            "pixel (0,0) outside color window: math gated off"
        );
        // A pixel inside the window (x=12,y=0) is backdrop here (no tile), but its
        // window membership is exercised via the clip path staying unclipped.
        let inside = (0 * 256 + 12) * 4;
        assert_eq!(
            &out[inside..inside + 4],
            &[0, 0, 0, 0xff],
            "in-window backdrop pixel still renders (not clipped)"
        );
    }

    /// Mosaic source-snap: a single BG1 tile carrying a 2-D index gradient
    /// (index = sy*8 + sx + 1) with mosaic_size=4 must snap every pixel within a
    /// 4×4 block to the block-origin pixel's color, so pixels in the same block are
    /// equal and pixels in different blocks differ.
    #[test]
    fn mosaic_snaps_bg_to_block_origin() {
        // 8×8 cell: distinct palette index per pixel (1..64), so each source pixel
        // resolves to a distinct color.
        let mut indices = [0u8; 64];
        for (k, slot) in indices.iter_mut().enumerate() {
            *slot = (k + 1) as u8;
        }
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        // Distinct color per index (palette 0). Spread k across the channels' UPPER
        // bits so the compositor's `>>3` (8-bit → 5-bit) keeps them distinct.
        for k in 1..=64usize {
            let k = k as u8;
            frame.cgram_rgba[k as usize] =
                [(k & 7) << 5, ((k >> 3) & 7) << 5, ((k >> 6) & 3) << 6, 0xff];
        }
        frame.bg_layers[0]
            .index_tiles
            .push(ModernIndexTileInstance {
                cell_id: 0,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                screen_x: 0,
                screen_y: 0,
                palette: 0,
                hflip: false,
                vflip: false,
                priority: false,
            });
        frame.screen_enabled_main = 0x01; // BG1 on main
        frame.brightness = 15;
        frame.mosaic_enabled = 0x01; // BG1 mosaiced
        frame.mosaic_size = 4; // 4×4 blocks

        let out = render_modern_frame_full(&frame, &cells, &[]);
        let at = |x: usize, y: usize| {
            let o = (y * 256 + x) * 4;
            [out[o], out[o + 1], out[o + 2]]
        };
        // Block (0,0): origin (0,0). Every pixel in 0..3 × 0..3 equals (0,0).
        assert_eq!(at(2, 1), at(0, 0), "block (0,0) pixel snaps to origin");
        assert_eq!(at(3, 3), at(0, 0), "block (0,0) corner snaps to origin");
        // Block (4,0): origin (4,0).
        assert_eq!(at(6, 3), at(4, 0), "block (4,0) pixel snaps to origin");
        // Block (0,4): origin (0,4).
        assert_eq!(at(1, 6), at(0, 4), "block (0,4) pixel snaps to origin");
        // Different blocks resolve to different colors (snap actually moved color).
        assert_ne!(at(2, 1), at(6, 3), "different blocks differ");
        assert_ne!(at(2, 1), at(1, 6), "different blocks differ");
        // And the snapped pixel is NOT its own un-snapped color (index at (2,1) is
        // 1*8+2+1=11; origin index is 1 → colors differ).
        assert_ne!(
            at(2, 1),
            {
                let c = frame.cgram_rgba[11];
                [
                    expand_brightness(i32::from(c[0]), 15),
                    expand_brightness(i32::from(c[1]), 15),
                    expand_brightness(i32::from(c[2]), 15),
                ]
            },
            "mosaic actually replaced the pixel's own color"
        );
    }

    #[test]
    fn software_sprites_indexed_draw_over_bg_with_obj_cgram() {
        // Sprite cell 0: only pixel (0,0) is index 1; everything else transparent.
        let mut indices = [0u8; 64];
        indices[0] = 1;
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        // OBJ palette base is 0x80; palette 3, index 1.
        frame.cgram_rgba[0x80 + 3 * 16 + 1] = [200, 10, 20, 0xff];
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 5,
            screen_y: 7,
            palette: 3,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });

        // BG is empty (no index_tiles) → backdrop only, then sprites composite over.
        let mut out = render_modern_frame_software_indexed(&frame, &[]);
        draw_modern_sprites_indexed(&mut out, &frame, &sprite_cells);

        let px = (7usize * 256 + 5) * 4;
        assert_eq!(&out[px..px + 4], &[200, 10, 20, 0xff], "sprite pixel (5,7)");
        // Neighbour stays backdrop (index 0 transparent).
        let nb = (7usize * 256 + 6) * 4;
        assert_eq!(&out[nb..nb + 4], &[0, 0, 0, 0xff], "neighbour backdrop");
    }

    #[test]
    fn software_sprites_indexed_apply_hflip() {
        // Sprite cell 0: only pixel (7,0) is index 2.
        let mut indices = [0u8; 64];
        indices[7] = 2;
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.cgram_rgba[0x80 + 3 * 16 + 2] = [9, 99, 199, 0xff];
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 5,
            screen_y: 7,
            palette: 3,
            priority: 0,
            hflip: true,
            vflip: false,
            row_mask: 0xff,
        });

        let mut out = render_modern_frame_software_indexed(&frame, &[]);
        draw_modern_sprites_indexed(&mut out, &frame, &sprite_cells);

        // hflip: source pixel (7,0) lands at screen x = screen_x + 0.
        let px = (7usize * 256 + 5) * 4;
        assert_eq!(
            &out[px..px + 4],
            &[9, 99, 199, 0xff],
            "hflipped pixel at x=5"
        );
    }

    #[test]
    fn software_indexed_renderer_applies_live_cgram() {
        // Synthetic atlas: one cell (id=0) — all indices zero except (0,0)->1 and (1,0)->2.
        let mut indices = [0u8; 64];
        indices[0] = 1; // pixel (0,0): sx=0, sy=0 → indices[sy*8+sx]=indices[0]
        indices[1] = 2; // pixel (1,0): sx=1, sy=0 → indices[sy*8+sx]=indices[1]
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        // Frame: palette P=3 so that the palette offset (not P=0) is exercised.
        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.cgram_rgba[3 * 16 + 1] = [10, 20, 30, 0xff]; // palette 3, index 1
        frame.cgram_rgba[3 * 16 + 2] = [40, 50, 60, 0xff]; // palette 3, index 2

        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 3,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = layer;

        let rgba = render_modern_frame_software_indexed(&frame, &cells);

        // pixel (0,0): index 1, palette 3 → cgram_rgba[3*16+1]
        let px00 = (0 * 256 + 0) * 4;
        assert_eq!(&rgba[px00..px00 + 4], &[10, 20, 30, 0xff], "pixel (0,0)");
        // pixel (1,0): index 2, palette 3 → cgram_rgba[3*16+2]
        let px10 = (0 * 256 + 1) * 4;
        assert_eq!(&rgba[px10..px10 + 4], &[40, 50, 60, 0xff], "pixel (1,0)");
        // pixel (2,0): index 0 → transparent → backdrop
        let px20 = (0 * 256 + 2) * 4;
        assert_eq!(
            &rgba[px20..px20 + 4],
            &[0, 0, 0, 0xff],
            "pixel (2,0) should be backdrop"
        );
    }

    #[test]
    fn software_renderer_blits_one_opaque_tile_from_atlas() {
        let mut atlas = vec![0u8; 8 * 8 * 4];
        for px in atlas.chunks_exact_mut(4) {
            px.copy_from_slice(&[10, 20, 30, 0xff]);
        }
        let mut frame = ModernFrame::empty();
        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.blend_mode = ModernBlendMode::Opaque;
        layer.tiles.push(ModernTileInstance {
            atlas_id: 0,
            atlas_x_px: 0,
            atlas_y_px: 0,
            atlas_width_px: 8,
            atlas_height_px: 8,
            screen_width_px: 8,
            screen_height_px: 8,
            screen_x: 4,
            screen_y: 5,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: false,
            transparent_color_zero: false,
        });
        frame.bg_layers[0] = layer;

        let rgba = render_modern_frame_software(&frame, &atlas, 8, 8);
        let offset = ((5usize * 256) + 4usize) * 4;

        assert_eq!(&rgba[offset..offset + 4], &[10, 20, 30, 0xff]);
    }

    /// A pixel-distinct 8x8 pattern: color depends on BOTH x and y so the test
    /// catches scale being ignored, x/y swaps, and mirror mistakes.
    fn pattern_8x8(x: usize, y: usize) -> [u8; 4] {
        [(x as u8) * 30 + 5, (y as u8) * 30 + 7, 100, 0xff]
    }

    #[test]
    fn software_renderer_downsamples_scaled_atlas_tile() {
        const SCALE: usize = 4;
        const SRC: usize = 8 * SCALE; // 32
                                      // Build a 32x32 atlas that is a 4x nearest upscale of the 8x8 pattern.
        let mut atlas = vec![0u8; SRC * SRC * 4];
        for ay in 0..SRC {
            for ax in 0..SRC {
                let px = pattern_8x8(ax / SCALE, ay / SCALE);
                let o = (ay * SRC + ax) * 4;
                atlas[o..o + 4].copy_from_slice(&px);
            }
        }

        let mut frame = ModernFrame::empty();
        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.blend_mode = ModernBlendMode::Opaque;
        layer.tiles.push(ModernTileInstance {
            atlas_id: 0,
            atlas_x_px: 0,
            atlas_y_px: 0,
            atlas_width_px: SRC as u16,
            atlas_height_px: SRC as u16,
            screen_width_px: 8,
            screen_height_px: 8,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: false,
            transparent_color_zero: false,
        });
        frame.bg_layers[0] = layer;

        let rgba = render_modern_frame_software(&frame, &atlas, SRC as u16, SRC as u16);

        // The 8x8 footprint at screen (0,0) must equal the original 8x8 pattern.
        for y in 0..8usize {
            for x in 0..8usize {
                let o = (y * 256 + x) * 4;
                assert_eq!(
                    &rgba[o..o + 4],
                    &pattern_8x8(x, y),
                    "screen pixel ({x},{y}) should equal downsampled pattern"
                );
            }
        }

        // Pixels just outside the 8x8 footprint must remain backdrop (not 32x32).
        let backdrop = frame.backdrop_color_rgba;
        for &(x, y) in &[(8usize, 0usize), (0, 8), (8, 8), (20, 20)] {
            let o = (y * 256 + x) * 4;
            assert_eq!(&rgba[o..o + 4], &backdrop, "({x},{y}) should be backdrop");
        }
    }

    /// Smallest full-path fixture with ONE BG index tile (BG1, palette 3, index 1,
    /// opaque at screen (0,0)) and ONE index sprite (palette 5, index 1, opaque at
    /// screen (50,60)), each cell carrying a distinct real `source_key`. Live palette
    /// channels ([16,32,48] BG / [48,16,32] sprite) are each a `2 * c5` value with
    /// `c5 < 4` (0..3), so `expand_brightness(c5, 15) == c5*8` exactly (no low-bit
    /// fill spills into the zeroed low 3 bits) — halving these channels stays exact
    /// through the render's 8-bit -> 5-bit -> 8-bit round trip, not just in raw
    /// arithmetic. `screen_enabled_main` carries BG1 (bit0) + OBJ (bit4); no
    /// color-math/windows/mosaic, so `render_modern_frame_full`'s only per-pixel
    /// transform is that quantize/expand.
    fn tiny_bg_and_sprite_fixture() -> (
        ModernFrame,
        Vec<ModernIndexTile>,
        Vec<ModernIndexTile>,
        (usize, usize, u8),
        (usize, usize, u8),
    ) {
        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.screen_enabled_main = 0x11; // BG1 (bit0) + OBJ (bit4)

        // BG: one opaque index-1 pixel at screen (0,0), palette 3.
        let mut bg_indices = [0u8; 64];
        bg_indices[0] = 1; // pixel (0,0)
        let bg_cells = vec![ModernIndexTile {
            id: 0,
            indices: bg_indices,
            source_key: 0x0000_0001_0000_0000,
            hflip: false,
            vflip: false,
        }];
        let bg_cgram_idx = 3 * 16 + 1;
        frame.cgram_rgba[bg_cgram_idx] = [16, 32, 48, 0xff];
        frame.bg_layers[0]
            .index_tiles
            .push(ModernIndexTileInstance {
                cell_id: 0,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                screen_x: 0,
                screen_y: 0,
                palette: 3,
                hflip: false,
                vflip: false,
                priority: false,
            });
        let bg_probe = (0usize, bg_cgram_idx, 1u8); // screen (0,0) -> byte offset 0

        // Sprite: one opaque index-1 pixel at screen (50,60), palette 5.
        let mut spr_indices = [0u8; 64];
        spr_indices[0] = 1; // pixel (0,0) of the sprite cell
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices: spr_indices,
            source_key: 0x0000_0002_0000_0000,
            hflip: false,
            vflip: false,
        }];
        let spr_cgram_idx = 0x80 + 5 * 16 + 1;
        frame.cgram_rgba[spr_cgram_idx] = [48, 16, 32, 0xff];
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 50,
            screen_y: 60,
            palette: 5,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });
        let spr_probe = ((60usize * 256 + 50) * 4, spr_cgram_idx, 1u8);

        (frame, bg_cells, sprite_cells, bg_probe, spr_probe)
    }

    /// End-to-end: a source-keyed HD override authored as the reference palette
    /// (detail 1) renders byte-identical to the no-override render for BOTH a BG
    /// index tile and an index sprite; an override authored as half the reference
    /// recolors the probed BG and sprite pixels to exactly half their live value.
    #[test]
    fn source_keyed_overrides_recolor_bg_and_sprite() {
        use crate::modern_hd_overrides::{HdCell, HdOverrideCtx, ModernHdOverrides};
        use std::collections::HashMap;

        let (frame, bg_cells, sprite_cells, bg_probe, spr_probe) = tiny_bg_and_sprite_fixture();

        let bg_key = bg_cells[0].source_key;
        let spr_key = sprite_cells[0].source_key;
        assert_ne!(bg_key, crate::modern_hd_overrides::NO_SOURCE_KEY);
        assert_ne!(spr_key, crate::modern_hd_overrides::NO_SOURCE_KEY);

        let plain = render_modern_frame_full(&frame, &bg_cells, &sprite_cells);

        // (a) reference == the frame's live palette, HD art == reference -> detail 1
        // -> byte-identical to the plain (no-override) render.
        let mut ref_pal = [[0u8; 4]; 256];
        for (i, e) in ref_pal.iter_mut().enumerate() {
            *e = frame.cgram_rgba[i];
        }
        let hd_identity = |cgram_idx: usize| {
            let c = ref_pal[cgram_idx];
            HdCell {
                width: 8,
                height: 8,
                rgba: vec![c[0], c[1], c[2], 0xff].repeat(64),
            }
        };
        let mut by_key = HashMap::new();
        by_key.insert(bg_key, hd_identity(bg_probe.1));
        by_key.insert(spr_key, hd_identity(spr_probe.1));
        let store_identity = ModernHdOverrides::from_parts(by_key, ref_pal);
        let identity = render_modern_frame_full_with_overrides(
            &frame,
            &bg_cells,
            &sprite_cells,
            &HdOverrideCtx::new(&store_identity),
        );
        assert_eq!(
            identity, plain,
            "detail=1 override must match no-override render"
        );

        // (b) HD art = half the reference -> live halved at the probed pixels.
        let hd_half = |cgram_idx: usize| {
            let c = ref_pal[cgram_idx];
            HdCell {
                width: 8,
                height: 8,
                rgba: vec![c[0] / 2, c[1] / 2, c[2] / 2, 0xff].repeat(64),
            }
        };
        let mut by_key2 = HashMap::new();
        by_key2.insert(bg_key, hd_half(bg_probe.1));
        by_key2.insert(spr_key, hd_half(spr_probe.1));
        let store_half = ModernHdOverrides::from_parts(by_key2, ref_pal);
        let recolored = render_modern_frame_full_with_overrides(
            &frame,
            &bg_cells,
            &sprite_cells,
            &HdOverrideCtx::new(&store_half),
        );

        for probe in [bg_probe, spr_probe] {
            let (off, cgram_idx, _base) = probe;
            let live = ref_pal[cgram_idx];
            let expected = [live[0] / 2, live[1] / 2, live[2] / 2];
            assert_eq!(&recolored[off..off + 3], &expected, "recolor at {off}");
            assert_ne!(
                &recolored[off..off + 3],
                &plain[off..off + 3],
                "must differ from plain"
            );
        }
    }

    #[test]
    fn variant_atlas_software_draws_vwf_glyph_runs_from_source_png_entries() {
        use crate::modern_frame::ModernVwfGlyphRun;
        use crate::modern_variant_atlas::{
            dialogue_text_color_palette_name, dialogue_vwf_variant_key, DialogueVwfGlyphAtlas,
            DialogueVwfGlyphCell, ModernVariantAtlas, VariantAtlasEntry,
            DIALOGUE_TEXT_MAIN_PALETTE,
        };

        fn vwf_entry(code: u16, quadrant: u16, palette: &str, rect: [u32; 4]) -> VariantAtlasEntry {
            VariantAtlasEntry {
                id: format!("dialogue_vwf:kDialogueVwfGlyphs:pack{code}:tile{quadrant}:2bpp"),
                key: dialogue_vwf_variant_key(code, quadrant, palette),
                rect,
                sha1: format!("test:{quadrant}:{palette}"),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: None,
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }
        }

        let mut rgba = vec![0u8; 16 * 16 * 4];
        let main_src = (3 * 16 + 2) * 4;
        rgba[main_src..main_src + 4].copy_from_slice(&[248, 248, 248, 255]);
        let color_src = (3 * 16 + 10) * 4;
        rgba[color_src..color_src + 4].copy_from_slice(&[248, 64, 64, 255]);
        let color_palette = dialogue_text_color_palette_name(7);
        let mut entries = (0..4)
            .map(|quadrant| {
                vwf_entry(
                    0x41,
                    quadrant,
                    DIALOGUE_TEXT_MAIN_PALETTE,
                    [
                        u32::from(quadrant & 1) * 8,
                        u32::from(quadrant >> 1) * 8,
                        8,
                        8,
                    ],
                )
            })
            .collect::<Vec<_>>();
        entries.push(vwf_entry(0x41, 0, &color_palette, [8, 0, 8, 8]));
        let atlas = ModernVariantAtlas {
            width: 16,
            height: 16,
            rgba,
            entries,
            effects: Vec::new(),
            mode7_source_chars: None,
            dialogue_glyph_atlas: None,
            dialogue_vwf_font: None,
            dialogue_vwf_glyph_atlas: Some(DialogueVwfGlyphAtlas {
                width: 16,
                height: 16,
                rgba: Vec::new(),
                glyphs: vec![DialogueVwfGlyphCell {
                    code: 0x41,
                    hex: "41".to_string(),
                    width: 8,
                    palette: DIALOGUE_TEXT_MAIN_PALETTE.to_string(),
                    rect: [0, 0, 16, 16],
                    indices: Vec::new(),
                }],
            }),
        };
        let mut frame = ModernFrame::empty();
        frame.bg3_vwf_glyph_runs.push(ModernVwfGlyphRun {
            glyph_code: 0xee,
            screen_x: 40,
            screen_y: 40,
            width: 8,
            dialogue_offset: None,
            dialogue_ir_kind: None,
            dialogue_color: None,
        });
        frame
            .dialogue_layout_vwf_glyph_runs
            .push(ModernVwfGlyphRun {
                glyph_code: 0x41,
                screen_x: 10,
                screen_y: 20,
                width: 8,
                dialogue_offset: Some(0x12),
                dialogue_ir_kind: Some(zelda3_dialogue::DialogueIrKind::Glyph { code: 0x41 }),
                dialogue_color: Some(7),
            });

        let (out, stats) = render_modern_frame_software_variant_atlas(
            &frame,
            &[],
            &[],
            &atlas,
            "unused",
            "unused",
        );

        let dst = ((20 + 3) * 256 + (10 + 2)) * 4;
        assert_eq!(&out[dst..dst + 4], &[248, 64, 64, 255]);
        assert_eq!(stats.stable_preview_draws, 4);
        assert_eq!(stats.stable_draws, 4);
    }

    #[test]
    fn variant_atlas_software_matches_indexed_bg_tile() {
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{ModernVariantAtlas, VariantAtlasEntry, VariantAtlasKey};

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.bg_layers[0].enabled_main = true;
        frame.cgram_rgba[3 * 16 + 1] = [16, 32, 48, 0xff];
        frame.bg_layers[0]
            .index_tiles
            .push(ModernIndexTileInstance {
                cell_id: 0,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                screen_x: 0,
                screen_y: 0,
                palette: 3,
                hflip: false,
                vflip: false,
                priority: false,
            });
        let mut indices = [0u8; 64];
        indices[0] = 1;
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];
        let mut atlas_rgba = vec![0u8; 8 * 8 * 4];
        atlas_rgba[0..4].copy_from_slice(&[16, 32, 48, 0xff]);
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: atlas_rgba,
            entries: vec![VariantAtlasEntry {
                id: "bg:kBgGfx:pack0:tile0:3bpp:palette_main_spr:row3".to_string(),
                key: VariantAtlasKey {
                    source_kind: "bg".to_string(),
                    asset: "kBgGfx".to_string(),
                    pack: 0,
                    tile: 0,
                    bpp: 3,
                    palette: "palette_main_spr".to_string(),
                    palette_row: 3,
                },
                rect: [0, 0, 8, 8],
                sha1: "test".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
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

        let indexed = render_modern_frame_software_indexed(&frame, &cells);
        let (variant, stats) = render_modern_frame_software_variant_atlas(
            &frame,
            &cells,
            &[],
            &atlas,
            "palette_main_spr",
            "palette_main_spr",
        );

        assert_eq!(variant, indexed);
        assert_eq!(stats.stable_draws, 1);
        assert_eq!(stats.fallback_draws, 0);
        assert_eq!(stats.missing_variant_draws, 0);
        assert_eq!(stats.dynamic_palette_draws, 0);
        assert_eq!(stats.stable_preview_draws, 1);
        assert_eq!(stats.stable_effect_draws, 0);
        assert_eq!(stats.dynamic_material_draws, 0);
        assert_eq!(stats.missing_art_draws, 0);
        assert_eq!(stats.unkeyed_fallback_draws, 0);
    }

    #[test]
    fn variant_atlas_software_uses_palette_effect_for_bg_color() {
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.bg_layers[0].enabled_main = true;
        frame.cgram_rgba[3 * 16 + 2] = [11, 22, 33, 0xff];
        frame.bg_layers[0]
            .index_tiles
            .push(ModernIndexTileInstance {
                cell_id: 0,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                screen_x: 0,
                screen_y: 0,
                palette: 3,
                hflip: false,
                vflip: false,
                priority: false,
            });
        let mut indices = [0u8; 64];
        indices[0] = 2;
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];
        let mut atlas_rgba = vec![0u8; 8 * 8 * 4];
        atlas_rgba[0..4].copy_from_slice(&[200, 1, 1, 0xff]);
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: atlas_rgba,
            entries: vec![VariantAtlasEntry {
                id: "bg:kBgGfx:pack0:tile0:3bpp".to_string(),
                key: VariantAtlasKey {
                    source_kind: "bg".to_string(),
                    asset: "kBgGfx".to_string(),
                    pack: 0,
                    tile: 0,
                    bpp: 3,
                    palette: "palette_main_spr".to_string(),
                    palette_row: 3,
                },
                rect: [0, 0, 8, 8],
                sha1: "test".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_main_spr:8color:row3".to_string(),
                palette: "palette_main_spr".to_string(),
                palette_row: 3,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [1, 1, 1, 0xff],
                    [11, 22, 33, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
            mode7_source_chars: None,
            dialogue_glyph_atlas: None,
            dialogue_vwf_font: None,
            dialogue_vwf_glyph_atlas: None,
        };

        let (variant, stats) = render_modern_frame_software_variant_atlas(
            &frame,
            &cells,
            &[],
            &atlas,
            "palette_main_spr",
            "palette_main_spr",
        );

        assert_eq!(&variant[0..4], &[11, 22, 33, 0xff]);
        assert_eq!(stats.stable_draws, 0);
        assert_eq!(stats.effect_draws, 1);
        assert_eq!(stats.fallback_draws, 0);
        assert_eq!(stats.stable_preview_draws, 0);
        assert_eq!(stats.stable_effect_draws, 0);
        assert_eq!(stats.dynamic_material_draws, 1);
    }

    #[test]
    fn variant_atlas_software_resolves_art_by_source_and_effect_by_live_palette() {
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.bg_layers[0].enabled_main = true;
        frame.bg_layers[0]
            .index_tiles
            .push(ModernIndexTileInstance {
                cell_id: 0,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                screen_x: 0,
                screen_y: 0,
                palette: 3,
                hflip: false,
                vflip: false,
                priority: false,
            });
        let mut indices = [0u8; 64];
        indices[0] = 2;
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];
        let mut atlas_rgba = vec![0u8; 8 * 8 * 4];
        atlas_rgba[0..4].copy_from_slice(&[200, 1, 1, 0xff]);
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: atlas_rgba,
            entries: vec![VariantAtlasEntry {
                id: "bg:kBgGfx:pack0:tile0:3bpp".to_string(),
                key: VariantAtlasKey {
                    source_kind: "bg".to_string(),
                    asset: "kBgGfx".to_string(),
                    pack: 0,
                    tile: 0,
                    bpp: 3,
                    palette: "palette_dung_bg_main".to_string(),
                    palette_row: 0,
                },
                rect: [0, 0, 8, 8],
                sha1: "test".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row3".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 3,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [1, 1, 1, 0xff],
                    [31, 41, 51, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
            mode7_source_chars: None,
            dialogue_glyph_atlas: None,
            dialogue_vwf_font: None,
            dialogue_vwf_glyph_atlas: None,
        };

        let (variant, stats) = render_modern_frame_software_variant_atlas(
            &frame,
            &cells,
            &[],
            &atlas,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(&variant[0..4], &[31, 41, 51, 0xff]);
        assert_eq!(stats.stable_draws, 0);
        assert_eq!(stats.effect_draws, 1);
        assert_eq!(stats.fallback_draws, 0);
        assert_eq!(stats.missing_variant_draws, 0);
        assert_eq!(stats.stable_preview_draws, 0);
        assert_eq!(stats.stable_effect_draws, 0);
        assert_eq!(stats.dynamic_material_draws, 1);
        assert_eq!(stats.missing_art_draws, 0);
        assert_eq!(stats.unkeyed_fallback_draws, 0);
    }

    #[test]
    fn variant_atlas_software_counts_unmodeled_material_fallback() {
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{ModernVariantAtlas, VariantAtlasEntry, VariantAtlasKey};

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.bg_layers[0].enabled_main = true;
        frame.cgram_rgba[3 * 16 + 1] = [44, 55, 66, 0xff];
        frame.bg_layers[0]
            .index_tiles
            .push(ModernIndexTileInstance {
                cell_id: 0,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                screen_x: 0,
                screen_y: 0,
                palette: 3,
                hflip: false,
                vflip: false,
                priority: false,
            });
        let mut indices = [0u8; 64];
        indices[0] = 1;
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![VariantAtlasEntry {
                id: "bg:kBgGfx:pack0:tile0:3bpp".to_string(),
                key: VariantAtlasKey {
                    source_kind: "bg".to_string(),
                    asset: "kBgGfx".to_string(),
                    pack: 0,
                    tile: 0,
                    bpp: 3,
                    palette: "palette_dung_bg_main".to_string(),
                    palette_row: 0,
                },
                rect: [0, 0, 8, 8],
                sha1: "test".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("shader_magic".to_string()),
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

        let (variant, stats) = render_modern_frame_software_variant_atlas(
            &frame,
            &cells,
            &[],
            &atlas,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(&variant[0..4], &[44, 55, 66, 0xff]);
        assert_eq!(stats.stable_draws, 0);
        assert_eq!(stats.fallback_draws, 1);
        assert_eq!(stats.dynamic_palette_draws, 1);
        assert_eq!(stats.stable_preview_draws, 0);
        assert_eq!(stats.stable_effect_draws, 0);
        assert_eq!(stats.dynamic_material_draws, 1);
        assert_eq!(stats.unsupported_material_draws, 1);
        assert_eq!(stats.missing_art_draws, 0);
        assert_eq!(stats.unkeyed_fallback_draws, 0);
    }

    #[test]
    fn variant_atlas_software_uses_palette_effect_for_sprite_color() {
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.cgram_rgba[0x80 + 4 * 16 + 3] = [21, 31, 41, 0xff];
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 4,
            hflip: false,
            vflip: false,
            priority: 0,
            row_mask: 0xff,
        });
        let mut indices = [0u8; 64];
        indices[0] = 3;
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: modern_source_key(2, 0, 0),
            hflip: false,
            vflip: false,
        }];
        let mut atlas_rgba = vec![0u8; 8 * 8 * 4];
        atlas_rgba[0..4].copy_from_slice(&[210, 2, 2, 0xff]);
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: atlas_rgba,
            entries: vec![VariantAtlasEntry {
                id: "sprite:kSprGfx:pack0:tile0:3bpp".to_string(),
                key: VariantAtlasKey {
                    source_kind: "sprite".to_string(),
                    asset: "kSprGfx".to_string(),
                    pack: 0,
                    tile: 0,
                    bpp: 3,
                    palette: "palette_main_spr".to_string(),
                    palette_row: 4,
                },
                rect: [0, 0, 8, 8],
                sha1: "test".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_main_spr:8color:row4".to_string(),
                palette: "palette_main_spr".to_string(),
                palette_row: 4,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [1, 1, 1, 0xff],
                    [2, 2, 2, 0xff],
                    [21, 31, 41, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
            mode7_source_chars: None,
            dialogue_glyph_atlas: None,
            dialogue_vwf_font: None,
            dialogue_vwf_glyph_atlas: None,
        };

        let (variant, stats) = render_modern_frame_software_variant_atlas(
            &frame,
            &[],
            &cells,
            &atlas,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(&variant[0..4], &[21, 31, 41, 0xff]);
        assert_eq!(stats.stable_draws, 0);
        assert_eq!(stats.effect_draws, 1);
        assert_eq!(stats.fallback_draws, 0);
        assert_eq!(stats.stable_preview_draws, 0);
        assert_eq!(stats.stable_effect_draws, 0);
        assert_eq!(stats.dynamic_material_draws, 1);
    }

    /// Regression test for the BG-flip HD-sampling fix: a BG cell baked with
    /// `hflip: true` must sample its HD-override art at the UN-FLIPPED source
    /// coordinate, not the post-flip screen coordinate. Unlike
    /// `source_keyed_overrides_recolor_bg_and_sprite` (which uses SOLID-color HD
    /// cells and therefore cannot distinguish a coordinate bug from a passing test),
    /// this uses a SPATIALLY-VARYING HD cell — color encodes the sampled `(x,y)` — so
    /// a wrong `(lx,ly)` produces a different, detectably-wrong pixel.
    ///
    /// Layout: one BG1 tile at screen (0,0), palette 2, base index 1 (cgram_idx =
    /// 2*16+1 = 33). `cell.indices` holds the BAKED (post-hflip) pattern: opaque ONLY
    /// at screen-local (0,0) — i.e. the pre-flip SOURCE tile was opaque at column 7,
    /// which mirrors to baked column 0 under hflip. `cell.hflip = true`.
    ///
    /// HD art (8x8, unflipped SOURCE orientation) encodes position:
    /// `r(x,y) = x*32`, `g(x,y) = y*32`, `b = 128` (constant). The reference palette
    /// at cgram_idx 33 is `[224, 128, 128, 0xff]` (R = 7*32, matching HD r at source
    /// x=7 exactly), so the CORRECT un-flipped sample (source x=7, y=0) gives:
    ///   detail_r = 224/224 = 1.0 -> final_r = live_r = 24
    ///   detail_g = 0/128   = 0.0 -> final_g = 0
    ///   detail_b = 128/128 = 1.0 -> final_b = live_b = 8
    /// All three predicted bytes (24, 0, 8) are `< 32` and multiples of 8, so the
    /// render's internal 8->5->8 (c5) quantize/expand round-trips them losslessly
    /// (`expand_brightness` at full brightness is the identity for c5 < 4) — the
    /// assertion below compares exact final bytes, not an approximation.
    ///
    /// A WRONG sample (the coordinate bug: reading screen coords `(sx,sy)=(0,0)`
    /// instead of the un-flipped `(7,0)`) would read `hd_rgb(0,0) = [0,0,128]` ->
    /// final_r = 0, NOT 24 — a different, failing result. (Confirmed by temporarily
    /// reverting the un-flip at the 3 BG resolve sites: this test then fails,
    /// asserting a 0 red byte where 24 is expected.)
    #[test]
    fn flipped_bg_cell_samples_unflipped_hd_source() {
        use crate::modern_hd_overrides::{HdCell, HdOverrideCtx, ModernHdOverrides};
        use std::collections::HashMap;

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.screen_enabled_main = 0x01; // BG1 only

        const CGRAM_IDX: usize = 2 * 16 + 1; // palette=2, index=1
        frame.cgram_rgba[CGRAM_IDX] = [24, 16, 8, 0xff]; // live

        const SOURCE_KEY: u64 = 0x0000_0009_0000_0000;
        let mut indices = [0u8; 64];
        indices[0] = 1; // baked (post-hflip) opaque pixel at screen-local (0,0)
        let bg_cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: SOURCE_KEY,
            hflip: true,
            vflip: false,
        }];
        frame.bg_layers[0]
            .index_tiles
            .push(ModernIndexTileInstance {
                cell_id: 0,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                screen_x: 0,
                screen_y: 0,
                palette: 2,
                // BG instance flip is always false; the CELL carries the baked flip.
                hflip: false,
                vflip: false,
                priority: false,
            });

        // Spatially-varying HD source art (unflipped orientation): r=x*32, g=y*32, b=128.
        let mut hd_rgba = vec![0u8; 8 * 8 * 4];
        for y in 0..8usize {
            for x in 0..8usize {
                let o = (y * 8 + x) * 4;
                hd_rgba[o] = (x as u8) * 32;
                hd_rgba[o + 1] = (y as u8) * 32;
                hd_rgba[o + 2] = 128;
                hd_rgba[o + 3] = 0xff;
            }
        }
        let hd_cell = HdCell {
            width: 8,
            height: 8,
            rgba: hd_rgba,
        };

        let mut reference = [[0u8; 4]; 256];
        reference[CGRAM_IDX] = [224, 128, 128, 0xff];

        let mut by_key = HashMap::new();
        by_key.insert(SOURCE_KEY, hd_cell);
        let store = ModernHdOverrides::from_parts(by_key, reference);
        let ctx = HdOverrideCtx::new(&store);

        let sprite_cells: Vec<ModernIndexTile> = Vec::new();
        let out = render_modern_frame_full_with_overrides(&frame, &bg_cells, &sprite_cells, &ctx);

        let off = 0usize; // screen (0,0) -> byte offset 0
        assert_eq!(
            &out[off..off + 3],
            &[24, 0, 8],
            "un-flipped HD sample must read source (7,0) (detail_r=1.0 -> final_r=24); \
             a wrong sample at screen/source (0,0) would give final_r=0"
        );
    }

    #[test]
    fn upscale_nearest_block_replicates() {
        // 2×1 source, scale 2 → 4×2 output; each pixel a 2×2 block.
        let src = vec![10, 20, 30, 40, /*px1*/ 50, 60, 70, 80];
        let out = upscale_rgba_nearest(&src, 2, 1, 2);
        assert_eq!(out.len(), 4 * 2 * 4);
        // row 0: px0 px0 px1 px1
        assert_eq!(&out[0..4], &[10, 20, 30, 40]);
        assert_eq!(&out[4..8], &[10, 20, 30, 40]);
        assert_eq!(&out[8..12], &[50, 60, 70, 80]);
        // row 1 mirrors row 0
        let row1 = 4 * 4;
        assert_eq!(&out[row1..row1 + 4], &[10, 20, 30, 40]);
        // scale 1 is identity
        assert_eq!(upscale_rgba_nearest(&src, 2, 1, 1), src);
    }

    /// Minimal frame for the N× compositor tests: BG1 enabled on main, full brightness,
    /// one fully-opaque 8×8 BG1 tile at screen (0,0), palette 0. No mosaic, no
    /// per-scanline scroll, no overrides — takes the simple (parameterizable) path.
    fn tiny_simple_bg_fixture() -> (ModernFrame, Vec<ModernIndexTile>) {
        let indices = [1u8; 64]; // fully opaque 8×8
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];
        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.brightness = 15;
        frame.screen_enabled_main = 0x01; // BG1
        frame.cgram_rgba[1] = [20 << 3, 10 << 3, 5 << 3, 0xff];
        frame.bg_layers[0]
            .index_tiles
            .push(ModernIndexTileInstance {
                cell_id: 0,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                screen_x: 0,
                screen_y: 0,
                palette: 0,
                hflip: false,
                vflip: false,
                priority: false,
            });
        (frame, cells)
    }

    #[test]
    fn composite_scale1_is_identity_and_scale2_block_upscales() {
        // One opaque BG tile (no mosaic, no scanline scroll, no overrides).
        let (frame, bg_cells) = tiny_simple_bg_fixture();
        // Native reference.
        let native = render_modern_frame_full(&frame, &bg_cells, &[]); // 256×224×4
                                                                       // scale=2 via the new entry: every native pixel must equal its 2×2 block.
        let hd = render_modern_frame_full_scaled(
            &frame,
            &bg_cells,
            &[],
            &crate::modern_hd_overrides::HdOverrideCtx::disabled(),
            2,
        );
        assert_eq!(hd.len(), 512 * 448 * 4);
        for ny in 0..224usize {
            for nx in 0..256usize {
                let src = (ny * 256 + nx) * 4;
                for dy in 0..2 {
                    for dx in 0..2 {
                        let o = ((ny * 2 + dy) * 512 + (nx * 2 + dx)) * 4;
                        assert_eq!(&hd[o..o + 3], &native[src..src + 3], "block ({nx},{ny})");
                    }
                }
            }
        }
    }

    #[test]
    fn scaled_entry_scale1_equals_native() {
        let (frame, bg_cells) = tiny_simple_bg_fixture();
        let native = render_modern_frame_full(&frame, &bg_cells, &[]);
        let via = render_modern_frame_full_scaled(
            &frame,
            &bg_cells,
            &[],
            &crate::modern_hd_overrides::HdOverrideCtx::disabled(),
            1,
        );
        assert_eq!(via, native);
    }

    #[test]
    fn complex_frame_falls_back_to_native_upscaled() {
        // A mosaic-active frame: scale=2 output must equal the native render block-upscaled.
        let (mut frame, bg_cells) = tiny_simple_bg_fixture();
        frame.mosaic_size = 2;
        frame.mosaic_enabled = 0x01; // BG1 mosaic
        frame.screen_enabled_main |= 0x01; // BG1 enabled
        assert!(frame_uses_complex_bg_path(&frame));
        let native = render_modern_frame_full(&frame, &bg_cells, &[]);
        let expected = upscale_rgba_nearest(&native, 256, 224, 2);
        let hd = render_modern_frame_full_scaled(
            &frame,
            &bg_cells,
            &[],
            &crate::modern_hd_overrides::HdOverrideCtx::disabled(),
            2,
        );
        assert_eq!(hd, expected);
    }

    /// HD overrides must sample SUB-PIXEL at N×: at scale=2, two adjacent output pixels
    /// of the SAME native texel (output (0,0) and (1,0) both map to native texel 0)
    /// sample ADJACENT HD texels (0 and 1). A spatially-varying 16×16 HD cell (red =
    /// column·16) makes those two pixels differ — impossible if the wiring sampled the
    /// native texel coordinate instead of the output sub-pixel. Reverting the `(ox,oy)`
    /// footprint sampling in `composite_index_tiles_c5` to the native texel (`nsx`) makes
    /// this assertion fail (both output pixels collapse to HD texel 0).
    #[test]
    fn hd_override_samples_subpixel_at_scale2() {
        use crate::modern_hd_overrides::{HdCell, HdOverrideCtx, ModernHdOverrides};
        use std::collections::HashMap;

        const CGRAM_IDX: usize = 1; // palette 0, index 1
        const SOURCE_KEY: u64 = 0x0000_0007_0000_0000;

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.brightness = 15;
        frame.screen_enabled_main = 0x01; // BG1
        frame.cgram_rgba[CGRAM_IDX] = [128, 128, 128, 0xff]; // live

        let indices = [1u8; 64]; // fully opaque, base index 1 everywhere
        let bg_cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];
        frame.bg_layers[0]
            .index_tiles
            .push(ModernIndexTileInstance {
                cell_id: 0,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                screen_x: 0,
                screen_y: 0,
                palette: 0,
                hflip: false,
                vflip: false,
                priority: false,
            });

        // 16×16 (2×) HD art: red encodes the HD column (x·16), g/b constant 128.
        let mut hd_rgba = vec![0u8; 16 * 16 * 4];
        for y in 0..16usize {
            for x in 0..16usize {
                let o = (y * 16 + x) * 4;
                hd_rgba[o] = (x as u8) * 16;
                hd_rgba[o + 1] = 128;
                hd_rgba[o + 2] = 128;
                hd_rgba[o + 3] = 0xff;
            }
        }
        let hd_cell = HdCell {
            width: 16,
            height: 16,
            rgba: hd_rgba,
        };

        let mut reference = [[0u8; 4]; 256];
        reference[CGRAM_IDX] = [128, 128, 128, 0xff]; // detail baseline == live

        let mut by_key = HashMap::new();
        by_key.insert(SOURCE_KEY, hd_cell);
        let store = ModernHdOverrides::from_parts(by_key, reference);
        let ctx = HdOverrideCtx::new(&store);

        let hd2 = render_modern_frame_full_scaled(&frame, &bg_cells, &[], &ctx, 2);
        // Output (0,0) samples HD texel col 0 (red 0); output (1,0) — SAME native texel
        // 0 but sub-pixel — samples HD texel col 1 (red 16). They must differ in red.
        let p00 = 0usize; // (0,0)
        let p10 = 4usize; // (1,0)
        assert_eq!(hd2[p00], 0, "output (0,0) red from HD texel col 0");
        assert_eq!(
            hd2[p10], 16,
            "output (1,0) red from HD texel col 1 (sub-pixel)"
        );
        assert_ne!(
            hd2[p00], hd2[p10],
            "adjacent output pixels must sample adjacent HD texels"
        );

        // A block-upscale of the scale=1 render CANNOT reproduce the sub-pixel detail:
        // the N× output must differ from a nearest-upscale of the native render.
        let native1 = render_modern_frame_full_scaled(&frame, &bg_cells, &[], &ctx, 1);
        let block = upscale_rgba_nearest(&native1, 256, 224, 2);
        assert_ne!(
            hd2, block,
            "sub-pixel HD sampling must differ from a block upscale"
        );
    }
}
