use crate::modern_frame::{ModernFrame, ModernIndexSpriteInstance, ModernIndexTileInstance};
use crate::modern_index_atlas::ModernIndexTile;
use crate::modern_software::VariantAtlasRenderStats;
use crate::modern_variant_atlas::{
    variant_key_for_index_tile, variant_key_for_source_key, DynamicFallbackReason,
    ModernVariantAtlas, SourceEntryIndex, VariantAtlasDraw, VariantAtlasEntry, VariantAtlasKey,
};

#[derive(Clone, Debug)]
pub struct VariantDrawPlan<'a> {
    pub bg: Vec<VariantBgDrawPacket<'a>>,
    pub sprites: Vec<VariantSpriteDrawPacket<'a>>,
    pub stats: VariantAtlasRenderStats,
}

impl<'a> VariantDrawPlan<'a> {
    pub fn material_packets(&self) -> impl Iterator<Item = VariantDrawPacket<'_, 'a>> {
        self.bg
            .iter()
            .enumerate()
            .map(|(packet_index, packet)| VariantDrawPacket::Bg {
                packet_index,
                packet,
            })
            .chain(
                self.sprites
                    .iter()
                    .enumerate()
                    .map(|(packet_index, packet)| VariantDrawPacket::Sprite {
                        packet_index,
                        packet,
                    }),
            )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModernDrawMaterial {
    RgbaAtlas,
    PaletteEffect,
    DynamicPalette(DynamicFallbackReason),
    MissingArt,
    LiveIndex,
}

impl ModernDrawMaterial {
    pub fn name(self) -> &'static str {
        match self {
            Self::RgbaAtlas => "rgba_atlas",
            Self::PaletteEffect => "palette_effect",
            Self::DynamicPalette(_) => "dynamic_palette",
            Self::MissingArt => "missing_art",
            Self::LiveIndex => "live_index",
        }
    }
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

#[derive(Clone, Copy, Debug)]
pub enum VariantDrawPacket<'plan, 'frame> {
    Bg {
        packet_index: usize,
        packet: &'plan VariantBgDrawPacket<'frame>,
    },
    Sprite {
        packet_index: usize,
        packet: &'plan VariantSpriteDrawPacket<'frame>,
    },
}

impl<'a> VariantBgDrawPacket<'a> {
    pub fn material(&self) -> ModernDrawMaterial {
        material_for_draw(self.draw)
    }

    pub fn mode1_rank(&self) -> Option<u8> {
        VariantDrawPacket::Bg {
            packet_index: 0,
            packet: self,
        }
        .mode1_rank()
    }
}

impl<'a> VariantSpriteDrawPacket<'a> {
    pub fn material(&self) -> ModernDrawMaterial {
        material_for_draw(self.draw)
    }

    pub fn mode1_rank(&self) -> Option<u8> {
        VariantDrawPacket::Sprite {
            packet_index: 0,
            packet: self,
        }
        .mode1_rank()
    }
}

impl<'plan, 'frame> VariantDrawPacket<'plan, 'frame> {
    pub fn surface(self) -> VariantDrawSurface {
        match self {
            Self::Bg { .. } => VariantDrawSurface::Bg,
            Self::Sprite { .. } => VariantDrawSurface::Sprite,
        }
    }

    pub fn as_bg(self) -> Option<(usize, &'plan VariantBgDrawPacket<'frame>)> {
        match self {
            Self::Bg {
                packet_index,
                packet,
            } => Some((packet_index, packet)),
            Self::Sprite { .. } => None,
        }
    }

    pub fn as_sprite(self) -> Option<(usize, &'plan VariantSpriteDrawPacket<'frame>)> {
        match self {
            Self::Bg { .. } => None,
            Self::Sprite {
                packet_index,
                packet,
            } => Some((packet_index, packet)),
        }
    }

    pub fn packet_index(self) -> usize {
        match self {
            Self::Bg { packet_index, .. } | Self::Sprite { packet_index, .. } => packet_index,
        }
    }

    pub fn layer_index(self) -> Option<usize> {
        match self {
            Self::Bg { packet, .. } => Some(packet.layer_index),
            Self::Sprite { .. } => None,
        }
    }

    pub fn material(self) -> ModernDrawMaterial {
        material_for_draw(self.draw())
    }

    pub fn draw(self) -> VariantAtlasDraw<'frame> {
        match self {
            Self::Bg { packet, .. } => packet.draw,
            Self::Sprite { packet, .. } => packet.draw,
        }
    }

    pub fn key(self) -> Option<&'plan VariantAtlasKey> {
        match self {
            Self::Bg { packet, .. } => packet.key.as_ref(),
            Self::Sprite { packet, .. } => packet.key.as_ref(),
        }
    }

    pub fn cell_id(self) -> u32 {
        match self {
            Self::Bg { packet, .. } => packet.cell.id,
            Self::Sprite { packet, .. } => packet.cell.id,
        }
    }

    pub fn screen_origin(self) -> (i16, i16) {
        match self {
            Self::Bg { packet, .. } => (packet.inst.screen_x, packet.inst.screen_y),
            Self::Sprite { packet, .. } => (packet.inst.screen_x, packet.inst.screen_y),
        }
    }

