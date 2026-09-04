use super::*;

#[test]
fn interrupted_palette_filter_continuation_preserves_its_cpu_caller() {
    let spiral = InterruptedPaletteFilterCaller::from_dungeon_submodule(0x0e);
    let straight = InterruptedPaletteFilterCaller::from_dungeon_submodule(0x12);

    assert_eq!(spiral, InterruptedPaletteFilterCaller::SpiralStairs);
    assert!(spiral.requeues_core_dma_after_nmi());
    assert_eq!(
        straight,
        InterruptedPaletteFilterCaller::StraightInterroomStairs
    );
    assert!(!straight.requeues_core_dma_after_nmi());
}

#[test]
fn staircase_34_gameplay_handoffs_decode_the_early_host_link_cache_batch() {
    let gameplay = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0,
        ..Default::default()
    };
    let spiral = crate::game_state::FrameState {
        main_module: 7,
        submodule: 4,
        ..Default::default()
    };
    let supertile = crate::game_state::FrameState {
        main_module: 7,
        submodule: 2,
        ..Default::default()
    };
    let host = GraphicsDmaGeneration::HostBoundaryBeforeMain;

    assert!(
        staircase_34_gameplay_handoff_decodes_early_host_link_obj_cache(
            gameplay, spiral, 0x34, host, host,
        )
    );
    assert!(
        staircase_34_gameplay_handoff_decodes_early_host_link_obj_cache(
            gameplay, supertile, 0x34, host, host,
        )
    );
    assert!(
        !staircase_34_gameplay_handoff_decodes_early_host_link_obj_cache(
            gameplay, supertile, 0x30, host, host,
        )
    );
    assert!(
        !staircase_34_gameplay_handoff_decodes_early_host_link_obj_cache(
            gameplay,
            crate::game_state::FrameState {
                main_module: 7,
                submodule: 3,
                ..Default::default()
            },
            0x34,
            host,
            host,
        )
    );
    assert!(
        !staircase_34_gameplay_handoff_decodes_early_host_link_obj_cache(
            gameplay,
            spiral,
            0x34,
            GraphicsDmaGeneration::LiveAfterMain,
            host,
        )
    );
}

#[test]
fn room_82_staircase_30_gameplay_handoff_selects_only_the_live_obj_cache() {
    let gameplay = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0,
        ..Default::default()
    };
    let supertile = crate::game_state::FrameState {
        main_module: 7,
        submodule: 2,
        ..Default::default()
    };
    let host = GraphicsDmaGeneration::HostBoundaryBeforeMain;

    assert!(room_82_staircase_30_gameplay_handoff_uses_live_obj_cache(
        gameplay, supertile, 0x82, 0x30, host, host,
    ));
    assert!(!room_82_staircase_30_gameplay_handoff_uses_live_obj_cache(
        gameplay, supertile, 0x81, 0x30, host, host,
    ));
    assert!(!room_82_staircase_30_gameplay_handoff_uses_live_obj_cache(
        gameplay, supertile, 0x82, 0x34, host, host,
    ));
    assert!(!room_82_staircase_30_gameplay_handoff_uses_live_obj_cache(
        gameplay,
        supertile,
        0x82,
        0x30,
        GraphicsDmaGeneration::LiveAfterMain,
        host,
    ));
}

fn captured_display_snapshot() -> DisplaySnapshot {
    let mut state = ZeldaState::new();
    state.capture_display_snapshot();
    *state.display_snapshot.take().unwrap()
}

#[test]
fn straight_interroom_fadeout_advances_only_its_live_decoded_obj_cache() {
    let fade = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0x12,
        subsubmodule: 1,
        ..Default::default()
    };
    assert!(straight_interroom_fadeout_uses_live_decoded_obj_cache(
        fade, 0x51, 0x30,
    ));
    let completed_main = crate::game_state::FrameState {
        frame_counter: 1,
        ..fade
    };
    assert!(straight_interroom_fadeout_main_slice_publishes_host_oam(
        fade,
        completed_main,
        0x51,
        0x30,
        OamScanoutSource::RetainResidentPpuOam,
    ));
    assert!(!straight_interroom_fadeout_main_slice_publishes_host_oam(
        fade,
        fade,
        0x51,
        0x30,
        OamScanoutSource::RetainResidentPpuOam,
    ));
    assert!(
        straight_interroom_fadeout_no_nmi_caller_retains_presented_display(
            fade,
            fade,
            0x51,
            0x30,
            0,
            GraphicsDmaGeneration::LiveAfterMain,
        )
    );
    assert!(
        !straight_interroom_fadeout_no_nmi_caller_retains_presented_display(
            fade,
            completed_main,
            0x51,
            0x30,
            1,
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
        )
    );
    assert!(straight_interroom_palette_filter_retains_presented_oam(
        crate::game_state::FrameState {
            subsubmodule: 5,
            ..fade
        },
        0x51,
        0x30,
    ));
    assert!(!straight_interroom_palette_filter_retains_presented_oam(
        crate::game_state::FrameState {
            subsubmodule: 7,
            ..fade
        },
        0x51,
        0x30,
    ));
    for subsubmodule in 6..=8 {
        assert!(straight_interroom_palette_filter_retains_captured_oam(
            crate::game_state::FrameState {
                subsubmodule,
                ..fade
            },
            0x51,
            0x30,
        ));
    }
    for subsubmodule in 0x0d..=0x0f {
        assert!(
            straight_interroom_post_sprite_graphics_uses_host_link_obj_cache(
                crate::game_state::FrameState {
                    subsubmodule,
                    ..fade
                },
                0x51,
                0x30,
            )
        );
    }
    let palette_caller = crate::game_state::FrameState {
        subsubmodule: 0x0f,
        frame_counter: 0x42,
        ..fade
    };
    assert!(straight_interroom_palette_caller_retains_presented_display(
        palette_caller,
        palette_caller,
        0x51,
        0x30,
    ));
    assert!(
        !straight_interroom_palette_caller_retains_presented_display(
            palette_caller,
            crate::game_state::FrameState {
                frame_counter: 0x43,
                ..palette_caller
            },
            0x51,
            0x30,
        )
    );
    assert!(
        straight_interroom_palette_completion_retains_presented_cgram(
            palette_caller,
            crate::game_state::FrameState {
                subsubmodule: 0x10,
                ..palette_caller
            },
            0x51,
            0x30,
        )
    );
    assert!(
        !straight_interroom_palette_completion_retains_presented_cgram(
            crate::game_state::FrameState {
                subsubmodule: 0x0e,
                ..palette_caller
            },
            crate::game_state::FrameState {
                subsubmodule: 0x10,
                ..palette_caller
            },
            0x51,
            0x30,
        )
    );
    for (frame, room, stairs) in [
        (
            crate::game_state::FrameState {
                subsubmodule: 0,
                ..fade
            },
            0x51,
            0x30,
        ),
        (fade, 0x50, 0x30),
        (fade, 0x51, 0x31),
    ] {
        assert!(!straight_interroom_fadeout_uses_live_decoded_obj_cache(
            frame, room, stairs,
        ));
    }
    for entry_subsubmodule in 0x0c..=0x0e {
        assert!(straight_interroom_quadrant_pipeline_publishes_host_oam(
            crate::game_state::FrameState {
                subsubmodule: entry_subsubmodule,
                ..fade
            },
            crate::game_state::FrameState {
                subsubmodule: entry_subsubmodule + 1,
                ..fade
            },
            0x51,
            0x30,
        ));
    }
}

#[test]
fn straight_interroom_entry_publishes_link_after_main_but_keeps_entry_oam() {
    let gameplay = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0,
        ..Default::default()
    };
    let straight_stair_entry = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0x12,
        subsubmodule: 0,
        ..Default::default()
    };

    assert_eq!(
        oam_scanout_across_main(
            gameplay,
            straight_stair_entry,
            OamScanoutSource::ComposeLiveAfterNmi,
        ),
        OamScanoutSource::ComposePublishedShadowDma,
    );
    assert_eq!(
        link_obj_scanout_across_main(
            gameplay,
            straight_stair_entry,
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
        ),
        GraphicsDmaGeneration::LiveAfterMain,
    );
}

#[test]
fn palette_filter_input_cgram_capture_is_explicit_and_one_shot() {
    let mut state = ZeldaState::new();
    state.ram[MAIN_PALETTE_BUFFER..MAIN_PALETTE_BUFFER + 4]
        .copy_from_slice(&[0x11, 0x11, 0x22, 0x22]);
    state.retain_palette_filter_input_cgram_on_next_display_capture();
    state.ram[MAIN_PALETTE_BUFFER..MAIN_PALETTE_BUFFER + 4]
        .copy_from_slice(&[0x33, 0x33, 0x44, 0x44]);
    state.capture_display_snapshot();
    assert_eq!(
        state
            .display_snapshot
            .as_ref()
            .unwrap()
            .cgram_scanout_override
            .as_ref()
            .unwrap()[..2],
        [0x1111, 0x2222],
    );

    state.capture_display_snapshot();
    assert!(state
        .display_snapshot
        .as_ref()
        .unwrap()
        .cgram_scanout_override
        .is_none(),);
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
        oam_scanout_across_main(entry, exit, OamScanoutSource::ComposeLiveAfterNmi),
        OamScanoutSource::ComposePublishedShadowDma
    );
}

#[test]
fn dungeon_game_over_entry_publishes_the_entry_oam_shadow() {
    let gameplay = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0,
        frame_counter: 0x46,
        ..Default::default()
    };
    let death = crate::game_state::FrameState {
        main_module: 0x12,
        submodule: 1,
        frame_counter: 0x47,
        ..Default::default()
    };

    assert_eq!(
        oam_scanout_across_main(gameplay, death, OamScanoutSource::ComposeLiveAfterNmi),
        OamScanoutSource::ComposePublishedShadowDma,
    );
}

#[test]
fn game_over_pre_iris_entry_publishes_the_entry_oam_shadow() {
    let death_initializer = crate::game_state::FrameState {
        main_module: 0x12,
        submodule: 1,
        frame_counter: 0x47,
        ..Default::default()
    };
    let pre_iris_delay = crate::game_state::FrameState {
        main_module: 0x12,
        submodule: 2,
        frame_counter: 0x48,
        ..Default::default()
    };

    assert_eq!(
        oam_scanout_across_main(
            death_initializer,
            pre_iris_delay,
            OamScanoutSource::ComposeLiveAfterNmi,
        ),
        OamScanoutSource::ComposePublishedShadowDma,
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
        link_obj_scanout_across_main(landing, shutter, GraphicsDmaGeneration::LiveAfterMain),
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
fn dungeon_supertile_filter_entry_keeps_animated_tiles_at_the_host_boundary() {
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

    // C's Graphics_IncrementalVRAMUpload only authors the source/destination
    // operands consumed by the following NMI. The state-7 palette call then
    // advances to state 8 without performing a PPU write itself.
    assert_eq!(
        animated_bg_scanout_across_main(
            rom_graphics_dma_plan_at_host_boundary(filter_return),
            rom_graphics_dma_plan_at_host_boundary(first_scroll),
        ),
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi,
    );
    assert!(rom_dungeon_module_iteration_runs_after_leading_nmi(
        first_scroll,
        0x60,
    ));
    assert!(!rom_dungeon_module_iteration_runs_after_leading_nmi(
        first_scroll,
        0x72,
    ));
    let straight_quadrant_pipeline = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0x12,
        ..Default::default()
    };
    for subsubmodule in 0x0b..=0x0f {
        assert!(straight_interroom_upload_pipeline_runs_after_leading_nmi(
            crate::game_state::FrameState {
                subsubmodule,
                ..straight_quadrant_pipeline
            },
            true,
        ));
    }
    assert!(!straight_interroom_upload_pipeline_runs_after_leading_nmi(
        crate::game_state::FrameState {
            subsubmodule: 0x10,
            ..straight_quadrant_pipeline
        },
        true,
    ));
    assert!(!straight_interroom_upload_pipeline_runs_after_leading_nmi(
        crate::game_state::FrameState {
            subsubmodule: 0x0b,
            ..straight_quadrant_pipeline
        },
        false,
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
        oam_scanout_across_main(entry, exit, OamScanoutSource::ComposeLiveAfterNmi),
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
        oam_scanout_across_main(entry, exit, OamScanoutSource::ComposeLiveAfterNmi),
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
            oam_scanout_across_main(entry, exit, OamScanoutSource::ComposeLiveAfterNmi),
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
        oam_scanout_across_main(entry, exit, OamScanoutSource::ComposeLiveAfterNmi),
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
fn completed_pre_main_palette_filter_publishes_only_cgram_after_nmi() {
    let in_progress = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0x0e,
        subsubmodule: 2,
        ..Default::default()
    };
    let completed = crate::game_state::FrameState {
        subsubmodule: 0x0f,
        ..in_progress
    };
    assert!(!rom_spiral_second_palette_return_publishes_live_cgram(
        in_progress,
        1,
        0x30,
    ));
    assert!(rom_spiral_second_palette_return_publishes_live_cgram(
        completed, 1, 0x30,
    ));
    assert!(!rom_spiral_second_palette_return_publishes_live_cgram(
        completed, 0x72, 0x34,
    ));

    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_subsubmodule(0x0f);
    state.ppu.cgram[35] = 0x0022;
    state.ppu.oam[107 * 2] = u16::from_le_bytes([137, 46]);
    state.capture_display_snapshot();

    let captured_oam_source = state.display_snapshot.as_ref().unwrap().oam_scanout_source;
    state.publish_completed_palette_filter_cgram_scanout();
    let snapshot = state.display_snapshot.as_ref().unwrap();
    assert_eq!(
        snapshot.cgram_scanout_generation,
        CgramScanoutGeneration::LiveAfterNmi
    );
    assert_eq!(snapshot.oam_scanout_source, captured_oam_source);

    state.cgram_upload_latch = Some(state.ppu.cgram.to_vec());
    state.ppu.cgram[35] = 0x0443;
    state.ppu.oam[107 * 2] = u16::from_le_bytes([138, 44]);
    let presented =
        state.with_display_snapshot(|display| (display.ppu.cgram[35], display.ppu.oam[107 * 2]));

    assert_eq!(presented.0, 0x0443);
    assert_eq!(presented.1, u16::from_le_bytes([137, 46]));
}

#[test]
fn noncanonical_pre_main_palette_filter_retains_previous_cgram_without_upload() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_subsubmodule(0x0f);
    state.ppu.cgram[35] = 0x0421;
    state.capture_display_snapshot();
    state.retain_completed_palette_filter_cgram_scanout();

    state.ppu.cgram[35] = 0x0842;
    let presented = state.with_display_snapshot(|display| display.ppu.cgram[35]);

    assert_eq!(presented, 0x0421);
    assert_eq!(state.ppu.cgram[35], 0x0842);
}

#[test]
fn retained_spiral_palette_uses_the_previous_presented_cgram_not_the_capture() {
    let mut state = ZeldaState::new();
    let mut previous = state.ppu.cgram.clone();
    previous[35] = 0x0210;
    state.last_presented_cgram = Some(previous);
    state.ppu.cgram[35] = 0x0421;
    state.capture_display_snapshot();
    state.retain_completed_palette_filter_cgram_scanout();

    state.ppu.cgram[35] = 0x0842;
    let presented = state.with_display_snapshot(|display| display.ppu.cgram[35]);

    assert_eq!(presented, 0x0210);
    assert_eq!(state.ppu.cgram[35], 0x0842);
}

#[test]
fn entry_scoped_spiral_cgram_retention_survives_canonical_completion_state() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.ppu.cgram[35] = 0x0421;
    state.capture_display_snapshot();
    state.retain_completed_palette_filter_cgram_scanout();

    state.set_dungeon_room_index(1);
    state.dungeon_stair_movement_mut().set_staircase_index(0x30);
    state.publish_completed_palette_filter_cgram_scanout();
    state.publish_completed_spiral_palette_filter_scanout();

    assert_eq!(
        state
            .display_snapshot
            .as_ref()
            .unwrap()
            .cgram_scanout_generation,
        CgramScanoutGeneration::RetainPreviousPresented
    );
}

#[test]
fn noncanonical_spiral_palette_slice_retention_follows_the_rom_frame_boundary() {
    let entry = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0x0e,
        subsubmodule: 0x0f,
        frame_counter: 0x69,
        ..Default::default()
    };
    let held = entry;
    let terminal = crate::game_state::FrameState {
        subsubmodule: 0x10,
        ..entry
    };

    assert!(!rom_spiral_palette_slice_retains_previous_cgram(
        entry, held, 0x72, 0x34, 0x1b,
    ));
    assert!(rom_spiral_palette_slice_retains_previous_cgram(
        entry, terminal, 0x72, 0x34, 0,
    ));
    assert!(!rom_spiral_palette_slice_retains_previous_cgram(
        entry,
        crate::game_state::FrameState {
            frame_counter: 0x6a,
            ..held
        },
        0x72,
        0x34,
        0x1b,
    ));
    assert!(!rom_spiral_palette_slice_retains_previous_cgram(
        entry, held, 1, 0x30, 0x18,
    ));
    assert!(!rom_spiral_palette_slice_retains_previous_cgram(
        entry,
        crate::game_state::FrameState {
            frame_counter: 0x6a,
            ..held
        },
        0x72,
        0x34,
        0x18,
    ));
    assert!(rom_spiral_palette_slice_retains_previous_cgram(
        entry, held, 0x72, 0x34, 0x17,
    ));
    assert!(!rom_spiral_palette_slice_retains_previous_cgram(
        entry, held, 0x72, 0x34, 0x18,
    ));
}

