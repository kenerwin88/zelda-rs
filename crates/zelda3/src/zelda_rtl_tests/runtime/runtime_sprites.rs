//! ZeldaState runtime tests — sprites.
//! Split mechanically from the hub file; bodies unchanged.

use super::*;

#[test]
fn ancilla_slot_accessors_keep_native_state_and_ram_synced() {
    let mut state = ZeldaState::new();

    state.ancilla_slot_view_mut(3).set_x_velocity(0x80);
    state.ancilla_slot_view_mut(3).set_x_low(0x44);
    state.ancilla_slot_view_mut(0).increment_ancilla_type();

    assert_eq!(state.ancilla_slot_view(3).x_velocity(), 0x80);
    assert_eq!(state.ancilla_slot_view(3).x_low(), 0x44);
    assert_eq!(state.ancilla_slot_view(0).ancilla_type(), 1);
    assert_eq!(state.ram[ANCILLA_X_VELOCITY + 3], 0x80);
    assert_eq!(state.ram[ANCILLA_X_LO + 3], 0x44);
    assert_eq!(state.ram[ANCILLA_TYPE], 1);
}

#[test]
fn credits_module_sets_oam_region_words_like_c() {
    let mut state = ZeldaState::new();
    state.set_submodule(38);
    for offset in 0..6 {
        state.ram[0x0fe0 + offset] = 0xff;
    }

    state.module1_a_credits();

    assert_eq!(read_le_u16(&state.ram, 0x0fe0), 0x0030);
    assert_eq!(read_le_u16(&state.ram, 0x0fe2), 0x01d0);
    assert_eq!(read_le_u16(&state.ram, 0x0fe4), 0x0000);
}

#[test]
fn credits_prep_resets_sprite_properties_before_scene_setup() {
    let mut state = ZeldaState::new();
    state.set_submodule(0);
    let k = 15;
    for base in [
        SPRITE_PAUSE,
        SPRITE_E,
        SPRITE_X_VEL,
        SPRITE_Y_VEL,
        SPRITE_AI_STATE,
        SPRITE_A,
        SPRITE_DELAY_MAIN,
        SPRITE_OAM_FLAGS,
        SPRITE_STATE,
        SPRITE_FLAGS5,
        SPRITE_DEFL_BITS,
    ] {
        state.ram[base + k] = 0xa5;
    }

    state.credits_prep_and_load_sprites();

    for base in [
        SPRITE_PAUSE,
        SPRITE_E,
        SPRITE_X_VEL,
        SPRITE_Y_VEL,
        SPRITE_AI_STATE,
        SPRITE_A,
        SPRITE_DELAY_MAIN,
        SPRITE_OAM_FLAGS,
        SPRITE_STATE,
        SPRITE_FLAGS5,
        SPRITE_DEFL_BITS,
    ] {
        assert_eq!(state.ram[base + k], 0, "base ${base:04x}");
    }
}

#[test]
fn lanmola_draw_uses_named_flat_trail_reader() {
    let source = include_str!("../../sprite_main_draw.rs");
    for needle in [
        "self.ram[MOLDORM_HISTORY_X_LO +",
        "self.ram[MOLDORM_HISTORY_Y_LO +",
        "self.ram[BEAMOS_LASER_HISTORY_X_HI +",
        "self.ram[BEAMOS_LASER_HISTORY_Y_HI +",
    ] {
        assert!(
            !source.contains(needle),
            "sprite_main_draw.rs should use lanmola_flat_trail_entry instead of {needle}"
        );
    }
    assert!(
        source.contains("lanmola_flat_trail_entry("),
        "sprite_main_draw.rs should route Lanmola trail reads through the named API"
    );
}

#[test]
fn run4786_link_oam_completes_the_build_before_run4787_finishes_its_suffix() {
    let mut state = run4786_spotlight_build_link_oam_state();
    let iteration = SpotlightIteration::closing(SpotlightIterationPhase::WholeTable);
    let run4786_epoch = state.display_snapshot_epoch;

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonExitSpotlightLinkOam {
            iteration: pending,
        }) if pending == iteration
    ));
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 112);
    assert_eq!(read_le_u16(&state.ram, LINK_DMA_COUNTDOWN), 9);
    assert_eq!(state.game_state.display.bg_tile_animation_countdown, 7);
    assert_eq!(state.display_snapshot_epoch, run4786_epoch + 2);
    assert_eq!(
        read_le_u16(
            &state
                .display_snapshot
                .as_ref()
                .expect("run4786 lost its active Held-boundary capture")
                .ram,
            SPOTLIGHT_WINDOW_RADIUS,
        ),
        119,
        "the active run4786 field must retain its pre-Build spotlight radius",
    );
    assert!(
        state.deferred_display_snapshot.is_none(),
        "WholeTable completion retains the active publication instead of staging a second field",
    );
    assert!(matches!(
        state
            .display_snapshot
            .as_ref()
            .expect("run4786 lost its retained active publication")
            .hdma_table_generation,
        DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout { .. }
    ));
    assert!(
        !state
            .display_snapshot
            .as_ref()
            .expect("run4786 lost its retained active publication")
            .accepts_nmi_dma_receipts
    );
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(!state.main_loop_sprite_preparation_completed);
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert!(state.original_timing_semantic_receipts.is_none());
    assert_eq!(
        state.with_display_snapshot(|display| {
            read_le_u16(&display.ram, SPOTLIGHT_WINDOW_RADIUS)
        }),
        119,
        "run4786 must present its retained active field while live state owns the completed Build",
    );
    assert_eq!(
        read_le_u16(
            &state
                .display_snapshot
                .as_ref()
                .expect("run4786 lost its retained active generation")
                .ram,
            SPOTLIGHT_WINDOW_RADIUS,
        ),
        119,
    );

    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            4_787,
            0,
            run4784_terminal_spotlight_semantic(),
        ))
        .unwrap();
    let run4787_epoch = state.display_snapshot_epoch;
    let radius_before_return = state.game_state.display.spotlight_hdma.window_radius();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(state.main_loop_sprite_preparation_completed);
    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        radius_before_return,
        "the terminal LinkOam suffix must not replay the spotlight table body",
    );
    assert_eq!(read_le_u16(&state.ram, LINK_DMA_COUNTDOWN), 8);
    assert_eq!(state.game_state.display.bg_tile_animation_countdown, 6);
    assert_eq!(state.display_snapshot_epoch, run4787_epoch + 1);
    assert_eq!(
        read_le_u16(
            &state
                .display_snapshot
                .as_ref()
                .expect("run4787 lost its post-Build Held capture")
                .ram,
            SPOTLIGHT_WINDOW_RADIUS,
        ),
        112,
    );
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn live_cached_sprite_progress_is_taken_once_without_cpu_provenance() {
    let progress = crate::CachedSpriteExecutionProgress::Restoring {
        slot: 7,
        live_fields: 4,
    };
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![cached_sprite_progress_receipt(
            progress,
            OriginalTimingBoundary::NmiAccepted,
        )],
    ));

    assert_eq!(
        state.take_original_timing_cached_sprite_execution_progress(),
        Some(crate::CachedSpriteExecutionProgressReceipt {
            progress,
            boundary: OriginalTimingBoundary::NmiAccepted,
        }),
    );
    assert_eq!(
        state.take_original_timing_cached_sprite_execution_progress(),
        None,
    );
}

#[test]
fn live_sprite_main_call_stack_holds_then_advances_only_the_observed_slot_delta() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(7);
    state.set_submodule(0);
    state.set_subsubmodule(0);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    for slot in 0..=4 {
        state.sprite_slot_view_mut(slot).set_state(8);
        state.sprite_slot_view_mut(slot).set_sprite_type(0x6d);
    }
    // The active continuation begins after slot 4's source call returned.
    state.sprite_slot_view_mut(4).set_state(9);
    state.active_dungeon_sprite_main_return = Some(DungeonSpriteMainReturn {
        link_oam: None,
        bg2_x: 0,
        bg2_y: 0,
        bg1_x: 0,
        bg1_y: 0,
    });
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterSlot(4),
            caller: SpriteMainCpuCaller::DungeonModule07,
        },
        1,
    );
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        20_258,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::SpriteMainProgressed(
                crate::SpriteMainProgress::AfterSlot(4),
            ),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
        ],
    ));

    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterSlot(4),
            caller: SpriteMainCpuCaller::DungeonModule07,
        }),
    );
    assert_eq!(state.sprite_slot_view(3).state(), 8);

    let interruption = crate::MainLoopInterruption::SpriteMainAfterSlot(3);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        20_259,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(interruption),
        ],
    ));

    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert_eq!(state.sprite_slot_view(3).state(), 9);
    for slot in 0..3 {
        assert_eq!(state.sprite_slot_view(slot).state(), 8);
    }
    assert!(state.active_dungeon_sprite_main_return.is_some());
    assert!(state.original_timing_nmi_publication_pending);
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterSlot(3),
            caller: SpriteMainCpuCaller::DungeonModule07,
        }),
    );

    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        20_260,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
        ],
    ));

    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    for slot in 0..=4 {
        assert_eq!(state.sprite_slot_view(slot).state(), 9);
    }
    assert!(state.active_dungeon_sprite_main_return.is_none());
    assert!(state.game_execution_scheduler.is_idle());
}