    pub fn palette_row(self) -> u8 {
        match self {
            Self::Bg { packet, .. } => packet.inst.palette,
            Self::Sprite { packet, .. } => packet.inst.palette,
        }
    }

    pub fn source_flip_with_entry(self, entry: &VariantAtlasEntry) -> (bool, bool) {
        match self {
            Self::Bg { packet, .. } => (
                packet.cell.hflip ^ entry.source_hflip,
                packet.cell.vflip ^ entry.source_vflip,
            ),
            Self::Sprite { packet, .. } => (
                packet.inst.hflip ^ entry.source_hflip,
                packet.inst.vflip ^ entry.source_vflip,
            ),
        }
    }

    pub fn mode1_rank(self) -> Option<u8> {
        match self {
            Self::Bg { packet, .. } => match (packet.layer_index, packet.inst.priority) {
                (2, false) => Some(0), // BG3-lo
                (1, false) => Some(3), // BG2-lo
                (0, false) => Some(4), // BG1-lo
                (1, true) => Some(6),  // BG2-hi
                (0, true) => Some(7),  // BG1-hi
                (2, true) => Some(9),  // BG3-hi
                _ => None,
            },
            Self::Sprite { packet, .. } => match packet.inst.priority {
                0 => Some(1), // OBJ0
                1 => Some(2), // OBJ1
                2 => Some(5), // OBJ2
                3 => Some(8), // OBJ3
                _ => None,
            },
        }
    }

    pub fn local_xy(self, screen_x: i16, screen_y: i16) -> Option<(usize, usize)> {
        let (origin_x, origin_y) = self.screen_origin();
        let local_x = screen_x - origin_x;
        let local_y = screen_y - origin_y;
        if !(0..8).contains(&local_x) || !(0..8).contains(&local_y) {
            return None;
        }
        Some((local_x as usize, local_y as usize))
    }

    pub fn overlap_index_at_screen(self, screen_x: i16, screen_y: i16) -> Option<u8> {
        let (local_x, local_y) = self.local_xy(screen_x, screen_y)?;
        match self {
            Self::Bg { packet, .. } => Some(packet.cell.indices[local_y * 8 + local_x]),
            Self::Sprite { packet, .. } => {
                if packet.inst.row_mask & (1 << local_y) == 0 {
                    return None;
                }
                let source_x = if packet.inst.hflip {
                    7 - local_x
                } else {
                    local_x
                };
                let source_y = if packet.inst.vflip {
                    7 - local_y
                } else {
                    local_y
                };
                Some(packet.cell.indices[source_y * 8 + source_x])
            }
        }
    }

    fn trace_pixel(
        self,
        frame: &ModernFrame,
        atlas: &ModernVariantAtlas,
        screen_x: i16,
        screen_y: i16,
    ) -> Option<VariantPixelTrace> {
        let (local_x, local_y) = self.local_xy(screen_x, screen_y)?;
        match self {
            Self::Bg {
                packet_index,
                packet,
            } => trace_bg_packet_pixel(
                frame,
                atlas,
                packet_index,
                packet,
                local_x as u8,
                local_y as u8,
            ),
            Self::Sprite {
                packet_index,
                packet,
            } => {
                if packet.inst.row_mask & (1 << local_y) == 0 {
                    return None;
                }
                Some(trace_sprite_packet_pixel(
                    frame,
                    atlas,
                    packet_index,
                    packet,
                    local_x as u8,
                    local_y as u8,
                ))
            }
        }
    }
}

