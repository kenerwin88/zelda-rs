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
fn subtile_shutter_handoff_keeps_the_host_boundary_link_scanout() {
    let landing = crate::game_state::FrameState {
        main_module: 7,
        submodule: 1,
        subsubmodule: 7,
        ..Default::default()
    };
    let shutter = crate::game_state::FrameState {
        main_module: 7,
        submodule: 5,
        ..Default::default()
    };

    assert_eq!(
        link_obj_scanout_across_main(
            landing,
            shutter,
            GraphicsDmaGeneration::LiveAfterMain,
            0,
        ),
        GraphicsDmaGeneration::HostBoundaryBeforeMain,
    );
    assert!(dungeon_subtile_landing_enters_shutter(landing, shutter));
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
fn dungeon_supertile_filter_entry_publishes_live_animated_tiles() {
    let filter_return = crate::game_state::FrameState {
        main_module: 7,
        submodule: 2,
        subsubmodule: 7,
        ..Default::default()
    };
    let first_scroll = crate::game_state::FrameState {
        subsubmodule: 8,
        ..filter_return
    };

    assert!(rom_dungeon_supertile_filter_entry_publishes_live_animated_bg(
        filter_return,
        first_scroll,
    ));
    assert!(!rom_dungeon_supertile_filter_entry_publishes_live_animated_bg(
        first_scroll,
        first_scroll,
    ));
    assert!(
        rom_dungeon_supertile_filter_return_resumes_first_scroll_after_nmi(
            filter_return,
            first_scroll,
            0x60,
        )
    );
    assert!(
        !rom_dungeon_supertile_filter_return_resumes_first_scroll_after_nmi(
            filter_return,
            first_scroll,
            0x71,
        )
    );
    assert!(rom_dungeon_supertile_scroll_runs_after_leading_nmi(
        first_scroll,
        0x60,
    ));
    assert!(!rom_dungeon_supertile_scroll_runs_after_leading_nmi(
        first_scroll,
        0x72,
    ));
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
fn room_71_subtile_shutter_publishes_only_the_live_link_head_cache() {
    let entry = crate::game_state::FrameState {
        main_module: 7,
        submodule: 1,
        subsubmodule: 7,
        ..Default::default()
    };
    let shutter = crate::game_state::FrameState {
        main_module: 7,
        submodule: 5,
        subsubmodule: 0,
        ..Default::default()
    };

    assert!(room_71_subtile_shutter_publishes_live_link_head_obj_cache(
        entry, shutter, 0x71, 5,
    ));
    assert!(!room_71_subtile_shutter_publishes_live_link_head_obj_cache(
        entry, shutter, 0x72, 5,
    ));
    assert!(!room_71_subtile_shutter_publishes_live_link_head_obj_cache(
        entry, shutter, 0x71, 6,
    ));
    assert_eq!(
        Room71ObjCachePublication::resolve(false, true),
        Room71ObjCachePublication::UseCapturedWithLiveLinkHead,
    );
    assert_eq!(
        Room71ObjCachePublication::resolve(true, false),
        Room71ObjCachePublication::UseLive,
    );
    assert_eq!(
        Room71ObjCachePublication::resolve(false, false),
        Room71ObjCachePublication::RetainCurrent,
    );

    let mut display = vec![0xaaaa; 0x8000];
    let mut resident = vec![0xbbbb; 0x8000];
    resident[0x4000..0x4020].fill(0x1000);
    resident[0x4020..0x4040].fill(0x2000);
    resident[0x4100..0x4120].fill(0x3000);
    resident[0x4120..0x4140].fill(0x4000);

    publish_live_link_head_obj_cache(&mut display, &resident);

    assert!(display[0x4000..0x4020].iter().all(|word| *word == 0xaaaa));
    assert!(display[0x4020..0x4040].iter().all(|word| *word == 0x2000));
    assert!(display[0x4100..0x4120].iter().all(|word| *word == 0xaaaa));
    assert!(display[0x4120..0x4140].iter().all(|word| *word == 0x4000));

    // Exercise the actual captured/resident roles after the display-state
    // swap. Distinct body and head markers make a reversed call direction
    // observable without a full-route replay.
    let mut state = ZeldaState::new();
    state.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    state.ram[crate::game_state::constants::SUBMODULE] = 1;
    state.ram[crate::game_state::constants::SUBSUBMODULE] = 7;
    state.ppu.vram[0x4000] = 0x1111;
    state.ppu.vram[0x4020] = 0x2222;

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 5;
    following.ram[crate::game_state::constants::SUBSUBMODULE] = 0;
    following.ram[crate::game_state::constants::DUNGEON_ROOM] = 0x71;
    write_le_u16(&mut following.ram, LINK_DMA_COUNTDOWN, 5);
    following.ppu.vram[0x4000] = 0xaaaa;
    following.ppu.vram[0x4020] = 0xbbbb;
    following.link_obj_scanout_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    following.link_obj_source_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    following.oam_scanout_source = OamScanoutSource::RetainResidentPpuOam;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    let cache = state.ppu.obj_vram_latch.as_ref().unwrap();
    assert_eq!(cache[0x4000], 0x1111, "captured Link body stays retained");
    assert_eq!(cache[0x4020], 0xbbbb, "resident Link head is decoded");
}

#[test]
fn dialogue_text_holds_published_oam_until_the_rom_frame_counter_advances() {
    let published = crate::game_state::FrameState {
        main_module: 14,
        submodule: 2,
        frame_counter: 5,
        ..Default::default()
    };
    let same_rom_tick = crate::game_state::FrameState {
        main_module: 14,
        submodule: 2,
        frame_counter: 5,
        ..Default::default()
    };
    let next_rom_tick = crate::game_state::FrameState {
        frame_counter: 6,
        ..same_rom_tick
    };

    assert!(dialogue_text_frame_holds_published_oam(
        published,
        same_rom_tick,
        3
    ));
    assert!(!dialogue_text_frame_holds_published_oam(
        published,
        next_rom_tick,
        3
    ));
    assert!(!dialogue_text_frame_holds_published_oam(
        published,
        same_rom_tick,
        4
    ));
}

#[test]
fn dungeon_landing_scanout_keeps_the_resident_oam_tail() {
    let mut state = ZeldaState::new();
    let landing_entries = [102, 116, 117, 118, 119];
    for (offset, entry) in landing_entries.into_iter().enumerate() {
        state.ppu.oam[entry * 2] = u16::from_le_bytes([0x30 + offset as u8, 0x50]);
        state.ppu.oam[entry * 2 + 1] = u16::from_le_bytes([0x0a, 0x2d]);
        let high_word = 256 + entry / 8;
        let high_shift = (entry % 8) * 2;
        state.ppu.oam[high_word] |= 0b10 << high_shift;
    }
    let resident_oam = state.ppu.oam.clone();

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 0x0f;
    following.ram[crate::game_state::constants::SUBSUBMODULE] = 1;
    following.ppu.oam[40] = u16::from_le_bytes([0x66, 0x77]);
    following.oam_scanout_source = OamScanoutSource::ComposeLiveAfterNmi;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[40].to_le_bytes(), [0x66, 0x77]);
    for entry in landing_entries {
        assert_eq!(
            state.ppu.oam[entry * 2..entry * 2 + 2],
            resident_oam[entry * 2..entry * 2 + 2]
        );
        let high_word = 256 + entry / 8;
        let high_shift = (entry % 8) * 2;
        assert_eq!(
            state.ppu.oam[high_word] & (0b11 << high_shift),
            resident_oam[high_word] & (0b11 << high_shift),
        );
    }
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
        DisplayVramGeneration::RetainCapturedBeforeNmi
    );
    assert_eq!(
        retained.hud_vram_generation,
        DisplayVramGeneration::RetainCapturedBeforeNmi
    );
    assert_eq!(
        retained.animated_bg_scanout_generation,
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
    );
    assert_eq!(
        retained.link_obj_scanout_generation,
        GraphicsDmaGeneration::LiveAfterMain
    );
    assert_eq!(
        retained.link_obj_source_generation,
        GraphicsDmaGeneration::LiveAfterMain
    );
    assert!(!state.next_display_interrupted_item_receipt_obj_cache);
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(atomic_item_graphics_return_obj_scanout(
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x14 },
        ))
    );
    assert!(state.publish_live_hud_vram_on_next_capture);
    assert_eq!(
        state.next_display_vram_generation,
        DisplayVramGeneration::RetainCapturedBeforeNmi
    );
    assert_eq!(
        state.next_display_animated_bg_scanout_generation,
        Some(AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi)
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
        DisplayVramGeneration::RetainCapturedBeforeNmi
    );
}

