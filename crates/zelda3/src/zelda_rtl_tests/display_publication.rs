use super::*;

fn captured_display_snapshot() -> DisplaySnapshot {
    let mut state = ZeldaState::new();
    state.capture_display_snapshot();
    *state.display_snapshot.take().unwrap()
}

#[test]
fn parity_diagnostics_stay_under_the_repository_target() {
    let path = parity_trace_path("display.trace");
    assert!(path.ends_with("target/parity-traces/display.trace"));
    assert!(!path.starts_with("/tmp"));
}

#[test]
fn item_receipt_dismissal_publishes_the_entry_oam_shadow() {
    let entry = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0,
        ..Default::default()
    };
    let exit = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0x0a,
        ..Default::default()
    };

    assert_eq!(
        oam_scanout_across_main(entry, exit, OamScanoutSource::ComposeLiveAfterNmi, 0),
        OamScanoutSource::ComposePublishedShadowDma
    );
}

#[test]
fn dungeon_item_hold_entry_selects_the_live_authored_oam_shadow() {
    let gameplay = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0,
        ..Default::default()
    };

    let item_hold_entry = is_dungeon_item_hold_entry(gameplay, gameplay, 0, 21);
    assert!(item_hold_entry);
    assert_eq!(
        oam_scanout_for_dungeon_item_hold_entry(
            OamScanoutSource::ComposeLiveAfterNmi,
            item_hold_entry,
        ),
        OamScanoutSource::ComposeLivePlayerOamAfterMain
    );
    let following_hold_frame = is_dungeon_item_hold_entry(gameplay, gameplay, 21, 21);
    assert!(!following_hold_frame);
    assert_eq!(
        oam_scanout_for_dungeon_item_hold_entry(
            OamScanoutSource::ComposeLiveAfterNmi,
            following_hold_frame,
        ),
        OamScanoutSource::ComposeLiveAfterNmi
    );
}

#[test]
fn dungeon_item_hold_dialogue_entry_publishes_live_animated_tiles() {
    let gameplay = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0,
        ..Default::default()
    };
    let dialogue = crate::game_state::FrameState {
        main_module: 14,
        submodule: 2,
        ..Default::default()
    };

    assert!(rom_dungeon_item_hold_to_dialogue_publishes_live_animated_bg(gameplay, dialogue,));
    assert!(!rom_dungeon_item_hold_to_dialogue_publishes_live_animated_bg(dialogue, dialogue,));
}

#[test]
fn dungeon_submodule_two_handoff_publishes_the_entry_oam_shadow() {
    let entry = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0,
        ..Default::default()
    };
    let exit = crate::game_state::FrameState {
        main_module: 7,
        submodule: 2,
        ..Default::default()
    };

    assert_eq!(
        oam_scanout_across_main(entry, exit, OamScanoutSource::ComposeLiveAfterNmi, 0),
        OamScanoutSource::ComposePublishedShadowDma
    );
}

#[test]
fn dungeon_submodule_five_handoff_publishes_the_entry_oam_shadow() {
    let entry = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0,
        ..Default::default()
    };
    let exit = crate::game_state::FrameState {
        main_module: 7,
        submodule: 5,
        ..Default::default()
    };

    assert_eq!(
        oam_scanout_across_main(entry, exit, OamScanoutSource::ComposeLiveAfterNmi, 0),
        OamScanoutSource::ComposePublishedShadowDma
    );
}

#[test]
fn dungeon_dialogue_entry_publishes_the_entry_oam_shadow() {
    let entry = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0,
        frame_counter: 0x42,
        ..Default::default()
    };
    for submodule in [1, 2] {
        let exit = crate::game_state::FrameState {
            main_module: 0x0e,
            submodule,
            frame_counter: 0x43,
            ..Default::default()
        };

        assert_eq!(
            oam_operands_across_main(entry, exit, GraphicsDmaGeneration::LiveAfterMain,),
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        );
        assert_eq!(
            oam_scanout_across_main(entry, exit, OamScanoutSource::ComposeLiveAfterNmi, 0),
            if submodule == 1 {
                OamScanoutSource::ComposePublishedShadowDma
            } else {
                OamScanoutSource::ComposeLiveAfterNmi
            }
        );

        let later_initializer_slice = crate::game_state::FrameState {
            frame_counter: 0x44,
            ..exit
        };
        assert_eq!(
            oam_operands_across_main(
                entry,
                later_initializer_slice,
                GraphicsDmaGeneration::LiveAfterMain,
            ),
            GraphicsDmaGeneration::LiveAfterMain
        );
    }
}