#[test]
fn completed_spiral_palette_filter_publishes_its_cgram_and_oam_after_nmi() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_dungeon_room_index(1);
    state.dungeon_stair_movement_mut().set_staircase_index(0x30);
    state.ppu.cgram[35] = 0x0000;
    state.ppu.oam[107 * 2] = u16::from_le_bytes([137, 46]);
    state.capture_display_snapshot();

    let retained_domains = {
        let snapshot = state.display_snapshot.as_ref().unwrap();
        (
            snapshot.vram_generation,
            snapshot.animated_bg_scanout_generation,
            snapshot.link_obj_scanout_generation,
        )
    };
    state.publish_completed_spiral_palette_filter_scanout();
    let snapshot = state.display_snapshot.as_ref().unwrap();
    assert_eq!(
        snapshot.cgram_scanout_generation,
        CgramScanoutGeneration::LiveAfterNmi
    );
    assert_eq!(
        snapshot.oam_scanout_source,
        OamScanoutSource::ComposeCompletedWorkAfterNmi
    );
    assert_eq!(
        (
            snapshot.vram_generation,
            snapshot.animated_bg_scanout_generation,
            snapshot.link_obj_scanout_generation,
        ),
        retained_domains,
        "the boundary override must not advance unrelated graphics domains"
    );

    state.cgram_upload_latch = Some(state.ppu.cgram.to_vec());
    state.ppu.cgram[35] = 0x0421;
    state.ppu.oam[107 * 2] = u16::from_le_bytes([138, 44]);
    let presented =
        state.with_display_snapshot(|display| (display.ppu.cgram[35], display.ppu.oam[107 * 2]));

    assert_eq!(presented, (0x0421, u16::from_le_bytes([138, 44])));
    assert_eq!(state.ppu.cgram[35], 0x0421);
    assert_eq!(state.ppu.oam[107 * 2], u16::from_le_bytes([138, 44]));
}

#[test]
fn noncanonical_spiral_completion_retains_captured_cgram_and_oam() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_dungeon_room_index(0x72);
    state.dungeon_stair_movement_mut().set_staircase_index(0x34);
    state.ppu.cgram[35] = 0x0421;
    state.ppu.oam[107 * 2] = u16::from_le_bytes([99, 79]);
    state.capture_display_snapshot();
    state.publish_completed_spiral_palette_filter_scanout();

    state.ppu.cgram[35] = 0x0842;
    state.ppu.oam[107 * 2] = u16::from_le_bytes([99, 80]);
    let presented =
        state.with_display_snapshot(|display| (display.ppu.cgram[35], display.ppu.oam[107 * 2]));

    assert_eq!(presented, (0x0421, u16::from_le_bytes([99, 79])));
}

#[test]
fn spiral_stair_landing_publishes_live_screen_layers_and_oam() {
    let landing = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0x0e,
        subsubmodule: 0x11,
        ..Default::default()
    };
    assert!(!rom_spiral_stair_landing_publishes_live_display(
        landing, 1, 0x30, 0
    ));
    assert!(rom_spiral_stair_landing_publishes_live_display(
        landing, 1, 0x30, 1
    ));
    assert!(!rom_spiral_stair_landing_publishes_live_display(
        landing, 0x72, 0x34, 1
    ));
    assert!(!rom_spiral_stair_landing_publishes_live_display(
        crate::game_state::FrameState {
            subsubmodule: 0x12,
            ..landing
        },
        1,
        0x30,
        1,
    ));

    let mut state = ZeldaState::new();
    state.set_main_module(landing.main_module);
    state.set_submodule(landing.submodule);
    state.set_subsubmodule(landing.subsubmodule);
    state.set_dungeon_room_index(1);
    state.dungeon_stair_movement_mut().set_staircase_index(0x30);
    state.follower_link_state_mut().set_y_button_action_step(1);
    state.set_main_screen_layers(0x06);
    state.set_sub_screen_layers(0x11);
    state.ppu.screen_enabled = [0x16, 0x01];
    state.ppu.oam[107 * 2] = u16::from_le_bytes([140, 240]);
    state.capture_display_snapshot();

    state.ppu.screen_enabled = [0x06, 0x11];
    state.ppu.oam[107 * 2] = u16::from_le_bytes([116, 81]);
    let presented = state
        .with_display_snapshot(|display| (display.ppu.screen_enabled, display.ppu.oam[107 * 2]));

    assert_eq!(presented.0, [0x06, 0x11]);
    assert_eq!(presented.1, u16::from_le_bytes([116, 81]));
}

#[test]
fn spiral_stair_motion_publishes_only_live_oam() {
    let motion = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0x0e,
        subsubmodule: 0x12,
        ..Default::default()
    };
    assert!(rom_spiral_stair_motion_publishes_live_oam(motion, 1, 0x30));
    assert!(rom_spiral_stair_motion_publishes_live_oam(
        crate::game_state::FrameState {
            subsubmodule: 0x13,
            ..motion
        },
        1,
        0x30,
    ));
    assert!(!rom_spiral_stair_motion_publishes_live_oam(
        motion, 0x72, 0x34
    ));
    assert!(!rom_spiral_stair_motion_publishes_live_oam(
        crate::game_state::FrameState {
            subsubmodule: 0x14,
            ..motion
        },
        1,
        0x30,
    ));
    let plan = DisplayPublicationPlan::resolve(
        &captured_display_snapshot(),
        DisplayPublicationSignals {
            spiral_stair_motion_publishes_live_oam: true,
            ..DisplayPublicationSignals::default()
        },
    );
    assert_eq!(
        plan.oam_scanout_source,
        OamScanoutSource::ComposeCompletedWorkAfterNmi
    );
    assert!(plan.publish_live_spiral_stair_obj_cache);
    assert!(!plan.publish_live_spiral_stair_return_obj_vram);
    assert!(!plan.publish_spiral_stair_return_equipment_handoff);
    assert!(!plan.publish_live_screen_layers);

    let mut state = ZeldaState::new();
    state.set_main_module(motion.main_module);
    state.set_submodule(motion.submodule);
    state.set_subsubmodule(motion.subsubmodule);
    state.set_dungeon_room_index(1);
    state.dungeon_stair_movement_mut().set_staircase_index(0x30);
    state.follower_link_state_mut().set_y_button_action_step(1);
    state.ppu.screen_enabled = [0x16, 0x01];
    state.ppu.oam[102 * 2] = u16::from_le_bytes([119, 78]);
    state.capture_display_snapshot();

    state.ppu.screen_enabled = [0x06, 0x11];
    state.ppu.oam[102 * 2] = u16::from_le_bytes([119, 76]);
    let presented = state
        .with_display_snapshot(|display| (display.ppu.screen_enabled, display.ppu.oam[102 * 2]));

    assert_eq!(presented.0, [0x16, 0x01]);
    assert_eq!(presented.1, u16::from_le_bytes([119, 76]));
}

#[test]
fn dungeon_brightness_publishes_live_screen_layers_and_post_main_hud_dma() {
    const DESTINATION: usize = 0x6040;
    const MAGIC_METER_WORD: usize = 131;
    let brightness = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0x0a,
        ..Default::default()
    };
    assert!(dungeon_brightness_screen_layers_are_live(
        brightness, brightness
    ));
    assert!(!dungeon_brightness_screen_layers_are_live(
        crate::game_state::FrameState {
            main_module: 7,
            submodule: 0,
            ..Default::default()
        },
        brightness,
    ));

    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0x0a);
    state.set_message_dma_destination_address(DESTINATION as u16);
    state.set_sub_screen_layers(1);
    state.ppu.screen_enabled = [0x16, 0x01];
    state.set_hud_tile_word(MAGIC_METER_WORD, 0x3c4e);
    state.ppu.vram[DESTINATION + MAGIC_METER_WORD] = 0x3c4e;
    state.capture_display_snapshot();

    // The torch/brightness suffix has crossed vblank and cleared TS in live
    // state, while the general display snapshot still owns the pre-NMI value.
    state.set_sub_screen_layers(0);
    state.ppu.screen_enabled = [0x16, 0x00];
    state.set_hud_tile_word(MAGIC_METER_WORD, 0x3c4d);
    assert_eq!(state.ppu.vram[DESTINATION + MAGIC_METER_WORD], 0x3c4e);

    let presented = state.with_display_snapshot(|display| {
        (
            display.ppu.screen_enabled,
            display.ppu.vram[DESTINATION + MAGIC_METER_WORD],
        )
    });

    assert_eq!(presented, ([0x16, 0x00], 0x3c4d));
}

#[test]
fn spiral_stair_return_publishes_split_oam_and_live_obj_across_repeated_captures() {
    let plan = DisplayPublicationPlan::resolve(
        &captured_display_snapshot(),
        DisplayPublicationSignals {
            spiral_stair_return_publishes_live_shadow_oam: true,
            spiral_stair_return_publishes_live_registers: true,
            ..DisplayPublicationSignals::default()
        },
    );
    assert_eq!(
        plan.oam_scanout_source,
        OamScanoutSource::ComposeSpiralReturnPlayerShadowAfterMain
    );
    assert!(plan.publish_live_screen_layers);
    assert!(!plan.publish_live_spiral_stair_obj_cache);
    assert!(!plan.publish_live_spiral_stair_return_obj_vram);
    assert!(!plan.publish_spiral_stair_return_equipment_handoff);
    assert_eq!(
        plan.bg_scroll_source,
        DisplayedBgScrollSource::CapturedBeforeNmi
    );

    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_subsubmodule(0x13);
    state.set_dungeon_room_index(1);
    state.dungeon_stair_movement_mut().set_staircase_index(0x30);
    state.follower_link_state_mut().set_y_button_action_step(2);
    state.ppu.screen_enabled = [0x06, 0x11];
    state.ppu.bg_layer[0].h_scroll = 640;
    state.ppu.bg_layer[1].h_scroll = 640;
    state.ppu.oam[102 * 2] = u16::from_le_bytes([116, 108]);
    state.capture_display_snapshot();

    state.ppu.screen_enabled = [0x16, 0x01];
    state.ppu.bg_layer[0].h_scroll = 641;
    state.ppu.bg_layer[1].h_scroll = 641;
    state.ppu.oam[102 * 2] = u16::from_le_bytes([116, 107]);
    let live_shadow = [116, 106, 0x07, 0x1a];
    let byte_start = OAM_BUF + 102 * 4;
    state.ram[byte_start..byte_start + 4].copy_from_slice(&live_shadow);
    state.spiral_stair_return_oam_publication_host_frame = Some(state.frame_ctr_dbg);
    let mut player_oam = [0; 24];
    player_oam[0] = u16::from_le_bytes([live_shadow[0], live_shadow[1]]);
    player_oam[1] = u16::from_le_bytes([live_shadow[2], live_shadow[3]]);
    state.spiral_stair_return_player_oam_scanout = Some(player_oam);
    let presented = state.with_display_snapshot(|display| {
        (
            display.ppu.screen_enabled,
            display.ppu.bg_layer[0].h_scroll,
            display.ppu.bg_layer[1].h_scroll,
            oam_entry_bytes(&display.ppu.oam, 102),
        )
    });

    assert_eq!(presented.0, [0x16, 0x01]);
    assert_eq!((presented.1, presented.2), (640, 640));
    assert_eq!(presented.3, live_shadow);
    let presented_again =
        state.with_display_snapshot(|display| oam_entry_bytes(&display.ppu.oam, 102));
    assert_eq!(presented_again, live_shadow);

    let return_obj = DisplayPublicationPlan::resolve(
        &captured_display_snapshot(),
        DisplayPublicationSignals {
            spiral_stair_return_publishes_live_obj_cache: true,
            ..DisplayPublicationSignals::default()
        },
    );
    assert_eq!(
        return_obj.link_obj_scanout_generation,
        GraphicsDmaGeneration::LiveAfterMain
    );
    assert_eq!(
        return_obj.link_obj_source_generation,
        GraphicsDmaGeneration::LiveAfterMain
    );
    assert_eq!(
        return_obj.oam_scanout_source,
        OamScanoutSource::ComposePublishedShadowDma
    );
    assert!(return_obj.publish_live_spiral_stair_obj_cache);
    assert!(return_obj.publish_live_spiral_stair_return_obj_vram);
    assert!(return_obj.publish_spiral_stair_return_equipment_handoff);

    state.ppu.vram[0x4020] = 0xbeef;
    let mut returning_player_oam = state
        .spiral_stair_return_player_oam_scanout
        .expect("returning player OAM was installed above");
    returning_player_oam[20] = u16::from_le_bytes([120, 84]);
    returning_player_oam[21] = u16::from_le_bytes([0x20, 0x28]);
    returning_player_oam[22] = u16::from_le_bytes([120, 94]);
    returning_player_oam[23] = u16::from_le_bytes([0x22, 0x28]);
    state.spiral_stair_return_player_oam_scanout = Some(returning_player_oam);
    let mut retained_oam = state.ppu.oam.clone();
    retained_oam[112 * 2] = u16::from_le_bytes([120, 240]);
    let mut published_oam = retained_oam.clone();
    published_oam[102 * 2] = u16::from_le_bytes([119, 86]);
    let published_body_xy = published_oam[102 * 2];
    let snapshot = state.display_snapshot.as_mut().unwrap();
    snapshot.obj_generation = DisplayObjGeneration::RetainCapturedOam { oam: retained_oam };
    snapshot.published_shadow_oam_dma = Some(published_oam);
    state.frame_ctr_dbg = state.frame_ctr_dbg.wrapping_add(1);
    let (presented_return_obj, presented_body_xy, presented_sword, presented_shield) = state
        .with_display_snapshot(|display| {
            (
                display.ppu.obj_vram_latch.as_ref().unwrap()[0x4020],
                display.ppu.oam[102 * 2],
                oam_entry_bytes(&display.ppu.oam, 112),
                oam_entry_bytes(&display.ppu.oam, 113),
            )
        });
    assert_eq!(presented_return_obj, 0xbeef);
    // C's NMI_DoUpdates copies the complete $7e0800 OAM shadow to PPU OAM.
    // The explicit published-DMA payload above therefore owns the body slot;
    // only the two equipment entries use the typed spiral-return handoff.
    assert_eq!(presented_body_xy, published_body_xy);
    assert_eq!(presented_sword, [120, 85, 0x20, 0x28]);
    assert_eq!(presented_shield, [120, 95, 0x22, 0x68]);
}