#[test]
fn room_71_item_graphics_return_crosses_completed_nmi_boundary() {
    let return_frame = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0,
        subsubmodule: 0,
        ..Default::default()
    };

    assert!(room_71_item_graphics_return_crosses_completed_nmi(
        return_frame,
        0x71,
        GraphicsDmaGeneration::HostBoundaryBeforeMain,
        OamScanoutSource::ComposePublishedShadowDma,
    ));
    assert!(!room_71_item_graphics_return_crosses_completed_nmi(
        return_frame,
        0x71,
        GraphicsDmaGeneration::LiveAfterMain,
        OamScanoutSource::ComposePublishedShadowDma,
    ));
}

#[test]
fn gfx_21_item_return_uses_the_v1_ordinary_module_epilogue() {
    let mut state = ZeldaState::new();
    let gfx_21 = ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x21 };

    assert!(state.item_receipt_graphics_return_uses_ordinary_module_epilogue(gfx_21));
    state.follower_link_state_mut().set_handler_state(21);
    assert!(!state.item_receipt_graphics_return_uses_ordinary_module_epilogue(gfx_21));
    state.follower_link_state_mut().set_handler_state(0);
    assert!(state.item_receipt_graphics_return_uses_ordinary_module_epilogue(gfx_21));

    for gfx in [0x14, 0x22, 0x24] {
        assert!(!state.item_receipt_graphics_return_uses_ordinary_module_epilogue(
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx },
        ));
    }
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
fn completed_gfx_24_return_publishes_live_animated_tiles() {
    let mut state = ZeldaState::new();
    state.capture_display_snapshot();
    let continuation = ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x24 };

    state.stage_atomic_item_graphics_return_obj_scanout(continuation);

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
    assert!(state.next_display_interrupted_item_receipt_obj_cache);
}

