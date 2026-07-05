use crate::gpu_work_item::GpuRenderPlan;
#[cfg(test)]
use crate::gpu_work_item::GpuWorkItemKind;
use crate::modern_assets::ModernTileAtlasAsset;
use crate::modern_bg_renderer::modern_frame_clear_op;
use crate::modern_finalizer::ModernGpuFinalizer;
use crate::modern_frame::ModernFrame;
#[cfg(test)]
use crate::modern_gpu_work_command::ModernGpuWorkCommandKind;
use crate::modern_gpu_work_command::{
    BgEffectMaterialGroup, EffectMaterial, EffectMaterialGroup, ModernGpuCommandLoad,
    ModernGpuWorkCommand, ModernGpuWorkItem, SpriteEffectMaterialGroup,
};
use crate::modern_index_atlas::ModernIndexTile;
use crate::modern_index_renderer::{
    build_index_atlas, ModernGpuIndexRenderer, INDEX_GRID_COLS, INDEX_INSTANCE_STRIDE,
};
use crate::modern_mode1_effect_plan::{
    Mode1EffectCommandSource, PreparedMode1EffectDrawWork, PreparedMode1EffectRenderCommand,
    PreparedMode1EffectRenderPlan,
};
#[cfg(test)]
use crate::modern_mode1_effect_plan::{
    PreparedMode1EffectRankRenderPlan, PreparedMode1EffectRenderCommandKind,
    PreparedMode1EffectRenderStepKind,
};
use crate::modern_screen_builder::ModernGpuScreenBuilder;
use crate::modern_sprite_renderer::ModernGpuSpriteRenderer;
#[cfg(test)]
use crate::modern_variant_render_plan::{headless_variant_render_path, live_variant_render_path};
use crate::modern_variant_render_plan::{
    ModernVariantRenderPath, PreparedModernVariantOutput, PreparedModernVariantRender,
    PreparedModernVariantStats,
};
use std::cell::RefCell;

pub use crate::modern_bg_renderer::ModernGpuRenderer;

pub struct ModernGpuVariantRenderer {
    atlas: crate::modern_variant_atlas::ModernVariantAtlas,
    renderer: ModernGpuRenderer,
    effect_renderer: ModernGpuVariantEffectRenderer,
}

struct PreparedModernVariantExecution<'p, 'frame> {
    prepared: &'p PreparedModernVariantRender<'frame>,
    render_path: ModernVariantRenderPath,
    mode1_effect_draw_work: PreparedMode1EffectDrawWork<'frame>,
    stats: PreparedModernVariantStats,
}

impl<'p, 'frame> PreparedModernVariantExecution<'p, 'frame> {
    fn new(
        prepared: &'p PreparedModernVariantRender<'frame>,
        output: PreparedModernVariantOutput,
    ) -> Self {
        Self {
            prepared,
            render_path: prepared.render_path(output),
            mode1_effect_draw_work: PreparedMode1EffectDrawWork::from_plan(prepared.plan()),
            stats: PreparedModernVariantStats::new(prepared),
        }
    }

    fn frame(&self) -> &'frame ModernFrame {
        self.prepared.frame()
    }

    fn bg_cells(&self) -> &'frame [ModernIndexTile] {
        self.prepared.bg_cells()
    }

    fn sprite_cells(&self) -> &'frame [ModernIndexTile] {
        self.prepared.sprite_cells()
    }

    fn plan(&self) -> &crate::modern_variant_draw::VariantDrawPlan<'frame> {
        self.prepared.plan()
    }

    fn variant_frame(&self) -> &ModernFrame {
        self.prepared.variant_frame()
    }

    fn render_path(&self) -> ModernVariantRenderPath {
        self.render_path
    }

    #[cfg(test)]
    fn mode1_effect_rank_dispatches(&self) -> &[Mode1EffectRankDispatch<'frame>] {
        self.mode1_effect_draw_work.rank_dispatches()
    }

    fn mode1_effect_render_plan<'work>(
        &'work self,
        atlas: &'work crate::modern_variant_atlas::ModernVariantAtlas,
    ) -> PreparedMode1EffectRenderPlan<'work, 'frame> {
        self.mode1_effect_draw_work.render_plan(atlas)
    }

    fn stats(&self) -> &PreparedModernVariantStats {
        &self.stats
    }

    fn stats_mut(&mut self) -> &mut PreparedModernVariantStats {
        &mut self.stats
    }

    fn finish(self) -> crate::modern_software::VariantAtlasRenderStats {
        self.stats.finish()
    }
}

struct LiveIndexVariantBase<'a> {
    frame: &'a ModernFrame,
    bg_cells: &'a [ModernIndexTile],
    sprite_cells: &'a [ModernIndexTile],
}

impl<'a> LiveIndexVariantBase<'a> {
    fn frame(&self) -> &'a ModernFrame {
        self.frame
    }

    fn bg_cells(&self) -> &'a [ModernIndexTile] {
        self.bg_cells
    }

    fn sprite_cells(&self) -> &'a [ModernIndexTile] {
        self.sprite_cells
    }
}

fn debug_variant_missing_key(key: &crate::modern_variant_atlas::VariantAtlasKey) {
    static PRINTED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let Ok(limit) = std::env::var("ZELDA3_VARIANT_DEBUG_MISSING") else {
        return;
    };
    let limit = limit.parse::<usize>().unwrap_or(16);
    let printed = PRINTED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if printed < limit {
        eprintln!(
            "variant_missing source_kind={} asset={} pack={} tile={} bpp={} palette={} row={}",
            key.source_kind, key.asset, key.pack, key.tile, key.bpp, key.palette, key.palette_row
        );
    }
}

impl ModernGpuVariantRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
        format: wgpu::TextureFormat,
    ) -> Self {
        let atlas_asset = ModernTileAtlasAsset {
            tile_width_px: 8,
            tile_height_px: 8,
            atlas_scale: 1,
            width_px: atlas.width,
            height_px: atlas.height,
            rgba: atlas.rgba.clone(),
            entries: Vec::new(),
        };
        Self {
            atlas: atlas.clone(),
            renderer: ModernGpuRenderer::new(device, queue, &atlas_asset, format),
            effect_renderer: ModernGpuVariantEffectRenderer::new(device, queue, atlas, format),
        }
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
        bg_palette_name: &str,
        sprite_palette_name: &str,
        output_view: &wgpu::TextureView,
    ) -> crate::modern_software::VariantAtlasRenderStats {
        let prepared = self.prepare_variant_render(
            frame,
            bg_cells,
            sprite_cells,
            bg_palette_name,
            sprite_palette_name,
        );
        self.render_prepared_variant(device, queue, &prepared, output_view)
    }

    fn render_prepared_variant(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        prepared: &PreparedModernVariantRender<'_>,
        output_view: &wgpu::TextureView,
    ) -> crate::modern_software::VariantAtlasRenderStats {
        let mut execution =
            PreparedModernVariantExecution::new(prepared, PreparedModernVariantOutput::Live);
        self.render_live_execution(device, queue, output_view, &mut execution);
        execution.finish()
    }

    fn render_live_execution(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_view: &wgpu::TextureView,
        execution: &mut PreparedModernVariantExecution<'_, '_>,
    ) {
        match execution.render_path() {
            ModernVariantRenderPath::EffectMaterialMode1Order => {
                self.render_effect_material_mode1_order(device, queue, execution, output_view);
            }
            ModernVariantRenderPath::LiveIndexBaseWithOverlay => {
                self.render_live_index_base_with_overlay(device, queue, output_view, execution);
            }
            ModernVariantRenderPath::EffectMaterialWithStableOverlay => {
                self.render_effect_material_with_stable_overlay(
                    device,
                    queue,
                    execution,
                    output_view,
                );
            }
            ModernVariantRenderPath::StableVariantFrame => {
                self.render_stable_variant_frame(device, queue, execution, output_view);
            }
        }
    }

    fn prepare_variant_render<'a>(
        &'a self,
        frame: &'a ModernFrame,
        bg_cells: &'a [ModernIndexTile],
        sprite_cells: &'a [ModernIndexTile],
        bg_palette_name: &str,
        sprite_palette_name: &str,
    ) -> PreparedModernVariantRender<'a> {
        let plan = crate::modern_variant_draw::compile_variant_draws(
            frame,
            bg_cells,
            sprite_cells,
            &self.atlas,
            bg_palette_name,
            sprite_palette_name,
        );
        let variant_frame = self.build_variant_frame_from_plan(frame, &plan);
        PreparedModernVariantRender::new(frame, bg_cells, sprite_cells, plan, variant_frame)
    }

    fn render_live_index_base_with_overlay(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_view: &wgpu::TextureView,
        execution: &mut PreparedModernVariantExecution<'_, '_>,
    ) {
        let frame = execution.frame();
        let bg_cells = execution.bg_cells();
        let bg = ModernGpuIndexRenderer::new(device, queue, wgpu::TextureFormat::Rgba8Unorm);
        let spr = ModernGpuSpriteRenderer::new(device, queue, wgpu::TextureFormat::Rgba8Unorm);
        bg.render(device, queue, bg_cells, frame, output_view);
        spr.render(device, queue, execution.sprite_cells(), frame, output_view);
        let overlay = mixed_variant_overlay_bg_packets(frame, execution.plan());
        record_live_mixed_overlay_bg_effect_selection_stats(
            execution.stats_mut().as_mut(),
            &overlay,
        );
    }

    fn render_effect_material_with_stable_overlay(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        execution: &PreparedModernVariantExecution<'_, '_>,
        output_view: &wgpu::TextureView,
    ) {
        self.render_effect_material_mode1_order(device, queue, execution, output_view);
        if execution.stats().needs_live_stable_preview_overlay() {
            self.renderer
                .render_overlay(device, queue, execution.variant_frame(), output_view);
        }
    }

    fn render_stable_variant_frame(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        execution: &PreparedModernVariantExecution<'_, '_>,
        output_view: &wgpu::TextureView,
    ) {
        self.renderer
            .render(device, queue, execution.variant_frame(), output_view);
    }

    fn render_effect_material_mode1_order(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        execution: &PreparedModernVariantExecution<'_, '_>,
        output_view: &wgpu::TextureView,
    ) {
        let frame = execution.frame();
        let bg_cells = execution.bg_cells();
        let sprite_cells = execution.sprite_cells();
        let command_plan = execution
            .mode1_effect_render_plan(&self.atlas)
            .into_command_plan();
        command_plan.execute_with(|command| {
            self.render_mode1_effect_command(
                device,
                queue,
                frame,
                bg_cells,
                sprite_cells,
                output_view,
                command,
            );
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn render_mode1_effect_command(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
        output_view: &wgpu::TextureView,
        command: PreparedMode1EffectRenderCommand<'_, '_>,
    ) {
        if let Mode1EffectCommandSource::Rank(rank_index) = command.source {
            debug_assert!(rank_index <= 9);
        }
        render_modern_gpu_work_command(
            &self.effect_renderer,
            device,
            queue,
            frame,
            bg_cells,
            sprite_cells,
            &self.atlas,
            None,
            output_view,
            command.command,
        );
    }

    fn build_variant_frame_from_plan(
        &self,
        frame: &ModernFrame,
        plan: &crate::modern_variant_draw::VariantDrawPlan<'_>,
    ) -> ModernFrame {
        let mut out = ModernFrame::empty();
        out.backdrop_color_rgba = frame.backdrop_color_rgba;
        out.forced_blank = frame.forced_blank;
        if frame.forced_blank {
            return out;
        }

        out.bg_layers[0].enabled_main = true;
        out.bg_layers[1].enabled_main = true;
        for packet in plan.material_packets() {
            match packet.draw() {
                crate::modern_variant_atlas::VariantAtlasDraw::Stable { entry } => {
                    let target_layer = match packet.surface() {
                        crate::modern_variant_draw::VariantDrawSurface::Bg => 0,
                        crate::modern_variant_draw::VariantDrawSurface::Sprite => 1,
                    };
                    let (screen_x, screen_y) = packet.screen_origin();
                    let (hflip, vflip) = packet.source_flip_with_entry(entry);
                    out.bg_layers[target_layer]
                        .tiles
                        .push(variant_tile_instance(
                            entry, screen_x, screen_y, hflip, vflip,
                        ));
                }
                crate::modern_variant_atlas::VariantAtlasDraw::MaterialEffect { .. } => {}
                crate::modern_variant_atlas::VariantAtlasDraw::DynamicPalette { .. } => {}
                crate::modern_variant_atlas::VariantAtlasDraw::MissingArt => {
                    if let Some(key) = packet.key() {
                        debug_variant_missing_key(key);
                    }
                }
                crate::modern_variant_atlas::VariantAtlasDraw::Unkeyed => {}
            }
        }

        out
    }
}

#[allow(clippy::too_many_arguments)]
fn render_modern_gpu_work_item(
    effect_renderer: &ModernGpuVariantEffectRenderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
    atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
    bg_effect_frame: Option<&ModernFrame>,
    output_view: &wgpu::TextureView,
    work_item: ModernGpuWorkItem<'_, '_>,
    bg_load: wgpu::LoadOp<wgpu::Color>,
) {
    match work_item {
        ModernGpuWorkItem::ClearBackdrop => {
            effect_renderer.render_bg(
                device,
                queue,
                bg_cells,
                atlas,
                &[],
                output_view,
                modern_frame_clear_op(frame),
            );
        }
        ModernGpuWorkItem::BgEffect(group) => {
            effect_renderer.render_bg_material_group(
                device,
                queue,
                bg_cells,
                bg_effect_frame,
                atlas,
                group,
                output_view,
                bg_load,
            );
        }
        ModernGpuWorkItem::SpriteEffects(sprite_groups) => {
            effect_renderer.render_sprite_material_groups(
                device,
                queue,
                sprite_cells,
                frame,
                atlas,
                &sprite_groups,
                output_view,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_modern_gpu_work_command(
    effect_renderer: &ModernGpuVariantEffectRenderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
    atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
    bg_effect_frame: Option<&ModernFrame>,
    output_view: &wgpu::TextureView,
    command: ModernGpuWorkCommand<'_, '_>,
) {
    let bg_load = match command.target_load {
        ModernGpuCommandLoad::ClearFrame => modern_frame_clear_op(frame),
        ModernGpuCommandLoad::Load => wgpu::LoadOp::Load,
    };
    render_modern_gpu_work_item(
        effect_renderer,
        device,
        queue,
        frame,
        bg_cells,
        sprite_cells,
        atlas,
        bg_effect_frame,
        output_view,
        command.work_item,
        bg_load,
    );
}

struct ModernGpuVariantEffectRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    #[allow(dead_code)]
    effect_lut_texture: wgpu::Texture,
    effect_lut_view: wgpu::TextureView,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EffectSurface {
    Bg,
    Sprite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EffectMaterialPacket {
    surface: EffectSurface,
    material: EffectMaterial,
    effect_row: u32,
    instance: EffectInstancePacket,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EffectInstancePacket {
    cell_id: u32,
    screen_x: i16,
    screen_y: i16,
    row_mask: u8,
    hflip: bool,
    vflip: bool,
    source_hflip: bool,
    source_vflip: bool,
    effect_row: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct Mode1EffectRankDispatch<'a> {
    pub(crate) bg: Vec<crate::modern_variant_draw::VariantBgDrawPacket<'a>>,
    pub(crate) sprites: Vec<crate::modern_variant_draw::VariantSpriteDrawPacket<'a>>,
}

pub(crate) type Mode1EffectRankRenderPlan<'rank, 'frame> =
    GpuRenderPlan<ModernGpuWorkCommand<'rank, 'frame>>;

impl<'a> Mode1EffectRankDispatch<'a> {
    fn empty() -> Self {
        Self {
            bg: Vec::new(),
            sprites: Vec::new(),
        }
    }

    fn bg_material_groups(&self) -> impl Iterator<Item = BgEffectMaterialGroup<'_, 'a>> {
        [EffectMaterialGroup {
            material: EffectMaterial::StaticEffect,
            packets: self.bg.as_slice(),
        }]
        .into_iter()
        .filter(|group| !group.packets.is_empty())
    }

    fn sprite_material_groups(
        &self,
        atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
    ) -> Vec<SpriteEffectMaterialGroup<'_, 'a>> {
        sprite_effect_material_groups(atlas, &self.sprites)
    }

    pub(crate) fn render_plan(
        &self,
        atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
        rendered_any: bool,
    ) -> Mode1EffectRankRenderPlan<'_, 'a> {
        let bg_groups = self.bg_material_groups().collect::<Vec<_>>();
        let sprite_groups = self.sprite_material_groups(atlas);
        let clear_before_sprites =
            !rendered_any && bg_groups.is_empty() && !sprite_groups.is_empty();
        let mut work_items = Vec::new();
        if clear_before_sprites {
            work_items.push(ModernGpuWorkItem::ClearBackdrop);
        }
        work_items.extend(bg_groups.into_iter().map(ModernGpuWorkItem::BgEffect));
        if !sprite_groups.is_empty() {
            work_items.push(ModernGpuWorkItem::SpriteEffects(sprite_groups));
        }
        let mut rendered_any = rendered_any;
        let commands = work_items
            .into_iter()
            .map(|work_item| {
                let target_load = if rendered_any {
                    ModernGpuCommandLoad::Load
                } else {
                    ModernGpuCommandLoad::ClearFrame
                };
                rendered_any = true;
                ModernGpuWorkCommand {
                    target_load,
                    work_item,
                }
            })
            .collect();
        GpuRenderPlan::new(commands)
    }
}

pub(crate) fn mode1_effect_rank_dispatches<'a>(
    plan: &crate::modern_variant_draw::VariantDrawPlan<'a>,
) -> Vec<Mode1EffectRankDispatch<'a>> {
    let mut ranks = (0..=9)
        .map(|_| Mode1EffectRankDispatch::empty())
        .collect::<Vec<_>>();
    for packet in plan.material_packets() {
        let Some(rank) = packet.mode1_rank() else {
            continue;
        };
        match packet {
            crate::modern_variant_draw::VariantDrawPacket::Bg { packet, .. } => {
                ranks[usize::from(rank)].bg.push(packet.clone());
            }
            crate::modern_variant_draw::VariantDrawPacket::Sprite { packet, .. } => {
                ranks[usize::from(rank)].sprites.push(packet.clone());
            }
        }
    }
    ranks
}

#[derive(Default)]
struct OverlayBgEffectDispatch<'a> {
    static_bg: Vec<crate::modern_variant_draw::VariantBgDrawPacket<'a>>,
    live_cgram_bg: Vec<crate::modern_variant_draw::VariantBgDrawPacket<'a>>,
}

impl<'a> OverlayBgEffectDispatch<'a> {
    fn material_groups(&self) -> impl Iterator<Item = BgEffectMaterialGroup<'_, 'a>> {
        [
            EffectMaterialGroup {
                material: EffectMaterial::StaticEffect,
                packets: &self.static_bg,
            },
            EffectMaterialGroup {
                material: EffectMaterial::LiveCgram,
                packets: &self.live_cgram_bg,
            },
        ]
        .into_iter()
        .filter(|group| !group.packets.is_empty())
    }

    #[cfg(test)]
    fn render_plan(&self) -> GpuRenderPlan<ModernGpuWorkItem<'_, 'a>> {
        GpuRenderPlan::new(
            self.material_groups()
                .map(ModernGpuWorkItem::BgEffect)
                .collect(),
        )
    }

    #[cfg(test)]
    fn static_bg_len(&self) -> usize {
        self.material_group_len(EffectMaterial::StaticEffect)
    }

    #[cfg(test)]
    fn live_cgram_bg_len(&self) -> usize {
        self.material_group_len(EffectMaterial::LiveCgram)
    }

    #[cfg(test)]
    fn static_bg_packets(&self) -> &[crate::modern_variant_draw::VariantBgDrawPacket<'a>] {
        self.material_group_packets(EffectMaterial::StaticEffect)
    }

    #[cfg(test)]
    fn material_group_len(&self, material: EffectMaterial) -> usize {
        self.material_groups()
            .find(|group| group.material == material)
            .map_or(0, |group| group.packets.len())
    }

    #[cfg(test)]
    fn material_group_packets(
        &self,
        material: EffectMaterial,
    ) -> &[crate::modern_variant_draw::VariantBgDrawPacket<'a>] {
        self.material_groups()
            .find(|group| group.material == material)
            .map_or(&[], |group| group.packets)
    }

    fn prefinal_bg_packets(
        &self,
        mut include: impl FnMut(&crate::modern_variant_draw::VariantBgDrawPacket<'a>) -> bool,
    ) -> Vec<MixedVariantPrefinalBgPacket<'a>> {
        let mut packets = Vec::new();
        for group in self.material_groups() {
            let material = PrefinalBgMaterial::from_effect_material(group.material);
            packets.extend(
                group
                    .packets
                    .iter()
                    .filter(|packet| include(packet))
                    .cloned()
                    .map(|packet| MixedVariantPrefinalBgPacket { material, packet }),
            );
        }
        packets
    }
}

impl PrefinalBgMaterial {
    fn from_effect_material(material: EffectMaterial) -> Self {
        match material {
            EffectMaterial::StaticEffect => Self::StaticEffect,
            EffectMaterial::LiveCgram => Self::LiveCgram,
        }
    }
}

#[derive(Default)]
struct MixedVariantOverlayBgSelection<'a> {
    effects: OverlayBgEffectDispatch<'a>,
    candidates: u32,
    culled_invisible_main: u32,
    reject_complex_frame: u32,
    reject_complex_brightness: u32,
    reject_complex_invalid_layer: u32,
    reject_complex_mosaic: u32,
    reject_complex_sub_window: u32,
    reject_complex_effect_bounds: u32,
    reject_complex_scanline_main: u32,
    reject_complex_layer_window: u32,
    reject_complex_color_math: u32,
    reject_complex_color_math_clip: u32,
    reject_complex_color_math_subscreen: u32,
    reject_complex_color_math_fixed_color: u32,
    reject_cgram_mismatch: u32,
    reject_overlap: u32,
    reject_overlap_bg: u32,
    reject_overlap_obj: u32,
    reject_overlap_bg_deeper_chain: u32,
    reject_overlap_bg_unrepresentable_front: u32,
    reject_overlap_bg_mixed_static_live_order: u32,
    reject_overlap_bg_unrepresentable_front_no_effect: u32,
    reject_overlap_bg_unrepresentable_front_complex: u32,
    reject_overlap_bg_unrepresentable_front_cgram_mismatch: u32,
}

fn record_live_mixed_overlay_bg_effect_selection_stats(
    stats: &mut crate::modern_software::VariantAtlasRenderStats,
    overlay: &MixedVariantOverlayBgSelection<'_>,
) {
    stats.mixed_overlay_bg_effect_candidates += overlay.candidates;
    stats.mixed_overlay_bg_effect_culled_invisible_main += overlay.culled_invisible_main;
    stats.mixed_overlay_bg_effect_reject_complex_frame += overlay.reject_complex_frame;
    stats.mixed_overlay_bg_effect_reject_complex_brightness += overlay.reject_complex_brightness;
    stats.mixed_overlay_bg_effect_reject_complex_invalid_layer +=
        overlay.reject_complex_invalid_layer;
    stats.mixed_overlay_bg_effect_reject_complex_mosaic += overlay.reject_complex_mosaic;
    stats.mixed_overlay_bg_effect_reject_complex_sub_window += overlay.reject_complex_sub_window;
    stats.mixed_overlay_bg_effect_reject_complex_effect_bounds +=
        overlay.reject_complex_effect_bounds;
    stats.mixed_overlay_bg_effect_reject_complex_scanline_main +=
        overlay.reject_complex_scanline_main;
    stats.mixed_overlay_bg_effect_reject_complex_layer_window +=
        overlay.reject_complex_layer_window;
    stats.mixed_overlay_bg_effect_reject_complex_color_math += overlay.reject_complex_color_math;
    stats.mixed_overlay_bg_effect_reject_complex_color_math_clip +=
        overlay.reject_complex_color_math_clip;
    stats.mixed_overlay_bg_effect_reject_complex_color_math_subscreen +=
        overlay.reject_complex_color_math_subscreen;
    stats.mixed_overlay_bg_effect_reject_complex_color_math_fixed_color +=
        overlay.reject_complex_color_math_fixed_color;
    stats.mixed_overlay_bg_effect_reject_cgram_mismatch += overlay.reject_cgram_mismatch;
    stats.mixed_overlay_bg_effect_reject_overlap += overlay.reject_overlap;
}

fn record_headless_live_index_overlay_stats(
    stats: &mut crate::modern_software::VariantAtlasRenderStats,
    frame: &ModernFrame,
    overlay: &MixedVariantOverlayBgSelection<'_>,
    prefinal_packets: &MixedVariantPrefinalPackets<'_>,
) {
    let prefinal_color_math = prefinal_packets
        .bg_packets()
        .filter_map(|packet| bg_packet_prefinal_color_math_reason(frame, packet))
        .collect::<Vec<_>>();
    let accepted_prefinal_color_math = prefinal_color_math.len() as u32;
    let accepted_prefinal_color_math_clip = prefinal_color_math
        .iter()
        .filter(|reason| **reason == MixedOverlayComplexRejectReason::ColorMathClip)
        .count() as u32;
    let accepted_prefinal_color_math_subscreen = prefinal_color_math
        .iter()
        .filter(|reason| **reason == MixedOverlayComplexRejectReason::ColorMathSubscreen)
        .count() as u32;
    let accepted_prefinal_color_math_fixed_color = prefinal_color_math
        .iter()
        .filter(|reason| **reason == MixedOverlayComplexRejectReason::ColorMathFixedColor)
        .count() as u32;
    stats.mixed_overlay_bg_effect_draws += prefinal_packets.bg_len() as u32;
    stats.mixed_overlay_bg_effect_candidates += overlay.candidates;
    stats.mixed_overlay_bg_effect_culled_invisible_main += overlay.culled_invisible_main;
    stats.mixed_overlay_bg_effect_reject_complex_frame += overlay
        .reject_complex_frame
        .saturating_sub(accepted_prefinal_color_math);
    stats.mixed_overlay_bg_effect_reject_complex_brightness += overlay.reject_complex_brightness;
    stats.mixed_overlay_bg_effect_reject_complex_invalid_layer +=
        overlay.reject_complex_invalid_layer;
    stats.mixed_overlay_bg_effect_reject_complex_mosaic += overlay.reject_complex_mosaic;
    stats.mixed_overlay_bg_effect_reject_complex_sub_window += overlay.reject_complex_sub_window;
    stats.mixed_overlay_bg_effect_reject_complex_effect_bounds +=
        overlay.reject_complex_effect_bounds;
    stats.mixed_overlay_bg_effect_reject_complex_scanline_main +=
        overlay.reject_complex_scanline_main;
    stats.mixed_overlay_bg_effect_reject_complex_layer_window +=
        overlay.reject_complex_layer_window;
    stats.mixed_overlay_bg_effect_reject_complex_color_math += overlay
        .reject_complex_color_math
        .saturating_sub(accepted_prefinal_color_math);
    stats.mixed_overlay_bg_effect_reject_complex_color_math_clip += overlay
        .reject_complex_color_math_clip
        .saturating_sub(accepted_prefinal_color_math_clip);
    stats.mixed_overlay_bg_effect_reject_complex_color_math_subscreen += overlay
        .reject_complex_color_math_subscreen
        .saturating_sub(accepted_prefinal_color_math_subscreen);
    stats.mixed_overlay_bg_effect_reject_complex_color_math_fixed_color += overlay
        .reject_complex_color_math_fixed_color
        .saturating_sub(accepted_prefinal_color_math_fixed_color);
    stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch +=
        overlay.reject_cgram_mismatch;
    stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap +=
        overlay.reject_overlap;
    stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg +=
        overlay.reject_overlap_bg;
    stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj +=
        overlay.reject_overlap_obj;
    stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain +=
        overlay.reject_overlap_bg_deeper_chain;
    stats
        .mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front +=
        overlay.reject_overlap_bg_unrepresentable_front;
    stats
        .mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order +=
        overlay.reject_overlap_bg_mixed_static_live_order;
    stats
        .mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect +=
        overlay.reject_overlap_bg_unrepresentable_front_no_effect;
    stats
        .mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex +=
        overlay.reject_overlap_bg_unrepresentable_front_complex;
    stats
        .mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch +=
        overlay.reject_overlap_bg_unrepresentable_front_cgram_mismatch;
    stats.mixed_overlay_bg_effect_reject_cgram_mismatch += overlay.reject_cgram_mismatch;
    stats.mixed_overlay_bg_effect_reject_overlap += overlay.reject_overlap;
}

#[derive(Clone, Debug, Default)]
struct MixedVariantPrefinalPackets<'a> {
    bg: Vec<MixedVariantPrefinalBgPacket<'a>>,
    sprites: Vec<crate::modern_variant_draw::VariantSpriteDrawPacket<'a>>,
}

#[derive(Clone, Debug)]
struct MixedVariantPrefinalBgPacket<'a> {
    material: PrefinalBgMaterial,
    packet: crate::modern_variant_draw::VariantBgDrawPacket<'a>,
}

impl<'a> MixedVariantPrefinalPackets<'a> {
    fn from_overlay(
        frame: &ModernFrame,
        overlay: &MixedVariantOverlayBgSelection<'a>,
        plan: &crate::modern_variant_draw::VariantDrawPlan<'a>,
    ) -> Self {
        Self {
            bg: overlay
                .effects
                .prefinal_bg_packets(|packet| bg_packet_needs_prefinal_color_math(frame, packet)),
            sprites: plan.sprites.clone(),
        }
    }

    fn from_all_overlay(
        overlay: &MixedVariantOverlayBgSelection<'a>,
        plan: &crate::modern_variant_draw::VariantDrawPlan<'a>,
    ) -> Self {
        Self {
            bg: overlay.effects.prefinal_bg_packets(|_| true),
            sprites: plan.sprites.clone(),
        }
    }

    #[cfg(test)]
    fn static_bg_len(&self) -> usize {
        self.bg
            .iter()
            .filter(|packet| packet.material == PrefinalBgMaterial::StaticEffect)
            .count()
    }

    #[cfg(test)]
    fn live_cgram_bg_len(&self) -> usize {
        self.bg
            .iter()
            .filter(|packet| packet.material == PrefinalBgMaterial::LiveCgram)
            .count()
    }

    fn is_bg_empty(&self) -> bool {
        self.bg.is_empty()
    }

    fn bg_len(&self) -> usize {
        self.bg.len()
    }

    fn bg_packets(
        &self,
    ) -> impl Iterator<Item = &crate::modern_variant_draw::VariantBgDrawPacket<'a>> {
        self.bg.iter().map(|packet| &packet.packet)
    }

    fn bg_material_packets(&self) -> impl Iterator<Item = &MixedVariantPrefinalBgPacket<'a>> {
        self.bg.iter()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MixedOverlayComplexRejectReason {
    Brightness,
    InvalidLayer,
    Mosaic,
    SubWindow,
    EffectBounds,
    InvisibleMain,
    LayerWindow,
    ColorMathClip,
    ColorMathSubscreen,
    ColorMathFixedColor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MixedOverlayOverlapRejectReason {
    Bg,
    BgDeeperChain,
    BgUnrepresentableFront,
    BgUnrepresentableFrontNoEffect,
    BgUnrepresentableFrontComplex,
    BgUnrepresentableFrontCgramMismatch,
    BgMixedStaticLiveOrder,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PrefinalBgMaterial {
    StaticEffect,
    LiveCgram,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PrefinalBgMaterialRejectReason {
    NoEffect,
    Complex,
    CgramMismatch,
}

impl<'a> MixedVariantOverlayBgSelection<'a> {
    fn record_complex_reject(&mut self, reason: MixedOverlayComplexRejectReason) {
        if reason == MixedOverlayComplexRejectReason::InvisibleMain {
            self.culled_invisible_main += 1;
            return;
        }
        self.reject_complex_frame += 1;
        match reason {
            MixedOverlayComplexRejectReason::Brightness => self.reject_complex_brightness += 1,
            MixedOverlayComplexRejectReason::InvalidLayer => {
                self.reject_complex_invalid_layer += 1;
            }
            MixedOverlayComplexRejectReason::Mosaic => self.reject_complex_mosaic += 1,
            MixedOverlayComplexRejectReason::SubWindow => self.reject_complex_sub_window += 1,
            MixedOverlayComplexRejectReason::EffectBounds => {
                self.reject_complex_effect_bounds += 1;
            }
            MixedOverlayComplexRejectReason::InvisibleMain => unreachable!(),
            MixedOverlayComplexRejectReason::LayerWindow => {
                self.reject_complex_layer_window += 1;
            }
            MixedOverlayComplexRejectReason::ColorMathClip => {
                self.reject_complex_color_math += 1;
                self.reject_complex_color_math_clip += 1;
            }
            MixedOverlayComplexRejectReason::ColorMathSubscreen => {
                self.reject_complex_color_math += 1;
                self.reject_complex_color_math_subscreen += 1;
            }
            MixedOverlayComplexRejectReason::ColorMathFixedColor => {
                self.reject_complex_color_math += 1;
                self.reject_complex_color_math_fixed_color += 1;
            }
        }
    }

    fn record_overlap_reject(&mut self, reason: MixedOverlayOverlapRejectReason) {
        self.reject_overlap += 1;
        match reason {
            MixedOverlayOverlapRejectReason::Bg => self.reject_overlap_bg += 1,
            MixedOverlayOverlapRejectReason::BgDeeperChain => {
                self.reject_overlap_bg += 1;
                self.reject_overlap_bg_deeper_chain += 1;
            }
            MixedOverlayOverlapRejectReason::BgUnrepresentableFront => {
                self.reject_overlap_bg += 1;
                self.reject_overlap_bg_unrepresentable_front += 1;
            }
            MixedOverlayOverlapRejectReason::BgUnrepresentableFrontNoEffect => {
                self.reject_overlap_bg += 1;
                self.reject_overlap_bg_unrepresentable_front += 1;
                self.reject_overlap_bg_unrepresentable_front_no_effect += 1;
            }
            MixedOverlayOverlapRejectReason::BgUnrepresentableFrontComplex => {
                self.reject_overlap_bg += 1;
                self.reject_overlap_bg_unrepresentable_front += 1;
                self.reject_overlap_bg_unrepresentable_front_complex += 1;
            }
            MixedOverlayOverlapRejectReason::BgUnrepresentableFrontCgramMismatch => {
                self.reject_overlap_bg += 1;
                self.reject_overlap_bg_unrepresentable_front += 1;
                self.reject_overlap_bg_unrepresentable_front_cgram_mismatch += 1;
            }
            MixedOverlayOverlapRejectReason::BgMixedStaticLiveOrder => {
                self.reject_overlap_bg += 1;
                self.reject_overlap_bg_mixed_static_live_order += 1;
            }
        }
    }
}

fn mixed_variant_overlay_bg_packets<'a>(
    frame: &ModernFrame,
    plan: &crate::modern_variant_draw::VariantDrawPlan<'a>,
) -> MixedVariantOverlayBgSelection<'a> {
    mixed_variant_overlay_bg_packets_with_policy(frame, plan, false)
}

fn mixed_variant_prefinal_bg_packets<'a>(
    frame: &ModernFrame,
    plan: &crate::modern_variant_draw::VariantDrawPlan<'a>,
) -> MixedVariantOverlayBgSelection<'a> {
    mixed_variant_overlay_bg_packets_with_policy(frame, plan, true)
}

fn mixed_variant_overlay_bg_packets_with_policy<'a>(
    frame: &ModernFrame,
    plan: &crate::modern_variant_draw::VariantDrawPlan<'a>,
    allow_color_math: bool,
) -> MixedVariantOverlayBgSelection<'a> {
    let mut out = MixedVariantOverlayBgSelection::default();
    let candidates = plan
        .material_packets()
        .filter_map(|packet| packet.as_bg())
        .filter(|(_, packet)| packet.draw.material_effect().is_some())
        .count() as u32;
    out.candidates = candidates;

    for (packet_index, packet) in plan.material_packets().filter_map(|packet| packet.as_bg()) {
        let Some((entry, effect)) = packet.draw.material_effect() else {
            continue;
        };
        if let Some(reason) =
            bg_effect_packet_complex_reject_reason(frame, packet, entry, effect, allow_color_math)
        {
            out.record_complex_reject(reason);
            continue;
        }
        let overlap_reject = if allow_color_math {
            bg_packet_prefinal_overlap_reject_reason(frame, packet_index, packet, plan)
        } else if bg_packet_overlaps_other_packets(frame, packet_index, packet, plan) {
            Some(MixedOverlayOverlapRejectReason::Bg)
        } else {
            None
        };
        if let Some(reason) = overlap_reject {
            out.record_overlap_reject(reason);
            continue;
        }
        if bg_effect_matches_live_cgram(packet.cell, packet.inst.palette, effect, frame) {
            out.effects.static_bg.push(packet.clone());
            continue;
        }
        if bg_packet_can_use_live_cgram(packet, frame) {
            out.effects.live_cgram_bg.push(packet.clone());
            continue;
        } else {
            out.reject_cgram_mismatch += 1;
        }
    }
    out
}