#[test]
fn spiral_stair_entry_uses_host_link_operands_and_entry_oam_shadow() {
    let plan = rom_graphics_dma_plan(7, 0x0e);
    assert_eq!(plan.link_obj_operands, GraphicsDmaGeneration::LiveAfterMain);
    assert_eq!(plan.oam_scanout, OamScanoutSource::RetainCapturedBeforeNmi);

    let entry = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0,
        ..Default::default()
    };
    let exit = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0x0e,
        ..Default::default()
    };
    assert_eq!(
        link_obj_operands_across_main(entry, exit, plan.link_obj_operands),
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(
        oam_scanout_across_main(entry, exit, OamScanoutSource::ComposeLiveAfterNmi, 0),
        OamScanoutSource::ComposePublishedShadowDma
    );

    assert_eq!(
        link_obj_operands_across_main(exit, exit, plan.link_obj_operands),
        GraphicsDmaGeneration::LiveAfterMain
    );

    let steady_animation = crate::game_state::FrameState {
        subsubmodule: 2,
        ..exit
    };
    assert_eq!(
        link_obj_operands_across_main(exit, steady_animation, plan.link_obj_operands),
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
}

#[test]
fn publication_plan_keeps_memory_domains_independent() {
    let mut snapshot = captured_display_snapshot();
    snapshot.vram_generation = DisplayVramGeneration::RetainCapturedBeforeNmi;
    snapshot.oam_scanout_source = OamScanoutSource::ComposePublishedShadowDma;
    snapshot.link_obj_scanout_generation = GraphicsDmaGeneration::LiveAfterMain;
    snapshot.animated_bg_scanout_generation = AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi;
    snapshot.bg_scroll_generation = DisplayBgScrollGeneration::RetainCapturedBeforeNmi;

    let plan = DisplayPublicationPlan::resolve(
        &snapshot,
        DisplayPublicationSignals {
            publish_live_overworld_transition_half_color: true,
            ..DisplayPublicationSignals::default()
        },
    );

    assert_eq!(
        plan.vram_generation,
        DisplayVramGeneration::RetainCapturedBeforeNmi
    );
    assert!(plan.compose_live_cgram);
    assert_eq!(
        plan.oam_scanout_source,
        OamScanoutSource::ComposePublishedShadowDma
    );
    assert_eq!(
        plan.link_obj_scanout_generation,
        GraphicsDmaGeneration::LiveAfterMain
    );
    assert_eq!(
        plan.animated_bg_scanout_generation,
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
    );
    assert_eq!(
        plan.bg_scroll_source,
        DisplayedBgScrollSource::CapturedBeforeNmi
    );
    assert!(plan.publish_live_overworld_transition_half_color);
}

#[test]
fn dungeon_item_hold_entry_publishes_its_live_camera_scroll() {
    let gameplay = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0,
        ..Default::default()
    };
    assert!(!dungeon_item_hold_publishes_live_scroll(gameplay, 21, true,));
    assert!(dungeon_item_hold_publishes_live_scroll(gameplay, 21, false,));

    let mut snapshot = captured_display_snapshot();
    snapshot.bg_scroll_generation = DisplayBgScrollGeneration::RetainCapturedBeforeNmi;

    let plan = DisplayPublicationPlan::resolve(
        &snapshot,
        DisplayPublicationSignals {
            dungeon_item_hold_publishes_live_scroll: true,
            ..DisplayPublicationSignals::default()
        },
    );

    assert_eq!(plan.bg_scroll_source, DisplayedBgScrollSource::LiveAfterNmi);
}

#[test]
fn explicit_resident_oam_overrides_the_generic_module_defer_rule() {
    let mut snapshot = captured_display_snapshot();
    snapshot.oam_scanout_source = OamScanoutSource::RetainResidentPpuOam;

    let plan = DisplayPublicationPlan::resolve(
        &snapshot,
        DisplayPublicationSignals {
            module_oam_publication_is_deferred: true,
            ..DisplayPublicationSignals::default()
        },
    );

    assert_eq!(
        plan.oam_scanout_source,
        OamScanoutSource::RetainResidentPpuOam
    );
}

