use crate::modern_frame::ModernFrame;
use crate::modern_index_atlas::ModernIndexTile;
use crate::modern_software::VariantAtlasRenderStats;
use crate::modern_variant_draw::VariantDrawPlan;

pub(crate) struct PreparedModernVariantRender<'a> {
    frame: &'a ModernFrame,
    bg_cells: &'a [ModernIndexTile],
    sprite_cells: &'a [ModernIndexTile],
    plan: VariantDrawPlan<'a>,
    variant_frame: ModernFrame,
    stats: VariantAtlasRenderStats,
    live_render_path: ModernVariantRenderPath,
    headless_render_path: ModernVariantRenderPath,
}

impl<'a> PreparedModernVariantRender<'a> {
    pub(crate) fn new(
        frame: &'a ModernFrame,
        bg_cells: &'a [ModernIndexTile],
        sprite_cells: &'a [ModernIndexTile],
        plan: VariantDrawPlan<'a>,
        variant_frame: ModernFrame,
    ) -> Self {
        let stats = plan.stats;
        Self {
            frame,
            bg_cells,
            sprite_cells,
            plan,
            variant_frame,
            stats,
            live_render_path: live_variant_render_path(&stats),
            headless_render_path: headless_variant_render_path(&stats),
        }
    }

    pub(crate) fn frame(&self) -> &'a ModernFrame {
        self.frame
    }

    pub(crate) fn bg_cells(&self) -> &'a [ModernIndexTile] {
        self.bg_cells
    }

    pub(crate) fn sprite_cells(&self) -> &'a [ModernIndexTile] {
        self.sprite_cells
    }

    pub(crate) fn plan(&self) -> &VariantDrawPlan<'a> {
        &self.plan
    }

    pub(crate) fn variant_frame(&self) -> &ModernFrame {
        &self.variant_frame
    }

    pub(crate) fn initial_stats(&self) -> VariantAtlasRenderStats {
        self.stats
    }

    pub(crate) fn render_path(
        &self,
        output: PreparedModernVariantOutput,
    ) -> ModernVariantRenderPath {
        match output {
            PreparedModernVariantOutput::Live => self.live_render_path,
            PreparedModernVariantOutput::Headless => self.headless_render_path,
        }
    }
}

pub(crate) struct PreparedModernVariantStats {
    stats: VariantAtlasRenderStats,
}

impl PreparedModernVariantStats {
    pub(crate) fn new(prepared: &PreparedModernVariantRender<'_>) -> Self {
        Self {
            stats: prepared.initial_stats(),
        }
    }

    pub(crate) fn as_mut(&mut self) -> &mut VariantAtlasRenderStats {
        &mut self.stats
    }

    pub(crate) fn needs_live_stable_preview_overlay(&self) -> bool {
        self.stats.stable_preview_draws != 0
    }

    pub(crate) fn needs_headless_stable_overlay(&self) -> bool {
        self.stats.stable_draws != self.stats.effect_draws
    }

    pub(crate) fn finish(self) -> VariantAtlasRenderStats {
        self.stats
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreparedModernVariantOutput {
    Live,
    Headless,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ModernVariantRenderPath {
    EffectMaterialMode1Order,
    LiveIndexBaseWithOverlay,
    EffectMaterialWithStableOverlay,
    StableVariantFrame,
}

pub(crate) fn live_variant_render_path(stats: &VariantAtlasRenderStats) -> ModernVariantRenderPath {
    if !stats.needs_live_index_base() && stats.effect_draws == stats.stable_draws {
        ModernVariantRenderPath::EffectMaterialMode1Order
    } else if stats.needs_live_index_base() {
        ModernVariantRenderPath::LiveIndexBaseWithOverlay
    } else if stats.effect_draws != 0 {
        ModernVariantRenderPath::EffectMaterialWithStableOverlay
    } else {
        ModernVariantRenderPath::StableVariantFrame
    }
}

pub(crate) fn headless_variant_render_path(
    stats: &VariantAtlasRenderStats,
) -> ModernVariantRenderPath {
    if stats.needs_live_index_base() {
        ModernVariantRenderPath::LiveIndexBaseWithOverlay
    } else if stats.effect_draws != 0 && stats.effect_draws == stats.stable_draws {
        ModernVariantRenderPath::EffectMaterialMode1Order
    } else if stats.effect_draws != 0 {
        ModernVariantRenderPath::EffectMaterialWithStableOverlay
    } else {
        ModernVariantRenderPath::StableVariantFrame
    }
}