fn bg_effect_packet_complex_reject_reason(
    frame: &ModernFrame,
    packet: &crate::modern_variant_draw::VariantBgDrawPacket<'_>,
    entry: &crate::modern_variant_atlas::VariantAtlasEntry,
    effect: &crate::modern_variant_atlas::TileEffect,
    allow_color_math: bool,
) -> Option<MixedOverlayComplexRejectReason> {
    if frame.brightness != 15 {
        return Some(MixedOverlayComplexRejectReason::Brightness);
    }
    let Ok(layer) = u8::try_from(packet.layer_index) else {
        return Some(MixedOverlayComplexRejectReason::InvalidLayer);
    };
    if layer >= 4 {
        return Some(MixedOverlayComplexRejectReason::InvalidLayer);
    }
    let layer_bit = 1u8 << layer;
    if frame.mosaic_size > 1 && (frame.mosaic_enabled & layer_bit) != 0 {
        return Some(MixedOverlayComplexRejectReason::Mosaic);
    }
    if frame.screen_windowed_sub != 0 {
        return Some(MixedOverlayComplexRejectReason::SubWindow);
    }

    let mut saw_visible_pixel = false;
    let mut saw_scanline_disabled_pixel = false;
    let mut saw_layer_window_pixel = false;
    for y in 0..8usize {
        for x in 0..8usize {
            let src_x = if packet.cell.hflip ^ entry.source_hflip {
                7 - x
            } else {
                x
            };
            let src_y = if packet.cell.vflip ^ entry.source_vflip {
                7 - y
            } else {
                y
            };
            let index = packet.cell.indices[src_y * 8 + src_x];
            if index == 0 {
                continue;
            }
            let dst_x = packet.inst.screen_x + x as i16;
            let dst_y = packet.inst.screen_y + y as i16;
            if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                continue;
            }
            let sx = dst_x as u32;
            let sy = dst_y as usize;
            if frame.screen_enabled_main != 0 && frame.screen_enabled_main & layer_bit == 0 {
                saw_scanline_disabled_pixel = true;
                continue;
            }
            if frame
                .main_tm_scanlines
                .get(sy)
                .is_some_and(|tm| tm & layer_bit == 0)
            {
                saw_scanline_disabled_pixel = true;
                continue;
            }
            if bg_layer_window_masks_packet_pixel(frame, layer, sx, sy) {
                saw_layer_window_pixel = true;
                continue;
            }
            saw_visible_pixel = true;
            if usize::from(index) >= effect.colors_per_row as usize
                || usize::from(index) >= effect.index_to_rgba.len()
            {
                return Some(MixedOverlayComplexRejectReason::EffectBounds);
            }
            if let Some(reason) = bg_packet_pixel_math_reject_reason(frame, layer, sx, sy) {
                if allow_color_math
                    && matches!(
                        reason,
                        MixedOverlayComplexRejectReason::ColorMathClip
                            | MixedOverlayComplexRejectReason::ColorMathSubscreen
                            | MixedOverlayComplexRejectReason::ColorMathFixedColor
                    )
                {
                    continue;
                }
                return Some(reason);
            }
        }
    }

    if !saw_visible_pixel {
        if saw_scanline_disabled_pixel {
            return Some(MixedOverlayComplexRejectReason::InvisibleMain);
        }
        if saw_layer_window_pixel {
            return Some(MixedOverlayComplexRejectReason::LayerWindow);
        }
    }

    None
}

fn bg_layer_window_masks_packet_pixel(frame: &ModernFrame, layer: u8, sx: u32, sy: usize) -> bool {
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

fn bg_packet_pixel_math_reject_reason(
    frame: &ModernFrame,
    layer: u8,
    sx: u32,
    sy: usize,
) -> Option<MixedOverlayComplexRejectReason> {
    if frame.clip_mode == 0
        && frame.prevent_math_mode == 3
        && frame.windowsel_cm == 0
        && frame.math_enabled & (1u8 << layer) == 0
    {
        return None;
    }

    let win = frame.window_scanlines.get(sy).copied().unwrap_or([0u8; 4]);
    let cm_window = bg_packet_in_color_math_window(sx, win, frame.windowsel_cm);
    if !bg_packet_color_window_bit(cm_window, frame.clip_mode) {
        return Some(MixedOverlayComplexRejectReason::ColorMathClip);
    }
    if frame.math_enabled & (1u8 << layer) == 0 {
        return None;
    }
    if !bg_packet_color_window_bit(cm_window, frame.prevent_math_mode) {
        return None;
    }
    if frame.add_subscreen {
        return Some(MixedOverlayComplexRejectReason::ColorMathSubscreen);
    }
    if frame.fixed_color_r == 0
        && frame.fixed_color_g == 0
        && frame.fixed_color_b == 0
        && !frame.half_color
    {
        None
    } else {
        Some(MixedOverlayComplexRejectReason::ColorMathFixedColor)
    }
}

fn bg_packet_color_window_bit(in_window: bool, mode: u8) -> bool {
    const MASKS: [u32; 8] = [0x00, 0xff, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00];
    let w = if in_window { 0xffu32 } else { 0 };
    let m = mode as usize & 7;
    ((w & MASKS[m]) ^ MASKS[m + 4]) != 0
}

fn bg_packet_in_color_math_window(sx: u32, win: [u8; 4], windowsel_cm: u8) -> bool {
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

fn frame_needs_material_prefinal_finalizer(frame: &ModernFrame) -> bool {
    if frame.brightness != 15 || frame.clip_mode != 0 {
        return true;
    }
    if frame.math_enabled == 0 {
        return false;
    }
    frame.add_subscreen
        || frame.half_color
        || frame.fixed_color_r != 0
        || frame.fixed_color_g != 0
        || frame.fixed_color_b != 0
        || frame.windowsel_cm != 0
        || frame.prevent_math_mode != 3
}

fn frame_uses_direct_final_index_math(frame: &ModernFrame) -> bool {
    frame.screen_enabled_sub == 0
        && frame.math_enabled == 0
        && !frame.subtract_color
        && !frame.half_color
        && frame.fixed_color_r == 0
        && frame.fixed_color_g == 0
        && frame.fixed_color_b == 0
        && !frame.add_subscreen
        && frame.clip_mode == 0
        && frame.prevent_math_mode == 0
        && frame.windowsel_cm == 0
}

fn can_render_final_index_base_gpu(frame: &ModernFrame, bg_cells: &[ModernIndexTile]) -> bool {
    if frame.forced_blank
        || !frame_uses_direct_final_index_math(frame)
        || frame.windowsel != 0
        || frame.screen_windowed_main != 0
        || frame.screen_windowed_sub != 0
        || frame.mosaic_enabled != 0
        || frame.mosaic_size > 1
        || !frame.bg_scroll_scanlines.is_empty()
        || !frame.index_sprites.is_empty()
    {
        return false;
    }

    let enabled_layers = frame
        .bg_layers
        .iter()
        .filter(|layer| layer.enabled_main)
        .collect::<Vec<_>>();
    if enabled_layers.is_empty() {
        return false;
    }
    let enabled_mask = enabled_layers
        .iter()
        .fold(0u8, |mask, layer| mask | (1u8 << layer.index));
    if frame.screen_enabled_main != enabled_mask {
        return false;
    }
    if frame
        .main_tm_scanlines
        .iter()
        .any(|tm| (tm & enabled_mask) != enabled_mask)
    {
        return false;
    }

    enabled_layers
        .iter()
        .all(|layer| layer.scroll_x == 0 && layer.scroll_y == 0)
        && bg_layers_have_no_opaque_overlap(&enabled_layers, bg_cells)
}

fn bg_layers_have_no_opaque_overlap(
    layers: &[&crate::modern_frame::ModernBgLayer],
    bg_cells: &[ModernIndexTile],
) -> bool {
    let mut occupied = vec![
        false;
        usize::from(crate::modern_frame::MODERN_FRAME_WIDTH)
            * usize::from(crate::modern_frame::MODERN_FRAME_HEIGHT)
    ];
    let width = usize::from(crate::modern_frame::MODERN_FRAME_WIDTH);
    for layer in layers {
        for inst in &layer.index_tiles {
            let Some(cell) = bg_cells.get(inst.cell_id as usize) else {
                continue;
            };
            for y in 0..8usize {
                for x in 0..8usize {
                    if cell.indices[y * 8 + x] == 0 {
                        continue;
                    }
                    let dst_x = inst.screen_x + x as i16;
                    let dst_y = inst.screen_y + y as i16;
                    if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                        continue;
                    }
                    let offset = dst_y as usize * width + dst_x as usize;
                    if occupied[offset] {
                        return false;
                    }
                    occupied[offset] = true;
                }
            }
        }
    }
    true
}

fn finalize_snes_5bit_channel(channel: u8, brightness: u8) -> u8 {
    let c5 = channel >> 3;
    let expanded = (c5 << 3) | (c5 >> 2);
    ((u32::from(expanded) * u32::from(brightness)) / 15) as u8
}

fn finalize_modern_frame_colors_for_direct_index(frame: &mut ModernFrame) {
    for color in &mut frame.cgram_rgba {
        color[0] = finalize_snes_5bit_channel(color[0], frame.brightness);
        color[1] = finalize_snes_5bit_channel(color[1], frame.brightness);
        color[2] = finalize_snes_5bit_channel(color[2], frame.brightness);
    }
    frame.backdrop_color_rgba[0] =
        finalize_snes_5bit_channel(frame.backdrop_color_rgba[0], frame.brightness);
    frame.backdrop_color_rgba[1] =
        finalize_snes_5bit_channel(frame.backdrop_color_rgba[1], frame.brightness);
    frame.backdrop_color_rgba[2] =
        finalize_snes_5bit_channel(frame.backdrop_color_rgba[2], frame.brightness);
    frame.brightness = 15;
}

fn bg_effect_matches_live_cgram(
    cell: &ModernIndexTile,
    palette: u8,
    effect: &crate::modern_variant_atlas::TileEffect,
    frame: &ModernFrame,
) -> bool {
    let palette_base = usize::from(palette) * 16;
    for index in cell.indices {
        if index == 0 {
            continue;
        }
        let index = usize::from(index);
        if index >= effect.colors_per_row as usize || index >= effect.index_to_rgba.len() {
            return false;
        }
        let cgram_index = palette_base + index;
        let Some(live) = frame.cgram_rgba.get(cgram_index) else {
            return false;
        };
        let effect_color = effect.index_to_rgba[index];
        if effect_color[0..3] != live[0..3] {
            return false;
        }
    }
    true
}

fn bg_packet_can_use_live_cgram(
    packet: &crate::modern_variant_draw::VariantBgDrawPacket<'_>,
    frame: &ModernFrame,
) -> bool {
    let palette_base = usize::from(packet.inst.palette) * EFFECT_LUT_WIDTH as usize;
    for index in packet.cell.indices {
        if index == 0 {
            continue;
        }
        if usize::from(index) >= EFFECT_LUT_WIDTH as usize {
            return false;
        }
        if frame
            .cgram_rgba
            .get(palette_base + usize::from(index))
            .is_none()
        {
            return false;
        }
    }
    true
}

#[cfg(test)]
fn overlay_mixed_variant_bg_packets_on_main_screen(
    screens: &mut crate::modern_software::ModernCompositedScreens,
    frame: &ModernFrame,
    packets: &MixedVariantPrefinalPackets<'_>,
) {
    debug_assert_eq!(screens.scale, 1);
    let mut bg_overlay_ranks = vec![u8::MAX; screens.main.len()];
    for bg_packet in packets.bg_material_packets() {
        match bg_packet.material {
            PrefinalBgMaterial::StaticEffect => overlay_mixed_variant_bg_packet_on_main_screen(
                screens,
                frame,
                &bg_packet.packet,
                &mut bg_overlay_ranks,
                |index| {
                    let Some((_, effect)) = bg_packet.packet.draw.material_effect() else {
                        return None;
                    };
                    effect.index_to_rgba.get(usize::from(index)).copied()
                },
            ),
            PrefinalBgMaterial::LiveCgram => {
                overlay_mixed_variant_live_cgram_bg_packet_on_main_screen(
                    screens,
                    frame,
                    &bg_packet.packet,
                    &mut bg_overlay_ranks,
                )
            }
        }
    }
    overlay_front_variant_sprite_packets_on_main_screen(
        screens,
        frame,
        &packets.sprites,
        &bg_overlay_ranks,
    );
}

fn bg_packet_needs_prefinal_color_math(
    frame: &ModernFrame,
    packet: &crate::modern_variant_draw::VariantBgDrawPacket<'_>,
) -> bool {
    bg_packet_prefinal_color_math_reason(frame, packet).is_some()
}

fn bg_packet_prefinal_color_math_reason(
    frame: &ModernFrame,
    packet: &crate::modern_variant_draw::VariantBgDrawPacket<'_>,
) -> Option<MixedOverlayComplexRejectReason> {
    let Some((entry, _)) = packet.draw.material_effect() else {
        return None;
    };
    let Ok(layer) = u8::try_from(packet.layer_index) else {
        return None;
    };
    if layer >= 4 {
        return None;
    }
    for y in 0..8usize {
        for x in 0..8usize {
            if bg_effect_packet_index_at_local(packet, entry, x, y) == 0 {
                continue;
            }
            let dst_x = packet.inst.screen_x + x as i16;
            let dst_y = packet.inst.screen_y + y as i16;
            if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                continue;
            }
            let reason =
                bg_packet_pixel_math_reject_reason(frame, layer, dst_x as u32, dst_y as usize);
            if matches!(
                reason,
                Some(
                    MixedOverlayComplexRejectReason::ColorMathClip
                        | MixedOverlayComplexRejectReason::ColorMathSubscreen
                        | MixedOverlayComplexRejectReason::ColorMathFixedColor
                )
            ) {
                return reason;
            }
        }
    }
    None
}

#[cfg(test)]
fn overlay_mixed_variant_bg_packet_on_main_screen(
    screens: &mut crate::modern_software::ModernCompositedScreens,
    frame: &ModernFrame,
    packet: &crate::modern_variant_draw::VariantBgDrawPacket<'_>,
    bg_overlay_ranks: &mut [u8],
    color_for_index: impl Fn(u8) -> Option<[u8; 4]>,
) {
    let Some(entry) = packet.draw.entry() else {
        return;
    };
    let Some(bg_rank) = packet.mode1_rank() else {
        return;
    };
    let Ok(math_bit) = u8::try_from(packet.layer_index) else {
        return;
    };
    if math_bit >= 4 {
        return;
    }
    for y in 0..8usize {
        for x in 0..8usize {
            let index = bg_effect_packet_index_at_local(packet, entry, x, y);
            if index == 0 {
                continue;
            }
            let Some(rgba) = color_for_index(index) else {
                continue;
            };
            if rgba[3] == 0 {
                continue;
            }
            let dst_x = packet.inst.screen_x + x as i16;
            let dst_y = packet.inst.screen_y + y as i16;
            if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                continue;
            }
            if !bg_packet_visible_on_main_at_pixel(frame, packet, dst_x as u32, dst_y as usize) {
                continue;
            }
            let offset = dst_y as usize * screens.width + dst_x as usize;
            if let Some(pixel) = screens.main.get_mut(offset) {
                if packed_variant_prefinal_math_bit(*pixel) != math_bit {
                    continue;
                }
                *pixel = pack_variant_prefinal_pixel(rgba, math_bit);
                if let Some(rank) = bg_overlay_ranks.get_mut(offset) {
                    *rank = bg_rank;
                }
            }
        }
    }
}

#[cfg(test)]
fn overlay_mixed_variant_live_cgram_bg_packet_on_main_screen(
    screens: &mut crate::modern_software::ModernCompositedScreens,
    frame: &ModernFrame,
    packet: &crate::modern_variant_draw::VariantBgDrawPacket<'_>,
    bg_overlay_ranks: &mut [u8],
) {
    let Some(bg_rank) = packet.mode1_rank() else {
        return;
    };
    let Ok(math_bit) = u8::try_from(packet.layer_index) else {
        return;
    };
    if math_bit >= 4 {
        return;
    }
    let palette_base = usize::from(packet.inst.palette) * 16;
    for y in 0..8usize {
        for x in 0..8usize {
            let index = packet.cell.indices[y * 8 + x];
            if index == 0 {
                continue;
            }
            let Some(rgba) = frame.cgram_rgba.get(palette_base + usize::from(index)) else {
                continue;
            };
            let dst_x = packet.inst.screen_x + x as i16;
            let dst_y = packet.inst.screen_y + y as i16;
            if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                continue;
            }
            if !bg_packet_visible_on_main_at_pixel(frame, packet, dst_x as u32, dst_y as usize) {
                continue;
            }
            let offset = dst_y as usize * screens.width + dst_x as usize;
            if let Some(pixel) = screens.main.get_mut(offset) {
                if packed_variant_prefinal_math_bit(*pixel) != math_bit {
                    continue;
                }
                *pixel = pack_variant_prefinal_pixel(*rgba, math_bit);
                if let Some(rank) = bg_overlay_ranks.get_mut(offset) {
                    *rank = bg_rank;
                }
            }
        }
    }
}

#[cfg(test)]
fn overlay_front_variant_sprite_packets_on_main_screen(
    screens: &mut crate::modern_software::ModernCompositedScreens,
    frame: &ModernFrame,
    sprite_packets: &[crate::modern_variant_draw::VariantSpriteDrawPacket<'_>],
    bg_overlay_ranks: &[u8],
) {
    debug_assert_eq!(screens.scale, 1);
    if frame.screen_enabled_main != 0 && frame.screen_enabled_main & 0x10 == 0 {
        return;
    }
    for priority in 0..=3u8 {
        for packet in sprite_packets
            .iter()
            .filter(|packet| packet.inst.priority == priority)
        {
            let Some(sprite_rank) = packet.mode1_rank() else {
                continue;
            };
            let palette_base = 0x80 + usize::from(packet.inst.palette) * 16;
            let math_bit = if packet.inst.palette < 4 { 6 } else { 4 };
            for y in 0..8usize {
                if packet.inst.row_mask & (1 << y) == 0 {
                    continue;
                }
                for x in 0..8usize {
                    let src_x = if packet.inst.hflip { 7 - x } else { x };
                    let src_y = if packet.inst.vflip { 7 - y } else { y };
                    let index = packet.cell.indices[src_y * 8 + src_x];
                    if index == 0 {
                        continue;
                    }
                    let dst_x = packet.inst.screen_x + x as i16;
                    let dst_y = packet.inst.screen_y + y as i16;
                    if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                        continue;
                    }
                    if !sprite_packet_visible_on_main_at_pixel(frame, dst_x as u32, dst_y as usize)
                    {
                        continue;
                    }
                    let offset = dst_y as usize * screens.width + dst_x as usize;
                    let Some(&bg_rank) = bg_overlay_ranks.get(offset) else {
                        continue;
                    };
                    if bg_rank == u8::MAX || sprite_rank < bg_rank {
                        continue;
                    }
                    let Some(rgba) = frame.cgram_rgba.get(palette_base + usize::from(index)) else {
                        continue;
                    };
                    if let Some(pixel) = screens.main.get_mut(offset) {
                        *pixel = pack_variant_prefinal_pixel(*rgba, math_bit);
                    }
                }
            }
        }
    }
}

fn pack_variant_prefinal_pixel(rgba: [u8; 4], math_bit: u8) -> u32 {
    u32::from(rgba[0] >> 3)
        | (u32::from(rgba[1] >> 3) << 5)
        | (u32::from(rgba[2] >> 3) << 10)
        | (u32::from(math_bit) << 15)
        | (1u32 << 18)
}

#[cfg(test)]
fn packed_variant_prefinal_math_bit(pixel: u32) -> u8 {
    ((pixel >> 15) & 0x07) as u8
}

fn bg_packet_overlaps_other_packets(
    frame: &ModernFrame,
    packet_index: usize,
    packet: &crate::modern_variant_draw::VariantBgDrawPacket<'_>,
    plan: &crate::modern_variant_draw::VariantDrawPlan<'_>,
) -> bool {
    let Some(entry) = packet.draw.entry() else {
        return false;
    };
    for y in 0..8usize {
        for x in 0..8usize {
            if bg_effect_packet_index_at_local(packet, entry, x, y) == 0 {
                continue;
            }
            let screen_x = packet.inst.screen_x + x as i16;
            let screen_y = packet.inst.screen_y + y as i16;
            if screen_x < 0 || screen_y < 0 || screen_x >= 256 || screen_y >= 224 {
                continue;
            }
            if other_packet_has_opaque_bg_pixel(packet_index, plan, screen_x, screen_y)
                || front_sprite_packet_blocks_bg_packet(frame, packet, plan, screen_x, screen_y)
            {
                return true;
            }
        }
    }
    false
}

fn bg_packet_prefinal_overlap_reject_reason(
    frame: &ModernFrame,
    packet_index: usize,
    packet: &crate::modern_variant_draw::VariantBgDrawPacket<'_>,
    plan: &crate::modern_variant_draw::VariantDrawPlan<'_>,
) -> Option<MixedOverlayOverlapRejectReason> {
    let Some(entry) = packet.draw.entry() else {
        return None;
    };
    for y in 0..8usize {
        for x in 0..8usize {
            if bg_effect_packet_index_at_local(packet, entry, x, y) == 0 {
                continue;
            }
            let screen_x = packet.inst.screen_x + x as i16;
            let screen_y = packet.inst.screen_y + y as i16;
            if screen_x < 0 || screen_y < 0 || screen_x >= 256 || screen_y >= 224 {
                continue;
            }
            if let Some(reason) = front_or_same_bg_packet_blocks_prefinal_group(
                frame,
                packet_index,
                packet,
                plan,
                screen_x,
                screen_y,
            ) {
                return Some(reason);
            }
        }
    }
    None
}

fn bg_effect_packet_index_at_local(
    packet: &crate::modern_variant_draw::VariantBgDrawPacket<'_>,
    entry: &crate::modern_variant_atlas::VariantAtlasEntry,
    x: usize,
    y: usize,
) -> u8 {
    let src_x = if packet.cell.hflip ^ entry.source_hflip {
        7 - x
    } else {
        x
    };
    let src_y = if packet.cell.vflip ^ entry.source_vflip {
        7 - y
    } else {
        y
    };
    packet.cell.indices[src_y * 8 + src_x]
}

fn front_or_same_bg_packet_has_opaque_pixel(
    packet_index: usize,
    packet: &crate::modern_variant_draw::VariantBgDrawPacket<'_>,
    plan: &crate::modern_variant_draw::VariantDrawPlan<'_>,
    screen_x: i16,
    screen_y: i16,
) -> bool {
    let Some(packet_rank) = packet.mode1_rank() else {
        return true;
    };
    for other_packet in plan.material_packets() {
        let Some((other_index, _other)) = other_packet.as_bg() else {
            continue;
        };
        if other_index == packet_index {
            continue;
        }
        let Some(other_rank) = other_packet.mode1_rank() else {
            continue;
        };
        if other_rank < packet_rank || (other_rank == packet_rank && other_index < packet_index) {
            continue;
        }
        let Some(index) = other_packet.overlap_index_at_screen(screen_x, screen_y) else {
            continue;
        };
        if index != 0 {
            return true;
        }
    }
    false
}

fn front_or_same_bg_packet_blocks_prefinal_group(
    frame: &ModernFrame,
    packet_index: usize,
    packet: &crate::modern_variant_draw::VariantBgDrawPacket<'_>,
    plan: &crate::modern_variant_draw::VariantDrawPlan<'_>,
    screen_x: i16,
    screen_y: i16,
) -> Option<MixedOverlayOverlapRejectReason> {
    let Some(packet_rank) = packet.mode1_rank() else {
        return Some(MixedOverlayOverlapRejectReason::BgUnrepresentableFront);
    };
    let packet_material = bg_packet_prefinal_material(frame, packet).ok();
    for other_packet in plan.material_packets() {
        let Some((other_index, other)) = other_packet.as_bg() else {
            continue;
        };
        if other_index == packet_index {
            continue;
        }
        let Some(other_rank) = other_packet.mode1_rank() else {
            continue;
        };
        if other_rank < packet_rank || (other_rank == packet_rank && other_index < packet_index) {
            continue;
        }
        let Some(index) = other_packet.overlap_index_at_screen(screen_x, screen_y) else {
            continue;
        };
        if index == 0 {
            continue;
        }
        if !bg_packet_visible_on_main_at_pixel(frame, other, screen_x as u32, screen_y as usize) {
            continue;
        }
        if front_or_same_bg_packet_has_opaque_pixel(other_index, other, plan, screen_x, screen_y) {
            return Some(MixedOverlayOverlapRejectReason::BgDeeperChain);
        }
        let other_material = match bg_packet_prefinal_material(frame, other) {
            Ok(material) => material,
            Err(reason) => {
                return Some(match reason {
                    PrefinalBgMaterialRejectReason::NoEffect => {
                        MixedOverlayOverlapRejectReason::BgUnrepresentableFrontNoEffect
                    }
                    PrefinalBgMaterialRejectReason::Complex => {
                        MixedOverlayOverlapRejectReason::BgUnrepresentableFrontComplex
                    }
                    PrefinalBgMaterialRejectReason::CgramMismatch => {
                        MixedOverlayOverlapRejectReason::BgUnrepresentableFrontCgramMismatch
                    }
                });
            }
        };
        if matches!(
            (packet_material, other_material),
            (
                Some(PrefinalBgMaterial::LiveCgram),
                PrefinalBgMaterial::StaticEffect
            )
        ) {
            return Some(MixedOverlayOverlapRejectReason::BgMixedStaticLiveOrder);
        }
        return None;
    }
    None
}

fn bg_packet_visible_on_main_at_pixel(
    frame: &ModernFrame,
    packet: &crate::modern_variant_draw::VariantBgDrawPacket<'_>,
    sx: u32,
    sy: usize,
) -> bool {
    let Ok(layer) = u8::try_from(packet.layer_index) else {
        return false;
    };
    if layer >= 4 {
        return false;
    }
    let layer_bit = 1u8 << layer;
    if frame.screen_enabled_main != 0 && frame.screen_enabled_main & layer_bit == 0 {
        return false;
    }
    if frame
        .main_tm_scanlines
        .get(sy)
        .is_some_and(|tm| tm & layer_bit == 0)
    {
        return false;
    }
    !bg_layer_window_masks_packet_pixel(frame, layer, sx, sy)
}

fn sprite_packet_visible_on_main_at_pixel(frame: &ModernFrame, sx: u32, sy: usize) -> bool {
    if frame.screen_enabled_main != 0 && frame.screen_enabled_main & 0x10 == 0 {
        return false;
    }
    if frame
        .main_tm_scanlines
        .get(sy)
        .is_some_and(|tm| tm & 0x10 == 0)
    {
        return false;
    }
    !bg_layer_window_masks_packet_pixel(frame, 4, sx, sy)
}

fn bg_packet_prefinal_material(
    frame: &ModernFrame,
    packet: &crate::modern_variant_draw::VariantBgDrawPacket<'_>,
) -> Result<PrefinalBgMaterial, PrefinalBgMaterialRejectReason> {
    let Some((entry, effect)) = packet.draw.material_effect() else {
        return Err(PrefinalBgMaterialRejectReason::NoEffect);
    };
    if bg_effect_packet_complex_reject_reason(frame, packet, entry, effect, true).is_some() {
        return Err(PrefinalBgMaterialRejectReason::Complex);
    }
    if bg_effect_matches_live_cgram(packet.cell, packet.inst.palette, effect, frame) {
        return Ok(PrefinalBgMaterial::StaticEffect);
    }
    if bg_packet_can_use_live_cgram(packet, frame) {
        Ok(PrefinalBgMaterial::LiveCgram)
    } else {
        Err(PrefinalBgMaterialRejectReason::CgramMismatch)
    }
}

fn other_packet_has_opaque_bg_pixel(
    packet_index: usize,
    plan: &crate::modern_variant_draw::VariantDrawPlan<'_>,
    screen_x: i16,
    screen_y: i16,
) -> bool {
    for other_packet in plan.material_packets() {
        let Some((other_index, _other)) = other_packet.as_bg() else {
            continue;
        };
        if other_index == packet_index {
            continue;
        }
        let Some(index) = other_packet.overlap_index_at_screen(screen_x, screen_y) else {
            continue;
        };
        if index != 0 {
            return true;
        }
    }
    false
}

fn front_sprite_packet_blocks_bg_packet(
    frame: &ModernFrame,
    packet: &crate::modern_variant_draw::VariantBgDrawPacket<'_>,
    plan: &crate::modern_variant_draw::VariantDrawPlan<'_>,
    screen_x: i16,
    screen_y: i16,
) -> bool {
    let Some(bg_rank) = packet.mode1_rank() else {
        return true;
    };
    if frame.screen_enabled_main != 0 && frame.screen_enabled_main & 0x10 == 0 {
        return false;
    }
    for other_packet in plan.material_packets() {
        let Some((_, _other)) = other_packet.as_sprite() else {
            continue;
        };
        let Some(sprite_rank) = other_packet.mode1_rank() else {
            return true;
        };
        if sprite_rank < bg_rank {
            continue;
        }
        if other_packet
            .overlap_index_at_screen(screen_x, screen_y)
            .is_none_or(|index| index == 0)
        {
            continue;
        }
        if !sprite_packet_visible_on_main_at_pixel(frame, screen_x as u32, screen_y as usize) {
            continue;
        }
        return true;
    }
    false
}