#[test]
fn live_host_return_progress_establishes_the_sprite_main_owner_before_resume() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    for slot in 0..=4 {
        state.sprite_slot_view_mut(slot).set_state(8);
        state.sprite_slot_view_mut(slot).set_sprite_type(0x6d);
    }
    // The source receipt says slot 4 already returned and slot 3 is next.
    state.sprite_slot_view_mut(4).set_state(9);
    // The old reconstructed CPU plan may already have staged a coarser
    // checkpoint. The live semantic receipt must replace it before any native
    // Sprite_Main statement executes.
    state.sprite_main_cpu_boundary = Some(SpriteMainCpuBoundary::BeforeFirstSlot);
    state.sprite_main_cpu_nmi_slices = 3;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        20_257,
        0,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            crate::SpriteMainProgress::AfterSlot(4),
        )],
    ));

    state.run_module07_sprite_main_caller(DungeonSpriteMainReturn {
        link_oam: None,
        bg2_x: 0,
        bg2_y: 0,
        bg1_x: 0,
        bg1_y: 0,
    });

    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterSlot(4),
            caller: SpriteMainCpuCaller::DungeonModule07Live {
                boundary: crate::OriginalTimingBoundary::HostReturn,
            },
        }),
    );
    assert!(state.active_dungeon_sprite_main_return.is_some());
    assert_eq!(state.sprite_slot_view(3).state(), 8);
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn live_timer_oam_return_resumes_dispatch_without_replaying_timers() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(7);
    state.set_submodule(0);
    state.sprite_slot_view_mut(10).set_state(9);
    state.sprite_slot_view_mut(10).set_sprite_type(0x00);
    state.sprite_slot_view_mut(10).set_delay_aux1(2);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        666_274,
        0,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            crate::SpriteMainProgress::AfterTimersAndOam(10),
        )],
    ));

    state.run_module07_sprite_main_caller(DungeonSpriteMainReturn {
        link_oam: None,
        bg2_x: 0,
        bg2_y: 0,
        bg1_x: 0,
        bg1_y: 0,
    });

    assert_eq!(state.sprite_slot_view(10).delay_aux1(), 1);
    let boundary = match state.game_execution_scheduler.current_work() {
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary:
                SpriteMainCpuBoundary::AfterTimersAndOam {
                    slot: 10,
                    state: Some(9),
                },
            caller:
                SpriteMainCpuCaller::DungeonModule07Live {
                    boundary: crate::OriginalTimingBoundary::HostReturn,
                },
        }) => SpriteMainCpuBoundary::AfterTimersAndOam {
            slot: 10,
            state: Some(9),
        },
        other => panic!("unexpected timer/OAM continuation: {other:?}"),
    };

    state.complete_sprite_main_after_cpu_boundary(boundary);

    assert_eq!(state.sprite_slot_view(10).delay_aux1(), 1);
}

#[test]
fn sprite_main_source_checkpoint_identity_excludes_native_resume_payloads() {
    assert!(same_sprite_main_source_checkpoint(
        SpriteMainCpuBoundary::AfterTimersAndOam {
            slot: 12,
            state: None,
        },
        SpriteMainCpuBoundary::AfterTimersAndOam {
            slot: 12,
            state: Some(9),
        },
    ));
    assert!(same_sprite_main_source_checkpoint(
        SpriteMainCpuBoundary::AfterTimerDecrements {
            slot: 0,
            state: None,
        },
        SpriteMainCpuBoundary::AfterTimerDecrements {
            slot: 0,
            state: Some(4),
        },
    ));
    assert!(!same_sprite_main_source_checkpoint(
        SpriteMainCpuBoundary::AfterTimersAndOam {
            slot: 12,
            state: Some(9),
        },
        SpriteMainCpuBoundary::AfterTimersAndOam {
            slot: 11,
            state: Some(9),
        },
    ));
    assert!(direct_item_receipt_slot_pairs_with_boundary(
        12,
        SpriteMainCpuBoundary::AfterTimersAndOam {
            slot: 12,
            state: None,
        },
    ));
    assert!(!direct_item_receipt_slot_pairs_with_boundary(
        11,
        SpriteMainCpuBoundary::AfterTimersAndOam {
            slot: 12,
            state: None,
        },
    ));
}

#[test]
fn live_wallmaster_reset_prefix_publishes_before_resume_without_replaying_the_send() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(7);
    state.set_submodule(0);
    state.set_indoor_flag(1);
    state.set_bg2_v_copy2(0x0700);
    for slot in [1usize, 3, 5, 12] {
        state.sprite_slot_view_mut(slot).set_state(9);
        state
            .sprite_slot_view_mut(slot)
            .set_sprite_type(0x40 + slot as u8);
    }
    {
        let mut wallmaster = state.sprite_slot_view_mut(12);
        wallmaster.set_sprite_type(0x90);
        wallmaster.set_a(1);
        wallmaster.set_x(0x1262);
        wallmaster.set_y(0x070a);
        wallmaster.set_z(0);
        wallmaster.set_delay_main(0x20);
        wallmaster.set_pause(0);
        wallmaster.set_deflection_bits(0x80);
    }
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        665_511,
        0,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            crate::SpriteMainProgress::AfterWallmasterResetPrefix(12),
        )],
    ));

    state.run_module07_sprite_main_caller(DungeonSpriteMainReturn {
        link_oam: None,
        bg2_x: 0,
        bg2_y: 0x0700,
        bg1_x: 0,
        bg1_y: 0,
    });

    assert_eq!(state.game_state.player.follower_link.y(), 0x070d);
    assert!([1usize, 3, 5, 12]
        .into_iter()
        .all(|slot| state.sprite_slot_view(slot).state() == 0));
    assert_eq!(state.game_state.frame.main_module, 7);
    let boundary = match state.game_execution_scheduler.current_work() {
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterWallmasterResetPrefix(12),
            caller:
                SpriteMainCpuCaller::DungeonModule07Live {
                    boundary: crate::OriginalTimingBoundary::HostReturn,
                },
        }) => SpriteMainCpuBoundary::AfterWallmasterResetPrefix(12),
        other => panic!("unexpected Wallmaster reset continuation: {other:?}"),
    };

    state.complete_sprite_main_after_cpu_boundary(boundary);

    assert_eq!(state.game_state.player.follower_link.y(), 0x070d);
    assert_eq!(state.game_state.frame.main_module, 17);
}

#[test]
fn wallmaster_descending_clear_retains_the_unwritten_prefix_and_resumes_exactly() {
    for cleared_bytes in [0u16, 3170, 3171, 4096] {
        let mut state = ZeldaState::new();
        state.set_main_module(7);
        state.set_indoor_flag(1);
        for room in 0..2048 {
            state.sprite_workspace_mut().set_where_in_room(room, 0xa55a);
        }
        state.wall_master_send_player_through_reset_fixed_prefix();
        let mut atomic = state.clone();
        atomic.sprite_system_mut().set_cur_object_index(0);
        atomic.wall_master_send_player_after_reset_fixed_prefix();
        atomic.link_initialize();
        atomic.complete_sprite_main_after_interrupted_slot(0);

        let remaining = 4096 - usize::from(cleared_bytes);
        let original = state.ram[0x1df80..0x1ef80].to_vec();
        state
            .sprite_workspace_mut()
            .clear_where_in_room_range(remaining..4096);
        assert_eq!(
            &state.ram[0x1df80..0x1df80 + remaining],
            &original[..remaining]
        );
        assert!(state.ram[0x1df80 + remaining..0x1ef80]
            .iter()
            .all(|&byte| byte == 0));
        state.complete_sprite_main_after_cpu_boundary(
            SpriteMainCpuBoundary::WallmasterResetClear {
                slot: 0,
                cleared_bytes,
            },
        );
        assert_eq!(state.game_state, atomic.game_state);
        assert_eq!(state.ram, atomic.ram);
    }
}

#[test]
fn sprite_main_slot_boundary_refinement_and_resume_equal_atomic_c_order() {
    fn configured_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        for slot in 0..4 {
            state.sprite_slot_view_mut(slot).set_state(8);
            state.sprite_slot_view_mut(slot).set_sprite_type(0x6d);
        }
        state
    }

    let mut atomic = configured_state();
    atomic.complete_sprite_main_after_interrupted_slot(4);

    let mut refined = configured_state();
    refined.advance_sprite_main_after_slot_boundary(4, 3);
    refined.complete_sprite_main_after_interrupted_slot(3);

    assert_eq!(refined.ram, atomic.ram);
    assert_eq!(refined.game_state.sprites, atomic.game_state.sprites);
}

#[test]
fn sprite_main_entry_refinement_runs_higher_slots_and_stops_after_target_timers() {
    let mut state = ZeldaState::new();
    state.sprite_slot_view_mut(11).set_state(8);
    state.sprite_slot_view_mut(11).set_sprite_type(0x41);
    state.sprite_slot_view_mut(11).set_delay_aux1(2);

    let boundary = state.advance_sprite_main_before_first_slot_to_after_timers_and_oam(11);

    assert_eq!(
        boundary,
        SpriteMainCpuBoundary::AfterTimersAndOam {
            slot: 11,
            state: Some(8),
        },
    );
    assert_eq!(state.sprite_slot_view(11).delay_aux1(), 1);
    assert_eq!(
        state.sprite_slot_view(11).state(),
        8,
        "the state-8 dispatch must remain on the continuation side",
    );
}

#[test]
fn sprite_main_coarse_progress_cannot_rewind_a_finer_current_slot_checkpoint() {
    assert!(
        sprite_main_cpu_boundary_order(SpriteMainCpuBoundary::AfterSlot(1))
            < sprite_main_cpu_boundary_order(SpriteMainCpuBoundary::AfterZeldaFollowerGraphics {
                slot: 0,
                saved_follower_indicator: 0,
            },),
    );
    assert!(
        sprite_main_cpu_boundary_order(SpriteMainCpuBoundary::AfterZeldaFollowerGraphics {
            slot: 0,
            saved_follower_indicator: 0,
        }) < sprite_main_cpu_boundary_order(SpriteMainCpuBoundary::AfterSlot(0)),
    );
}