#[test]
fn room_41_state_13_publication_distinguishes_pre_main_entry_from_recurring_main() {
    let pre_main_entry = DisplayPublicationPlan::resolve(
        &captured_display_snapshot(),
        DisplayPublicationSignals {
            dungeon_state_13_phase: DungeonState13PublicationPhase::PreMainQuadrantNmiEntry,
            ..DisplayPublicationSignals::default()
        },
    );

    assert!(pre_main_entry.publish_live_dungeon_state_13_palette_and_registers);
    assert_eq!(
        pre_main_entry.oam_scanout_source,
        OamScanoutSource::ComposeLiveAfterNmi
    );

    let recurring_main = DisplayPublicationPlan::resolve(
        &captured_display_snapshot(),
        DisplayPublicationSignals {
            dungeon_state_13_phase: DungeonState13PublicationPhase::RecurringMain,
            ..DisplayPublicationSignals::default()
        },
    );
    assert!(!recurring_main.publish_live_dungeon_state_13_palette_and_registers);
    assert_eq!(
        recurring_main.oam_scanout_source,
        OamScanoutSource::RetainCapturedBeforeNmi
    );

    let atomic_caller_return = DisplayPublicationPlan::resolve(
        &captured_display_snapshot(),
        DisplayPublicationSignals {
            dungeon_state_13_phase: DungeonState13PublicationPhase::AtomicCallerReturn,
            ..DisplayPublicationSignals::default()
        },
    );
    assert_eq!(
        atomic_caller_return.oam_scanout_source,
        OamScanoutSource::RetainCapturedBeforeNmi
    );
    assert_eq!(
        atomic_caller_return.link_obj_scanout_generation,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(
        atomic_caller_return.link_obj_source_generation,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );

    let caller_return = DisplayPublicationPlan::resolve(
        &captured_display_snapshot(),
        DisplayPublicationSignals {
            dungeon_state_13_phase: DungeonState13PublicationPhase::CallerReturn,
            ..DisplayPublicationSignals::default()
        },
    );
    assert_eq!(
        caller_return.oam_scanout_source,
        OamScanoutSource::RetainCapturedBeforeNmi
    );
    assert_eq!(
        caller_return.link_obj_scanout_generation,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(
        caller_return.link_obj_source_generation,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );

    let mut state = ZeldaState::new();
    let mut previously_presented = state.ppu.oam.clone();
    previously_presented[102 * 2] = u16::from_le_bytes([118, 205]);
    state.last_presented_oam = Some(previously_presented.clone());
    let following = captured_display_snapshot();
    state.compose_display_oam(&following, &caller_return);
    assert_eq!(state.ppu.oam, previously_presented);
}

#[test]
fn faded_filter_caller_return_retains_every_previously_presented_display_memory_domain() {
    let plan = DisplayPublicationPlan::resolve(
        &captured_display_snapshot(),
        DisplayPublicationSignals {
            dungeon_faded_filter_phase: DungeonFadedFilterPublicationPhase::CallerReturn,
            ..DisplayPublicationSignals::default()
        },
    );
    assert_eq!(
        plan.oam_scanout_source,
        OamScanoutSource::RetainCapturedBeforeNmi
    );
    assert_eq!(
        plan.link_obj_scanout_generation,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(
        plan.link_obj_source_generation,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );

    let mut state = ZeldaState::new();
    let mut previous_cgram = state.ppu.cgram.clone();
    previous_cgram[35] = 0x1234;
    state.last_presented_cgram = Some(previous_cgram.clone());
    let mut previous_oam = state.ppu.oam.clone();
    previous_oam[102 * 2] = 0x5678;
    state.last_presented_oam = Some(previous_oam.clone());
    let mut previous_obj = vec![0x9abc; state.ppu.vram.len()];
    previous_obj[0x4020] = 0xdef0;
    state.last_presented_obj_vram = Some(previous_obj.clone());
    let mut following = captured_display_snapshot();
    following.ppu.cgram[35] = 0x1111;
    following.ppu.oam[102 * 2] = 0x2222;
    following.ppu.vram[0x4020] = 0x3333;

    state.compose_display_cgram(&following, &plan);
    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.cgram, previous_cgram);
    assert_eq!(state.ppu.oam, previous_oam);
    assert_eq!(
        &state.ppu.obj_vram_latch.as_ref().unwrap()[0x4000..0x4400],
        &previous_obj[0x4000..0x4400]
    );
}

#[test]
fn authoritative_presented_cgram_outranks_a_native_retained_generation() {
    let mut state = ZeldaState::new();
    state.ppu.cgram[35] = 0x1111;
    let mut following = captured_display_snapshot();
    let mut authority = following.ppu.cgram.clone();
    authority[35] = 0x2222;
    following.cgram_scanout_override = Some(authority.clone());
    let plan = DisplayPublicationPlan::resolve(
        &following,
        DisplayPublicationSignals {
            retain_previous_nmi_display_memory: true,
            ..DisplayPublicationSignals::default()
        },
    );
    assert!(!plan.compose_live_cgram);

    state.compose_display_cgram(&following, &plan);

    assert_eq!(state.ppu.cgram, authority);
}

#[test]
fn spiral_stair_landing_decodes_live_link_obj_cache_without_advancing_raw_vram() {
    let mut state = ZeldaState::new();
    state.ppu.vram[0x4000] = 0x1111;
    state.ppu.vram[0x4020] = 0x2222;
    state.ppu.vram[0x4100] = 0x3333;
    state.ppu.vram[0x4120] = 0x4444;
    state.ppu.vram[0x4320] = 0x5555;
    let mut link_graphics = vec![0; 0x640];
    for (offset, value) in [
        (0x000, 0xaaaau16),
        (0x200, 0xbbbbu16),
        (0x400, 0xccccu16),
        (0x600, 0xddddu16),
    ] {
        for bytes in link_graphics[offset..offset + 0x40].chunks_exact_mut(2) {
            bytes.copy_from_slice(&value.to_le_bytes());
        }
    }
    let mut ranges = vec![(0, 0); 58];
    ranges[57] = (0, link_graphics.len());
    state.assets = Some(AssetPack::from_data_ranges(link_graphics, ranges));

    let mut following = captured_display_snapshot();
    for (slot, source) in [
        (LinkDmaSourceSlot::BodyTop, 0x8000),
        (LinkDmaSourceSlot::BodyBottom, 0x8200),
        (LinkDmaSourceSlot::HeadTop, 0x8400),
        (LinkDmaSourceSlot::HeadBottom, 0x8600),
        (LinkDmaSourceSlot::BodyPointerLower, 0x0100),
    ] {
        write_le_u16(&mut following.ram, slot.ram_address(), source);
    }
    for bytes in following.ram[0x0100..0x0140].chunks_exact_mut(2) {
        bytes.copy_from_slice(&0xeeeeu16.to_le_bytes());
    }
    let plan = DisplayPublicationPlan::resolve(
        &following,
        DisplayPublicationSignals {
            spiral_stair_landing_publishes_live_display: true,
            ..DisplayPublicationSignals::default()
        },
    );
    state.compose_display_oam(&following, &plan);
    let cache = state.ppu.obj_vram_latch.as_ref().unwrap();

    assert_eq!(state.ppu.vram[0x4000], 0x1111);
    assert_eq!(state.ppu.vram[0x4020], 0x2222);
    assert_eq!(state.ppu.vram[0x4100], 0x3333);
    assert_eq!(state.ppu.vram[0x4120], 0x4444);
    assert_eq!(state.ppu.vram[0x4320], 0x5555);
    assert_eq!(cache[0x4000], 0xaaaa);
    assert_eq!(cache[0x4020], 0xcccc);
    assert_eq!(cache[0x4100], 0xbbbb);
    assert_eq!(cache[0x4120], 0xdddd);
    assert_eq!(cache[0x4320], 0xeeee);
}

#[test]
fn dungeon_dialogue_render_entry_decodes_host_link_obj_cache_only() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.ppu.vram[0x4100] = 0x1111;
    write_le_u16(
        &mut state.ram,
        LinkDmaSourceSlot::BodyBottom.ram_address(),
        0x8080,
    );

    let mut link_graphics = vec![0; 0x180];
    for bytes in link_graphics[0x80..0xc0].chunks_exact_mut(2) {
        bytes.copy_from_slice(&0xaaaau16.to_le_bytes());
    }
    for bytes in link_graphics[0x100..0x140].chunks_exact_mut(2) {
        bytes.copy_from_slice(&0xbbbbu16.to_le_bytes());
    }
    let mut ranges = vec![(0, 0); 58];
    ranges[57] = (0, link_graphics.len());
    state.assets = Some(AssetPack::from_data_ranges(link_graphics, ranges));

    let entry_frame = state.game_state.frame;
    state.pre_main_graphics_dma = Some(PreMainGraphicsDma {
        entry_frame,
        entry_plan: rom_graphics_dma_plan_at_host_boundary(entry_frame),
        entry_link_handler_state: 0,
        animated_tile: None,
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        obj_vram: state.ppu.vram.clone(),
        oam_shadow: vec![0; state.ppu.oam.len() * 2],
    });

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 0x0e;
    following.ram[crate::game_state::constants::SUBMODULE] = 2;
    following.ppu.vram[0x4100] = 0x2222;
    write_le_u16(
        &mut following.ram,
        LinkDmaSourceSlot::BodyBottom.ram_address(),
        0x8100,
    );
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.vram[0x4100], 0x1111);
    assert_eq!(state.ppu.obj_vram_latch.as_ref().unwrap()[0x4100], 0xaaaa);

    let mut later_dialogue = ZeldaState::new();
    later_dialogue.set_main_module(0x0e);
    later_dialogue.set_submodule(2);
    later_dialogue.ppu.vram[0x4100] = 0x3333;
    later_dialogue.compose_display_oam(&following, &plan);
    assert!(later_dialogue.ppu.obj_vram_latch.is_none());
}