impl ModernGpuVariantEffectRenderer {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
        format: wgpu::TextureFormat,
    ) -> Self {
        let effect_rows = atlas.effects.len().max(1) as u32;
        let effect_lut_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("modern_variant_effect_lut"),
            size: wgpu::Extent3d {
                width: EFFECT_LUT_WIDTH,
                height: effect_rows,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let mut lut_bytes = vec![0u8; (EFFECT_LUT_WIDTH * effect_rows * 4) as usize];
        for (row, effect) in atlas.effects.iter().enumerate() {
            for (index, color) in effect
                .index_to_rgba
                .iter()
                .enumerate()
                .take(EFFECT_LUT_WIDTH as usize)
            {
                let offset = (row * EFFECT_LUT_WIDTH as usize + index) * 4;
                lut_bytes[offset..offset + 4].copy_from_slice(color);
            }
        }
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &effect_lut_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &lut_bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(EFFECT_LUT_WIDTH * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: EFFECT_LUT_WIDTH,
                height: effect_rows,
                depth_or_array_layers: 1,
            },
        );
        let effect_lut_view =
            effect_lut_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("modern_variant_effect"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Uint,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("modern_variant_effect"),
            source: wgpu::ShaderSource::Wgsl(include_str!("modern_bg.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("modern_variant_effect"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: INDEX_INSTANCE_STRIDE,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Sint32x2,
                    offset: 8,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32,
                    offset: 16,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Uint32,
                    offset: 20,
                    shader_location: 3,
                },
            ],
        };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("modern_variant_effect"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_effect"),
                buffers: &[instance_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_effect"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            effect_lut_texture,
            effect_lut_view,
        }
    }

    fn render_bg(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bg_cells: &[ModernIndexTile],
        atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
        packets: &[crate::modern_variant_draw::VariantBgDrawPacket<'_>],
        output_view: &wgpu::TextureView,
        load: wgpu::LoadOp<wgpu::Color>,
    ) {
        self.render_bg_material_group(
            device,
            queue,
            bg_cells,
            None,
            atlas,
            BgEffectMaterialGroup {
                material: EffectMaterial::StaticEffect,
                packets,
            },
            output_view,
            load,
        );
    }

    fn render_bg_material_group(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bg_cells: &[ModernIndexTile],
        frame: Option<&ModernFrame>,
        atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
        group: BgEffectMaterialGroup<'_, '_>,
        output_view: &wgpu::TextureView,
        load: wgpu::LoadOp<wgpu::Color>,
    ) {
        let material = group.material;
        let batch = bg_effect_material_batch(atlas, group);
        match material {
            EffectMaterial::StaticEffect => self.render_bg_effect_batch(
                device,
                queue,
                bg_cells,
                &self.effect_lut_view,
                &batch,
                output_view,
                load,
                "modern_variant_effect_index_atlas",
                "modern_variant_effect",
                "modern_variant_effect_instances",
            ),
            EffectMaterial::LiveCgram => {
                let frame = frame.expect("live CGRAM BG effect rendering needs frame CGRAM");
                let (_live_lut_texture, live_lut_view) =
                    build_live_effect_lut(device, queue, frame);
                self.render_bg_effect_batch(
                    device,
                    queue,
                    bg_cells,
                    &live_lut_view,
                    &batch,
                    output_view,
                    load,
                    "modern_variant_live_cgram_index_atlas",
                    "modern_variant_live_cgram_effect",
                    "modern_variant_live_cgram_effect_instances",
                );
            }
        }
    }

    fn render_bg_effect_batch(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bg_cells: &[ModernIndexTile],
        effect_lut_view: &wgpu::TextureView,
        batch: &EffectMaterialBatch,
        output_view: &wgpu::TextureView,
        load: wgpu::LoadOp<wgpu::Color>,
        index_atlas_label: &'static str,
        label: &'static str,
        instance_label: &'static str,
    ) {
        let (_index_atlas_texture, index_atlas_view) =
            build_index_atlas(device, queue, bg_cells, index_atlas_label);
        self.render_effect_instances(
            device,
            queue,
            &index_atlas_view,
            effect_lut_view,
            batch.instance_bytes(),
            batch.instance_count(),
            output_view,
            load,
            label,
            instance_label,
        );
    }

    fn render_effect_instances(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        index_atlas_view: &wgpu::TextureView,
        effect_lut_view: &wgpu::TextureView,
        instance_bytes: &[u8],
        instance_count: u32,
        output_view: &wgpu::TextureView,
        load: wgpu::LoadOp<wgpu::Color>,
        label: &'static str,
        instance_label: &'static str,
    ) {
        debug_assert_eq!(
            instance_bytes.len() as u64,
            u64::from(instance_count) * INDEX_INSTANCE_STRIDE
        );
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(index_atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(effect_lut_view),
                },
            ],
        });
        let instance_buffer = if instance_count > 0 {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(instance_label),
                size: instance_bytes.len() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&buffer, 0, &instance_bytes);
            Some(buffer)
        } else {
            None
        };
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(label),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if let Some(buffer) = &instance_buffer {
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..6, 0..instance_count);
            }
        }
        queue.submit([encoder.finish()]);
    }

    fn render_sprite_material_groups(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sprite_cells: &[ModernIndexTile],
        frame: &ModernFrame,
        atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
        groups: &[SpriteEffectMaterialGroup<'_, '_>],
        output_view: &wgpu::TextureView,
    ) {
        let (_index_atlas_texture, index_atlas_view) = build_index_atlas(
            device,
            queue,
            sprite_cells,
            "modern_variant_effect_sprite_index_atlas",
        );
        let mut live_lut: Option<(wgpu::Texture, wgpu::TextureView)> = None;
        for group in groups {
            let mut batch = EffectMaterialBatch::default();
            for packet in group.packets {
                let Some(material_packet) = sprite_effect_material_packet(atlas, packet) else {
                    continue;
                };
                debug_assert_eq!(material_packet.material, group.material);
                batch.push(material_packet, EffectSurface::Sprite);
            }
            if batch.material().is_some() {
                self.render_sprite_effect_batch(
                    device,
                    queue,
                    frame,
                    &index_atlas_view,
                    &mut live_lut,
                    &batch,
                    output_view,
                );
            }
        }
    }

    fn render_sprite_effect_batch(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ModernFrame,
        index_atlas_view: &wgpu::TextureView,
        live_lut: &mut Option<(wgpu::Texture, wgpu::TextureView)>,
        batch: &EffectMaterialBatch,
        output_view: &wgpu::TextureView,
    ) {
        if batch.instance_count() == 0 {
            return;
        }
        debug_assert_eq!(
            batch.instance_bytes().len() as u64,
            u64::from(batch.instance_count()) * INDEX_INSTANCE_STRIDE
        );
        let material = batch
            .material()
            .expect("non-empty effect batch has material");
        let (effect_lut_view, label) = match material {
            EffectMaterial::StaticEffect => (&self.effect_lut_view, "modern_variant_effect_sprite"),
            EffectMaterial::LiveCgram => {
                let (_, view) =
                    live_lut.get_or_insert_with(|| build_live_effect_lut(device, queue, frame));
                (&*view, "modern_variant_live_cgram_sprite")
            }
        };
        self.render_effect_instances(
            device,
            queue,
            index_atlas_view,
            effect_lut_view,
            batch.instance_bytes(),
            batch.instance_count(),
            output_view,
            wgpu::LoadOp::Load,
            label,
            label,
        );
    }
}

#[derive(Default)]
struct EffectMaterialBatch {
    material: Option<EffectMaterial>,
    instance_bytes: Vec<u8>,
    instance_count: u32,
}

impl EffectMaterialBatch {
    fn needs_flush_for(&self, material: EffectMaterial) -> bool {
        self.material.is_some_and(|current| current != material)
    }

    fn push(&mut self, material_packet: EffectMaterialPacket, expected_surface: EffectSurface) {
        let material = material_packet.material;
        debug_assert!(!self.needs_flush_for(material));
        self.material = Some(material);
        append_effect_material_packet_instance(
            &mut self.instance_bytes,
            &mut self.instance_count,
            material_packet,
            expected_surface,
            Some(material),
        );
    }

    #[cfg(test)]
    fn clear(&mut self) {
        self.material = None;
        self.instance_bytes.clear();
        self.instance_count = 0;
    }

    fn material(&self) -> Option<EffectMaterial> {
        self.material
    }

    fn instance_bytes(&self) -> &[u8] {
        &self.instance_bytes
    }

    fn instance_count(&self) -> u32 {
        self.instance_count
    }
}

fn sprite_effect_covers_cell(
    cell: &ModernIndexTile,
    effect: &crate::modern_variant_atlas::TileEffect,
) -> bool {
    cell.indices
        .iter()
        .copied()
        .filter(|index| *index != 0)
        .all(|index| {
            usize::from(index) < effect.colors_per_row as usize
                && usize::from(index) < effect.index_to_rgba.len()
        })
}

fn static_bg_effect_material_packet<'packet, 'frame>(
    atlas: &'frame crate::modern_variant_atlas::ModernVariantAtlas,
    packet: &'packet crate::modern_variant_draw::VariantBgDrawPacket<'frame>,
) -> Option<EffectMaterialPacket> {
    let Some((entry, effect)) = packet.draw.material_effect() else {
        return None;
    };
    let effect_row = atlas.effect_row_for_effect(effect)?;
    Some(effect_material_packet(
        EffectSurface::Bg,
        EffectMaterial::StaticEffect,
        effect_row,
        EffectInstanceSource {
            cell_id: packet.inst.cell_id,
            screen_x: packet.inst.screen_x,
            screen_y: packet.inst.screen_y,
            row_mask: 0xff,
            hflip: packet.cell.hflip,
            vflip: packet.cell.vflip,
            source_hflip: entry.source_hflip,
            source_vflip: entry.source_vflip,
        },
    ))
}

fn live_cgram_bg_effect_material_packet<'packet, 'frame>(
    packet: &'packet crate::modern_variant_draw::VariantBgDrawPacket<'frame>,
) -> Option<EffectMaterialPacket> {
    let entry = packet.draw.entry()?;
    let effect_row = u32::from(packet.inst.palette);
    Some(effect_material_packet(
        EffectSurface::Bg,
        EffectMaterial::LiveCgram,
        effect_row,
        EffectInstanceSource {
            cell_id: packet.inst.cell_id,
            screen_x: packet.inst.screen_x,
            screen_y: packet.inst.screen_y,
            row_mask: 0xff,
            hflip: packet.cell.hflip,
            vflip: packet.cell.vflip,
            source_hflip: entry.source_hflip,
            source_vflip: entry.source_vflip,
        },
    ))
}

fn bg_effect_material_batch(
    atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
    group: BgEffectMaterialGroup<'_, '_>,
) -> EffectMaterialBatch {
    let mut batch = EffectMaterialBatch::default();
    for packet in group.packets {
        let material_packet = match group.material {
            EffectMaterial::StaticEffect => static_bg_effect_material_packet(atlas, packet),
            EffectMaterial::LiveCgram => live_cgram_bg_effect_material_packet(packet),
        };
        let Some(material_packet) = material_packet else {
            continue;
        };
        debug_assert_eq!(material_packet.material, group.material);
        batch.push(material_packet, EffectSurface::Bg);
    }
    batch
}