#[test]
fn live_module09_link_oam_boundary_resumes_to_the_atomic_c_endpoint() {
    fn configured_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.set_main_module(9);
        state.set_submodule(0);
        state.set_bg2_h_copy2(0x1010);
        state.set_bg2_v_copy2(0x2020);
        state.set_bg1_h_copy2(0x3030);
        state.set_bg1_v_copy2(0x4040);
        state.set_bg1_x_offset(3);
        state.set_bg1_y_offset(5);
        state.latch_nmi_update();
        state
    }

    // Atomic C authority: Module09 restores its four stack locals, calls
    // LinkOam_Main, HUD, and rain, then ZeldaRunGameLoop prepares sprites.
    let mut atomic = configured_state();
    atomic.complete_module09_overworld_after_submodule();
    atomic.nmi_prepare_sprites();
    atomic.clear_nmi_update_latch();

    let mut resumed = configured_state();
    resumed.restore_live_rom_timing_after_checkpoint();
    resumed.original_timing_owner = OriginalTimingOwnerState::Live;
    resumed.original_timing_semantic_receipts =
        Some(OriginalTimingHostReceipts::new(54_761, 0, vec![]));
    resumed.game_execution_scheduler.begin_host_frame();
    resumed.game_execution_scheduler.begin_main_loop_iteration();
    resumed.forward_original_timing_main_loop_interruption_to_native_owner(
        crate::MainLoopInterruption::LinkOam,
        OriginalTimingBoundary::NmiAccepted,
    );

    resumed.complete_module09_overworld_after_submodule();

    // LinkOam's interrupt occurs after the caller restored all four stack
    // locals. The NMI register mirrors still include the screen-shake offset.
    let scroll = &resumed.game_state.display.ppu_scroll_copy;
    assert_eq!(scroll.bg2_h_copy2(), 0x1010);
    assert_eq!(scroll.bg2_v_copy2(), 0x2020);
    assert_eq!(scroll.bg1_h_copy2(), 0x3030);
    assert_eq!(scroll.bg1_v_copy2(), 0x4040);
    assert_eq!(scroll.bg2_h_copy(), 0x1013);
    assert_eq!(scroll.bg2_v_copy(), 0x2025);

    let continuation = resumed
        .game_execution_scheduler
        .take_after_current_trailing_nmi()
        .expect("Module09 must retain its LinkOam caller across the interrupt");
    assert!(matches!(
        continuation,
        GameWorkContinuation::FinishModule09LinkOamCallerReturn { .. }
    ));
    assert!(resumed
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));

    resumed.complete_post_trailing_nmi_continuation(continuation, 0, false, false);

    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(resumed.game_state, atomic.game_state);
}

#[test]
fn cucco_animation_publication_then_lift_tail_matches_atomic_sprite_endpoint() {
    fn configured_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.set_main_module(9);
        state.set_submodule(0);
        state.game_state.frame.frame_counter = 0xc6;
        state.oam_reset_region_bases();
        state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
        state
            .oam_state_mut()
            .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
        state.sprite_system_mut().set_cur_object_index(6);
        let mut sprite = state.sprite_slot_view_mut(6);
        sprite.set_state(9);
        sprite.set_sprite_type(0x0b);
        sprite.set_c(1);
        sprite.set_x(0x0130);
        sprite.set_y(0x0920);
        sprite.set_x_velocity(1);
        sprite.set_y_velocity(0xff);
        sprite.set_subtype2(100);
        state
    }

    // Pinned ROM $86:a6e5 publishes four subtype increments and the graphics
    // generation before calling Sprite_ReturnIfLifted. The adapter may pause
    // only at that source boundary; completing its tail must reach the same
    // RAM and native-state endpoint as the unsplit translated call.
    let mut atomic = configured_state();
    atomic.sprite_0_b_cucco(6);

    let mut resumed = configured_state();
    resumed.sprite_main_cpu_boundary = Some(SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
        slot: 6,
        helper_ordinal: 0,
        continuation: None,
    });
    resumed.sprite_0_b_cucco(6);
    assert_eq!(resumed.sprite_slot_view(6).subtype2(), 104);
    assert_eq!(resumed.sprite_slot_view(6).graphics(), 0);
    assert_eq!(
        resumed.sprite_main_cpu_boundary,
        Some(SpriteMainCpuBoundary::AfterCuccoGraphicsPublication {
            slot: 6,
            helper_ordinal: 0,
            continuation: Some(CuccoSubtypeContinuation::ActiveC),
        }),
    );
    resumed.complete_cucco_after_graphics_publication(6, CuccoSubtypeContinuation::ActiveC);
    resumed.sprite_main_cpu_boundary = None;

    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(resumed.game_state, atomic.game_state);
}

#[test]
fn state10_cucco_subtype_checkpoint_resumes_to_the_atomic_sprite_endpoint() {
    fn configured_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.set_main_module(9);
        state.set_submodule(0);
        state.game_state.frame.frame_counter = 0xfc;
        state.oam_reset_region_bases();
        state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
        state
            .oam_state_mut()
            .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
        state.sprite_system_mut().set_cur_object_index(5);
        let mut sprite = state.sprite_slot_view_mut(5);
        sprite.set_state(10);
        sprite.set_sprite_type(0x0b);
        sprite.set_c(1);
        sprite.set_x(0x0435);
        sprite.set_y(0x094b);
        sprite.set_x_velocity(10);
        sprite.set_y_velocity(16);
        sprite.set_subtype2(0xfc);
        sprite.set_graphics(7);
        state
    }

    let mut atomic = configured_state();
    atomic.sprite_execute_single(5);

    let mut resumed = configured_state();
    resumed.sprite_main_cpu_boundary = Some(SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
        slot: 5,
        helper_ordinal: 0,
        completed: 3,
        total: 0,
        continuation: None,
    });
    resumed.sprite_execute_single(5);
    assert_eq!(resumed.sprite_slot_view(5).subtype2(), 0xff);
    assert_eq!(resumed.sprite_slot_view(5).graphics(), 7);
    assert_eq!(
        resumed.sprite_main_cpu_boundary,
        Some(SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
            slot: 5,
            helper_ordinal: 0,
            completed: 3,
            total: 3,
            continuation: Some(CuccoSubtypeContinuation::State10),
        }),
    );
    resumed.complete_cucco_after_subtype_increments(5, 3, 3, CuccoSubtypeContinuation::State10);
    resumed.sprite_main_cpu_boundary = None;

    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(resumed.game_state, atomic.game_state);
}

#[test]
fn fleeing_cucco_movement_checkpoint_resumes_to_the_atomic_sprite_endpoint() {
    fn configured_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.set_main_module(9);
        state.set_submodule(0);
        state.game_state.frame.frame_counter = 0;
        state.oam_reset_region_bases();
        state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
        state
            .oam_state_mut()
            .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
        state.sprite_system_mut().set_cur_object_index(5);
        let mut sprite = state.sprite_slot_view_mut(5);
        sprite.set_state(9);
        sprite.set_sprite_type(0x0b);
        sprite.set_ai_state(2);
        sprite.set_c(0);
        sprite.set_x(0x0435);
        sprite.set_y(0x094b);
        sprite.set_x_velocity(10);
        sprite.set_y_velocity(16);
        sprite.set_subtype2(0xfc);
        sprite.set_graphics(1);
        state
    }

    let mut atomic = configured_state();
    atomic.sprite_0_b_cucco(5);

    let mut resumed = configured_state();
    resumed.sprite_main_cpu_boundary = Some(SpriteMainCpuBoundary::AfterCuccoFleeMovement {
        slot: 5,
        helper_ordinal: 0,
    });
    resumed.sprite_0_b_cucco(5);
    assert_eq!(resumed.sprite_slot_view(5).subtype2(), 0xfc);
    assert_eq!(resumed.sprite_slot_view(5).graphics(), 1);
    assert_eq!(
        resumed.sprite_main_cpu_boundary,
        Some(SpriteMainCpuBoundary::AfterCuccoFleeMovement {
            slot: 5,
            helper_ordinal: 0,
        }),
    );

    resumed.complete_cucco_flee_after_movement(5, 0);
    resumed.sprite_main_cpu_boundary = None;

    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(resumed.game_state, atomic.game_state);
}

#[test]
fn fresh_module09_iteration_claims_the_authoritative_link_oam_timeline() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(9);
    // Use the source's asset-independent transition slot so this test proves
    // outer ZeldaRunGameLoop/Module09 ownership without needing a ROM asset
    // pack. The production boundary is common to every Module09 submodule.
    state.set_submodule(2);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
    state.capture_display_snapshot();
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        54_761,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                high: 0,
                low: 0,
                high_filtered: 0,
                low_filtered: 0,
            }),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                crate::MainLoopInterruption::LinkOam,
            ),
        ],
    ));

    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishModule09LinkOamCallerReturn { .. })
    ));
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(!state.original_timing_nmi_publication_pending);
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn run2319_idle_plan_claims_sprite_main_return_before_trailing_held() {
    let mut state = live_idle_dialogue_main_loop_state();
    state.set_main_module(7);
    state.set_submodule(0x0f);
    state.set_subsubmodule(1);
    state.set_spotlight_window_state(2);
    state.set_spotlight_window_radius(0x5b);
    state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::Open, NmiUpdateGate::LatchHeld];
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2_319,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                    high: 0,
                    low: 0,
                    high_filtered: 0,
                    low_filtered: 0,
                }),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::IterationStarted,
                ),
                OriginalTimingSemanticReceipt::SpriteMainReturned,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            ],
        ))
        .unwrap();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_host_dispatch_active = true;
    let plan = state
        .original_timing_uninterrupted_idle_main_loop_plan(
            crate::MainLoopProgress::IterationStarted,
        )
        .expect("run2319 must form one immutable idle-main plan");
    assert_eq!(plan.sprite_main_returned_claims, 1);
    assert_eq!(
        plan.suffix_action,
        OriginalTimingIdleMainLoopSuffixAction::ArmOrdinary,
    );
    assert_eq!(
        plan.in_module_phases_after_progress,
        [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld)],
    );
}

