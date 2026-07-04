use crate::gpu_frame::GpuFrame;
use crate::gpu_frame_work_command::{
    main_frame_work_command, post_process_frame_work_command, sub_frame_work_command,
    GpuFrameMainWorkCommand, GpuFrameRenderPlan, GpuFrameSubWorkCommand,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GpuFrameRenderPlanContext {
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
    pub(crate) fn from_frame(frame: &GpuFrame<'_>) -> Self {
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

    pub(crate) fn uses_sprites(&self) -> bool {
        match self {
            Self::Mode1 {
                has_main_sprites,
                has_sub_sprites,
                ..
            }
            | Self::Mode7 {
                has_main_sprites,
                has_sub_sprites,
                ..
            } => *has_main_sprites || *has_sub_sprites,
        }
    }

    pub(crate) fn render_plan(&self) -> GpuFrameRenderPlan {
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

fn build_mode1_render_plan(
    has_main_bg: bool,
    has_main_sprites: bool,
    has_sub_bg: bool,
    has_sub_sprites: bool,
) -> GpuFrameRenderPlan {
    let mut render_plan = GpuFrameRenderPlan::default();
    render_plan.push(main_frame_work_command(
        GpuFrameMainWorkCommand::ClearBackdrop,
    ));
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
                .map(GpuFrameMainWorkCommand::SpritePriority)
                .map(main_frame_work_command),
        );
    }

    if has_main_bg {
        // CPU Mode 1 z-order:
        //   BG3-lo, OBJ0, OBJ1, BG2-lo, BG1-lo, OBJ2,
        //   BG2-hi, BG1-hi, OBJ3, BG3-hi.
        render_plan.push(main_frame_work_command(main_bg_work_item(2, false, 2)));
        if has_main_sprites {
            render_plan.push(main_frame_work_command(
                GpuFrameMainWorkCommand::SpritePriority(0),
            ));
            render_plan.push(main_frame_work_command(
                GpuFrameMainWorkCommand::SpritePriority(1),
            ));
        }
        render_plan.push(main_frame_work_command(main_bg_work_item(1, false, 1)));
        render_plan.push(main_frame_work_command(main_bg_work_item(0, false, 0)));
        if has_main_sprites {
            render_plan.push(main_frame_work_command(
                GpuFrameMainWorkCommand::SpritePriority(2),
            ));
        }
        render_plan.push(main_frame_work_command(main_bg_work_item(1, true, 1)));
        render_plan.push(main_frame_work_command(main_bg_work_item(0, true, 0)));
        if has_main_sprites {
            render_plan.push(main_frame_work_command(
                GpuFrameMainWorkCommand::SpritePriority(3),
            ));
        }
        render_plan.push(main_frame_work_command(main_bg_work_item(2, true, 2)));
    }

    render_plan
}

fn build_mode1_sub_render_plan(has_sub_bg: bool, has_sub_sprites: bool) -> GpuFrameRenderPlan {
    let mut render_plan = GpuFrameRenderPlan::default();
    render_plan.push(sub_frame_work_command(
        GpuFrameSubWorkCommand::ClearBackdrop,
    ));
    render_plan.push(sub_frame_work_command(sub_bg_work_item(2, false)));
    if has_sub_sprites && !has_sub_bg {
        render_plan.extend(
            (0..=3)
                .map(GpuFrameSubWorkCommand::SpritePriority)
                .map(sub_frame_work_command),
        );
    }
    if has_sub_sprites && has_sub_bg {
        render_plan.push(sub_frame_work_command(
            GpuFrameSubWorkCommand::SpritePriority(0),
        ));
        render_plan.push(sub_frame_work_command(
            GpuFrameSubWorkCommand::SpritePriority(1),
        ));
    }
    render_plan.push(sub_frame_work_command(sub_bg_work_item(1, false)));
    render_plan.push(sub_frame_work_command(sub_bg_work_item(0, false)));
    if has_sub_sprites && has_sub_bg {
        render_plan.push(sub_frame_work_command(
            GpuFrameSubWorkCommand::SpritePriority(2),
        ));
    }
    render_plan.push(sub_frame_work_command(sub_bg_work_item(1, true)));
    render_plan.push(sub_frame_work_command(sub_bg_work_item(0, true)));
    if has_sub_sprites && has_sub_bg {
        render_plan.push(sub_frame_work_command(
            GpuFrameSubWorkCommand::SpritePriority(3),
        ));
    }
    render_plan.push(sub_frame_work_command(sub_bg_work_item(2, true)));

    render_plan
}

fn main_bg_work_item(
    layer_idx: usize,
    hi_priority: bool,
    math_bit_pos: u32,
) -> GpuFrameMainWorkCommand {
    GpuFrameMainWorkCommand::BgLayer {
        layer_idx,
        hi_priority,
        layer_bit: 1u32 << layer_idx,
        math_bit_pos,
    }
}

fn sub_bg_work_item(layer_idx: usize, hi_priority: bool) -> GpuFrameSubWorkCommand {
    GpuFrameSubWorkCommand::BgLayer {
        layer_idx,
        hi_priority,
        screen_layer_bit: 1u8 << layer_idx,
        render_layer_bit: 0, // skip per-scanline TM check for sub-screen
        math_bit_pos: 255,   // output alpha=1.0 (real pixel marker)
    }
}

fn main_mode7_bg_work_item() -> GpuFrameMainWorkCommand {
    GpuFrameMainWorkCommand::Mode7Bg {
        math_bit_pos: 0,
        layer_bit: 1,
    }
}

fn sub_mode7_bg_work_item() -> GpuFrameSubWorkCommand {
    GpuFrameSubWorkCommand::Mode7Bg {
        math_bit_pos: 255,
        layer_bit: 0,
    }
}

fn build_mode7_render_plan(
    has_main_sprites: bool,
    has_sub_mode7_bg: bool,
    has_sub_sprites: bool,
) -> GpuFrameRenderPlan {
    let mut render_plan = GpuFrameRenderPlan::default();
    render_plan.push(main_frame_work_command(
        GpuFrameMainWorkCommand::ClearBackdrop,
    ));
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
        render_plan.push(main_frame_work_command(
            GpuFrameMainWorkCommand::SpritePriority(0),
        ));
    }
    render_plan.push(main_frame_work_command(main_mode7_bg_work_item()));
    if has_main_sprites {
        render_plan.extend(
            (1..=3)
                .map(GpuFrameMainWorkCommand::SpritePriority)
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
    render_plan.push(sub_frame_work_command(
        GpuFrameSubWorkCommand::ClearBackdrop,
    ));
    if has_sub_mode7_bg {
        render_plan.push(sub_frame_work_command(sub_mode7_bg_work_item()));
    }
    if has_sub_sprites {
        render_plan.extend(
            (0..=3)
                .map(GpuFrameSubWorkCommand::SpritePriority)
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
        assert!(context.uses_sprites());
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
        assert!(context.uses_sprites());
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
        assert!(!context.uses_sprites());
    }

    #[test]
    fn sub_bg_work_item_carries_subscreen_render_metadata() {
        assert_eq!(
            sub_bg_work_item(2, true),
            GpuFrameSubWorkCommand::BgLayer {
                layer_idx: 2,
                hi_priority: true,
                screen_layer_bit: 0x04,
                render_layer_bit: 0,
                math_bit_pos: 255,
            }
        );
    }

    #[test]
    fn mode7_work_items_carry_render_metadata() {
        assert_eq!(
            main_mode7_bg_work_item(),
            GpuFrameMainWorkCommand::Mode7Bg {
                math_bit_pos: 0,
                layer_bit: 1,
            }
        );
        assert_eq!(
            sub_mode7_bg_work_item(),
            GpuFrameSubWorkCommand::Mode7Bg {
                math_bit_pos: 255,
                layer_bit: 0,
            }
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
                main_frame_work_command(GpuFrameMainWorkCommand::ClearBackdrop),
                main_frame_work_command(main_bg_work_item(2, false, 2)),
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(0)),
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(1)),
                main_frame_work_command(main_bg_work_item(1, false, 1)),
                main_frame_work_command(main_bg_work_item(0, false, 0)),
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(2)),
                main_frame_work_command(main_bg_work_item(1, true, 1)),
                main_frame_work_command(main_bg_work_item(0, true, 0)),
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(3)),
                main_frame_work_command(main_bg_work_item(2, true, 2)),
                sub_frame_work_command(GpuFrameSubWorkCommand::ClearBackdrop),
                sub_frame_work_command(sub_bg_work_item(2, false)),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(0)),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(1)),
                sub_frame_work_command(sub_bg_work_item(1, false)),
                sub_frame_work_command(sub_bg_work_item(0, false)),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(2)),
                sub_frame_work_command(sub_bg_work_item(1, true)),
                sub_frame_work_command(sub_bg_work_item(0, true)),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(3)),
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
                main_frame_work_command(GpuFrameMainWorkCommand::ClearBackdrop),
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(0)),
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(1)),
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(2)),
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(3)),
                sub_frame_work_command(GpuFrameSubWorkCommand::ClearBackdrop),
                sub_frame_work_command(sub_bg_work_item(2, false)),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(0)),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(1)),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(2)),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(3)),
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
                main_frame_work_command(GpuFrameMainWorkCommand::ClearBackdrop),
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(0)),
                main_frame_work_command(main_mode7_bg_work_item()),
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(1)),
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(2)),
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(3)),
                sub_frame_work_command(GpuFrameSubWorkCommand::ClearBackdrop),
                sub_frame_work_command(sub_mode7_bg_work_item()),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(0)),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(1)),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(2)),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(3)),
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
                main_frame_work_command(GpuFrameMainWorkCommand::ClearBackdrop),
                main_frame_work_command(main_mode7_bg_work_item()),
                sub_frame_work_command(GpuFrameSubWorkCommand::ClearBackdrop),
                post_process_frame_work_command(),
            ]
        );
    }
}