pub fn material_for_draw(draw: VariantAtlasDraw<'_>) -> ModernDrawMaterial {
    match draw {
        VariantAtlasDraw::Stable { .. } => ModernDrawMaterial::RgbaAtlas,
        VariantAtlasDraw::MaterialEffect { .. } => ModernDrawMaterial::PaletteEffect,
        VariantAtlasDraw::DynamicPalette { reason, .. } => {
            ModernDrawMaterial::DynamicPalette(reason)
        }
        VariantAtlasDraw::MissingArt => ModernDrawMaterial::MissingArt,
        VariantAtlasDraw::Unkeyed => ModernDrawMaterial::LiveIndex,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariantDrawSurface {
    Bg,
    Sprite,
}

impl VariantDrawSurface {
    pub fn name(self) -> &'static str {
        match self {
            Self::Bg => "bg",
            Self::Sprite => "sprite",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VariantPixelTrace {
    pub surface: VariantDrawSurface,
    pub packet_index: usize,
    pub layer_index: Option<usize>,
    pub material: ModernDrawMaterial,
    pub cell_id: u32,
    pub screen_x: i16,
    pub screen_y: i16,
    pub local_x: u8,
    pub local_y: u8,
    pub source_x: u8,
    pub source_y: u8,
    pub palette_row: u8,
    pub palette_index: u8,
    pub output_rgba: Option<[u8; 4]>,
    pub live_cgram_rgba: Option<[u8; 4]>,
    pub main_screen_enabled: bool,
    pub main_scanline_enabled: bool,
    pub main_window_masked: bool,
    pub key: Option<VariantAtlasKey>,
    pub entry_id: Option<String>,
    pub effect_id: Option<String>,
}

impl VariantPixelTrace {
    pub fn describe(&self) -> String {
        let rgba = self.output_rgba.map_or_else(
            || "none".to_string(),
            |rgba| format!("{},{},{},{}", rgba[0], rgba[1], rgba[2], rgba[3]),
        );
        let live_rgba = self.live_cgram_rgba.map_or_else(
            || "none".to_string(),
            |rgba| format!("{},{},{},{}", rgba[0], rgba[1], rgba[2], rgba[3]),
        );
        let key = self.key.as_ref().map_or_else(
            || "none".to_string(),
            |key| {
                format!(
                    "{}:{}:pack{}:tile{}:{}bpp:{}:row{}",
                    key.source_kind,
                    key.asset,
                    key.pack,
                    key.tile,
                    key.bpp,
                    key.palette,
                    key.palette_row
                )
            },
        );
        format!(
            "surface={} packet={} layer={} material={} cell={} screen=({}, {}) local=({}, {}) source=({}, {}) palette_row={} index={} rgba={} live_cgram_rgba={} main_screen_enabled={} main_scanline_enabled={} main_window_masked={} key={} entry={} effect={}",
            self.surface.name(),
            self.packet_index,
            self.layer_index
                .map(|layer| layer.to_string())
                .unwrap_or_else(|| "none".to_string()),
            self.material.name(),
            self.cell_id,
            self.screen_x,
            self.screen_y,
            self.local_x,
            self.local_y,
            self.source_x,
            self.source_y,
            self.palette_row,
            self.palette_index,
            rgba,
            live_rgba,
            self.main_screen_enabled,
            self.main_scanline_enabled,
            self.main_window_masked,
            key,
            self.entry_id.as_deref().unwrap_or("none"),
            self.effect_id.as_deref().unwrap_or("none"),
        )
    }
}

pub fn compile_variant_draws<'a>(
    frame: &'a ModernFrame,
    bg_cells: &'a [ModernIndexTile],
    sprite_cells: &'a [ModernIndexTile],
    atlas: &'a ModernVariantAtlas,
    bg_palette_name: &str,
    sprite_palette_name: &str,
) -> VariantDrawPlan<'a> {
    compile_variant_draws_with_source_index(
        frame,
        bg_cells,
        sprite_cells,
        atlas,
        None,
        bg_palette_name,
        sprite_palette_name,
    )
}

pub(crate) fn compile_variant_draws_with_source_index<'a>(
    frame: &'a ModernFrame,
    bg_cells: &'a [ModernIndexTile],
    sprite_cells: &'a [ModernIndexTile],
    atlas: &'a ModernVariantAtlas,
    source_index: Option<&'a SourceEntryIndex>,
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
            let uses_instance_source_key =
                inst.source_key != crate::modern_hd_overrides::NO_SOURCE_KEY;
            let source_key = if uses_instance_source_key {
                inst.source_key
            } else {
                cell.source_key
            };
            let key = variant_key_for_source_key(source_key, bg_palette_name, inst.palette);
            let draw =
                resolve_draw_for_frame(atlas, source_index, key.as_ref(), uses_instance_source_key);
            plan.stats.record_bg_draw(layer_index, &draw);
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
        let draw = resolve_draw_for_frame(atlas, source_index, key.as_ref(), false);
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

pub fn compile_variant_draw_stats(
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
    atlas: &ModernVariantAtlas,
    bg_palette_name: &str,
    sprite_palette_name: &str,
) -> VariantAtlasRenderStats {
    compile_variant_draw_stats_with_source_index(
        frame,
        bg_cells,
        sprite_cells,
        atlas,
        None,
        bg_palette_name,
        sprite_palette_name,
    )
}

pub(crate) fn compile_variant_draw_stats_with_source_index(
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
    atlas: &ModernVariantAtlas,
    source_index: Option<&SourceEntryIndex>,
    bg_palette_name: &str,
    sprite_palette_name: &str,
) -> VariantAtlasRenderStats {
    let mut stats = VariantAtlasRenderStats::default();

    if frame.forced_blank {
        return stats;
    }

    for (layer_index, layer) in frame.bg_layers.iter().enumerate() {
        if !layer.enabled_main {
            continue;
        }
        for inst in &layer.index_tiles {
            let Some(cell) = bg_cells.get(inst.cell_id as usize) else {
                continue;
            };
            let uses_instance_source_key =
                inst.source_key != crate::modern_hd_overrides::NO_SOURCE_KEY;
            let source_key = if uses_instance_source_key {
                inst.source_key
            } else {
                cell.source_key
            };
            let key = variant_key_for_source_key(source_key, bg_palette_name, inst.palette);
            let draw =
                resolve_draw_for_frame(atlas, source_index, key.as_ref(), uses_instance_source_key);
            stats.record_bg_draw(layer_index, &draw);
        }
    }

    for inst in frame.index_sprites.iter().rev() {
        let Some(cell) = sprite_cells.get(inst.cell_id as usize) else {
            continue;
        };
        let key = variant_key_for_index_tile(cell, sprite_palette_name, inst.palette);
        let draw = resolve_draw_for_frame(atlas, source_index, key.as_ref(), false);
        stats.record_sprite_draw(&draw);
    }

    stats
}

pub fn trace_variant_plan_pixel(
    frame: &ModernFrame,
    atlas: &ModernVariantAtlas,
    plan: &VariantDrawPlan<'_>,
    screen_x: i16,
    screen_y: i16,
) -> Vec<VariantPixelTrace> {
    let mut traces = Vec::new();
    for packet in plan.material_packets() {
        let Some(trace) = packet.trace_pixel(frame, atlas, screen_x, screen_y) else {
            continue;
        };
        traces.push(trace);
    }
    traces
}

fn trace_bg_packet_pixel(
    frame: &ModernFrame,
    atlas: &ModernVariantAtlas,
    packet_index: usize,
    packet: &VariantBgDrawPacket<'_>,
    local_x: u8,
    local_y: u8,
) -> Option<VariantPixelTrace> {
    let entry = packet.draw.entry();
    let source_hflip = entry.is_some_and(|entry| entry.source_hflip);
    let source_vflip = entry.is_some_and(|entry| entry.source_vflip);
    let source_x = if packet.cell.hflip ^ source_hflip {
        7 - local_x
    } else {
        local_x
    };
    let source_y = if packet.cell.vflip ^ source_vflip {
        7 - local_y
    } else {
        local_y
    };
    let palette_index = packet.cell.indices[usize::from(source_y) * 8 + usize::from(source_x)];
    let (main_screen_enabled, main_scanline_enabled, main_window_masked) = main_visibility_trace(
        frame,
        packet.layer_index as u8,
        packet.inst.screen_x + local_x as i16,
        packet.inst.screen_y + local_y as i16,
    );
    let output_rgba = bg_output_rgba(frame, atlas, packet, source_x, source_y, palette_index);
    Some(VariantPixelTrace {
        surface: VariantDrawSurface::Bg,
        packet_index,
        layer_index: Some(packet.layer_index),
        material: packet.material(),
        cell_id: packet.cell.id,
        screen_x: packet.inst.screen_x,
        screen_y: packet.inst.screen_y,
        local_x,
        local_y,
        source_x,
        source_y,
        palette_row: packet.inst.palette,
        palette_index,
        output_rgba,
        live_cgram_rgba: bg_live_cgram_rgba(frame, packet, palette_index),
        main_screen_enabled,
        main_scanline_enabled,
        main_window_masked,
        key: packet.key.clone(),
        entry_id: entry.map(|entry| entry.id.clone()),
        effect_id: packet
            .draw
            .material_effect()
            .map(|(_, effect)| effect.id.clone()),
    })
}

fn trace_sprite_packet_pixel(
    frame: &ModernFrame,
    atlas: &ModernVariantAtlas,
    packet_index: usize,
    packet: &VariantSpriteDrawPacket<'_>,
    local_x: u8,
    local_y: u8,
) -> VariantPixelTrace {
    let entry = packet.draw.entry();
    let source_hflip = entry.is_some_and(|entry| entry.source_hflip);
    let source_vflip = entry.is_some_and(|entry| entry.source_vflip);
    let source_x = if packet.inst.hflip ^ source_hflip {
        7 - local_x
    } else {
        local_x
    };
    let source_y = if packet.inst.vflip ^ source_vflip {
        7 - local_y
    } else {
        local_y
    };
    let palette_index = packet.cell.indices[usize::from(source_y) * 8 + usize::from(source_x)];
    let (main_screen_enabled, main_scanline_enabled, main_window_masked) = main_visibility_trace(
        frame,
        4,
        packet.inst.screen_x + local_x as i16,
        packet.inst.screen_y + local_y as i16,
    );
    VariantPixelTrace {
        surface: VariantDrawSurface::Sprite,
        packet_index,
        layer_index: None,
        material: packet.material(),
        cell_id: packet.cell.id,
        screen_x: packet.inst.screen_x,
        screen_y: packet.inst.screen_y,
        local_x,
        local_y,
        source_x,
        source_y,
        palette_row: packet.inst.palette,
        palette_index,
        output_rgba: sprite_output_rgba(frame, atlas, packet, source_x, source_y, palette_index),
        live_cgram_rgba: sprite_live_cgram_rgba(frame, packet, palette_index),
        main_screen_enabled,
        main_scanline_enabled,
        main_window_masked,
        key: packet.key.clone(),
        entry_id: entry.map(|entry| entry.id.clone()),
        effect_id: packet
            .draw
            .material_effect()
            .map(|(_, effect)| effect.id.clone()),
    }
}

fn main_visibility_trace(frame: &ModernFrame, layer: u8, sx: i16, sy: i16) -> (bool, bool, bool) {
    let layer_bit = 1u8 << layer;
    let screen_enabled =
        frame.screen_enabled_main == 0 || frame.screen_enabled_main & layer_bit != 0;
    let scanline_enabled = if (0..224).contains(&sy) {
        frame
            .main_tm_scanlines
            .get(sy as usize)
            .is_none_or(|tm| tm & layer_bit != 0)
    } else {
        false
    };
    let window_masked = if (0..256).contains(&sx) && (0..224).contains(&sy) {
        main_layer_window_masks_pixel(frame, layer, sx as u32, sy as usize)
    } else {
        false
    };
    (screen_enabled, scanline_enabled, window_masked)
}

fn main_layer_window_masks_pixel(frame: &ModernFrame, layer: u8, sx: u32, sy: usize) -> bool {
    if frame.screen_windowed_main & (1u8 << layer) == 0 {
        return false;
    }
    let window_flags = (frame.windowsel >> (u32::from(layer) * 4)) & 0x0f;
    let w1_enabled = window_flags & 0x2 != 0;
    let w2_enabled = window_flags & 0x8 != 0;
    if !w1_enabled && !w2_enabled {
        return false;
    }
    let [w1l, w1r, w2l, w2r] = frame
        .window_scanlines
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
    match (w1_enabled, w2_enabled) {
        (true, false) => test1,
        (false, true) => test2,
        (true, true) => test1 || test2,
        (false, false) => false,
    }
}

fn bg_output_rgba(
    frame: &ModernFrame,
    atlas: &ModernVariantAtlas,
    packet: &VariantBgDrawPacket<'_>,
    source_x: u8,
    source_y: u8,
    palette_index: u8,
) -> Option<[u8; 4]> {
    match packet.draw {
        VariantAtlasDraw::Stable { entry } => atlas_entry_rgba(atlas, entry, source_x, source_y),
        VariantAtlasDraw::MaterialEffect { effect, .. } => {
            if palette_index == 0 {
                None
            } else {
                effect
                    .index_to_rgba
                    .get(usize::from(palette_index))
                    .copied()
            }
        }
        VariantAtlasDraw::DynamicPalette { .. }
        | VariantAtlasDraw::MissingArt
        | VariantAtlasDraw::Unkeyed => {
            if palette_index == 0 {
                None
            } else {
                frame
                    .cgram_rgba
                    .get(usize::from(packet.inst.palette) * 16 + usize::from(palette_index))
                    .copied()
            }
        }
    }
}

fn bg_live_cgram_rgba(
    frame: &ModernFrame,
    packet: &VariantBgDrawPacket<'_>,
    palette_index: u8,
) -> Option<[u8; 4]> {
    if palette_index == 0 {
        return None;
    }
    frame
        .cgram_rgba
        .get(usize::from(packet.inst.palette) * 16 + usize::from(palette_index))
        .copied()
}

fn sprite_output_rgba(
    frame: &ModernFrame,
    atlas: &ModernVariantAtlas,
    packet: &VariantSpriteDrawPacket<'_>,
    source_x: u8,
    source_y: u8,
    palette_index: u8,
) -> Option<[u8; 4]> {
    match packet.draw {
        VariantAtlasDraw::Stable { entry } => atlas_entry_rgba(atlas, entry, source_x, source_y),
        VariantAtlasDraw::MaterialEffect { effect, .. } => {
            if palette_index == 0 {
                None
            } else {
                effect
                    .index_to_rgba
                    .get(usize::from(palette_index))
                    .copied()
            }
        }
        VariantAtlasDraw::DynamicPalette { .. }
        | VariantAtlasDraw::MissingArt
        | VariantAtlasDraw::Unkeyed => {
            if palette_index == 0 {
                None
            } else {
                frame
                    .cgram_rgba
                    .get(0x80 + usize::from(packet.inst.palette) * 16 + usize::from(palette_index))
                    .copied()
            }
        }
    }
}

fn sprite_live_cgram_rgba(
    frame: &ModernFrame,
    packet: &VariantSpriteDrawPacket<'_>,
    palette_index: u8,
) -> Option<[u8; 4]> {
    if palette_index == 0 {
        return None;
    }
    frame
        .cgram_rgba
        .get(0x80 + usize::from(packet.inst.palette) * 16 + usize::from(palette_index))
        .copied()
}

fn atlas_entry_rgba(
    atlas: &ModernVariantAtlas,
    entry: &crate::modern_variant_atlas::VariantAtlasEntry,
    source_x: u8,
    source_y: u8,
) -> Option<[u8; 4]> {
    let atlas_x = entry.rect[0] + u32::from(source_x);
    let atlas_y = entry.rect[1] + u32::from(source_y);
    if atlas_x >= atlas.width || atlas_y >= atlas.height {
        return None;
    }
    let offset = (atlas_y as usize * atlas.width as usize + atlas_x as usize) * 4;
    let rgba = [
        atlas.rgba[offset],
        atlas.rgba[offset + 1],
        atlas.rgba[offset + 2],
        atlas.rgba[offset + 3],
    ];
    (rgba[3] != 0).then_some(rgba)
}

fn resolve_draw_for_frame<'a>(
    atlas: &'a ModernVariantAtlas,
    source_index: Option<&'a SourceEntryIndex>,
    key: Option<&VariantAtlasKey>,
    force_dynamic: bool,
) -> VariantAtlasDraw<'a> {
    if force_dynamic {
        let draw = match source_index {
            Some(source_index) => atlas.resolve_draw_in_index(key, source_index),
            None => atlas.resolve_draw(key),
        };
        if matches!(
            draw,
            VariantAtlasDraw::MaterialEffect { .. } | VariantAtlasDraw::Stable { .. }
        ) {
            draw
        } else {
            match source_index {
                Some(source_index) => atlas.resolve_dynamic_draw_in_index(
                    key,
                    DynamicFallbackReason::InstanceSourceKey,
                    source_index,
                ),
                None => atlas.resolve_dynamic_draw(key, DynamicFallbackReason::InstanceSourceKey),
            }
        }
    } else {
        match source_index {
            Some(source_index) => atlas.resolve_draw_in_index(key, source_index),
            None => atlas.resolve_draw(key),
        }
    }
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
            runtime_material: Some("palette_lut".to_string()),
            runtime_colors_per_row: None,
            source_hflip: false,
            source_vflip: false,
        }
    }

    fn sprite_key(pack: u16, tile: u16, palette_row: u8) -> VariantAtlasKey {
        VariantAtlasKey {
            source_kind: "sprite".to_string(),
            asset: "kSprGfx".to_string(),
            pack,
            tile,
            bpp: 3,
            palette: "palette_main_spr".to_string(),
            palette_row,
        }
    }

    fn sprite_entry(pack: u16, tile: u16, palette_row: u8) -> VariantAtlasEntry {
        VariantAtlasEntry {
            id: format!("sprite:kSprGfx:pack{pack}:tile{tile}:3bpp"),
            key: sprite_key(pack, tile, palette_row),
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

    fn sprite_effect(palette_row: u8) -> TileEffect {
        TileEffect {
            id: format!("palette_main_spr:8color:row{palette_row}"),
            palette: "palette_main_spr".to_string(),
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
            mode7_source_chars: None,
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
            VariantAtlasDraw::MaterialEffect { .. }
        ));
        assert_eq!(plan.bg[0].material(), ModernDrawMaterial::PaletteEffect);
        assert_eq!(plan.bg[1].cell.id, 1);
        assert!(matches!(plan.bg[1].draw, VariantAtlasDraw::MissingArt));
        assert_eq!(plan.bg[1].material(), ModernDrawMaterial::MissingArt);
        assert_eq!(plan.sprites.len(), 1);
        assert_eq!(plan.sprites[0].inst.screen_y, 24);
        assert!(matches!(plan.sprites[0].draw, VariantAtlasDraw::Unkeyed));
        assert_eq!(plan.sprites[0].material(), ModernDrawMaterial::LiveIndex);
        assert_eq!(plan.stats.stable_effect_draws, 0);
        assert_eq!(plan.stats.effect_material_draws, 1);
        assert_eq!(plan.stats.dynamic_material_draws, 1);
        assert_eq!(plan.stats.dynamic_material_fallback_draws, 0);
        assert_eq!(plan.stats.missing_art_draws, 1);
        assert_eq!(plan.stats.fallback_draws, 1);
        assert_eq!(plan.stats.live_index_draws, 1);
        assert_eq!(plan.stats.live_index_bg_draws, 0);
        assert_eq!(plan.stats.live_index_sprite_draws, 1);
        assert_eq!(plan.stats.unkeyed_fallback_draws, 1);
        assert_eq!(plan.stats.unkeyed_bg_fallback_draws, 0);
        assert_eq!(plan.stats.unkeyed_sprite_fallback_draws, 1);

        let stats = compile_variant_draw_stats(
            &frame,
            &bg_cells,
            &sprite_cells,
            &atlas,
            "palette_dung_bg_main",
            "palette_main_spr",
        );
        assert_eq!(stats, plan.stats);
    }

    #[test]
    fn material_packets_expose_single_ordered_gpu_packet_stream() {
        let mut frame = ModernFrame::empty();
        let mut bg0 = ModernBgLayer::new(0);
        bg0.enabled_main = true;
        bg0.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: NO_SOURCE_KEY,
            screen_x: 4,
            screen_y: 8,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        bg0.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            source_key: NO_SOURCE_KEY,
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
        let sprite_cells = vec![index_cell(2, NO_SOURCE_KEY)];
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![bg_entry(0, 0, 0)],
            effects: vec![effect(2)],
            mode7_source_chars: None,
        };

        let plan = compile_variant_draws(
            &frame,
            &bg_cells,
            &sprite_cells,
            &atlas,
            "palette_dung_bg_main",
            "palette_main_spr",
        );
        let packets: Vec<_> = plan.material_packets().collect();

        assert_eq!(packets.len(), 3);
        assert_eq!(packets[0].surface(), VariantDrawSurface::Bg);
        assert_eq!(packets[0].packet_index(), 0);
        assert_eq!(packets[0].layer_index(), Some(0));
        assert_eq!(packets[0].cell_id(), 0);
        assert_eq!(packets[0].screen_origin(), (4, 8));
        assert_eq!(packets[0].palette_row(), 2);
        assert_eq!(packets[0].material(), ModernDrawMaterial::PaletteEffect);
        assert_eq!(packets[0].mode1_rank(), Some(4));
        assert_eq!(packets[0].local_xy(6, 11), Some((2, 3)));

        assert_eq!(packets[1].surface(), VariantDrawSurface::Bg);
        assert_eq!(packets[1].packet_index(), 1);
        assert_eq!(packets[1].cell_id(), 1);
        assert_eq!(packets[1].material(), ModernDrawMaterial::MissingArt);
        assert_eq!(packets[1].mode1_rank(), Some(4));

        assert_eq!(packets[2].surface(), VariantDrawSurface::Sprite);
        assert_eq!(packets[2].packet_index(), 0);
        assert_eq!(packets[2].layer_index(), None);
        assert_eq!(packets[2].cell_id(), 2);
        assert_eq!(packets[2].screen_origin(), (20, 24));
        assert_eq!(packets[2].palette_row(), 0);
        assert_eq!(packets[2].material(), ModernDrawMaterial::LiveIndex);
        assert_eq!(packets[2].mode1_rank(), Some(1));
    }

    #[test]
    fn material_packets_resolve_overlap_indices_for_bg_and_sprite_rules() {
        let mut frame = ModernFrame::empty();
        let mut bg0 = ModernBgLayer::new(0);
        bg0.enabled_main = true;
        bg0.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: NO_SOURCE_KEY,
            screen_x: 4,
            screen_y: 8,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: true,
        });
        frame.bg_layers[0] = bg0;
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 20,
            screen_y: 24,
            palette: 0,
            priority: 2,
            hflip: true,
            vflip: true,
            row_mask: 0b0000_0010,
        });
        let mut bg_cell = index_cell(0, modern_source_key(1, 0, 0));
        bg_cell.indices[3 * 8 + 2] = 5;
        let mut sprite_cell = index_cell(1, NO_SOURCE_KEY);
        sprite_cell.indices[6 * 8 + 5] = 7;
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![],
            effects: vec![],
            mode7_source_chars: None,
        };
        let bg_cells = vec![bg_cell];
        let sprite_cells = vec![sprite_cell];

        let plan = compile_variant_draws(
            &frame,
            &bg_cells,
            &sprite_cells,
            &atlas,
            "palette_dung_bg_main",
            "palette_main_spr",
        );
        let packets: Vec<_> = plan.material_packets().collect();

        assert_eq!(packets[0].mode1_rank(), Some(7));
        assert_eq!(packets[0].overlap_index_at_screen(6, 11), Some(5));
        assert_eq!(packets[0].overlap_index_at_screen(3, 11), None);
        assert_eq!(packets[1].mode1_rank(), Some(5));
        assert_eq!(packets[1].overlap_index_at_screen(22, 25), Some(7));
        assert_eq!(packets[1].overlap_index_at_screen(22, 24), None);
    }

    #[test]
    fn classifies_unkeyed_bg_live_index_draws_by_layer_group() {
        let mut frame = ModernFrame::empty();
        for layer_index in [0usize, 2] {
            let mut layer = ModernBgLayer::new(layer_index as u8);
            layer.enabled_main = true;
            let cell_id = if layer_index == 2 { 1 } else { 0 };
            layer.index_tiles.push(ModernIndexTileInstance {
                cell_id,
                source_key: NO_SOURCE_KEY,
                screen_x: (layer_index * 8) as i16,
                screen_y: 0,
                palette: 0,
                hflip: false,
                vflip: false,
                priority: false,
            });
            frame.bg_layers[layer_index] = layer;
        }
        let bg_cells = vec![index_cell(0, NO_SOURCE_KEY), index_cell(1, NO_SOURCE_KEY)];
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![],
            effects: vec![],
            mode7_source_chars: None,
        };

        let plan = compile_variant_draws(
            &frame,
            &bg_cells,
            &[],
            &atlas,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(plan.stats.fallback_draws, 0);
        assert_eq!(plan.stats.live_index_draws, 2);
        assert_eq!(plan.stats.live_index_bg_draws, 2);
        assert_eq!(plan.stats.live_index_bg12_draws, 1);
        assert_eq!(plan.stats.live_index_bg3_draws, 1);
        assert_eq!(plan.stats.live_index_sprite_draws, 0);
        assert_eq!(plan.stats.unkeyed_fallback_draws, 2);
        assert_eq!(plan.stats.unkeyed_bg_fallback_draws, 2);
        assert_eq!(plan.stats.unkeyed_bg12_fallback_draws, 1);
        assert_eq!(plan.stats.unkeyed_bg3_fallback_draws, 1);
        assert_eq!(plan.stats.unkeyed_sprite_fallback_draws, 0);
    }

    #[test]
    fn bg_instance_source_key_resolves_deduped_cell_as_material_effect() {
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
            mode7_source_chars: None,
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
            VariantAtlasDraw::MaterialEffect { .. }
        ));
        assert_eq!(plan.stats.dynamic_palette_draws, 0);
        assert_eq!(plan.stats.effect_material_draws, 1);
        assert_eq!(plan.stats.dynamic_material_fallback_draws, 0);
        assert_eq!(
            plan.stats.dynamic_material_fallback_instance_source_draws,
            0
        );
        assert_eq!(plan.stats.stable_effect_draws, 0);
        assert_eq!(plan.stats.unkeyed_fallback_draws, 0);
        assert_eq!(plan.stats.unkeyed_bg_fallback_draws, 0);
    }

    #[test]
    fn stable_effect_art_is_classified_as_material_draw_not_baked_art() {
        let mut frame = ModernFrame::empty();
        let mut bg0 = ModernBgLayer::new(0);
        bg0.enabled_main = true;
        bg0.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: NO_SOURCE_KEY,
            screen_x: 4,
            screen_y: 8,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = bg0;
        let bg_cells = vec![index_cell(0, modern_source_key(1, 3, 5))];
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![bg_entry(3, 5, 0)],
            effects: vec![effect(2)],
            mode7_source_chars: None,
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
            VariantAtlasDraw::MaterialEffect { .. }
        ));
        assert_eq!(plan.stats.effect_draws, 1);
        assert_eq!(plan.stats.effect_material_draws, 1);
        assert_eq!(plan.stats.dynamic_material_draws, 1);
        assert_eq!(plan.stats.dynamic_material_fallback_draws, 0);
        assert_eq!(plan.stats.stable_effect_draws, 0);
        assert_eq!(plan.stats.stable_preview_draws, 0);
    }

    #[test]
    fn bg_instance_source_key_keeps_keyed_cell_on_material_effect_path() {
        let mut frame = ModernFrame::empty();
        let mut bg0 = ModernBgLayer::new(0);
        bg0.enabled_main = true;
        let source_key = modern_source_key(1, 3, 5);
        bg0.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key,
            screen_x: 4,
            screen_y: 8,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = bg0;
        let bg_cells = vec![index_cell(0, source_key)];
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![bg_entry(3, 5, 2)],
            effects: vec![effect(2)],
            mode7_source_chars: None,
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
            VariantAtlasDraw::MaterialEffect { .. }
        ));
        assert_eq!(plan.stats.dynamic_palette_draws, 0);
        assert_eq!(plan.stats.effect_material_draws, 1);
        assert_eq!(plan.stats.dynamic_material_fallback_draws, 0);
        assert_eq!(
            plan.stats.dynamic_material_fallback_instance_source_draws,
            0
        );
        assert_eq!(plan.stats.stable_effect_draws, 0);
        assert_eq!(plan.stats.unkeyed_bg_fallback_draws, 0);
    }

    #[test]
    fn bg_instance_source_key_keeps_exact_stable_art_on_source_backed_path() {
        let mut frame = ModernFrame::empty();
        let mut bg0 = ModernBgLayer::new(0);
        bg0.enabled_main = true;
        let source_key = modern_source_key(1, 3, 5);
        bg0.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key,
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
            effects: vec![],
            mode7_source_chars: None,
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
        assert!(matches!(plan.bg[0].draw, VariantAtlasDraw::Stable { .. }));
        assert_eq!(plan.bg[0].material(), ModernDrawMaterial::RgbaAtlas);
        assert_eq!(plan.stats.stable_draws, 1);
        assert_eq!(plan.stats.stable_preview_draws, 1);
        assert_eq!(plan.stats.dynamic_palette_draws, 0);
        assert_eq!(plan.stats.dynamic_material_fallback_draws, 0);
        assert_eq!(
            plan.stats.dynamic_material_fallback_instance_source_draws,
            0
        );
        assert_eq!(plan.stats.unkeyed_bg_fallback_draws, 0);
    }

    #[test]
    fn non_full_brightness_keeps_stable_sprite_art_on_material_effect_path() {
        let mut frame = ModernFrame::empty();
        frame.brightness = 14;
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 20,
            screen_y: 24,
            palette: 4,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });
        let sprite_cells = vec![index_cell(0, modern_source_key(2, 3, 5))];
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![sprite_entry(3, 5, 4)],
            effects: vec![sprite_effect(4)],
            mode7_source_chars: None,
        };

        let plan = compile_variant_draws(
            &frame,
            &[],
            &sprite_cells,
            &atlas,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(plan.sprites.len(), 1);
        assert!(matches!(
            plan.sprites[0].draw,
            VariantAtlasDraw::MaterialEffect { .. }
        ));
        assert_eq!(plan.stats.dynamic_palette_draws, 0);
        assert_eq!(plan.stats.effect_material_draws, 1);
        assert_eq!(plan.stats.dynamic_material_fallback_draws, 0);
        assert_eq!(plan.stats.dynamic_material_fallback_brightness_draws, 0);
        assert_eq!(plan.stats.stable_effect_draws, 0);
        assert_eq!(plan.stats.unkeyed_sprite_fallback_draws, 0);
    }

    #[test]
    fn traces_material_packet_covering_a_pixel() {
        let mut frame = ModernFrame::empty();
        let mut bg0 = ModernBgLayer::new(0);
        bg0.enabled_main = true;
        bg0.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 10,
            screen_y: 20,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = bg0;
        let mut cell = index_cell(0, modern_source_key(1, 3, 5));
        cell.indices[2 * 8 + 3] = 4;
        let mut effect = effect(2);
        effect.index_to_rgba[4] = [40, 50, 60, 0xff];
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![bg_entry(3, 5, 0)],
            effects: vec![effect],
            mode7_source_chars: None,
        };
        let bg_cells = vec![cell];
        let plan = compile_variant_draws(
            &frame,
            &bg_cells,
            &[],
            &atlas,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        let traces = trace_variant_plan_pixel(&frame, &atlas, &plan, 13, 22);

        assert_eq!(traces.len(), 1);
        let trace = &traces[0];
        assert_eq!(trace.surface, VariantDrawSurface::Bg);
        assert_eq!(trace.material, ModernDrawMaterial::PaletteEffect);
        assert_eq!(trace.local_x, 3);
        assert_eq!(trace.local_y, 2);
        assert_eq!(trace.palette_index, 4);
        assert_eq!(trace.output_rgba, Some([40, 50, 60, 0xff]));
        assert!(trace.describe().contains("material=palette_effect"));
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
            mode7_source_chars: None,
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