#[test]
fn dungeon_subtile_scanout_publishes_leading_nmi_animated_chr_independently() {
    let frame = crate::game_state::FrameState {
        main_module: 7,
        submodule: 1,
        subsubmodule: 3,
        ..Default::default()
    };

    let plan = rom_graphics_dma_plan_at_host_boundary(frame);

    assert_eq!(
        plan.animated_bg_scanout,
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
    );
    assert!(rom_dungeon_subtile_direction_one_publishes_live_animated_bg(frame, frame, 1,));
    assert!(!rom_dungeon_subtile_direction_one_publishes_live_animated_bg(frame, frame, 0,));
    assert_eq!(plan.oam_scanout, OamScanoutSource::RetainResidentPpuOam);
    assert_eq!(
        plan.link_obj_operands,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(plan.link_obj_scanout, GraphicsDmaGeneration::LiveAfterMain);
}

#[test]
fn publication_plan_keeps_memory_domains_independent() {
    let mut snapshot = captured_display_snapshot();
    snapshot.vram_generation = DisplayVramGeneration::RetainCapturedBeforeNmi;
    snapshot.oam_scanout_source = OamScanoutSource::ComposePublishedShadowDma;
    snapshot.link_obj_scanout_generation = GraphicsDmaGeneration::LiveAfterMain;
    snapshot.animated_bg_scanout_generation = AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi;
    snapshot.bg_scroll_generation = DisplayBgScrollGeneration::RetainCapturedBeforeNmi;

    let plan = DisplayPublicationPlan::resolve(&snapshot, DisplayPublicationSignals::default());

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
}

#[test]
fn effective_presented_dma_advances_only_written_vram_words_and_obj_cache() {
    let mut state = ZeldaState::new();
    state.ppu.vram.fill(0x1111);
    state.ppu.oam.fill(0x2222);
    state.ppu.cgram.fill(0x3333);
    state.capture_display_snapshot();

    // Model one explicit leading NMI which writes ordinary BG VRAM and one
    // word in each OBJ name page. Every completed VRAM destination advances
    // independently, while CGRAM still requires its own transfer receipt.
    state.ppu.vram[0x1234] = 0x4444;
    state.ppu.vram[0x4009] = 0x5555;
    state.ppu.vram[0x5a20] = 0x6666;
    state.ppu.vram[0x5a40] = 0x7777;
    state.ppu.cgram[7] = 0x5555;
    let mut writes = EffectiveDmaWriteSet::new(state.ppu.vram.len(), true);
    writes.vram_words[0x1234] = true;
    writes.vram_words[0x4009] = true;
    writes.vram_words[0x5a20] = true;
    writes.vram_words[0x5a40] = true;
    let receipt = EffectivePresentedDma::from_write_set(writes.clone(), &state);
    assert_eq!(
        receipt.vram_writes,
        vec![
            (0x1234, 0x4444),
            (0x4009, 0x5555),
            (0x5a20, 0x6666),
            (0x5a40, 0x7777),
        ]
    );

    state.active_effective_dma_writes = Some(writes);
    write_le_u16(
        &mut state.ram,
        LinkDmaSourceSlot::HeadTop.ram_address(),
        0xcd80,
    );
    let completed_link_sources = LinkDmaSources::load_from_ram(&state.ram);
    state.record_completed_link_obj_dma_for_display_boundary(
        completed_link_sources,
        GraphicsDmaGeneration::LiveAfterMain,
    );
    state.record_effective_presented_dma_for_active_scanout();

    // The early Link DMA completes in vblank before this active scanout fetches
    // OBJ tiles, even though its immutable CPU/register snapshot was captured
    // before the interrupt.
    let active = state.display_snapshot.as_deref().unwrap().clone();
    assert_eq!(
        active.effective_presented_dma.as_ref().unwrap().vram_writes,
        receipt.vram_writes
    );
    assert_eq!(
        active
            .effective_presented_dma
            .as_ref()
            .unwrap()
            .completed_link_obj_dma,
        Some(CompletedLinkObjDma {
            sources: completed_link_sources,
            source_generation: GraphicsDmaGeneration::LiveAfterMain,
        }),
    );
    state.last_presented_oam = Some(vec![0xbbbb; state.ppu.oam.len()]);
    state.last_presented_obj_vram = Some(vec![0xdddd; state.ppu.vram.len()]);

    state.ppu.vram.fill(0xeeee);
    // Direct helper calls model the state after `with_display_snapshot` has
    // swapped the captured scanout into `self.ppu`.
    state.ppu.obj_tile_adr1 = 0x4000;
    state.ppu.obj_tile_adr2 = 0x5000;
    let mut preserved_obj_cache = vec![0xaaaa; state.ppu.vram.len()];
    preserved_obj_cache[0x4008] = 0x7777;
    preserved_obj_cache[0x5a21] = 0x8888;
    preserved_obj_cache[0x1234] = 0x9999;
    state.ppu.obj_vram_latch = Some(preserved_obj_cache);
    state.ppu.oam.fill(0xeeee);
    state.ppu.cgram.fill(0xeeee);
    state.compose_effective_presented_vram(&active);
    state.compose_effective_presented_obj(&active);

    assert_eq!(state.ppu.vram[0x1234], 0x4444);
    assert_eq!(state.ppu.vram[0x4009], 0x5555);
    assert_eq!(state.ppu.cgram[7], 0xeeee);
    assert_eq!(state.ppu.oam[0], 0xeeee);
    let obj = state.ppu.obj_vram_latch.as_ref().unwrap();
    assert_eq!(obj[0x4008], 0x7777);
    assert_eq!(obj[0x4009], 0x5555);
    assert_eq!(obj[0x5a20], 0x6666);
    assert_eq!(obj[0x5a40], 0x7777);
    assert_eq!(obj[0x5a21], 0x8888);
    assert_eq!(obj[0x1234], 0x9999);
}

#[test]
fn sparse_presented_obj_tiles_replace_only_their_addressed_tiles() {
    let mut state = ZeldaState::new();
    state.set_main_module(0);
    state.set_submodule(5);
    state.ppu.obj_tile_adr1 = 0x4000;
    state.ppu.obj_tile_adr2 = 0x5000;
    state.ppu.vram.fill(0x1111);
    state.ppu.vram[0x5a20] = 0xcccc;
    state.ppu.vram[0x5a40] = 0xdddd;
    let mut preserved_obj_cache = vec![0xaaaa; state.ppu.vram.len()];
    preserved_obj_cache[0x5a20] = 0x2020;
    preserved_obj_cache[0x5a21] = 0x2121;
    preserved_obj_cache[0x5a40] = 0x4040;
    preserved_obj_cache[0x1234] = 0x3434;
    state.ppu.obj_vram_latch = Some(preserved_obj_cache.clone());
    state.capture_display_snapshot();
    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .effective_presented_dma = Some(EffectivePresentedDma {
        vram_writes: vec![(0x1234, 0xbbbb)],
        decoded_bg_vram_writes: Vec::new(),
        completed_oam: None,
        completed_link_obj_dma: None,
        completed_cgram: None,
        completed_ppu_registers: None,
        completed_dialogue_metadata: None,
    });
    let mut presented_pixels = vec![0; 2 * crate::PresentedObjTiles::PIXELS_PER_TILE];
    presented_pixels[0] = 0x0f;
    presented_pixels[crate::PresentedObjTiles::PIXELS_PER_TILE] = 0x04;
    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .presented_obj_tiles_override =
        Some(crate::PresentedObjTiles::new(vec![0x4000, 0x5a20], presented_pixels).unwrap());

    state.with_display_snapshot(|presented| {
        assert_eq!(presented.ppu.vram[0x1234], 0xbbbb);
        let obj = presented.ppu.obj_vram_latch.as_ref().unwrap();
        assert_eq!(obj[0x5a20], 0x0000);
        assert_eq!(obj[0x5a28], 0x0080);
        assert_eq!(obj[0x5a40], 0x4040);
        assert_eq!(obj[0x5a21], 0x0000);
        assert_eq!(obj[0x4000], 0x8080);
        assert_eq!(obj[0x4008], 0x8080);
    });

    let stored = state
        .display_snapshot
        .as_ref()
        .unwrap()
        .ppu
        .obj_vram_latch
        .as_ref()
        .unwrap();
    assert_eq!(stored, &preserved_obj_cache);
}

#[test]
fn sparse_presented_obj_tiles_preserve_unmentioned_prior_presented_cache_without_a_captured_latch()
{
    let mut state = ZeldaState::new();
    state.set_main_module(0);
    state.set_submodule(5);
    state.ppu.obj_tile_adr1 = 0x4000;
    state.ppu.obj_tile_adr2 = 0x5000;
    state.ppu.vram.fill(0x1111);
    state.ppu.vram[0x5a20] = 0xcccc;
    state.ppu.vram[0x5a40] = 0xdddd;
    let mut prior_presented_cache = vec![0xaaaa; state.ppu.vram.len()];
    prior_presented_cache[0x5a20] = 0x2020;
    prior_presented_cache[0x5a40] = 0x4040;
    prior_presented_cache[0x1234] = 0x3434;
    state.last_presented_obj_vram = Some(prior_presented_cache);
    state.ppu.obj_vram_latch = None;
    state
        .vram_chr_source
        .record_tiles(0x5a40, 1, crate::chr_source::CHR_KIND_LINK, 0x1234);
    state.capture_display_snapshot();

    let mut presented_pixels = vec![0; crate::PresentedObjTiles::PIXELS_PER_TILE];
    presented_pixels[0] = 0x04;
    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .presented_obj_tiles_override =
        Some(crate::PresentedObjTiles::new(vec![0x5a20], presented_pixels).unwrap());

    state.with_display_snapshot(|presented| {
        let obj = presented.ppu.obj_vram_latch.as_ref().unwrap();
        assert_eq!(obj[0x5a20], 0x0000);
        assert_eq!(obj[0x5a28], 0x0080);
        assert_eq!(obj[0x5a40], 0x4040);
        assert_eq!(obj[0x1234], 0x3434);
        let exact = presented.vram_chr_source.get(0x5a20 / 16);
        assert_eq!(exact.kind, crate::chr_source::CHR_KIND_BG_STREAM);
        assert_eq!(
            (u32::from(exact.pack) << 16) | u32::from(exact.tile_off),
            crate::chr_source::chr_content_hash32(&obj[0x5a20..0x5a30])
        );
        assert_eq!(
            presented.vram_chr_source.get(0x5a40 / 16).kind,
            crate::chr_source::CHR_KIND_LINK
        );
    });

    assert!(state.ppu.obj_vram_latch.is_none());
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.ppu.obj_vram_latch.is_none()));
}

#[test]
fn same_value_effective_obj_write_retains_an_explicit_cache_owner() {
    let mut state = ZeldaState::new();
    state.ppu.obj_tile_adr1 = 0x4000;
    state.ppu.obj_tile_adr2 = 0x5000;
    state.ppu.vram.fill(0x1111);
    let mut last_presented = state.ppu.vram.clone();
    last_presented[0x5a20] = 0x2222;
    state.last_presented_obj_vram = Some(last_presented);
    state.ppu.obj_vram_latch = None;
    let mut active = captured_display_snapshot();
    active.effective_presented_dma = Some(EffectivePresentedDma {
        vram_writes: vec![(0x5a20, 0x1111)],
        decoded_bg_vram_writes: Vec::new(),
        completed_oam: None,
        completed_link_obj_dma: None,
        completed_cgram: None,
        completed_ppu_registers: None,
        completed_dialogue_metadata: None,
    });

    state.compose_effective_presented_obj(&active);
    let mut sparse_pixels = vec![0; crate::PresentedObjTiles::PIXELS_PER_TILE];
    sparse_pixels[0] = 4;
    state.apply_original_timing_presented_obj_tiles(
        &crate::PresentedObjTiles::new(vec![0x5a40], sparse_pixels).unwrap(),
    );

    let composed = state.ppu.obj_vram_latch.as_ref().unwrap();
    assert_eq!(
        composed[0x5a20], 0x1111,
        "the sparse overlay must not resurrect the stale last-presented word which the effective write restored to raw VRAM",
    );
    assert_eq!(composed[0x5a40], 0x0000);
    assert_eq!(composed[0x5a48], 0x0080);
}

#[test]
fn host_boundary_obj_cache_retains_both_hardware_name_pages() {
    let mut state = ZeldaState::new();
    state.ppu.vram.fill(0x2222);
    let mut host_boundary_vram = state.ppu.vram.clone();
    host_boundary_vram[0x4020] = 0x1111;
    host_boundary_vram[0x59a0] = 0x3333;
    let entry_frame = state.game_state.frame;
    state.pre_main_graphics_dma = Some(PreMainGraphicsDma {
        entry_frame,
        entry_plan: rom_graphics_dma_plan_at_host_boundary(entry_frame),
        entry_link_handler_state: 0,
        animated_tile: None,
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        obj_vram: host_boundary_vram,
        oam_shadow: vec![0; state.ppu.oam.len() * 2],
    });
    let mut active = captured_display_snapshot();
    active.obj_cache_generation = DisplayObjCacheGeneration::HostBoundaryBeforeMain;
    let plan = DisplayPublicationPlan::resolve(&active, DisplayPublicationSignals::default());

    state.compose_display_oam(&active, &plan);

    let cache = state.ppu.obj_vram_latch.as_ref().unwrap();
    assert_eq!(cache[0x4020], 0x1111);
    assert_eq!(cache[0x59a0], 0x3333);
    assert_eq!(state.ppu.vram[0x59a0], 0x2222);
}

#[test]
fn explicit_obj_cache_owner_is_not_replaced_by_a_bulk_nmi_receipt() {
    let mut state = ZeldaState::new();
    state.ppu.vram.fill(0x1111);
    state.capture_display_snapshot();
    let mut active = state.display_snapshot.as_deref().unwrap().clone();
    active.effective_presented_dma = Some(EffectivePresentedDma {
        vram_writes: vec![(0x4020, 0x2222)],
        decoded_bg_vram_writes: Vec::new(),
        completed_oam: None,
        completed_link_obj_dma: None,
        completed_cgram: None,
        completed_ppu_registers: None,
        completed_dialogue_metadata: None,
    });
    active.explicit_obj_cache_vram = Some(vec![0x4444; state.ppu.vram.len()]);
    state.last_presented_obj_vram = Some(vec![0x3333; state.ppu.vram.len()]);
    state.ppu.obj_vram_latch = Some(vec![0x4444; state.ppu.vram.len()]);

    state.compose_effective_presented_obj(&active);

    assert_eq!(state.ppu.obj_vram_latch.as_ref().unwrap()[0x4020], 0x4444);
}

#[test]
fn host_boundary_animated_bg_leading_nmi_receipt_does_not_override_stored_scanouts() {
    const DESTINATION: usize = 0x3b00;
    const ORDINARY_WORD: usize = 0x1234;
    let mut state = ZeldaState::new();
    state.set_animated_tile_vram_destination_address(DESTINATION as u16);
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::AdvanceStaged);
    state.display_snapshot.as_mut().unwrap().vram_generation =
        DisplayVramGeneration::ComposeLiveAfterNmi;

    state.ppu.vram[DESTINATION] = 0xaaaa;
    state.ppu.vram[ORDINARY_WORD] = 0xbbbb;
    let mut writes = EffectiveDmaWriteSet::new(state.ppu.vram.len(), true);
    writes.vram_words[DESTINATION] = true;
    writes.vram_words[ORDINARY_WORD] = true;
    state.active_effective_dma_writes = Some(writes);
    state.record_effective_presented_dma_for_active_scanout();

    let receipt = state
        .display_snapshot
        .as_ref()
        .unwrap()
        .effective_presented_dma
        .as_ref()
        .unwrap();
    assert_eq!(receipt.vram_writes, vec![(ORDINARY_WORD, 0xbbbb)]);
    assert!(receipt.decoded_bg_vram_writes.is_empty());
    assert!(state
        .deferred_display_snapshot
        .as_ref()
        .unwrap()
        .effective_presented_dma
        .is_none());

    let active = state.display_snapshot.as_deref().unwrap().clone();
    state.ppu.vram[DESTINATION] = 0x1111;
    state.compose_effective_presented_vram(&active);
    state.compose_effective_presented_bg_chr_cache(&active);

    assert_eq!(state.ppu.vram[DESTINATION], 0x1111);
    assert!(state.ppu.bg_vram_latch.is_none());
}