#[test]
fn atomic_sprite_main_consumes_one_linear_return_claim_at_slot_zero() {
    let mut state = ZeldaState::new();
    state.set_main_module(9);
    state.set_submodule(0);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        2_319,
        0,
        vec![OriginalTimingSemanticReceipt::SpriteMainReturned],
    ));
    state.original_timing_sprite_main_return_claims_remaining = Some(1);

    state.sprite_main();

    assert_eq!(
        state.original_timing_sprite_main_return_claims_remaining,
        Some(0),
    );
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn consecutive_sprite_main_calls_consume_two_ordered_return_claims() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0x0f);
    state.set_subsubmodule(1);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        2_320,
        0,
        vec![
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::SpriteMainReturned,
        ],
    ));
    state.original_timing_sprite_main_return_claims_remaining = Some(2);

    // Module07_0F_01 can run one spotlight-local Sprite_Main before Module 7's
    // shared caller runs a second. With no semantic boundary between them,
    // both returns belong to this one linear host-body scope.
    state.sprite_main();
    state.sprite_main();

    assert_eq!(
        state.original_timing_sprite_main_return_claims_remaining,
        Some(0),
    );
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn sprite_main_early_return_does_not_consume_the_slot_zero_claim() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        2_321,
        0,
        vec![OriginalTimingSemanticReceipt::SpriteMainReturned],
    ));
    state.original_timing_sprite_main_return_claims_remaining = Some(1);
    state.sprite_main_cpu_boundary = Some(SpriteMainCpuBoundary::BeforeFirstSlot);
    state.sprite_main_cpu_nmi_slices = 1;

    state.sprite_main();

    assert_eq!(
        state.original_timing_sprite_main_return_claims_remaining,
        Some(1),
    );
    assert_eq!(
        state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic(),
        [OriginalTimingSemanticReceipt::SpriteMainReturned],
    );
}

#[test]
fn sprite_main_claim_scope_is_runtime_only_and_blocks_host_installation() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.original_timing_sprite_main_return_claims_remaining = Some(0);
    assert!(!state.paired_resume_cpu_boundary_is_quiescent());
    assert_eq!(
        state.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            1,
            0,
            Vec::new(),
        )),
        Err(OriginalTimingReceiptInstallError::ActiveSpriteMainReturnClaim),
    );
    state.invalidate_original_timing_after_checkpoint();
    assert_eq!(
        state.original_timing_sprite_main_return_claims_remaining,
        None,
    );
}

#[test]
fn sprite_repel_dash_uses_facing_as_rebound_direction() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_running_state(1);
    set_link_test_byte(&mut state, LINK_DASH_CTR, 32);
    state.follower_link_state_mut().set_facing(4);

    state.sprite_repel_dash();

    assert_eq!(link_test_byte(&state, LINK_LAST_DIRECTION_MOVED_TOWARDS), 2);
    assert_eq!(
        state.game_state.player.follower_link.actual_x_velocity(),
        24
    );
    assert_eq!(state.game_state.player.follower_link.actual_y_velocity(), 0);
    assert_eq!(
        state.game_state.player.follower_link.incapacitated_timer(),
        24
    );
}

#[test]
fn edge_transition_recoil_guard_restores_previous_position() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    set_link_test_byte(&mut state, LINK_X_VEL, 1);
    state.follower_link_state_mut().set_incapacitated_timer(5);
    state.follower_link_state_mut().set_actual_x_velocity(12);
    state.follower_link_state_mut().set_actual_y_velocity(34);
    set_link_test_word(&mut state, LINK_X_COORD, 0x01e9);
    set_link_test_word(&mut state, LINK_Y_COORD, 0x0123);
    set_link_test_word(&mut state, LINK_X_COORD_PREV, 0x0088);
    set_link_test_word(&mut state, LINK_Y_COORD_PREV, 0x0099);

    state.Dungeon_TryScreenEdgeTransition();

    assert_eq!(state.game_state.player.follower_link.actual_x_velocity(), 0);
    assert_eq!(state.game_state.player.follower_link.actual_y_velocity(), 0);
    assert_eq!(link_test_byte(&state, LINK_RECOILMODE_TIMER), 3);
    assert_eq!(link_test_word(&state, LINK_X_COORD), 0x0088);
    assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x0099);
    assert_eq!(state.game_state.frame.submodule, 0);
}

#[test]
fn cancel_dash_clears_running_state_and_dash_ancilla() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_running_state(1);
    set_link_test_byte(&mut state, LINK_COUNTDOWN_FOR_DASH, 12);
    state.follower_link_state_mut().set_speed_setting(16);
    set_link_test_byte(&mut state, LINK_CANT_CHANGE_DIRECTION, 1);
    state.swim_acceleration_mut().set_mode(0, 0x1234);
    state.ancilla_slot_view_mut(0).set_ancilla_type(0x1e);
    state.ancilla_slot_view_mut(4).set_ancilla_type(0x1e);

    state.link_cancel_dash();

    assert_eq!(state.ancilla_slot_view(0).ancilla_type(), 0);
    assert_eq!(state.ancilla_slot_view(4).ancilla_type(), 0);
    assert_eq!(link_test_byte(&state, LINK_COUNTDOWN_FOR_DASH), 0);
    assert_eq!(state.game_state.player.follower_link.speed_setting(), 0);
    assert_eq!(state.game_state.player.follower_link.running_state(), 0);
    assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION), 0);
    assert_eq!(state.game_state.player.swim_acceleration.mode(0), 0);
}

#[test]
fn disabling_rom_timing_clears_the_transient_sprite_main_return_scope() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.original_timing_sprite_main_return_claims_remaining = Some(0);

    state.set_rom_startup_timing(false);

    assert_eq!(
        state.original_timing_sprite_main_return_claims_remaining,
        None,
    );
}

#[test]
fn sprite_reset_all_semantic_split_resumes_without_replaying_disable() {
    let mut state = ZeldaState::new();
    state.set_indoor_flag(1);
    for slot in 0..16 {
        let mut sprite = state.sprite_slot_view_mut(slot);
        sprite.set_state(9);
        sprite.set_sprite_type(0x40 + slot as u8);
    }
    state.sprite_system_mut().set_limit_instance(7);
    state.oam_state_mut().set_sprite_sorting_setting(5);

    let mut atomic = state.clone();
    atomic.sprite_reset_all();

    state.sprite_disable_all();
    assert!(
        (0..16).all(|slot| state.sprite_slot_view(slot).state() == 0),
        "the semantic checkpoint must include every completed C disable",
    );
    assert_eq!(
        state.ram[crate::game_state::constants::SORT_SPRITES_SETTING],
        5,
        "Sprite_ResetAll_noDisable must remain pending at the checkpoint",
    );

    state.sprite_reset_all_no_disable();

    assert_eq!(state.ram, atomic.ram);
    assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
}

#[test]
fn lanmola_draw_prefix_checkpoints_resume_to_the_atomic_draw() {
    for completed in 0..=5u8 {
        let mut state = ZeldaState::new();
        state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
        state.game_state.world.location.set_indoor_flag(1);
        state.sprite_slot_view_mut(2).set_sprite_type(0x54);
        state.sprite_slot_view_mut(2).set_state(9);
        state.sprite_slot_view_mut(2).set_ai_state(3);
        state.sprite_slot_view_mut(2).set_subtype2(8);
        state.sprite_slot_view_mut(2).set_x_velocity(0x10);
        state.sprite_slot_view_mut(2).set_y_velocity(0xf0);
        state.sprite_slot_view_mut(2).set_z_velocity(0x08);
        state.sprite_slot_view_mut(2).set_z(0x20);
        state.sprite_slot_view_mut(2).set_x_low(0x68);
        state.sprite_slot_view_mut(2).set_x_high(0x18);
        state.sprite_slot_view_mut(2).set_y_low(0x70);
        state.sprite_slot_view_mut(2).set_y_high(0x0d);
        let mut atomic = state.clone();
        atomic.sprite_54_lanmolas(2);
        state.begin_lanmola_draw_prefix_checkpoint(2, completed);
        if completed == 0 {
            assert_eq!(state.sprite_slot_view(2).graphics(), 0);
        }
        state.resume_lanmola_draw_prefix(2, completed);
        assert_eq!(state.ram, atomic.ram, "completed {completed}");
        assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
    }
}

#[test]
fn lanmola_draw_prefix_stores_keep_flat_trail_bytes_past_the_moldorm_model() {
    let mut state = ZeldaState::new();
    state.game_state.world.location.set_indoor_flag(1);
    let k = 2;
    state.sprite_slot_view_mut(k).set_sprite_type(0x54);
    state.sprite_slot_view_mut(k).set_state(9);
    state.sprite_slot_view_mut(k).set_subtype2(5);
    state.sprite_slot_view_mut(k).set_x_low(0x77);
    state.sprite_slot_view_mut(k).set_y_low(0x4c);
    let j = k * 64 + 5;
    assert!(j >= 128);
    state.lanmola_draw_prefix_stores(k, 3, 5);
    assert_eq!(
        state.ram[crate::game_state::constants::MOLDORM_HISTORY_X_LO + j],
        0x77
    );
    assert_eq!(
        state.ram[crate::game_state::constants::MOLDORM_HISTORY_Y_LO + j],
        0x4c
    );
}