#[test]
fn uncle_item_return_stages_the_interrupted_receipt_cache_and_live_oam() {
    let mut state = ZeldaState::new();
    state.capture_display_snapshot();
    let continuation = ItemReceiptGraphicsContinuation::ResumeUnclePassage {
        receipt: ItemReceiptReturn {
            ancilla_slot: 4,
            item: 0,
            chest_position: 0,
        },
        sprite_slot: 0,
        dungeon: DungeonSpriteMainReturn {
            bg2_x: 1,
            bg2_y: 2,
            bg1_x: 3,
            bg1_y: 4,
        },
    };

    state.stage_atomic_item_graphics_return_obj_scanout(continuation);

    assert_eq!(
        state.display_snapshot.as_ref().unwrap().oam_scanout_source,
        OamScanoutSource::ComposeLiveAfterNmi
    );
    assert!(state.next_display_interrupted_item_receipt_obj_cache);
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(atomic_item_graphics_return_obj_scanout(continuation))
    );
    assert_eq!(
        state.next_display_obj_scanout_generation.unwrap().oam,
        OamScanoutSource::ComposeLiveAfterNmi
    );
}

#[test]
fn atomic_item_return_builds_the_measured_mixed_obj_tile_cache() {
    let mut state = ZeldaState::new();
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishItemReceiptGraphics {
            continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x14 },
        },
        ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES,
    );
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
fn subtile_shutter_handoff_decodes_live_obj_page_without_advancing_raw_vram() {
    let mut state = ZeldaState::new();
    state.ppu.vram[0x4032] = 0x1111;

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 5;
    following.ram[crate::game_state::constants::DUNGEON_ROOM] = 0x72;
    following.ppu.vram[0x4032] = 0xaaaa;
    following.link_obj_scanout_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    following.link_obj_source_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.vram[0x4032], 0x1111);
    assert_eq!(state.ppu.obj_vram_latch.as_ref().unwrap()[0x4032], 0xaaaa);

    let mut other_room = ZeldaState::new();
    following.ram[crate::game_state::constants::DUNGEON_ROOM] = 0x55;
    other_room.compose_display_oam(&following, &plan);
    assert!(other_room.ppu.obj_vram_latch.is_none());
}

#[test]
fn room_71_supertile_room_load_decodes_the_live_obj_page() {
    let mut state = ZeldaState::new();
    state.ppu.vram[0x4032] = 0x1111;
    state.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    state.ram[crate::game_state::constants::SUBMODULE] = 2;
    state.ram[crate::game_state::constants::SUBSUBMODULE] = 15;

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 5;
    following.ram[crate::game_state::constants::DUNGEON_ROOM] = 0x71;
    following.ppu.vram[0x4032] = 0xbbbb;
    following.link_obj_scanout_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    following.link_obj_source_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    following.oam_scanout_source = OamScanoutSource::RetainResidentPpuOam;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.vram[0x4032], 0x1111);
    assert_eq!(state.ppu.obj_vram_latch.as_ref().unwrap()[0x4032], 0xbbbb);
}

