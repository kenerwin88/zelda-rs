use crate::gpu_frame_work_command::{
    main_frame_work_command, post_process_frame_work_command, sub_frame_work_command,
    GpuFrameMainWorkCommand, GpuFrameRenderPlan, GpuFrameSubWorkCommand,
};

pub(crate) fn build_mode1_render_plan(
    has_main_bg: bool,
    has_main_sprites: bool,
    has_sub_bg: bool,
    has_sub_sprites: bool,
) -> GpuFrameRenderPlan {
    let mut render_plan = GpuFrameRenderPlan::default();
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
    render_plan.push(sub_frame_work_command(GpuFrameSubWorkCommand::BgLayer {
        layer_idx: 2,
        hi_priority: false,
    }));
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
    render_plan.push(sub_frame_work_command(GpuFrameSubWorkCommand::BgLayer {
        layer_idx: 1,
        hi_priority: false,
    }));
    render_plan.push(sub_frame_work_command(GpuFrameSubWorkCommand::BgLayer {
        layer_idx: 0,
        hi_priority: false,
    }));
    if has_sub_sprites && has_sub_bg {
        render_plan.push(sub_frame_work_command(
            GpuFrameSubWorkCommand::SpritePriority(2),
        ));
    }
    render_plan.push(sub_frame_work_command(GpuFrameSubWorkCommand::BgLayer {
        layer_idx: 1,
        hi_priority: true,
    }));
    render_plan.push(sub_frame_work_command(GpuFrameSubWorkCommand::BgLayer {
        layer_idx: 0,
        hi_priority: true,
    }));
    if has_sub_sprites && has_sub_bg {
        render_plan.push(sub_frame_work_command(
            GpuFrameSubWorkCommand::SpritePriority(3),
        ));
    }
    render_plan.push(sub_frame_work_command(GpuFrameSubWorkCommand::BgLayer {
        layer_idx: 2,
        hi_priority: true,
    }));

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

pub(crate) fn build_mode7_render_plan(
    has_main_sprites: bool,
    has_sub_mode7_bg: bool,
    has_sub_sprites: bool,
) -> GpuFrameRenderPlan {
    let mut render_plan = GpuFrameRenderPlan::default();
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
    render_plan.push(main_frame_work_command(GpuFrameMainWorkCommand::Mode7Bg));
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
        render_plan.push(sub_frame_work_command(GpuFrameSubWorkCommand::Mode7Bg));
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

    #[test]
    fn build_mode1_render_plan_preserves_full_gpu_draw_order() {
        let plan = build_mode1_render_plan(true, true, true, true);

        assert_eq!(
            plan.kinds(),
            vec![
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
                sub_frame_work_command(GpuFrameSubWorkCommand::BgLayer {
                    layer_idx: 2,
                    hi_priority: false,
                }),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(0)),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(1)),
                sub_frame_work_command(GpuFrameSubWorkCommand::BgLayer {
                    layer_idx: 1,
                    hi_priority: false,
                }),
                sub_frame_work_command(GpuFrameSubWorkCommand::BgLayer {
                    layer_idx: 0,
                    hi_priority: false,
                }),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(2)),
                sub_frame_work_command(GpuFrameSubWorkCommand::BgLayer {
                    layer_idx: 1,
                    hi_priority: true,
                }),
                sub_frame_work_command(GpuFrameSubWorkCommand::BgLayer {
                    layer_idx: 0,
                    hi_priority: true,
                }),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(3)),
                sub_frame_work_command(GpuFrameSubWorkCommand::BgLayer {
                    layer_idx: 2,
                    hi_priority: true,
                }),
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
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(0)),
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(1)),
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(2)),
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(3)),
                sub_frame_work_command(GpuFrameSubWorkCommand::ClearBackdrop),
                sub_frame_work_command(GpuFrameSubWorkCommand::BgLayer {
                    layer_idx: 2,
                    hi_priority: false,
                }),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(0)),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(1)),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(2)),
                sub_frame_work_command(GpuFrameSubWorkCommand::SpritePriority(3)),
                sub_frame_work_command(GpuFrameSubWorkCommand::BgLayer {
                    layer_idx: 1,
                    hi_priority: false,
                }),
                sub_frame_work_command(GpuFrameSubWorkCommand::BgLayer {
                    layer_idx: 0,
                    hi_priority: false,
                }),
                sub_frame_work_command(GpuFrameSubWorkCommand::BgLayer {
                    layer_idx: 1,
                    hi_priority: true,
                }),
                sub_frame_work_command(GpuFrameSubWorkCommand::BgLayer {
                    layer_idx: 0,
                    hi_priority: true,
                }),
                sub_frame_work_command(GpuFrameSubWorkCommand::BgLayer {
                    layer_idx: 2,
                    hi_priority: true,
                }),
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
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(0)),
                main_frame_work_command(GpuFrameMainWorkCommand::Mode7Bg),
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(1)),
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(2)),
                main_frame_work_command(GpuFrameMainWorkCommand::SpritePriority(3)),
                sub_frame_work_command(GpuFrameSubWorkCommand::ClearBackdrop),
                sub_frame_work_command(GpuFrameSubWorkCommand::Mode7Bg),
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
                main_frame_work_command(GpuFrameMainWorkCommand::Mode7Bg),
                sub_frame_work_command(GpuFrameSubWorkCommand::ClearBackdrop),
                post_process_frame_work_command(),
            ]
        );
    }
}