#[test]
fn accepted_sprite_reset_nmi_resumes_without_executing_the_following_nmi() {
    // The pinned room-$72 receipt records the Y-high publication at V224:H1326
    // and the NMI which suspends Dungeon_ResetSprites at V225:H24. The next
    // host resumes that call stack; it must not sample the following host's
    // controller input or advance another NMI before returning to main wait.
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(2);
    state.set_dungeon_room_index(0x72);
    let progress = sprite::DungeonResetSpritesCpuProgress::Cache {
        slot: 1,
        field: CachedSpriteCacheField::YHigh,
    };
    state
        .game_execution_scheduler
        .schedule_after_current_trailing_nmi(
            GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::RoomLoadSpriteReset { progress },
            },
        );
    state.game_execution_scheduler.begin_host_frame();
    assert!(state
        .game_execution_scheduler
        .take_after_current_trailing_nmi()
        .is_some());
    let joypad_before = state.game_state.player.follower_link.joypad1h_last();

    assert!(state.continue_dungeon_room_load_after_sprite_reset(
        DungeonRoomLoadCpuSchedule::default(),
        0x8000,
        None,
        true,
    ));

    assert_eq!(
        state.game_state.player.follower_link.joypad1h_last(),
        joypad_before,
        "resuming an already-accepted NMI must not sample the next host input",
    );
    state.game_execution_scheduler.begin_host_frame();
    assert!(state
        .game_execution_scheduler
        .main_return_requires_leading_nmi());
}

#[test]
fn sprite_initializer_promotes_state_before_type_specific_prep() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    state
        .oam_state_mut()
        .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
    {
        let mut sprite = state.sprite_slot_view_mut(0);
        sprite.set_state(8);
        sprite.set_sprite_type(0x1d);
        sprite.set_graphics(6);
        sprite.set_ignore_projectile(0);
    }
    state.arm_sprite_main_cpu_continuation(
        SpriteMainCpuBoundary::InitializePrepPending { slot: 0 },
        1,
        SpriteMainCpuCaller::DungeonModule07,
    );
    state.sprite_main();
    assert_eq!(state.sprite_slot_view(0).state(), 9);
    assert_eq!(state.sprite_slot_view(0).graphics(), 0);
    assert_eq!(state.sprite_slot_view(0).ignore_projectile(), 0);
    state.complete_sprite_main_after_cpu_boundary(SpriteMainCpuBoundary::InitializePrepPending {
        slot: 0,
    });
    assert_eq!(state.sprite_slot_view(0).state(), 9);
    assert_eq!(state.sprite_slot_view(0).ignore_projectile(), 1);
}

#[test]
fn swamola_segment_draw_resumes_without_repeating_history_or_oam_steps() {
    for (velocity, y) in [(8, 0x80), (0xf8, 0x80), (8, 0x500), (0xf8, 0x500)] {
        for segment in 0..4u8 {
            let mut state = ZeldaState::new();
            state.set_main_module(9);
            state.set_submodule(0);
            {
                let mut sprite = state.sprite_slot_view_mut(4);
                sprite.set_state(9);
                sprite.set_sprite_type(0xcf);
                sprite.set_ai_state(2);
                sprite.set_x(0x80);
                sprite.set_y(y);
                sprite.set_subtype2(10);
                sprite.set_x_velocity(8);
                sprite.set_y_velocity(velocity);
                sprite.set_deflection_bits(0x80);
            }
            for index in 0..32 {
                state
                    .swamola_history_mut(4 * 32 + index)
                    .set_position(0x80 + index as u16, 0x80);
            }
            state.sprite_get16_bit_coords(4);
            let mut atomic = state.clone();
            atomic.swamola_draw(4);
            let mut head = state.clone();
            let history_before = head.game_state.effects.sprite_histories.clone();
            let oam_before = head.game_state.oam.clone();
            head.swamola_prepare_head(4);
            assert_eq!(head.game_state.effects.sprite_histories, history_before);
            assert_eq!(head.game_state.oam, oam_before);
            head.swamola_draw_after_head_checkpoint(4);
            assert_eq!(head.game_state, atomic.game_state);
            assert_eq!(head.ram, atomic.ram);
            let mut completed_head = state.clone();
            completed_head.swamola_prepare_head(4);
            completed_head.sprite_draw_single_large(4);
            assert_eq!(
                completed_head.game_state.effects.sprite_histories,
                history_before
            );
            completed_head.swamola_draw_after_completed_head(4);
            assert_eq!(completed_head.game_state, atomic.game_state);
            assert_eq!(completed_head.ram, atomic.ram);
            state.swamola_draw_until_segment(4, segment);
            assert_eq!(
                state.sprite_slot_view(4).graphics(),
                [0, 0, 1, 2][usize::from(segment)]
            );
            assert_eq!(
                state.game_state.sprites.workspace.shared_scratch_a(),
                segment
            );
            state.swamola_draw_after_segment_checkpoint(4, segment);
            assert_eq!(state.game_state, atomic.game_state);
            assert_eq!(state.ram, atomic.ram);
        }
    }
}

#[test]
fn mini_moldorm_ai_dispatch_retains_movement_without_replaying_it() {
    for direction in [0, 4, 8, 12] {
        let mut state = ZeldaState::new();
        state.set_main_module(7);
        state.set_submodule(0);
        state.follower_link_state_mut().set_position(0x10, 0x10);
        {
            let mut sprite = state.sprite_slot_view_mut(1);
            sprite.set_state(9);
            sprite.set_sprite_type(0x18);
            sprite.set_x(0x80);
            sprite.set_y(0x80);
            sprite.set_direction(direction);
            sprite.set_ai_state(0);
            sprite.set_delay_main(0);
        }
        let mut atomic = state.clone();
        atomic.sprite_18_mini_moldorm(1);
        state.sprite_main_cpu_boundary =
            Some(SpriteMainCpuBoundary::MiniMoldormAiPending { slot: 1 });
        state.sprite_18_mini_moldorm(1);
        assert_eq!(state.sprite_slot_view(1).subtype2(), 1);
        assert_eq!(state.sprite_slot_view(1).ai_state(), 0);
        state.sprite_main_cpu_boundary = None;
        state.mini_moldorm_ai_after_collision(1);
        assert_eq!(state.game_state, atomic.game_state);
        assert_eq!(state.ram, atomic.ram);
        assert_eq!(state.get_random_number(), atomic.get_random_number());
    }
}

#[test]
fn sprite_disable_resumes_after_each_exact_state_clear() {
    for slot in 0..16 {
        let mut state = ZeldaState::new();
        for k in 0..16 {
            state.sprite_slot_view_mut(k).set_state(9);
        }
        let mut atomic = state.clone();
        atomic.sprite_disable_all();
        state.apply_sprite_disable_actions_through(
            None,
            crate::DungeonSpriteDisableCpuProgress::SpriteStatesThrough { slot },
        );
        for k in 0..16 {
            assert_eq!(
                state.sprite_slot_view(k).state(),
                if k >= usize::from(slot) { 0 } else { 9 }
            );
        }
        state.complete_sprite_disable_after_states(slot);
        assert_eq!(state.game_state, atomic.game_state);
        assert_eq!(state.ram, atomic.ram);
    }
}

#[test]
fn super_bomb_purchase_pays_once_before_suspended_follower_graphics() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.follower_link_state_mut().set_position(0x80, 0x80);
    state.follower_link_state_mut().set_filtered_joypad_l(0x80);
    state.player_resources_mut().set_rupees_goal(300);
    {
        let mut sprite = state.sprite_slot_view_mut(14);
        sprite.set_state(9);
        sprite.set_sprite_type(0xb5);
        sprite.set_subtype2(2);
        sprite.set_x(0x80);
        sprite.set_y(0x80);
    }
    state.sprite_prep_load_properties(14);
    state.sprite_slot_view_mut(14).set_subtype2(2);
    let mut atomic = state.clone();
    atomic.sprite_bomb_shop_super_bomb(14);
    assert!(state.sprite_bomb_shop_super_bomb_before_follower_graphics(14));
    assert_eq!(
        state.game_state.inventory.player_resources.rupees_goal(),
        200
    );
    assert_eq!(state.game_state.sprites.follower_runtime.indicator(), 13);
    assert_eq!(state.sprite_slot_view(14).state(), 9);
    state.load_follower_graphics();
    state.sprite_bomb_shop_super_bomb_after_follower_graphics(14);
    assert_eq!(
        state.game_state.inventory.player_resources.rupees_goal(),
        200
    );
    assert_eq!(state.game_state, atomic.game_state);
    assert_eq!(state.ram, atomic.ram);
}

#[test]
fn returned_sprite_call_advances_only_lower_slots_to_timer_boundary() {
    assert!(sprite_main_in_flight_checkpoint_advances(
        SpriteMainCpuBoundary::HappinessPondRupeeGraphicsStarted(5),
        SpriteMainCpuBoundary::AfterTimersAndOam {
            slot: 3,
            state: None
        },
    ));
    assert!(!sprite_main_in_flight_checkpoint_advances(
        SpriteMainCpuBoundary::HappinessPondRupeeGraphicsStarted(5),
        SpriteMainCpuBoundary::AfterTimersAndOam {
            slot: 5,
            state: None
        },
    ));
    let mut state = ZeldaState::new();
    for slot in [3, 5] {
        let mut sprite = state.sprite_slot_view_mut(slot);
        sprite.set_state(9);
        sprite.set_delay_main(80);
    }
    let boundary = state.advance_sprite_main_after_slot_to_after_timers(5, 3);
    assert_eq!(
        boundary,
        SpriteMainCpuBoundary::AfterTimersAndOam {
            slot: 3,
            state: Some(9)
        }
    );
    assert_eq!(state.sprite_slot_view(5).delay_main(), 80);
    assert_eq!(state.sprite_slot_view(3).delay_main(), 79);
}