#[test]
fn room_71_subtile_room_load_uses_live_obj_only_after_the_visible_guard_boundary() {
    let mut state = ZeldaState::new();
    state.ppu.vram[0x4032] = 0x1111;
    state.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    state.ram[crate::game_state::constants::SUBMODULE] = 1;
    state.ram[crate::game_state::constants::SUBSUBMODULE] = 7;

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 5;
    following.ram[crate::game_state::constants::DUNGEON_ROOM] = 0x71;
    following.ppu.vram[0x4032] = 0xbbbb;
    following.link_obj_scanout_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    following.link_obj_source_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    following.oam_scanout_source = OamScanoutSource::RetainResidentPpuOam;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.vram[0x4032], 0x1111);
    assert!(state.ppu.obj_vram_latch.is_none());

    let mut workload = SpriteMainTimingWorkload::default();
    workload.record_active_sprite(0x41, 0);
    workload.record_blue_guard_full_animation();
    workload.record_garnish_table(false, 0);
    state.last_sprite_main_timing_workload = Some(workload);
    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.vram[0x4032], 0x1111);
    assert_eq!(state.ppu.obj_vram_latch.as_ref().unwrap()[0x4032], 0xbbbb);
}

#[test]
fn ordinary_dungeon_link_split_does_not_reuse_an_item_receipt_cache() {
    let mut state = ZeldaState::new();
    write_le_u16(&mut state.ram, LINK_DMA_COUNTDOWN, 2);
    state.ppu.obj_previous_frame_vram = Some(vec![0xaaaa; state.ppu.vram.len()]);
    state.ppu.oam[0] = 0x1111;
    state.ram[OAM_BUF..OAM_BUF + 2].copy_from_slice(&0x2222u16.to_le_bytes());

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 1;
    following.ram[crate::game_state::constants::SUBSUBMODULE] = 5;
    following.link_obj_scanout_generation = GraphicsDmaGeneration::LiveAfterMain;
    following.link_obj_source_generation = GraphicsDmaGeneration::LiveAfterMain;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert!(state.ppu.obj_vram_latch.is_none());
    assert_ne!(state.ppu.oam[0], 0x2222);
}

#[test]
fn subtile_palette_filter_decodes_captured_link_sources_without_mutating_raw_obj_vram() {
    let mut state = ZeldaState::new();
    state.ppu.vram[0x4000] = 0x3333;
    state.ppu.vram[0x4020] = 0x2222;
    state.link_obj_dma_completed_this_frame = true;
    write_le_u16(
        &mut state.ram,
        LinkDmaSourceSlot::HeadTop.ram_address(),
        0x8080,
    );

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 1;
    following.ram[crate::game_state::constants::SUBSUBMODULE] = 7;
    write_le_u16(
        &mut following.ram,
        LinkDmaSourceSlot::HeadTop.ram_address(),
        0x8100,
    );
    let mut link_graphics = vec![0; 0x200];
    for bytes in link_graphics[0x80..0xc0].chunks_exact_mut(2) {
        bytes.copy_from_slice(&0xaaaau16.to_le_bytes());
    }
    for bytes in link_graphics[0x100..0x140].chunks_exact_mut(2) {
        bytes.copy_from_slice(&0xbbbbu16.to_le_bytes());
    }
    let mut ranges = vec![(0, 0); 58];
    ranges[57] = (0, link_graphics.len());
    state.assets = Some(AssetPack::from_data_ranges(link_graphics, ranges));
    following.oam_scanout_source = OamScanoutSource::RetainResidentPpuOam;
    following.link_obj_scanout_generation = GraphicsDmaGeneration::LiveAfterMain;
    following.link_obj_source_generation = GraphicsDmaGeneration::LiveAfterMain;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.vram[0x4000], 0x3333);
    assert_eq!(state.ppu.vram[0x4020], 0x2222);
    assert_eq!(state.ppu.obj_vram_latch.as_ref().unwrap()[0x4020], 0xaaaa);
}

#[test]
fn held_subtile_nmi_reuses_the_last_presented_link_obj_cache() {
    let mut state = ZeldaState::new();
    state.ppu.vram[0x4020] = 0x2222;
    state.last_presented_obj_vram = Some(vec![0x7777; 0x400]);
    state.last_presented_obj_vram.as_mut().unwrap()[0x20] = 0xaaaa;
    state.link_obj_dma_completed_this_frame = false;

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 1;
    following.ram[crate::game_state::constants::SUBSUBMODULE] = 6;
    following.oam_scanout_source = OamScanoutSource::RetainResidentPpuOam;
    following.link_obj_scanout_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    following.link_obj_source_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.vram[0x4020], 0x2222);
    assert_eq!(state.ppu.obj_vram_latch.as_ref().unwrap()[0x4020], 0xaaaa);
}