#[test]
fn first_interrupted_landing_field_retires_the_pre_spotlight_scanout() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0x0f);
    state.set_subsubmodule(0);
    state.begin_dungeon_landing_spotlight_publication(true);
    state.ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + ZeldaState::HDMA_DYNAMIC_TABLE_LEN]
        .fill(0xff);
    state.ram[RESERVED_HDMA_TABLE..RESERVED_HDMA_TABLE + ZeldaState::HDMA_DYNAMIC_TABLE_LEN]
        .fill(0x7e);
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::AdvanceStaged);

    state.set_subsubmodule(1);
    state.ram[crate::game_state::constants::W12SEL_COPY] = 0x33;
    state.ram[crate::game_state::constants::W34SEL_COPY] = 0x03;
    state.ram[crate::game_state::constants::WOBJSEL_COPY] = 0x33;
    state.ram[crate::game_state::constants::TMW_COPY] = 0x16;
    state.ram[crate::game_state::constants::TSW_COPY] = 0x01;
    state.ram[crate::game_state::constants::HDMAEN_COPY] = 0x80;
    state.ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + ZeldaState::HDMA_DYNAMIC_TABLE_LEN]
        .fill(0x00);
    state.ram[RESERVED_HDMA_TABLE..RESERVED_HDMA_TABLE + ZeldaState::HDMA_DYNAMIC_TABLE_LEN]
        .fill(0x11);
    state.game_execution_scheduler.begin_host_frame();
    state
        .game_execution_scheduler
        .mark_main_iteration_after_leading_nmi();
    state.game_execution_scheduler.begin_main_loop_iteration();
    state
        .game_execution_scheduler
        .schedule_cpu_timed_work_from_current_main_iteration(
            GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
            1,
        );
    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .animated_bg_scanout_generation = AnimatedBgScanoutGeneration::LiveAfterNmi;
    state.stage_interrupted_dungeon_submodule_publication();

    let pending = state
        .interrupted_dungeon_submodule_publication
        .as_ref()
        .unwrap();
    assert!(state.dungeon_landing_entry_started_after_leading_nmi);
    let retiring = pending.retiring_pre_spotlight_scanout.as_ref().unwrap();
    assert_eq!(retiring.windowsel, 0x33_03_33);
    assert_eq!(retiring.screen_windowed, [0x16, 0x01]);
    assert_eq!(retiring.hdma_enable_mask, 0x80);
    assert!(retiring.hdma_tables[0].iter().all(|&byte| byte == 0x00));
    assert!(retiring.hdma_tables[1].iter().all(|&byte| byte == 0x11));
    assert!(pending.spotlight.is_none());

    state.apply_interrupted_dungeon_submodule_publication();
    assert_eq!(
        state.display_snapshot.as_ref().unwrap().oam_scanout_source,
        OamScanoutSource::RetainPreviousPresented
    );
    assert_eq!(
        state
            .display_snapshot
            .as_ref()
            .unwrap()
            .animated_bg_scanout_generation,
        AnimatedBgScanoutGeneration::LiveAfterNmi,
        "a held NMI publishes registers but cannot roll back the animated-BG DMA generation",
    );
}

#[test]
fn trailing_animated_bg_dma_refines_only_an_already_staged_following_scanout() {
    const DESTINATION: usize = 0x3c00;
    let mut state = ZeldaState::new();
    state.set_animated_tile_vram_destination_address(DESTINATION as u16);
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::AdvanceStaged);

    state.begin_effective_presented_dma();
    state.ppu.vram[DESTINATION] = 0x2222;
    state.mark_effective_dma_vram_word(DESTINATION);
    state.record_trailing_nmi_receipts();

    assert!(state
        .display_snapshot
        .as_ref()
        .unwrap()
        .effective_presented_dma
        .is_none());
    assert_eq!(
        state
            .deferred_display_snapshot
            .as_ref()
            .unwrap()
            .effective_presented_dma
            .as_ref()
            .unwrap()
            .decoded_bg_vram_writes,
        vec![(DESTINATION, 0x2222)]
    );
}

#[test]
fn effective_presented_cgram_uses_the_palette_installed_by_the_leading_nmi() {
    let mut state = ZeldaState::new();
    state.ppu.cgram.fill(0x1111);
    state.capture_display_snapshot();

    state.begin_effective_presented_dma();
    state.ppu.cgram.fill(0x2222);
    state.record_completed_cgram_dma_for_display_boundary();
    state.record_effective_presented_dma_for_active_scanout();

    let active = state.display_snapshot.as_deref().unwrap().clone();
    state.ppu.cgram.fill(0xeeee);
    state.compose_effective_presented_cgram(&active);

    assert!(state.ppu.cgram.iter().all(|&word| word == 0x2222));
}

#[test]
fn effective_presented_ppu_registers_use_the_values_installed_by_the_leading_nmi() {
    let mut state = ZeldaState::new();
    state.capture_display_snapshot();

    state.begin_effective_presented_dma();
    state.ppu.bg_layer[0].h_scroll = 0x0123;
    state.ppu.bg_layer[1].h_scroll = 0x0456;
    state.ppu.screen_enabled = [0x16, 0x02];
    state.ppu.screen_windowed = [0x04, 0x08];
    state.ppu.brightness = 0x0f;
    state.ppu.forced_blank = false;
    state.ppu.mode = 7;
    state.ppu.mosaic_enabled = 0x03;
    state.ppu.mosaic_size = 5;
    state.ppu.m7_matrix = [1, 2, 3, 4, 5, 6, 7, 8];
    for (index, layer) in state.ppu.bg_layer.iter_mut().enumerate() {
        layer.tile_adr = 0x1000 * index as u16;
    }
    state.record_completed_ppu_registers_for_display_boundary();
    state.record_effective_presented_dma_for_active_scanout();

    let active = state.display_snapshot.as_deref().unwrap().clone();
    state.ppu.bg_layer[0].h_scroll = 0xaaaa;
    state.ppu.bg_layer[1].h_scroll = 0xbbbb;
    state.ppu.screen_enabled = [0xff, 0xff];
    state.ppu.screen_windowed = [0xff, 0xff];
    state.ppu.brightness = 0;
    state.ppu.forced_blank = true;
    state.ppu.mode = 1;
    state.ppu.mosaic_enabled = 0;
    state.ppu.mosaic_size = 1;
    state.ppu.m7_matrix = [0; 8];
    for layer in &mut state.ppu.bg_layer {
        layer.tile_adr = 0x7000;
    }
    state.ppu.forced_blank_scanlines = 10;
    state.ppu.forced_blank_from_scanline = Some(10);
    state.ppu.retain_active_display_history = true;
    state.compose_effective_presented_ppu_registers(
        &active,
        DisplayedBgScrollSource::CapturedBeforeNmi,
    );

    assert_eq!(state.ppu.bg_layer[0].h_scroll, 0x0123);
    assert_eq!(state.ppu.bg_layer[1].h_scroll, 0x0456);
    assert_eq!(state.ppu.screen_enabled, [0x16, 0x02]);
    assert_eq!(state.ppu.screen_windowed, [0x04, 0x08]);
    assert_eq!(state.ppu.brightness, 0x0f);
    assert!(!state.ppu.forced_blank);
    assert_eq!(state.ppu.mode, 7);
    assert_eq!(state.ppu.mosaic_enabled, 0x03);
    assert_eq!(state.ppu.mosaic_size, 5);
    assert_eq!(state.ppu.m7_matrix, [1, 2, 3, 4, 5, 6, 7, 8]);
    let tile_addresses: [u16; 4] = std::array::from_fn(|index| state.ppu.bg_layer[index].tile_adr);
    assert_eq!(tile_addresses, [0x0000, 0x1000, 0x2000, 0x3000]);
    assert_eq!(state.ppu.forced_blank_scanlines, 10);
    assert_eq!(state.ppu.forced_blank_from_scanline, None);
    assert!(!state.ppu.retain_active_display_history);
}

#[test]
fn c_write_ppu_registers_trailing_receipt_publishes_rain_half_color_without_a_route_rule() {
    let mut state = ZeldaState::new();
    state.ppu.half_color = false;
    state.ppu.math_enabled = 0x32;
    state.capture_display_snapshot();

    // C sources:
    // - `src/overworld.c::OverworldOverlay_HandleRain` writes
    //   `CGADSUB_copy = 0x72` when frame_counter is 44.
    // - `src/nmi.c::WritePpuRegisters` writes that mirror to CGADSUB as part
    //   of the following coupled register publication.
    state.set_color_math_control(0x72);
    state.begin_effective_presented_dma();
    state
        .active_effective_dma_writes
        .as_mut()
        .unwrap()
        .completed_ppu_registers_own_active_scanout = true;
    state.write_ppu_registers();
    state.record_trailing_nmi_receipts();

    let active = state.display_snapshot.as_deref().unwrap().clone();
    let registers = active
        .effective_presented_dma
        .as_ref()
        .and_then(|receipt| receipt.completed_ppu_registers)
        .expect("the accepted trailing NMI must retain its completed register write");
    assert!(registers.color_math.half_color);
    assert_eq!(registers.color_math.math_enabled, 0x32);

    state.ppu.half_color = false;
    state.ppu.math_enabled = 0;
    state.compose_effective_presented_ppu_registers(
        &active,
        DisplayedBgScrollSource::CapturedBeforeNmi,
    );
    assert!(state.ppu.half_color);
    assert_eq!(state.ppu.math_enabled, 0x32);
}

#[test]
fn completed_main_trailing_nmi_does_not_rewrite_the_active_scanout() {
    let mut state = ZeldaState::new();
    state.ppu.brightness = 15;
    state.capture_display_snapshot();

    // An ordinary atomic main slice has returned before this synthetic
    // trailing NMI. Its register writes are resident for the next capture,
    // unlike the interrupted-C-call boundary covered above.
    state.game_state.display.screen_brightness = 14;
    state.begin_effective_presented_dma();
    state.write_ppu_registers();
    state.record_trailing_nmi_receipts();

    assert!(state
        .display_snapshot
        .as_ref()
        .unwrap()
        .effective_presented_dma
        .is_none());
}

#[test]
fn oam_dma_after_closed_boundary_preserves_that_publications_active_image() {
    let mut state = ZeldaState::new();
    state.ppu.oam.fill(0x1111);
    state.capture_display_snapshot();
    state.close_display_boundary_dma_receipts();

    state.ppu.oam.fill(0x2222);
    state.record_completed_oam_dma_for_display_boundary();
    let receipt = state
        .display_snapshot
        .as_deref()
        .unwrap()
        .closed_oam_boundary_receipt
        .as_ref()
        .unwrap();
    assert_eq!(receipt.publication_host_frame, state.frame_ctr_dbg);
    assert!(receipt.active_oam.iter().all(|&word| word == 0x1111));

    let mut active = state.display_snapshot.as_deref().unwrap().clone();
    active.oam_scanout_source = OamScanoutSource::ComposeLiveAfterNmi;
    let plan = DisplayPublicationPlan::resolve(&active, DisplayPublicationSignals::default());
    state.ppu.oam.fill(0x3333);
    state.compose_display_oam(&active, &plan);
    assert!(state.ppu.oam.iter().all(|&word| word == 0x1111));
    assert!(state.ppu.oam.iter().all(|&word| word != 0x2222));

    // A same-host recapture retains the receipt because it is still the same
    // publication epoch.
    state.capture_display_snapshot();
    assert!(state
        .display_snapshot
        .as_deref()
        .unwrap()
        .closed_oam_boundary_receipt
        .is_some());

    // Retaining the snapshot into another host cannot replay a closed-boundary
    // receipt as though it belonged to the new publication.
    state.frame_ctr_dbg += 1;
    state.capture_display_snapshot();
    assert!(state
        .display_snapshot
        .as_deref()
        .unwrap()
        .closed_oam_boundary_receipt
        .is_none());
}

#[test]
fn late_oam_dma_preserves_the_last_dma_accepted_by_the_active_boundary() {
    let mut state = ZeldaState::new();
    state.ppu.oam.fill(0x1111);
    state.capture_display_snapshot();

    state.ppu.oam.fill(0x1212);
    state.record_completed_oam_dma_for_display_boundary();
    state.close_display_boundary_dma_receipts();
    state.ppu.oam.fill(0x2222);
    state.record_completed_oam_dma_for_display_boundary();

    let active_oam = state
        .display_snapshot
        .as_deref()
        .unwrap()
        .closed_oam_boundary_receipt
        .as_ref()
        .unwrap()
        .active_oam
        .as_slice();
    assert!(active_oam.iter().all(|&word| word == 0x1212));
}

#[test]
fn late_oam_dma_uses_the_retiring_hardware_scanout() {
    let mut state = ZeldaState::new();
    state.ppu.oam.fill(0x1111);
    state.capture_display_snapshot();
    state.close_display_boundary_dma_receipts();
    state.last_presented_oam = Some(vec![0x1313; state.ppu.oam.len()]);
    state.ppu.oam.fill(0x2222);
    state.record_completed_oam_dma_for_display_boundary();

    let active_oam = state
        .display_snapshot
        .as_deref()
        .unwrap()
        .closed_oam_boundary_receipt
        .as_ref()
        .unwrap()
        .active_oam
        .as_slice();
    assert!(active_oam.iter().all(|&word| word == 0x1313));
}

#[test]
fn retained_publication_does_not_accept_a_following_nmi_oam_dma() {
    let mut state = ZeldaState::new();
    state.ppu.oam.fill(0x1111);
    state.capture_display_snapshot();
    state.ppu.oam.fill(0x1212);
    state.record_completed_oam_dma_for_display_boundary();
    state.close_display_boundary_dma_receipts();

    state.frame_ctr_dbg += 1;
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::RetainPublished);
    assert!(
        !state
            .display_snapshot
            .as_deref()
            .unwrap()
            .accepts_nmi_dma_receipts
    );
    assert!(state
        .display_snapshot
        .as_deref()
        .unwrap()
        .completed_oam_dma_after_capture
        .as_ref()
        .is_some_and(|oam| oam.iter().all(|&word| word == 0x1212)));

    state.ppu.oam.fill(0x2222);
    state.record_completed_oam_dma_for_display_boundary();

    let retained = state.display_snapshot.as_deref().unwrap();
    assert!(retained
        .completed_oam_dma_after_capture
        .as_ref()
        .is_some_and(|oam| oam.iter().all(|&word| word == 0x1212)));
    assert!(retained.ppu.oam.iter().all(|&word| word == 0x1111));
    assert!(retained
        .closed_oam_boundary_receipt
        .as_ref()
        .is_some_and(|receipt| receipt.active_oam.iter().all(|&word| word == 0x1212)));
}

#[test]
fn room_72_state_7_retains_the_exact_dma_latched_oam() {
    let mut state = ZeldaState::new();
    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 2;
    following.ram[crate::game_state::constants::SUBSUBMODULE] = 7;
    following.ram[crate::game_state::constants::DUNGEON_ROOM] = 0x72;
    following.oam_scanout_source = OamScanoutSource::RetainResidentPpuOam;
    following.ppu.oam[92 * 2] = u16::from_le_bytes([0x30, 14]);
    following.ppu.oam[93 * 2] = u16::from_le_bytes([0x30, 24]);
    state.ppu.oam.clone_from(&following.ppu.oam);
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[92 * 2].to_le_bytes()[1], 14);
    assert_eq!(state.ppu.oam[93 * 2].to_le_bytes()[1], 24);
}