#[test]
fn antfairy_sprite_main_boundary_commits_every_source_reached_increment() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    state
        .oam_state_mut()
        .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
    for slot in 0..=1 {
        state.sprite_slot_view_mut(slot).set_state(9);
        state.sprite_slot_view_mut(slot).set_sprite_type(0x15);
        state
            .sprite_slot_view_mut(slot)
            .set_x(0x1000 + slot as u16 * 0x20);
        state.sprite_slot_view_mut(slot).set_y(0x1000);
    }
    state.arm_sprite_main_cpu_continuation(
        SpriteMainCpuBoundary::AfterAntfairySubtype2Increment {
            slot: 0,
            continuation: None,
        },
        1,
        SpriteMainCpuCaller::DungeonModule07,
    );

    state.sprite_main();

    assert_eq!(state.sprite_slot_view(1).subtype2(), 1);
    assert_eq!(state.sprite_slot_view(0).subtype2(), 1);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterAntfairySubtype2Increment {
                slot: 0,
                continuation: Some(AntfairyDrawContinuation::Antifairy),
            },
            caller: SpriteMainCpuCaller::DungeonModule07,
        })
    ));
}

#[test]
fn lanmola_sprite_main_boundary_commits_source_prefix_once_and_saves_draw_locals() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.clear_modal_pause_flag();
    state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    state
        .oam_state_mut()
        .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
    state.sprite_slot_view_mut(0).set_state(9);
    state.sprite_slot_view_mut(0).set_sprite_type(0x54);
    state.sprite_slot_view_mut(0).set_x(0x0040);
    state.sprite_slot_view_mut(0).set_y(0x0080);
    state.sprite_slot_view_mut(0).set_subtype2(3);
    state.arm_sprite_main_cpu_continuation(
        SpriteMainCpuBoundary::AfterLanmolaSubtype2Increment {
            slot: 0,
            continuation: None,
        },
        1,
        SpriteMainCpuCaller::DungeonModule07,
    );

    state.sprite_main();

    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterLanmolaSubtype2Increment {
                slot: 0,
                continuation: Some(LanmolaDrawContinuation { r2: 3, r5: 3 }),
            },
            caller: SpriteMainCpuCaller::DungeonModule07,
        })
    ));
    assert_eq!(state.sprite_slot_view(0).state(), 9);
    assert_eq!(state.game_state.frame.submodule, 0);
    assert_eq!(state.game_state.frame.modal_pause_flag, 0);
    assert_eq!(state.sprite_slot_view(0).subtype2(), 4);
}

#[test]
fn state_13_nmi_after_sprite_main_resumes_only_the_common_suffix() {
    for phase in [
        ModuleCpuPhase::InterruptedAfterSpriteMain,
        ModuleCpuPhase::InterruptedInLinkOam,
    ] {
        let mut state = ZeldaState::new();
        state.restore_live_rom_timing_after_checkpoint();
        state.set_main_module(7);
        state.set_submodule(2);
        state.set_subsubmodule(13);
        state.set_dungeon_room_index(0x72);
        state.set_countdown_word(26);
        state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
            phase,
            resumed_phase: None,
            submodule_nmi_slices: 0,
            subsubmodule: 13,
            palette_countdown: 26,
            sprite_main_boundary: None,
            cached_sprite_interruption: None,
        });
        state.game_execution_scheduler.begin_host_frame();
        state.game_execution_scheduler.begin_main_loop_iteration();

        state.Dungeon_InterRoomTrans_State13();
        assert_eq!(state.game_state.frame.subsubmodule, 13);
        assert!(state.dungeon_post_sprite_main_return_pending);
        assert!(state.game_execution_scheduler.is_idle());

        state.complete_module07_dungeon_after_submodule();
        assert!(!state.dungeon_post_sprite_main_return_pending);
        assert!(state.active_dungeon_sprite_main_return.is_some());
        assert_eq!(
            state.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn)
        );
    }
}

#[test]
fn scheduled_caller_then_fresh_sprite_preparation_uses_the_completed_leading_oam_dma() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(14);
    state.set_dungeon_room_index(0x42);
    state.set_frame_counter(0x44);
    state.project_native_game_state_to_ram();
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::CompleteBeforeNmi,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 14,
        palette_countdown: 0,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });
    assert!(state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::State13CallerReturn,
    ));
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                high: 0,
                low: 0,
                high_filtered: 0,
                low_filtered: 0,
            }),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                crate::MainLoopInterruption::SpritePreparation,
            ),
        ],
    ));

    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    // The old suspended caller returns first, then the source begins one fresh
    // ZeldaRunGameLoop iteration and reaches NMI_PrepareSprites. Module 7 has
    // therefore authored the next software OAM shadow, but the active scanout
    // still consumes the leading handler's already-completed OAM/Link DMA.
    assert_eq!(state.game_state.frame.frame_counter, 0x45);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishNmiPrepareSpritesCallerReturn {
            caller: NmiPrepareSpritesCpuCaller::DungeonModule07,
        }),
    );
    let captured = state
        .display_snapshot
        .as_ref()
        .expect("the completed source host must capture its outgoing scanout");
    assert_eq!(
        captured.oam_scanout_source,
        OamScanoutSource::ComposeLiveAfterNmi
    );
    assert_eq!(
        captured.link_obj_scanout_generation,
        GraphicsDmaGeneration::HostBoundaryBeforeMain,
    );
    assert_eq!(
        captured.link_obj_source_generation,
        GraphicsDmaGeneration::HostBoundaryBeforeMain,
    );
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn state_12_nmi_inside_sprite_preparation_resumes_only_that_caller() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(12);
    state.set_overworld_map_state(5);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInNmiPrepareSprites,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 13,
        palette_countdown: 0,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();

    state.module07_dungeon();

    assert_eq!(state.game_state.frame.subsubmodule, 13);
    assert!(!state.dungeon_nmi_prepare_sprites_return_pending);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishNmiPrepareSpritesCallerReturn {
            caller: NmiPrepareSpritesCpuCaller::DungeonModule07,
        }),
    );
}

#[test]
fn module09_host_return_progress_stops_before_a_newly_spawned_lower_sprite_slot() {
    let mut state = ZeldaState::new();
    let slot = 2usize;
    state.rom_startup_timing = true;
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(9);
    state.set_submodule(0);
    {
        let mut sprite = state.sprite_slot_view_mut(slot);
        sprite.set_state(9);
        sprite.set_sprite_type(0x0b);
        sprite.set_c(1);
        sprite.set_x(0x00d2);
        sprite.set_y(0x0966);
        sprite.set_x_velocity(0x20);
        sprite.set_y_velocity(0xf7);
        sprite.set_subtype2(0);
    }
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            crate::SpriteMainProgress::AfterSlot(3),
        )],
    ));
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();

    state.complete_module09_sprite_and_hud_suffix();

    let sprite = state.sprite_slot_view(slot);
    assert_eq!(sprite.x(), 0x00d2);
    assert_eq!(sprite.y(), 0x0966);
    assert_eq!(sprite.x_subpixel(), 0);
    assert_eq!(sprite.y_subpixel(), 0);
    assert_eq!(sprite.subtype2(), 0);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterSlot(3),
            caller: SpriteMainCpuCaller::Module09 {
                boundary: OriginalTimingBoundary::HostReturn,
            },
        })
    ));
}

#[test]
fn file_select_return_rejects_an_extended_oam_suffix_before_install_or_caller_mutation() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(1);
    state.set_submodule(2);
    state.set_subsubmodule(249);

    state.module_erase_file_1();
    state.pending_main_loop_common_suffix = Some(
        MainLoopCommonSuffixContinuation::ResumeSpritePreparationExtendedOamPackingAndClearNmiLatch {
            next_group_start: 4,
        },
    );
    state.latch_nmi_update();
    let wrong_suffix = state.pending_main_loop_common_suffix;

    let result = state.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
        1032,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    ));

    assert_eq!(
        result,
        Err(OriginalTimingReceiptInstallError::InvalidMainLoopIterationReturn),
    );
    assert_eq!(state.game_state.frame.submodule, 2);
    assert_eq!(state.game_state.display.bg_vram_load_mode, 0);
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(state
        .pre_main_caller_continuation_is(PreMainCallerContinuation::FileSelectCheckerboardUpload));
    assert_eq!(state.pending_main_loop_common_suffix, wrong_suffix);
    assert!(state.original_timing_semantic_receipts.is_none());
    assert_eq!(state.last_consumed_original_timing_host_call(), None);
}

#[test]
fn faded_filter_uses_the_interior_sprite_boundary_as_the_single_nmi_owner() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(2);
    state.set_countdown_word(30);
    state.set_mosaic_target_level(31);
    state.dungeon_torch_mut().set_lights_out_request(1);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInSpriteMain,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 3,
        palette_countdown: 0,
        sprite_main_boundary: Some(SpriteMainCpuBoundary::AfterSlot(0)),
        cached_sprite_interruption: None,
    });
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();

    state.Module07_02_FadedFilter();

    assert_eq!(state.game_state.frame.subsubmodule, 3);
    assert_eq!(state.game_execution_scheduler.pre_main_nmi_resume(), None);
    assert_eq!(
        state.sprite_main_cpu_boundary,
        Some(SpriteMainCpuBoundary::AfterSlot(0)),
    );

    state.complete_module07_dungeon_after_submodule();
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterSlot(0),
            caller: SpriteMainCpuCaller::DungeonModule07,
        }),
    );
}

#[test]
fn sprite_main_interruption_preserves_the_c_zelda_initializer_prefix() {
    assert_eq!(ZELDA_FOLLOWER_GRAPHICS_RETURN_ADDRESS, 0x05_ebf5);
    assert_eq!(
        sprite_main_cpu_interruption_boundary(
            Some(0),
            Some(1),
            Some(0),
            ZELDA_FOLLOWER_GRAPHICS_RETURN_ADDRESS,
        ),
        Some(SpriteMainCpuBoundary::BeforeZeldaFollowerGraphics(0)),
    );
}