fn sprite_effect_material_groups<'dispatch, 'frame>(
    atlas: &crate::modern_variant_atlas::ModernVariantAtlas,
    packets: &'dispatch [crate::modern_variant_draw::VariantSpriteDrawPacket<'frame>],
) -> Vec<SpriteEffectMaterialGroup<'dispatch, 'frame>> {
    let mut groups = Vec::new();
    let mut current_material = None;
    let mut current_start = 0;

    for (index, packet) in packets.iter().enumerate() {
        let Some(material_packet) = sprite_effect_material_packet(atlas, packet) else {
            continue;
        };
        match current_material {
            None => {
                current_material = Some(material_packet.material);
                current_start = index;
            }
            Some(material) if material == material_packet.material => {}
            Some(material) => {
                groups.push(EffectMaterialGroup {
                    material,
                    packets: &packets[current_start..index],
                });
                current_material = Some(material_packet.material);
                current_start = index;
            }
        }
    }

    if let Some(material) = current_material {
        groups.push(EffectMaterialGroup {
            material,
            packets: &packets[current_start..],
        });
    }

    groups
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EffectInstanceSource {
    cell_id: u32,
    screen_x: i16,
    screen_y: i16,
    row_mask: u8,
    hflip: bool,
    vflip: bool,
    source_hflip: bool,
    source_vflip: bool,
}

fn effect_material_packet(
    surface: EffectSurface,
    material: EffectMaterial,
    effect_row: u32,
    source: EffectInstanceSource,
) -> EffectMaterialPacket {
    EffectMaterialPacket {
        surface,
        material,
        effect_row,
        instance: EffectInstancePacket {
            cell_id: source.cell_id,
            screen_x: source.screen_x,
            screen_y: source.screen_y,
            row_mask: source.row_mask,
            hflip: source.hflip,
            vflip: source.vflip,
            source_hflip: source.source_hflip,
            source_vflip: source.source_vflip,
            effect_row,
        },
    }
}

fn append_effect_material_packet_instance(
    out: &mut Vec<u8>,
    count: &mut u32,
    material_packet: EffectMaterialPacket,
    expected_surface: EffectSurface,
    expected_material: Option<EffectMaterial>,
) {
    debug_assert_eq!(material_packet.surface, expected_surface);
    if let Some(expected_material) = expected_material {
        debug_assert_eq!(material_packet.material, expected_material);
    }
    append_effect_instance_words(out, material_packet.instance);
    *count += 1;
}

fn append_effect_instance_words(out: &mut Vec<u8>, packet: EffectInstancePacket) {
    let col = packet.cell_id % INDEX_GRID_COLS;
    let row = packet.cell_id / INDEX_GRID_COLS;
    out.extend_from_slice(&(col * 8).to_le_bytes());
    out.extend_from_slice(&(row * 8).to_le_bytes());
    out.extend_from_slice(&(i32::from(packet.screen_x)).to_le_bytes());
    out.extend_from_slice(&(i32::from(packet.screen_y)).to_le_bytes());
    let mut flags = u32::from(packet.row_mask) << 8;
    if packet.hflip ^ packet.source_hflip {
        flags |= 0b001;
    }
    if packet.vflip ^ packet.source_vflip {
        flags |= 0b010;
    }
    out.extend_from_slice(&flags.to_le_bytes());
    out.extend_from_slice(&packet.effect_row.to_le_bytes());
}

fn sprite_effect_material_packet<'packet, 'frame>(
    atlas: &'frame crate::modern_variant_atlas::ModernVariantAtlas,
    packet: &'packet crate::modern_variant_draw::VariantSpriteDrawPacket<'frame>,
) -> Option<EffectMaterialPacket> {
    let Some((entry, effect)) = packet.draw.material_effect() else {
        return None;
    };
    let static_effect_row = atlas.effect_row_for_effect(effect);
    let uses_static_effect =
        static_effect_row.is_some_and(|_| sprite_effect_covers_cell(packet.cell, effect));
    let (material, effect_row) = if uses_static_effect {
        (
            EffectMaterial::StaticEffect,
            static_effect_row.expect("checked above"),
        )
    } else {
        (
            EffectMaterial::LiveCgram,
            8 + u32::from(packet.inst.palette),
        )
    };
    Some(effect_material_packet(
        EffectSurface::Sprite,
        material,
        effect_row,
        EffectInstanceSource {
            cell_id: packet.inst.cell_id,
            screen_x: packet.inst.screen_x,
            screen_y: packet.inst.screen_y,
            row_mask: packet.inst.row_mask,
            hflip: packet.inst.hflip,
            vflip: packet.inst.vflip,
            source_hflip: entry.source_hflip,
            source_vflip: entry.source_vflip,
        },
    ))
}

fn variant_tile_instance(
    entry: &crate::modern_variant_atlas::VariantAtlasEntry,
    screen_x: i16,
    screen_y: i16,
    hflip: bool,
    vflip: bool,
) -> crate::modern_frame::ModernTileInstance {
    crate::modern_frame::ModernTileInstance {
        atlas_id: 0,
        atlas_x_px: entry.rect[0] as u16,
        atlas_y_px: entry.rect[1] as u16,
        atlas_width_px: entry.rect[2] as u16,
        atlas_height_px: entry.rect[3] as u16,
        screen_width_px: 8,
        screen_height_px: 8,
        screen_x,
        screen_y,
        palette: 0,
        priority: 0,
        hflip,
        vflip,
        transparent_color_zero: true,
    }
}

const EFFECT_LUT_WIDTH: u32 = 16;

fn build_live_effect_lut(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    frame: &ModernFrame,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("modern_variant_live_cgram_lut"),
        size: wgpu::Extent3d {
            width: EFFECT_LUT_WIDTH,
            height: 16,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let mut lut_bytes = Vec::with_capacity(16 * EFFECT_LUT_WIDTH as usize * 4);
    for color in &frame.cgram_rgba {
        lut_bytes.extend_from_slice(&final_live_cgram_rgba(*color, frame.brightness));
    }
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &lut_bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(EFFECT_LUT_WIDTH * 4),
            rows_per_image: Some(16),
        },
        wgpu::Extent3d {
            width: EFFECT_LUT_WIDTH,
            height: 16,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

fn final_live_cgram_rgba(color: [u8; 4], brightness: u8) -> [u8; 4] {
    [
        expand_live_cgram_channel(color[0], brightness),
        expand_live_cgram_channel(color[1], brightness),
        expand_live_cgram_channel(color[2], brightness),
        color[3],
    ]
}

fn expand_live_cgram_channel(component: u8, brightness: u8) -> u8 {
    let c5 = u32::from(component >> 3).min(31);
    let v8 = (c5 << 3) | (c5 >> 2);
    ((v8 * u32::from(brightness)) / 15) as u8
}

fn u32s_to_le_bytes(words: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(words.len() * 4);
    for word in words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    bytes
}

struct ModernGpuPrefinalOverlay {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl ModernGpuPrefinalOverlay {
    fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("modern_prefinal_overlay"),
            entries: &[
                storage_entry(0, false),
                storage_entry(1, true),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("modern_prefinal_overlay"),
            source: wgpu::ShaderSource::Wgsl(include_str!("modern_prefinal_overlay.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("modern_prefinal_overlay"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("modern_prefinal_overlay"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Self {
            pipeline,
            bind_group_layout,
        }
    }

    fn render_into(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        main_buffer: &wgpu::Buffer,
        frame: &ModernFrame,
        packets: &MixedVariantPrefinalPackets<'_>,
    ) {
        let (data_words, params) = modern_prefinal_overlay_data_words(frame, packets);
        let data_buffer =
            storage_buffer_with_words(device, queue, "modern_prefinal_overlay_data", &data_words);
        let params_buffer =
            uniform_buffer_with_words(device, queue, "modern_prefinal_overlay_params", &params);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("modern_prefinal_overlay"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: main_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        let pixel_count = u32::from(crate::modern_frame::MODERN_FRAME_WIDTH)
            * u32::from(crate::modern_frame::MODERN_FRAME_HEIGHT);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("modern_prefinal_overlay"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("modern_prefinal_overlay"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(pixel_count.div_ceil(64), 1, 1);
        }
        queue.submit([encoder.finish()]);
    }
}

fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn storage_buffer_with_words(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    words: &[u32],
) -> wgpu::Buffer {
    let bytes = u32s_to_le_bytes(if words.is_empty() { &[0] } else { words });
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&buffer, 0, &bytes);
    buffer
}

fn uniform_buffer_with_words(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    words: &[u32],
) -> wgpu::Buffer {
    let bytes = u32s_to_le_bytes(words);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes.len() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&buffer, 0, &bytes);
    buffer
}

fn modern_prefinal_overlay_data_words(
    frame: &ModernFrame,
    packets: &MixedVariantPrefinalPackets<'_>,
) -> (Vec<u32>, [u32; 8]) {
    let mut data = Vec::new();
    let mut bg_packet_words = Vec::new();
    let mut sprite_packet_words = Vec::new();

    for bg_packet in packets.bg_material_packets() {
        let packet = &bg_packet.packet;
        let cell_offset = data.len() as u32;
        data.extend_from_slice(&modern_prefinal_overlay_bg_packet_pixels(frame, bg_packet));
        if let Some(rank) = packet.mode1_rank() {
            bg_packet_words.extend_from_slice(&[
                i32::from(packet.inst.screen_x) as u32,
                i32::from(packet.inst.screen_y) as u32,
                u32::from(rank),
                cell_offset,
            ]);
        }
    }
    for priority in 0..=3u8 {
        for packet in packets
            .sprites
            .iter()
            .filter(|packet| packet.inst.priority == priority)
        {
            let Some(rank) = packet.mode1_rank() else {
                continue;
            };
            let cell_offset = data.len() as u32;
            data.extend_from_slice(&modern_prefinal_overlay_sprite_pixels(frame, packet));
            sprite_packet_words.extend_from_slice(&[
                i32::from(packet.inst.screen_x) as u32,
                i32::from(packet.inst.screen_y) as u32,
                u32::from(rank),
                cell_offset,
            ]);
        }
    }

    let bg_packets_offset = data.len() as u32;
    data.extend_from_slice(if bg_packet_words.is_empty() {
        &[0]
    } else {
        &bg_packet_words
    });
    let sprite_packets_offset = data.len() as u32;
    data.extend_from_slice(if sprite_packet_words.is_empty() {
        &[0]
    } else {
        &sprite_packet_words
    });

    (
        data,
        [
            u32::from(crate::modern_frame::MODERN_FRAME_WIDTH)
                * u32::from(crate::modern_frame::MODERN_FRAME_HEIGHT),
            (bg_packet_words.len() / 4) as u32,
            (sprite_packet_words.len() / 4) as u32,
            0,
            0,
            0,
            bg_packets_offset,
            sprite_packets_offset,
        ],
    )
}

fn modern_prefinal_overlay_bg_packet_pixels(
    frame: &ModernFrame,
    packet: &MixedVariantPrefinalBgPacket<'_>,
) -> [u32; 64] {
    match packet.material {
        PrefinalBgMaterial::StaticEffect => {
            modern_prefinal_overlay_static_bg_pixels(frame, &packet.packet)
        }
        PrefinalBgMaterial::LiveCgram => {
            modern_prefinal_overlay_live_bg_pixels(frame, &packet.packet)
        }
    }
}

fn modern_prefinal_overlay_static_bg_pixels(
    frame: &ModernFrame,
    packet: &crate::modern_variant_draw::VariantBgDrawPacket<'_>,
) -> [u32; 64] {
    let Some((entry, effect)) = packet.draw.material_effect() else {
        return [0xffffffff; 64];
    };
    modern_prefinal_overlay_bg_pixels(frame, packet, |x, y| {
        let index = bg_effect_packet_index_at_local(packet, entry, x, y);
        if index == 0 {
            return None;
        }
        effect.index_to_rgba.get(usize::from(index)).copied()
    })
}

fn modern_prefinal_overlay_live_bg_pixels(
    frame: &ModernFrame,
    packet: &crate::modern_variant_draw::VariantBgDrawPacket<'_>,
) -> [u32; 64] {
    let palette_base = usize::from(packet.inst.palette) * 16;
    modern_prefinal_overlay_bg_pixels(frame, packet, |x, y| {
        let index = packet.cell.indices[y * 8 + x];
        if index == 0 {
            return None;
        }
        frame
            .cgram_rgba
            .get(palette_base + usize::from(index))
            .copied()
    })
}

fn modern_prefinal_overlay_bg_pixels(
    frame: &ModernFrame,
    packet: &crate::modern_variant_draw::VariantBgDrawPacket<'_>,
    color_for_local: impl Fn(usize, usize) -> Option<[u8; 4]>,
) -> [u32; 64] {
    let mut pixels = [0xffffffffu32; 64];
    let Ok(math_bit) = u8::try_from(packet.layer_index) else {
        return pixels;
    };
    if math_bit >= 4 {
        return pixels;
    }
    for y in 0..8usize {
        for x in 0..8usize {
            let Some(rgba) = color_for_local(x, y) else {
                continue;
            };
            if rgba[3] == 0 {
                continue;
            }
            let dst_x = packet.inst.screen_x + x as i16;
            let dst_y = packet.inst.screen_y + y as i16;
            if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                continue;
            }
            if !bg_packet_visible_on_main_at_pixel(frame, packet, dst_x as u32, dst_y as usize) {
                continue;
            }
            pixels[y * 8 + x] = pack_variant_prefinal_pixel(rgba, math_bit);
        }
    }
    pixels
}

fn modern_prefinal_overlay_sprite_pixels(
    frame: &ModernFrame,
    packet: &crate::modern_variant_draw::VariantSpriteDrawPacket<'_>,
) -> [u32; 64] {
    let mut pixels = [0xffffffffu32; 64];
    let palette_base = 0x80 + usize::from(packet.inst.palette) * 16;
    let math_bit = if packet.inst.palette < 4 { 6 } else { 4 };
    for y in 0..8usize {
        if packet.inst.row_mask & (1 << y) == 0 {
            continue;
        }
        for x in 0..8usize {
            let src_x = if packet.inst.hflip { 7 - x } else { x };
            let src_y = if packet.inst.vflip { 7 - y } else { y };
            let index = packet.cell.indices[src_y * 8 + src_x];
            if index == 0 {
                continue;
            }
            let dst_x = packet.inst.screen_x + x as i16;
            let dst_y = packet.inst.screen_y + y as i16;
            if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                continue;
            }
            if !sprite_packet_visible_on_main_at_pixel(frame, dst_x as u32, dst_y as usize) {
                continue;
            }
            let Some(rgba) = frame.cgram_rgba.get(palette_base + usize::from(index)) else {
                continue;
            };
            pixels[y * 8 + x] = pack_variant_prefinal_pixel(*rgba, math_bit);
        }
    }
    pixels
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModernScreenBuilderResult {
    Gpu,
}

fn record_screen_builder_result(
    stats: &mut crate::modern_software::VariantAtlasRenderStats,
    result: ModernScreenBuilderResult,
) {
    match result {
        ModernScreenBuilderResult::Gpu => {
            stats.gpu_prefinal_base_frames += 1;
            stats.gpu_screen_builder_frames += 1;
        }
    }
}

/// GPU finalizer compositor for the PNG index-atlas path. The Mode-1 priority
/// MAIN/SUB screens are built through the same packed intermediate as the
/// byte-exact CPU renderer; the final color-math, windows, and master brightness
/// resolve runs as a compute pass into the caller's `Rgba8Unorm` texture.
pub struct ModernGpuCompositor {
    finalizer: ModernGpuFinalizer,
    screen_builder: ModernGpuScreenBuilder,
    prefinal_overlay: ModernGpuPrefinalOverlay,
}

impl ModernGpuCompositor {
    pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue, _format: wgpu::TextureFormat) -> Self {
        Self {
            finalizer: ModernGpuFinalizer::new(device),
            screen_builder: ModernGpuScreenBuilder::new(device),
            prefinal_overlay: ModernGpuPrefinalOverlay::new(device),
        }
    }

    /// Build the packed MAIN/SUB screens and resolve the final RGBA through the
    /// GPU finalizer into `output_texture`.
    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
        output_texture: &wgpu::Texture,
    ) -> bool {
        matches!(
            self.render_with_screen_builder_status(
                device,
                queue,
                frame,
                bg_cells,
                sprite_cells,
                output_texture,
            ),
            ModernScreenBuilderResult::Gpu
        )
    }

    fn render_with_screen_builder_status(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
        output_texture: &wgpu::Texture,
    ) -> ModernScreenBuilderResult {
        self.screen_builder.render_into(
            device,
            queue,
            frame,
            bg_cells,
            sprite_cells,
            &self.finalizer.main_buffer,
            &self.finalizer.sub_buffer,
        );
        self.finalizer.render_current_buffers_to_texture(
            device,
            queue,
            frame,
            u32::from(crate::modern_frame::MODERN_FRAME_WIDTH)
                * u32::from(crate::modern_frame::MODERN_FRAME_HEIGHT),
            u32::from(crate::modern_frame::MODERN_FRAME_WIDTH),
            1,
            output_texture,
        );
        ModernScreenBuilderResult::Gpu
    }

    fn render_prefinal_screens_with_final_frame(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_frame: &ModernFrame,
        final_frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
        output_texture: &wgpu::Texture,
    ) -> ModernScreenBuilderResult {
        self.screen_builder.render_into(
            device,
            queue,
            screen_frame,
            bg_cells,
            sprite_cells,
            &self.finalizer.main_buffer,
            &self.finalizer.sub_buffer,
        );
        self.finalizer.render_current_buffers_to_texture(
            device,
            queue,
            final_frame,
            u32::from(crate::modern_frame::MODERN_FRAME_WIDTH)
                * u32::from(crate::modern_frame::MODERN_FRAME_HEIGHT),
            u32::from(crate::modern_frame::MODERN_FRAME_WIDTH),
            1,
            output_texture,
        );
        ModernScreenBuilderResult::Gpu
    }

    fn render_prefinal_overlay_screens_with_final_frame(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_frame: &ModernFrame,
        final_frame: &ModernFrame,
        overlay_frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
        packets: &MixedVariantPrefinalPackets<'_>,
        output_texture: &wgpu::Texture,
    ) -> ModernScreenBuilderResult {
        self.screen_builder.render_into(
            device,
            queue,
            screen_frame,
            bg_cells,
            sprite_cells,
            &self.finalizer.main_buffer,
            &self.finalizer.sub_buffer,
        );
        self.prefinal_overlay.render_into(
            device,
            queue,
            &self.finalizer.main_buffer,
            overlay_frame,
            packets,
        );
        self.finalizer.render_current_buffers_to_texture(
            device,
            queue,
            final_frame,
            u32::from(crate::modern_frame::MODERN_FRAME_WIDTH)
                * u32::from(crate::modern_frame::MODERN_FRAME_HEIGHT),
            u32::from(crate::modern_frame::MODERN_FRAME_WIDTH),
            1,
            output_texture,
        );
        ModernScreenBuilderResult::Gpu
    }
}

pub struct ModernIndexCompareRender {
    pub rgba: Vec<u8>,
    pub via: &'static str,
    pub variant_stats: Option<crate::modern_software::VariantAtlasRenderStats>,
    pub variant_traces: Vec<crate::modern_variant_draw::VariantPixelTrace>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernGpuPathFallback {
    pub reason: &'static str,
    pub count: u32,
}

pub fn modern_gpu_path_fallback_reason(
    via: &str,
    stats: Option<&crate::modern_software::VariantAtlasRenderStats>,
) -> Option<ModernGpuPathFallback> {
    match via {
        "mode7-cpu" => {
            return Some(ModernGpuPathFallback {
                reason: "mode7-cpu",
                count: 1,
            });
        }
        "sources" => {
            return Some(ModernGpuPathFallback {
                reason: "sources-cpu",
                count: 1,
            });
        }
        "vram" => {
            return Some(ModernGpuPathFallback {
                reason: "vram-cpu",
                count: 1,
            });
        }
        _ => {}
    }

    let stats = stats?;
    let checks = [
        ("prefinal-overlay-cpu", stats.cpu_prefinal_overlay_frames),
        (
            "prefinal-composite-cpu",
            stats.cpu_prefinal_composite_frames,
        ),
    ];
    checks
        .into_iter()
        .find(|(_, count)| *count != 0)
        .map(|(reason, count)| ModernGpuPathFallback { reason, count })
}

pub fn render_modern_index_compare_frame<S: crate::modern_extract::SourceTableView + ?Sized>(
    frame: &crate::gpu_frame::GpuFrame<'_>,
    src_table: Option<&S>,
    source_atlas: Option<&crate::modern_source_atlas::ModernSourceAtlas>,
    headless: Option<&ModernGpuHeadless>,
    variant_headless: Option<&ModernGpuVariantHeadless>,
    scene: crate::ModernAssetFrameScene,
    trace_pixel: Option<(i16, i16)>,
    allow_source_cpu_fallback: bool,
) -> ModernIndexCompareRender {
    if let Some(variant_headless) = variant_headless {
        if frame.mode == 7 {
            let headless = headless.expect("mode7 helper allocated for variant compare");
            return ModernIndexCompareRender {
                rgba: headless.render_mode7_rgba(frame),
                via: "mode7-gpu",
                variant_stats: None,
                variant_traces: Vec::new(),
            };
        }
        let atlas = source_atlas.expect("atlas loaded for gpu compare");
        let src_table = src_table.expect("source table loaded for gpu compare");
        let (rgba, stats, traces) = variant_headless
            .render_rgba_with_live_index_base_from_sources_traced(
                frame,
                src_table,
                atlas,
                scene.bg_palette_name(),
                scene.sprite_palette_name(),
                trace_pixel,
            );
        return ModernIndexCompareRender {
            rgba,
            via: "variant-gpu",
            variant_stats: Some(stats),
            variant_traces: traces,
        };
    }

    if let Some(headless) = headless {
        if frame.mode == 7 {
            return ModernIndexCompareRender {
                rgba: headless.render_mode7_rgba(frame),
                via: "mode7-gpu",
                variant_stats: None,
                variant_traces: Vec::new(),
            };
        }
        let atlas = source_atlas.expect("atlas loaded for gpu compare");
        let src_table = src_table.expect("source table loaded for gpu compare");
        return ModernIndexCompareRender {
            rgba: headless.render_rgba_from_sources(frame, src_table, atlas),
            via: "gpu",
            variant_stats: None,
            variant_traces: Vec::new(),
        };
    }

    if frame.mode == 7 {
        return ModernIndexCompareRender {
            rgba: crate::modern_software::render_modern_mode7_frame(frame),
            via: "mode7-cpu",
            variant_stats: None,
            variant_traces: Vec::new(),
        };
    }

    if allow_source_cpu_fallback {
        if let (Some(atlas), Some(src_table)) = (source_atlas, src_table) {
            let ctx = crate::modern_hd_overrides::HdOverrideCtx::disabled();
            return ModernIndexCompareRender {
                rgba: crate::modern_extract::render_modern_frame_full_scaled_from_sources(
                    frame, src_table, atlas, &ctx, 1,
                ),
                via: "sources",
                variant_stats: None,
                variant_traces: Vec::new(),
            };
        }
    }

    ModernIndexCompareRender {
        rgba: crate::modern_extract::render_modern_frame_full_from_vram(frame),
        via: "vram",
        variant_stats: None,
        variant_traces: Vec::new(),
    }
}

pub struct ModernAtlasCompareRender {
    pub rgba: Vec<u8>,
    pub hash: u32,
    pub via: &'static str,
}

pub struct ModernAtlasCompareResult {
    pub classic_hash: u32,
    pub modern_hash: u32,
    pub matches: bool,
    pub render: ModernAtlasCompareRender,
}

pub fn render_modern_atlas_compare_frame(
    frame: &crate::gpu_frame::GpuFrame<'_>,
    atlas: &ModernTileAtlasAsset,
) -> ModernAtlasCompareRender {
    let modern = crate::modern_extract::extract_modern_frame_with_atlas(frame, atlas);
    let rgba = crate::modern_software::render_modern_frame_software(
        &modern,
        &atlas.rgba,
        atlas.width_px as u16,
        atlas.height_px as u16,
    );
    let hash = crate::frame_compare::render_frame_rgb_hash_rgba(&rgba);
    ModernAtlasCompareRender {
        rgba,
        hash,
        via: "atlas-software",
    }
}

pub fn compare_modern_atlas_to_rgba(
    classic_rgba: &[u8],
    frame: &crate::gpu_frame::GpuFrame<'_>,
    atlas: &ModernTileAtlasAsset,
) -> ModernAtlasCompareResult {
    let classic_hash = crate::frame_compare::render_frame_rgb_hash_rgba(classic_rgba);
    let render = render_modern_atlas_compare_frame(frame, atlas);
    ModernAtlasCompareResult {
        classic_hash,
        modern_hash: render.hash,
        matches: classic_hash == render.hash,
        render,
    }
}

/// Owns a headless wgpu device + a 256x224 offscreen `Rgba8Unorm` target + the
/// compositor. Construct once and reuse; device creation is expensive.
pub struct ModernGpuHeadless {
    device: wgpu::Device,
    queue: wgpu::Queue,
    compositor: ModernGpuCompositor,
    gpu_frame_renderer: RefCell<crate::gpu_renderer::GpuFrameRenderer>,
    target: wgpu::Texture,
    target_view: wgpu::TextureView,
}

impl ModernGpuHeadless {
    pub fn new() -> Self {
        let instance = crate::create_wgpu_instance();
        let (_adapter, device, queue) =
            pollster::block_on(crate::create_device_queue(&instance, None));
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let compositor = ModernGpuCompositor::new(&device, &queue, format);
        let gpu_frame_renderer = RefCell::new(crate::gpu_renderer::GpuFrameRenderer::new(
            &device, &queue, None,
        ));
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("modern_gpu_headless_target"),
            size: wgpu::Extent3d {
                width: 256,
                height: 224,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            device,
            queue,
            compositor,
            gpu_frame_renderer,
            target,
            target_view,
        }
    }

    pub fn render_rgba(
        &self,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
    ) -> Vec<u8> {
        let _ = self.compositor.render(
            &self.device,
            &self.queue,
            frame,
            bg_cells,
            sprite_cells,
            &self.target,
        );

        self.read_target_rgba()
    }

    pub fn render_rgba_from_sources<S: crate::modern_extract::SourceTableView + ?Sized>(
        &self,
        frame: &crate::gpu_frame::GpuFrame<'_>,
        src_table: &S,
        atlas: &crate::modern_source_atlas::ModernSourceAtlas,
    ) -> Vec<u8> {
        debug_assert_ne!(frame.mode, 7);
        let (mut modern, bg_cells) =
            crate::modern_extract::extract_modern_frame_from_sources(frame, src_table, atlas);
        let (sprite_cells, sprites) =
            crate::modern_extract::extract_modern_sprites_from_sources(frame, src_table, atlas);
        modern.index_sprites = sprites;
        self.render_rgba(&modern, &bg_cells, &sprite_cells)
    }

    pub fn render_mode7_rgba(&self, frame: &crate::gpu_frame::GpuFrame<'_>) -> Vec<u8> {
        debug_assert_eq!(frame.mode, 7);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("modern_gpu_mode7"),
            });
        self.gpu_frame_renderer.borrow_mut().render_frame(
            &mut encoder,
            &self.queue,
            frame,
            &self.target_view,
        );
        self.queue.submit([encoder.finish()]);
        self.read_target_rgba()
    }

    fn read_target_rgba(&self) -> Vec<u8> {
        let (width, height) = (256u32, 224u32);
        let bytes_per_row = width * 4;
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_gpu_headless_readback"),
            size: (bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);

        let slice = readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("GPU poll failed during readback");
        let mapped = slice.get_mapped_range();
        let out = mapped.to_vec();
        drop(mapped);
        readback.unmap();
        out
    }
}

pub struct ModernGpuVariantHeadless {
    device: wgpu::Device,
    queue: wgpu::Queue,
    compositor: ModernGpuCompositor,
    renderer: ModernGpuVariantRenderer,
    target: wgpu::Texture,
    target_view: wgpu::TextureView,
}

impl ModernGpuVariantHeadless {
    pub fn new(atlas: &crate::modern_variant_atlas::ModernVariantAtlas) -> Self {
        let instance = crate::create_wgpu_instance();
        let (_adapter, device, queue) =
            pollster::block_on(crate::create_device_queue(&instance, None));
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let compositor = ModernGpuCompositor::new(&device, &queue, format);
        let renderer = ModernGpuVariantRenderer::new(&device, &queue, atlas, format);
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("modern_gpu_variant_headless_target"),
            size: wgpu::Extent3d {
                width: 256,
                height: 224,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            device,
            queue,
            compositor,
            renderer,
            target,
            target_view,
        }
    }

    pub fn render_rgba(
        &self,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
        bg_palette_name: &str,
        sprite_palette_name: &str,
    ) -> (Vec<u8>, crate::modern_software::VariantAtlasRenderStats) {
        self.render_rgba_with_live_index_base(
            frame,
            bg_cells,
            sprite_cells,
            frame,
            bg_cells,
            sprite_cells,
            bg_palette_name,
            sprite_palette_name,
        )
    }

    pub fn render_rgba_with_live_index_base(
        &self,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
        live_index_frame: &ModernFrame,
        live_index_bg_cells: &[ModernIndexTile],
        live_index_sprite_cells: &[ModernIndexTile],
        bg_palette_name: &str,
        sprite_palette_name: &str,
    ) -> (Vec<u8>, crate::modern_software::VariantAtlasRenderStats) {
        let (rgba, stats, _traces) = self.render_rgba_with_live_index_base_and_trace(
            frame,
            bg_cells,
            sprite_cells,
            live_index_frame,
            live_index_bg_cells,
            live_index_sprite_cells,
            bg_palette_name,
            sprite_palette_name,
            None,
        );
        (rgba, stats)
    }

    fn render_rgba_with_live_index_base_and_trace(
        &self,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
        live_index_frame: &ModernFrame,
        live_index_bg_cells: &[ModernIndexTile],
        live_index_sprite_cells: &[ModernIndexTile],
        bg_palette_name: &str,
        sprite_palette_name: &str,
        trace_pixel: Option<(i16, i16)>,
    ) -> (
        Vec<u8>,
        crate::modern_software::VariantAtlasRenderStats,
        Vec<crate::modern_variant_draw::VariantPixelTrace>,
    ) {
        let prepared = self.renderer.prepare_variant_render(
            frame,
            bg_cells,
            sprite_cells,
            bg_palette_name,
            sprite_palette_name,
        );
        let traces = trace_pixel
            .map(|(trace_x, trace_y)| {
                crate::modern_variant_draw::trace_variant_plan_pixel(
                    frame,
                    &self.renderer.atlas,
                    prepared.plan(),
                    trace_x,
                    trace_y,
                )
            })
            .unwrap_or_default();
        let live_index_base = LiveIndexVariantBase {
            frame: live_index_frame,
            bg_cells: live_index_bg_cells,
            sprite_cells: live_index_sprite_cells,
        };
        let (rgba, stats) = self.render_prepared_variant_rgba(&live_index_base, &prepared);
        (rgba, stats, traces)
    }

    pub fn render_rgba_with_live_index_base_from_sources<
        S: crate::modern_extract::SourceTableView + ?Sized,
    >(
        &self,
        frame: &crate::gpu_frame::GpuFrame<'_>,
        src_table: &S,
        atlas: &crate::modern_source_atlas::ModernSourceAtlas,
        bg_palette_name: &str,
        sprite_palette_name: &str,
    ) -> (Vec<u8>, crate::modern_software::VariantAtlasRenderStats) {
        let (rgba, stats, _traces) = self.render_rgba_with_live_index_base_from_sources_traced(
            frame,
            src_table,
            atlas,
            bg_palette_name,
            sprite_palette_name,
            None,
        );
        (rgba, stats)
    }

    pub fn render_rgba_with_live_index_base_from_sources_traced<
        S: crate::modern_extract::SourceTableView + ?Sized,
    >(
        &self,
        frame: &crate::gpu_frame::GpuFrame<'_>,
        src_table: &S,
        atlas: &crate::modern_source_atlas::ModernSourceAtlas,
        bg_palette_name: &str,
        sprite_palette_name: &str,
        trace_pixel: Option<(i16, i16)>,
    ) -> (
        Vec<u8>,
        crate::modern_software::VariantAtlasRenderStats,
        Vec<crate::modern_variant_draw::VariantPixelTrace>,
    ) {
        debug_assert_ne!(frame.mode, 7);
        let (mut modern, bg_cells) =
            crate::modern_extract::extract_modern_frame_from_sources(frame, src_table, atlas);
        let (sprite_cells, sprites) =
            crate::modern_extract::extract_modern_sprites_from_sources(frame, src_table, atlas);
        modern.index_sprites = sprites;

        let (mut live_index_modern, live_index_bg_cells) =
            crate::modern_extract::extract_modern_frame_from_vram(frame);
        let (live_index_sprite_cells, live_index_sprites) =
            crate::modern_extract::extract_modern_sprites_from_vram(frame);
        live_index_modern.index_sprites = live_index_sprites;

        self.render_rgba_with_live_index_base_and_trace(
            &modern,
            &bg_cells,
            &sprite_cells,
            &live_index_modern,
            &live_index_bg_cells,
            &live_index_sprite_cells,
            bg_palette_name,
            sprite_palette_name,
            trace_pixel,
        )
    }

    fn render_prepared_variant_rgba(
        &self,
        live_index_base: &LiveIndexVariantBase<'_>,
        prepared: &PreparedModernVariantRender<'_>,
    ) -> (Vec<u8>, crate::modern_software::VariantAtlasRenderStats) {
        let mut execution =
            PreparedModernVariantExecution::new(prepared, PreparedModernVariantOutput::Headless);
        self.render_headless_execution(live_index_base, &mut execution);
        (self.read_target_rgba(), execution.finish())
    }

    fn render_headless_execution(
        &self,
        live_index_base: &LiveIndexVariantBase<'_>,
        execution: &mut PreparedModernVariantExecution<'_, '_>,
    ) {
        match execution.render_path() {
            ModernVariantRenderPath::EffectMaterialMode1Order => {
                self.render_effect_material_mode1_order(execution);
            }
            ModernVariantRenderPath::LiveIndexBaseWithOverlay => {
                self.render_live_index_with_overlay(live_index_base, execution);
            }
            ModernVariantRenderPath::EffectMaterialWithStableOverlay => {
                self.render_effect_material_with_stable_overlay(live_index_base, execution);
            }
            ModernVariantRenderPath::StableVariantFrame => {
                self.render_stable_variant_frame(execution);
            }
        }
    }

    fn render_effect_material_mode1_order(
        &self,
        execution: &PreparedModernVariantExecution<'_, '_>,
    ) {
        self.renderer.render_effect_material_mode1_order(
            &self.device,
            &self.queue,
            execution,
            &self.target_view,
        );
    }

    fn render_stable_variant_frame(&self, execution: &PreparedModernVariantExecution<'_, '_>) {
        self.renderer.renderer.render(
            &self.device,
            &self.queue,
            execution.variant_frame(),
            &self.target_view,
        );
    }

    fn render_live_index_with_overlay(
        &self,
        live_index_base: &LiveIndexVariantBase<'_>,
        execution: &mut PreparedModernVariantExecution<'_, '_>,
    ) {
        let frame = execution.frame();
        let plan = execution.plan();
        let overlay = mixed_variant_prefinal_bg_packets(frame, plan);
        let prefinal_packets = MixedVariantPrefinalPackets::from_overlay(frame, &overlay, plan);
        record_headless_live_index_overlay_stats(
            execution.stats_mut().as_mut(),
            frame,
            &overlay,
            &prefinal_packets,
        );
        self.render_live_index_prefinal_base(live_index_base, &prefinal_packets, execution);
    }

    fn render_live_index_prefinal_base(
        &self,
        live_index_base: &LiveIndexVariantBase<'_>,
        prefinal_packets: &MixedVariantPrefinalPackets<'_>,
        execution: &mut PreparedModernVariantExecution<'_, '_>,
    ) {
        if prefinal_packets.is_bg_empty()
            && (live_index_base.frame().forced_blank
                || can_render_final_index_base_gpu(
                    live_index_base.frame(),
                    live_index_base.bg_cells(),
                ))
        {
            execution.stats_mut().as_mut().gpu_prefinal_base_frames += 1;
            let bg = ModernGpuIndexRenderer::new(
                &self.device,
                &self.queue,
                wgpu::TextureFormat::Rgba8Unorm,
            );
            let mut final_live_index_frame = live_index_base.frame().clone();
            finalize_modern_frame_colors_for_direct_index(&mut final_live_index_frame);
            bg.render(
                &self.device,
                &self.queue,
                live_index_base.bg_cells(),
                &final_live_index_frame,
                &self.target_view,
            );
        } else if prefinal_packets.is_bg_empty() {
            let build_result = self.compositor.render_with_screen_builder_status(
                &self.device,
                &self.queue,
                live_index_base.frame(),
                live_index_base.bg_cells(),
                live_index_base.sprite_cells(),
                &self.target,
            );
            record_screen_builder_result(execution.stats_mut().as_mut(), build_result);
        } else {
            let build_result = self
                .compositor
                .render_prefinal_overlay_screens_with_final_frame(
                    &self.device,
                    &self.queue,
                    live_index_base.frame(),
                    live_index_base.frame(),
                    execution.frame(),
                    live_index_base.bg_cells(),
                    live_index_base.sprite_cells(),
                    prefinal_packets,
                    &self.target,
                );
            record_screen_builder_result(execution.stats_mut().as_mut(), build_result);
        }
    }

    fn render_effect_material_with_stable_overlay(
        &self,
        live_index_base: &LiveIndexVariantBase<'_>,
        execution: &mut PreparedModernVariantExecution<'_, '_>,
    ) {
        let frame = execution.frame();
        if frame_needs_material_prefinal_finalizer(frame) {
            let build_result =
                self.render_effect_material_with_prefinal_base(live_index_base, execution);
            record_screen_builder_result(execution.stats_mut().as_mut(), build_result);
        } else {
            self.renderer.render_effect_material_mode1_order(
                &self.device,
                &self.queue,
                execution,
                &self.target_view,
            );
        }
        if execution.stats().needs_headless_stable_overlay() {
            self.renderer.renderer.render_overlay(
                &self.device,
                &self.queue,
                execution.variant_frame(),
                &self.target_view,
            );
        }
    }

    fn render_effect_material_with_prefinal_base(
        &self,
        live_index_base: &LiveIndexVariantBase<'_>,
        execution: &PreparedModernVariantExecution<'_, '_>,
    ) -> ModernScreenBuilderResult {
        let frame = execution.frame();
        let plan = execution.plan();
        let overlay = mixed_variant_prefinal_bg_packets(frame, plan);
        let prefinal_packets = MixedVariantPrefinalPackets::from_all_overlay(&overlay, plan);
        if prefinal_packets.is_bg_empty() {
            return self.compositor.render_prefinal_screens_with_final_frame(
                &self.device,
                &self.queue,
                live_index_base.frame(),
                frame,
                live_index_base.bg_cells(),
                live_index_base.sprite_cells(),
                &self.target,
            );
        }
        self.compositor
            .render_prefinal_overlay_screens_with_final_frame(
                &self.device,
                &self.queue,
                live_index_base.frame(),
                frame,
                frame,
                live_index_base.bg_cells(),
                live_index_base.sprite_cells(),
                &prefinal_packets,
                &self.target,
            )
    }

    fn read_target_rgba(&self) -> Vec<u8> {
        let (width, height) = (256u32, 224u32);
        let bytes_per_row = width * 4;
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_gpu_variant_headless_readback"),
            size: (bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);

        let slice = readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("GPU poll failed during variant readback");
        let mapped = slice.get_mapped_range();
        let out = mapped.to_vec();
        drop(mapped);
        readback.unmap();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_frame::{ModernBgLayer, ModernFrame, ModernTileInstance};
    use crate::modern_software::{render_modern_frame_software, VariantAtlasRenderStats};

    #[test]
    fn variant_render_path_names_live_renderer_fallback_choices() {
        let empty = VariantAtlasRenderStats::default();
        assert_eq!(
            live_variant_render_path(&empty),
            ModernVariantRenderPath::EffectMaterialMode1Order
        );

        let stable_only = VariantAtlasRenderStats {
            stable_draws: 1,
            ..Default::default()
        };
        assert_eq!(
            live_variant_render_path(&stable_only),
            ModernVariantRenderPath::StableVariantFrame
        );

        let all_effect = VariantAtlasRenderStats {
            stable_draws: 1,
            effect_draws: 1,
            ..Default::default()
        };
        assert_eq!(
            live_variant_render_path(&all_effect),
            ModernVariantRenderPath::EffectMaterialMode1Order
        );

        let mixed_stable_effect = VariantAtlasRenderStats {
            stable_draws: 2,
            effect_draws: 1,
            ..Default::default()
        };
        assert_eq!(
            live_variant_render_path(&mixed_stable_effect),
            ModernVariantRenderPath::EffectMaterialWithStableOverlay
        );

        let live_index = VariantAtlasRenderStats {
            fallback_draws: 1,
            ..Default::default()
        };
        assert_eq!(
            live_variant_render_path(&live_index),
            ModernVariantRenderPath::LiveIndexBaseWithOverlay
        );
    }

    #[test]
    fn variant_render_path_preserves_headless_empty_frame_path() {
        let empty = VariantAtlasRenderStats::default();
        assert_eq!(
            headless_variant_render_path(&empty),
            ModernVariantRenderPath::StableVariantFrame
        );

        let all_effect = VariantAtlasRenderStats {
            stable_draws: 1,
            effect_draws: 1,
            ..Default::default()
        };
        assert_eq!(
            headless_variant_render_path(&all_effect),
            ModernVariantRenderPath::EffectMaterialWithStableOverlay
        );

        let live_index = VariantAtlasRenderStats {
            live_index_draws: 1,
            ..Default::default()
        };
        assert_eq!(
            headless_variant_render_path(&live_index),
            ModernVariantRenderPath::LiveIndexBaseWithOverlay
        );
    }

    #[test]
    fn prepared_variant_render_selects_output_specific_path() {
        let frame = ModernFrame::empty();
        let stats = VariantAtlasRenderStats {
            stable_draws: 1,
            effect_draws: 1,
            ..Default::default()
        };
        let prepared = PreparedModernVariantRender::new(
            &frame,
            &[],
            &[],
            crate::modern_variant_draw::VariantDrawPlan {
                bg: Vec::new(),
                sprites: Vec::new(),
                stats,
            },
            ModernFrame::empty(),
        );

        assert_eq!(
            prepared.render_path(PreparedModernVariantOutput::Live),
            ModernVariantRenderPath::EffectMaterialMode1Order
        );
        assert_eq!(
            prepared.render_path(PreparedModernVariantOutput::Headless),
            ModernVariantRenderPath::EffectMaterialWithStableOverlay
        );
    }

    #[test]
    fn prepared_variant_execution_carries_path_and_stats() {
        let frame = ModernFrame::empty();
        let stats = VariantAtlasRenderStats {
            stable_draws: 1,
            effect_draws: 1,
            ..Default::default()
        };
        let prepared = PreparedModernVariantRender::new(
            &frame,
            &[],
            &[],
            crate::modern_variant_draw::VariantDrawPlan {
                bg: Vec::new(),
                sprites: Vec::new(),
                stats,
            },
            ModernFrame::empty(),
        );

        let mut execution =
            PreparedModernVariantExecution::new(&prepared, PreparedModernVariantOutput::Headless);
        assert_eq!(
            execution.render_path(),
            ModernVariantRenderPath::EffectMaterialWithStableOverlay
        );

        execution.stats_mut().as_mut().gpu_prefinal_base_frames += 1;
        let finished = execution.finish();
        assert_eq!(finished.stable_draws, 1);
        assert_eq!(finished.effect_draws, 1);
        assert_eq!(finished.gpu_prefinal_base_frames, 1);
    }

    #[test]
    fn prepared_variant_execution_prepares_mode1_effect_rank_dispatches() {
        use crate::modern_frame::{ModernIndexSpriteInstance, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{ModernVariantAtlas, VariantAtlasDraw};
        use crate::modern_variant_draw::{
            VariantBgDrawPacket, VariantDrawPlan, VariantSpriteDrawPacket,
        };

        let frame = ModernFrame::empty();
        let cell = ModernIndexTile {
            id: 0,
            indices: [1u8; 64],
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        };
        let bg3_low = ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        };
        let sprite_priority_two = ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            priority: 2,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        };
        let stats = VariantAtlasRenderStats {
            effect_draws: 2,
            ..Default::default()
        };
        let prepared = PreparedModernVariantRender::new(
            &frame,
            &[],
            &[],
            VariantDrawPlan {
                bg: vec![VariantBgDrawPacket {
                    layer_index: 2,
                    cell: &cell,
                    inst: &bg3_low,
                    key: None,
                    draw: VariantAtlasDraw::MissingArt,
                }],
                sprites: vec![VariantSpriteDrawPacket {
                    cell: &cell,
                    inst: &sprite_priority_two,
                    key: None,
                    draw: VariantAtlasDraw::MissingArt,
                }],
                stats,
            },
            ModernFrame::empty(),
        );

        let execution =
            PreparedModernVariantExecution::new(&prepared, PreparedModernVariantOutput::Live);
        let ranks = execution.mode1_effect_rank_dispatches();

        assert_eq!(ranks.len(), 10);
        assert_eq!(ranks[0].bg.len(), 1);
        assert_eq!(ranks[0].bg[0].layer_index, 2);
        assert_eq!(ranks[5].sprites.len(), 1);
        assert_eq!(ranks[5].sprites[0].inst.priority, 2);
        assert_eq!(ranks.iter().map(|rank| rank.bg.len()).sum::<usize>(), 1);
        assert_eq!(
            ranks.iter().map(|rank| rank.sprites.len()).sum::<usize>(),
            1
        );

        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: Vec::new(),
            effects: Vec::new(),
        };
        let render_plan = execution.mode1_effect_render_plan(&atlas);
        let rank_plans = render_plan.rank_plans();

        assert!(!render_plan.needs_empty_frame_fallback());
        assert_eq!(rank_plans.len(), 10);
        assert_eq!(rank_plans[0].rank_index(), 0);
        assert!(!rank_plans[0].rendered_before());
        assert_eq!(rank_plans[0].kinds(), vec![GpuWorkItemKind::BgEffect]);
        assert_eq!(rank_plans[1].rank_index(), 1);
        assert!(rank_plans[1].rendered_before());
        assert!(rank_plans[1].is_empty());
        assert_eq!(rank_plans[5].rank_index(), 5);
        assert!(rank_plans[5].rendered_before());
        assert!(rank_plans[5].is_empty());

        let steps = render_plan.steps();
        assert_eq!(steps.len(), 10);
        assert_eq!(
            steps[0],
            PreparedMode1EffectRenderStepKind::Rank {
                rank_index: 0,
                is_empty: false,
                rendered_before: false,
            }
        );
        assert_eq!(
            steps[1],
            PreparedMode1EffectRenderStepKind::Rank {
                rank_index: 1,
                is_empty: true,
                rendered_before: true,
            }
        );
        assert!(!steps
            .iter()
            .any(|step| matches!(step, PreparedMode1EffectRenderStepKind::EmptyFrameFallback)));
        assert_eq!(
            execution.mode1_effect_render_plan(&atlas).into_step_kinds(),
            steps
        );
        assert_eq!(
            execution
                .mode1_effect_render_plan(&atlas)
                .into_command_kinds(),
            vec![PreparedMode1EffectRenderCommandKind::Work {
                source: Mode1EffectCommandSource::Rank(0),
                command: ModernGpuWorkCommandKind {
                    target_load: ModernGpuCommandLoad::ClearFrame,
                    work_item: GpuWorkItemKind::BgEffect,
                },
            }]
        );
    }

    #[test]
    fn prepared_variant_execution_prepares_mode1_empty_frame_fallback() {
        use crate::modern_variant_atlas::ModernVariantAtlas;
        use crate::modern_variant_draw::VariantDrawPlan;

        let frame = ModernFrame::empty();
        let stats = VariantAtlasRenderStats::default();
        let prepared = PreparedModernVariantRender::new(
            &frame,
            &[],
            &[],
            VariantDrawPlan {
                bg: Vec::new(),
                sprites: Vec::new(),
                stats,
            },
            ModernFrame::empty(),
        );
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: Vec::new(),
            effects: Vec::new(),
        };

        let execution =
            PreparedModernVariantExecution::new(&prepared, PreparedModernVariantOutput::Live);
        let render_plan = execution.mode1_effect_render_plan(&atlas);

        assert!(render_plan.needs_empty_frame_fallback());
        assert_eq!(render_plan.rank_plans().len(), 10);
        assert!(render_plan.rank_plans().iter().all(|rank| rank.is_empty()));
        assert!(render_plan
            .rank_plans()
            .iter()
            .all(|rank| !rank.rendered_before()));
        let steps = render_plan.steps();
        assert_eq!(steps.len(), 11);
        assert_eq!(
            steps.last(),
            Some(&PreparedMode1EffectRenderStepKind::EmptyFrameFallback)
        );
        assert_eq!(
            execution.mode1_effect_render_plan(&atlas).into_step_kinds(),
            steps
        );
        assert_eq!(
            execution
                .mode1_effect_render_plan(&atlas)
                .into_command_kinds(),
            vec![PreparedMode1EffectRenderCommandKind::Work {
                source: Mode1EffectCommandSource::EmptyFrameFallback,
                command: ModernGpuWorkCommandKind {
                    target_load: ModernGpuCommandLoad::ClearFrame,
                    work_item: GpuWorkItemKind::ClearBackdrop,
                },
            }]
        );
    }

    #[test]
    fn live_overlay_stats_helper_records_selection_counters_without_draws() {
        let mut stats = VariantAtlasRenderStats {
            mixed_overlay_bg_effect_draws: 3,
            ..Default::default()
        };
        let overlay = MixedVariantOverlayBgSelection {
            candidates: 4,
            culled_invisible_main: 1,
            reject_complex_frame: 2,
            reject_complex_brightness: 3,
            reject_complex_invalid_layer: 4,
            reject_complex_mosaic: 5,
            reject_complex_sub_window: 6,
            reject_complex_effect_bounds: 7,
            reject_complex_scanline_main: 8,
            reject_complex_layer_window: 9,
            reject_complex_color_math: 10,
            reject_complex_color_math_clip: 11,
            reject_complex_color_math_subscreen: 12,
            reject_complex_color_math_fixed_color: 13,
            reject_cgram_mismatch: 14,
            reject_overlap: 15,
            ..Default::default()
        };

        record_live_mixed_overlay_bg_effect_selection_stats(&mut stats, &overlay);

        assert_eq!(stats.mixed_overlay_bg_effect_draws, 3);
        assert_eq!(stats.mixed_overlay_bg_effect_candidates, 4);
        assert_eq!(stats.mixed_overlay_bg_effect_culled_invisible_main, 1);
        assert_eq!(stats.mixed_overlay_bg_effect_reject_complex_frame, 2);
        assert_eq!(stats.mixed_overlay_bg_effect_reject_complex_brightness, 3);
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_invalid_layer,
            4
        );
        assert_eq!(stats.mixed_overlay_bg_effect_reject_complex_mosaic, 5);
        assert_eq!(stats.mixed_overlay_bg_effect_reject_complex_sub_window, 6);
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_effect_bounds,
            7
        );
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_scanline_main,
            8
        );
        assert_eq!(stats.mixed_overlay_bg_effect_reject_complex_layer_window, 9);
        assert_eq!(stats.mixed_overlay_bg_effect_reject_complex_color_math, 10);
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_clip,
            11
        );
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_subscreen,
            12
        );
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_fixed_color,
            13
        );
        assert_eq!(stats.mixed_overlay_bg_effect_reject_cgram_mismatch, 14);
        assert_eq!(stats.mixed_overlay_bg_effect_reject_overlap, 15);
    }

    #[test]
    fn modern_gpu_one_tile_matches_software() {
        pollster::block_on(async {
            let instance = crate::create_wgpu_instance();
            let (_adapter, device, queue) = crate::create_device_queue(&instance, None).await;

            // 8x8 atlas: one solid red opaque tile.
            let atlas_rgba: Vec<u8> = {
                let mut v = vec![0u8; 8 * 8 * 4];
                for px in v.chunks_exact_mut(4) {
                    px.copy_from_slice(&[255, 0, 0, 0xff]);
                }
                v
            };
            let atlas_asset = ModernTileAtlasAsset {
                tile_width_px: 8,
                tile_height_px: 8,
                atlas_scale: 1,
                width_px: 8,
                height_px: 8,
                rgba: atlas_rgba.clone(),
                entries: Vec::new(),
            };

            // One tile at (0,0) on enabled main layer 0, opaque (no transparency).
            let mut frame = ModernFrame::empty();
            let mut layer = ModernBgLayer::new(0);
            layer.enabled_main = true;
            layer.tiles.push(ModernTileInstance {
                atlas_id: 0,
                atlas_x_px: 0,
                atlas_y_px: 0,
                atlas_width_px: 8,
                atlas_height_px: 8,
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

            let renderer = ModernGpuRenderer::new(
                &device,
                &queue,
                &atlas_asset,
                wgpu::TextureFormat::Rgba8Unorm,
            );

            // Render target: exact game resolution, readable back.
            let width = 256u32;
            let height = 224u32;
            let target = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("modern_gpu_test_target"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let view = target.create_view(&wgpu::TextureViewDescriptor::default());

            renderer.render(&device, &queue, &frame, &view);

            // Read back the rendered pixels. 256*4 = 1024 is already a multiple of
            // COPY_BYTES_PER_ROW_ALIGNMENT (256), so there is no row padding.
            let bytes_per_row = width * 4;
            let readback = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("modern_gpu_test_readback"),
                size: (bytes_per_row * height) as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &target,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &readback,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(bytes_per_row),
                        rows_per_image: None,
                    },
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            queue.submit([encoder.finish()]);

            let slice = readback.slice(..);
            slice.map_async(wgpu::MapMode::Read, |_| {});
            device
                .poll(wgpu::PollType::wait_indefinitely())
                .expect("GPU poll failed during readback");
            let mapped = slice.get_mapped_range();
            let gpu_rgba = mapped.to_vec();
            drop(mapped);
            readback.unmap();

            let software_rgba = render_modern_frame_software(&frame, &atlas_rgba, 8, 8);

            assert_eq!(gpu_rgba, software_rgba);
        });
    }

    #[test]
    fn modern_gpu_compositor_forced_blank_uses_screen_builder() {
        pollster::block_on(async {
            let instance = crate::create_wgpu_instance();
            let (_adapter, device, queue) = crate::create_device_queue(&instance, None).await;
            let compositor =
                ModernGpuCompositor::new(&device, &queue, wgpu::TextureFormat::Rgba8Unorm);
            let target = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("modern_gpu_forced_blank_screen_builder_test_target"),
                size: wgpu::Extent3d {
                    width: 256,
                    height: 224,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let mut frame = ModernFrame::empty();
            frame.forced_blank = true;

            assert!(compositor.render(&device, &queue, &frame, &[], &[], &target));
        });
    }

    #[test]
    fn modern_gpu_downsampled_tile_matches_software() {
        pollster::block_on(async {
            let instance = crate::create_wgpu_instance();
            let (_adapter, device, queue) = crate::create_device_queue(&instance, None).await;

            // 32x32 atlas = 4x nearest upscale of a pixel-distinct 8x8 pattern.
            const SCALE: usize = 4;
            const SRC: usize = 8 * SCALE;
            let pattern = |x: usize, y: usize| -> [u8; 4] {
                [(x as u8) * 30 + 5, (y as u8) * 30 + 7, 100, 0xff]
            };
            let mut atlas_rgba = vec![0u8; SRC * SRC * 4];
            for ay in 0..SRC {
                for ax in 0..SRC {
                    let px = pattern(ax / SCALE, ay / SCALE);
                    let o = (ay * SRC + ax) * 4;
                    atlas_rgba[o..o + 4].copy_from_slice(&px);
                }
            }
            let atlas_asset = ModernTileAtlasAsset {
                tile_width_px: 8,
                tile_height_px: 8,
                atlas_scale: SCALE as u16,
                width_px: SRC as u32,
                height_px: SRC as u32,
                rgba: atlas_rgba.clone(),
                entries: Vec::new(),
            };

            // One tile: 32x32 source rect downsampled into an 8x8 footprint.
            let mut frame = ModernFrame::empty();
            let mut layer = ModernBgLayer::new(0);
            layer.enabled_main = true;
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

            let renderer = ModernGpuRenderer::new(
                &device,
                &queue,
                &atlas_asset,
                wgpu::TextureFormat::Rgba8Unorm,
            );

            let width = 256u32;
            let height = 224u32;
            let target = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("modern_gpu_downsample_test_target"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let view = target.create_view(&wgpu::TextureViewDescriptor::default());

            renderer.render(&device, &queue, &frame, &view);

            let bytes_per_row = width * 4;
            let readback = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("modern_gpu_downsample_test_readback"),
                size: (bytes_per_row * height) as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &target,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &readback,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(bytes_per_row),
                        rows_per_image: None,
                    },
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            queue.submit([encoder.finish()]);

            let slice = readback.slice(..);
            slice.map_async(wgpu::MapMode::Read, |_| {});
            device
                .poll(wgpu::PollType::wait_indefinitely())
                .expect("GPU poll failed during readback");
            let mapped = slice.get_mapped_range();
            let gpu_rgba = mapped.to_vec();
            drop(mapped);
            readback.unmap();

            let software_rgba =
                render_modern_frame_software(&frame, &atlas_rgba, SRC as u16, SRC as u16);

            assert_eq!(gpu_rgba, software_rgba);
        });
    }

    #[test]
    fn modern_gpu_renderer_constructs() {
        pollster::block_on(async {
            let instance = crate::create_wgpu_instance();
            let (_adapter, device, queue) = crate::create_device_queue(&instance, None).await;
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
            let atlas = crate::modern_assets::load_modern_overworld_tile_atlas(&root)
                .expect("atlas should load");

            let _renderer =
                ModernGpuRenderer::new(&device, &queue, &atlas, wgpu::TextureFormat::Rgba8Unorm);
        });
    }

    #[test]
    fn modern_gpu_indexed_matches_software() {
        use crate::modern_frame::ModernIndexTileInstance;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_software::render_modern_frame_software_indexed;

        pollster::block_on(async {
            let instance = crate::create_wgpu_instance();
            let (_adapter, device, queue) = crate::create_device_queue(&instance, None).await;

            // Same synthetic cells + frame as the Task 5 software test.
            let mut indices = [0u8; 64];
            indices[0] = 1; // pixel (0,0)
            indices[1] = 2; // pixel (1,0)
            let cells = vec![ModernIndexTile {
                id: 0,
                indices,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                hflip: false,
                vflip: false,
            }];

            let mut frame = ModernFrame::empty();
            frame.backdrop_color_rgba = [0, 0, 0, 0xff];
            frame.cgram_rgba[3 * 16 + 1] = [10, 20, 30, 0xff];
            frame.cgram_rgba[3 * 16 + 2] = [40, 50, 60, 0xff];

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

            let renderer =
                ModernGpuIndexRenderer::new(&device, &queue, wgpu::TextureFormat::Rgba8Unorm);

            let width = 256u32;
            let height = 224u32;
            let target = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("modern_gpu_index_test_target"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let view = target.create_view(&wgpu::TextureViewDescriptor::default());

            renderer.render(&device, &queue, &cells, &frame, &view);

            let bytes_per_row = width * 4;
            let readback = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("modern_gpu_index_test_readback"),
                size: (bytes_per_row * height) as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &target,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &readback,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(bytes_per_row),
                        rows_per_image: None,
                    },
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            queue.submit([encoder.finish()]);

            let slice = readback.slice(..);
            slice.map_async(wgpu::MapMode::Read, |_| {});
            device
                .poll(wgpu::PollType::wait_indefinitely())
                .expect("GPU poll failed during readback");
            let mapped = slice.get_mapped_range();
            let gpu_rgba = mapped.to_vec();
            drop(mapped);
            readback.unmap();

            let software_rgba = render_modern_frame_software_indexed(&frame, &cells);

            assert_eq!(gpu_rgba.len(), software_rgba.len());
            assert_eq!(gpu_rgba, software_rgba);
        });
    }

    #[test]
    fn modern_gpu_compositor_matches_full_software_basic_bg_obj() {
        use crate::modern_frame::{
            ModernBgLayer, ModernIndexSpriteInstance, ModernIndexTileInstance,
        };
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_software::render_modern_frame_full;

        let mut a = [0u8; 64];
        a[0] = 1;
        a[9] = 2;
        let mut b = [0u8; 64];
        b[63] = 3;
        let bg_cells = vec![
            ModernIndexTile {
                id: 0,
                indices: a,
                source_key: NO_SOURCE_KEY,
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: b,
                source_key: NO_SOURCE_KEY,
                hflip: false,
                vflip: false,
            },
        ];

        let mut s0 = [0u8; 64];
        s0[0] = 4;
        let mut s1 = [0u8; 64];
        s1[7] = 5;
        let sprite_cells = vec![
            ModernIndexTile {
                id: 0,
                indices: s0,
                source_key: NO_SOURCE_KEY,
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: s1,
                source_key: NO_SOURCE_KEY,
                hflip: false,
                vflip: false,
            },
        ];

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.cgram_rgba[3 * 16 + 1] = [10, 20, 30, 0xff];
        frame.cgram_rgba[3 * 16 + 2] = [40, 50, 60, 0xff];
        frame.cgram_rgba[3 * 16 + 3] = [70, 80, 90, 0xff];
        frame.cgram_rgba[0x80 + 16 + 4] = [100, 110, 120, 0xff];
        frame.cgram_rgba[0x80 + 16 + 5] = [130, 140, 150, 0xff];

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
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 16,
            screen_y: 8,
            palette: 3,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = layer;

        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 40,
            screen_y: 40,
            palette: 1,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 1,
            screen_x: 48,
            screen_y: 40,
            palette: 1,
            priority: 0,
            hflip: true,
            vflip: true,
            row_mask: 0xff,
        });
        frame.screen_enabled_main = 0x11; // BG1 + OBJ.
        frame.brightness = 15;

        let gpu = ModernGpuHeadless::new().render_rgba(&frame, &bg_cells, &sprite_cells);
        let software = render_modern_frame_full(&frame, &bg_cells, &sprite_cells);

        assert_eq!(gpu.len(), software.len());
        assert_eq!(
            gpu, software,
            "GPU compositor must match full CPU reference"
        );
    }

    #[test]
    fn modern_gpu_screen_builder_resolves_obj_order_before_priority_slotting() {
        use crate::modern_frame::ModernIndexSpriteInstance;
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;

        let mut indices = [0u8; 64];
        indices[0] = 1;
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.cgram_rgba[0x80 + 4 * 16 + 1] = [255, 0, 0, 0xff];
        frame.cgram_rgba[0x80 + 5 * 16 + 1] = [0, 255, 0, 0xff];
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 4,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 5,
            priority: 3,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });
        frame.screen_enabled_main = 0x10;
        frame.brightness = 15;

        let gpu = ModernGpuHeadless::new().render_rgba(&frame, &[], &sprite_cells);
        let software = crate::modern_software::render_modern_frame_full(&frame, &[], &sprite_cells);

        assert_eq!(&software[0..3], &[255, 0, 0]);
        assert_eq!(
            &gpu[0..3],
            &[255, 0, 0],
            "GPU screen builder must resolve OBJ-vs-OBJ by OAM index before BG priority slotting"
        );
    }

    #[test]
    fn modern_gpu_compositor_matches_full_software_color_math() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_software::render_modern_frame_full;

        let mut indices = [0u8; 64];
        indices[0] = 1;
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.cgram_rgba[1] = [20 << 3, 18 << 3, 16 << 3, 0xff];
        frame.cgram_rgba[16 + 1] = [10 << 3, 8 << 3, 6 << 3, 0xff];

        let mut main = ModernBgLayer::new(0);
        main.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = main;

        let mut sub = ModernBgLayer::new(1);
        sub.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 1,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[1] = sub;

        frame.screen_enabled_main = 0x01; // BG1.
        frame.screen_enabled_sub = 0x02; // BG2.
        frame.math_enabled = 0x01; // Math on winning BG1.
        frame.add_subscreen = true;
        frame.half_color = true;
        frame.brightness = 11;

        let gpu = ModernGpuHeadless::new().render_rgba(&frame, &cells, &[]);
        let software = render_modern_frame_full(&frame, &cells, &[]);

        assert_eq!(gpu.len(), software.len());
        assert_eq!(
            &gpu[0..4],
            &software[0..4],
            "first pixel exercises sub-screen half-add plus brightness"
        );
        assert_eq!(gpu, software);
    }

    #[test]
    fn modern_gpu_compositor_matches_full_software_layer_windows() {
        use crate::modern_frame::{
            ModernBgLayer, ModernIndexSpriteInstance, ModernIndexTileInstance,
        };
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_software::render_modern_frame_full;

        let mut bg_indices = [0u8; 64];
        bg_indices[0] = 1;
        bg_indices[1] = 1;
        let bg_cells = vec![ModernIndexTile {
            id: 0,
            indices: bg_indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut sprite_indices = [0u8; 64];
        sprite_indices[0] = 1;
        sprite_indices[1] = 1;
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices: sprite_indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.cgram_rgba[1] = [80, 0, 0, 0xff];
        frame.cgram_rgba[16 + 1] = [0, 80, 0, 0xff];
        frame.cgram_rgba[0x80 + 16 + 1] = [0, 0, 80, 0xff];

        let mut main = ModernBgLayer::new(0);
        main.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = main;

        let mut sub = ModernBgLayer::new(1);
        sub.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 1,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[1] = sub;

        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 8,
            palette: 1,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });

        frame.screen_enabled_main = 0x11; // BG1 + OBJ.
        frame.screen_enabled_sub = 0x02; // BG2.
        frame.math_enabled = 0x01;
        frame.add_subscreen = true;
        frame.brightness = 15;
        frame.windowsel = (0x2 << 4) | (0x2 << 16); // BG2 + OBJ W1 enabled.
        frame.screen_windowed_sub = 0x02;
        frame.screen_windowed_main = 0x10;
        frame.window_scanlines =
            vec![[0, 0, 0, 0]; usize::from(crate::modern_frame::MODERN_FRAME_HEIGHT)];

        let gpu = ModernGpuHeadless::new().render_rgba(&frame, &bg_cells, &sprite_cells);
        let software = render_modern_frame_full(&frame, &bg_cells, &sprite_cells);

        assert_eq!(&gpu[0..4], &software[0..4], "sub BG2 is masked at x=0");
        assert_eq!(&gpu[4..8], &software[4..8], "sub BG2 contributes at x=1");
        let sprite_row = 8usize * 256 * 4;
        assert_eq!(
            &gpu[sprite_row..sprite_row + 4],
            &software[sprite_row..sprite_row + 4],
            "OBJ is masked at x=0"
        );
        assert_eq!(
            &gpu[sprite_row + 4..sprite_row + 8],
            &software[sprite_row + 4..sprite_row + 8],
            "OBJ draws at x=1"
        );
        assert_eq!(gpu, software);
    }

    #[test]
    fn modern_gpu_variant_headless_missing_tiles_fallback_matches_full_compositor() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::ModernVariantAtlas;

        let mut indices = [0u8; 64];
        indices[0] = 1;
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.cgram_rgba[1] = [20 << 3, 18 << 3, 16 << 3, 0xff];
        frame.cgram_rgba[16 + 1] = [10 << 3, 8 << 3, 6 << 3, 0xff];

        let mut main = ModernBgLayer::new(0);
        main.enabled_main = true;
        main.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = main;

        let mut sub = ModernBgLayer::new(1);
        sub.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 1,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[1] = sub;

        frame.screen_enabled_main = 0x01;
        frame.screen_enabled_sub = 0x02;
        frame.math_enabled = 0x01;
        frame.add_subscreen = true;
        frame.half_color = true;
        frame.brightness = 11;

        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: Vec::new(),
            effects: Vec::new(),
        };
        let (variant, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba(
            &frame,
            &cells,
            &[],
            "palette_dung_bg_main",
            "palette_main_spr",
        );
        let full = ModernGpuHeadless::new().render_rgba(&frame, &cells, &[]);

        assert_eq!(stats.stable_draws, 0);
        assert_eq!(stats.fallback_draws, 1);
        assert_eq!(stats.dynamic_palette_draws, 0);
        assert_eq!(stats.missing_variant_draws, 1);
        assert_eq!(stats.stable_preview_draws, 0);
        assert_eq!(stats.stable_effect_draws, 0);
        assert_eq!(stats.dynamic_material_draws, 0);
        assert_eq!(stats.missing_art_draws, 1);
        assert_eq!(stats.unkeyed_fallback_draws, 0);
        assert_eq!(variant, full);
    }

    #[test]
    fn modern_gpu_variant_headless_trace_does_not_change_render_output() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::ModernVariantAtlas;
        use crate::modern_variant_draw::{ModernDrawMaterial, VariantDrawSurface};

        let mut indices = [0u8; 64];
        indices[0] = 1;
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.cgram_rgba[1] = [20 << 3, 18 << 3, 16 << 3, 0xff];
        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = layer;
        frame.screen_enabled_main = 0x01;
        frame.brightness = 15;

        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: Vec::new(),
            effects: Vec::new(),
        };
        let headless = ModernGpuVariantHeadless::new(&atlas);
        let (untraced, untraced_stats) = headless.render_rgba(
            &frame,
            &cells,
            &[],
            "palette_dung_bg_main",
            "palette_main_spr",
        );
        let (traced, traced_stats, traces) = headless.render_rgba_with_live_index_base_and_trace(
            &frame,
            &cells,
            &[],
            &frame,
            &cells,
            &[],
            "palette_dung_bg_main",
            "palette_main_spr",
            Some((0, 0)),
        );

        assert_eq!(traced, untraced);
        assert_eq!(traced_stats, untraced_stats);
        assert_eq!(traces.len(), 1);
        let trace = &traces[0];
        assert_eq!(trace.surface, VariantDrawSurface::Bg);
        assert_eq!(trace.material, ModernDrawMaterial::MissingArt);
        assert_eq!(trace.cell_id, 0);
        assert_eq!(trace.palette_index, 1);
    }

    #[test]
    fn modern_gpu_variant_headless_uses_live_cgram_for_sprite_effect_indices_outside_static_row() {
        use crate::modern_frame::ModernIndexSpriteInstance;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut sprite_indices = [0u8; 64];
        sprite_indices[0] = 9;
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices: sprite_indices,
            source_key: modern_source_key(2, 0, 0),
            hflip: false,
            vflip: false,
        }];
        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.cgram_rgba[0x80 + 16 + 9] = [248, 248, 248, 0xff];
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 1,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![VariantAtlasEntry {
                id: "sprite:kSprGfx:pack0:tile0:3bpp:palette_main_spr:row1".to_string(),
                key: VariantAtlasKey {
                    source_kind: "sprite".to_string(),
                    asset: "kSprGfx".to_string(),
                    pack: 0,
                    tile: 0,
                    bpp: 3,
                    palette: "palette_main_spr".to_string(),
                    palette_row: 1,
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
                id: "palette_main_spr:8color:row1".to_string(),
                palette: "palette_main_spr".to_string(),
                palette_row: 1,
                colors_per_row: 8,
                index_to_rgba: vec![[0, 0, 0, 0xff]; 8],
                dynamic_policy: "stable".to_string(),
            }],
        };

        let (variant, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba(
            &frame,
            &[],
            &sprite_cells,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(stats.effect_draws, 1);
        assert_eq!(stats.effect_material_draws, 1);
        assert_eq!(stats.dynamic_material_draws, 1);
        assert_eq!(stats.fallback_draws, 0);
        assert_eq!(&variant[0..4], &[255, 255, 255, 0xff]);
    }

    #[test]
    fn modern_gpu_variant_headless_finalizes_all_material_sprite_pixels_when_color_window_clips() {
        use crate::modern_frame::ModernIndexSpriteInstance;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut sprite_indices = [0u8; 64];
        sprite_indices[0] = 1;
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices: sprite_indices,
            source_key: modern_source_key(2, 0, 0),
            hflip: false,
            vflip: false,
        }];
        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.screen_enabled_main = 0x10;
        frame.clip_mode = 3;
        frame.cgram_rgba[0x80 + 16 + 1] = [40, 180, 40, 0xff];
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 1,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![VariantAtlasEntry {
                id: "sprite:kSprGfx:pack0:tile0:3bpp:palette_main_spr:row1".to_string(),
                key: VariantAtlasKey {
                    source_kind: "sprite".to_string(),
                    asset: "kSprGfx".to_string(),
                    pack: 0,
                    tile: 0,
                    bpp: 3,
                    palette: "palette_main_spr".to_string(),
                    palette_row: 1,
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
                id: "palette_main_spr:8color:row1".to_string(),
                palette: "palette_main_spr".to_string(),
                palette_row: 1,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [200, 20, 20, 0xff],
                    [2, 2, 2, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };

        let (variant, stats) = ModernGpuVariantHeadless::new(&atlas)
            .render_rgba_with_live_index_base(
                &frame,
                &[],
                &sprite_cells,
                &frame,
                &[],
                &sprite_cells,
                "palette_dung_bg_main",
                "palette_main_spr",
            );

        assert_eq!(stats.effect_draws, 1);
        assert_eq!(stats.effect_material_draws, 1);
        assert_eq!(stats.dynamic_material_draws, 1);
        assert_eq!(stats.fallback_draws, 0);
        assert_eq!(stats.gpu_prefinal_base_frames, 1);
        assert_eq!(stats.gpu_screen_builder_frames, 1);
        assert_eq!(stats.cpu_prefinal_composite_frames, 0);
        assert_eq!(&variant[0..4], &[0, 0, 0, 0xff]);
    }

    #[test]
    fn effect_instance_packet_encodes_shared_gpu_layout() {
        let mut words = Vec::new();
        append_effect_instance_words(
            &mut words,
            EffectInstancePacket {
                cell_id: 2,
                screen_x: 12,
                screen_y: 20,
                row_mask: 0x7f,
                hflip: true,
                vflip: false,
                source_hflip: false,
                source_vflip: true,
                effect_row: 9,
            },
        );

        assert_eq!(words.len() as u64, INDEX_INSTANCE_STRIDE);
        let encoded: Vec<u32> = words
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();
        assert_eq!(encoded, vec![16, 0, 12, 20, 0x7f00 | 0b011, 9]);
    }

    #[test]
    fn effect_material_packet_appends_instance_and_count() {
        let mut words = Vec::new();
        let mut count = 0;
        append_effect_material_packet_instance(
            &mut words,
            &mut count,
            EffectMaterialPacket {
                surface: EffectSurface::Sprite,
                material: EffectMaterial::LiveCgram,
                effect_row: 9,
                instance: EffectInstancePacket {
                    cell_id: 2,
                    screen_x: 12,
                    screen_y: 20,
                    row_mask: 0x7f,
                    hflip: true,
                    vflip: false,
                    source_hflip: false,
                    source_vflip: true,
                    effect_row: 9,
                },
            },
            EffectSurface::Sprite,
            Some(EffectMaterial::LiveCgram),
        );

        assert_eq!(count, 1);
        assert_eq!(words.len() as u64, INDEX_INSTANCE_STRIDE);
        let encoded: Vec<u32> = words
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();
        assert_eq!(encoded, vec![16, 0, 12, 20, 0x7f00 | 0b011, 9]);
    }

    #[test]
    fn effect_material_batch_tracks_flush_boundary() {
        let mut batch = EffectMaterialBatch::default();
        let static_packet = EffectMaterialPacket {
            surface: EffectSurface::Sprite,
            material: EffectMaterial::StaticEffect,
            effect_row: 2,
            instance: EffectInstancePacket {
                cell_id: 1,
                screen_x: 3,
                screen_y: 4,
                row_mask: 0xff,
                hflip: false,
                vflip: true,
                source_hflip: false,
                source_vflip: false,
                effect_row: 2,
            },
        };

        assert!(!batch.needs_flush_for(EffectMaterial::StaticEffect));
        batch.push(static_packet, EffectSurface::Sprite);

        assert_eq!(batch.material(), Some(EffectMaterial::StaticEffect));
        assert_eq!(batch.instance_count(), 1);
        assert_eq!(batch.instance_bytes().len() as u64, INDEX_INSTANCE_STRIDE);
        assert!(!batch.needs_flush_for(EffectMaterial::StaticEffect));
        assert!(batch.needs_flush_for(EffectMaterial::LiveCgram));

        batch.clear();

        assert_eq!(batch.material(), None);
        assert_eq!(batch.instance_count(), 0);
        assert!(batch.instance_bytes().is_empty());
        assert!(!batch.needs_flush_for(EffectMaterial::LiveCgram));

        batch.push(
            EffectMaterialPacket {
                surface: EffectSurface::Bg,
                material: EffectMaterial::LiveCgram,
                effect_row: 4,
                instance: EffectInstancePacket {
                    cell_id: 2,
                    screen_x: 5,
                    screen_y: 6,
                    row_mask: 0xff,
                    hflip: true,
                    vflip: false,
                    source_hflip: true,
                    source_vflip: false,
                    effect_row: 4,
                },
            },
            EffectSurface::Bg,
        );

        assert_eq!(batch.material(), Some(EffectMaterial::LiveCgram));
        assert_eq!(batch.instance_count(), 1);
        assert_eq!(batch.instance_bytes().len() as u64, INDEX_INSTANCE_STRIDE);
    }

    #[test]
    fn mode1_effect_rank_dispatches_partition_bg_and_sprites_once() {
        use crate::modern_frame::{ModernIndexSpriteInstance, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::VariantAtlasDraw;
        use crate::modern_variant_draw::{
            VariantBgDrawPacket, VariantDrawPlan, VariantSpriteDrawPacket,
        };

        let cell = ModernIndexTile {
            id: 0,
            indices: [1u8; 64],
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        };
        let bg3_low = ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        };
        let bg1_high = ModernIndexTileInstance {
            priority: true,
            ..bg3_low
        };
        let unsupported_bg = ModernIndexTileInstance {
            priority: false,
            ..bg3_low
        };
        let sprite_priority_two = ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            priority: 2,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        };
        let plan = VariantDrawPlan {
            bg: vec![
                VariantBgDrawPacket {
                    layer_index: 2,
                    cell: &cell,
                    inst: &bg3_low,
                    key: None,
                    draw: VariantAtlasDraw::MissingArt,
                },
                VariantBgDrawPacket {
                    layer_index: 0,
                    cell: &cell,
                    inst: &bg1_high,
                    key: None,
                    draw: VariantAtlasDraw::MissingArt,
                },
                VariantBgDrawPacket {
                    layer_index: 3,
                    cell: &cell,
                    inst: &unsupported_bg,
                    key: None,
                    draw: VariantAtlasDraw::MissingArt,
                },
            ],
            sprites: vec![VariantSpriteDrawPacket {
                cell: &cell,
                inst: &sprite_priority_two,
                key: None,
                draw: VariantAtlasDraw::MissingArt,
            }],
            stats: Default::default(),
        };

        let ranks = mode1_effect_rank_dispatches(&plan);
        let rank0_bg_groups = ranks[0].bg_material_groups().collect::<Vec<_>>();
        let rank7_bg_groups = ranks[7].bg_material_groups().collect::<Vec<_>>();

        assert_eq!(ranks.len(), 10);
        assert_eq!(rank0_bg_groups.len(), 1);
        assert_eq!(rank0_bg_groups[0].material, EffectMaterial::StaticEffect);
        assert_eq!(rank0_bg_groups[0].packets[0].layer_index, 2);
        assert_eq!(ranks[5].sprites.len(), 1);
        assert_eq!(ranks[5].sprites[0].inst.priority, 2);
        assert_eq!(rank7_bg_groups.len(), 1);
        assert_eq!(rank7_bg_groups[0].material, EffectMaterial::StaticEffect);
        assert_eq!(rank7_bg_groups[0].packets[0].layer_index, 0);
        assert_eq!(
            ranks
                .iter()
                .flat_map(|rank| rank.bg_material_groups())
                .map(|group| group.packets.len())
                .sum::<usize>(),
            2
        );
        assert_eq!(
            ranks.iter().map(|rank| rank.sprites.len()).sum::<usize>(),
            1
        );
    }

    #[test]
    fn overlay_bg_effect_dispatch_exposes_material_groups_in_render_order() {
        use crate::modern_frame::ModernIndexTileInstance;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::VariantAtlasDraw;
        use crate::modern_variant_draw::VariantBgDrawPacket;

        let cell = ModernIndexTile {
            id: 0,
            indices: [1u8; 64],
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        };
        let static_inst = ModernIndexTileInstance {
            cell_id: 7,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        };
        let live_inst = ModernIndexTileInstance {
            cell_id: 9,
            ..static_inst
        };
        let static_packet = VariantBgDrawPacket {
            layer_index: 0,
            cell: &cell,
            inst: &static_inst,
            key: None,
            draw: VariantAtlasDraw::MissingArt,
        };
        let live_packet = VariantBgDrawPacket {
            layer_index: 0,
            cell: &cell,
            inst: &live_inst,
            key: None,
            draw: VariantAtlasDraw::MissingArt,
        };
        let dispatch = OverlayBgEffectDispatch {
            static_bg: vec![static_packet],
            live_cgram_bg: vec![live_packet],
        };

        let groups = dispatch.material_groups().collect::<Vec<_>>();

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].material, EffectMaterial::StaticEffect);
        assert_eq!(groups[0].packets[0].inst.cell_id, 7);
        assert_eq!(groups[1].material, EffectMaterial::LiveCgram);
        assert_eq!(groups[1].packets[0].inst.cell_id, 9);
        let render_plan = dispatch.render_plan();
        assert_eq!(render_plan.len(), 2);
        assert_eq!(
            render_plan.kinds(),
            vec![GpuWorkItemKind::BgEffect, GpuWorkItemKind::BgEffect]
        );
        assert!(matches!(
            &render_plan.work_items()[0],
            ModernGpuWorkItem::BgEffect(group)
                if group.material == EffectMaterial::StaticEffect
                    && group.packets[0].inst.cell_id == 7
        ));
        assert!(matches!(
            &render_plan.work_items()[1],
            ModernGpuWorkItem::BgEffect(group)
                if group.material == EffectMaterial::LiveCgram
                    && group.packets[0].inst.cell_id == 9
        ));
    }

    #[test]
    fn bg_effect_material_packet_selects_static_or_live_rows() {
        use crate::modern_frame::ModernIndexTileInstance;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasDraw, VariantAtlasEntry, VariantAtlasKey,
        };
        use crate::modern_variant_draw::VariantBgDrawPacket;

        let entry = VariantAtlasEntry {
            id: "bg:kBgGfx:pack0:tile0:3bpp:palette_dung_bg_main:row2".to_string(),
            key: VariantAtlasKey {
                source_kind: "bg".to_string(),
                asset: "kBgGfx".to_string(),
                pack: 0,
                tile: 0,
                bpp: 3,
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
            },
            rect: [0, 0, 8, 8],
            sha1: "static".to_string(),
            duplicate_of: None,
            dynamic_policy: "stable".to_string(),
            runtime_material: Some("palette_lut".to_string()),
            runtime_colors_per_row: None,
            source_hflip: false,
            source_vflip: true,
        };
        let effect = TileEffect {
            id: "palette_dung_bg_main:8color:row2".to_string(),
            palette: "palette_dung_bg_main".to_string(),
            palette_row: 2,
            colors_per_row: 8,
            index_to_rgba: vec![[0, 0, 0, 0xff], [80, 0, 0, 0xff]],
            dynamic_policy: "stable".to_string(),
        };
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![entry.clone()],
            effects: vec![effect.clone()],
        };
        let cell = ModernIndexTile {
            id: 0,
            indices: [1u8; 64],
            source_key: modern_source_key(1, 0, 0),
            hflip: true,
            vflip: false,
        };
        let inst = ModernIndexTileInstance {
            cell_id: 2,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 12,
            screen_y: 20,
            palette: 3,
            hflip: false,
            vflip: false,
            priority: false,
        };
        let static_packet = VariantBgDrawPacket {
            layer_index: 0,
            cell: &cell,
            inst: &inst,
            key: None,
            draw: VariantAtlasDraw::MaterialEffect {
                entry: &entry,
                effect: &effect,
            },
        };
        let static_material = static_bg_effect_material_packet(&atlas, &static_packet)
            .expect("static BG should produce a material packet");

        assert_eq!(static_material.material, EffectMaterial::StaticEffect);
        assert_eq!(static_material.surface, EffectSurface::Bg);
        assert_eq!(static_material.effect_row, 0);
        assert_eq!(
            static_material.instance,
            EffectInstancePacket {
                cell_id: 2,
                screen_x: 12,
                screen_y: 20,
                row_mask: 0xff,
                hflip: true,
                vflip: false,
                source_hflip: false,
                source_vflip: true,
                effect_row: 0,
            }
        );
        let mut static_words = Vec::new();
        append_effect_instance_words(&mut static_words, static_material.instance);
        assert_eq!(static_words.len() as u64, INDEX_INSTANCE_STRIDE);
        let static_encoded: Vec<u32> = static_words
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();
        assert_eq!(static_encoded, vec![16, 0, 12, 20, 0xff00 | 0b011, 0]);

        let live_packet = VariantBgDrawPacket {
            layer_index: 0,
            cell: &cell,
            inst: &inst,
            key: None,
            draw: VariantAtlasDraw::Stable { entry: &entry },
        };
        let live_material = live_cgram_bg_effect_material_packet(&live_packet)
            .expect("live BG should produce a material packet");

        assert_eq!(live_material.material, EffectMaterial::LiveCgram);
        assert_eq!(live_material.surface, EffectSurface::Bg);
        assert_eq!(live_material.effect_row, 3);

        let static_batch = bg_effect_material_batch(
            &atlas,
            BgEffectMaterialGroup {
                material: EffectMaterial::StaticEffect,
                packets: std::slice::from_ref(&static_packet),
            },
        );
        assert_eq!(static_batch.material(), Some(EffectMaterial::StaticEffect));
        assert_eq!(static_batch.instance_count(), 1);

        let live_batch = bg_effect_material_batch(
            &atlas,
            BgEffectMaterialGroup {
                material: EffectMaterial::LiveCgram,
                packets: std::slice::from_ref(&live_packet),
            },
        );
        assert_eq!(live_batch.material(), Some(EffectMaterial::LiveCgram));
        assert_eq!(live_batch.instance_count(), 1);
    }

    #[test]
    fn sprite_effect_material_packet_selects_static_or_live_rows() {
        use crate::modern_frame::ModernIndexSpriteInstance;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasDraw, VariantAtlasEntry, VariantAtlasKey,
        };
        use crate::modern_variant_draw::VariantSpriteDrawPacket;

        let entry = VariantAtlasEntry {
            id: "sprite:kSprGfx:pack0:tile0:3bpp:palette_main_spr:row1".to_string(),
            key: VariantAtlasKey {
                source_kind: "sprite".to_string(),
                asset: "kSprGfx".to_string(),
                pack: 0,
                tile: 0,
                bpp: 3,
                palette: "palette_main_spr".to_string(),
                palette_row: 1,
            },
            rect: [0, 0, 8, 8],
            sha1: "static".to_string(),
            duplicate_of: None,
            dynamic_policy: "stable".to_string(),
            runtime_material: Some("palette_lut".to_string()),
            runtime_colors_per_row: None,
            source_hflip: true,
            source_vflip: false,
        };
        let effect = TileEffect {
            id: "palette_main_spr:8color:row1".to_string(),
            palette: "palette_main_spr".to_string(),
            palette_row: 1,
            colors_per_row: 8,
            index_to_rgba: vec![
                [0, 0, 0, 0xff],
                [200, 20, 20, 0xff],
                [2, 2, 2, 0xff],
                [3, 3, 3, 0xff],
                [4, 4, 4, 0xff],
                [5, 5, 5, 0xff],
                [6, 6, 6, 0xff],
                [7, 7, 7, 0xff],
            ],
            dynamic_policy: "stable".to_string(),
        };
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![entry.clone()],
            effects: vec![effect.clone()],
        };

        let mut static_indices = [0u8; 64];
        static_indices[0] = 1;
        let static_cell = ModernIndexTile {
            id: 0,
            indices: static_indices,
            source_key: modern_source_key(2, 0, 0),
            hflip: false,
            vflip: false,
        };
        let static_inst = ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 12,
            screen_y: 20,
            palette: 1,
            priority: 0,
            hflip: false,
            vflip: true,
            row_mask: 0x7f,
        };
        let static_packet = VariantSpriteDrawPacket {
            cell: &static_cell,
            inst: &static_inst,
            key: None,
            draw: VariantAtlasDraw::MaterialEffect {
                entry: &entry,
                effect: &effect,
            },
        };
        let static_material = sprite_effect_material_packet(&atlas, &static_packet)
            .expect("static sprite should produce a material packet");

        assert_eq!(static_material.material, EffectMaterial::StaticEffect);
        assert_eq!(static_material.surface, EffectSurface::Sprite);
        assert_eq!(static_material.effect_row, 0);
        assert_eq!(
            static_material.instance,
            EffectInstancePacket {
                cell_id: 0,
                screen_x: 12,
                screen_y: 20,
                row_mask: 0x7f,
                hflip: false,
                vflip: true,
                source_hflip: true,
                source_vflip: false,
                effect_row: 0,
            }
        );
        let mut static_words = Vec::new();
        append_effect_instance_words(&mut static_words, static_material.instance);
        assert_eq!(static_words.len() as u64, INDEX_INSTANCE_STRIDE);
        let static_encoded: Vec<u32> = static_words
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();
        assert_eq!(static_encoded, vec![0, 0, 12, 20, 0x7f00 | 0b011, 0]);

        let mut live_indices = [0u8; 64];
        live_indices[0] = 9;
        let live_cell = ModernIndexTile {
            id: 1,
            indices: live_indices,
            source_key: modern_source_key(2, 0, 1),
            hflip: false,
            vflip: false,
        };
        let live_inst = ModernIndexSpriteInstance {
            cell_id: 1,
            screen_x: 0,
            screen_y: 0,
            palette: 1,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        };
        let live_packet = VariantSpriteDrawPacket {
            cell: &live_cell,
            inst: &live_inst,
            key: None,
            draw: VariantAtlasDraw::MaterialEffect {
                entry: &entry,
                effect: &effect,
            },
        };
        let live_material = sprite_effect_material_packet(&atlas, &live_packet)
            .expect("live sprite should produce a material packet");

        assert_eq!(live_material.material, EffectMaterial::LiveCgram);
        assert_eq!(live_material.surface, EffectSurface::Sprite);
        assert_eq!(live_material.effect_row, 9);
    }

    #[test]
    fn sprite_effect_material_groups_preserve_contiguous_material_runs() {
        use crate::modern_frame::ModernIndexSpriteInstance;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasDraw, VariantAtlasEntry, VariantAtlasKey,
        };
        use crate::modern_variant_draw::VariantSpriteDrawPacket;

        let entry = VariantAtlasEntry {
            id: "sprite:kSprGfx:pack0:tile0:3bpp:palette_main_spr:row1".to_string(),
            key: VariantAtlasKey {
                source_kind: "sprite".to_string(),
                asset: "kSprGfx".to_string(),
                pack: 0,
                tile: 0,
                bpp: 3,
                palette: "palette_main_spr".to_string(),
                palette_row: 1,
            },
            rect: [0, 0, 8, 8],
            sha1: "static".to_string(),
            duplicate_of: None,
            dynamic_policy: "stable".to_string(),
            runtime_material: Some("palette_lut".to_string()),
            runtime_colors_per_row: None,
            source_hflip: false,
            source_vflip: false,
        };
        let effect = TileEffect {
            id: "palette_main_spr:8color:row1".to_string(),
            palette: "palette_main_spr".to_string(),
            palette_row: 1,
            colors_per_row: 8,
            index_to_rgba: vec![[0, 0, 0, 0xff]; 8],
            dynamic_policy: "stable".to_string(),
        };
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![entry.clone()],
            effects: vec![effect.clone()],
        };

        let mut static_indices = [0u8; 64];
        static_indices[0] = 1;
        let static_cell = ModernIndexTile {
            id: 0,
            indices: static_indices,
            source_key: modern_source_key(2, 0, 0),
            hflip: false,
            vflip: false,
        };
        let mut live_indices = [0u8; 64];
        live_indices[0] = 9;
        let live_cell = ModernIndexTile {
            id: 1,
            indices: live_indices,
            source_key: modern_source_key(2, 0, 1),
            hflip: false,
            vflip: false,
        };
        let static_a = ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 1,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        };
        let live = ModernIndexSpriteInstance {
            cell_id: 1,
            ..static_a
        };
        let static_b = ModernIndexSpriteInstance {
            cell_id: 2,
            ..static_a
        };
        let packets = vec![
            VariantSpriteDrawPacket {
                cell: &static_cell,
                inst: &static_a,
                key: None,
                draw: VariantAtlasDraw::MaterialEffect {
                    entry: &entry,
                    effect: &effect,
                },
            },
            VariantSpriteDrawPacket {
                cell: &live_cell,
                inst: &live,
                key: None,
                draw: VariantAtlasDraw::MaterialEffect {
                    entry: &entry,
                    effect: &effect,
                },
            },
            VariantSpriteDrawPacket {
                cell: &static_cell,
                inst: &static_b,
                key: None,
                draw: VariantAtlasDraw::MaterialEffect {
                    entry: &entry,
                    effect: &effect,
                },
            },
        ];

        let groups = sprite_effect_material_groups(&atlas, &packets);

        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].material, EffectMaterial::StaticEffect);
        assert_eq!(groups[0].packets[0].inst.cell_id, 0);
        assert_eq!(groups[1].material, EffectMaterial::LiveCgram);
        assert_eq!(groups[1].packets[0].inst.cell_id, 1);
        assert_eq!(groups[2].material, EffectMaterial::StaticEffect);
        assert_eq!(groups[2].packets[0].inst.cell_id, 2);

        let sprite_only_rank = Mode1EffectRankDispatch {
            bg: Vec::new(),
            sprites: packets.clone(),
        };
        let first_rank_plan = sprite_only_rank.render_plan(&atlas, false);
        assert_eq!(first_rank_plan.len(), 2);
        assert_eq!(
            first_rank_plan.kinds(),
            vec![
                GpuWorkItemKind::ClearBackdrop,
                GpuWorkItemKind::SpriteEffects
            ]
        );
        assert!(matches!(
            first_rank_plan.work_items().first(),
            Some(ModernGpuWorkCommand {
                work_item: ModernGpuWorkItem::ClearBackdrop,
                ..
            })
        ));
        match &first_rank_plan.work_items()[1] {
            ModernGpuWorkCommand {
                work_item: ModernGpuWorkItem::SpriteEffects(groups),
                ..
            } => assert_eq!(groups.len(), 3),
            _ => panic!("sprite-only rank should submit sprite groups after clear"),
        }
        let prepared_first_rank = PreparedMode1EffectRenderPlan {
            rank_plans: vec![PreparedMode1EffectRankRenderPlan {
                rank_index: 0,
                rendered_before: false,
                render_plan: sprite_only_rank.render_plan(&atlas, false),
            }],
            needs_empty_frame_fallback: false,
        };
        assert_eq!(
            prepared_first_rank.into_command_kinds(),
            vec![
                PreparedMode1EffectRenderCommandKind::Work {
                    source: Mode1EffectCommandSource::Rank(0),
                    command: ModernGpuWorkCommandKind {
                        target_load: ModernGpuCommandLoad::ClearFrame,
                        work_item: GpuWorkItemKind::ClearBackdrop,
                    },
                },
                PreparedMode1EffectRenderCommandKind::Work {
                    source: Mode1EffectCommandSource::Rank(0),
                    command: ModernGpuWorkCommandKind {
                        target_load: ModernGpuCommandLoad::Load,
                        work_item: GpuWorkItemKind::SpriteEffects,
                    },
                },
            ]
        );
        let later_rank_plan = sprite_only_rank.render_plan(&atlas, true);
        assert_eq!(later_rank_plan.len(), 1);
        assert_eq!(
            later_rank_plan.kinds(),
            vec![GpuWorkItemKind::SpriteEffects]
        );
        assert!(matches!(
            later_rank_plan.work_items().first(),
            Some(ModernGpuWorkCommand {
                work_item: ModernGpuWorkItem::SpriteEffects(_),
                ..
            })
        ));
    }

    #[test]
    fn modern_gpu_variant_headless_preserves_sprite_order_across_static_and_live_effect_batches() {
        use crate::modern_frame::ModernIndexSpriteInstance;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut static_indices = [0u8; 64];
        static_indices[0] = 1;
        let mut live_indices = [0u8; 64];
        live_indices[0] = 9;
        let sprite_cells = vec![
            ModernIndexTile {
                id: 0,
                indices: static_indices,
                source_key: modern_source_key(2, 0, 0),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: live_indices,
                source_key: modern_source_key(2, 0, 1),
                hflip: false,
                vflip: false,
            },
        ];
        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.cgram_rgba[0x80 + 16 + 9] = [9, 90, 9, 0xff];
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 1,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 1,
            screen_x: 0,
            screen_y: 0,
            palette: 1,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![
                VariantAtlasEntry {
                    id: "sprite:kSprGfx:pack0:tile0:3bpp:palette_main_spr:row1".to_string(),
                    key: VariantAtlasKey {
                        source_kind: "sprite".to_string(),
                        asset: "kSprGfx".to_string(),
                        pack: 0,
                        tile: 0,
                        bpp: 3,
                        palette: "palette_main_spr".to_string(),
                        palette_row: 1,
                    },
                    rect: [0, 0, 8, 8],
                    sha1: "static".to_string(),
                    duplicate_of: None,
                    dynamic_policy: "stable".to_string(),
                    runtime_material: Some("palette_lut".to_string()),
                    runtime_colors_per_row: None,
                    source_hflip: false,
                    source_vflip: false,
                },
                VariantAtlasEntry {
                    id: "sprite:kSprGfx:pack0:tile1:3bpp:palette_main_spr:row1".to_string(),
                    key: VariantAtlasKey {
                        source_kind: "sprite".to_string(),
                        asset: "kSprGfx".to_string(),
                        pack: 0,
                        tile: 1,
                        bpp: 3,
                        palette: "palette_main_spr".to_string(),
                        palette_row: 1,
                    },
                    rect: [0, 0, 8, 8],
                    sha1: "live".to_string(),
                    duplicate_of: None,
                    dynamic_policy: "stable".to_string(),
                    runtime_material: Some("palette_lut".to_string()),
                    runtime_colors_per_row: None,
                    source_hflip: false,
                    source_vflip: false,
                },
            ],
            effects: vec![TileEffect {
                id: "palette_main_spr:8color:row1".to_string(),
                palette: "palette_main_spr".to_string(),
                palette_row: 1,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [200, 20, 20, 0xff],
                    [2, 2, 2, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };

        let (variant, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba(
            &frame,
            &[],
            &sprite_cells,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(stats.effect_draws, 2);
        assert_eq!(stats.effect_material_draws, 2);
        assert_eq!(stats.dynamic_material_draws, 2);
        assert_eq!(stats.fallback_draws, 0);
        assert_eq!(&variant[0..4], &[200, 20, 20, 0xff]);
    }

    #[test]
    fn modern_gpu_variant_headless_orders_effect_bg_and_sprites_by_mode1_priority() {
        use crate::modern_frame::{
            ModernBgLayer, ModernIndexSpriteInstance, ModernIndexTileInstance,
        };
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut bg_indices = [0u8; 64];
        bg_indices[0] = 1;
        let bg_cells = vec![ModernIndexTile {
            id: 0,
            indices: bg_indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];
        let mut sprite_indices = [0u8; 64];
        sprite_indices[0] = 1;
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices: sprite_indices,
            source_key: modern_source_key(2, 0, 0),
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: true,
        });
        frame.bg_layers[0] = layer;
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 1,
            priority: 2,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });
        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![
                VariantAtlasEntry {
                    id: "bg:kBgGfx:pack0:tile0:3bpp:palette_dung_bg_main:row2".to_string(),
                    key: VariantAtlasKey {
                        source_kind: "bg".to_string(),
                        asset: "kBgGfx".to_string(),
                        pack: 0,
                        tile: 0,
                        bpp: 3,
                        palette: "palette_dung_bg_main".to_string(),
                        palette_row: 2,
                    },
                    rect: [0, 0, 8, 8],
                    sha1: "bg".to_string(),
                    duplicate_of: None,
                    dynamic_policy: "stable".to_string(),
                    runtime_material: Some("palette_lut".to_string()),
                    runtime_colors_per_row: None,
                    source_hflip: false,
                    source_vflip: false,
                },
                VariantAtlasEntry {
                    id: "sprite:kSprGfx:pack0:tile0:3bpp:palette_main_spr:row1".to_string(),
                    key: VariantAtlasKey {
                        source_kind: "sprite".to_string(),
                        asset: "kSprGfx".to_string(),
                        pack: 0,
                        tile: 0,
                        bpp: 3,
                        palette: "palette_main_spr".to_string(),
                        palette_row: 1,
                    },
                    rect: [0, 0, 8, 8],
                    sha1: "sprite".to_string(),
                    duplicate_of: None,
                    dynamic_policy: "stable".to_string(),
                    runtime_material: Some("palette_lut".to_string()),
                    runtime_colors_per_row: None,
                    source_hflip: false,
                    source_vflip: false,
                },
            ],
            effects: vec![
                TileEffect {
                    id: "palette_dung_bg_main:8color:row2".to_string(),
                    palette: "palette_dung_bg_main".to_string(),
                    palette_row: 2,
                    colors_per_row: 8,
                    index_to_rgba: vec![
                        [0, 0, 0, 0xff],
                        [180, 40, 40, 0xff],
                        [2, 2, 2, 0xff],
                        [3, 3, 3, 0xff],
                        [4, 4, 4, 0xff],
                        [5, 5, 5, 0xff],
                        [6, 6, 6, 0xff],
                        [7, 7, 7, 0xff],
                    ],
                    dynamic_policy: "stable".to_string(),
                },
                TileEffect {
                    id: "palette_main_spr:8color:row1".to_string(),
                    palette: "palette_main_spr".to_string(),
                    palette_row: 1,
                    colors_per_row: 8,
                    index_to_rgba: vec![
                        [0, 0, 0, 0xff],
                        [40, 180, 40, 0xff],
                        [2, 2, 2, 0xff],
                        [3, 3, 3, 0xff],
                        [4, 4, 4, 0xff],
                        [5, 5, 5, 0xff],
                        [6, 6, 6, 0xff],
                        [7, 7, 7, 0xff],
                    ],
                    dynamic_policy: "stable".to_string(),
                },
            ],
        };

        let (variant, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba(
            &frame,
            &bg_cells,
            &sprite_cells,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(stats.effect_draws, 2);
        assert_eq!(stats.effect_material_draws, 2);
        assert_eq!(stats.dynamic_material_draws, 2);
        assert_eq!(stats.fallback_draws, 0);
        assert_eq!(&variant[0..4], &[180, 40, 40, 0xff]);
    }

    #[test]
    fn modern_gpu_variant_headless_missing_tiles_use_live_fallback_cells() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::ModernVariantAtlas;

        let mut source_indices = [0u8; 64];
        source_indices[0] = 1;
        let source_cells = vec![ModernIndexTile {
            id: 0,
            indices: source_indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];

        let mut fallback_indices = [0u8; 64];
        fallback_indices[0] = 1;
        let fallback_cells = vec![ModernIndexTile {
            id: 0,
            indices: fallback_indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];

        let mut source_frame = ModernFrame::empty();
        source_frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        source_frame.cgram_rgba[1] = [200, 0, 0, 0xff];
        let mut source_layer = ModernBgLayer::new(0);
        source_layer.enabled_main = true;
        source_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        source_frame.bg_layers[0] = source_layer;
        source_frame.screen_enabled_main = 0x01;

        let mut fallback_frame = ModernFrame::empty();
        fallback_frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        fallback_frame.brightness = 11;
        fallback_frame.cgram_rgba[1] = [0, 160, 80, 0xff];
        let mut fallback_layer = ModernBgLayer::new(0);
        fallback_layer.enabled_main = true;
        fallback_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        fallback_frame.bg_layers[0] = fallback_layer;
        fallback_frame.screen_enabled_main = 0x01;

        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: Vec::new(),
            effects: Vec::new(),
        };
        let (variant, stats) = ModernGpuVariantHeadless::new(&atlas)
            .render_rgba_with_live_index_base(
                &source_frame,
                &source_cells,
                &[],
                &fallback_frame,
                &fallback_cells,
                &[],
                "palette_dung_bg_main",
                "palette_main_spr",
            );
        let fallback = ModernGpuHeadless::new().render_rgba(&fallback_frame, &fallback_cells, &[]);

        assert_eq!(stats.stable_draws, 0);
        assert_eq!(stats.fallback_draws, 1);
        assert_eq!(stats.dynamic_palette_draws, 0);
        assert_eq!(stats.missing_variant_draws, 1);
        assert_eq!(stats.stable_preview_draws, 0);
        assert_eq!(stats.stable_effect_draws, 0);
        assert_eq!(stats.dynamic_material_draws, 0);
        assert_eq!(stats.missing_art_draws, 1);
        assert_eq!(stats.unkeyed_fallback_draws, 0);
        assert_eq!(stats.gpu_prefinal_base_frames, 1);
        assert_eq!(stats.cpu_prefinal_composite_frames, 0);
        assert_eq!(variant, fallback);
    }

    #[test]
    fn modern_gpu_variant_headless_mixed_fallback_keeps_compositor_pixels() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut stable_indices = [0u8; 64];
        stable_indices[0] = 1;
        let mut missing_indices = [0u8; 64];
        missing_indices[0] = 1;
        let source_cells = vec![
            ModernIndexTile {
                id: 0,
                indices: stable_indices,
                source_key: modern_source_key(1, 0, 0),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: missing_indices,
                source_key: modern_source_key(1, 9, 9),
                hflip: false,
                vflip: false,
            },
        ];
        let fallback_cells = source_cells.clone();

        let mut source_frame = ModernFrame::empty();
        source_frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        let mut source_layer = ModernBgLayer::new(0);
        source_layer.enabled_main = true;
        source_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        source_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 8,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        source_frame.bg_layers[0] = source_layer;

        let mut fallback_frame = source_frame.clone();
        fallback_frame.cgram_rgba[1] = [0, 160, 80, 0xff];

        let mut atlas_rgba = vec![0u8; 8 * 8 * 4];
        for px in atlas_rgba.chunks_exact_mut(4) {
            px.copy_from_slice(&[180, 20, 40, 0xff]);
        }
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
                    palette_row: 2,
                },
                rect: [0, 0, 8, 8],
                sha1: "stable".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [90, 100, 110, 0xff],
                    [2, 2, 2, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };

        let (variant, stats) = ModernGpuVariantHeadless::new(&atlas)
            .render_rgba_with_live_index_base(
                &source_frame,
                &source_cells,
                &[],
                &fallback_frame,
                &fallback_cells,
                &[],
                "palette_dung_bg_main",
                "palette_main_spr",
            );
        let fallback = ModernGpuHeadless::new().render_rgba(&fallback_frame, &fallback_cells, &[]);

        assert_eq!(stats.stable_draws, 0);
        assert_eq!(stats.effect_draws, 1);
        assert_eq!(stats.fallback_draws, 1);
        assert_eq!(stats.dynamic_palette_draws, 0);
        assert_eq!(stats.missing_variant_draws, 1);
        assert_eq!(stats.stable_preview_draws, 0);
        assert_eq!(stats.stable_effect_draws, 0);
        assert_eq!(stats.dynamic_material_draws, 1);
        assert_eq!(stats.missing_art_draws, 1);
        assert_eq!(stats.unkeyed_fallback_draws, 0);
        assert_eq!(&variant[0..4], &fallback[0..4]);
        let missing_offset = 8 * 4;
        assert_eq!(
            &variant[missing_offset..missing_offset + 4],
            &fallback[missing_offset..missing_offset + 4]
        );
    }

    #[test]
    fn mixed_variant_overlay_selects_only_cgram_matching_disjoint_effect_bg_packets() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut stable_indices = [0u8; 64];
        stable_indices[0] = 1;
        let mut fallback_indices = [0u8; 64];
        fallback_indices[0] = 1;
        let bg_cells = vec![
            ModernIndexTile {
                id: 0,
                indices: stable_indices,
                source_key: modern_source_key(1, 0, 0),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: fallback_indices,
                source_key: modern_source_key(1, 9, 9),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 2,
                indices: stable_indices,
                source_key: modern_source_key(1, 0, 1),
                hflip: false,
                vflip: false,
            },
        ];

        let mut frame = ModernFrame::empty();
        frame.cgram_rgba[33] = [90, 100, 110, 0xff];
        frame.cgram_rgba[49] = [12, 34, 56, 0xff];
        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 16,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 2,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 16,
            screen_y: 0,
            palette: 3,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = layer;

        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: vec![
                VariantAtlasEntry {
                    id: "bg:kBgGfx:pack0:tile0:3bpp".to_string(),
                    key: VariantAtlasKey {
                        source_kind: "bg".to_string(),
                        asset: "kBgGfx".to_string(),
                        pack: 0,
                        tile: 0,
                        bpp: 3,
                        palette: "palette_dung_bg_main".to_string(),
                        palette_row: 2,
                    },
                    rect: [0, 0, 8, 8],
                    sha1: "stable".to_string(),
                    duplicate_of: None,
                    dynamic_policy: "stable".to_string(),
                    runtime_material: Some("palette_lut".to_string()),
                    runtime_colors_per_row: None,
                    source_hflip: false,
                    source_vflip: false,
                },
                VariantAtlasEntry {
                    id: "bg:kBgGfx:pack0:tile1:3bpp".to_string(),
                    key: VariantAtlasKey {
                        source_kind: "bg".to_string(),
                        asset: "kBgGfx".to_string(),
                        pack: 0,
                        tile: 1,
                        bpp: 3,
                        palette: "palette_dung_bg_main".to_string(),
                        palette_row: 3,
                    },
                    rect: [0, 0, 8, 8],
                    sha1: "stable2".to_string(),
                    duplicate_of: None,
                    dynamic_policy: "stable".to_string(),
                    runtime_material: Some("palette_lut".to_string()),
                    runtime_colors_per_row: None,
                    source_hflip: false,
                    source_vflip: false,
                },
            ],
            effects: vec![
                TileEffect {
                    id: "palette_dung_bg_main:8color:row2".to_string(),
                    palette: "palette_dung_bg_main".to_string(),
                    palette_row: 2,
                    colors_per_row: 8,
                    index_to_rgba: vec![
                        [0, 0, 0, 0xff],
                        [90, 100, 110, 0xff],
                        [2, 2, 2, 0xff],
                        [3, 3, 3, 0xff],
                        [4, 4, 4, 0xff],
                        [5, 5, 5, 0xff],
                        [6, 6, 6, 0xff],
                        [7, 7, 7, 0xff],
                    ],
                    dynamic_policy: "stable".to_string(),
                },
                TileEffect {
                    id: "palette_dung_bg_main:8color:row3".to_string(),
                    palette: "palette_dung_bg_main".to_string(),
                    palette_row: 3,
                    colors_per_row: 8,
                    index_to_rgba: vec![[90, 100, 110, 0xff]; 8],
                    dynamic_policy: "stable".to_string(),
                },
            ],
        };
        let plan = crate::modern_variant_draw::compile_variant_draws(
            &frame,
            &bg_cells,
            &[],
            &atlas,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        let selection = mixed_variant_overlay_bg_packets(&frame, &plan);

        assert_eq!(selection.effects.static_bg_len(), 1);
        assert_eq!(selection.effects.static_bg_packets()[0].inst.cell_id, 0);
        assert_eq!(selection.candidates, 2);
        assert_eq!(selection.reject_complex_frame, 0);
        assert_eq!(selection.reject_cgram_mismatch, 0);
        assert_eq!(selection.reject_overlap, 1);

        let prefinal_packets = MixedVariantPrefinalPackets::from_all_overlay(&selection, &plan);
        assert_eq!(prefinal_packets.static_bg_len(), 1);
        assert_eq!(prefinal_packets.live_cgram_bg_len(), 0);
        assert_eq!(prefinal_packets.sprites.len(), 0);
        assert_eq!(prefinal_packets.bg_len(), 1);

        let prefinal_selection = mixed_variant_prefinal_bg_packets(&frame, &plan);
        let render_prefinal_packets =
            MixedVariantPrefinalPackets::from_overlay(&frame, &prefinal_selection, &plan);
        assert_eq!(
            render_prefinal_packets.bg_len(),
            0,
            "the headless render path only records selected BG packets that need prefinal color math"
        );

        let (_rgba, stats) = ModernGpuVariantHeadless::new(&atlas)
            .render_rgba_with_live_index_base(
                &frame,
                &bg_cells,
                &[],
                &frame,
                &bg_cells,
                &[],
                "palette_dung_bg_main",
                "palette_main_spr",
            );

        assert_eq!(stats.mixed_overlay_bg_effect_draws, 0);
        assert_eq!(stats.mixed_overlay_bg_effect_candidates, 2);
        assert_eq!(stats.mixed_overlay_bg_effect_reject_complex_frame, 0);
        assert_eq!(stats.mixed_overlay_bg_effect_reject_cgram_mismatch, 0);
        assert_eq!(stats.mixed_overlay_bg_effect_reject_overlap, 0);
    }

    #[test]
    fn prefinal_bg_overlay_only_recolors_the_winning_layer() {
        use crate::modern_frame::{ModernFrame, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_software::ModernCompositedScreens;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            TileEffect, VariantAtlasDraw, VariantAtlasEntry, VariantAtlasKey,
        };
        use crate::modern_variant_draw::VariantBgDrawPacket;

        let mut indices = [0u8; 64];
        indices[0] = 1;
        let cell = ModernIndexTile {
            id: 0,
            indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        };
        let inst = ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        };
        let entry = VariantAtlasEntry {
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
            sha1: "stable".to_string(),
            duplicate_of: None,
            dynamic_policy: "stable".to_string(),
            runtime_material: Some("palette_lut".to_string()),
            runtime_colors_per_row: None,
            source_hflip: false,
            source_vflip: false,
        };
        let effect = TileEffect {
            id: "palette_dung_bg_main:8color:row0".to_string(),
            palette: "palette_dung_bg_main".to_string(),
            palette_row: 0,
            colors_per_row: 8,
            index_to_rgba: vec![[0, 0, 0, 0xff], [248, 0, 0, 0xff]],
            dynamic_policy: "stable".to_string(),
        };
        let packet = VariantBgDrawPacket {
            layer_index: 0,
            cell: &cell,
            inst: &inst,
            key: None,
            draw: VariantAtlasDraw::MaterialEffect {
                entry: &entry,
                effect: &effect,
            },
        };
        let frame = ModernFrame::empty();
        let original = pack_variant_prefinal_pixel([0, 0, 248, 0xff], 2);
        let replacement = pack_variant_prefinal_pixel([248, 0, 0, 0xff], 0);
        let mut screens = ModernCompositedScreens {
            width: 256,
            scale: 1,
            main: vec![original; 256 * 224],
        };

        overlay_mixed_variant_bg_packets_on_main_screen(
            &mut screens,
            &frame,
            &MixedVariantPrefinalPackets {
                bg: vec![MixedVariantPrefinalBgPacket {
                    material: PrefinalBgMaterial::StaticEffect,
                    packet: packet.clone(),
                }],
                sprites: Vec::new(),
            },
        );

        assert_eq!(screens.main[0], original);
        screens.main[0] = pack_variant_prefinal_pixel([0, 0, 248, 0xff], 0);
        overlay_mixed_variant_bg_packets_on_main_screen(
            &mut screens,
            &frame,
            &MixedVariantPrefinalPackets {
                bg: vec![MixedVariantPrefinalBgPacket {
                    material: PrefinalBgMaterial::StaticEffect,
                    packet: packet.clone(),
                }],
                sprites: Vec::new(),
            },
        );

        assert_eq!(screens.main[0], replacement);
    }

    #[test]
    fn prefinal_overlay_data_words_preserve_material_packet_sources() {
        use crate::modern_frame::{ModernFrame, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            TileEffect, VariantAtlasDraw, VariantAtlasEntry, VariantAtlasKey,
        };
        use crate::modern_variant_draw::VariantBgDrawPacket;

        fn entry(id: &str, row: u8) -> VariantAtlasEntry {
            VariantAtlasEntry {
                id: id.to_string(),
                key: VariantAtlasKey {
                    source_kind: "bg".to_string(),
                    asset: "kBgGfx".to_string(),
                    pack: 0,
                    tile: u16::from(row),
                    bpp: 3,
                    palette: "palette_dung_bg_main".to_string(),
                    palette_row: row,
                },
                rect: [0, 0, 8, 8],
                sha1: id.to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }
        }

        fn effect(id: &str, row: u8, rgba: [u8; 4]) -> TileEffect {
            TileEffect {
                id: id.to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: row,
                colors_per_row: 8,
                index_to_rgba: vec![[0, 0, 0, 0xff], rgba],
                dynamic_policy: "stable".to_string(),
            }
        }

        let mut static_indices = [0u8; 64];
        static_indices[0] = 1;
        let static_cell = ModernIndexTile {
            id: 0,
            indices: static_indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        };
        let static_inst = ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        };
        let static_entry = entry("static-bg", 0);
        let static_effect = effect("static-effect", 0, [248, 0, 0, 0xff]);
        let static_packet = VariantBgDrawPacket {
            layer_index: 0,
            cell: &static_cell,
            inst: &static_inst,
            key: None,
            draw: VariantAtlasDraw::MaterialEffect {
                entry: &static_entry,
                effect: &static_effect,
            },
        };

        let mut live_indices = [0u8; 64];
        live_indices[0] = 1;
        let live_cell = ModernIndexTile {
            id: 1,
            indices: live_indices,
            source_key: modern_source_key(1, 0, 1),
            hflip: false,
            vflip: false,
        };
        let live_inst = ModernIndexTileInstance {
            cell_id: 1,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 8,
            screen_y: 0,
            palette: 1,
            hflip: false,
            vflip: false,
            priority: false,
        };
        let live_entry = entry("live-bg", 1);
        let live_effect = effect("live-effect", 1, [0, 0, 248, 0xff]);
        let live_packet = VariantBgDrawPacket {
            layer_index: 1,
            cell: &live_cell,
            inst: &live_inst,
            key: None,
            draw: VariantAtlasDraw::MaterialEffect {
                entry: &live_entry,
                effect: &live_effect,
            },
        };

        let mut frame = ModernFrame::empty();
        frame.cgram_rgba[17] = [0, 248, 0, 0xff];
        let packets = MixedVariantPrefinalPackets {
            bg: vec![
                MixedVariantPrefinalBgPacket {
                    material: PrefinalBgMaterial::StaticEffect,
                    packet: static_packet,
                },
                MixedVariantPrefinalBgPacket {
                    material: PrefinalBgMaterial::LiveCgram,
                    packet: live_packet,
                },
            ],
            sprites: Vec::new(),
        };

        let (data_words, params) = modern_prefinal_overlay_data_words(&frame, &packets);

        assert_eq!(params[1], 2);
        assert_eq!(params[2], 0);
        assert_eq!(params[6], 128);
        assert_eq!(params[7], 136);
        assert_eq!(
            data_words[0],
            pack_variant_prefinal_pixel([248, 0, 0, 0xff], 0)
        );
        assert_eq!(
            data_words[64],
            pack_variant_prefinal_pixel([0, 248, 0, 0xff], 1)
        );
        assert_eq!(
            &data_words[params[6] as usize..params[6] as usize + 8],
            &[0, 0, 4, 0, 8, 0, 3, 64]
        );
        assert_eq!(data_words[params[7] as usize], 0);
    }

    #[test]
    fn mixed_variant_overlay_allows_subscreen_when_color_math_cannot_change_packet() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut stable_indices = [0u8; 64];
        stable_indices[0] = 1;
        let bg_cells = vec![ModernIndexTile {
            id: 0,
            indices: stable_indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.screen_enabled_sub = 0x02;
        frame.cgram_rgba[33] = [90, 100, 110, 0xff];
        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = layer;

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
                    palette_row: 2,
                },
                rect: [0, 0, 8, 8],
                sha1: "stable".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [90, 100, 110, 0xff],
                    [2, 2, 2, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };
        let plan = crate::modern_variant_draw::compile_variant_draws(
            &frame,
            &bg_cells,
            &[],
            &atlas,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        let selection = mixed_variant_overlay_bg_packets(&frame, &plan);

        assert_eq!(selection.effects.static_bg_len(), 1);
        assert_eq!(selection.candidates, 1);
        assert_eq!(selection.reject_complex_frame, 0);
        assert_eq!(selection.reject_cgram_mismatch, 0);
        assert_eq!(selection.reject_overlap, 0);
    }

    #[test]
    fn mixed_variant_overlay_counts_brightness_complex_rejects() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut stable_indices = [0u8; 64];
        stable_indices[0] = 1;
        let bg_cells = vec![ModernIndexTile {
            id: 0,
            indices: stable_indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.brightness = 14;
        frame.cgram_rgba[33] = [90, 100, 110, 0xff];
        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = layer;

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
                    palette_row: 2,
                },
                rect: [0, 0, 8, 8],
                sha1: "stable".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [90, 100, 110, 0xff],
                    [2, 2, 2, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };
        let plan = crate::modern_variant_draw::compile_variant_draws(
            &frame,
            &bg_cells,
            &[],
            &atlas,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        let selection = mixed_variant_overlay_bg_packets(&frame, &plan);

        assert_eq!(selection.effects.static_bg_len(), 0);
        assert_eq!(selection.effects.live_cgram_bg_len(), 0);
        assert_eq!(selection.candidates, 1);
        assert_eq!(selection.reject_complex_frame, 1);
        assert_eq!(selection.reject_complex_brightness, 1);
    }

    #[test]
    fn mixed_variant_overlay_counts_fixed_color_math_rejects() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut stable_indices = [0u8; 64];
        stable_indices[0] = 1;
        let bg_cells = vec![ModernIndexTile {
            id: 0,
            indices: stable_indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.cgram_rgba[33] = [90, 100, 110, 0xff];
        frame.math_enabled = 0x01;
        frame.fixed_color_r = 1;
        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = layer;

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
                    palette_row: 2,
                },
                rect: [0, 0, 8, 8],
                sha1: "stable".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [90, 100, 110, 0xff],
                    [2, 2, 2, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };
        let plan = crate::modern_variant_draw::compile_variant_draws(
            &frame,
            &bg_cells,
            &[],
            &atlas,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        let selection = mixed_variant_overlay_bg_packets(&frame, &plan);

        assert_eq!(selection.reject_complex_frame, 1);
        assert_eq!(selection.reject_complex_color_math, 1);
        assert_eq!(selection.reject_complex_color_math_fixed_color, 1);
    }

    #[test]
    fn mixed_variant_overlay_selects_live_cgram_when_static_effect_differs() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut stable_indices = [0u8; 64];
        stable_indices[0] = 1;
        let bg_cells = vec![ModernIndexTile {
            id: 0,
            indices: stable_indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.screen_enabled_sub = 0x02;
        frame.cgram_rgba[33] = [90, 100, 110, 0xff];
        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = layer;

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
                    palette_row: 2,
                },
                rect: [0, 0, 8, 8],
                sha1: "stable".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![[12, 34, 56, 0xff]; 8],
                dynamic_policy: "stable".to_string(),
            }],
        };
        let plan = crate::modern_variant_draw::compile_variant_draws(
            &frame,
            &bg_cells,
            &[],
            &atlas,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        let selection = mixed_variant_overlay_bg_packets(&frame, &plan);

        assert_eq!(selection.effects.static_bg_len(), 0);
        assert_eq!(selection.effects.live_cgram_bg_len(), 1);
        assert_eq!(selection.candidates, 1);
        assert_eq!(selection.reject_complex_frame, 0);
        assert_eq!(selection.reject_cgram_mismatch, 0);
        assert_eq!(selection.reject_overlap, 0);
    }

    #[test]
    fn mixed_variant_overlay_allows_rect_overlap_when_opaque_pixels_do_not_overlap() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut stable_indices = [0u8; 64];
        stable_indices[0] = 1;
        let mut neighbor_indices = [0u8; 64];
        neighbor_indices[7] = 1;
        let bg_cells = vec![
            ModernIndexTile {
                id: 0,
                indices: stable_indices,
                source_key: modern_source_key(1, 0, 0),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: neighbor_indices,
                source_key: modern_source_key(1, 9, 9),
                hflip: false,
                vflip: false,
            },
        ];

        let mut frame = ModernFrame::empty();
        frame.cgram_rgba[33] = [90, 100, 110, 0xff];
        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 1,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = layer;

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
                    palette_row: 2,
                },
                rect: [0, 0, 8, 8],
                sha1: "stable".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [90, 100, 110, 0xff],
                    [2, 2, 2, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };
        let plan = crate::modern_variant_draw::compile_variant_draws(
            &frame,
            &bg_cells,
            &[],
            &atlas,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        let selection = mixed_variant_overlay_bg_packets(&frame, &plan);

        assert_eq!(selection.effects.static_bg_len(), 1);
        assert_eq!(selection.candidates, 1);
        assert_eq!(selection.reject_overlap, 0);
    }

    #[test]
    fn modern_gpu_variant_headless_unkeyed_tiles_are_live_index_not_missing() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_variant_atlas::ModernVariantAtlas;

        let mut indices = [0u8; 64];
        indices[0] = 1;
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.backdrop_color_rgba = [0, 0, 0, 0xff];
        frame.cgram_rgba[1] = [40, 80, 120, 0xff];
        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = layer;

        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: Vec::new(),
            effects: Vec::new(),
        };
        let (_variant, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba(
            &frame,
            &cells,
            &[],
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(stats.stable_draws, 0);
        assert_eq!(stats.fallback_draws, 0);
        assert_eq!(stats.live_index_draws, 1);
        assert_eq!(stats.live_index_bg_draws, 1);
        assert_eq!(stats.live_index_bg12_draws, 1);
        assert_eq!(stats.live_index_bg3_draws, 0);
        assert_eq!(stats.live_index_sprite_draws, 0);
        assert_eq!(stats.dynamic_palette_draws, 0);
        assert_eq!(stats.missing_variant_draws, 0);
        assert_eq!(stats.stable_preview_draws, 0);
        assert_eq!(stats.stable_effect_draws, 0);
        assert_eq!(stats.dynamic_material_draws, 0);
        assert_eq!(stats.missing_art_draws, 0);
        assert_eq!(stats.unkeyed_fallback_draws, 1);
        assert_eq!(stats.gpu_prefinal_base_frames, 1);
        assert_eq!(stats.gpu_screen_builder_frames, 1);
        assert_eq!(stats.cpu_prefinal_composite_frames, 0);
    }

    #[test]
    fn modern_gpu_variant_headless_builds_prefinal_main_obj_sub_bg_on_gpu() {
        use crate::modern_frame::{
            ModernBgLayer, ModernIndexSpriteInstance, ModernIndexTileInstance,
        };
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::ModernVariantAtlas;

        let mut bg_indices = [0u8; 64];
        bg_indices[0] = 1;
        let bg_cells = vec![ModernIndexTile {
            id: 0,
            indices: bg_indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];
        let mut sprite_indices = [0u8; 64];
        sprite_indices[0] = 1;
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices: sprite_indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.screen_enabled_main = 0x10;
        frame.screen_enabled_sub = 0x01;
        frame.math_enabled = 0x10;
        frame.add_subscreen = true;
        frame.cgram_rgba[1] = [0, 80, 0, 0xff];
        frame.cgram_rgba[0x80 + 4 * 16 + 1] = [80, 0, 0, 0xff];

        let mut sub_layer = ModernBgLayer::new(0);
        sub_layer.enabled_sub = true;
        sub_layer.scroll_x = 1;
        sub_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = sub_layer;
        frame.bg_scroll_scanlines =
            vec![[[0u16; 2]; 4]; usize::from(crate::modern_frame::MODERN_FRAME_HEIGHT)];
        for row in &mut frame.bg_scroll_scanlines {
            row[0] = [1, 0];
        }
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 4,
            priority: 3,
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
        let (variant, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba(
            &frame,
            &bg_cells,
            &sprite_cells,
            "palette_dung_bg_main",
            "palette_main_spr",
        );
        let cpu =
            crate::modern_software::render_modern_frame_full(&frame, &bg_cells, &sprite_cells);

        assert_eq!(stats.gpu_prefinal_base_frames, 1);
        assert_eq!(stats.gpu_screen_builder_frames, 1);
        assert_eq!(stats.cpu_prefinal_composite_frames, 0);
        assert_eq!(variant, cpu);
    }

    #[test]
    fn modern_gpu_variant_headless_short_scroll_table_stays_on_gpu() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_variant_atlas::ModernVariantAtlas;

        let indices = [1u8; 64];
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.screen_enabled_main = 0x01;
        frame.brightness = 15;
        frame.cgram_rgba[1] = [160, 0, 0, 0xff];
        frame.cgram_rgba[16 + 1] = [0, 160, 0, 0xff];

        let mut bg1 = ModernBgLayer::new(0);
        bg1.enabled_main = true;
        bg1.wrap_w = 256;
        bg1.wrap_h = 256;
        bg1.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        bg1.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: NO_SOURCE_KEY,
            screen_x: 8,
            screen_y: 0,
            palette: 1,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = bg1;
        frame.bg_scroll_scanlines = vec![[[0u16; 2]; 4]];
        frame.bg_scroll_scanlines[0][0] = [8, 0];

        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: Vec::new(),
            effects: Vec::new(),
        };
        let (variant, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba(
            &frame,
            &cells,
            &[],
            "palette_dung_bg_main",
            "palette_main_spr",
        );
        let cpu = crate::modern_software::render_modern_frame_full(&frame, &cells, &[]);

        assert_eq!(stats.gpu_prefinal_base_frames, 1);
        assert_eq!(stats.gpu_screen_builder_frames, 1);
        assert_eq!(stats.cpu_prefinal_composite_frames, 0);
        assert_eq!(variant, cpu);
    }

    #[test]
    fn modern_gpu_variant_headless_mosaic_stays_on_gpu() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_variant_atlas::ModernVariantAtlas;

        let mut indices = [0u8; 64];
        for (i, index) in indices.iter_mut().enumerate() {
            *index = (i + 1) as u8;
        }
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.screen_enabled_main = 0x01;
        frame.brightness = 15;
        frame.mosaic_enabled = 0x01;
        frame.mosaic_size = 4;
        for i in 1..=64usize {
            let k = i as u8;
            frame.cgram_rgba[i] = [(k & 7) << 5, ((k >> 3) & 7) << 5, ((k >> 6) & 3) << 6, 0xff];
        }

        let mut bg1 = ModernBgLayer::new(0);
        bg1.enabled_main = true;
        bg1.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = bg1;

        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: Vec::new(),
            effects: Vec::new(),
        };
        let (variant, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba(
            &frame,
            &cells,
            &[],
            "palette_dung_bg_main",
            "palette_main_spr",
        );
        let cpu = crate::modern_software::render_modern_frame_full(&frame, &cells, &[]);

        assert_eq!(stats.gpu_prefinal_base_frames, 1);
        assert_eq!(stats.gpu_screen_builder_frames, 1);
        assert_eq!(stats.cpu_prefinal_composite_frames, 0);
        assert_eq!(variant, cpu);
    }

    #[test]
    fn modern_gpu_variant_headless_short_bg_layers_stays_on_gpu() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;

        pollster::block_on(async {
            let instance = crate::create_wgpu_instance();
            let (_adapter, device, queue) = crate::create_device_queue(&instance, None).await;
            let compositor =
                ModernGpuCompositor::new(&device, &queue, wgpu::TextureFormat::Rgba8Unorm);

            let indices = [1u8; 64];
            let cells = vec![ModernIndexTile {
                id: 0,
                indices,
                source_key: NO_SOURCE_KEY,
                hflip: false,
                vflip: false,
            }];

            let mut frame = ModernFrame::empty();
            frame.screen_enabled_main = 0x01;
            frame.brightness = 15;
            frame.cgram_rgba[1] = [160, 0, 0, 0xff];

            let mut bg1 = ModernBgLayer::new(0);
            bg1.enabled_main = true;
            bg1.index_tiles.push(ModernIndexTileInstance {
                cell_id: 0,
                source_key: NO_SOURCE_KEY,
                screen_x: 0,
                screen_y: 0,
                palette: 0,
                hflip: false,
                vflip: false,
                priority: false,
            });
            frame.bg_layers = vec![bg1];
            let target = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("modern_gpu_short_bg_layers_screen_builder_test_target"),
                size: wgpu::Extent3d {
                    width: 256,
                    height: 224,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            assert!(compositor.render(&device, &queue, &frame, &cells, &[], &target));
        });
    }

    #[test]
    fn modern_gpu_variant_headless_bg4_enabled_stays_on_gpu() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;

        pollster::block_on(async {
            let instance = crate::create_wgpu_instance();
            let (_adapter, device, queue) = crate::create_device_queue(&instance, None).await;
            let compositor =
                ModernGpuCompositor::new(&device, &queue, wgpu::TextureFormat::Rgba8Unorm);

            let indices = [1u8; 64];
            let cells = vec![ModernIndexTile {
                id: 0,
                indices,
                source_key: NO_SOURCE_KEY,
                hflip: false,
                vflip: false,
            }];

            let mut frame = ModernFrame::empty();
            frame.screen_enabled_main = 0x08;
            frame.brightness = 15;
            frame.backdrop_color_rgba = [24, 32, 40, 0xff];
            frame.cgram_rgba[1] = [160, 0, 0, 0xff];

            let mut bg4 = ModernBgLayer::new(3);
            bg4.enabled_main = true;
            bg4.index_tiles.push(ModernIndexTileInstance {
                cell_id: 0,
                source_key: NO_SOURCE_KEY,
                screen_x: 0,
                screen_y: 0,
                palette: 0,
                hflip: false,
                vflip: false,
                priority: false,
            });
            frame.bg_layers[3] = bg4;
            let target = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("modern_gpu_bg4_screen_builder_test_target"),
                size: wgpu::Extent3d {
                    width: 256,
                    height: 224,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            assert!(compositor.render(&device, &queue, &frame, &cells, &[], &target));
        });
    }

    #[test]
    fn modern_gpu_variant_headless_forced_blank_fallback_stays_direct_gpu() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_variant_atlas::ModernVariantAtlas;

        let mut indices = [0u8; 64];
        indices[0] = 1;
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut source_frame = ModernFrame::empty();
        source_frame.cgram_rgba[1] = [248, 248, 248, 0xff];
        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        source_frame.bg_layers[0] = layer;
        source_frame.screen_enabled_main = 0x01;

        let mut fallback_frame = source_frame.clone();
        fallback_frame.forced_blank = true;

        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: Vec::new(),
            effects: Vec::new(),
        };
        let (variant, stats) = ModernGpuVariantHeadless::new(&atlas)
            .render_rgba_with_live_index_base(
                &source_frame,
                &cells,
                &[],
                &fallback_frame,
                &cells,
                &[],
                "palette_dung_bg_main",
                "palette_main_spr",
            );

        assert_eq!(stats.fallback_draws, 0);
        assert_eq!(stats.live_index_draws, 1);
        assert_eq!(stats.live_index_bg_draws, 1);
        assert_eq!(stats.live_index_bg12_draws, 1);
        assert_eq!(stats.live_index_bg3_draws, 0);
        assert_eq!(stats.live_index_sprite_draws, 0);
        assert_eq!(stats.gpu_prefinal_base_frames, 1);
        assert_eq!(stats.cpu_prefinal_composite_frames, 0);
        assert_eq!(&variant[0..4], &[0, 0, 0, 0xff]);
    }

    #[test]
    fn modern_gpu_variant_headless_nonoverlapping_bg_layers_stay_direct_gpu() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_variant_atlas::ModernVariantAtlas;

        let mut indices = [0u8; 64];
        indices[0] = 1;
        let cells = vec![ModernIndexTile {
            id: 0,
            indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.cgram_rgba[1] = [80, 120, 160, 0xff];
        frame.brightness = 9;
        for layer_index in 0..2u8 {
            let mut layer = ModernBgLayer::new(layer_index);
            layer.enabled_main = true;
            layer.index_tiles.push(ModernIndexTileInstance {
                cell_id: 0,
                source_key: NO_SOURCE_KEY,
                screen_x: i16::from(layer_index) * 8,
                screen_y: 0,
                palette: 0,
                hflip: false,
                vflip: false,
                priority: layer_index == 1,
            });
            frame.bg_layers[layer_index as usize] = layer;
        }
        frame.screen_enabled_main = 0x03;

        let atlas = ModernVariantAtlas {
            width: 8,
            height: 8,
            rgba: vec![0u8; 8 * 8 * 4],
            entries: Vec::new(),
            effects: Vec::new(),
        };
        let (variant, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba(
            &frame,
            &cells,
            &[],
            "palette_dung_bg_main",
            "palette_main_spr",
        );
        let fallback = ModernGpuHeadless::new().render_rgba(&frame, &cells, &[]);

        assert_eq!(stats.fallback_draws, 0);
        assert_eq!(stats.live_index_draws, 2);
        assert_eq!(stats.live_index_bg_draws, 2);
        assert_eq!(stats.live_index_bg12_draws, 2);
        assert_eq!(stats.live_index_bg3_draws, 0);
        assert_eq!(stats.live_index_sprite_draws, 0);
        assert_eq!(stats.gpu_prefinal_base_frames, 1);
        assert_eq!(stats.cpu_prefinal_composite_frames, 0);
        assert_eq!(variant, fallback);
    }

    #[test]
    fn modern_gpu_variant_headless_keeps_live_index_base_when_mixed_overlay_palette_diverges() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let source_key = modern_source_key(1, 0, 0);
        let mut material_indices = [0u8; 64];
        material_indices[0] = 1;
        let mut live_indices = [0u8; 64];
        live_indices[0] = 1;
        let cells = vec![
            ModernIndexTile {
                id: 0,
                indices: material_indices,
                source_key,
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: live_indices,
                source_key: NO_SOURCE_KEY,
                hflip: false,
                vflip: false,
            },
        ];

        let mut source_frame = ModernFrame::empty();
        source_frame.screen_enabled_main = 0x01;
        source_frame.brightness = 15;
        source_frame.cgram_rgba[1] = [80, 0, 0, 0xff];
        let mut source_layer = ModernBgLayer::new(0);
        source_layer.enabled_main = true;
        source_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        source_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            source_key: NO_SOURCE_KEY,
            screen_x: 8,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        source_frame.bg_layers[0] = source_layer;

        let mut live_index_frame = source_frame.clone();
        live_index_frame.cgram_rgba[1] = [40, 0, 0, 0xff];

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
                sha1: "stable".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row0".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 0,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [8, 0, 0, 0xff],
                    [2, 2, 2, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };

        let (variant, stats) = ModernGpuVariantHeadless::new(&atlas)
            .render_rgba_with_live_index_base(
                &source_frame,
                &cells,
                &[],
                &live_index_frame,
                &cells,
                &[],
                "palette_dung_bg_main",
                "palette_main_spr",
            );
        let fallback = ModernGpuHeadless::new().render_rgba(&live_index_frame, &cells, &[]);

        assert_eq!(stats.live_index_draws, 1);
        assert_eq!(stats.mixed_overlay_bg_effect_candidates, 1);
        assert_eq!(stats.mixed_overlay_bg_effect_draws, 0);
        assert_eq!(stats.gpu_prefinal_base_frames, 1);
        assert_eq!(stats.cpu_prefinal_composite_frames, 0);
        assert_eq!(&variant[0..4], &fallback[0..4]);
    }

    #[test]
    fn modern_gpu_variant_headless_applies_subscreen_math_to_mixed_effect_bg() {
        use crate::modern_frame::{
            ModernBgLayer, ModernIndexSpriteInstance, ModernIndexTileInstance,
        };
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut main_indices = [0u8; 64];
        main_indices[0] = 1;
        let mut sub_indices = [0u8; 64];
        sub_indices[0] = 1;
        let bg_cells = vec![
            ModernIndexTile {
                id: 0,
                indices: main_indices,
                source_key: modern_source_key(1, 0, 0),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: sub_indices,
                source_key: modern_source_key(1, 9, 9),
                hflip: false,
                vflip: false,
            },
        ];
        let mut sprite_indices = [0u8; 64];
        sprite_indices[0] = 1;
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices: sprite_indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.screen_enabled_main = 0x11;
        frame.screen_enabled_sub = 0x02;
        frame.math_enabled = 0x01;
        frame.add_subscreen = true;
        frame.cgram_rgba[33] = [80, 0, 0, 0xff];
        frame.cgram_rgba[1] = [24, 0, 0, 0xff];
        frame.cgram_rgba[0x81] = [0, 80, 0, 0xff];

        let mut main_layer = ModernBgLayer::new(0);
        main_layer.enabled_main = true;
        main_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = main_layer;

        let mut sub_layer = ModernBgLayer::new(1);
        sub_layer.enabled_sub = true;
        sub_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[1] = sub_layer;
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 32,
            screen_y: 32,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });

        let mut fallback_frame = frame.clone();
        fallback_frame.cgram_rgba[33] = [40, 0, 0, 0xff];

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
                    palette_row: 2,
                },
                rect: [0, 0, 8, 8],
                sha1: "stable".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [80, 0, 0, 0xff],
                    [2, 2, 2, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };

        let (rgba, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba_with_live_index_base(
            &frame,
            &bg_cells,
            &sprite_cells,
            &fallback_frame,
            &bg_cells,
            &sprite_cells,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(&rgba[0..4], &[107, 0, 0, 0xff]);
        assert_eq!(stats.mixed_overlay_bg_effect_draws, 1);
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_subscreen,
            0
        );
    }

    #[test]
    fn modern_gpu_variant_headless_masks_scanline_disabled_effect_bg_pixels() {
        use crate::modern_frame::{
            ModernBgLayer, ModernIndexSpriteInstance, ModernIndexTileInstance,
        };
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut main_indices = [0u8; 64];
        main_indices[0] = 1;
        main_indices[8] = 1;
        let mut sub_indices = [0u8; 64];
        sub_indices[0] = 1;
        sub_indices[8] = 1;
        let bg_cells = vec![
            ModernIndexTile {
                id: 0,
                indices: main_indices,
                source_key: modern_source_key(1, 0, 0),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: sub_indices,
                source_key: modern_source_key(1, 9, 9),
                hflip: false,
                vflip: false,
            },
        ];
        let mut sprite_indices = [0u8; 64];
        sprite_indices[0] = 1;
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices: sprite_indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.screen_enabled_main = 0x11;
        frame.screen_enabled_sub = 0x02;
        frame.math_enabled = 0x01;
        frame.add_subscreen = true;
        frame.cgram_rgba[33] = [80, 0, 0, 0xff];
        frame.cgram_rgba[1] = [24, 0, 0, 0xff];
        frame.cgram_rgba[0x81] = [0, 80, 0, 0xff];
        frame.main_tm_scanlines[1] = 0x10;

        let mut main_layer = ModernBgLayer::new(0);
        main_layer.enabled_main = true;
        main_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = main_layer;

        let mut sub_layer = ModernBgLayer::new(1);
        sub_layer.enabled_sub = true;
        sub_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[1] = sub_layer;
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 32,
            screen_y: 32,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });

        let mut fallback_frame = frame.clone();
        fallback_frame.cgram_rgba[33] = [40, 0, 0, 0xff];

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
                    palette_row: 2,
                },
                rect: [0, 0, 8, 8],
                sha1: "stable".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [80, 0, 0, 0xff],
                    [2, 2, 2, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };

        let (rgba, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba_with_live_index_base(
            &frame,
            &bg_cells,
            &sprite_cells,
            &fallback_frame,
            &bg_cells,
            &sprite_cells,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        let row1 = 256 * 4;
        assert_eq!(&rgba[0..4], &[107, 0, 0, 0xff]);
        assert_eq!(&rgba[row1..row1 + 4], &[0, 0, 0, 0xff]);
        assert_eq!(stats.mixed_overlay_bg_effect_draws, 1, "{stats:?}");
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_scanline_main,
            0
        );
    }

    #[test]
    fn modern_gpu_variant_headless_culls_fully_scanline_disabled_effect_bg_packets() {
        use crate::modern_frame::{
            ModernBgLayer, ModernIndexSpriteInstance, ModernIndexTileInstance,
        };
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut main_indices = [0u8; 64];
        main_indices[0] = 1;
        let bg_cells = vec![ModernIndexTile {
            id: 0,
            indices: main_indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];
        let mut sprite_indices = [0u8; 64];
        sprite_indices[0] = 1;
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices: sprite_indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.screen_enabled_main = 0x11;
        frame.cgram_rgba[33] = [80, 0, 0, 0xff];
        frame.cgram_rgba[0x81] = [0, 80, 0, 0xff];
        frame.main_tm_scanlines[0] = 0x10;

        let mut main_layer = ModernBgLayer::new(0);
        main_layer.enabled_main = true;
        main_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = main_layer;
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 32,
            screen_y: 32,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });

        let fallback_frame = frame.clone();
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
                    palette_row: 2,
                },
                rect: [0, 0, 8, 8],
                sha1: "stable".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [80, 0, 0, 0xff],
                    [2, 2, 2, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };

        let (_rgba, stats) = ModernGpuVariantHeadless::new(&atlas)
            .render_rgba_with_live_index_base(
                &frame,
                &bg_cells,
                &sprite_cells,
                &fallback_frame,
                &bg_cells,
                &sprite_cells,
                "palette_dung_bg_main",
                "palette_main_spr",
            );

        assert_eq!(stats.mixed_overlay_bg_effect_candidates, 1, "{stats:?}");
        assert_eq!(
            stats.mixed_overlay_bg_effect_culled_invisible_main, 1,
            "{stats:?}"
        );
        assert_eq!(stats.mixed_overlay_bg_effect_draws, 0, "{stats:?}");
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_frame, 0,
            "{stats:?}"
        );
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_scanline_main, 0,
            "{stats:?}"
        );
    }

    #[test]
    fn modern_gpu_variant_headless_applies_subscreen_math_to_mixed_live_cgram_bg() {
        use crate::modern_frame::{
            ModernBgLayer, ModernIndexSpriteInstance, ModernIndexTileInstance,
        };
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut main_indices = [0u8; 64];
        main_indices[0] = 1;
        main_indices[7] = 2;
        let mut sub_indices = [0u8; 64];
        sub_indices[0] = 1;
        let bg_cells = vec![
            ModernIndexTile {
                id: 0,
                indices: main_indices,
                source_key: modern_source_key(1, 0, 0),
                hflip: true,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: sub_indices,
                source_key: modern_source_key(1, 9, 9),
                hflip: false,
                vflip: false,
            },
        ];
        let mut sprite_indices = [0u8; 64];
        sprite_indices[0] = 1;
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices: sprite_indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.screen_enabled_main = 0x11;
        frame.screen_enabled_sub = 0x02;
        frame.math_enabled = 0x01;
        frame.add_subscreen = true;
        frame.cgram_rgba[33] = [80, 0, 0, 0xff];
        frame.cgram_rgba[34] = [160, 0, 0, 0xff];
        frame.cgram_rgba[1] = [24, 0, 0, 0xff];
        frame.cgram_rgba[0x81] = [0, 80, 0, 0xff];

        let mut main_layer = ModernBgLayer::new(0);
        main_layer.enabled_main = true;
        main_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = main_layer;

        let mut sub_layer = ModernBgLayer::new(1);
        sub_layer.enabled_sub = true;
        sub_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[1] = sub_layer;
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 32,
            screen_y: 32,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });

        let mut fallback_frame = frame.clone();
        fallback_frame.cgram_rgba[33] = [40, 0, 0, 0xff];

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
                    palette_row: 2,
                },
                rect: [0, 0, 8, 8],
                sha1: "stable".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [10, 0, 0, 0xff],
                    [20, 0, 0, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };

        let (rgba, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba_with_live_index_base(
            &frame,
            &bg_cells,
            &sprite_cells,
            &fallback_frame,
            &bg_cells,
            &sprite_cells,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(&rgba[0..4], &[107, 0, 0, 0xff]);
        assert_eq!(stats.mixed_overlay_bg_effect_draws, 1);
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_subscreen,
            0
        );
    }

    #[test]
    fn modern_gpu_variant_headless_counts_prefinal_overlap_color_math_reject() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut stable_indices = [0u8; 64];
        stable_indices[0] = 1;
        let mut overlap_indices = [0u8; 64];
        overlap_indices[0] = 1;
        let bg_cells = vec![
            ModernIndexTile {
                id: 0,
                indices: stable_indices,
                source_key: modern_source_key(1, 0, 0),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: overlap_indices,
                source_key: modern_source_key(1, 9, 9),
                hflip: false,
                vflip: false,
            },
        ];

        let mut frame = ModernFrame::empty();
        frame.screen_enabled_main = 0x01;
        frame.screen_enabled_sub = 0x02;
        frame.math_enabled = 0x01;
        frame.add_subscreen = true;
        frame.cgram_rgba[33] = [80, 0, 0, 0xff];
        frame.cgram_rgba[1] = [24, 0, 0, 0xff];

        let mut main_layer = ModernBgLayer::new(0);
        main_layer.enabled_main = true;
        main_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        main_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = main_layer;

        let mut sub_layer = ModernBgLayer::new(1);
        sub_layer.enabled_sub = true;
        sub_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[1] = sub_layer;

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
                    palette_row: 2,
                },
                rect: [0, 0, 8, 8],
                sha1: "stable".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [80, 0, 0, 0xff],
                    [2, 2, 2, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };

        let (_rgba, stats) = ModernGpuVariantHeadless::new(&atlas)
            .render_rgba_with_live_index_base(
                &frame,
                &bg_cells,
                &[],
                &frame,
                &bg_cells,
                &[],
                "palette_dung_bg_main",
                "palette_main_spr",
            );

        assert_eq!(stats.mixed_overlay_bg_effect_candidates, 1);
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_subscreen,
            0
        );
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap,
            1
        );
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg,
            1
        );
        assert_eq!(
            stats
                .mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front,
            1
        );
        assert_eq!(
            stats
                .mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect,
            1
        );
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj,
            0
        );
    }

    #[test]
    fn modern_gpu_variant_headless_applies_prefinal_bg_over_behind_overlap() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut target_indices = [0u8; 64];
        target_indices[0] = 1;
        let mut behind_indices = [0u8; 64];
        behind_indices[0] = 1;
        let bg_cells = vec![
            ModernIndexTile {
                id: 0,
                indices: target_indices,
                source_key: modern_source_key(1, 0, 0),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: behind_indices,
                source_key: modern_source_key(1, 9, 9),
                hflip: false,
                vflip: false,
            },
        ];

        let mut frame = ModernFrame::empty();
        frame.screen_enabled_main = 0x01;
        frame.screen_enabled_sub = 0x02;
        frame.math_enabled = 0x01;
        frame.add_subscreen = true;
        frame.cgram_rgba[33] = [80, 0, 0, 0xff];
        frame.cgram_rgba[1] = [24, 0, 0, 0xff];

        let mut main_layer = ModernBgLayer::new(0);
        main_layer.enabled_main = true;
        main_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        main_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = main_layer;

        let mut sub_layer = ModernBgLayer::new(1);
        sub_layer.enabled_sub = true;
        sub_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[1] = sub_layer;

        let mut fallback_frame = frame.clone();
        fallback_frame.cgram_rgba[33] = [40, 0, 0, 0xff];

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
                    palette_row: 2,
                },
                rect: [0, 0, 8, 8],
                sha1: "stable".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [80, 0, 0, 0xff],
                    [2, 2, 2, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };

        let (rgba, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba_with_live_index_base(
            &frame,
            &bg_cells,
            &[],
            &fallback_frame,
            &bg_cells,
            &[],
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(&rgba[0..4], &[107, 0, 0, 0xff]);
        assert_eq!(stats.mixed_overlay_bg_effect_draws, 1);
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_subscreen,
            0
        );
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap,
            0
        );
        assert_eq!(stats.gpu_screen_builder_frames, 1);
        assert_eq!(stats.cpu_prefinal_composite_frames, 0);
        assert_eq!(stats.cpu_prefinal_overlay_frames, 0);
    }

    #[test]
    fn modern_gpu_variant_headless_applies_ordered_prefinal_bg_group() {
        use crate::modern_frame::{
            ModernBgLayer, ModernIndexSpriteInstance, ModernIndexTileInstance,
        };
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut behind_indices = [0u8; 64];
        behind_indices[0] = 1;
        let mut front_indices = [0u8; 64];
        front_indices[0] = 2;
        let mut sub_indices = [0u8; 64];
        sub_indices[0] = 1;
        let bg_cells = vec![
            ModernIndexTile {
                id: 0,
                indices: behind_indices,
                source_key: modern_source_key(1, 0, 0),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: front_indices,
                source_key: modern_source_key(1, 0, 1),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 2,
                indices: sub_indices,
                source_key: modern_source_key(1, 9, 9),
                hflip: false,
                vflip: false,
            },
        ];
        let mut sprite_indices = [0u8; 64];
        sprite_indices[0] = 1;
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices: sprite_indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.screen_enabled_main = 0x11;
        frame.screen_enabled_sub = 0x02;
        frame.math_enabled = 0x01;
        frame.add_subscreen = true;
        frame.cgram_rgba[33] = [40, 0, 0, 0xff];
        frame.cgram_rgba[34] = [80, 0, 0, 0xff];
        frame.cgram_rgba[1] = [24, 0, 0, 0xff];
        frame.cgram_rgba[0x81] = [0, 80, 0, 0xff];

        let mut main_layer = ModernBgLayer::new(0);
        main_layer.enabled_main = true;
        main_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        main_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = main_layer;

        let mut sub_layer = ModernBgLayer::new(1);
        sub_layer.enabled_sub = true;
        sub_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 2,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[1] = sub_layer;
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 32,
            screen_y: 32,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });

        let mut fallback_frame = frame.clone();
        fallback_frame.cgram_rgba[33] = [8, 0, 0, 0xff];
        fallback_frame.cgram_rgba[34] = [16, 0, 0, 0xff];

        let atlas = ModernVariantAtlas {
            width: 16,
            height: 8,
            rgba: vec![0u8; 16 * 8 * 4],
            entries: vec![
                VariantAtlasEntry {
                    id: "bg:kBgGfx:pack0:tile0:3bpp".to_string(),
                    key: VariantAtlasKey {
                        source_kind: "bg".to_string(),
                        asset: "kBgGfx".to_string(),
                        pack: 0,
                        tile: 0,
                        bpp: 3,
                        palette: "palette_dung_bg_main".to_string(),
                        palette_row: 2,
                    },
                    rect: [0, 0, 8, 8],
                    sha1: "behind".to_string(),
                    duplicate_of: None,
                    dynamic_policy: "stable".to_string(),
                    runtime_material: Some("palette_lut".to_string()),
                    runtime_colors_per_row: None,
                    source_hflip: false,
                    source_vflip: false,
                },
                VariantAtlasEntry {
                    id: "bg:kBgGfx:pack0:tile1:3bpp".to_string(),
                    key: VariantAtlasKey {
                        source_kind: "bg".to_string(),
                        asset: "kBgGfx".to_string(),
                        pack: 0,
                        tile: 1,
                        bpp: 3,
                        palette: "palette_dung_bg_main".to_string(),
                        palette_row: 2,
                    },
                    rect: [8, 0, 8, 8],
                    sha1: "front".to_string(),
                    duplicate_of: None,
                    dynamic_policy: "stable".to_string(),
                    runtime_material: Some("palette_lut".to_string()),
                    runtime_colors_per_row: None,
                    source_hflip: false,
                    source_vflip: false,
                },
            ],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [40, 0, 0, 0xff],
                    [80, 0, 0, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };

        let (rgba, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba_with_live_index_base(
            &frame,
            &bg_cells,
            &sprite_cells,
            &fallback_frame,
            &bg_cells,
            &sprite_cells,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(&rgba[0..4], &[107, 0, 0, 0xff]);
        assert_eq!(stats.mixed_overlay_bg_effect_draws, 2);
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap,
            0
        );
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg,
            0
        );
        assert_eq!(stats.gpu_screen_builder_frames, 1);
        assert_eq!(stats.cpu_prefinal_composite_frames, 0);
        assert_eq!(stats.cpu_prefinal_overlay_frames, 0);
    }

    #[test]
    fn modern_gpu_variant_headless_ignores_scanline_disabled_front_bg_for_prefinal_group() {
        use crate::modern_frame::{
            ModernBgLayer, ModernIndexSpriteInstance, ModernIndexTileInstance,
        };
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut target_indices = [0u8; 64];
        target_indices[0] = 2;
        let mut front_indices = [0u8; 64];
        front_indices[0] = 3;
        let mut sub_indices = [0u8; 64];
        sub_indices[0] = 1;
        let bg_cells = vec![
            ModernIndexTile {
                id: 0,
                indices: target_indices,
                source_key: modern_source_key(1, 0, 0),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: front_indices,
                source_key: modern_source_key(1, 0, 1),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 2,
                indices: sub_indices,
                source_key: modern_source_key(1, 9, 9),
                hflip: false,
                vflip: false,
            },
        ];
        let mut sprite_indices = [0u8; 64];
        sprite_indices[0] = 1;
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices: sprite_indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.screen_enabled_main = 0x13;
        frame.screen_enabled_sub = 0x04;
        frame.math_enabled = 0x02;
        frame.add_subscreen = true;
        frame.cgram_rgba[34] = [80, 0, 0, 0xff];
        frame.cgram_rgba[35] = [120, 0, 0, 0xff];
        frame.cgram_rgba[1] = [24, 0, 0, 0xff];
        frame.cgram_rgba[0x81] = [0, 80, 0, 0xff];
        frame.main_tm_scanlines[0] = 0x12;

        let mut front_layer = ModernBgLayer::new(0);
        front_layer.enabled_main = true;
        front_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = front_layer;

        let mut target_layer = ModernBgLayer::new(1);
        target_layer.enabled_main = true;
        target_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[1] = target_layer;

        let mut sub_layer = ModernBgLayer::new(2);
        sub_layer.enabled_sub = true;
        sub_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 2,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[2] = sub_layer;
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 32,
            screen_y: 32,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });

        let mut fallback_frame = frame.clone();
        fallback_frame.cgram_rgba[34] = [16, 0, 0, 0xff];
        fallback_frame.cgram_rgba[35] = [32, 0, 0, 0xff];

        let atlas = ModernVariantAtlas {
            width: 16,
            height: 8,
            rgba: vec![0u8; 16 * 8 * 4],
            entries: vec![
                VariantAtlasEntry {
                    id: "bg:kBgGfx:pack0:tile0:3bpp".to_string(),
                    key: VariantAtlasKey {
                        source_kind: "bg".to_string(),
                        asset: "kBgGfx".to_string(),
                        pack: 0,
                        tile: 0,
                        bpp: 3,
                        palette: "palette_dung_bg_main".to_string(),
                        palette_row: 2,
                    },
                    rect: [0, 0, 8, 8],
                    sha1: "behind".to_string(),
                    duplicate_of: None,
                    dynamic_policy: "stable".to_string(),
                    runtime_material: Some("palette_lut".to_string()),
                    runtime_colors_per_row: None,
                    source_hflip: false,
                    source_vflip: false,
                },
                VariantAtlasEntry {
                    id: "bg:kBgGfx:pack0:tile1:3bpp".to_string(),
                    key: VariantAtlasKey {
                        source_kind: "bg".to_string(),
                        asset: "kBgGfx".to_string(),
                        pack: 0,
                        tile: 1,
                        bpp: 3,
                        palette: "palette_dung_bg_main".to_string(),
                        palette_row: 2,
                    },
                    rect: [8, 0, 8, 8],
                    sha1: "front".to_string(),
                    duplicate_of: None,
                    dynamic_policy: "stable".to_string(),
                    runtime_material: Some("palette_lut".to_string()),
                    runtime_colors_per_row: None,
                    source_hflip: false,
                    source_vflip: false,
                },
            ],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [40, 0, 0, 0xff],
                    [80, 0, 0, 0xff],
                    [120, 0, 0, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };

        let (rgba, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba_with_live_index_base(
            &frame,
            &bg_cells,
            &sprite_cells,
            &fallback_frame,
            &bg_cells,
            &sprite_cells,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(&rgba[0..4], &[107, 0, 0, 0xff]);
        assert_eq!(stats.mixed_overlay_bg_effect_draws, 1, "{stats:?}");
        assert_eq!(stats.mixed_overlay_bg_effect_culled_invisible_main, 1);
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_scanline_main,
            0
        );
        assert_eq!(
            stats
                .mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex,
            0
        );
    }

    #[test]
    fn modern_gpu_variant_headless_counts_prefinal_deeper_bg_chain_reject() {
        use crate::modern_frame::{
            ModernBgLayer, ModernIndexSpriteInstance, ModernIndexTileInstance,
        };
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut behind_indices = [0u8; 64];
        behind_indices[0] = 1;
        let mut middle_indices = [0u8; 64];
        middle_indices[0] = 2;
        let mut front_indices = [0u8; 64];
        front_indices[0] = 3;
        let mut sub_indices = [0u8; 64];
        sub_indices[0] = 1;
        let bg_cells = vec![
            ModernIndexTile {
                id: 0,
                indices: behind_indices,
                source_key: modern_source_key(1, 0, 0),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: middle_indices,
                source_key: modern_source_key(1, 0, 1),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 2,
                indices: front_indices,
                source_key: modern_source_key(1, 0, 2),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 3,
                indices: sub_indices,
                source_key: modern_source_key(1, 9, 9),
                hflip: false,
                vflip: false,
            },
        ];
        let mut sprite_indices = [0u8; 64];
        sprite_indices[0] = 1;
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices: sprite_indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.screen_enabled_main = 0x11;
        frame.screen_enabled_sub = 0x02;
        frame.math_enabled = 0x01;
        frame.add_subscreen = true;
        frame.cgram_rgba[33] = [40, 0, 0, 0xff];
        frame.cgram_rgba[34] = [80, 0, 0, 0xff];
        frame.cgram_rgba[35] = [120, 0, 0, 0xff];
        frame.cgram_rgba[1] = [24, 0, 0, 0xff];
        frame.cgram_rgba[0x81] = [0, 80, 0, 0xff];

        let mut main_layer = ModernBgLayer::new(0);
        main_layer.enabled_main = true;
        for cell_id in 0..3 {
            main_layer.index_tiles.push(ModernIndexTileInstance {
                cell_id,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                screen_x: 0,
                screen_y: 0,
                palette: 2,
                hflip: false,
                vflip: false,
                priority: false,
            });
        }
        frame.bg_layers[0] = main_layer;

        let mut sub_layer = ModernBgLayer::new(1);
        sub_layer.enabled_sub = true;
        sub_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 3,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[1] = sub_layer;
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 32,
            screen_y: 32,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });

        let atlas = ModernVariantAtlas {
            width: 24,
            height: 8,
            rgba: vec![0u8; 24 * 8 * 4],
            entries: vec![
                VariantAtlasEntry {
                    id: "bg:kBgGfx:pack0:tile0:3bpp".to_string(),
                    key: VariantAtlasKey {
                        source_kind: "bg".to_string(),
                        asset: "kBgGfx".to_string(),
                        pack: 0,
                        tile: 0,
                        bpp: 3,
                        palette: "palette_dung_bg_main".to_string(),
                        palette_row: 2,
                    },
                    rect: [0, 0, 8, 8],
                    sha1: "behind".to_string(),
                    duplicate_of: None,
                    dynamic_policy: "stable".to_string(),
                    runtime_material: Some("palette_lut".to_string()),
                    runtime_colors_per_row: None,
                    source_hflip: false,
                    source_vflip: false,
                },
                VariantAtlasEntry {
                    id: "bg:kBgGfx:pack0:tile1:3bpp".to_string(),
                    key: VariantAtlasKey {
                        source_kind: "bg".to_string(),
                        asset: "kBgGfx".to_string(),
                        pack: 0,
                        tile: 1,
                        bpp: 3,
                        palette: "palette_dung_bg_main".to_string(),
                        palette_row: 2,
                    },
                    rect: [8, 0, 8, 8],
                    sha1: "middle".to_string(),
                    duplicate_of: None,
                    dynamic_policy: "stable".to_string(),
                    runtime_material: Some("palette_lut".to_string()),
                    runtime_colors_per_row: None,
                    source_hflip: false,
                    source_vflip: false,
                },
                VariantAtlasEntry {
                    id: "bg:kBgGfx:pack0:tile2:3bpp".to_string(),
                    key: VariantAtlasKey {
                        source_kind: "bg".to_string(),
                        asset: "kBgGfx".to_string(),
                        pack: 0,
                        tile: 2,
                        bpp: 3,
                        palette: "palette_dung_bg_main".to_string(),
                        palette_row: 2,
                    },
                    rect: [16, 0, 8, 8],
                    sha1: "front".to_string(),
                    duplicate_of: None,
                    dynamic_policy: "stable".to_string(),
                    runtime_material: Some("palette_lut".to_string()),
                    runtime_colors_per_row: None,
                    source_hflip: false,
                    source_vflip: false,
                },
            ],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [40, 0, 0, 0xff],
                    [80, 0, 0, 0xff],
                    [120, 0, 0, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };

        let (_rgba, stats) = ModernGpuVariantHeadless::new(&atlas)
            .render_rgba_with_live_index_base(
                &frame,
                &bg_cells,
                &sprite_cells,
                &frame,
                &bg_cells,
                &sprite_cells,
                "palette_dung_bg_main",
                "palette_main_spr",
            );

        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg, 1,
            "{stats:?}"
        );
        assert_eq!(
            stats
                .mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain,
            1
        );
    }

    #[test]
    fn modern_gpu_variant_headless_counts_prefinal_mixed_static_live_order_reject() {
        use crate::modern_frame::{
            ModernBgLayer, ModernIndexSpriteInstance, ModernIndexTileInstance,
        };
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut behind_live_indices = [0u8; 64];
        behind_live_indices[0] = 1;
        let mut front_static_indices = [0u8; 64];
        front_static_indices[0] = 2;
        let mut sub_indices = [0u8; 64];
        sub_indices[0] = 1;
        let bg_cells = vec![
            ModernIndexTile {
                id: 0,
                indices: behind_live_indices,
                source_key: modern_source_key(1, 0, 0),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 1,
                indices: front_static_indices,
                source_key: modern_source_key(1, 0, 1),
                hflip: false,
                vflip: false,
            },
            ModernIndexTile {
                id: 2,
                indices: sub_indices,
                source_key: modern_source_key(1, 9, 9),
                hflip: false,
                vflip: false,
            },
        ];
        let mut sprite_indices = [0u8; 64];
        sprite_indices[0] = 1;
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices: sprite_indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.screen_enabled_main = 0x11;
        frame.screen_enabled_sub = 0x02;
        frame.math_enabled = 0x01;
        frame.add_subscreen = true;
        frame.cgram_rgba[33] = [44, 0, 0, 0xff];
        frame.cgram_rgba[34] = [80, 0, 0, 0xff];
        frame.cgram_rgba[1] = [24, 0, 0, 0xff];
        frame.cgram_rgba[0x81] = [0, 80, 0, 0xff];

        let mut main_layer = ModernBgLayer::new(0);
        main_layer.enabled_main = true;
        main_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        main_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 1,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = main_layer;

        let mut sub_layer = ModernBgLayer::new(1);
        sub_layer.enabled_sub = true;
        sub_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 2,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[1] = sub_layer;
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 32,
            screen_y: 32,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });

        let atlas = ModernVariantAtlas {
            width: 16,
            height: 8,
            rgba: vec![0u8; 16 * 8 * 4],
            entries: vec![
                VariantAtlasEntry {
                    id: "bg:kBgGfx:pack0:tile0:3bpp".to_string(),
                    key: VariantAtlasKey {
                        source_kind: "bg".to_string(),
                        asset: "kBgGfx".to_string(),
                        pack: 0,
                        tile: 0,
                        bpp: 3,
                        palette: "palette_dung_bg_main".to_string(),
                        palette_row: 2,
                    },
                    rect: [0, 0, 8, 8],
                    sha1: "behind-live".to_string(),
                    duplicate_of: None,
                    dynamic_policy: "stable".to_string(),
                    runtime_material: Some("palette_lut".to_string()),
                    runtime_colors_per_row: None,
                    source_hflip: false,
                    source_vflip: false,
                },
                VariantAtlasEntry {
                    id: "bg:kBgGfx:pack0:tile1:3bpp".to_string(),
                    key: VariantAtlasKey {
                        source_kind: "bg".to_string(),
                        asset: "kBgGfx".to_string(),
                        pack: 0,
                        tile: 1,
                        bpp: 3,
                        palette: "palette_dung_bg_main".to_string(),
                        palette_row: 2,
                    },
                    rect: [8, 0, 8, 8],
                    sha1: "front-static".to_string(),
                    duplicate_of: None,
                    dynamic_policy: "stable".to_string(),
                    runtime_material: Some("palette_lut".to_string()),
                    runtime_colors_per_row: None,
                    source_hflip: false,
                    source_vflip: false,
                },
            ],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [40, 0, 0, 0xff],
                    [80, 0, 0, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };

        let (_rgba, stats) = ModernGpuVariantHeadless::new(&atlas)
            .render_rgba_with_live_index_base(
                &frame,
                &bg_cells,
                &sprite_cells,
                &frame,
                &bg_cells,
                &sprite_cells,
                "palette_dung_bg_main",
                "palette_main_spr",
            );

        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg,
            1
        );
        assert_eq!(
            stats
                .mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order,
            1
        );
    }

    #[test]
    fn bg_packet_prefinal_material_classifies_cgram_mismatch() {
        use crate::modern_frame::ModernIndexTileInstance;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            TileEffect, VariantAtlasDraw, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut indices = [0u8; 64];
        indices[0] = 1;
        let cell = ModernIndexTile {
            id: 0,
            indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        };
        let inst = ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 16,
            hflip: false,
            vflip: false,
            priority: false,
        };
        let entry = VariantAtlasEntry {
            id: "bg:kBgGfx:pack0:tile0:3bpp".to_string(),
            key: VariantAtlasKey {
                source_kind: "bg".to_string(),
                asset: "kBgGfx".to_string(),
                pack: 0,
                tile: 0,
                bpp: 3,
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
            },
            rect: [0, 0, 8, 8],
            sha1: "stable-effect".to_string(),
            duplicate_of: None,
            dynamic_policy: "stable".to_string(),
            runtime_material: Some("palette_lut".to_string()),
            runtime_colors_per_row: None,
            source_hflip: false,
            source_vflip: false,
        };
        let effect = TileEffect {
            id: "palette_dung_bg_main:8color:row2".to_string(),
            palette: "palette_dung_bg_main".to_string(),
            palette_row: 2,
            colors_per_row: 8,
            index_to_rgba: vec![
                [0, 0, 0, 0xff],
                [40, 0, 0, 0xff],
                [80, 0, 0, 0xff],
                [3, 3, 3, 0xff],
                [4, 4, 4, 0xff],
                [5, 5, 5, 0xff],
                [6, 6, 6, 0xff],
                [7, 7, 7, 0xff],
            ],
            dynamic_policy: "stable".to_string(),
        };
        let packet = crate::modern_variant_draw::VariantBgDrawPacket {
            layer_index: 0,
            cell: &cell,
            inst: &inst,
            key: None,
            draw: VariantAtlasDraw::MaterialEffect {
                entry: &entry,
                effect: &effect,
            },
        };

        let mut frame = ModernFrame::empty();
        frame.cgram_rgba[1] = [40, 0, 0, 0xff];

        assert_eq!(
            bg_packet_prefinal_material(&frame, &packet),
            Err(PrefinalBgMaterialRejectReason::CgramMismatch)
        );
    }

    #[test]
    fn modern_gpu_variant_headless_restores_front_obj_over_prefinal_bg() {
        use crate::modern_frame::{
            ModernBgLayer, ModernIndexSpriteInstance, ModernIndexTileInstance,
        };
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut stable_indices = [0u8; 64];
        stable_indices[0] = 1;
        let bg_cells = vec![ModernIndexTile {
            id: 0,
            indices: stable_indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];
        let mut sprite_indices = [0u8; 64];
        sprite_indices[0] = 1;
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices: sprite_indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.screen_enabled_main = 0x11;
        frame.screen_enabled_sub = 0x02;
        frame.math_enabled = 0x01;
        frame.add_subscreen = true;
        frame.cgram_rgba[33] = [80, 0, 0, 0xff];
        frame.cgram_rgba[1] = [24, 0, 0, 0xff];
        frame.cgram_rgba[0x81] = [0, 80, 0, 0xff];

        let mut main_layer = ModernBgLayer::new(0);
        main_layer.enabled_main = true;
        main_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = main_layer;

        let mut sub_layer = ModernBgLayer::new(1);
        sub_layer.enabled_sub = true;
        sub_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[1] = sub_layer;
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            priority: 3,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });

        let mut fallback_frame = frame.clone();
        fallback_frame.cgram_rgba[33] = [40, 0, 0, 0xff];
        fallback_frame.cgram_rgba[0x81] = [0, 40, 0, 0xff];

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
                    palette_row: 2,
                },
                rect: [0, 0, 8, 8],
                sha1: "stable".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [80, 0, 0, 0xff],
                    [2, 2, 2, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };

        let (rgba, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba_with_live_index_base(
            &frame,
            &bg_cells,
            &sprite_cells,
            &fallback_frame,
            &bg_cells,
            &sprite_cells,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(&rgba[0..4], &[0, 82, 0, 0xff]);
        assert_eq!(stats.mixed_overlay_bg_effect_draws, 1, "{stats:?}");
        assert_eq!(stats.mixed_overlay_bg_effect_candidates, 1);
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_subscreen,
            0
        );
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap,
            0
        );
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg,
            0
        );
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj,
            0
        );
        assert_eq!(stats.gpu_screen_builder_frames, 1);
        assert_eq!(stats.cpu_prefinal_composite_frames, 0);
        assert_eq!(stats.cpu_prefinal_overlay_frames, 0);
    }

    #[test]
    fn modern_gpu_variant_headless_allows_prefinal_bg_over_behind_obj_overlap() {
        use crate::modern_frame::{
            ModernBgLayer, ModernIndexSpriteInstance, ModernIndexTileInstance,
        };
        use crate::modern_hd_overrides::NO_SOURCE_KEY;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        let mut stable_indices = [0u8; 64];
        stable_indices[0] = 1;
        let bg_cells = vec![ModernIndexTile {
            id: 0,
            indices: stable_indices,
            source_key: modern_source_key(1, 0, 0),
            hflip: false,
            vflip: false,
        }];
        let mut sprite_indices = [0u8; 64];
        sprite_indices[0] = 1;
        let sprite_cells = vec![ModernIndexTile {
            id: 0,
            indices: sprite_indices,
            source_key: NO_SOURCE_KEY,
            hflip: false,
            vflip: false,
        }];

        let mut frame = ModernFrame::empty();
        frame.screen_enabled_main = 0x11;
        frame.screen_enabled_sub = 0x02;
        frame.math_enabled = 0x01;
        frame.add_subscreen = true;
        frame.cgram_rgba[33] = [80, 0, 0, 0xff];
        frame.cgram_rgba[1] = [24, 0, 0, 0xff];
        frame.cgram_rgba[0x81] = [0, 80, 0, 0xff];

        let mut main_layer = ModernBgLayer::new(0);
        main_layer.enabled_main = true;
        main_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 2,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[0] = main_layer;

        let mut sub_layer = ModernBgLayer::new(1);
        sub_layer.enabled_sub = true;
        sub_layer.index_tiles.push(ModernIndexTileInstance {
            cell_id: 0,
            source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            hflip: false,
            vflip: false,
            priority: false,
        });
        frame.bg_layers[1] = sub_layer;
        frame.index_sprites.push(ModernIndexSpriteInstance {
            cell_id: 0,
            screen_x: 0,
            screen_y: 0,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: false,
            row_mask: 0xff,
        });

        let mut fallback_frame = frame.clone();
        fallback_frame.cgram_rgba[33] = [40, 0, 0, 0xff];

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
                    palette_row: 2,
                },
                rect: [0, 0, 8, 8],
                sha1: "stable".to_string(),
                duplicate_of: None,
                dynamic_policy: "stable".to_string(),
                runtime_material: Some("palette_lut".to_string()),
                runtime_colors_per_row: None,
                source_hflip: false,
                source_vflip: false,
            }],
            effects: vec![TileEffect {
                id: "palette_dung_bg_main:8color:row2".to_string(),
                palette: "palette_dung_bg_main".to_string(),
                palette_row: 2,
                colors_per_row: 8,
                index_to_rgba: vec![
                    [0, 0, 0, 0xff],
                    [80, 0, 0, 0xff],
                    [2, 2, 2, 0xff],
                    [3, 3, 3, 0xff],
                    [4, 4, 4, 0xff],
                    [5, 5, 5, 0xff],
                    [6, 6, 6, 0xff],
                    [7, 7, 7, 0xff],
                ],
                dynamic_policy: "stable".to_string(),
            }],
        };

        let (rgba, stats) = ModernGpuVariantHeadless::new(&atlas).render_rgba_with_live_index_base(
            &frame,
            &bg_cells,
            &sprite_cells,
            &fallback_frame,
            &bg_cells,
            &sprite_cells,
            "palette_dung_bg_main",
            "palette_main_spr",
        );

        assert_eq!(&rgba[0..4], &[107, 0, 0, 0xff]);
        assert_eq!(stats.mixed_overlay_bg_effect_draws, 1, "{stats:?}");
        assert_eq!(
            stats.mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj,
            0
        );
        assert_eq!(stats.gpu_screen_builder_frames, 1);
        assert_eq!(stats.cpu_prefinal_composite_frames, 0);
        assert_eq!(stats.cpu_prefinal_overlay_frames, 0);
    }

    #[test]
    fn modern_gpu_mode7_matches_modern_software_oracle() {
        let mut vram = vec![0u16; 0x8000];
        vram[0] = 0x0002; // tilemap entry (0,0): low byte = tile number 2
        vram[2 * 64 + 8] = 5u16 << 8; // tile 2, texel row1 col0: high byte = index 5
        let mut cgram = vec![0u16; 0x100];
        cgram[5] = 0x7c1f; // BGR555 magenta
        let oam = vec![0u16; 0x110];
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.mode = 7;
        frame.screen_enabled = [0x01, 0];
        for sl in frame.scanlines.iter_mut() {
            sl.mode7_matrix = [256, 0, 0, 256, 0, 0, 0, 0];
            sl.screen_enabled_main = 0x01;
        }

        let gpu = ModernGpuHeadless::new().render_mode7_rgba(&frame);
        let software = crate::modern_software::render_modern_mode7_frame(&frame);

        assert_eq!(gpu.len(), software.len());
        assert_eq!(gpu, software);
    }

    #[test]
    fn modern_index_compare_frame_selects_cpu_mode7_without_gpu_helpers() {
        let vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.mode = 7;

        let render = render_modern_index_compare_frame(
            &frame,
            None::<&dyn crate::modern_extract::SourceTableView>,
            None,
            None,
            None,
            crate::ModernAssetFrameScene::from_in_dungeon(true),
            None,
            false,
        );

        assert_eq!(render.via, "mode7-cpu");
        assert_eq!(render.rgba.len(), 256 * 224 * 4);
        assert!(render.variant_stats.is_none());
        assert!(render.variant_traces.is_empty());
    }

    #[test]
    fn modern_index_compare_frame_preserves_source_cpu_fallback_toggle() {
        let vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.mode = 1;
        let atlas = crate::modern_source_atlas::ModernSourceAtlas::from_keyed_cells_for_test(
            Vec::new(),
            &[],
        );
        let table = |_: usize| (0, 0, 0);

        let source_render = render_modern_index_compare_frame(
            &frame,
            Some(&table),
            Some(&atlas),
            None,
            None,
            crate::ModernAssetFrameScene::from_in_dungeon(true),
            None,
            true,
        );
        let vram_render = render_modern_index_compare_frame(
            &frame,
            Some(&table),
            Some(&atlas),
            None,
            None,
            crate::ModernAssetFrameScene::from_in_dungeon(true),
            None,
            false,
        );

        assert_eq!(source_render.via, "sources");
        assert_eq!(vram_render.via, "vram");
        assert_eq!(source_render.rgba.len(), 256 * 224 * 4);
        assert_eq!(vram_render.rgba.len(), 256 * 224 * 4);
    }

    #[test]
    fn modern_atlas_compare_frame_owns_legacy_extract_and_software_render() {
        let atlas = ModernTileAtlasAsset {
            tile_width_px: 8,
            tile_height_px: 8,
            atlas_scale: 1,
            width_px: 8,
            height_px: 8,
            rgba: vec![0x44; 8 * 8 * 4],
            entries: vec![crate::modern_assets::ModernTileAtlasEntry {
                id: 0,
                atlas_x_px: 0,
                atlas_y_px: 0,
                atlas_width_px: 8,
                atlas_height_px: 8,
                tilemap_entry: 0x0001,
                tilemap_variants: vec![0x0001],
            }],
        };
        let mut vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        vram[0] = 0x0001;
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.bg[0].tilemap_adr = 0;
        frame.screen_enabled = [0x01, 0x00];

        let render = render_modern_atlas_compare_frame(&frame, &atlas);
        let modern = crate::modern_extract::extract_modern_frame_with_atlas(&frame, &atlas);
        let manual = crate::modern_software::render_modern_frame_software(
            &modern,
            &atlas.rgba,
            atlas.width_px as u16,
            atlas.height_px as u16,
        );

        assert_eq!(render.via, "atlas-software");
        assert_eq!(render.rgba, manual);
        assert_eq!(
            render.hash,
            crate::frame_compare::render_frame_rgb_hash_rgba(&manual)
        );
    }

    #[test]
    fn modern_atlas_compare_owns_hash_and_match_assembly() {
        let atlas = ModernTileAtlasAsset {
            tile_width_px: 8,
            tile_height_px: 8,
            atlas_scale: 1,
            width_px: 8,
            height_px: 8,
            rgba: vec![0x44; 8 * 8 * 4],
            entries: vec![crate::modern_assets::ModernTileAtlasEntry {
                id: 0,
                atlas_x_px: 0,
                atlas_y_px: 0,
                atlas_width_px: 8,
                atlas_height_px: 8,
                tilemap_entry: 0x0001,
                tilemap_variants: vec![0x0001],
            }],
        };
        let mut vram = vec![0u16; 0x8000];
        let cgram = vec![0u16; 0x100];
        let oam = vec![0u16; 0x110];
        vram[0] = 0x0001;
        let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
        frame.bg[0].tilemap_adr = 0;
        frame.screen_enabled = [0x01, 0x00];

        let expected = render_modern_atlas_compare_frame(&frame, &atlas);
        let compare = compare_modern_atlas_to_rgba(&expected.rgba, &frame, &atlas);

        assert_eq!(
            compare.classic_hash,
            crate::frame_compare::render_frame_rgb_hash_rgba(&expected.rgba)
        );
        assert_eq!(compare.modern_hash, compare.render.hash);
        assert_eq!(compare.modern_hash, expected.hash);
        assert!(compare.matches);
    }

    #[test]
    fn modern_gpu_path_fallback_reason_accepts_gpu_routes() {
        assert_eq!(modern_gpu_path_fallback_reason("mode7-gpu", None), None);

        let stats = VariantAtlasRenderStats {
            gpu_prefinal_base_frames: 1,
            gpu_screen_builder_frames: 1,
            ..Default::default()
        };

        assert_eq!(
            modern_gpu_path_fallback_reason("variant-gpu", Some(&stats)),
            None
        );
    }

    #[test]
    fn modern_gpu_path_fallback_reason_reports_first_cpu_route_or_blocker() {
        assert_eq!(
            modern_gpu_path_fallback_reason("mode7-cpu", None),
            Some(ModernGpuPathFallback {
                reason: "mode7-cpu",
                count: 1,
            })
        );
        assert_eq!(
            modern_gpu_path_fallback_reason("sources", None),
            Some(ModernGpuPathFallback {
                reason: "sources-cpu",
                count: 1,
            })
        );
        assert_eq!(
            modern_gpu_path_fallback_reason("vram", None),
            Some(ModernGpuPathFallback {
                reason: "vram-cpu",
                count: 1,
            })
        );

        let stats = VariantAtlasRenderStats {
            cpu_prefinal_overlay_frames: 2,
            cpu_prefinal_composite_frames: 1,
            ..Default::default()
        };

        assert_eq!(
            modern_gpu_path_fallback_reason("variant-gpu", Some(&stats)),
            Some(ModernGpuPathFallback {
                reason: "prefinal-overlay-cpu",
                count: 2,
            })
        );
    }

    fn test_gpu_frame<'a>(
        vram: &'a [u16],
        cgram: &'a [u16],
        oam: &'a [u16],
        brightness: u8,
        forced_blank: bool,
    ) -> crate::gpu_frame::GpuFrame<'a> {
        crate::gpu_frame::GpuFrame {
            vram,
            cgram,
            oam,
            mode: 1,
            bg: Default::default(),
            obj: Default::default(),
            mosaic_enabled: 0,
            mosaic_size: 0,
            extra_left_right: 0,
            mode7: Default::default(),
            screen_enabled: [0, 0],
            screen_windowed: [0, 0],
            brightness,
            forced_blank,
            math_enabled: 0,
            subtract_color: false,
            half_color: false,
            fixed_color_r: 0,
            fixed_color_g: 0,
            fixed_color_b: 0,
            add_subscreen: false,
            clip_mode: 0,
            prevent_math_mode: 0,
            windowsel_cm: 0,
            windowsel: 0,
            scanlines: Box::new([crate::gpu_frame::ScanlineRegs::default(); 224]),
        }
    }

    /// Render the BG index pass then the sprite pass on the GPU, reading back the
    /// composited 256x224 RGBA target.
    async fn gpu_bg_then_sprites(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
    ) -> Vec<u8> {
        let bg = ModernGpuIndexRenderer::new(device, queue, wgpu::TextureFormat::Rgba8Unorm);
        let spr = ModernGpuSpriteRenderer::new(device, queue, wgpu::TextureFormat::Rgba8Unorm);

        let width = 256u32;
        let height = 224u32;
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("modern_gpu_sprite_test_target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());

        bg.render(device, queue, bg_cells, frame, &view);
        spr.render(device, queue, sprite_cells, frame, &view);

        let bytes_per_row = width * 4;
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_gpu_sprite_test_readback"),
            size: (bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit([encoder.finish()]);

        let slice = readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("GPU poll failed during readback");
        let mapped = slice.get_mapped_range();
        let gpu_rgba = mapped.to_vec();
        drop(mapped);
        readback.unmap();
        gpu_rgba
    }

    #[test]
    fn modern_gpu_variant_atlas_bg_tile_matches_software_variant() {
        use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_software::render_modern_frame_software_variant_atlas;
        use crate::modern_source_atlas::modern_source_key;
        use crate::modern_variant_atlas::{
            ModernVariantAtlas, TileEffect, VariantAtlasEntry, VariantAtlasKey,
        };

        pollster::block_on(async {
            let instance = crate::create_wgpu_instance();
            let (_adapter, device, queue) = crate::create_device_queue(&instance, None).await;

            let mut atlas_rgba = vec![0u8; 8 * 8 * 4];
            for px in atlas_rgba.chunks_exact_mut(4) {
                px.copy_from_slice(&[180, 20, 40, 0xff]);
            }
            let atlas = ModernVariantAtlas {
                width: 8,
                height: 8,
                rgba: atlas_rgba,
                entries: vec![
                    VariantAtlasEntry {
                        id: "bg:kBgGfx:pack0:tile0:3bpp:palette_dung_bg_main:row2".to_string(),
                        key: VariantAtlasKey {
                            source_kind: "bg".to_string(),
                            asset: "kBgGfx".to_string(),
                            pack: 0,
                            tile: 0,
                            bpp: 3,
                            palette: "palette_dung_bg_main".to_string(),
                            palette_row: 2,
                        },
                        rect: [0, 0, 8, 8],
                        sha1: "test".to_string(),
                        duplicate_of: None,
                        dynamic_policy: "stable".to_string(),
                        runtime_material: Some("palette_lut".to_string()),
                        runtime_colors_per_row: None,
                        source_hflip: false,
                        source_vflip: false,
                    },
                    VariantAtlasEntry {
                        id: "sprite:kSprGfx:pack0:tile0:3bpp:palette_main_spr:row4".to_string(),
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
                        sha1: "test-sprite".to_string(),
                        duplicate_of: None,
                        dynamic_policy: "stable".to_string(),
                        runtime_material: Some("palette_lut".to_string()),
                        runtime_colors_per_row: None,
                        source_hflip: false,
                        source_vflip: false,
                    },
                ],
                effects: vec![
                    TileEffect {
                        id: "palette_dung_bg_main:8color:row2".to_string(),
                        palette: "palette_dung_bg_main".to_string(),
                        palette_row: 2,
                        colors_per_row: 8,
                        index_to_rgba: vec![
                            [0, 0, 0, 0xff],
                            [90, 100, 110, 0xff],
                            [2, 2, 2, 0xff],
                            [3, 3, 3, 0xff],
                            [4, 4, 4, 0xff],
                            [5, 5, 5, 0xff],
                            [6, 6, 6, 0xff],
                            [7, 7, 7, 0xff],
                        ],
                        dynamic_policy: "stable".to_string(),
                    },
                    TileEffect {
                        id: "palette_main_spr:8color:row4".to_string(),
                        palette: "palette_main_spr".to_string(),
                        palette_row: 4,
                        colors_per_row: 8,
                        index_to_rgba: vec![
                            [0, 0, 0, 0xff],
                            [1, 1, 1, 0xff],
                            [2, 2, 2, 0xff],
                            [120, 130, 140, 0xff],
                            [4, 4, 4, 0xff],
                            [5, 5, 5, 0xff],
                            [6, 6, 6, 0xff],
                            [7, 7, 7, 0xff],
                        ],
                        dynamic_policy: "stable".to_string(),
                    },
                ],
            };

            let cells = vec![ModernIndexTile {
                id: 0,
                indices: [1u8; 64],
                source_key: modern_source_key(1, 0, 0),
                hflip: false,
                vflip: false,
            }];
            let mut sprite_indices = [0u8; 64];
            sprite_indices[0] = 3;
            let sprite_cells = vec![ModernIndexTile {
                id: 0,
                indices: sprite_indices,
                source_key: modern_source_key(2, 0, 0),
                hflip: false,
                vflip: false,
            }];
            let mut frame = ModernFrame::empty();
            frame.backdrop_color_rgba = [0, 0, 0, 0xff];
            let mut layer = ModernBgLayer::new(0);
            layer.enabled_main = true;
            layer.index_tiles.push(ModernIndexTileInstance {
                cell_id: 0,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                screen_x: 3,
                screen_y: 5,
                palette: 2,
                hflip: false,
                vflip: false,
                priority: false,
            });
            frame.bg_layers[0] = layer;
            frame
                .index_sprites
                .push(crate::modern_frame::ModernIndexSpriteInstance {
                    cell_id: 0,
                    screen_x: 0,
                    screen_y: 0,
                    palette: 4,
                    hflip: false,
                    vflip: false,
                    priority: 0,
                    row_mask: 0xff,
                });

            let renderer = ModernGpuVariantRenderer::new(
                &device,
                &queue,
                &atlas,
                wgpu::TextureFormat::Rgba8Unorm,
            );
            let width = 256u32;
            let height = 224u32;
            let target = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("modern_gpu_variant_test_target"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let view = target.create_view(&wgpu::TextureViewDescriptor::default());

            let stats = renderer.render(
                &device,
                &queue,
                &frame,
                &cells,
                &sprite_cells,
                "palette_dung_bg_main",
                "palette_main_spr",
                &view,
            );

            let bytes_per_row = width * 4;
            let readback = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("modern_gpu_variant_test_readback"),
                size: (bytes_per_row * height) as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &target,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &readback,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(bytes_per_row),
                        rows_per_image: None,
                    },
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            queue.submit([encoder.finish()]);

            let slice = readback.slice(..);
            slice.map_async(wgpu::MapMode::Read, |_| {});
            device
                .poll(wgpu::PollType::wait_indefinitely())
                .expect("GPU poll failed during readback");
            let mapped = slice.get_mapped_range();
            let gpu_rgba = mapped.to_vec();
            drop(mapped);
            readback.unmap();

            let (software_rgba, software_stats) = render_modern_frame_software_variant_atlas(
                &frame,
                &cells,
                &sprite_cells,
                &atlas,
                "palette_dung_bg_main",
                "palette_main_spr",
            );

            assert_eq!(stats.stable_draws, 0);
            assert_eq!(stats.effect_draws, 2);
            assert_eq!(stats.dynamic_material_draws, 2);
            assert_eq!(stats.fallback_draws, 0);
            assert_eq!(stats, software_stats);
            if gpu_rgba != software_rgba {
                let mismatch = gpu_rgba
                    .chunks_exact(4)
                    .zip(software_rgba.chunks_exact(4))
                    .enumerate()
                    .find(|(_, (gpu, software))| gpu != software)
                    .expect("framebuffers differ but no pixel mismatch was found");
                let pixel = mismatch.0;
                panic!(
                    "first GPU/software variant mismatch at ({}, {}): gpu={:?} software={:?}",
                    pixel % width as usize,
                    pixel / width as usize,
                    mismatch.1 .0,
                    mismatch.1 .1
                );
            }
        });
    }

    #[test]
    fn modern_gpu_sprite_matches_software() {
        use crate::modern_frame::ModernIndexSpriteInstance;
        use crate::modern_index_atlas::ModernIndexTile;
        use crate::modern_software::{
            draw_modern_sprites_indexed, render_modern_frame_software_indexed,
        };

        pollster::block_on(async {
            let instance = crate::create_wgpu_instance();
            let (_adapter, device, queue) = crate::create_device_queue(&instance, None).await;

            // ── Case 1: no flip ────────────────────────────────────────────────
            let mut indices = [0u8; 64];
            indices[0] = 1; // pixel (0,0) → index 1
            let sprite_cells = vec![ModernIndexTile {
                id: 0,
                indices,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                hflip: false,
                vflip: false,
            }];

            let mut frame = ModernFrame::empty();
            frame.backdrop_color_rgba = [0, 0, 0, 0xff];
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

            let gpu_rgba = gpu_bg_then_sprites(&device, &queue, &frame, &[], &sprite_cells).await;
            let mut software_rgba = render_modern_frame_software_indexed(&frame, &[]);
            draw_modern_sprites_indexed(&mut software_rgba, &frame, &sprite_cells);
            assert_eq!(gpu_rgba.len(), software_rgba.len());
            assert_eq!(gpu_rgba, software_rgba, "no-flip sprite gpu==software");

            // ── Case 2: hflip ──────────────────────────────────────────────────
            let mut indices2 = [0u8; 64];
            indices2[7] = 2; // pixel (7,0) → index 2; hflip lands it at screen x=0
            let sprite_cells2 = vec![ModernIndexTile {
                id: 0,
                indices: indices2,
                source_key: crate::modern_hd_overrides::NO_SOURCE_KEY,
                hflip: false,
                vflip: false,
            }];

            let mut frame2 = ModernFrame::empty();
            frame2.backdrop_color_rgba = [0, 0, 0, 0xff];
            frame2.cgram_rgba[0x80 + 3 * 16 + 2] = [9, 99, 199, 0xff];
            frame2.index_sprites.push(ModernIndexSpriteInstance {
                cell_id: 0,
                screen_x: 5,
                screen_y: 7,
                palette: 3,
                priority: 0,
                hflip: true,
                vflip: false,
                row_mask: 0xff,
            });

            let gpu_rgba2 =
                gpu_bg_then_sprites(&device, &queue, &frame2, &[], &sprite_cells2).await;
            let mut software_rgba2 = render_modern_frame_software_indexed(&frame2, &[]);
            draw_modern_sprites_indexed(&mut software_rgba2, &frame2, &sprite_cells2);
            // The flipped pixel must land at screen_x (5,7).
            let px = (7usize * 256 + 5) * 4;
            assert_eq!(
                &software_rgba2[px..px + 4],
                &[9, 99, 199, 0xff],
                "hflip software pixel at x=5"
            );
            assert_eq!(gpu_rgba2, software_rgba2, "hflip sprite gpu==software");
        });
    }
}