#[test]
fn retained_oam_scanout_uses_the_previously_published_shadow_dma() {
    let mut state = ZeldaState::new();
    state.ppu.oam[40] = u16::from_le_bytes([0x38, 0x34]);

    let mut following = captured_display_snapshot();
    let mut published_shadow = following.ppu.oam.clone();
    published_shadow[40] = u16::from_le_bytes([0x38, 0xf0]);
    following.published_shadow_oam_dma = Some(published_shadow);
    following.oam_scanout_source = OamScanoutSource::RetainCapturedBeforeNmi;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[40].to_le_bytes(), [0x38, 0xf0]);
}

#[test]
fn resident_ppu_oam_scanout_ignores_the_following_published_shadow() {
    let mut state = ZeldaState::new();
    state.ppu.oam[40] = u16::from_le_bytes([0x38, 0x3f]);

    let mut following = captured_display_snapshot();
    following.ppu.oam[40] = u16::from_le_bytes([0x38, 0x41]);
    let mut published_shadow = following.ppu.oam.clone();
    published_shadow[40] = u16::from_le_bytes([0x38, 0x41]);
    following.published_shadow_oam_dma = Some(published_shadow);
    following.oam_scanout_source = OamScanoutSource::RetainResidentPpuOam;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[40].to_le_bytes(), [0x38, 0x3f]);
}

#[test]
fn live_shadow_scanout_reads_the_oam_authored_by_main() {
    let mut state = ZeldaState::new();
    state.ppu.bg_layer[1].h_scroll = 1;
    state.ppu.oam[0] = u16::from_le_bytes([0x11, 0x22]);
    state.ppu.oam[24] = u16::from_le_bytes([0xfa, 0x30]);

    let mut following = captured_display_snapshot();
    following.ppu.oam[0] = u16::from_le_bytes([0x11, 0x22]);
    following.ppu.oam[24] = u16::from_le_bytes([0xfa, 0x30]);
    following.ram[OAM_BUF..OAM_BUF + 2].copy_from_slice(&[0x33, 0x44]);
    following.ram[OAM_BUF + 48..OAM_BUF + 50].copy_from_slice(&[0xf7, 0x30]);
    write_le_u16(&mut following.ram, BG2_X_SCROLL, 1);
    following.oam_scanout_source = OamScanoutSource::ComposeLivePlayerOamAfterMain;
    following.dungeon_item_hold_entry_bg2_scroll = Some((0, 0));
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[0].to_le_bytes(), [0x11, 0x22]);
    assert_eq!(state.ppu.oam[24].to_le_bytes(), [0xf8, 0x30]);
}

#[test]
fn atomic_item_return_marks_the_retained_and_following_link_generations() {
    let mut state = ZeldaState::new();
    state.capture_display_snapshot();

    state.stage_atomic_item_graphics_return_obj_scanout(
        ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x14 },
    );

    let retained = state.display_snapshot.as_ref().unwrap();
    assert_eq!(
        retained.oam_scanout_source,
        OamScanoutSource::ComposePublishedShadowDma
    );
    assert_eq!(
        retained.vram_generation,
        DisplayVramGeneration::ComposeLiveAfterNmi
    );
    assert_eq!(
        retained.animated_bg_scanout_generation,
        AnimatedBgScanoutGeneration::LiveAfterNmi
    );
    assert_eq!(
        retained.link_obj_scanout_generation,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(
        retained.link_obj_source_generation,
        GraphicsDmaGeneration::LiveAfterMain
    );
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(atomic_item_graphics_return_obj_scanout(
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x14 },
        ))
    );

    let plan = DisplayPublicationPlan::resolve(
        retained,
        DisplayPublicationSignals {
            retain_previous_nmi_display_memory: true,
            ..DisplayPublicationSignals::default()
        },
    );
    assert_eq!(
        plan.vram_generation,
        DisplayVramGeneration::ComposeLiveAfterNmi
    );
}