#[test]
fn effective_presented_obj_dma_empty_receipt_does_not_synthesize_a_cache() {
    let mut state = ZeldaState::new();
    state.ppu.vram.fill(0x1111);
    state.capture_display_snapshot();
    state.last_presented_obj_vram = Some(vec![0x2222; state.ppu.vram.len()]);

    // CPU-authored source words and live VRAM may move while NMI is held. An
    // explicit boundary receipt with no OBJ writes must not synthesize a new
    // decoded cache from either of them.
    state.ppu.vram[0x4000..0x4400].fill(0x3333);
    state.active_effective_dma_writes = Some(EffectiveDmaWriteSet::new(state.ppu.vram.len(), true));
    state.record_effective_presented_dma_for_active_scanout();

    let active = state.display_snapshot.as_deref().unwrap().clone();
    assert!(active
        .effective_presented_dma
        .as_ref()
        .unwrap()
        .vram_writes
        .is_empty());
    state.compose_effective_presented_obj(&active);

    assert!(state.ppu.obj_vram_latch.is_none());
}

#[test]
fn unrelated_effective_dma_does_not_replace_or_synthesize_an_obj_cache() {
    let mut state = ZeldaState::new();
    let mut active = captured_display_snapshot();
    active.ppu.obj_tile_adr1 = 0x4000;
    active.ppu.obj_tile_adr2 = 0x5000;
    active.effective_presented_dma = Some(EffectivePresentedDma {
        vram_writes: vec![(0x1234, 0xbbbb)],
        decoded_bg_vram_writes: Vec::new(),
        completed_oam: None,
        completed_link_obj_dma: None,
        completed_cgram: None,
        completed_ppu_registers: None,
        completed_dialogue_metadata: None,
    });
    let preserved = vec![0xaaaa; state.ppu.vram.len()];
    state.ppu.obj_vram_latch = Some(preserved.clone());

    state.compose_effective_presented_obj(&active);

    assert_eq!(state.ppu.obj_vram_latch.as_ref(), Some(&preserved));
}

#[test]
fn effective_presented_obj_dma_wraps_name_pages_in_15_bit_vram_space() {
    let mut state = ZeldaState::new();
    let mut active = captured_display_snapshot();
    state.ppu.obj_tile_adr1 = 0x4000;
    state.ppu.obj_tile_adr2 = 0x8000;
    active.effective_presented_dma = Some(EffectivePresentedDma {
        vram_writes: vec![(0x0000, 0xaaaa), (0x0fff, 0xbbbb), (0x1000, 0xcccc)],
        decoded_bg_vram_writes: Vec::new(),
        completed_oam: None,
        completed_link_obj_dma: None,
        completed_cgram: None,
        completed_ppu_registers: None,
        completed_dialogue_metadata: None,
    });
    let preserved = vec![0x1111; state.ppu.vram.len()];
    state.ppu.obj_vram_latch = Some(preserved);

    state.compose_effective_presented_obj(&active);

    let composed = state.ppu.obj_vram_latch.as_ref().unwrap();
    assert_eq!(composed[0x0000], 0xaaaa);
    assert_eq!(composed[0x0fff], 0xbbbb);
    assert_eq!(composed[0x1000], 0x1111);
}

#[test]
fn display_composition_patches_captured_obj_page_and_restores_live_ppu() {
    let mut state = ZeldaState::new();
    state.ppu.obj_tile_adr1 = 0x4000;
    state.ppu.obj_tile_adr2 = 0x5000;
    let mut captured_cache = vec![0x1111; state.ppu.vram.len()];
    captured_cache[0x5a20] = 0x2222;
    state.ppu.obj_vram_latch = Some(captured_cache);
    state.capture_display_snapshot();
    state
        .display_snapshot
        .as_deref_mut()
        .unwrap()
        .effective_presented_dma = Some(EffectivePresentedDma {
        vram_writes: vec![(0x5a20, 0xaaaa)],
        decoded_bg_vram_writes: Vec::new(),
        completed_oam: None,
        completed_link_obj_dma: None,
        completed_cgram: None,
        completed_ppu_registers: None,
        completed_dialogue_metadata: None,
    });

    state.ppu.obj_tile_adr1 = 0;
    state.ppu.obj_tile_adr2 = 0x1000;
    let mut live_cache = vec![0x3333; state.ppu.vram.len()];
    live_cache[0x5a20] = 0x4444;
    state.ppu.obj_vram_latch = Some(live_cache.clone());

    state.with_display_snapshot(|presented| {
        assert_eq!(presented.ppu.obj_tile_adr1, 0x4000);
        assert_eq!(presented.ppu.obj_tile_adr2, 0x5000);
        let composed = presented.ppu.obj_vram_latch.as_ref().unwrap();
        assert_eq!(composed[0x5a20], 0xaaaa);
        assert_eq!(composed[0x5a21], 0x1111);
    });

    assert_eq!(state.ppu.obj_tile_adr1, 0);
    assert_eq!(state.ppu.obj_tile_adr2, 0x1000);
    assert_eq!(state.ppu.obj_vram_latch.as_ref(), Some(&live_cache));
}

#[test]
fn effective_presented_obj_dma_merge_keeps_latest_write_per_address() {
    let mut state = ZeldaState::new();
    state.ppu.vram[0x4009] = 0x1111;
    let mut first_writes = EffectiveDmaWriteSet::new(state.ppu.vram.len(), true);
    first_writes.vram_words[0x4009] = true;
    let mut merged = EffectivePresentedDma::from_write_set(first_writes, &state);

    state.ppu.vram[0x4009] = 0x3333;
    state.ppu.vram[0x400c] = 0x4444;
    let mut second_writes = EffectiveDmaWriteSet::new(state.ppu.vram.len(), true);
    second_writes.vram_words[0x4009] = true;
    second_writes.vram_words[0x400c] = true;
    let later = EffectivePresentedDma::from_write_set(second_writes, &state);
    merged.merge_after(later);

    assert_eq!(merged.vram_writes, vec![(0x4009, 0x3333), (0x400c, 0x4444)]);
}

#[test]
fn effective_presented_obj_dma_records_same_value_rewrites() {
    let mut state = ZeldaState::new();
    state.ppu.vram[0x4009] = 0x1111;
    let mut writes = EffectiveDmaWriteSet::new(state.ppu.vram.len(), true);
    writes.vram_words[0x4009] = true;

    let receipt = EffectivePresentedDma::from_write_set(writes, &state);

    assert_eq!(receipt.vram_writes, vec![(0x4009, 0x1111)]);
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
fn retained_oam_scanout_without_a_completed_dma_keeps_resident_ppu_oam() {
    let mut state = ZeldaState::new();
    state.ppu.oam[40] = u16::from_le_bytes([0x38, 0x34]);

    let mut following = captured_display_snapshot();
    let mut published_shadow = following.ppu.oam.clone();
    published_shadow[40] = u16::from_le_bytes([0x38, 0xf0]);
    following.published_shadow_oam_dma = Some(published_shadow);
    following.oam_scanout_source = OamScanoutSource::RetainCapturedBeforeNmi;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[40].to_le_bytes(), [0x38, 0x34]);
}

#[test]
fn retained_oam_scanout_without_new_work_keeps_the_last_published_oam() {
    let mut state = ZeldaState::new();
    state.ppu.oam[40] = u16::from_le_bytes([0x38, 0x34]);
    let mut last_published = state.ppu.oam.clone();
    last_published[40] = u16::from_le_bytes([0x38, 0xf0]);
    state.last_presented_oam = Some(last_published);

    let mut following = captured_display_snapshot();
    following.oam_scanout_source = OamScanoutSource::RetainCapturedBeforeNmi;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[40].to_le_bytes(), [0x38, 0xf0]);
}

#[test]
fn retained_oam_scanout_publishes_completed_dma_when_main_iteration_finishes() {
    let mut state = ZeldaState::new();
    state.ppu.oam[40] = u16::from_le_bytes([0x38, 0x34]);
    state.ram[crate::game_state::constants::FRAME_COUNTER] = 0x10;

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::FRAME_COUNTER] = 0x11;
    following.ram[crate::game_state::constants::NMI_BOOLEAN] = 1;
    let mut completed_dma = following.ppu.oam.clone();
    completed_dma[40] = u16::from_le_bytes([0x38, 0xf0]);
    following.completed_oam_dma_after_capture = Some(completed_dma);
    let mut retained_oam = following.ppu.oam.clone();
    retained_oam[40] = u16::from_le_bytes([0x38, 0x55]);
    following.obj_generation = DisplayObjGeneration::RetainCapturedOam { oam: retained_oam };
    following.oam_scanout_source = OamScanoutSource::RetainResidentPpuOam;
    following.link_obj_scanout_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[40].to_le_bytes(), [0x38, 0xf0]);
}

#[test]
fn resident_oam_scanout_does_not_advance_during_an_unchanged_continuation() {
    let mut state = ZeldaState::new();
    state.ppu.oam[40] = u16::from_le_bytes([0x38, 0x34]);

    let mut following = captured_display_snapshot();
    let mut completed_dma = following.ppu.oam.clone();
    completed_dma[40] = u16::from_le_bytes([0x38, 0xf0]);
    following.completed_oam_dma_after_capture = Some(completed_dma);
    following.oam_scanout_source = OamScanoutSource::RetainResidentPpuOam;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[40].to_le_bytes(), [0x38, 0x34]);
}

#[test]
fn interrupted_sprite_main_scanout_publishes_its_partial_shadow_dma() {
    let mut state = ZeldaState::new();
    state.ppu.oam.fill(0x1111);

    let mut following = captured_display_snapshot();
    following.obj_generation = DisplayObjGeneration::RetainCapturedOam {
        oam: vec![0x2222; following.ppu.oam.len()],
    };
    following.completed_oam_dma_after_capture = Some(vec![0x3333; following.ppu.oam.len()]);
    following.oam_scanout_source = OamScanoutSource::ComposeInterruptedSpriteMainShadowDma;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam, vec![0x2222; state.ppu.oam.len()]);
}

#[test]
fn resident_oam_scanout_does_not_advance_after_its_nmi_was_already_consumed() {
    let mut state = ZeldaState::new();
    state.ppu.oam[40] = u16::from_le_bytes([0x38, 0x34]);
    state.ram[crate::game_state::constants::FRAME_COUNTER] = 0x10;

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::FRAME_COUNTER] = 0x11;
    following.ram[crate::game_state::constants::NMI_BOOLEAN] = 0;
    let mut completed_dma = following.ppu.oam.clone();
    completed_dma[40] = u16::from_le_bytes([0x38, 0xf0]);
    following.completed_oam_dma_after_capture = Some(completed_dma);
    following.oam_scanout_source = OamScanoutSource::RetainResidentPpuOam;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[40].to_le_bytes(), [0x38, 0x34]);
}

#[test]
fn resident_oam_scanout_keeps_independent_live_link_generation() {
    let mut state = ZeldaState::new();
    state.ppu.oam[40] = u16::from_le_bytes([0x38, 0x34]);
    state.ram[crate::game_state::constants::FRAME_COUNTER] = 0x10;

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::FRAME_COUNTER] = 0x11;
    following.ram[crate::game_state::constants::NMI_BOOLEAN] = 1;
    let mut completed_dma = following.ppu.oam.clone();
    completed_dma[40] = u16::from_le_bytes([0x38, 0xf0]);
    following.completed_oam_dma_after_capture = Some(completed_dma);
    following.oam_scanout_source = OamScanoutSource::RetainResidentPpuOam;
    following.link_obj_scanout_generation = GraphicsDmaGeneration::LiveAfterMain;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[40].to_le_bytes(), [0x38, 0x34]);
}

#[test]
fn captured_oam_scanout_does_not_advance_to_a_later_completed_dma() {
    let mut state = ZeldaState::new();
    state.ppu.oam[40] = u16::from_le_bytes([0x38, 0x34]);

    let mut following = captured_display_snapshot();
    following.ppu.oam[40] = u16::from_le_bytes([0x38, 0x55]);
    let mut completed_dma = following.ppu.oam.clone();
    completed_dma[40] = u16::from_le_bytes([0x38, 0xf0]);
    following.completed_oam_dma_after_capture = Some(completed_dma);
    following.oam_scanout_source = OamScanoutSource::RetainCapturedBeforeNmi;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[40].to_le_bytes(), [0x38, 0x34]);
}

#[test]
fn captured_oam_scanout_rejects_a_later_dma_when_main_iteration_finishes() {
    let mut state = ZeldaState::new();
    state.ppu.oam[40] = u16::from_le_bytes([0x38, 0x34]);
    state.ram[crate::game_state::constants::FRAME_COUNTER] = 0x10;

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::FRAME_COUNTER] = 0x11;
    following.ram[crate::game_state::constants::NMI_BOOLEAN] = 0;
    let mut completed_dma = following.ppu.oam.clone();
    completed_dma[40] = u16::from_le_bytes([0x38, 0xf0]);
    following.completed_oam_dma_after_capture = Some(completed_dma);
    following.oam_scanout_source = OamScanoutSource::RetainCapturedBeforeNmi;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[40].to_le_bytes(), [0x38, 0x34]);
}

#[test]
fn captured_oam_scanout_with_pending_nmi_publishes_the_queued_shadow() {
    let mut state = ZeldaState::new();
    state.ppu.oam[40] = u16::from_le_bytes([0x38, 0x34]);
    state.ram[crate::game_state::constants::FRAME_COUNTER] = 0x10;

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::FRAME_COUNTER] = 0x11;
    following.ram[crate::game_state::constants::NMI_BOOLEAN] = 1;
    let mut completed_dma = following.ppu.oam.clone();
    completed_dma[40] = u16::from_le_bytes([0x38, 0xf0]);
    following.completed_oam_dma_after_capture = Some(completed_dma);
    let mut published_shadow = following.ppu.oam.clone();
    published_shadow[40] = u16::from_le_bytes([0x38, 0x55]);
    following.published_shadow_oam_dma = Some(published_shadow);
    following.oam_scanout_source = OamScanoutSource::RetainCapturedBeforeNmi;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[40].to_le_bytes(), [0x38, 0x55]);
}

#[test]
fn retained_oam_scanout_publishes_the_queue_after_a_completed_main_iteration() {
    let mut state = ZeldaState::new();
    state.ppu.oam[40] = u16::from_le_bytes([0x38, 0x34]);
    state.ram[crate::game_state::constants::FRAME_COUNTER] = 0x10;

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::FRAME_COUNTER] = 0x11;
    let mut published_shadow = following.ppu.oam.clone();
    published_shadow[40] = u16::from_le_bytes([0x38, 0xf0]);
    following.published_shadow_oam_dma = Some(published_shadow);
    following.oam_scanout_source = OamScanoutSource::RetainCapturedBeforeNmi;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.oam[40].to_le_bytes(), [0x38, 0xf0]);
}