#[test]
fn uncle_item_return_builds_the_same_mixed_cache_at_the_next_boundary() {
    let mut state = ZeldaState::new();
    write_le_u16(&mut state.ram, LINK_DMA_COUNTDOWN, 11);
    write_le_u16(
        &mut state.ram,
        LinkDmaSourceSlot::ShieldUpper.ram_address(),
        0x1000,
    );
    write_le_u16(
        &mut state.ram,
        LinkDmaSourceSlot::ShieldLower.ram_address(),
        0x1040,
    );
    for bytes in state.ram[0x1000..0x1040].chunks_exact_mut(2) {
        bytes.copy_from_slice(&0xaaaau16.to_le_bytes());
    }
    for bytes in state.ram[0x1040..0x1080].chunks_exact_mut(2) {
        bytes.copy_from_slice(&0xbbbbu16.to_le_bytes());
    }
    state.ppu.vram[0x4240] = 0x2222;
    state.ppu.vram[0x4250] = 0x3333;
    state.ppu.vram[0x4350] = 0x4444;

    let mut following = captured_display_snapshot();
    following.ppu.vram[0x4240] = 0xbbbb;
    following.ppu.vram[0x4250] = 0xcccc;
    following.ppu.vram[0x4350] = 0xdddd;
    following.link_obj_scanout_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    following.link_obj_source_generation = GraphicsDmaGeneration::LiveAfterMain;
    following.interrupted_item_receipt_obj_cache = true;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    let cache = state.ppu.obj_vram_latch.as_ref().unwrap();
    assert_eq!(cache[0x4070], 0xaaaa);
    assert_eq!(cache[0x4170], 0xbbbb);
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
fn room_72_second_state8_scroll_decodes_live_link_obj_without_advancing_raw_vram() {
    let mut state = ZeldaState::new();
    state.ppu.vram[0x4030] = 0x1111;
    state.set_screen_transition(1);

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 2;
    following.ram[crate::game_state::constants::SUBSUBMODULE] = 8;
    following.ram[crate::game_state::constants::FRAME_COUNTER] = 1;
    following.ram[crate::game_state::constants::DUNGEON_ROOM] = 0x72;
    write_le_u16(&mut following.ram, LINK_DMA_COUNTDOWN, 4);
    following.ppu.vram[0x4030] = 0xaaaa;
    following.oam_scanout_source = OamScanoutSource::ComposePublishedShadowDma;
    following.link_obj_scanout_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    following.link_obj_source_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.vram[0x4030], 0x1111);
    assert_eq!(state.ppu.obj_vram_latch.as_ref().unwrap()[0x4030], 0xaaaa);

    let mut first_scroll = ZeldaState::new();
    first_scroll.ppu.vram[0x4030] = 0x1111;
    first_scroll.set_screen_transition(1);
    following.ram[crate::game_state::constants::FRAME_COUNTER] = 0;
    write_le_u16(&mut following.ram, LINK_DMA_COUNTDOWN, 5);
    first_scroll.compose_display_oam(&following, &plan);
    assert!(first_scroll.ppu.obj_vram_latch.is_none());
}

#[test]
fn room_82_deferred_sprite_conversion_decodes_the_resident_obj_page() {
    let mut state = ZeldaState::new();
    state.ppu.vram[0x4020] = 0x1111;

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 2;
    following.ram[crate::game_state::constants::SUBSUBMODULE] = 3;
    following.ram[crate::game_state::constants::DUNGEON_ROOM] = 0x82;
    following.ppu.vram[0x4020] = 0xaaaa;
    following.room_82_sprite_conversion_deferred_nmi = true;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.vram[0x4020], 0x1111);
    assert_eq!(state.ppu.obj_vram_latch.as_ref().unwrap()[0x4020], 0x1111);

    let mut ordinary_state_3 = ZeldaState::new();
    ordinary_state_3.ppu.vram[0x4020] = 0x1111;
    following.room_82_sprite_conversion_deferred_nmi = false;
    ordinary_state_3.compose_display_oam(&following, &plan);
    assert_eq!(
        ordinary_state_3.ppu.obj_vram_latch.as_ref().unwrap()[0x4020],
        0xaaaa
    );
}