#[test]
fn uncle_sword_item_return_stages_the_prepared_oam_for_the_next_boundary() {
    let mut state = ZeldaState::new();
    state.capture_display_snapshot();
    state.ram[OAM_BUF..OAM_BUF + 4].copy_from_slice(&[0x11, 0x22, 0x33, 0x44]);

    state.stage_atomic_item_graphics_return_obj_scanout(
        ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x24 },
    );

    assert_eq!(
        state.display_snapshot.as_ref().unwrap().oam_scanout_source,
        OamScanoutSource::ComposePublishedShadowDma
    );
    assert_eq!(
        state.next_display_obj_memory_generation,
        Some(DisplayObjGeneration::RetainCapturedOam {
            oam: {
                let mut oam = vec![0; state.ppu.oam.len()];
                oam[0] = 0x2211;
                oam[1] = 0x4433;
                oam
            },
        })
    );
}

#[test]
fn atomic_item_return_builds_the_measured_mixed_obj_tile_cache() {
    let mut state = ZeldaState::new();
    write_le_u16(&mut state.ram, LINK_DMA_COUNTDOWN, 2);
    state.ppu.vram[0x4020] = 0x1111;
    state.ppu.vram[0x4240] = 0x2222;
    state.ppu.vram[0x4250] = 0x3333;
    state.ppu.vram[0x4350] = 0x4444;

    let mut following = captured_display_snapshot();
    following.ppu.vram[0x4020] = 0xaaaa;
    following.ppu.vram[0x4240] = 0xbbbb;
    following.ppu.vram[0x4250] = 0xcccc;
    following.ppu.vram[0x4350] = 0xdddd;
    following.link_obj_scanout_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    following.link_obj_source_generation = GraphicsDmaGeneration::LiveAfterMain;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    let cache = state.ppu.obj_vram_latch.as_ref().unwrap();
    assert_eq!(state.ppu.vram[0x4020], 0x1111);
    assert_eq!(cache[0x4020], 0xaaaa);
    assert_eq!(cache[0x4240], 0xbbbb);
    assert_eq!(cache[0x4250], 0);
    assert_eq!(cache[0x4350], 0);
}

#[test]
fn first_supertile_scroll_retains_raw_obj_vram_but_decodes_the_live_cache() {
    let mut state = ZeldaState::new();
    state.ppu.vram[0x4020] = 0x1111;

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 2;
    following.ram[crate::game_state::constants::SUBSUBMODULE] = 1;
    following.ppu.vram[0x4020] = 0xaaaa;
    following.oam_scanout_source = OamScanoutSource::RetainResidentPpuOam;
    following.link_obj_scanout_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    following.link_obj_source_generation = GraphicsDmaGeneration::LiveAfterMain;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.vram[0x4020], 0x1111);
    assert_eq!(state.ppu.obj_vram_latch.as_ref().unwrap()[0x4020], 0xaaaa);
}

#[test]
fn completed_room_load_can_publish_live_animated_bg_independently() {
    let mut state = ZeldaState::new();
    state.capture_display_snapshot();
    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .animated_bg_scanout_generation = AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi;

    state.stage_live_animated_bg_scanout();

    assert_eq!(
        state
            .display_snapshot
            .as_ref()
            .unwrap()
            .animated_bg_scanout_generation,
        AnimatedBgScanoutGeneration::LiveAfterNmi
    );
    assert_eq!(
        state.next_display_animated_bg_scanout_generation,
        Some(AnimatedBgScanoutGeneration::LiveAfterNmi)
    );
}

#[test]
fn live_animated_bg_composes_over_retained_general_vram() {
    let mut state = ZeldaState::new();
    let destination = 0x3b00;
    state.set_animated_tile_vram_destination_address(destination as u16);
    state.ppu.vram[0] = 0x1111;
    state.ppu.vram[destination] = 0x2222;

    let mut following = captured_display_snapshot();
    write_le_u16(
        &mut following.ram,
        ANIMATED_TILE_VRAM_ADDR,
        destination as u16,
    );
    following.ppu.vram[0] = 0xaaaa;
    following.ppu.vram[destination] = 0xbbbb;
    following.vram_generation = DisplayVramGeneration::RetainCapturedBeforeNmi;
    following.animated_bg_scanout_generation = AnimatedBgScanoutGeneration::LiveAfterNmi;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_vram(&following, &plan, None);

    assert_eq!(state.ppu.vram[0], 0x1111);
    assert_eq!(state.ppu.vram[destination], 0xbbbb);
}