#[test]
fn completed_oam_dma_records_the_exact_installed_generation() {
    let mut state = ZeldaState::new();
    state.frame_ctr_dbg = 17;
    let mut display = captured_display_snapshot();
    display.publication_host_frame = 17;
    display.published_shadow_oam_dma = Some(vec![0x1234; state.ppu.oam.len()]);
    state.display_snapshot = Some(Box::new(display));
    state.ppu.oam.fill(0x5678);

    state.record_completed_oam_dma_for_display_boundary();

    assert_eq!(
        state
            .display_snapshot
            .as_ref()
            .and_then(|display| display.completed_oam_dma_after_capture.as_ref())
            .map(|oam| oam[0]),
        Some(0x5678),
    );

    state.ppu.oam.fill(0xabcd);
    state.record_completed_oam_dma_for_display_boundary();
    assert_eq!(
        state
            .display_snapshot
            .as_ref()
            .and_then(|display| display.completed_oam_dma_after_capture.as_ref())
            .map(|oam| oam[0]),
        Some(0xabcd),
    );

    state.close_display_boundary_dma_receipts();
    state.ppu.oam.fill(0xdef0);
    state.record_completed_oam_dma_for_display_boundary();
    assert_eq!(
        state
            .display_snapshot
            .as_ref()
            .and_then(|display| display.completed_oam_dma_after_capture.as_ref())
            .map(|oam| oam[0]),
        Some(0xabcd),
    );
}

#[test]
fn hud_dma_inherits_the_persistent_oam_target_without_touching_vram() {
    let mut state = ZeldaState::new();
    let destination = 0x6040;
    state.ppu.vram[destination..destination + HUD_TILEMAP_NMI_WORDS].fill(0x7f7f);
    state.ppu.oam_adr = 0;
    state.ppu.oam_second_write = false;
    state.program_dma0_ppu_target(0, 0x04);
    let source = vec![0x5a; HUD_TILEMAP_NMI_WORDS * 2];

    state.complete_hud_dma_from_persistent_channel0(&source, destination);

    assert!(
        state.ppu.vram[destination..destination + HUD_TILEMAP_NMI_WORDS]
            .iter()
            .all(|&word| word == 0x7f7f)
    );
    assert_eq!(state.ppu.oam[0], 0x5a5a);
    assert_eq!(state.dma.channel[0].mode, 0);
    assert_eq!(state.dma.channel[0].b_adr, 0x04);
    assert_eq!(state.dma.channel[0].a_bank, 0x7e);
    assert_eq!(state.dma.channel[0].size, 0);
}

#[test]
fn programmed_vram_dma_target_is_reused_by_the_following_hud_upload() {
    let mut state = ZeldaState::new();
    let destination = 0x6040;
    state.program_dma0_ppu_target(0, 0x04);
    state.copy_to_vram_slice(0x4000, &[0x12, 0x34], 2);
    assert_eq!(state.dma.channel[0].mode, 1);
    assert_eq!(state.dma.channel[0].b_adr, 0x18);

    let mut source = vec![0; HUD_TILEMAP_NMI_WORDS * 2];
    source[0] = 0x1e;
    source[1] = 0x25;
    state.complete_hud_dma_from_persistent_channel0(&source, destination);

    assert_eq!(state.ppu.vram[destination], 0x251e);
    assert_eq!(state.dma.channel[0].mode, 1);
    assert_eq!(state.dma.channel[0].b_adr, 0x18);
}

#[test]
fn explicit_published_shadow_scanout_still_uses_the_software_shadow() {
    let mut state = ZeldaState::new();
    state.ppu.oam[40] = u16::from_le_bytes([0x38, 0x34]);

    let mut following = captured_display_snapshot();
    let mut published_shadow = following.ppu.oam.clone();
    published_shadow[40] = u16::from_le_bytes([0x38, 0xf0]);
    following.published_shadow_oam_dma = Some(published_shadow);
    following.oam_scanout_source = OamScanoutSource::ComposePublishedShadowDma;
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
        ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
            gfx: 0x14,
            ground_apress_tail: None,
        },
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
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                gfx: 0x14,
                ground_apress_tail: None
            },
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
    let gfx_21 = ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
        gfx: 0x21,
        ground_apress_tail: None,
    };

    assert!(state.item_receipt_graphics_return_uses_ordinary_module_epilogue(gfx_21));
    state.follower_link_state_mut().set_handler_state(21);
    assert!(!state.item_receipt_graphics_return_uses_ordinary_module_epilogue(gfx_21));
    state.follower_link_state_mut().set_handler_state(0);
    assert!(state.item_receipt_graphics_return_uses_ordinary_module_epilogue(gfx_21));

    for gfx in [0x14, 0x22, 0x24] {
        assert!(
            !state.item_receipt_graphics_return_uses_ordinary_module_epilogue(
                ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                    gfx,
                    ground_apress_tail: None
                },
            )
        );
    }
}

#[test]
fn uncle_sword_item_return_stages_the_prepared_oam_for_the_next_boundary() {
    let mut state = ZeldaState::new();
    state.capture_display_snapshot();
    state.ram[OAM_BUF..OAM_BUF + 4].copy_from_slice(&[0x11, 0x22, 0x33, 0x44]);

    state.stage_atomic_item_graphics_return_obj_scanout(
        ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
            gfx: 0x24,
            ground_apress_tail: None,
        },
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
fn interrupted_sprite_main_stages_the_exact_partial_oam_shadow_dma() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.ram[OAM_BUF..OAM_BUF + 4].copy_from_slice(&[0x11, 0x22, 0x33, 0x44]);

    state.stage_interrupted_sprite_main_oam_scanout();

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
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposeInterruptedSpriteMainShadowDma,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        })
    );
}

#[test]
fn interrupted_sprite_main_with_latched_nmi_retains_resident_oam() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.ram[NMI_BOOLEAN] = 1;
    state.ram[OAM_BUF..OAM_BUF + 4].copy_from_slice(&[0x11, 0x22, 0x33, 0x44]);

    state.stage_interrupted_sprite_main_oam_scanout();

    assert_eq!(state.next_display_obj_memory_generation, None);
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        })
    );
}

#[test]
fn resumed_sprite_main_return_keeps_trailing_nmi_out_of_active_field() {
    let mut state = ZeldaState::new();

    state.stage_resumed_sprite_main_return_obj_scanout();

    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        })
    );
}

#[test]
fn completed_gfx_24_return_publishes_live_animated_tiles() {
    let mut state = ZeldaState::new();
    state.capture_display_snapshot();
    let continuation = ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
        gfx: 0x24,
        ground_apress_tail: None,
    };

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
            link_oam: None,
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
            continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                gfx: 0x14,
                ground_apress_tail: None,
            },
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
fn room_72_northward_palette_tail_decodes_the_following_link_cache() {
    let mut state = ZeldaState::new();
    state.ppu.vram[0x4020] = 0x2222;

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 1;
    following.ram[crate::game_state::constants::SUBSUBMODULE] = 7;
    following.ram[crate::game_state::constants::DUNGEON_ROOM] = 0x72;
    following.ram[crate::game_state::constants::LINK_LAST_DIRECTION] = 8;
    following.ppu.vram[0x4020] = 0xbbbb;
    following.oam_scanout_source = OamScanoutSource::RetainResidentPpuOam;
    following.link_obj_scanout_generation = GraphicsDmaGeneration::LiveAfterMain;
    following.link_obj_source_generation = GraphicsDmaGeneration::LiveAfterMain;
    let frame = crate::game_state::FrameState::load_from_ram(&following.ram);
    assert!(room_72_northward_subtile_palette_tail_uses_live_obj_cache(
        frame, 0x72, 8,
    ));
    assert!(!room_72_northward_subtile_palette_tail_uses_live_obj_cache(
        frame, 0x72, 4,
    ));
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.vram[0x4020], 0x2222);
    assert_eq!(state.ppu.obj_vram_latch.as_ref().unwrap()[0x4020], 0xbbbb);
}

#[test]
fn room_72_northward_shutter_retains_the_presented_link_cache() {
    let mut state = ZeldaState::new();
    state.ppu.vram[0x4020] = 0x2222;
    state.last_presented_obj_vram = Some(vec![0x7777; state.ppu.vram.len()]);
    state.last_presented_obj_vram.as_mut().unwrap()[0x4020] = 0xaaaa;

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 5;
    following.ram[crate::game_state::constants::SUBSUBMODULE] = 0;
    following.ram[crate::game_state::constants::DUNGEON_ROOM] = 0x72;
    following.ram[crate::game_state::constants::LINK_LAST_DIRECTION] = 8;
    following.ppu.vram[0x4020] = 0xbbbb;
    following.oam_scanout_source = OamScanoutSource::RetainResidentPpuOam;
    following.link_obj_scanout_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    following.link_obj_source_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    let frame = crate::game_state::FrameState::load_from_ram(&following.ram);
    assert!(room_72_northward_subtile_shutter_retains_presented_obj_cache(frame, 0x72, 8,));
    assert!(!room_72_northward_subtile_shutter_retains_presented_obj_cache(frame, 0x72, 4,));
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_oam(&following, &plan);

    assert_eq!(state.ppu.vram[0x4020], 0x2222);
    assert_eq!(state.ppu.obj_vram_latch.as_ref().unwrap()[0x4020], 0xaaaa);
}

#[test]
fn held_subtile_nmi_reuses_the_last_presented_link_obj_cache() {
    let mut state = ZeldaState::new();
    state.ppu.vram[0x4020] = 0x2222;
    state.last_presented_obj_vram = Some(vec![0x7777; state.ppu.vram.len()]);
    state.last_presented_obj_vram.as_mut().unwrap()[0x4020] = 0xaaaa;
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
fn brightness_entry_authors_retained_link_pages_from_captured_dma_operands() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.set_subsubmodule(0);
    write_le_u16(
        &mut state.ram,
        crate::game_state::constants::LINK_DMA_GRAPHICS_INDEX,
        10,
    );
    for (slot, source) in [
        (LinkDmaSourceSlot::BodyTop, 0x8040),
        (LinkDmaSourceSlot::BodyBottom, 0x8140),
        (LinkDmaSourceSlot::HeadTop, 0x8240),
        (LinkDmaSourceSlot::HeadBottom, 0x8340),
        (LinkDmaSourceSlot::HandLeft, 0x8440),
        (LinkDmaSourceSlot::HandRight, 0x8540),
    ] {
        write_le_u16(&mut state.ram, slot.ram_address(), source);
    }
    let mut ranges = vec![(0, 0); 58];
    ranges[57] = (0, 0x600);
    state.assets = Some(AssetPack::from_data_ranges(vec![0; 0x600], ranges));
    let entry_frame = state.game_state.frame;
    state.pre_main_graphics_dma = Some(PreMainGraphicsDma {
        entry_frame,
        entry_plan: rom_graphics_dma_plan_at_host_boundary(entry_frame),
        entry_link_handler_state: 0,
        animated_tile: None,
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        obj_vram: state.ppu.vram.clone(),
        oam_shadow: vec![0; state.ppu.oam.len() * 2],
    });

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 0x0a;
    following.ram[crate::game_state::constants::SUBSUBMODULE] = 0;
    following.ram[crate::game_state::constants::DUNGEON_ROOM] = 0x41;
    following
        .vram_chr_source
        .record_tiles_from(0x4020, 1, crate::chr_source::CHR_KIND_LINK, 5, 6);
    following
        .vram_chr_source
        .record_tiles_from(0x4120, 1, crate::chr_source::CHR_KIND_LINK, 7, 8);
    following.vram_chr_source.record_tiles_from(
        0x4200,
        1,
        crate::chr_source::CHR_KIND_SPRITE,
        9,
        10,
    );
    let live_other = following.vram_chr_source.get(0x420);
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_chr_sources(&following, &plan);

    assert_eq!(
        state.vram_chr_source.get(0x400),
        crate::chr_source::LogicalChrSrc {
            kind: crate::chr_source::CHR_KIND_LINK,
            pack: 5,
            tile_off: 2,
        }
    );
    assert_eq!(
        state.vram_chr_source.get(0x402),
        crate::chr_source::LogicalChrSrc {
            kind: crate::chr_source::CHR_KIND_LINK,
            pack: 5,
            tile_off: 18,
        }
    );
    assert_eq!(
        state.vram_chr_source.get(0x410),
        crate::chr_source::LogicalChrSrc {
            kind: crate::chr_source::CHR_KIND_LINK,
            pack: 5,
            tile_off: 10,
        }
    );
    assert_eq!(
        state.vram_chr_preview_source.get(0x412),
        crate::chr_source::LogicalChrSrc {
            kind: crate::chr_source::CHR_KIND_LINK,
            pack: 5,
            tile_off: 26,
        }
    );
    assert_eq!(state.vram_chr_source.get(0x420), live_other);
}

#[test]
fn brightness_entry_retains_the_player_palette_until_the_next_nmi() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.set_subsubmodule(0);
    state.ppu.cgram[0x91] = 0x1234;

    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 0x0a;
    following.ram[crate::game_state::constants::SUBSUBMODULE] = 0;
    following.ram[crate::game_state::constants::DUNGEON_ROOM] = 0x41;
    following.ppu.cgram[0x91] = 0x5678;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());
    assert!(plan.compose_live_cgram);

    state.compose_display_cgram(&following, &plan);

    assert_eq!(state.ppu.cgram[0x91], 0x1234);
}

#[test]
fn room_72_state8_scroll_decodes_live_link_pages_after_first_rom_tick() {
    let mut following = captured_display_snapshot();
    following.ram[crate::game_state::constants::MAIN_MODULE] = 7;
    following.ram[crate::game_state::constants::SUBMODULE] = 2;
    following.ram[crate::game_state::constants::SUBSUBMODULE] = 8;
    following.ram[crate::game_state::constants::DUNGEON_ROOM] = 0x72;
    following.ppu.vram[0x4030] = 0xaaaa;
    following.oam_scanout_source = OamScanoutSource::ComposePublishedShadowDma;
    following.link_obj_scanout_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;
    following.link_obj_source_generation = GraphicsDmaGeneration::HostBoundaryBeforeMain;

    following.ram[crate::game_state::constants::FRAME_COUNTER] = 1;
    for countdown in [6, 4, 3, 2, 1] {
        let mut state = ZeldaState::new();
        state.ppu.vram[0x4030] = 0x1111;
        state.set_screen_transition(1);
        write_le_u16(&mut following.ram, LINK_DMA_COUNTDOWN, countdown);
        let plan =
            DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

        state.compose_display_oam(&following, &plan);

        assert_eq!(state.ppu.vram[0x4030], 0x1111);
        assert_eq!(state.ppu.obj_vram_latch.as_ref().unwrap()[0x4030], 0xaaaa);
    }

    let mut first_scroll = ZeldaState::new();
    first_scroll.ppu.vram[0x4030] = 0x1111;
    first_scroll.set_screen_transition(1);
    following.ram[crate::game_state::constants::FRAME_COUNTER] = 0;
    write_le_u16(&mut following.ram, LINK_DMA_COUNTDOWN, 5);
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());
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
    entry_oam_shadow[LINK_BODY_BYTE..LINK_BODY_BYTE + 2].copy_from_slice(&0x1111u16.to_le_bytes());
    state.pre_main_graphics_dma = Some(PreMainGraphicsDma {
        entry_frame,
        entry_plan: rom_graphics_dma_plan_at_host_boundary(entry_frame),
        entry_link_handler_state: 0,
        animated_tile: None,
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        obj_vram: state.ppu.vram.clone(),
        oam_shadow: entry_oam_shadow,
    });
    // Exercise the generic state-$03 cache branch that previously overwrote a
    // room-specific publication placed earlier in compose_display_oam.
    state.last_presented_obj_vram = Some(state.ppu.vram.clone());

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
        entry_link_handler_state: 0,
        animated_tile: None,
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        obj_vram: state.ppu.vram.clone(),
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
        entry_link_handler_state: 0,
        animated_tile: None,
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        obj_vram: state.ppu.vram.clone(),
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
        entry_link_handler_state: 0,
        animated_tile: None,
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        obj_vram: state.ppu.vram.clone(),
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
        write_le_u16(&mut ram, slot.ram_address(), 0x8000 + source_offset as u16);
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

    for (index, (destination, _, len)) in EARLY_LINK_OBJ_DMA_TRANSFERS.iter().copied().enumerate() {
        assert!(composed[destination..destination + len / 2]
            .iter()
            .all(|&word| word == 0x1100 + index as u16));
    }
    assert_eq!(composed[0x4060], 0x7777);
}