#[test]
fn room_82_horizontal_state3_boundaries_publish_entry_oam_after_cache_composition() {
    const LINK_BODY_WORD: usize = 102 * 2;
    const LINK_BODY_BYTE: usize = LINK_BODY_WORD * 2;

    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(2);
    state.set_screen_transition(2);
    let entry_frame = state.game_state.frame;
    let mut entry_oam_shadow = vec![0; state.ppu.oam.len() * 2];
    entry_oam_shadow[LINK_BODY_BYTE..LINK_BODY_BYTE + 2]
        .copy_from_slice(&0x1111u16.to_le_bytes());
    state.pre_main_graphics_dma = Some(PreMainGraphicsDma {
        entry_frame,
        entry_plan: rom_graphics_dma_plan_at_host_boundary(entry_frame),
        entry_dialogue_text_render_state: 0,
        entry_link_handler_state: 0,
        animated_tile: None,
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        link_obj_vram: state.ppu.vram[0x4000..0x4400].to_vec(),
        oam_shadow: entry_oam_shadow,
    });
    // Exercise the generic state-$03 cache branch that previously overwrote a
    // room-specific publication placed earlier in compose_display_oam.
    state.last_presented_obj_vram = Some(state.ppu.vram[0x4000..0x4400].to_vec());

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 2;
    following.ram[crate::game_state::constants::SUBSUBMODULE] = 3;
    following.ram[crate::game_state::constants::DUNGEON_ROOM] = 0x82;
    following.ppu.oam.fill(0x2222);
    following.published_shadow_oam_dma = Some(vec![0x3333; following.ppu.oam.len()]);
    following.oam_scanout_source = OamScanoutSource::ComposeLiveAfterNmi;
    following.link_obj_scanout_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    following.room_82_sprite_conversion_deferred_nmi = true;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[LINK_BODY_WORD], 0x1111);

    state.set_subsubmodule(3);
    let followup_entry = state.game_state.frame;
    let graphics = state.pre_main_graphics_dma.as_mut().unwrap();
    graphics.entry_frame = followup_entry;
    graphics.oam_shadow[LINK_BODY_BYTE..LINK_BODY_BYTE + 2]
        .copy_from_slice(&0x4444u16.to_le_bytes());
    state.ppu.oam.fill(0x5555);
    following.room_82_sprite_conversion_deferred_nmi = false;
    following.oam_scanout_source = OamScanoutSource::RetainResidentPpuOam;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[LINK_BODY_WORD], 0x4444);

    let graphics = state.pre_main_graphics_dma.as_mut().unwrap();
    graphics.oam_shadow[LINK_BODY_BYTE..LINK_BODY_BYTE + 2]
        .copy_from_slice(&0xf077u16.to_le_bytes());
    state.last_presented_oam = Some(vec![0x6666; state.ppu.oam.len()]);
    state.ppu.oam.fill(0x5555);

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[LINK_BODY_WORD], 0x6666);

    following.ram[crate::game_state::constants::SUBSUBMODULE] = 4;
    following.oam_scanout_source = OamScanoutSource::ComposeLiveAfterNmi;
    state.last_presented_oam = Some(vec![0x8888; state.ppu.oam.len()]);
    state.ppu.oam.fill(0x5555);
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[LINK_BODY_WORD], 0x8888);
}

#[test]
fn room_82_horizontal_quadrant_filter_entry_publishes_host_boundary_oam() {
    const LINK_LOWER_BODY_WORD: usize = 112 * 2;
    const LINK_LOWER_BODY_BYTE: usize = LINK_LOWER_BODY_WORD * 2;

    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(5);
    state.set_screen_transition(2);
    let entry_frame = state.game_state.frame;
    let mut host_boundary_oam = vec![0; state.ppu.oam.len() * 2];
    host_boundary_oam[LINK_LOWER_BODY_BYTE..LINK_LOWER_BODY_BYTE + 2]
        .copy_from_slice(&0x5edcu16.to_le_bytes());
    state.pre_main_graphics_dma = Some(PreMainGraphicsDma {
        entry_frame,
        entry_plan: rom_graphics_dma_plan_at_host_boundary(entry_frame),
        entry_dialogue_text_render_state: 0,
        entry_link_handler_state: 0,
        animated_tile: None,
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        link_obj_vram: state.ppu.vram[0x4000..0x4400].to_vec(),
        oam_shadow: host_boundary_oam,
    });
    state.ppu.oam[LINK_LOWER_BODY_WORD] = 0x5eda;
    state.last_presented_oam = Some(vec![0x5eda; state.ppu.oam.len()]);

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 2;
    following.ram[crate::game_state::constants::SUBSUBMODULE] = 6;
    following.ram[crate::game_state::constants::DUNGEON_ROOM] = 0x82;
    following.ppu.oam[LINK_LOWER_BODY_WORD] = 0x5edc;
    let mut published_shadow = vec![0; following.ppu.oam.len()];
    published_shadow[LINK_LOWER_BODY_WORD] = 0x5eda;
    following.published_shadow_oam_dma = Some(published_shadow);
    following.oam_scanout_source = OamScanoutSource::ComposePublishedShadowDma;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[LINK_LOWER_BODY_WORD], 0x5edc);
}