#[test]
fn dungeon_exit_publication_plan_promotes_only_live_boundary_domains() {
    let mut snapshot = captured_display_snapshot();
    snapshot.oam_scanout_source = OamScanoutSource::RetainCapturedBeforeNmi;
    snapshot.link_obj_scanout_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    snapshot.bg_scroll_generation = DisplayBgScrollGeneration::RetainCapturedBeforeNmi;

    let plan = DisplayPublicationPlan::resolve(
        &snapshot,
        DisplayPublicationSignals {
            dungeon_exit_crosses_nmi_boundary: true,
            ..DisplayPublicationSignals::default()
        },
    );

    assert_eq!(
        plan.oam_scanout_source,
        OamScanoutSource::ComposeLiveAfterNmi
    );
    assert_eq!(
        plan.link_obj_scanout_generation,
        GraphicsDmaGeneration::LiveAfterMain
    );
    assert_eq!(plan.bg_scroll_source, DisplayedBgScrollSource::LiveAfterNmi);
}

#[test]
fn dungeon_exit_entry_detects_both_crossed_nmi_boundaries() {
    assert!(!rom_dungeon_exit_entry_crosses_nmi_boundary(
        0x0f, 0, 0x0f, 0, false
    ));
    assert!(rom_dungeon_exit_entry_crosses_nmi_boundary(
        0x0f, 0, 0x0f, 0, true
    ));
    assert!(rom_dungeon_exit_entry_crosses_nmi_boundary(
        0x0f, 0, 0x0f, 1, false
    ));
    assert!(!rom_dungeon_exit_entry_crosses_nmi_boundary(
        7, 0, 0x0f, 0, true
    ));
    assert!(!rom_dungeon_exit_entry_crosses_nmi_boundary(
        0x0f, 1, 0x0f, 1, true
    ));
}

#[test]
fn attract_memory_retention_still_publishes_the_live_scroll_origin() {
    let mut snapshot = captured_display_snapshot();
    snapshot.vram_generation = DisplayVramGeneration::ComposeLiveAfterNmi;
    snapshot.bg_scroll_generation = DisplayBgScrollGeneration::RetainCapturedBeforeNmi;

    let plan = DisplayPublicationPlan::resolve(
        &snapshot,
        DisplayPublicationSignals {
            retain_previous_nmi_display_memory: true,
            attract_map_retains_display_memory: true,
            world_map_fade_display: true,
            world_map_mode7_brightness_is_early_published: true,
            ..DisplayPublicationSignals::default()
        },
    );

    assert_eq!(
        plan.vram_generation,
        DisplayVramGeneration::RetainCapturedBeforeNmi
    );
    assert!(!plan.compose_live_cgram);
    assert_eq!(plan.bg_scroll_source, DisplayedBgScrollSource::LiveAfterNmi);
    assert!(plan.world_map_fade_display);
    assert!(plan.world_map_mode7_brightness_is_early_published);
}

#[test]
fn bad_weather_publication_promotes_bg1_and_animated_tiles_together() {
    let mut snapshot = captured_display_snapshot();
    snapshot.bg_scroll_generation = DisplayBgScrollGeneration::RetainCapturedBeforeNmi;
    snapshot.animated_bg_scanout_generation = AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi;

    let plan = DisplayPublicationPlan::resolve(
        &snapshot,
        DisplayPublicationSignals {
            publish_live_overworld_bad_weather_scroll: true,
            ..DisplayPublicationSignals::default()
        },
    );

    assert_eq!(
        plan.bg_scroll_source,
        DisplayedBgScrollSource::LiveBg1AfterNmi
    );
    assert_eq!(
        plan.animated_bg_scanout_generation,
        AnimatedBgScanoutGeneration::LiveAfterNmi
    );
}

