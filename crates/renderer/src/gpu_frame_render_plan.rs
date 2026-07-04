use crate::gpu_frame::GpuFrame;
use crate::gpu_frame_work_command::{
    main_frame_work_command, post_process_frame_work_command, sub_frame_work_command,
    GpuFrameBackdropClearPass, GpuFrameBgPass, GpuFrameMode7Pass, GpuFrameRenderPlan,
    GpuFrameScreenWorkCommand, GpuFrameSpritePass, GpuFrameWindowSelector,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GpuFrameRenderPlanContext {
    Mode1 {
        has_main_bg: bool,
        has_main_sprites: bool,
        has_sub_bg: bool,
        has_sub_sprites: bool,
    },
    Mode7 {
        has_main_sprites: bool,
        has_sub_mode7_bg: bool,
        has_sub_sprites: bool,
    },
}

impl GpuFrameRenderPlanContext {
    fn from_frame(frame: &GpuFrame<'_>) -> Self {
        let has_main_sprites = frame
            .scanlines
            .iter()
            .any(|scanline| scanline.screen_enabled_main & 0x10 != 0);
        let has_sub_sprites = frame.screen_enabled[1] & 0x10 != 0;

        if frame.mode == 7 {
            return Self::Mode7 {
                has_main_sprites,
                has_sub_mode7_bg: frame.screen_enabled[1] & 1 != 0,
                has_sub_sprites,
            };
        }

        Self::Mode1 {
            has_main_bg: frame
                .scanlines
                .iter()
                .any(|scanline| scanline.screen_enabled_main & 0x07 != 0),
            has_main_sprites,
            has_sub_bg: frame.screen_enabled[1] & 0x07 != 0,
            has_sub_sprites,
        }
    }

    fn render_plan(&self) -> GpuFrameRenderPlan {
        match *self {
            Self::Mode1 {
                has_main_bg,
                has_main_sprites,
                has_sub_bg,
                has_sub_sprites,
            } => {
                build_mode1_render_plan(has_main_bg, has_main_sprites, has_sub_bg, has_sub_sprites)
            }
            Self::Mode7 {
                has_main_sprites,
                has_sub_mode7_bg,
                has_sub_sprites,
            } => build_mode7_render_plan(has_main_sprites, has_sub_mode7_bg, has_sub_sprites),
        }
    }
}

impl GpuFrameRenderPlan {
    pub(crate) fn from_frame(frame: &GpuFrame<'_>) -> Self {
        GpuFrameRenderPlanContext::from_frame(frame).render_plan()
    }
}

fn build_mode1_render_plan(
    has_main_bg: bool,
    has_main_sprites: bool,
    has_sub_bg: bool,
    has_sub_sprites: bool,
) -> GpuFrameRenderPlan {
    let mut render_plan = GpuFrameRenderPlan::default();
    render_plan.push(main_frame_work_command(main_backdrop_clear_work_item()));
    render_plan.extend(build_mode1_main_render_plan(has_main_bg, has_main_sprites));
    render_plan.extend(build_mode1_sub_render_plan(has_sub_bg, has_sub_sprites));
    render_plan.extend(build_post_process_render_plan());
    render_plan
}

fn build_mode1_main_render_plan(has_main_bg: bool, has_main_sprites: bool) -> GpuFrameRenderPlan {
    let mut render_plan = GpuFrameRenderPlan::default();
    if has_main_sprites && !has_main_bg {
        render_plan.extend(
            (0..=3)
                .map(main_sprite_work_item)
                .map(main_frame_work_command),
        );
    }

    if has_main_bg {
        // CPU Mode 1 z-order:
        //   BG3-lo, OBJ0, OBJ1, BG2-lo, BG1-lo, OBJ2,
        //   BG2-hi, BG1-hi, OBJ3, BG3-hi.
        render_plan.push(main_frame_work_command(main_bg_work_item(2, false, 2)));
        if has_main_sprites {
            render_plan.push(main_frame_work_command(main_sprite_work_item(0)));
            render_plan.push(main_frame_work_command(main_sprite_work_item(1)));
        }
        render_plan.push(main_frame_work_command(main_bg_work_item(1, false, 1)));
        render_plan.push(main_frame_work_command(main_bg_work_item(0, false, 0)));
        if has_main_sprites {
            render_plan.push(main_frame_work_command(main_sprite_work_item(2)));
        }
        render_plan.push(main_frame_work_command(main_bg_work_item(1, true, 1)));
        render_plan.push(main_frame_work_command(main_bg_work_item(0, true, 0)));
        if has_main_sprites {
            render_plan.push(main_frame_work_command(main_sprite_work_item(3)));
        }
        render_plan.push(main_frame_work_command(main_bg_work_item(2, true, 2)));
    }

    render_plan
}

fn build_mode1_sub_render_plan(has_sub_bg: bool, has_sub_sprites: bool) -> GpuFrameRenderPlan {
    let mut render_plan = GpuFrameRenderPlan::default();
    render_plan.push(sub_frame_work_command(sub_backdrop_clear_work_item()));
    render_plan.push(sub_frame_work_command(sub_bg_work_item(2, false)));
    if has_sub_sprites && !has_sub_bg {
        render_plan.extend(
            (0..=3)
                .map(sub_sprite_work_item)
                .map(sub_frame_work_command),
        );
    }
    if has_sub_sprites && has_sub_bg {
        render_plan.push(sub_frame_work_command(sub_sprite_work_item(0)));
        render_plan.push(sub_frame_work_command(sub_sprite_work_item(1)));
    }
    render_plan.push(sub_frame_work_command(sub_bg_work_item(1, false)));
    render_plan.push(sub_frame_work_command(sub_bg_work_item(0, false)));
    if has_sub_sprites && has_sub_bg {
        render_plan.push(sub_frame_work_command(sub_sprite_work_item(2)));
    }
    render_plan.push(sub_frame_work_command(sub_bg_work_item(1, true)));
    render_plan.push(sub_frame_work_command(sub_bg_work_item(0, true)));
    if has_sub_sprites && has_sub_bg {
        render_plan.push(sub_frame_work_command(sub_sprite_work_item(3)));
    }
    render_plan.push(sub_frame_work_command(sub_bg_work_item(2, true)));

    render_plan
}

fn main_bg_work_item(
    layer_idx: usize,
    hi_priority: bool,
    math_bit_pos: u32,
) -> GpuFrameScreenWorkCommand {
    GpuFrameScreenWorkCommand::BgLayer(GpuFrameBgPass::mode1_main(
        layer_idx,
        hi_priority,
        math_bit_pos,
    ))
}

fn main_backdrop_clear_work_item() -> GpuFrameScreenWorkCommand {
    GpuFrameScreenWorkCommand::ClearBackdrop(GpuFrameBackdropClearPass::main_cgram())
}

fn main_sprite_work_item(priority: u32) -> GpuFrameScreenWorkCommand {
    GpuFrameScreenWorkCommand::SpritePriority(GpuFrameSpritePass::new(
        priority,
        4,
        GpuFrameWindowSelector::main(0x10, 16),
    ))
}

fn sub_backdrop_clear_work_item() -> GpuFrameScreenWorkCommand {
    GpuFrameScreenWorkCommand::ClearBackdrop(GpuFrameBackdropClearPass::sub_transparent())
}

fn sub_bg_work_item(layer_idx: usize, hi_priority: bool) -> GpuFrameScreenWorkCommand {
    GpuFrameScreenWorkCommand::BgLayer(GpuFrameBgPass::mode1_sub(layer_idx, hi_priority))
}

fn sub_sprite_work_item(priority: u32) -> GpuFrameScreenWorkCommand {
    GpuFrameScreenWorkCommand::SpritePriority(GpuFrameSpritePass::new(
        priority,
        255,
        GpuFrameWindowSelector::sub(0x10, 16),
    ))
}

fn main_mode7_bg_work_item() -> GpuFrameScreenWorkCommand {
    GpuFrameScreenWorkCommand::Mode7Bg(GpuFrameMode7Pass::main_bg())
}

fn sub_mode7_bg_work_item() -> GpuFrameScreenWorkCommand {
    GpuFrameScreenWorkCommand::Mode7Bg(GpuFrameMode7Pass::sub_bg())
}

fn build_mode7_render_plan(
    has_main_sprites: bool,
    has_sub_mode7_bg: bool,
    has_sub_sprites: bool,
) -> GpuFrameRenderPlan {
    let mut render_plan = GpuFrameRenderPlan::default();
    render_plan.push(main_frame_work_command(main_backdrop_clear_work_item()));
    render_plan.extend(build_mode7_main_render_plan(has_main_sprites));
    render_plan.extend(build_mode7_sub_render_plan(
        has_sub_mode7_bg,
        has_sub_sprites,
    ));
    render_plan.extend(build_post_process_render_plan());
    render_plan
}

fn build_mode7_main_render_plan(has_main_sprites: bool) -> GpuFrameRenderPlan {
    let mut render_plan = GpuFrameRenderPlan::default();
    if has_main_sprites {
        render_plan.push(main_frame_work_command(main_sprite_work_item(0)));
    }
    render_plan.push(main_frame_work_command(main_mode7_bg_work_item()));
    if has_main_sprites {
        render_plan.extend(
            (1..=3)
                .map(main_sprite_work_item)
                .map(main_frame_work_command),
        );
    }

    render_plan
}

fn build_mode7_sub_render_plan(
    has_sub_mode7_bg: bool,
    has_sub_sprites: bool,
) -> GpuFrameRenderPlan {
    let mut render_plan = GpuFrameRenderPlan::default();
    render_plan.push(sub_frame_work_command(sub_backdrop_clear_work_item()));
    if has_sub_mode7_bg {
        render_plan.push(sub_frame_work_command(sub_mode7_bg_work_item()));
    }
    if has_sub_sprites {
        render_plan.extend(
            (0..=3)
                .map(sub_sprite_work_item)
                .map(sub_frame_work_command),
        );
    }
    render_plan
}

fn build_post_process_render_plan() -> GpuFrameRenderPlan {
    let mut render_plan = GpuFrameRenderPlan::default();
    render_plan.push(post_process_frame_work_command());
    render_plan
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu_frame::{BgLayerRegs, Mode7Regs, ObjRegs, ScanlineRegs};
    use crate::gpu_frame_work_command::{
        post_process_frame_work_command, GpuFrameRenderPhase, GpuFrameWorkCommand,
    };
    use crate::gpu_work_item::GpuWorkItemKind;

    fn frame_plan_phases(plan: &GpuFrameRenderPlan) -> Vec<GpuFrameRenderPhase> {
        plan.work_items()
            .iter()
            .map(GpuFrameWorkCommand::phase)
            .collect()
    }

    fn frame_plan_commands(plan: &GpuFrameRenderPlan) -> Vec<GpuFrameWorkCommand> {
        plan.work_items().to_vec()
    }

    fn test_frame(mode: u8, screen_enabled: [u8; 2]) -> GpuFrame<'static> {
        GpuFrame {
            vram: &[],
            cgram: &[],
            oam: &[],
            mode,
            bg: [BgLayerRegs::default(); 4],
            obj: ObjRegs::default(),
            mosaic_enabled: 0,
            mosaic_size: 0,
            extra_left_right: 0,
            mode7: Mode7Regs::default(),
            screen_enabled,
            screen_windowed: [0; 2],
            brightness: 15,
            forced_blank: false,
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
            scanlines: Box::new([ScanlineRegs::default(); 224]),
        }
    }

    #[test]
    fn render_plan_context_detects_mode1_surface_activity() {
        let mut frame = test_frame(1, [0, 0x17]);
        frame.scanlines[7].screen_enabled_main = 0x12;

        let context = GpuFrameRenderPlanContext::from_frame(&frame);

        assert_eq!(
            context,
            GpuFrameRenderPlanContext::Mode1 {
                has_main_bg: true,
                has_main_sprites: true,
                has_sub_bg: true,
                has_sub_sprites: true,
            }
        );
        assert!(context.render_plan().uses_sprites());
    }

    #[test]
    fn render_plan_context_detects_mode7_surface_activity() {
        let mut frame = test_frame(7, [0, 0x11]);
        frame.scanlines[7].screen_enabled_main = 0x10;

        let context = GpuFrameRenderPlanContext::from_frame(&frame);

        assert_eq!(
            context,
            GpuFrameRenderPlanContext::Mode7 {
                has_main_sprites: true,
                has_sub_mode7_bg: true,
                has_sub_sprites: true,
            }
        );
        assert!(context.render_plan().uses_sprites());
    }

    #[test]
    fn render_plan_context_reports_sprite_free_frames() {
        let frame = test_frame(1, [0, 0]);
        let context = GpuFrameRenderPlanContext::from_frame(&frame);

        assert_eq!(
            context,
            GpuFrameRenderPlanContext::Mode1 {
                has_main_bg: false,
                has_main_sprites: false,
                has_sub_bg: false,
                has_sub_sprites: false,
            }
        );
        assert!(!context.render_plan().uses_sprites());
    }

    #[test]
    fn frame_render_plan_builds_directly_from_frame() {
        let mut sprite_frame = test_frame(1, [0, 0]);
        sprite_frame.scanlines[7].screen_enabled_main = 0x10;

        assert!(GpuFrameRenderPlan::from_frame(&sprite_frame).uses_sprites());

        let sprite_free_frame = test_frame(1, [0, 0]);

        assert!(!GpuFrameRenderPlan::from_frame(&sprite_free_frame).uses_sprites());
    }

    #[test]
    fn main_bg_work_item_carries_main_screen_render_metadata() {
        assert_eq!(
            main_bg_work_item(2, true, 2),
            GpuFrameScreenWorkCommand::BgLayer(GpuFrameBgPass::mode1_main(2, true, 2))
        );
    }

    #[test]
    fn sub_bg_work_item_carries_subscreen_render_metadata() {
        assert_eq!(
            sub_bg_work_item(2, true),
            GpuFrameScreenWorkCommand::BgLayer(GpuFrameBgPass::mode1_sub(2, true))
        );
    }

    #[test]
    fn mode7_work_items_carry_render_metadata() {
        assert_eq!(
            main_mode7_bg_work_item(),
            GpuFrameScreenWorkCommand::Mode7Bg(GpuFrameMode7Pass::main_bg())
        );
        assert_eq!(
            sub_mode7_bg_work_item(),
            GpuFrameScreenWorkCommand::Mode7Bg(GpuFrameMode7Pass::sub_bg())
        );
    }

    #[test]
    fn sprite_work_items_carry_render_metadata() {
        assert_eq!(
            main_sprite_work_item(2),
            GpuFrameScreenWorkCommand::SpritePriority(GpuFrameSpritePass::new(
                2,
                4,
                GpuFrameWindowSelector::main(0x10, 16),
            ))
        );
        assert_eq!(
            sub_sprite_work_item(2),
            GpuFrameScreenWorkCommand::SpritePriority(GpuFrameSpritePass::new(
                2,
                255,
                GpuFrameWindowSelector::sub(0x10, 16),
            ))
        );
    }

    #[test]
    fn build_mode1_render_plan_preserves_full_gpu_draw_order() {
        let plan = build_mode1_render_plan(true, true, true, true);

        assert_eq!(
            plan.kinds(),
            vec![
                GpuWorkItemKind::ClearBackdrop,
                GpuWorkItemKind::MainBgLayer,
                GpuWorkItemKind::MainSpritePriority,
                GpuWorkItemKind::MainSpritePriority,
                GpuWorkItemKind::MainBgLayer,
                GpuWorkItemKind::MainBgLayer,
                GpuWorkItemKind::MainSpritePriority,
                GpuWorkItemKind::MainBgLayer,
                GpuWorkItemKind::MainBgLayer,
                GpuWorkItemKind::MainSpritePriority,
                GpuWorkItemKind::MainBgLayer,
                GpuWorkItemKind::ClearSubBackdrop,
                GpuWorkItemKind::SubBgLayer,
                GpuWorkItemKind::SubSpritePriority,
                GpuWorkItemKind::SubSpritePriority,
                GpuWorkItemKind::SubBgLayer,
                GpuWorkItemKind::SubBgLayer,
                GpuWorkItemKind::SubSpritePriority,
                GpuWorkItemKind::SubBgLayer,
                GpuWorkItemKind::SubBgLayer,
                GpuWorkItemKind::SubSpritePriority,
                GpuWorkItemKind::SubBgLayer,
                GpuWorkItemKind::PostProcess,
            ]
        );
        assert_eq!(
            frame_plan_phases(&plan),
            vec![
                GpuFrameRenderPhase::Main,
                GpuFrameRenderPhase::Main,
                GpuFrameRenderPhase::Main,
                GpuFrameRenderPhase::Main,
                GpuFrameRenderPhase::Main,
                GpuFrameRenderPhase::Main,
                GpuFrameRenderPhase::Main,
                GpuFrameRenderPhase::Main,
                GpuFrameRenderPhase::Main,
                GpuFrameRenderPhase::Main,
                GpuFrameRenderPhase::Main,
                GpuFrameRenderPhase::Sub,
                GpuFrameRenderPhase::Sub,
                GpuFrameRenderPhase::Sub,
                GpuFrameRenderPhase::Sub,
                GpuFrameRenderPhase::Sub,
                GpuFrameRenderPhase::Sub,
                GpuFrameRenderPhase::Sub,
                GpuFrameRenderPhase::Sub,
                GpuFrameRenderPhase::Sub,
                GpuFrameRenderPhase::Sub,
                GpuFrameRenderPhase::Sub,
                GpuFrameRenderPhase::PostProcess,
            ]
        );
        assert_eq!(
            frame_plan_commands(&plan),
            vec![
                main_frame_work_command(main_backdrop_clear_work_item()),
                main_frame_work_command(main_bg_work_item(2, false, 2)),
                main_frame_work_command(main_sprite_work_item(0)),
                main_frame_work_command(main_sprite_work_item(1)),
                main_frame_work_command(main_bg_work_item(1, false, 1)),
                main_frame_work_command(main_bg_work_item(0, false, 0)),
                main_frame_work_command(main_sprite_work_item(2)),
                main_frame_work_command(main_bg_work_item(1, true, 1)),
                main_frame_work_command(main_bg_work_item(0, true, 0)),
                main_frame_work_command(main_sprite_work_item(3)),
                main_frame_work_command(main_bg_work_item(2, true, 2)),
                sub_frame_work_command(sub_backdrop_clear_work_item()),
                sub_frame_work_command(sub_bg_work_item(2, false)),
                sub_frame_work_command(sub_sprite_work_item(0)),
                sub_frame_work_command(sub_sprite_work_item(1)),
                sub_frame_work_command(sub_bg_work_item(1, false)),
                sub_frame_work_command(sub_bg_work_item(0, false)),
                sub_frame_work_command(sub_sprite_work_item(2)),
                sub_frame_work_command(sub_bg_work_item(1, true)),
                sub_frame_work_command(sub_bg_work_item(0, true)),
                sub_frame_work_command(sub_sprite_work_item(3)),
                sub_frame_work_command(sub_bg_work_item(2, true)),
                post_process_frame_work_command(),
            ]
        );
    }

    #[test]
    fn build_mode1_render_plan_preserves_sprite_only_draw_order() {
        let plan = build_mode1_render_plan(false, true, false, true);

        assert_eq!(
            frame_plan_commands(&plan),
            vec![
                main_frame_work_command(main_backdrop_clear_work_item()),
                main_frame_work_command(main_sprite_work_item(0)),
                main_frame_work_command(main_sprite_work_item(1)),
                main_frame_work_command(main_sprite_work_item(2)),
                main_frame_work_command(main_sprite_work_item(3)),
                sub_frame_work_command(sub_backdrop_clear_work_item()),
                sub_frame_work_command(sub_bg_work_item(2, false)),
                sub_frame_work_command(sub_sprite_work_item(0)),
                sub_frame_work_command(sub_sprite_work_item(1)),
                sub_frame_work_command(sub_sprite_work_item(2)),
                sub_frame_work_command(sub_sprite_work_item(3)),
                sub_frame_work_command(sub_bg_work_item(1, false)),
                sub_frame_work_command(sub_bg_work_item(0, false)),
                sub_frame_work_command(sub_bg_work_item(1, true)),
                sub_frame_work_command(sub_bg_work_item(0, true)),
                sub_frame_work_command(sub_bg_work_item(2, true)),
                post_process_frame_work_command(),
            ]
        );
    }

    #[test]
    fn build_mode7_render_plan_preserves_full_gpu_draw_order() {
        let plan = build_mode7_render_plan(true, true, true);

        assert_eq!(
            plan.kinds(),
            vec![
                GpuWorkItemKind::ClearBackdrop,
                GpuWorkItemKind::MainSpritePriority,
                GpuWorkItemKind::Mode7MainBg,
                GpuWorkItemKind::MainSpritePriority,
                GpuWorkItemKind::MainSpritePriority,
                GpuWorkItemKind::MainSpritePriority,
                GpuWorkItemKind::ClearSubBackdrop,
                GpuWorkItemKind::Mode7SubBg,
                GpuWorkItemKind::SubSpritePriority,
                GpuWorkItemKind::SubSpritePriority,
                GpuWorkItemKind::SubSpritePriority,
                GpuWorkItemKind::SubSpritePriority,
                GpuWorkItemKind::PostProcess,
            ]
        );
        assert_eq!(
            frame_plan_phases(&plan),
            vec![
                GpuFrameRenderPhase::Main,
                GpuFrameRenderPhase::Main,
                GpuFrameRenderPhase::Main,
                GpuFrameRenderPhase::Main,
                GpuFrameRenderPhase::Main,
                GpuFrameRenderPhase::Main,
                GpuFrameRenderPhase::Sub,
                GpuFrameRenderPhase::Sub,
                GpuFrameRenderPhase::Sub,
                GpuFrameRenderPhase::Sub,
                GpuFrameRenderPhase::Sub,
                GpuFrameRenderPhase::Sub,
                GpuFrameRenderPhase::PostProcess,
            ]
        );
        assert_eq!(
            frame_plan_commands(&plan),
            vec![
                main_frame_work_command(main_backdrop_clear_work_item()),
                main_frame_work_command(main_sprite_work_item(0)),
                main_frame_work_command(main_mode7_bg_work_item()),
                main_frame_work_command(main_sprite_work_item(1)),
                main_frame_work_command(main_sprite_work_item(2)),
                main_frame_work_command(main_sprite_work_item(3)),
                sub_frame_work_command(sub_backdrop_clear_work_item()),
                sub_frame_work_command(sub_mode7_bg_work_item()),
                sub_frame_work_command(sub_sprite_work_item(0)),
                sub_frame_work_command(sub_sprite_work_item(1)),
                sub_frame_work_command(sub_sprite_work_item(2)),
                sub_frame_work_command(sub_sprite_work_item(3)),
                post_process_frame_work_command(),
            ]
        );
    }

    #[test]
    fn build_mode7_render_plan_skips_disabled_surfaces_without_skipping_clears() {
        let plan = build_mode7_render_plan(false, false, false);

        assert_eq!(
            frame_plan_commands(&plan),
            vec![
                main_frame_work_command(main_backdrop_clear_work_item()),
                main_frame_work_command(main_mode7_bg_work_item()),
                sub_frame_work_command(sub_backdrop_clear_work_item()),
                post_process_frame_work_command(),
            ]
        );
    }
}