#[test]
fn room_82_horizontal_first_scroll_releases_stale_capture_to_host_boundary_oam() {
    const LINK_LOWER_BODY_WORD: usize = 112 * 2;
    const LINK_LOWER_BODY_BYTE: usize = LINK_LOWER_BODY_WORD * 2;

    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(7);
    state.set_screen_transition(2);
    let entry_frame = state.game_state.frame;
    let mut host_boundary_oam = vec![0; state.ppu.oam.len() * 2];
    host_boundary_oam[LINK_LOWER_BODY_BYTE..LINK_LOWER_BODY_BYTE + 2]
        .copy_from_slice(&0x5fdfu16.to_le_bytes());
    state.pre_main_graphics_dma = Some(PreMainGraphicsDma {
        entry_frame,
        entry_plan: rom_graphics_dma_plan_at_host_boundary(entry_frame),
        entry_dialogue_text_render_state: 0,
        entry_link_handler_state: 0,
        animated_tile: None,
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        link_obj_vram: state.ppu.vram[0x4000..0x4400].to_vec(),
        oam_shadow: host_boundary_oam,
    });
    state.ppu.oam[LINK_LOWER_BODY_WORD] = 0x5edd;
    state.last_presented_oam = Some(vec![0x5edc; state.ppu.oam.len()]);

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 2;
    following.ram[crate::game_state::constants::SUBSUBMODULE] = 8;
    following.ram[crate::game_state::constants::DUNGEON_ROOM] = 0x82;
    following.ppu.oam[LINK_LOWER_BODY_WORD] = 0x5fdf;
    let mut queued_capture = vec![0; following.ppu.oam.len()];
    queued_capture[LINK_LOWER_BODY_WORD] = 0x5edd;
    following.obj_generation = DisplayObjGeneration::RetainCapturedOam {
        oam: queued_capture,
    };
    let mut published_shadow = vec![0; following.ppu.oam.len()];
    published_shadow[LINK_LOWER_BODY_WORD] = 0x5eda;
    following.published_shadow_oam_dma = Some(published_shadow);
    following.oam_scanout_source = OamScanoutSource::ComposeLiveAfterNmi;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());
    assert!(plan.retain_captured_oam);

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[LINK_LOWER_BODY_WORD], 0x5fdf);
}

#[test]
fn room_82_horizontal_first_scroll_host_boundary_survives_final_publication() {
    const LINK_LOWER_BODY_WORD: usize = 112 * 2;
    const LINK_LOWER_BODY_BYTE: usize = LINK_LOWER_BODY_WORD * 2;

    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(7);
    state.set_dungeon_room_index(0x82);
    state.set_screen_transition(2);
    let entry_frame = state.game_state.frame;
    let mut host_boundary_oam = vec![0; state.ppu.oam.len() * 2];
    host_boundary_oam[LINK_LOWER_BODY_BYTE..LINK_LOWER_BODY_BYTE + 2]
        .copy_from_slice(&0x5fdfu16.to_le_bytes());
    state.pre_main_graphics_dma = Some(PreMainGraphicsDma {
        entry_frame,
        entry_plan: rom_graphics_dma_plan_at_host_boundary(entry_frame),
        entry_dialogue_text_render_state: 0,
        entry_link_handler_state: 0,
        animated_tile: None,
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        link_obj_vram: state.ppu.vram[0x4000..0x4400].to_vec(),
        oam_shadow: host_boundary_oam,
    });
    state.capture_display_snapshot();
    let snapshot = state.display_snapshot.as_mut().unwrap();
    snapshot.obj_generation = DisplayObjGeneration::RetainCapturedOam {
        oam: vec![0x5edd; snapshot.ppu.oam.len()],
    };
    snapshot.oam_scanout_source = OamScanoutSource::ComposeLiveAfterNmi;

    state.set_subsubmodule(8);
    state.ppu.oam[LINK_LOWER_BODY_WORD] = 0x5fdf;
    state.ram[OAM_BUF + LINK_LOWER_BODY_BYTE..OAM_BUF + LINK_LOWER_BODY_BYTE + 2]
        .copy_from_slice(&0x5fe0u16.to_le_bytes());

    let displayed = state.with_display_snapshot(|display| display.ppu.oam[LINK_LOWER_BODY_WORD]);

    assert_eq!(displayed, 0x5fdf);
}