#[test]
fn post_nmi_bg_scroll_writes_target_the_following_scanout() {
    let mut state = ZeldaState::new();
    let current_scanout = [
        [0x0111, 0x0122],
        [0x0233, 0x0244],
        [0x0355, 0x0366],
        [0x0077, 0x0088],
    ];
    for (layer, [h_scroll, v_scroll]) in state.ppu.bg_layer.iter_mut().zip(current_scanout) {
        layer.h_scroll = h_scroll;
        layer.v_scroll = v_scroll;
    }
    state.capture_display_snapshot();

    let following_scanout = [
        [0x1111, 0x1122],
        [0x1233, 0x1244],
        [0x1355, 0x1366],
        [0x1077, 0x1088],
    ];
    state.publish_bg_scroll_for_following_scanout(BgScrollRegisterScanout {
        offsets: following_scanout,
    });

    let displayed = state.with_display_snapshot(|display| {
        display
            .ppu
            .bg_layer
            .map(|layer| [layer.h_scroll, layer.v_scroll])
    });
    assert_eq!(displayed, current_scanout);

    state.capture_display_snapshot();
    let displayed = state.with_display_snapshot(|display| {
        display
            .ppu
            .bg_layer
            .map(|layer| [layer.h_scroll, layer.v_scroll])
    });
    assert_eq!(displayed, following_scanout);
}

#[test]
fn display_snapshot_consumes_vram_once_and_retains_active_obj_generation() {
    let mut state = ZeldaState::new();
    let held_oam = vec![0x1234; state.ppu.oam.len()];
    let held_obj_vram = vec![0x5678; 0x400];
    state.next_display_vram_generation = DisplayVramGeneration::RetainCapturedBeforeNmi;
    state.next_display_bg_scroll_generation = DisplayBgScrollGeneration::ComposeLiveAfterNmi;
    state.next_display_obj_scanout_generation = Some(ObjScanoutGenerations::coherent(
        GraphicsDmaGeneration::HostBoundaryBeforeMain,
    ));
    state.active_display_obj_generation = DisplayObjGeneration::RetainCapturedMemory {
        oam: held_oam.clone(),
        vram: held_obj_vram.clone(),
    };

    state.capture_display_snapshot();

    let snapshot = state.display_snapshot.as_ref().expect("display snapshot");
    assert_eq!(
        snapshot.vram_generation,
        DisplayVramGeneration::RetainCapturedBeforeNmi,
    );
    assert_eq!(
        snapshot.bg_scroll_generation,
        DisplayBgScrollGeneration::ComposeLiveAfterNmi,
    );
    assert_eq!(
        snapshot.oam_scanout_source,
        OamScanoutSource::RetainCapturedBeforeNmi,
    );
    assert_eq!(
        snapshot.link_obj_scanout_generation,
        GraphicsDmaGeneration::HostBoundaryBeforeMain,
    );
    assert_eq!(
        snapshot.obj_generation,
        DisplayObjGeneration::RetainCapturedMemory {
            oam: held_oam.clone(),
            vram: held_obj_vram.clone(),
        },
    );
    assert_eq!(
        state.next_display_vram_generation,
        DisplayVramGeneration::ComposeLiveAfterNmi,
    );
    assert_eq!(
        state.next_display_bg_scroll_generation,
        DisplayBgScrollGeneration::RetainCapturedBeforeNmi,
    );
    assert_eq!(state.next_display_obj_scanout_generation, None);
    assert_eq!(
        state.active_display_obj_generation,
        DisplayObjGeneration::RetainCapturedMemory {
            oam: held_oam.clone(),
            vram: held_obj_vram.clone(),
        },
    );

    state.capture_display_snapshot();
    assert_eq!(
        state
            .display_snapshot
            .as_ref()
            .expect("second display snapshot")
            .obj_generation,
        DisplayObjGeneration::RetainCapturedMemory {
            oam: held_oam,
            vram: held_obj_vram,
        },
    );
}

#[test]
fn presented_vram_generation_combines_snapshot_and_domain_retention_once() {
    assert_eq!(
        DisplayVramGeneration::ComposeLiveAfterNmi.resolve_for_scanout(false),
        DisplayVramGeneration::ComposeLiveAfterNmi,
    );
    assert_eq!(
        DisplayVramGeneration::ComposeLiveAfterNmi.resolve_for_scanout(true),
        DisplayVramGeneration::RetainCapturedBeforeNmi,
    );
    assert_eq!(
        DisplayVramGeneration::RetainCapturedBeforeNmi.resolve_for_scanout(false),
        DisplayVramGeneration::RetainCapturedBeforeNmi,
    );
}

