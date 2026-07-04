use crate::gpu_work_item::{GpuRenderPlan, GpuWorkItem, GpuWorkItemKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GpuFrameWindowSelector {
    pub(crate) screen_idx: usize,
    pub(crate) layer_bit: u8,
    pub(crate) flags_shift: u32,
}

impl GpuFrameWindowSelector {
    pub(crate) fn main(layer_bit: u8, flags_shift: u32) -> Self {
        Self {
            screen_idx: 0,
            layer_bit,
            flags_shift,
        }
    }

    pub(crate) fn sub(layer_bit: u8, flags_shift: u32) -> Self {
        Self {
            screen_idx: 1,
            layer_bit,
            flags_shift,
        }
    }

    pub(crate) fn flags(self, windowsel: u32) -> u32 {
        (windowsel >> self.flags_shift) & 0x0f
    }

    pub(crate) fn is_windowed(self, screen_windowed: [u8; 2]) -> bool {
        screen_windowed[self.screen_idx] & self.layer_bit != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GpuFrameSpritePass {
    pub(crate) priority: u32,
    pub(crate) math_bit_pos: u32,
    pub(crate) window: GpuFrameWindowSelector,
}

impl GpuFrameSpritePass {
    pub(crate) fn new(priority: u32, math_bit_pos: u32, window: GpuFrameWindowSelector) -> Self {
        Self {
            priority,
            math_bit_pos,
            window,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GpuFrameBgPass {
    pub(crate) layer_idx: usize,
    pub(crate) hi_priority: bool,
    pub(crate) is_2bpp: bool,
    pub(crate) screen_enabled_layer_bit: Option<u8>,
    pub(crate) render_layer_bit: u32,
    pub(crate) math_bit_pos: u32,
    pub(crate) mosaic_layer_bit: u8,
    pub(crate) window: GpuFrameWindowSelector,
}

impl GpuFrameBgPass {
    pub(crate) fn mode1_main(layer_idx: usize, hi_priority: bool, math_bit_pos: u32) -> Self {
        let layer_bit = 1u8 << layer_idx;
        Self {
            layer_idx,
            hi_priority,
            is_2bpp: layer_idx == 2,
            screen_enabled_layer_bit: None,
            render_layer_bit: u32::from(layer_bit),
            math_bit_pos,
            mosaic_layer_bit: layer_bit,
            window: GpuFrameWindowSelector::main(layer_bit, (layer_idx as u32) * 4),
        }
    }

    pub(crate) fn mode1_sub(layer_idx: usize, hi_priority: bool) -> Self {
        let layer_bit = 1u8 << layer_idx;
        Self {
            layer_idx,
            hi_priority,
            is_2bpp: layer_idx == 2,
            screen_enabled_layer_bit: Some(layer_bit),
            render_layer_bit: 0,
            math_bit_pos: 255,
            mosaic_layer_bit: layer_bit,
            window: GpuFrameWindowSelector::sub(layer_bit, (layer_idx as u32) * 4),
        }
    }

    pub(crate) fn is_screen_enabled(self, screen_enabled: [u8; 2]) -> bool {
        match self.screen_enabled_layer_bit {
            Some(layer_bit) => screen_enabled[self.window.screen_idx] & layer_bit != 0,
            None => true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GpuFrameMode7Pass {
    pub(crate) math_bit_pos: u32,
    pub(crate) layer_bit: u32,
    pub(crate) window: GpuFrameWindowSelector,
}

impl GpuFrameMode7Pass {
    pub(crate) fn main_bg() -> Self {
        Self {
            math_bit_pos: 0,
            layer_bit: 1,
            window: GpuFrameWindowSelector::main(0x01, 0),
        }
    }

    pub(crate) fn sub_bg() -> Self {
        Self {
            math_bit_pos: 255,
            layer_bit: 0,
            window: GpuFrameWindowSelector::sub(0x01, 0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuFrameBackdropClearPass {
    MainCgram,
    SubTransparent,
}

impl GpuFrameBackdropClearPass {
    pub(crate) fn main_cgram() -> Self {
        Self::MainCgram
    }

    pub(crate) fn sub_transparent() -> Self {
        Self::SubTransparent
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuFrameRenderPhase {
    Main,
    Sub,
    PostProcess,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GpuFramePostProcessPass;

impl GpuFramePostProcessPass {
    pub(crate) fn final_output() -> Self {
        Self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuFrameRenderScreen {
    Main,
    Sub,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuFrameScreenWorkCommand {
    ClearBackdrop(GpuFrameBackdropClearPass),
    SpritePriority(GpuFrameSpritePass),
    BgLayer(GpuFrameBgPass),
    Mode7Bg(GpuFrameMode7Pass),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuFrameWorkCommand {
    Screen {
        screen: GpuFrameRenderScreen,
        command: GpuFrameScreenWorkCommand,
    },
    PostProcess(GpuFramePostProcessPass),
}

pub(crate) struct GpuFrameRenderPlan {
    work_items: GpuRenderPlan<GpuFrameWorkCommand>,
}

pub(crate) struct GpuFramePlan {
    prepare_plan: GpuFramePreparePlan,
    render_plan: GpuFrameRenderPlan,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuFramePlanCommand {
    Prepare(GpuFramePrepareCommand),
    Render(GpuFrameWorkCommand),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuFramePrepareCommand {
    CgramPalette,
    TileAtlas,
    Mode7Vram,
    Sprites,
}

pub(crate) struct GpuFramePreparePlan {
    work_items: GpuRenderPlan<GpuFramePrepareCommand>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuFrameRenderResourceRequirements {
    uses_cgram_palette: bool,
    uses_tile_atlas: bool,
    uses_mode7_vram: bool,
    uses_sprites: bool,
}

impl GpuFrameRenderResourceRequirements {
    pub(crate) fn include(self, other: Self) -> Self {
        Self {
            uses_cgram_palette: self.uses_cgram_palette || other.uses_cgram_palette,
            uses_tile_atlas: self.uses_tile_atlas || other.uses_tile_atlas,
            uses_mode7_vram: self.uses_mode7_vram || other.uses_mode7_vram,
            uses_sprites: self.uses_sprites || other.uses_sprites,
        }
    }

    pub(crate) fn uses_cgram_palette(self) -> bool {
        self.uses_cgram_palette
    }

    pub(crate) fn uses_tile_atlas(self) -> bool {
        self.uses_tile_atlas
    }

    pub(crate) fn uses_mode7_vram(self) -> bool {
        self.uses_mode7_vram
    }

    pub(crate) fn uses_sprites(self) -> bool {
        self.uses_sprites
    }
}

impl GpuFramePlan {
    pub(crate) fn from_render_plan(render_plan: GpuFrameRenderPlan) -> Self {
        Self {
            prepare_plan: render_plan.prepare_plan(),
            render_plan,
        }
    }

    pub(crate) fn execute_with<F>(self, mut execute: F)
    where
        F: FnMut(GpuFramePlanCommand),
    {
        self.prepare_plan
            .execute_with(|command| execute(GpuFramePlanCommand::Prepare(command)));
        self.render_plan
            .execute_with(|command| execute(GpuFramePlanCommand::Render(command)));
    }

    #[cfg(test)]
    pub(crate) fn prepare_plan(&self) -> &GpuFramePreparePlan {
        &self.prepare_plan
    }

    #[cfg(test)]
    pub(crate) fn render_plan(&self) -> &GpuFrameRenderPlan {
        &self.render_plan
    }
}

impl GpuFrameRenderPlan {
    pub(crate) fn push(&mut self, work_item: GpuFrameWorkCommand) {
        self.work_items.push(work_item);
    }

    pub(crate) fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = GpuFrameWorkCommand>,
    {
        self.work_items.extend(iter);
    }

    pub(crate) fn resource_requirements(&self) -> GpuFrameRenderResourceRequirements {
        self.work_items.fold(
            GpuFrameRenderResourceRequirements::default(),
            |requirements, work_item| requirements.include(work_item.resource_requirements()),
        )
    }

    pub(crate) fn prepare_plan(&self) -> GpuFramePreparePlan {
        GpuFramePreparePlan::from_resource_requirements(self.resource_requirements())
    }

    #[cfg(test)]
    pub(crate) fn uses_sprites(&self) -> bool {
        self.resource_requirements().uses_sprites()
    }

    pub(crate) fn execute_with<F>(self, execute: F)
    where
        F: FnMut(GpuFrameWorkCommand),
    {
        self.work_items.execute_with(execute);
    }

    #[cfg(test)]
    pub(crate) fn work_items(&self) -> &[GpuFrameWorkCommand] {
        self.work_items.work_items()
    }

    #[cfg(test)]
    pub(crate) fn kinds(&self) -> Vec<GpuWorkItemKind> {
        self.work_items.kinds()
    }
}

impl Default for GpuFrameRenderPlan {
    fn default() -> Self {
        Self {
            work_items: GpuRenderPlan::default(),
        }
    }
}

impl FromIterator<GpuFrameWorkCommand> for GpuFrameRenderPlan {
    fn from_iter<I: IntoIterator<Item = GpuFrameWorkCommand>>(iter: I) -> Self {
        Self {
            work_items: GpuRenderPlan::from_iter(iter),
        }
    }
}

impl IntoIterator for GpuFrameRenderPlan {
    type Item = GpuFrameWorkCommand;
    type IntoIter = <GpuRenderPlan<GpuFrameWorkCommand> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.work_items.into_iter()
    }
}

impl GpuFramePreparePlan {
    fn from_resource_requirements(requirements: GpuFrameRenderResourceRequirements) -> Self {
        let mut prepare_plan = Self::default();
        if requirements.uses_cgram_palette() {
            prepare_plan.push(GpuFramePrepareCommand::CgramPalette);
        }
        if requirements.uses_tile_atlas() {
            prepare_plan.push(GpuFramePrepareCommand::TileAtlas);
        }
        if requirements.uses_mode7_vram() {
            prepare_plan.push(GpuFramePrepareCommand::Mode7Vram);
        }
        if requirements.uses_sprites() {
            prepare_plan.push(GpuFramePrepareCommand::Sprites);
        }
        prepare_plan
    }

    fn push(&mut self, work_item: GpuFramePrepareCommand) {
        self.work_items.push(work_item);
    }

    pub(crate) fn execute_with<F>(self, execute: F)
    where
        F: FnMut(GpuFramePrepareCommand),
    {
        self.work_items.execute_with(execute);
    }

    #[cfg(test)]
    pub(crate) fn work_items(&self) -> &[GpuFramePrepareCommand] {
        self.work_items.work_items()
    }
}

impl Default for GpuFramePreparePlan {
    fn default() -> Self {
        Self {
            work_items: GpuRenderPlan::default(),
        }
    }
}

impl GpuFrameRenderScreen {
    #[cfg(test)]
    pub(crate) fn phase(self) -> GpuFrameRenderPhase {
        match self {
            Self::Main => GpuFrameRenderPhase::Main,
            Self::Sub => GpuFrameRenderPhase::Sub,
        }
    }

    pub(crate) fn work_item_kind(self, command: GpuFrameScreenWorkCommand) -> GpuWorkItemKind {
        match (self, command) {
            (Self::Main, GpuFrameScreenWorkCommand::ClearBackdrop(_)) => {
                GpuWorkItemKind::ClearBackdrop
            }
            (Self::Main, GpuFrameScreenWorkCommand::SpritePriority(_)) => {
                GpuWorkItemKind::MainSpritePriority
            }
            (Self::Main, GpuFrameScreenWorkCommand::BgLayer(_)) => GpuWorkItemKind::MainBgLayer,
            (Self::Main, GpuFrameScreenWorkCommand::Mode7Bg(_)) => GpuWorkItemKind::Mode7MainBg,
            (Self::Sub, GpuFrameScreenWorkCommand::ClearBackdrop(_)) => {
                GpuWorkItemKind::ClearSubBackdrop
            }
            (Self::Sub, GpuFrameScreenWorkCommand::SpritePriority(_)) => {
                GpuWorkItemKind::SubSpritePriority
            }
            (Self::Sub, GpuFrameScreenWorkCommand::BgLayer(_)) => GpuWorkItemKind::SubBgLayer,
            (Self::Sub, GpuFrameScreenWorkCommand::Mode7Bg(_)) => GpuWorkItemKind::Mode7SubBg,
        }
    }
}

impl GpuFrameWorkCommand {
    pub(crate) fn resource_requirements(&self) -> GpuFrameRenderResourceRequirements {
        match self {
            Self::Screen {
                command: GpuFrameScreenWorkCommand::BgLayer(_),
                ..
            } => GpuFrameRenderResourceRequirements {
                uses_cgram_palette: true,
                uses_tile_atlas: true,
                uses_mode7_vram: false,
                uses_sprites: false,
            },
            Self::Screen {
                command: GpuFrameScreenWorkCommand::Mode7Bg(_),
                ..
            } => GpuFrameRenderResourceRequirements {
                uses_cgram_palette: true,
                uses_tile_atlas: false,
                uses_mode7_vram: true,
                uses_sprites: false,
            },
            Self::Screen {
                command: GpuFrameScreenWorkCommand::SpritePriority(_),
                ..
            } => GpuFrameRenderResourceRequirements {
                uses_cgram_palette: true,
                uses_tile_atlas: false,
                uses_mode7_vram: false,
                uses_sprites: true,
            },
            Self::Screen {
                command: GpuFrameScreenWorkCommand::ClearBackdrop(_),
                ..
            }
            | Self::PostProcess(_) => GpuFrameRenderResourceRequirements::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn phase(&self) -> GpuFrameRenderPhase {
        match self {
            Self::Screen { screen, .. } => screen.phase(),
            Self::PostProcess(_) => GpuFrameRenderPhase::PostProcess,
        }
    }

    #[cfg(test)]
    pub(crate) fn uses_cgram_palette(&self) -> bool {
        self.resource_requirements().uses_cgram_palette()
    }

    #[cfg(test)]
    pub(crate) fn uses_tile_atlas(&self) -> bool {
        self.resource_requirements().uses_tile_atlas()
    }

    #[cfg(test)]
    pub(crate) fn uses_mode7_vram(&self) -> bool {
        self.resource_requirements().uses_mode7_vram()
    }

    #[cfg(test)]
    pub(crate) fn uses_sprites(&self) -> bool {
        self.resource_requirements().uses_sprites()
    }
}

impl GpuWorkItem for GpuFrameWorkCommand {
    fn kind(&self) -> GpuWorkItemKind {
        match self {
            Self::Screen { screen, command } => screen.work_item_kind(*command),
            Self::PostProcess(_) => GpuWorkItemKind::PostProcess,
        }
    }
}

pub(crate) fn main_frame_work_command(command: GpuFrameScreenWorkCommand) -> GpuFrameWorkCommand {
    GpuFrameWorkCommand::Screen {
        screen: GpuFrameRenderScreen::Main,
        command,
    }
}

pub(crate) fn sub_frame_work_command(command: GpuFrameScreenWorkCommand) -> GpuFrameWorkCommand {
    GpuFrameWorkCommand::Screen {
        screen: GpuFrameRenderScreen::Sub,
        command,
    }
}

pub(crate) fn post_process_frame_work_command() -> GpuFrameWorkCommand {
    GpuFrameWorkCommand::PostProcess(GpuFramePostProcessPass::final_output())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_selector_reads_shifted_flags_and_screen_layer_bits() {
        let main_bg3 = GpuFrameWindowSelector::main(0x04, 8);
        assert_eq!(main_bg3.flags(0x0000_0a00), 0x0a);
        assert!(main_bg3.is_windowed([0x04, 0x00]));
        assert!(!main_bg3.is_windowed([0x00, 0x04]));

        let sub_obj = GpuFrameWindowSelector::sub(0x10, 16);
        assert_eq!(sub_obj.flags(0x000b_0000), 0x0b);
        assert!(sub_obj.is_windowed([0x00, 0x10]));
        assert!(!sub_obj.is_windowed([0x10, 0x00]));
    }

    #[test]
    fn sprite_pass_groups_priority_math_and_window_metadata() {
        let pass = GpuFrameSpritePass::new(3, 4, GpuFrameWindowSelector::main(0x10, 16));

        assert_eq!(pass.priority, 3);
        assert_eq!(pass.math_bit_pos, 4);
        assert_eq!(pass.window, GpuFrameWindowSelector::main(0x10, 16));
    }

    #[test]
    fn bg_pass_groups_main_and_subscreen_render_metadata() {
        let main_bg3 = GpuFrameBgPass::mode1_main(2, true, 2);
        assert_eq!(main_bg3.layer_idx, 2);
        assert!(main_bg3.hi_priority);
        assert!(main_bg3.is_2bpp);
        assert_eq!(main_bg3.screen_enabled_layer_bit, None);
        assert_eq!(main_bg3.render_layer_bit, 0x04);
        assert_eq!(main_bg3.math_bit_pos, 2);
        assert_eq!(main_bg3.mosaic_layer_bit, 0x04);
        assert_eq!(main_bg3.window, GpuFrameWindowSelector::main(0x04, 8));
        assert!(main_bg3.is_screen_enabled([0x00, 0x00]));

        let sub_bg3 = GpuFrameBgPass::mode1_sub(2, true);
        assert_eq!(sub_bg3.layer_idx, 2);
        assert!(sub_bg3.hi_priority);
        assert!(sub_bg3.is_2bpp);
        assert_eq!(sub_bg3.screen_enabled_layer_bit, Some(0x04));
        assert_eq!(sub_bg3.render_layer_bit, 0);
        assert_eq!(sub_bg3.math_bit_pos, 255);
        assert_eq!(sub_bg3.mosaic_layer_bit, 0x04);
        assert_eq!(sub_bg3.window, GpuFrameWindowSelector::sub(0x04, 8));
        assert!(sub_bg3.is_screen_enabled([0x00, 0x04]));
        assert!(!sub_bg3.is_screen_enabled([0x04, 0x00]));
    }

    #[test]
    fn mode7_pass_groups_main_and_subscreen_render_metadata() {
        let main = GpuFrameMode7Pass::main_bg();
        assert_eq!(main.math_bit_pos, 0);
        assert_eq!(main.layer_bit, 1);
        assert_eq!(main.window, GpuFrameWindowSelector::main(0x01, 0));

        let sub = GpuFrameMode7Pass::sub_bg();
        assert_eq!(sub.math_bit_pos, 255);
        assert_eq!(sub.layer_bit, 0);
        assert_eq!(sub.window, GpuFrameWindowSelector::sub(0x01, 0));
    }

    #[test]
    fn backdrop_clear_pass_names_main_and_subscreen_clear_modes() {
        assert_eq!(
            GpuFrameBackdropClearPass::main_cgram(),
            GpuFrameBackdropClearPass::MainCgram
        );
        assert_eq!(
            GpuFrameBackdropClearPass::sub_transparent(),
            GpuFrameBackdropClearPass::SubTransparent
        );
    }

    #[test]
    fn post_process_pass_names_final_output_target() {
        assert_eq!(
            GpuFramePostProcessPass::final_output(),
            GpuFramePostProcessPass
        );
        assert_eq!(
            post_process_frame_work_command(),
            GpuFrameWorkCommand::PostProcess(GpuFramePostProcessPass::final_output())
        );
    }

    #[test]
    fn frame_work_command_reports_sprite_usage_from_command_kind() {
        assert!(
            main_frame_work_command(GpuFrameScreenWorkCommand::SpritePriority(
                GpuFrameSpritePass::new(0, 4, GpuFrameWindowSelector::main(0x10, 16))
            ))
            .uses_sprites()
        );

        assert!(!main_frame_work_command(GpuFrameScreenWorkCommand::BgLayer(
            GpuFrameBgPass::mode1_main(0, false, 0)
        ))
        .uses_sprites());
        assert!(!post_process_frame_work_command().uses_sprites());
    }

    #[test]
    fn frame_work_command_reports_tile_atlas_usage_from_command_kind() {
        assert!(main_frame_work_command(GpuFrameScreenWorkCommand::BgLayer(
            GpuFrameBgPass::mode1_main(0, false, 0)
        ))
        .uses_tile_atlas());

        assert!(
            !main_frame_work_command(GpuFrameScreenWorkCommand::SpritePriority(
                GpuFrameSpritePass::new(0, 4, GpuFrameWindowSelector::main(0x10, 16))
            ))
            .uses_tile_atlas()
        );
        assert!(!main_frame_work_command(GpuFrameScreenWorkCommand::Mode7Bg(
            GpuFrameMode7Pass::main_bg()
        ))
        .uses_tile_atlas());
        assert!(!post_process_frame_work_command().uses_tile_atlas());
    }

    #[test]
    fn frame_work_command_reports_cgram_palette_usage_from_command_kind() {
        assert!(main_frame_work_command(GpuFrameScreenWorkCommand::BgLayer(
            GpuFrameBgPass::mode1_main(0, false, 0)
        ))
        .uses_cgram_palette());

        assert!(
            main_frame_work_command(GpuFrameScreenWorkCommand::SpritePriority(
                GpuFrameSpritePass::new(0, 4, GpuFrameWindowSelector::main(0x10, 16))
            ))
            .uses_cgram_palette()
        );
        assert!(main_frame_work_command(GpuFrameScreenWorkCommand::Mode7Bg(
            GpuFrameMode7Pass::main_bg()
        ))
        .uses_cgram_palette());
        assert!(
            !main_frame_work_command(GpuFrameScreenWorkCommand::ClearBackdrop(
                GpuFrameBackdropClearPass::main_cgram()
            ))
            .uses_cgram_palette()
        );
        assert!(!post_process_frame_work_command().uses_cgram_palette());
    }

    #[test]
    fn frame_work_command_reports_mode7_vram_usage_from_command_kind() {
        assert!(main_frame_work_command(GpuFrameScreenWorkCommand::Mode7Bg(
            GpuFrameMode7Pass::main_bg()
        ))
        .uses_mode7_vram());

        assert!(!main_frame_work_command(GpuFrameScreenWorkCommand::BgLayer(
            GpuFrameBgPass::mode1_main(0, false, 0)
        ))
        .uses_mode7_vram());
        assert!(
            !main_frame_work_command(GpuFrameScreenWorkCommand::SpritePriority(
                GpuFrameSpritePass::new(0, 4, GpuFrameWindowSelector::main(0x10, 16))
            ))
            .uses_mode7_vram()
        );
        assert!(!post_process_frame_work_command().uses_mode7_vram());
    }

    #[test]
    fn frame_render_plan_reports_sprite_usage_from_work_items() {
        let sprite_free_plan = GpuFrameRenderPlan::from_iter([
            main_frame_work_command(GpuFrameScreenWorkCommand::BgLayer(
                GpuFrameBgPass::mode1_main(0, false, 0),
            )),
            post_process_frame_work_command(),
        ]);
        assert!(!sprite_free_plan.uses_sprites());

        let sprite_plan = GpuFrameRenderPlan::from_iter([
            main_frame_work_command(GpuFrameScreenWorkCommand::BgLayer(
                GpuFrameBgPass::mode1_main(0, false, 0),
            )),
            main_frame_work_command(GpuFrameScreenWorkCommand::SpritePriority(
                GpuFrameSpritePass::new(0, 4, GpuFrameWindowSelector::main(0x10, 16)),
            )),
        ]);
        assert!(sprite_plan.uses_sprites());
    }

    #[test]
    fn frame_render_plan_reports_resource_requirements_from_work_items() {
        let sprite_free_plan = GpuFrameRenderPlan::from_iter([
            main_frame_work_command(GpuFrameScreenWorkCommand::BgLayer(
                GpuFrameBgPass::mode1_main(0, false, 0),
            )),
            post_process_frame_work_command(),
        ]);
        assert_eq!(
            sprite_free_plan.resource_requirements(),
            GpuFrameRenderResourceRequirements {
                uses_cgram_palette: true,
                uses_tile_atlas: true,
                uses_mode7_vram: false,
                uses_sprites: false
            }
        );

        let sprite_plan = GpuFrameRenderPlan::from_iter([
            main_frame_work_command(GpuFrameScreenWorkCommand::BgLayer(
                GpuFrameBgPass::mode1_main(0, false, 0),
            )),
            main_frame_work_command(GpuFrameScreenWorkCommand::SpritePriority(
                GpuFrameSpritePass::new(0, 4, GpuFrameWindowSelector::main(0x10, 16)),
            )),
        ]);
        assert!(sprite_plan.resource_requirements().uses_sprites());
        assert!(sprite_plan.resource_requirements().uses_tile_atlas());

        let sprite_only_plan = GpuFrameRenderPlan::from_iter([
            main_frame_work_command(GpuFrameScreenWorkCommand::SpritePriority(
                GpuFrameSpritePass::new(0, 4, GpuFrameWindowSelector::main(0x10, 16)),
            )),
            post_process_frame_work_command(),
        ]);
        assert_eq!(
            sprite_only_plan.resource_requirements(),
            GpuFrameRenderResourceRequirements {
                uses_cgram_palette: true,
                uses_tile_atlas: false,
                uses_mode7_vram: false,
                uses_sprites: true
            }
        );

        let mode7_plan = GpuFrameRenderPlan::from_iter([
            main_frame_work_command(GpuFrameScreenWorkCommand::Mode7Bg(
                GpuFrameMode7Pass::main_bg(),
            )),
            post_process_frame_work_command(),
        ]);
        assert_eq!(
            mode7_plan.resource_requirements(),
            GpuFrameRenderResourceRequirements {
                uses_cgram_palette: true,
                uses_tile_atlas: false,
                uses_mode7_vram: true,
                uses_sprites: false
            }
        );

        let clear_only_plan = GpuFrameRenderPlan::from_iter([
            main_frame_work_command(GpuFrameScreenWorkCommand::ClearBackdrop(
                GpuFrameBackdropClearPass::main_cgram(),
            )),
            post_process_frame_work_command(),
        ]);
        assert_eq!(
            clear_only_plan.resource_requirements(),
            GpuFrameRenderResourceRequirements {
                uses_cgram_palette: false,
                uses_tile_atlas: false,
                uses_mode7_vram: false,
                uses_sprites: false
            }
        );
    }

    #[test]
    fn frame_render_plan_builds_prepare_plan_from_resource_requirements() {
        let bg_sprite_plan = GpuFrameRenderPlan::from_iter([
            main_frame_work_command(GpuFrameScreenWorkCommand::BgLayer(
                GpuFrameBgPass::mode1_main(0, false, 0),
            )),
            main_frame_work_command(GpuFrameScreenWorkCommand::SpritePriority(
                GpuFrameSpritePass::new(0, 4, GpuFrameWindowSelector::main(0x10, 16)),
            )),
        ]);
        assert_eq!(
            bg_sprite_plan.prepare_plan().work_items(),
            &[
                GpuFramePrepareCommand::CgramPalette,
                GpuFramePrepareCommand::TileAtlas,
                GpuFramePrepareCommand::Sprites,
            ]
        );

        let mode7_sprite_plan = GpuFrameRenderPlan::from_iter([
            main_frame_work_command(GpuFrameScreenWorkCommand::Mode7Bg(
                GpuFrameMode7Pass::main_bg(),
            )),
            main_frame_work_command(GpuFrameScreenWorkCommand::SpritePriority(
                GpuFrameSpritePass::new(0, 4, GpuFrameWindowSelector::main(0x10, 16)),
            )),
        ]);
        assert_eq!(
            mode7_sprite_plan.prepare_plan().work_items(),
            &[
                GpuFramePrepareCommand::CgramPalette,
                GpuFramePrepareCommand::Mode7Vram,
                GpuFramePrepareCommand::Sprites,
            ]
        );

        let clear_only_plan = GpuFrameRenderPlan::from_iter([
            main_frame_work_command(GpuFrameScreenWorkCommand::ClearBackdrop(
                GpuFrameBackdropClearPass::main_cgram(),
            )),
            post_process_frame_work_command(),
        ]);
        assert_eq!(clear_only_plan.prepare_plan().work_items(), &[]);
    }

    #[test]
    fn frame_plan_carries_prepare_and_render_plans() {
        let bg = main_frame_work_command(GpuFrameScreenWorkCommand::BgLayer(
            GpuFrameBgPass::mode1_main(0, false, 0),
        ));
        let sprite = main_frame_work_command(GpuFrameScreenWorkCommand::SpritePriority(
            GpuFrameSpritePass::new(0, 4, GpuFrameWindowSelector::main(0x10, 16)),
        ));
        let post_process = post_process_frame_work_command();
        let frame_plan = GpuFramePlan::from_render_plan(GpuFrameRenderPlan::from_iter([
            bg,
            sprite,
            post_process,
        ]));

        assert_eq!(
            frame_plan.prepare_plan().work_items(),
            &[
                GpuFramePrepareCommand::CgramPalette,
                GpuFramePrepareCommand::TileAtlas,
                GpuFramePrepareCommand::Sprites,
            ]
        );
        assert_eq!(
            frame_plan.render_plan().work_items(),
            &[bg, sprite, post_process]
        );
    }

    #[test]
    fn frame_plan_executes_prepare_phase_before_render_phase() {
        let bg = main_frame_work_command(GpuFrameScreenWorkCommand::BgLayer(
            GpuFrameBgPass::mode1_main(0, false, 0),
        ));
        let sprite = main_frame_work_command(GpuFrameScreenWorkCommand::SpritePriority(
            GpuFrameSpritePass::new(0, 4, GpuFrameWindowSelector::main(0x10, 16)),
        ));
        let post_process = post_process_frame_work_command();
        let frame_plan = GpuFramePlan::from_render_plan(GpuFrameRenderPlan::from_iter([
            bg,
            sprite,
            post_process,
        ]));
        let trace = std::cell::RefCell::new(Vec::new());

        frame_plan.execute_with(|command| match command {
            GpuFramePlanCommand::Prepare(command) => {
                trace.borrow_mut().push(format!("prepare:{command:?}"));
            }
            GpuFramePlanCommand::Render(command) => {
                trace
                    .borrow_mut()
                    .push(format!("render:{:?}", command.kind()));
            }
        });

        assert_eq!(
            trace.into_inner(),
            vec![
                "prepare:CgramPalette",
                "prepare:TileAtlas",
                "prepare:Sprites",
                "render:MainBgLayer",
                "render:MainSpritePriority",
                "render:PostProcess",
            ]
        );
    }
}