#[test]
fn early_link_obj_cache_composes_body_head_and_hand_transfers_as_one_batch() {
    let mut ram = vec![0; 0x20000];
    let mut graphics = vec![0; EARLY_LINK_OBJ_DMA_TRANSFERS.len() * 0x80];
    let base_vram = vec![0x7777; 0x4400];

    for (index, (_, slot, len)) in EARLY_LINK_OBJ_DMA_TRANSFERS.iter().copied().enumerate() {
        let source_offset = index * 0x80;
        write_le_u16(
            &mut ram,
            slot.ram_address(),
            0x8000 + source_offset as u16,
        );
        let marker = 0x1100 + index as u16;
        for bytes in graphics[source_offset..source_offset + len].chunks_exact_mut(2) {
            bytes.copy_from_slice(&marker.to_le_bytes());
        }
    }

    let composed = compose_early_link_obj_cache(
        &base_vram,
        LinkDmaSources::load_from_ram(&ram),
        Some(&graphics),
    );

    for (index, (destination, _, len)) in EARLY_LINK_OBJ_DMA_TRANSFERS
        .iter()
        .copied()
        .enumerate()
    {
        assert!(composed[destination..destination + len / 2]
            .iter()
            .all(|&word| word == 0x1100 + index as u16));
    }
    assert_eq!(composed[0x4060], 0x7777);
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
    following.hud_vram_generation = DisplayVramGeneration::RetainCapturedBeforeNmi;
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
fn in_flight_obj_hold_prefers_the_retiring_scanout_then_falls_back_to_history() {
    let mut state = ZeldaState::new();
    let last_presented = vec![0x1111; state.ppu.oam.len()];
    let retiring = vec![0x2222; state.ppu.oam.len()];
    state.last_presented_oam = Some(last_presented.clone());
    state.staged_presented_oam = Some(retiring.clone());

    assert_eq!(
        state.retiring_or_last_presented_oam(),
        Some(retiring.as_slice()),
    );

    state.staged_presented_oam = None;
    assert_eq!(
        state.retiring_or_last_presented_oam(),
        Some(last_presented.as_slice()),
    );
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
fn enemy_drop_item_graphics_keep_the_resident_full_obj_cache() {
    let continuation = |gfx| GameWorkContinuation::FinishItemReceiptGraphics {
        continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx },
    };

    let mut ordinary_receipt = ZeldaState::new();
    ordinary_receipt
        .game_execution_scheduler
        .schedule_work(continuation(0x14), 1);
    assert!(ordinary_receipt.atomic_item_graphics_uses_partial_receipt_obj_cache());

    let mut enemy_drop = ZeldaState::new();
    enemy_drop
        .game_execution_scheduler
        .schedule_work(continuation(0x22), 1);
    assert!(!enemy_drop.atomic_item_graphics_uses_partial_receipt_obj_cache());
    assert!(enemy_drop.atomic_item_graphics_holds_following_nmi());
}

#[test]
fn enemy_drop_return_publishes_only_the_live_extended_oam_generation() {
    let mut state = ZeldaState::new();
    state.ppu.oam[0] = 0x1111;

    let mut following = captured_display_snapshot();
    following.ppu.oam[0] = 0x2222;
    following.ppu.oam[256] = 0x00a2;
    let mut published_shadow = following.ppu.oam.clone();
    published_shadow[256] = 0x00a0;
    following.published_shadow_oam_dma = Some(published_shadow);
    following.enemy_drop_item_graphics_live_extended_oam = true;
    following.oam_scanout_source = OamScanoutSource::ComposePublishedShadowDma;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[0], 0x2222);
    assert_eq!(state.ppu.oam[256], 0x00a2);

    let mut ordinary = ZeldaState::new();
    following.enemy_drop_item_graphics_live_extended_oam = false;
    ordinary.compose_display_oam(&following, &plan);
    assert_eq!(ordinary.ppu.oam[256], 0x00a0);
}

#[test]
fn enemy_drop_extended_oam_marker_survives_captures_until_presentation() {
    let mut state = ZeldaState::new();
    state.capture_display_snapshot();
    let continuation = ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x22 };

    state.stage_atomic_item_graphics_return_obj_scanout(continuation);
    assert!(
        state
            .display_snapshot
            .as_ref()
            .unwrap()
            .enemy_drop_item_graphics_live_extended_oam
    );

    state.capture_display_snapshot();
    assert!(
        state
            .display_snapshot
            .as_ref()
            .unwrap()
            .enemy_drop_item_graphics_live_extended_oam
    );

    state.capture_display_snapshot();
    assert!(
        state
            .display_snapshot
            .as_ref()
            .unwrap()
            .enemy_drop_item_graphics_live_extended_oam
    );

    {
        let display = state.display_snapshot.as_mut().unwrap();
        display.ppu.oam[256] = 0x00a2;
        let mut published_shadow = display.ppu.oam.clone();
        published_shadow[256] = 0x00a0;
        display.published_shadow_oam_dma = Some(published_shadow);
    }
    state.with_display_snapshot(|_| ());
    assert!(!state.enemy_drop_item_graphics_live_extended_oam_pending);

    state.capture_display_snapshot();
    assert!(
        !state
            .display_snapshot
            .as_ref()
            .unwrap()
            .enemy_drop_item_graphics_live_extended_oam
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