#[test]
fn interface_exit_stripe_upload_belongs_to_the_following_scanout() {
    // Ordinary dialogue and the save menu share this interface-exit boundary;
    // the destination module does not own the stripe until the next scanout.
    assert!(interface_exit_bg_upload_misses_current_scanout(
        0x0e, 9, true,
    ));
    assert!(interface_exit_bg_upload_misses_current_scanout(
        0x0e, 23, true,
    ));
    assert!(!interface_exit_bg_upload_misses_current_scanout(
        0x0e, 0x0e, true,
    ));
    assert!(!interface_exit_bg_upload_misses_current_scanout(9, 9, true));
    assert!(!interface_exit_bg_upload_misses_current_scanout(
        0x0e, 9, false,
    ));
}

#[test]
fn pre_main_nmi_resume_selects_display_domains_by_hardware_generation() {
    assert_eq!(
        PreMainNmiResume::OverworldAuxGraphicsReturn.scanout_generations(),
        PreMainNmiScanoutGenerations {
            publication: DisplaySnapshotPublication::PublishCaptured,
            vram: DisplayVramGeneration::ComposeLiveAfterNmi,
            animated_bg: None,
            bg_scroll: DisplayBgScrollGeneration::ComposeLiveAfterNmi,
            obj: None,
        },
    );
    for (return_phase, publication, bg_scroll) in [
        (
            NmiPhase::BeforeNmi,
            DisplaySnapshotPublication::PublishCaptured,
            DisplayBgScrollGeneration::ComposeLiveAfterNmi,
        ),
        (
            NmiPhase::AfterNmi,
            DisplaySnapshotPublication::RetainPublished,
            DisplayBgScrollGeneration::RetainCapturedBeforeNmi,
        ),
    ] {
        assert_eq!(
            PreMainNmiResume::OverworldSpriteReloadReturn {
                scanout: OverworldSpriteReloadResumeScanout::ByReturnPhase(return_phase),
            }
            .scanout_generations(),
            PreMainNmiScanoutGenerations {
                publication,
                vram: DisplayVramGeneration::RetainCapturedBeforeNmi,
                animated_bg: None,
                bg_scroll,
                obj: None,
            },
        );
    }
    assert_eq!(
        PreMainNmiResume::OverworldSpriteReloadReturn {
            scanout: OverworldSpriteReloadResumeScanout::CpuSliceEntry {
                scroll: BgScrollRegisterScanout {
                    offsets: [[0x1111, 0x2222]; 4],
                },
                bg1_generation: OverworldSpriteReloadBg1Generation::ComposeAtTransitionReturn,
            },
        }
        .scanout_generations(),
        PreMainNmiScanoutGenerations {
            publication: DisplaySnapshotPublication::PublishCaptured,
            vram: DisplayVramGeneration::ComposeLiveAfterNmi,
            animated_bg: None,
            bg_scroll: DisplayBgScrollGeneration::RetainCpuSliceEntry(BgScrollRegisterScanout {
                offsets: [[0x1111, 0x2222]; 4],
            }),
            obj: None,
        },
    );
    assert_eq!(
        PreMainNmiResume::DungeonSupertileQuadrantUploads.scanout_generations(),
        PreMainNmiScanoutGenerations {
            publication: DisplaySnapshotPublication::PublishCaptured,
            vram: DisplayVramGeneration::ComposeLiveAfterNmi,
            animated_bg: Some(AnimatedBgScanoutGeneration::LiveAfterNmi),
            bg_scroll: DisplayBgScrollGeneration::ComposeLiveAfterNmi,
            obj: Some(ObjScanoutGenerations {
                oam: OamScanoutSource::ComposeLiveAfterNmi,
                link_obj: GraphicsDmaGeneration::LiveAfterMain,
                link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
            }),
        },
    );
    let dungeon_phase = PreMainNmiResume::DungeonSupertileQuadrantUploads;
    for subsubmodule in 5..=15 {
        assert!(
            dungeon_phase.continues_after_main(crate::game_state::FrameState {
                main_module: 7,
                submodule: 2,
                subsubmodule,
                ..Default::default()
            })
        );
    }
    for subsubmodule in 8..=15 {
        assert!(
            dungeon_phase.continues_after_main(crate::game_state::FrameState {
                main_module: 7,
                submodule: 0x0e,
                subsubmodule,
                ..Default::default()
            })
        );
    }
    assert!(
        !dungeon_phase.continues_after_main(crate::game_state::FrameState {
            main_module: 7,
            submodule: 0,
            subsubmodule: 0,
            ..Default::default()
        })
    );
}