#[test]
fn sprite_main_interruption_does_not_promote_an_unreturned_slot() {
    assert_eq!(
        sprite_main_cpu_interruption_boundary(Some(0), Some(1), Some(0), 0x00_e7a6),
        Some(SpriteMainCpuBoundary::AfterSlot(1)),
    );
    assert_eq!(
        sprite_main_cpu_interruption_boundary(
            Some(0),
            Some(1),
            Some(2),
            ZELDA_FOLLOWER_GRAPHICS_RETURN_ADDRESS,
        ),
        Some(SpriteMainCpuBoundary::AfterSlot(1)),
    );
}

#[test]
fn faded_filter_preserves_nmi_prepare_sprites_caller_phase() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(14);
    state.set_countdown_word(30);
    state.set_mosaic_target_level(31);
    state.dungeon_torch_mut().set_lights_out_request(1);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInNmiPrepareSprites,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 15,
        palette_countdown: 0,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();

    state.module07_dungeon();

    assert_eq!(state.game_state.frame.subsubmodule, 15);
    assert!(!state.dungeon_nmi_prepare_sprites_return_pending);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishNmiPrepareSpritesCallerReturn {
            caller: NmiPrepareSpritesCpuCaller::DungeonModule07,
        }),
    );
    assert!(state
        .game_execution_scheduler
        .work_suspends_translated_call_stack());
}

#[test]
fn landing_nmi_inside_link_oam_resumes_only_the_common_suffix() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(14);
    state.set_countdown_word(0);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInLinkOam,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 15,
        palette_countdown: 0,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();

    state.Module07_02_FadedFilter();
    assert_eq!(state.game_state.frame.subsubmodule, 15);
    assert!(state.dungeon_post_sprite_main_return_pending);

    state.complete_module07_dungeon_after_submodule();
    assert!(!state.dungeon_post_sprite_main_return_pending);
    assert!(state.active_dungeon_sprite_main_return.is_some());
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn)
    );

    state.game_execution_scheduler.begin_host_frame();
    let continuation = match state.game_execution_scheduler.advance_work_one_nmi_slice() {
        Some(GameWorkStep::Complete(continuation)) => continuation,
        step => panic!("Link OAM return must resume after one interrupting NMI: {step:?}"),
    };
    state.capture_display_snapshot();
    let captured_palette = state.display_snapshot.as_ref().unwrap().ppu.cgram.clone();
    state.complete_post_trailing_nmi_continuation(continuation, 0, false, false);

    assert!(state.active_dungeon_sprite_main_return.is_none());
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.frame.subsubmodule, 15);
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|display| display.effective_presented_dma.is_none()
            && display.ppu.cgram == captured_palette));
}

#[test]
fn landing_nmi_inside_sprite_main_resumes_at_the_cpu_slot_boundary() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(14);
    state.set_countdown_word(0);
    for slot in 0..=4 {
        state.sprite_slot_view_mut(slot).set_state(8);
        state.sprite_slot_view_mut(slot).set_sprite_type(0x6d);
    }
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInSpriteMain,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 15,
        palette_countdown: 0,
        sprite_main_boundary: Some(SpriteMainCpuBoundary::AfterSlot(3)),
        cached_sprite_interruption: None,
    });
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();

    state.Module07_02_FadedFilter();
    assert_eq!(state.game_state.frame.subsubmodule, 15);
    assert_eq!(
        state.sprite_main_cpu_boundary,
        Some(SpriteMainCpuBoundary::AfterSlot(3)),
    );

    state.complete_module07_dungeon_after_submodule();
    assert_eq!(state.sprite_slot_view(4).state(), 9);
    assert_eq!(state.sprite_slot_view(3).state(), 9);
    for slot in 0..3 {
        assert_eq!(state.sprite_slot_view(slot).state(), 8);
    }
    assert!(state.active_dungeon_sprite_main_return.is_some());
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterSlot(3),
            caller: SpriteMainCpuCaller::DungeonModule07,
        }),
    );
}

#[test]
fn cached_sprite_copy_boundary_supersedes_the_coarse_sprite_slot_boundary() {
    let mut state = ZeldaState::new();
    let advance = DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInSpriteMain,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 15,
        palette_countdown: 0,
        sprite_main_boundary: Some(SpriteMainCpuBoundary::AfterSlot(0)),
        cached_sprite_interruption: Some(CachedSpriteCpuInterruption::Loading {
            slot: 2,
            copied_fields: 12,
        }),
    };

    assert!(state.arm_dungeon_sprite_main_cpu_continuation(advance));
    assert_eq!(state.sprite_main_cpu_boundary, None);
    assert_eq!(
        state.dungeon_cached_sprite_cpu_interruption_pending,
        Some(CachedSpriteCpuInterruption::Loading {
            slot: 2,
            copied_fields: 12,
        })
    );
}

#[test]
fn completed_cached_sprite_caller_carries_the_following_source_nmi_at_main_wait() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_indoor_flag(1);
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(5);
    {
        let mut slot = state.sprite_slot_view_mut(2);
        slot.set_state(8);
        slot.set_sprite_type(0x6d);
        slot.set_x(0x04ab);
        slot.set_y(0x0543);
    }
    state.dungeon_cache_trans_sprites();
    state.sprite_slot_view_mut(2).set_sprite_type(0x6f);
    state.dungeon_cached_sprite_cpu_interruption_pending =
        Some(CachedSpriteCpuInterruption::Restoring {
            slot: 2,
            live_fields: 9,
        });
    state.dungeon_cached_sprite_cpu_interruption_boundary =
        Some(OriginalTimingBoundary::HostReturn);
    state.active_dungeon_sprite_main_return = Some(DungeonSpriteMainReturn {
        link_oam: None,
        bg2_x: 0,
        bg2_y: 0,
        bg1_x: 0,
        bg1_y: 0,
    });
    state.dungeon_quadrant_cpu_continuation_active = true;
    state.execute_cached_sprites();
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonCachedSpriteMain { .. }),
    ));

    // Pinned source order: the NMI which suspended UncacheAndExecuteSprite
    // publishes first, the resumed Sprite_Main/Module07 suffix reaches the
    // main wait, and the following NMI is accepted before the host returns.
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        31_288,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
        ],
    ));
    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert!(!state.dungeon_quadrant_cpu_continuation_active);
    assert!(state.original_timing_nmi_publication_pending);
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn scroll_return_precedes_a_suspended_extended_oam_suffix() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(0xa680);
    state.set_main_module(14);
    state.set_submodule(2);
    state.messaging_state_mut().set_module(1);
    state.messaging_state_mut().set_text_render_state(3);
    state.messaging_state_mut().set_dialogue_scroll_speed(4);
    state
        .messaging_text_mut()
        .load_decoded_dialogue(&[TEXT_COMMAND_START_US + 12]);
    state.capture_display_snapshot();
    state.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );
    assert!(!state.render_text_scroll_pixels(2));
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    state.latch_nmi_update();
    state
        .install_original_timing_host_receipts(
            OriginalTimingHostReceipts::new(
                179852,
                0,
                vec![
                    OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                    OriginalTimingSemanticReceipt::MainLoopInterrupted(
                        crate::MainLoopInterruption::SpritePreparationExtendedOamPacking {
                            next_group_start: 20,
                        },
                    ),
                    OriginalTimingSemanticReceipt::MainLoopProgress(
                        crate::MainLoopProgress::CallStackContinued,
                    ),
                ],
            )
            .with_dialogue_scroll_progress(vec![
                crate::DialogueScrollProgressReceipt {
                    entered: false,
                    completed_pixel_passes: 3,
                    returned: true,
                },
            ]),
        )
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.dialogue_scroll_cpu_is_idle());
    assert_eq!(state.pending_main_loop_common_suffix, Some(MainLoopCommonSuffixContinuation::ResumeSpritePreparationExtendedOamPackingAndClearNmiLatch { next_group_start: 20 }));
    assert_eq!(
        state
            .game_state
            .messaging
            .dialogue_source_offset
            .bank_offset_low_nibble(),
        5
    );
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_semantic_receipts.is_none());
    // The carried Held handler cannot publish the staged text. The source
    // finishes packing and clears the latch before accepting the Open NMI.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            179853,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            ],
        ))
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::Open)
    );
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert_eq!(
        state
            .game_state
            .messaging
            .dialogue_source_offset
            .bank_offset_low_nibble(),
        5
    );
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn host_boundary_link_oam_composition_keeps_unrelated_live_sprites() {
    let mut live_oam = vec![0x1111; 0x110];
    let host_boundary_oam = vec![0x2222; 0x110];

    compose_host_boundary_link_oam(&mut live_oam, Some(&host_boundary_oam));

    for entry in HOST_BOUNDARY_LINK_OAM_ENTRIES {
        assert_eq!(&live_oam[entry * 2..entry * 2 + 2], &[0x2222, 0x2222]);
        let high_word = 256 + entry / 8;
        let high_shift = (entry % 8) * 2;
        let high_mask = 0b11 << high_shift;
        assert_eq!(
            live_oam[high_word] & high_mask,
            host_boundary_oam[high_word] & high_mask
        );
    }
    for entry in [101, 104, 106, 108, 109, 114] {
        assert_eq!(&live_oam[entry * 2..entry * 2 + 2], &[0x1111, 0x1111]);
    }
}