#[test]
fn link_cache_changes_only_from_a_completed_dma_receipt() {
    let mut completed_ram = vec![0; 0x20000];
    write_le_u16(
        &mut completed_ram,
        LinkDmaSourceSlot::HeadTop.ram_address(),
        0xcd80,
    );
    let completed = LinkDmaSources::load_from_ram(&completed_ram);
    let completed_live = CompletedLinkObjDma {
        sources: completed,
        source_generation: GraphicsDmaGeneration::LiveAfterMain,
    };
    let completed_host = CompletedLinkObjDma {
        sources: completed,
        source_generation: GraphicsDmaGeneration::HostBoundaryBeforeMain,
    };

    assert_eq!(link_obj_cache_sources_for_publication(None, false,), None,);
    assert_eq!(
        link_obj_cache_sources_for_publication(Some(completed_live), false,),
        Some(completed),
    );
    assert_eq!(link_obj_cache_sources_for_publication(None, false,), None,);
    assert_eq!(
        link_obj_cache_sources_for_publication(Some(completed_live), false,),
        Some(completed),
    );
    assert_eq!(
        link_obj_cache_sources_for_publication(Some(completed_live), true,),
        None,
    );
    assert_eq!(
        link_obj_cache_sources_for_publication(Some(completed_live), true,),
        None,
    );
    assert_eq!(
        link_obj_cache_sources_for_publication(Some(completed_host), false,),
        None,
    );
}

#[test]
fn ordinary_trailing_link_dma_does_not_reenter_the_captured_display_owner() {
    let mut state = ZeldaState::new();
    state.capture_display_snapshot();
    let mut ram = state.ram.clone();
    write_le_u16(&mut ram, LinkDmaSourceSlot::HeadTop.ram_address(), 0xcd80);
    let completed_sources = LinkDmaSources::load_from_ram(&ram);

    state.record_completed_link_obj_dma_for_display_boundary(
        completed_sources,
        GraphicsDmaGeneration::LiveAfterMain,
    );

    assert!(state
        .display_snapshot
        .as_ref()
        .unwrap()
        .effective_presented_dma
        .is_none());
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
fn dungeon_brightness_boundary_publishes_completed_nmi_copy_packets() {
    const DESTINATION: usize = 0x0798;
    let packet_base = crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF;
    let mut state = ZeldaState::new();
    state.ppu.vram[DESTINATION] = 0x0de0;
    write_le_u16(&mut state.ram, packet_base, DESTINATION as u16);
    state.ram[packet_base + 2] = 0x80;
    state.ram[packet_base + 3] = 2;
    write_le_u16(&mut state.ram, packet_base + 4, 0x0dc0);
    write_le_u16(&mut state.ram, packet_base + 6, 0xffff);
    state.ram[NMI_COPY_PACKETS_FLAG] = 1;

    let mut following = captured_display_snapshot();
    following.ppu.vram[DESTINATION] = 0x0dc0;
    let plan = DisplayPublicationPlan::resolve(
        &following,
        DisplayPublicationSignals {
            dungeon_brightness_publishes_live_display: true,
            ..DisplayPublicationSignals::default()
        },
    );

    state.compose_display_vram(&following, &plan, None);

    assert!(plan.publish_live_nmi_copy_packets);
    assert_eq!(state.ppu.vram[DESTINATION], 0x0dc0);
}

#[test]
fn dungeon_brightness_animated_bg_follows_the_animation_countdown_phase() {
    let brightness = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0x0a,
        ..Default::default()
    };
    // f28358: the countdown has just reloaded, so the freshly advanced page
    // lands too late and the host-boundary generation stays on screen.
    assert!(!dungeon_brightness_animated_bg_is_live(
        brightness, brightness, 9
    ));
    // f28602: mid-cycle, the upload completes before scanout.
    assert!(dungeon_brightness_animated_bg_is_live(
        brightness, brightness, 8
    ));
    assert!(dungeon_brightness_animated_bg_is_live(
        brightness, brightness, 1
    ));
    // Outside the brightness phase the countdown alone must not publish live.
    assert!(!dungeon_brightness_animated_bg_is_live(
        crate::game_state::FrameState {
            main_module: 7,
            submodule: 0,
            ..Default::default()
        },
        brightness,
        8,
    ));
}

#[test]
fn dungeon_brightness_retains_the_host_boundary_animated_bg() {
    // The dungeon brightness phase publishes its screen layers, HUD DMA and
    // NMI copy packets live, but the animated-BG domain stays independent: at
    // f28358 (room $41, module 7/$0a, animation countdown wrapping 0x01->0x09)
    // Snes9x scans out the host-boundary generation while the leading-NMI tile
    // DMA is still landing. Coupling this domain to the brightness signal
    // republished the live post-NMI tiles and diverged 442 VRAM bytes across
    // the whole retained $200-word block at $3b00.
    const DESTINATION: usize = 0x3b00;
    const TILE_WORD: usize = 0x3c00;
    let mut state = ZeldaState::new();
    state.set_animated_tile_vram_destination_address(DESTINATION as u16);
    state.ppu.vram[TILE_WORD] = 0x1111;
    let mut retained = vec![0; 0x200];
    retained[TILE_WORD - DESTINATION] = 0x2222;
    let mut retained_logical_sources = crate::chr_source::VramChrSourceTable::default();
    retained_logical_sources.record_tile_content_hash(
        TILE_WORD / 16,
        crate::chr_source::CHR_KIND_BG_STREAM,
        0x1122_3344,
    );
    let retained_scanout = AnimatedBgScanout {
        destination_address: DESTINATION,
        vram: retained,
        logical_sources: retained_logical_sources,
        preview_sources: crate::chr_source::VramChrSourceTable::default(),
    };
    state.pre_nmi_animated_bg_scanout = Some(retained_scanout.clone());

    let mut following = captured_display_snapshot();
    write_le_u16(
        &mut following.ram,
        ANIMATED_TILE_VRAM_ADDR,
        DESTINATION as u16,
    );
    following.ppu.vram[TILE_WORD] = 0x3333;
    following.animated_bg_scanout_generation = AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi;
    following.host_boundary_animated_bg_scanout = Some(retained_scanout);
    let plan = DisplayPublicationPlan::resolve(
        &following,
        DisplayPublicationSignals {
            dungeon_brightness_publishes_live_display: true,
            ..DisplayPublicationSignals::default()
        },
    );

    state.compose_display_vram(&following, &plan, None);
    state.compose_display_chr_sources(&following, &plan);

    assert_eq!(
        plan.animated_bg_scanout_generation,
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
    );
    assert_eq!(state.ppu.vram[TILE_WORD], 0x2222);
    assert_eq!(
        state.vram_chr_source.get(TILE_WORD / 16),
        following
            .host_boundary_animated_bg_scanout
            .as_ref()
            .unwrap()
            .logical_sources
            .get(TILE_WORD / 16),
    );
}

#[test]
fn retained_general_vram_still_publishes_the_snapshot_owned_animated_region() {
    const DESTINATION: usize = 0x3b00;
    const TILE_WORD: usize = DESTINATION + 0x100;
    let mut state = ZeldaState::new();
    state.ppu.vram[TILE_WORD] = 0x1111;

    let mut logical_sources = crate::chr_source::VramChrSourceTable::default();
    logical_sources.record_tile_content_hash(
        TILE_WORD / 16,
        crate::chr_source::CHR_KIND_BG_STREAM,
        0xaabb_ccdd,
    );
    let scanout = AnimatedBgScanout {
        destination_address: DESTINATION,
        vram: {
            let mut words = vec![0; 0x200];
            words[TILE_WORD - DESTINATION] = 0x2222;
            words
        },
        logical_sources,
        preview_sources: crate::chr_source::VramChrSourceTable::default(),
    };
    let mut following = captured_display_snapshot();
    following.vram_generation = DisplayVramGeneration::RetainCapturedBeforeNmi;
    following.animated_bg_scanout_generation = AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi;
    following.host_boundary_animated_bg_scanout = Some(scanout.clone());
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_vram(&following, &plan, None);
    state.compose_display_chr_sources(&following, &plan);

    assert_eq!(state.ppu.vram[TILE_WORD], 0x2222);
    assert_eq!(
        state.vram_chr_source.get(TILE_WORD / 16),
        scanout.logical_sources.get(TILE_WORD / 16),
    );
}

#[test]
fn pending_hud_authorship_never_guesses_that_nmi_already_published_it() {
    const DESTINATION: usize = 0x6040;

    for (room, staircase) in [(0x01, 0x30), (0x48, 0x34)] {
        let mut state = ZeldaState::new();
        state.set_main_module(7);
        state.set_dungeon_room_index(room);
        state
            .dungeon_stair_movement_mut()
            .set_staircase_index(staircase);
        state.set_message_dma_destination_address(DESTINATION as u16);
        state.set_hud_floor_changed_timer(0);
        state.set_hud_tile_word(0x79, 0x007f);
        state.ppu.vram[DESTINATION + 0x79] = 0x2508;
        state.increment_hud_update_flag();
        state.capture_display_snapshot();

        assert_eq!(
            state.display_snapshot.as_ref().unwrap().hud_vram_generation,
            DisplayVramGeneration::RetainCapturedBeforeNmi,
            "queued HUD authorship is not evidence of a completed NMI DMA",
        );
    }
}

#[test]
fn spiral_state_8_hud_dma_composes_over_retained_general_vram() {
    let mut state = ZeldaState::new();
    let destination = 0x6040;
    let floor_label_word = 0x60b9;
    state.ppu.vram[0] = 0x1111;
    state.ppu.vram[floor_label_word] = 0x207f;

    let mut following = captured_display_snapshot();
    following.ppu.vram[0] = 0xaaaa;
    following.ppu.vram[floor_label_word] = 0x2508;
    following.vram_generation = DisplayVramGeneration::RetainCapturedBeforeNmi;
    following.hud_vram_generation = DisplayVramGeneration::ComposeLiveAfterNmi;
    following.hud_vram_destination = destination;
    let plan = DisplayPublicationPlan::resolve(&following, DisplayPublicationSignals::default());

    state.compose_display_vram(&following, &plan, None);

    assert_eq!(state.ppu.vram[0], 0x1111);
    assert_eq!(state.ppu.vram[floor_label_word], 0x2508);
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
        OamScanoutSource::ComposeCompletedWorkAfterNmi
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
fn staged_promotion_retains_oam_only_when_the_c_iteration_is_still_resuming() {
    let staged = OamScanoutSource::ComposeCompletedWorkAfterNmi;
    assert_eq!(
        oam_scanout_source_for_staged_promotion(false, staged),
        OamScanoutSource::RetainPreviousPresented,
    );
    assert_eq!(
        oam_scanout_source_for_staged_promotion(true, staged),
        staged,
    );
    assert_eq!(
        oam_scanout_source_for_staged_promotion(false, OamScanoutSource::ComposeLiveAfterNmi,),
        OamScanoutSource::RetainPreviousPresented,
    );
    for retained in [
        OamScanoutSource::RetainCapturedBeforeNmi,
        OamScanoutSource::RetainImmutableCapturedPpu,
        OamScanoutSource::RetainPreviousPresented,
        OamScanoutSource::RetainResidentPpuOam,
        OamScanoutSource::ComposeInterruptedSpriteMainShadowDma,
    ] {
        assert_eq!(
            oam_scanout_source_for_staged_promotion(false, retained),
            retained,
        );
    }
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
fn queued_published_oam_source_uses_its_host_boundary_payload() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.capture_display_snapshot();
    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .published_shadow_oam_dma = Some(vec![0x1111; state.ppu.oam.len()]);

    let entry_frame = state.game_state.frame;
    state.pre_main_graphics_dma = Some(PreMainGraphicsDma {
        entry_frame,
        entry_plan: rom_graphics_dma_plan_at_host_boundary(entry_frame),
        entry_link_handler_state: 0,
        animated_tile: None,
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        obj_vram: state.ppu.vram.clone(),
        oam_shadow: vec![0x22; state.ppu.oam.len() * 2],
    });
    state.next_display_obj_scanout_generation = Some(ObjScanoutGenerations {
        oam: OamScanoutSource::ComposePublishedShadowDma,
        link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
    });

    state.capture_display_snapshot();

    let snapshot = state.display_snapshot.as_ref().unwrap();
    assert_eq!(
        snapshot.oam_scanout_source,
        OamScanoutSource::ComposePublishedShadowDma
    );
    assert!(snapshot
        .published_shadow_oam_dma
        .as_ref()
        .unwrap()
        .iter()
        .all(|&word| word == 0x2222));
}

#[test]
fn enemy_drop_item_graphics_keep_the_resident_full_obj_cache() {
    let continuation = |gfx| GameWorkContinuation::FinishItemReceiptGraphics {
        continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
            gfx,
            ground_apress_tail: None,
        },
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
    let continuation = ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
        gfx: 0x22,
        ground_apress_tail: None,
    };

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
fn game_over_iris_goal_snapshot_publishes_a_black_scanout() {
    let mut state = ZeldaState::new();
    state.ppu.brightness = 15;
    state.ppu.refresh_brightness_cache();
    state.capture_display_snapshot();
    {
        let display = state.display_snapshot.as_mut().unwrap();
        display.game_over_iris_goal_scanout_closed = true;
        display.ppu.brightness = 15;
        display.ppu.refresh_brightness_cache();
    }

    let presented_brightness = state.with_display_snapshot(|display| display.ppu.brightness);

    assert_eq!(presented_brightness, 0);
    assert_eq!(state.ppu.brightness, 15);
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