#[test]
fn continued_iteration_can_advance_its_existing_suffix_to_an_oam_pack_cursor() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(0);
    state.set_submodule(6);
    state.set_subsubmodule(41);
    state.set_frame_counter(106);
    state.reset_bg_tile_animation_countdown(5);
    state.latch_nmi_update();
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    for i in 0..32 {
        state.oam_state_mut().set_packed_extended_oam_byte(i, 0x5a);
    }
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            849,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    crate::MainLoopInterruption::SpritePreparationExtendedOamPacking {
                        next_group_start: 4,
                    },
                ),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
            ],
        ))
        .unwrap();

    state.run_frame_internal(0, crate::RUN_POLY);

    assert_eq!(state.game_state.frame.frame_counter, 106);
    assert_eq!(state.game_state.frame.subsubmodule, 41);
    assert_eq!(state.game_state.display.bg_tile_animation_countdown, 5);
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(
            MainLoopCommonSuffixContinuation::ResumeSpritePreparationExtendedOamPackingAndClearNmiLatch {
                next_group_start: 4,
            },
        ),
    );
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(849));
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn every_extended_oam_cursor_resumes_to_the_atomic_sprite_preparation_endpoint() {
    let mut base = ZeldaState::new();
    let bytewise_extended_oam = crate::game_state::constants::BYTEWISE_EXTENDED_OAM;
    for i in 0..0x80 {
        base.ram[bytewise_extended_oam + i] = (i as u8).wrapping_add(1) & 3;
    }
    base.sync_native_game_state_from_ram();
    base.reset_bg_tile_animation_countdown(5);
    for i in 0..32 {
        base.oam_state_mut().set_packed_extended_oam_byte(i, 0x5a);
    }
    let expected_packed = (0..32)
        .map(|i| base.game_state.oam.packed_extended_oam_byte(i))
        .collect::<Vec<_>>();
    let packed_extended_oam = crate::game_state::constants::EXTENDED_OAM;

    let mut atomic = base.clone();
    atomic.nmi_prepare_sprites();
    assert_eq!(atomic.game_state.display.bg_tile_animation_countdown, 4);
    let atomic_checkpoint = bincode::serialize(&atomic).unwrap();

    for next_group_start in [28u8, 24, 20, 16, 12, 8, 4, 0] {
        let mut split = base.clone();
        split.nmi_prepare_sprites_through_extended_oam_packing(next_group_start);

        for i in 0..32 {
            let group_start = i & !3;
            let expected = if group_start > usize::from(next_group_start) {
                expected_packed[i]
            } else {
                0x5a
            };
            assert_eq!(
                split.ram[packed_extended_oam + i],
                expected,
                "cursor {next_group_start} published the wrong prefix at byte {i}",
            );
        }
        assert_eq!(split.game_state.display.bg_tile_animation_countdown, 5);

        split.nmi_prepare_sprites_resume_after_extended_oam_packing(next_group_start);

        assert_eq!(split.game_state.display.bg_tile_animation_countdown, 4);
        assert_eq!(
            bincode::serialize(&split).unwrap(),
            atomic_checkpoint,
            "cursor {next_group_start} did not reach the atomic endpoint",
        );
    }
}

#[test]
fn game_loop_clears_oam_y_slots_and_keeps_nmi_update_latched() {
    let mut state = ZeldaState::new();
    state.latch_nmi_update();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.frame_counter, 1);
    assert!(state.game_state.display.nmi_update_is_latched());
    for i in 4..128 {
        assert_eq!(state.ram[OAM_BUF + i * 4 + 1], 0xf0);
    }
}

#[test]
fn graphics_half_slot_transforms_uncompressed_sprite_pack() {
    let mut state = ZeldaState::new();
    let mut pack = vec![0; 0x300 + 24 * 32];
    for i in 0..24 * 32 {
        pack[0x300 + i] = i as u8;
    }
    let mut data = vec![0; 8 * 2];
    data.extend_from_slice(&pack);
    data.extend_from_slice(&8u16.to_le_bytes());
    let mut ranges = vec![(0, 0); 65];
    ranges[64] = (0, data.len());
    state.assets = Some(AssetPack::from_data_ranges(data, ranges));

    state.set_chr_halfslot_request(20);
    state.graphics_load_chr_half_slot();

    assert_eq!(state.game_state.display.nmi_load_target_page(), 0x46);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 11);
    assert_eq!(&state.ram[0x11000..0x11004], &[0, 1, 2, 3]);
    assert_eq!(&state.ram[0x11010..0x11014], &[16, 17, 17, 19]);
    assert_eq!(&state.ram[0x11020..0x11024], &[24, 25, 26, 27]);
}

#[test]
fn intro_submodule_one_continues_memory_clear_and_logo_oam() {
    let mut state = ZeldaState::new();
    state.run_frame_internal(0, crate::RUN_MAIN);
    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.submodule, 1);
    assert_eq!(state.game_state.frame.subsubmodule, 2);
    assert_eq!(
        &state.ram[OAM_BUF..OAM_BUF + 16],
        &[
            0x60, 0x68, 0x69, 0x32, 0x70, 0x68, 0x6b, 0x32, 0x80, 0x68, 0x6d, 0x32, 0x88, 0x68,
            0x6e, 0x32
        ]
    );
    assert_eq!(state.game_state.dungeon.scratch_word.primary_word(), 0x17fe);
    assert_eq!(
        state.game_state.dungeon.scratch_word.secondary_word(),
        0x13fe
    );
}

#[test]
fn sprite_initialize_reset_prefix_commits_only_source_completed_stores() {
    let mut split = ZeldaState::new();
    let slot = 1;
    {
        let mut sprite = split.sprite_slot_view_mut(slot);
        sprite.set_state(8);
        sprite.set_sprite_type(2);
        sprite.set_x_velocity(0xe0);
        sprite.set_y_velocity(0x38);
        sprite.set_y_subpixel(0x80);
        sprite.set_ai_state(2);
        sprite.set_wall_collision(0x77);
        sprite.set_z(0x55);
        sprite.set_health(0x66);
    }
    let mut atomic = split.clone();

    split.sprite_prep_reset_properties_prefix(slot, 20);
    {
        let sprite = split.sprite_slot_view(slot);
        assert_eq!(sprite.x_velocity(), 0);
        assert_eq!(sprite.y_velocity(), 0);
        assert_eq!(sprite.y_subpixel(), 0);
        assert_eq!(sprite.ai_state(), 0);
        assert_eq!(sprite.wall_collision(), 0);
        assert_eq!(sprite.z(), 0x55, "store 21 remains pending");
        assert_eq!(sprite.health(), 0x66, "store 22 remains pending");
        assert_eq!(sprite.state(), 8, "the property loader has not returned");
    }

    split.sprite_prep_reset_properties_from(slot, 20);
    split.sprite_module_initialize_properties_after_reset(slot);
    split.sprite_module_initialize_after_properties(slot);
    atomic.sprite_module_initialize(slot);

    let split_sprite = split.sprite_slot_view(slot);
    let atomic_sprite = atomic.sprite_slot_view(slot);
    assert_eq!(split_sprite.state(), atomic_sprite.state());
    assert_eq!(split_sprite.x_velocity(), atomic_sprite.x_velocity());
    assert_eq!(split_sprite.y_velocity(), atomic_sprite.y_velocity());
    assert_eq!(split_sprite.y_subpixel(), atomic_sprite.y_subpixel());
    assert_eq!(split_sprite.ai_state(), atomic_sprite.ai_state());
    assert_eq!(
        split_sprite.wall_collision(),
        atomic_sprite.wall_collision()
    );
    assert_eq!(split_sprite.z(), atomic_sprite.z());
    assert_eq!(split_sprite.health(), atomic_sprite.health());
    assert_eq!(split_sprite.flags2(), atomic_sprite.flags2());
    assert_eq!(split_sprite.flags3(), atomic_sprite.flags3());
    assert_eq!(split_sprite.flags4(), atomic_sprite.flags4());
    assert_eq!(split_sprite.flags5(), atomic_sprite.flags5());
}

#[test]
fn ancilla_allocation_rotation_follows_the_rom_past_the_five_slots() {
    // On the hardware `ancilla_alloc_rotate` ($03C4) is also arr25[2]; the
    // fairy revival leaves 9 in it. `Ancilla_AddAncilla` then walks
    // `LDX $03C4 : DEX : LDA $0C4A,X` from index 8 down through raw RAM until
    // it finds a sparkle/arrow slot (types $3c/$13/$0a). Route frame 150466.
    let mut state = ZeldaState::new();
    for slot in 0..5usize {
        state.ancilla_slot_view_mut(slot).set_ancilla_type(0x20); // full, not reusable
    }
    state.set_ancilla_alloc_rotate(9);
    state.ram[ANCILLA_TYPE + 8] = 0; // garbage RAM past the arrays: no match
    state.ancilla_slot_view_mut(3).set_ancilla_type(0x3c);

    let slot = state.ancilla_add_simple(0x05, 4);

    assert_eq!(
        slot,
        Some(3),
        "walks 8,7,6,5,4 through raw RAM, then reuses the sparkle in slot 3"
    );
    assert_eq!(state.ancilla_alloc_rotate(), 3);
    assert_eq!(state.ram[ANCILLA_ALLOC_ROTATE], 3);
    assert_eq!(
        state.ancilla_slot_view(2).work_byte_25(),
        3,
        "the rotation byte is arr25[2] on the hardware"
    );

    // Control: the ordinary rotation (start 4) still reuses the highest-index
    // reusable slot below the start, exactly as before the relocation.
    let mut ordinary = ZeldaState::new();
    for slot in 0..5usize {
        ordinary.ancilla_slot_view_mut(slot).set_ancilla_type(0x20);
    }
    ordinary.ancilla_slot_view_mut(1).set_ancilla_type(0x13);
    ordinary.set_ancilla_alloc_rotate(4);
    assert_eq!(ordinary.ancilla_add_simple(0x05, 4), Some(1));
    assert_eq!(ordinary.ancilla_alloc_rotate(), 1);
}
