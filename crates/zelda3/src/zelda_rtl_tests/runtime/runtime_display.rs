//! ZeldaState runtime tests — display.
//! Split mechanically from the hub file; bodies unchanged.

use super::*;

#[test]
fn palette_filter_prefix_and_remainder_equal_one_uninterrupted_source_call() {
    fn configure(state: &mut ZeldaState) {
        state.set_subsubmodule(3);
        state.set_countdown_word(0);
        state.set_mosaic_target_level(0);
        state.set_darkening_or_lightening_screen_word(2);
        for index in 0..256 {
            state.set_main_color(index, 0x4210);
            state.set_aux_color(index, 0x4210);
        }
    }

    let mut split = ZeldaState::new();
    configure(&mut split);
    split.apply_palette_filter_bounce_prefix(213);
    assert_ne!(
        split.game_state.display.palette_buffer.main_color(212),
        0x4210,
    );
    assert_eq!(
        split.game_state.display.palette_buffer.main_color(213),
        0x4210,
    );
    split.complete_apply_palette_filter_bounce_from(213);

    let mut whole = ZeldaState::new();
    configure(&mut whole);
    whole.ApplyPaletteFilter_bounce();

    for index in 0..256 {
        assert_eq!(
            split.game_state.display.palette_buffer.main_color(index),
            whole.game_state.display.palette_buffer.main_color(index),
            "palette color {index} changed across continuation",
        );
    }
    assert_eq!(split.game_state.frame.subsubmodule, 4);
    assert_eq!(
        split
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen_word(),
        whole
            .game_state
            .display
            .palette_filter
            .darkening_or_lightening_screen_word(),
    );
}

#[test]
fn authoritative_nmi_joypad_publication_overrides_newer_host_input() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        7,
        0x0002,
        vec![OriginalTimingSemanticReceipt::JoypadPublication(
            JoypadPublication {
                high: 0,
                low: 0,
                high_filtered: 0,
                low_filtered: 0,
            },
        )],
    ));

    // Libretro has already supplied Right for this host call, but the NMI
    // completed before V228 auto-joy refreshed $4218. The atomic translated
    // handler initially sees the new host value; the timing authority then
    // publishes the exact Zelda bytes observed at the source handler return.
    state.interrupt_nmi_for_active_scanout(0x0002, None, false);
    assert_eq!(state.game_state.player.follower_link.joypad1h_last(), 0x40);
    state.apply_original_timing_joypad_publication();

    assert_eq!(state.game_state.player.follower_link.joypad1h_last(), 0);
    assert_eq!(state.game_state.player.follower_link.joypad1l_last(), 0);
    assert_eq!(state.game_state.player.follower_link.filtered_joypad_h(), 0);
    assert_eq!(state.game_state.player.follower_link.filtered_joypad_l(), 0);
    assert_eq!(state.game_state.player.follower_link.joypad1h_last2(), 0);
    assert_eq!(state.game_state.player.follower_link.joypad1l_last2(), 0);
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .unwrap()
        .semantic
        .is_empty());
}

#[test]
fn early_return_finishes_the_matching_authoritative_joypad_publication() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0x0002,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                    high: 0,
                    low: 0,
                    high_filtered: 0,
                    low_filtered: 0,
                }),
            ],
        ))
        .unwrap();
    let owns_dispatch = state.begin_original_timing_host_dispatch(0x0002);

    // The typed owner must drain the complete accepted-handler lifecycle.
    // Host close may carry a terminal acceptance, but it cannot synthesize a
    // handler or publish joypad bytes on behalf of an early-return branch.
    let phases = state.take_original_timing_nmi_phases();
    assert_eq!(
        phases,
        [
            OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open),
            OriginalTimingNmiPhase::HandlerCompleted,
        ],
    );
    let classification = classify_original_timing_nmi_phases_with_ownership(false, &phases);
    assert!(state
        .complete_original_timing_nmi_handler_for_active_scanout(
            classification.handler_completion,
            0x0002,
            None,
        )
        .completed());

    state.finish_original_timing_host_dispatch(owns_dispatch);

    let link = &state.game_state.player.follower_link;
    assert_eq!(link.joypad1h_last(), 0);
    assert_eq!(link.joypad1l_last(), 0);
    assert_eq!(link.filtered_joypad_h(), 0);
    assert_eq!(link.filtered_joypad_l(), 0);
    assert_eq!(link.joypad1h_last2(), 0);
    assert_eq!(link.joypad1l_last2(), 0);
    assert!(!state.original_timing_nmi_publication_pending);
    assert!(!state.original_timing_host_dispatch_active);
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn scratch_word_high_does_not_alias_nmi_subroutine_index() {
    let mut state = ZeldaState::new();
    state.scratch_word_mut().set_word(0x0200);
    state.set_pending_nmi_subroutine(11);

    assert_eq!(state.scratch_word_mut().decrement_high(), 1);

    assert_eq!(state.game_state.dungeon.scratch_word.word(), 0x0100);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 11);
}

#[test]
fn new_state_exposes_first_nmi_link_dma_source() {
    let state = ZeldaState::new();

    assert_eq!(&state.ram[0..3], &[0x00, 0x80, 0x00]);
}

#[test]
fn rom_palette_words_fall_back_to_generated_assets_without_rom() {
    let mut state = ZeldaState::new();
    let ranges: [(u32, usize, u16); 14] = [
        (PALETTE_DUNGEON_BG_MAIN_SNES_ADDR, 79, 0x1111),
        (PALETTE_MAIN_SPRITE_SNES_ADDR, 80, 0x2222),
        (PALETTE_ARMOR_AND_GLOVES_SNES_ADDR, 81, 0x3333),
        (PALETTE_SWORD_SNES_ADDR, 82, 0x4444),
        (PALETTE_SHIELD_SNES_ADDR, 83, 0x5555),
        (PALETTE_SPRITE_AUX3_SNES_ADDR, 84, 0x6666),
        (PALETTE_MISC_SPRITE_INDOORS_SNES_ADDR, 85, 0x7777),
        (PALETTE_SPRITE_AUX1_SNES_ADDR, 86, 0x8888),
        (PALETTE_OVERWORLD_BG_MAIN_SNES_ADDR, 87, 0x9999),
        (PALETTE_OVERWORLD_BG_AUX12_SNES_ADDR, 88, 0xaaaa),
        (PALETTE_OVERWORLD_BG_AUX3_SNES_ADDR, 89, 0xbbbb),
        (PALETTE_PALACE_MAP_BG_SNES_ADDR, 90, 0xcccc),
        (PALETTE_PALACE_MAP_SPRITE_SNES_ADDR, 91, 0xdddd),
        (HUD_PALETTE_SNES_ADDR, 92, 0xeeee),
    ];
    let mut data = Vec::new();
    let mut asset_ranges = vec![(0, 0); 93];
    for &(_, asset, value) in &ranges {
        let start = data.len();
        data.extend_from_slice(&value.to_le_bytes());
        data.extend_from_slice(&(value ^ 0xffff).to_le_bytes());
        asset_ranges[asset] = (start, data.len());
    }
    state.assets = Some(AssetPack::from_data_ranges(data, asset_ranges));

    for &(base, _, value) in &ranges {
        assert_eq!(state.rom_or_asset_word_snes(base), Some(value));
        assert_eq!(state.rom_or_asset_word_snes(base + 2), Some(value ^ 0xffff));
    }
}

#[test]
fn original_timing_resume_sidecar_rejects_pending_nmi_without_its_display_owner() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_last_oracle_host_call = Some(84);
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
    state.capture_display_snapshot();

    assert_eq!(
        state.capture_original_timing_resume_checkpoint(),
        Err(OriginalTimingResumeCheckpointError::ActiveTranslatedContinuation),
    );

    let mut restored = ZeldaState::new();
    restored.set_rom_startup_timing(true);
    let before = restored.clone();
    let checkpoint = crate::OriginalTimingResumeCheckpoint {
        schema: crate::OriginalTimingResumeCheckpoint::SCHEMA,
        last_consumed_host_call: Some(84),
        nmi_publication_pending: true,
        pending_nmi_update_gate: Some(NmiUpdateGate::Open),
        dungeon_exit_spotlight_entry_return_pending: false,
        pre_dungeon_return_pending: None,
        item_receipt_live_link_dma_host: None,
    };
    assert_eq!(
        restored.restore_original_timing_resume_checkpoint(checkpoint),
        Err(OriginalTimingResumeCheckpointError::ActiveTranslatedContinuation),
    );
    assert_eq!(
        restored.original_timing_nmi_publication_pending,
        before.original_timing_nmi_publication_pending,
    );
    assert_eq!(
        restored.original_timing_pending_nmi_update_gate,
        before.original_timing_pending_nmi_update_gate,
    );
    assert_eq!(
        restored.original_timing_last_oracle_host_call,
        before.original_timing_last_oracle_host_call,
    );
}

#[test]
fn terminal_cached_restore_keeps_its_backup_until_pre_nmi_stores_publish() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(6);
    state.set_animated_tile_data_source_address(1);
    state.ram[0x0de2] = 1;
    state.sync_native_game_state_from_ram();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.latch_nmi_update();
    state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::Open];
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonCachedSpriteMain {
            boundary: CachedSpriteCpuInterruption::Restoring {
                slot: 2,
                live_fields: 12,
            },
            live_slot_backup: [0; 24],
            dungeon: DungeonSpriteMainReturn {
                bg2_x: 0,
                bg2_y: 0,
                bg1_x: 0,
                bg1_y: 0,
                link_oam: None,
            },
        },
        1,
    );
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        31_288,
        0,
        vec![
            cached_sprite_progress_receipt(
                crate::CachedSpriteExecutionProgress::Restoring {
                    slot: 2,
                    live_fields: 11,
                },
                OriginalTimingBoundary::NmiAccepted,
            ),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    ));
    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);
    assert_eq!(state.ram[0x0de2], 0);
    assert!(state.game_execution_scheduler.is_idle());
    assert!(state.pending_main_loop_common_suffix.is_none());
}

#[test]
fn idle_live_host_continuing_an_nmi_call_stack_does_not_start_the_next_game_iteration() {
    let mut state = ZeldaState::new();
    state.initialized = true;
    state.set_main_module(9);
    state.set_submodule(0);
    state.set_animated_tile_data_source_address(1);
    state.set_frame_counter(0xc5);
    {
        let mut guard = state.sprite_slot_view_mut(5);
        guard.set_state(5);
        guard.set_sprite_type(0x41);
        guard.set_x(0x00ff);
        guard.set_y(0x08df);
        guard.set_subtype2(0x24);
    }
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::LatchHeld];
    state.latch_nmi_update();
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.capture_display_snapshot();
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        56_423,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
        ],
    ));

    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.frame_counter, 0xc5);
    let guard = state.sprite_slot_view(5);
    assert_eq!(guard.x(), 0x00ff);
    assert_eq!(guard.y(), 0x08df);
    assert_eq!(guard.subtype2(), 0x24);
    assert!(state.original_timing_nmi_publication_pending);
    assert!(state.game_execution_scheduler.is_idle());
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn fleeing_cucco_third_increment_checkpoint_preserves_graphics_until_publication() {
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
    resumed.sprite_main_cpu_boundary = Some(SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
        slot: 5,
        helper_ordinal: 0,
        completed: 3,
        total: 0,
        continuation: None,
    });
    resumed.sprite_0_b_cucco(5);

    assert_eq!(resumed.sprite_slot_view(5).subtype2(), 0xff);
    assert_eq!(resumed.sprite_slot_view(5).graphics(), 1);
    assert_eq!(
        resumed.sprite_main_cpu_boundary,
        Some(SpriteMainCpuBoundary::AfterCuccoSubtypeIncrements {
            slot: 5,
            helper_ordinal: 0,
            completed: 3,
            total: 5,
            continuation: Some(CuccoSubtypeContinuation::Flee),
        }),
    );

    resumed.complete_cucco_after_subtype_increments(5, 3, 5, CuccoSubtypeContinuation::Flee);
    resumed.sprite_main_cpu_boundary = None;

    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(resumed.game_state, atomic.game_state);
}

#[test]
fn live_scheduled_caller_can_return_after_handler_completion_without_another_nmi() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(15);
    state.set_subsubmodule(0);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::MainLoopInterrupted(
            crate::MainLoopInterruption::LinkOam,
        )],
    ));
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();
    state.module07_dungeon();

    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn),
    );

    // This source host accepts and completes the interrupting NMI, resumes the
    // saved caller, and returns without accepting a second NMI. The translated
    // scheduler must not synthesize its ordinary trailing interrupt.
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        1,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                crate::MainLoopInterruption::LinkOam,
            ),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
        ],
    ));
    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert!(!state.original_timing_nmi_publication_pending);
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn live_uninterrupted_scheduled_caller_preserves_the_complete_nmi_lifecycle() {
    let mut scheduled = ZeldaState::new();
    scheduled.restore_live_rom_timing_after_checkpoint();
    scheduled.set_main_module(7);
    scheduled.set_submodule(15);
    scheduled.set_subsubmodule(0);
    scheduled.original_timing_owner = OriginalTimingOwnerState::Live;
    scheduled.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::MainLoopInterrupted(
            crate::MainLoopInterruption::LinkOam,
        )],
    ));
    scheduled.game_execution_scheduler.begin_host_frame();
    scheduled
        .game_execution_scheduler
        .begin_main_loop_iteration();
    scheduled.module07_dungeon();

    assert_eq!(
        scheduled.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn),
    );

    // The pinned schema-11 cold receipt has both source-valid uninterrupted
    // orders: host 82 accepts and publishes one NMI before the saved caller
    // continues, while host 84 also accepts the following NMI at host return.
    // Neither handler was pending when the source host began.
    let mut completes_one_nmi = scheduled.clone();
    completes_one_nmi.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        82,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
        ],
    ));
    completes_one_nmi.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);
    assert!(!completes_one_nmi.original_timing_nmi_publication_pending);
    assert!(completes_one_nmi
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));

    scheduled.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        84,
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
    scheduled.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);
    assert!(scheduled.original_timing_nmi_publication_pending);
    assert!(scheduled
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn live_waiting_caller_accepts_nmi_independently_of_the_software_update_latch() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.latch_nmi_update();
    state.original_timing_nmi_publication_pending = true;
    state.capture_display_snapshot();
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDialogueInitializationPrefix {
            caller_nmi_crossings: 1,
        },
        2,
    );

    // Pinned cold host 2330 completes the prior NMI, accepts the next one,
    // and continues Text_Initialize while Zelda's `$12` software update latch
    // remains set. Snes9x accepts from NMIPending/NMITIMEN; ZeldaRunGameLoop
    // clears `$12` only after Module_MainRouting returns.
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        2_330,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
        ],
    ));
    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDialogueInitializationPrefix {
            caller_nmi_crossings: 1,
        }),
    );
    assert_eq!(
        state
            .game_execution_scheduler
            .scheduled_work_slices_remaining(),
        Some(1),
    );
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn live_interrupted_caller_can_publish_a_previously_accepted_nmi() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.latch_nmi_update();
    state.original_timing_nmi_publication_pending = true;
    state.capture_display_snapshot();
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDialogueInitializationPrefix {
            caller_nmi_crossings: 1,
        },
        2,
    );

    // Pinned cold host 4783 completes the NMI accepted by the preceding host,
    // then reaches LinkOam in the still-suspended spotlight caller. No second
    // NMI is accepted in this host; interruption placement does not change the
    // accepted/publication state machine.
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        4_783,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                crate::MainLoopInterruption::LinkOam,
            ),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
        ],
    ));
    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDialogueInitializationPrefix {
            caller_nmi_crossings: 1,
        }),
    );
    assert_eq!(
        state
            .game_execution_scheduler
            .scheduled_work_slices_remaining(),
        Some(1),
    );
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn run3827_dless_terminal_carries_only_the_post_suffix_open_nmi() {
    let mut state = live_idle_terminal_suspended_vwf_state();
    state.original_timing_semantic_receipts = None;
    state.original_timing_expected_nmi_update_gates.clear();
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            3_827,
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
    let epoch_before_terminal = state.display_snapshot_epoch;

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.display_snapshot_epoch, epoch_before_terminal + 1);
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::Open),
    );
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(!state.dialogue_fast_forward_hold_active);
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(state.original_timing_semantic_receipts.is_none());

    // The next host completes the carried Open handler on the receptive
    // run3827 capture, then accepts one new Held NMI after Sprite_Main. The
    // single epoch increment proves the carry-in completion did not recapture.
    let epoch_before_completion = state.display_snapshot_epoch;
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            3_828,
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
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            ],
        ))
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.display_snapshot_epoch, epoch_before_completion + 1);
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert_eq!(
        state.original_timing_sprite_main_return_claims_remaining,
        None,
    );
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn completed_idle_iteration_owns_only_its_post_suffix_open_nmi_lifecycle() {
    let mut state = live_idle_dialogue_main_loop_state();
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            3_817,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                    high: 0x11,
                    low: 0x22,
                    high_filtered: 0x33,
                    low_filtered: 0x44,
                }),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::IterationStarted,
                ),
                OriginalTimingSemanticReceipt::SpriteMainReturned,
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                    high: 0x55,
                    low: 0x66,
                    high_filtered: 0x77,
                    low_filtered: 0x88,
                }),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            ],
        ))
        .unwrap();
    let epoch_before = state.display_snapshot_epoch;

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.pending_main_loop_common_suffix.is_none());
    assert_eq!(state.display_snapshot_epoch, epoch_before + 3);
    assert_eq!(
        state.game_state.player.follower_link.joypad1h_last(),
        0x55,
        "the post-suffix Open handler must own the last Joypad publication",
    );
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn link_zap_mosaic_bounces_between_zero_and_c0() {
    let mut state = ZeldaState::new();
    state.set_mosaic_level(0xb0);

    state.LinkZap_HandleMosaic();

    assert_eq!(state.game_state.display.mosaic_level, 0xc0);
    assert_eq!(state.game_state.display.mosaic_direction, 1);
    assert_eq!(state.game_state.display.mosaic_copy, 0x63);
    assert_eq!(state.game_state.display.bg_mode, 9);

    state.set_mosaic_level(0x10);
    state.LinkZap_HandleMosaic();
    assert_eq!(state.game_state.display.mosaic_level, 0);
    assert_eq!(state.game_state.display.mosaic_direction, 0);
    assert_eq!(state.game_state.display.mosaic_copy, 3);
}

#[test]
fn load_actual_gear_palettes_applies_enhanced_glove_color() {
    let mut state = ZeldaState::new();
    state.enhanced_features_mut().set_bits(0x1000);
    state.inventory_items_mut().set_inventory_item(20, 2);

    state.load_actual_gear_palettes();

    assert_eq!(
        read_le_u16(&state.ram, AUX_PALETTE_BUFFER + 0xfd * 2),
        0x0376
    );
    assert_eq!(
        read_le_u16(&state.ram, MAIN_PALETTE_BUFFER + 0xfd * 2),
        0x0376
    );
    assert_eq!(state.ram[FLAG_UPDATE_CGRAM_IN_NMI], 2);
}

#[test]
fn ordinary_trailing_open_nmi_publishes_a_completion_staged_after_capture() {
    let mut state = ZeldaState::new();
    state.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );
    state.finish_dialogue_scroll_remaining_pixels();
    state.finish_dialogue_scroll_return();
    for index in 0..crate::PresentedDialogueText::WORD_COUNT {
        state.set_messaging_render_buffer_word(index, 0x5200 | index as u16);
    }
    state.capture_display_snapshot();
    let staged = state.dialogue_text_scanout_from_render_buffer();
    state.stage_dialogue_scroll_completion_after_return(staged);
    state.clear_nmi_update_latch();
    state.set_pending_nmi_subroutine(2);
    state.set_core_update_disable_flag(2);

    state.interrupt_nmi(0, None, false);

    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletedScroll,
    );
    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[0x7c00]),
        0x5200,
    );
}

#[test]
fn continued_host_nmi_precedes_common_suffix_and_next_nmi_publishes_pending_work() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    // `zelda_run_frame` increments the translated debug counter before it
    // dispatches source host call 81, so the historical frame-82 policy used
    // this value. The ordered receipt, not that counter, owns the NMI order.
    state.frame_ctr_dbg = 82;
    state.set_sound_effect_1(0x34);
    // Intro/file-load work can mark this host as partial before the source
    // call suspends. The suspension itself already skips the common suffix;
    // this one-host marker must not leak into the eventual resumed return.
    state.rom_load_partial_nmi_this_frame = true;
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            81,
            0x0002,
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
            ],
        ))
        .unwrap();

    state.run_frame_internal(0x0002, crate::RUN_MAIN);

    // Interrupt_NMI_AudioParts_Locked is unconditional in the C handler. A
    // fabricated second NMI would overwrite port 2 with zero after the first
    // call consumed this command. Keeping 0x34 proves one handler invocation.
    assert_eq!(state.zelda_debug_apu_write_ports()[2], 0x34);
    let frame = state.game_state.frame;
    assert_eq!(
        (frame.main_module, frame.submodule, frame.subsubmodule),
        (0, 1, 1),
    );
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 11);
    assert_eq!(state.game_state.display.core_update_disable_flag, 0x80);
    assert!(state.game_state.display.nmi_update_is_latched());
    assert_eq!(state.game_state.system_signals.sound_effect_2(), 10);

    // The source NMI publishes the preceding auto-joy sample even though this
    // host already supplied Right. The authoritative publication must win once.
    let link = &state.game_state.player.follower_link;
    assert_eq!(link.joypad1h_last(), 0);
    assert_eq!(link.joypad1l_last(), 0);
    assert_eq!(link.filtered_joypad_h(), 0);
    assert_eq!(link.filtered_joypad_l(), 0);

    assert_eq!(state.last_consumed_original_timing_host_call(), Some(81));
    assert!(!state.rom_load_partial_nmi_this_frame);
    assert!(!state.original_timing_nmi_publication_pending);
    assert!(state.original_timing_semantic_receipts.is_none());
    assert!(!state.original_timing_host_dispatch_active);

    // Host 81 returned while Module_MainRouting was still active. Its shared
    // ZeldaRunGameLoop suffix therefore remains a typed continuation instead
    // of being inferred from a host/frame number.
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert!(!state.main_loop_sprite_preparation_completed);
    assert!(state.resident_oam_dma.is_some());
    let frame_counter_after_host_81 = state.game_state.frame.frame_counter;

    // The next source host resumes that existing call stack. Its NMI occurs
    // while Module_MainRouting is still active, so it must observe the old
    // set latch and leave pending subroutine 11 untouched. Only the terminal
    // ReturnedToWait fact proves the C caller subsequently ran
    // NMI_PrepareSprites and cleared the latch.
    state.rom_load_partial_nmi_this_frame = true;
    state.set_sound_effect_1(0x56);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            82,
            0x4080,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopIterationReturnedToWait,
            ],
        ))
        .unwrap();

    state.run_frame_internal(0x4080, crate::RUN_MAIN);

    // Each host's accepted NMI runs the unconditional C audio prefix once.
    // A duplicate host-82 audio call would overwrite these consumed commands
    // with zero; replaying the main prefix/module would also re-arm SFX2.
    assert_eq!(state.zelda_debug_apu_write_ports()[2], 0x56);
    assert_eq!(state.zelda_debug_apu_write_ports()[3], 10);
    assert_eq!(state.game_state.system_signals.sound_effect_2(), 0);

    assert_eq!(
        state.game_state.frame.frame_counter, frame_counter_after_host_81,
        "CallStackContinued must not replay ZeldaRunGameLoop's frame-counter prefix",
    );
    let frame = state.game_state.frame;
    assert_eq!(
        (frame.main_module, frame.submodule, frame.subsubmodule),
        (0, 1, 1),
    );
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(!state.rom_load_partial_nmi_this_frame);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 11);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.resident_oam_dma.is_some());
    assert!(state.main_loop_sprite_preparation_completed);

    // The held-latch NMI skipped NMI_ReadJoypads. The authoritative adapter
    // therefore publishes no joypad receipt for host 82, and the preceding
    // sample remains visible.
    let link = &state.game_state.player.follower_link;
    assert_eq!(link.joypad1h_last(), 0);
    assert_eq!(link.joypad1l_last(), 0);
    assert_eq!(link.filtered_joypad_h(), 0);
    assert_eq!(link.filtered_joypad_l(), 0);
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(82));
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(state.original_timing_semantic_receipts.is_none());
    assert!(!state.original_timing_host_dispatch_active);

    // Host 83 accepts its own leading NMI after the common suffix. It publishes
    // pending subroutine 11 and the source joypad sample once, then begins a
    // fresh iteration which remains inside Module_MainRouting at host return.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            83,
            0x4080,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                    high: 0xa5,
                    low: 0x5a,
                    high_filtered: 0x81,
                    low_filtered: 0x42,
                }),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::IterationStarted,
                ),
            ],
        ))
        .unwrap();

    state.run_frame_internal(0x4080, crate::RUN_MAIN);

    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0);
    assert!(state.game_state.display.nmi_update_is_latched());
    assert_eq!(
        state.game_state.frame.frame_counter,
        frame_counter_after_host_81.wrapping_add(1),
    );
    let link = &state.game_state.player.follower_link;
    assert_eq!(link.joypad1h_last(), 0xa5);
    assert_eq!(link.joypad1l_last(), 0x5a);
    assert_eq!(link.filtered_joypad_h(), 0x81);
    assert_eq!(link.filtered_joypad_l(), 0x42);
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(83));
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert!(state.original_timing_semantic_receipts.is_none());
    assert!(!state.original_timing_host_dispatch_active);
}

#[test]
fn idle_continued_call_runs_nmi_then_common_suffix_without_replaying_main() {
    let mut state = ZeldaState::new();
    state.initialized = true;
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.intro_memory_darken_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(0);
    state.set_submodule(6);
    state.set_subsubmodule(41);
    state.set_frame_counter(106);
    state.latch_nmi_update();
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.set_sound_effect_1(0x56);

    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            891,
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
        ))
        .unwrap();

    state.run_frame_internal(0, crate::RUN_POLY);

    assert_eq!(state.zelda_debug_apu_write_ports()[2], 0x56);
    assert_eq!(state.game_state.frame.frame_counter, 106);
    assert_eq!(state.game_state.frame.submodule, 6);
    assert_eq!(state.game_state.frame.subsubmodule, 41);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.main_loop_sprite_preparation_completed);
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::Open),
    );
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(891));
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn post_trailing_nmi_work_cannot_be_resumed_at_the_next_host_entry() {
    let mut scheduler = GameExecutionScheduler::default();
    scheduler.schedule_post_trailing_nmi(GameWorkContinuation::FinishDungeonMapRoomDrawing);

    assert!(scheduler.work_is_pending());
    assert!(scheduler.work_suspends_translated_call_stack());
    assert_eq!(
        scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonMapRoomDrawing)
    );
    assert_eq!(scheduler.advance_work_one_nmi_slice(), None);
    assert_eq!(
        scheduler.take_post_trailing_nmi(),
        Some(GameWorkContinuation::FinishDungeonMapRoomDrawing)
    );
    assert!(scheduler.is_idle());
}

#[test]
fn palette_direction_toggle_defers_countdown_clear_and_substage_publication() {
    for (direction, countdown, target, exposed_countdown) in [(0, 30, 31, 31), (2, 0, 0, 0)] {
        let mut atomic = ZeldaState::new();
        atomic.set_main_module(7);
        atomic.set_submodule(7);
        atomic.set_subsubmodule(15);
        atomic.set_countdown_word(countdown);
        atomic.set_mosaic_target_level(target);
        atomic.set_darkening_or_lightening_screen_word(direction);
        let mut staged = atomic.clone();
        let cgram_requests = staged.ram[0x15];
        atomic.ApplyPaletteFilter_bounce();
        staged.apply_palette_filter_bounce_through_direction_toggle();
        assert_eq!(
            staged
                .game_state
                .display
                .palette_filter
                .darkening_or_lightening_screen_word(),
            direction ^ 2
        );
        assert_eq!(
            staged.game_state.display.palette_filter.countdown_word(),
            exposed_countdown
        );
        assert_eq!(staged.game_state.frame.subsubmodule, 15);
        assert_eq!(staged.ram[0x15], cgram_requests);
        staged.complete_apply_palette_filter_bounce_after_direction_toggle();
        assert_eq!(staged.game_state.frame.subsubmodule, 16);
        assert_eq!(staged.ram, atomic.ram);
    }
}

#[test]
fn module09_scroll_prefix_retains_the_pending_vertical_pair() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(9);
    state.set_submodule(5);
    state.set_bg2_x(100);
    state.set_bg2_y(200);
    state.set_bg1_x(300);
    state.set_bg1_y(400);
    state.set_bg1_x_offset(3);
    state.set_bg1_y_offset(5);
    let mut atomic = state.clone();
    let saved = atomic.begin_module09_sprite_main();
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::Module09FinalScrollPairPending],
    ));
    state.complete_module09_sprite_and_hud_suffix();
    assert_eq!(state.game_state.display.ppu_scroll_copy.bg1_v_copy2(), 400);
    assert_eq!(
        state.active_module09_sprite_main_return.unwrap().scroll,
        saved
    );
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::Module09FinalScrollPairPending,
            ..
        })
    ));
    state.complete_module09_final_scroll_pair();
    assert_eq!(state.game_state.display.ppu_scroll_copy.bg1_v_copy2(), 405);
    assert_eq!(state.game_state, atomic.game_state);
    assert_eq!(state.ram, atomic.ram);
}

#[test]
fn state_13_suspends_common_module_suffix_when_rom_run_reaches_nmi_after_module() {
    for phase in [
        ModuleCpuPhase::InterruptedInNmiPrepareSprites,
        ModuleCpuPhase::InterruptedAfterModule,
    ] {
        let mut state = ZeldaState::new();
        state.restore_live_rom_timing_after_checkpoint();
        state.set_main_module(7);
        state.set_submodule(2);
        state.set_subsubmodule(13);
        state.set_dungeon_room_index(0x41);
        state.set_countdown_word(24);
        state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
            phase,
            resumed_phase: None,
            submodule_nmi_slices: 0,
            subsubmodule: 13,
            palette_countdown: 24,
            sprite_main_boundary: None,
            cached_sprite_interruption: None,
        });

        state.Dungeon_InterRoomTrans_State13();

        assert_eq!(state.game_state.frame.subsubmodule, 13);
        assert_eq!(
            state.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::State13CallerReturn,
            })
        );
        assert!(state
            .game_execution_scheduler
            .work_suspends_translated_call_stack());
    }

    let state = ZeldaState::new();
    let completion =
        GameWorkStep::Complete(GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::State13CallerReturn,
        });
    assert!(scheduled_work_completion_clears_nmi_latch_after_interrupt(
        completion
    ));
    assert_eq!(
        GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::State13CallerReturn,
        }
        .completion_publication(BgScrollRegisterScanout::capture(&state.ppu)),
        GameWorkCompletionPublication {
            bg_scroll: Some(DisplayBgScrollGeneration::ComposeLiveAfterNmi),
            obj: Some(ObjScanoutGenerations {
                oam: OamScanoutSource::RetainCapturedBeforeNmi,
                link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            }),
        }
    );
}

#[test]
fn state_13_finishes_common_module_suffix_when_rom_run_completes_before_nmi() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(13);
    state.set_dungeon_room_index(0x41);
    state.set_countdown_word(22);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::CompleteBeforeNmi,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 13,
        palette_countdown: 22,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });

    state.Dungeon_InterRoomTrans_State13();

    assert_eq!(state.game_state.frame.subsubmodule, 13);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(
        state.dungeon_state_13_atomic_caller_return_publication_host_frame,
        Some(state.frame_ctr_dbg)
    );
}

#[test]
fn end_of_story_rom_work_resumes_after_memory_and_palette_nmis() {
    let mut work = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishAttractEndOfStory,
        ATTRACT_END_OF_STORY_NMI_SLICES,
    );

    for _ in 0..ATTRACT_END_OF_STORY_NMI_SLICES - 1 {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(GameWorkContinuation::FinishAttractEndOfStory)
    );
}

#[test]
fn module10_goal_return_publishes_the_rom_vblank_generation() {
    let plan = OverworldSpotlightCpuPlan {
        interrupted_pc: 0x00_f3b7,
        interrupted_return_address: 0,
        iterations_before_nmi: 0,
        // This is captured when the ROM reaches the main-loop wait after
        // IrisSpotlight_ResetTable. Later scanouts collected by the timing
        // plan must not overwrite the already-proven module-exit boundary.
        nmis_before_module_exit: Some(1),
        active_window_words: [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
        following_window_words: [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
        next_entry_earliest: None,
        next_entry_latest: None,
    };

    assert!(plan.exits_module_before_next_nmi());
    let iteration = SpotlightIteration::opening_from_rom_cpu_plan(Some(plan));
    assert_eq!(
        iteration.completion_publication(),
        DisplaySnapshotPublication::AdvanceStaged,
    );
    assert!(iteration.completed_hdma_table_owns_active_scanout());
    assert_eq!(
        SpotlightIteration::opening_from_rom_cpu_plan(None).completion_publication(),
        DisplaySnapshotPublication::AdvanceStaged,
    );
}

#[test]
fn animated_bg_phase_change_retains_the_completed_scanout_generation() {
    let gameplay = rom_graphics_dma_plan(7, 0);
    let brightness = rom_graphics_dma_plan(7, 10);
    let spiral_stairs = rom_graphics_dma_plan(7, 0x0e);

    assert_eq!(
        gameplay.oam_operands,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(gameplay.oam_scanout, OamScanoutSource::ComposeLiveAfterNmi);
    assert_eq!(
        gameplay.link_obj_scanout,
        GraphicsDmaGeneration::LiveAfterMain
    );
    assert_eq!(
        gameplay.link_obj_operands,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(
        gameplay.animated_bg_operands,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    let landing = crate::game_state::FrameState {
        main_module: 7,
        submodule: 1,
        ..Default::default()
    };
    assert_eq!(
        animated_bg_operands_for_dungeon_landing(
            landing,
            0x72,
            8,
            GraphicsDmaGeneration::LiveAfterMain,
        ),
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(
        animated_bg_operands_for_dungeon_landing(
            landing,
            0x71,
            8,
            GraphicsDmaGeneration::LiveAfterMain,
        ),
        GraphicsDmaGeneration::LiveAfterMain
    );
    assert_eq!(
        animated_bg_operands_for_dungeon_landing(
            landing,
            0x72,
            4,
            GraphicsDmaGeneration::LiveAfterMain,
        ),
        GraphicsDmaGeneration::LiveAfterMain
    );
    assert_eq!(
        animated_bg_scanout_across_main(gameplay, gameplay),
        AnimatedBgScanoutGeneration::LiveAfterNmi
    );
    assert_eq!(
        animated_bg_scanout_across_main(brightness, brightness),
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
    );
    assert_eq!(
        animated_bg_scanout_across_main(gameplay, brightness),
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
    );
    assert_eq!(
        animated_bg_scanout_across_main(brightness, gameplay),
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
    );
    assert_eq!(
        spiral_stairs.animated_bg_scanout,
        AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
    );
    let spiral_frame = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0x0e,
        ..Default::default()
    };
    assert_eq!(
        rom_spiral_stairs_suspended_animated_bg_source_address(spiral_frame, true, 1, 0xaa80,),
        Some(0xae80)
    );
    assert_eq!(
        rom_spiral_stairs_suspended_animated_bg_source_address(spiral_frame, true, 2, 0xaa80,),
        None
    );
    assert_eq!(
        rom_spiral_stairs_suspended_animated_bg_source_address(spiral_frame, false, 1, 0xaa80,),
        None
    );
    assert_eq!(
        rom_spiral_stairs_suspended_animated_bg_source_address(spiral_frame, true, 1, 0xae80,),
        Some(0xa680)
    );
}

#[test]
fn nmi_copy_packets_publish_only_after_the_dma_boundary() {
    let mut state = ZeldaState::new();
    let packet_base = crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF;
    state.ppu.vram[0x2000] = 0x1111;
    state.ppu.vram[0x2001] = 0x2222;
    state.ppu.vram[0x2020] = 0x3333;
    write_le_u16(&mut state.ram, packet_base, 0x2000);
    state.ram[packet_base + 2] = 0x80;
    state.ram[packet_base + 3] = 2;
    state.ram[packet_base + 4..packet_base + 6].copy_from_slice(&[0xaa, 0xaa]);
    write_le_u16(&mut state.ram, packet_base + 6, 0x2020);
    state.ram[packet_base + 8] = 0x81;
    state.ram[packet_base + 9] = 2;
    state.ram[packet_base + 10..packet_base + 12].copy_from_slice(&[0xbb, 0xbb]);
    write_le_u16(&mut state.ram, packet_base + 12, 0xffff);
    state.ram[NMI_COPY_PACKETS_FLAG] = 1;
    state.sync_native_game_state_from_ram();
    state.capture_display_snapshot();

    state.ppu.vram[0x2000] = 0xaaaa;
    state.ppu.vram[0x2001] = 0x9999;
    state.ppu.vram[0x2020] = 0xbbbb;

    let presented = state.with_display_snapshot(|display| {
        [
            display.ppu.vram[0x2000],
            display.ppu.vram[0x2001],
            display.ppu.vram[0x2020],
        ]
    });

    assert_eq!(presented, [0x1111, 0x9999, 0x3333]);
    assert_eq!(state.ppu.vram[0x2000], 0xaaaa);
    assert_eq!(state.ppu.vram[0x2020], 0xbbbb);
}

#[test]
fn measured_nmi_prepare_interruption_records_captured_dma_for_active_scanout_once() {
    const CAPTURED_SOURCE: usize = 0xaa80;
    const LIVE_SOURCE: usize = 0xae80;
    const DESTINATION: usize = 0x3000;

    let mut state = ZeldaState::new();
    state.set_main_module(14);
    state.set_submodule(2);
    state.set_animated_tile_data_source_address(LIVE_SOURCE as u16);
    state.set_animated_tile_vram_destination_address(DESTINATION as u16);
    state.ram[CAPTURED_SOURCE..CAPTURED_SOURCE + 0x400].fill(0x11);
    state.ram[LIVE_SOURCE..LIVE_SOURCE + 0x400].fill(0x22);
    state.ppu.vram[DESTINATION..DESTINATION + 0x200].fill(0x3333);
    state.capture_display_snapshot();

    let entry_frame = state.game_state.frame;
    let captured_dma = PreMainAnimatedTileDma {
        source_address: CAPTURED_SOURCE,
        destination_address: DESTINATION,
        data: state.ram[CAPTURED_SOURCE..CAPTURED_SOURCE + 0x400].to_vec(),
    };
    state.pre_main_graphics_dma = Some(PreMainGraphicsDma {
        entry_frame,
        entry_plan: rom_graphics_dma_plan_at_host_boundary(entry_frame),
        entry_link_handler_state: 0,
        animated_tile: Some(captured_dma),
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        obj_vram: state.ppu.vram.clone(),
        oam_shadow: vec![0; state.ppu.oam.len() * 2],
    });
    state.next_core_nmi_active_scanout_uses_host_animated_bg_operands = Some(true);

    state.nmi_do_updates();

    assert_eq!(state.ppu.vram[DESTINATION], 0x2222);
    assert!(state
        .next_core_nmi_active_scanout_uses_host_animated_bg_operands
        .is_none());
    assert_eq!(
        state.game_state.display.animated_tile_data_source_usize(),
        LIVE_SOURCE
    );
    assert!(state
        .pre_main_graphics_dma
        .as_ref()
        .unwrap()
        .animated_tile
        .is_some());
    let receipt = state
        .display_snapshot
        .as_ref()
        .unwrap()
        .effective_presented_dma
        .as_ref()
        .unwrap();
    assert!(receipt.vram_writes.is_empty());
    assert_eq!(receipt.decoded_bg_vram_writes[0], (DESTINATION, 0x1111));
    assert_eq!(receipt.decoded_bg_vram_writes.len(), 0x200);

    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .effective_presented_dma = None;

    state.nmi_do_updates();

    assert_eq!(state.ppu.vram[DESTINATION], 0x2222);
    assert!(state
        .display_snapshot
        .as_ref()
        .unwrap()
        .effective_presented_dma
        .is_none());
    assert!(state
        .pre_main_graphics_dma
        .as_ref()
        .unwrap()
        .animated_tile
        .is_some());
}

#[test]
fn gameplay_leading_nmi_does_not_restore_stale_animated_bg_over_full_tilemap() {
    let mut state = ZeldaState::new();
    let destination = 0x3b00;
    state.set_main_module(7);
    state.set_submodule(0);
    state.set_animated_tile_vram_destination_address(destination as u16);
    state.ppu.vram[destination..destination + 0x200].fill(0x1111);
    state.capture_display_snapshot();

    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .host_boundary_animated_bg_scanout = Some(AnimatedBgScanout {
        destination_address: destination,
        vram: vec![0x2222; 0x200],
        logical_sources: crate::chr_source::VramChrSourceTable::default(),
        preview_sources: crate::chr_source::VramChrSourceTable::default(),
    });
    state.ppu.vram[destination..destination + 0x200].fill(0x3333);

    let presented_word = state.with_display_snapshot(|display| display.ppu.vram[destination]);

    assert_eq!(presented_word, 0x3333);
    assert_eq!(state.ppu.vram[destination], 0x3333);
}

#[test]
fn animated_bg_scanout_requires_a_captured_dma_source() {
    let mut state = ZeldaState::new();
    state.ppu.vram[0] = 0x1111;
    state.capture_display_snapshot();
    state.ppu.vram[0] = 0x2222;

    let presented_word = state.with_display_snapshot(|display| display.ppu.vram[0]);

    assert_eq!(presented_word, 0x2222);
}

#[test]
fn rom_timed_audio_commands_written_by_main_wait_for_the_next_nmi() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(20);
    state.attract_scene_mut().set_state(9);
    state.set_screen_brightness(1);
    state.set_bg_mode(9);
    state.set_last_music_control(6);

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.system_signals.music_control(), 0xf1);
    assert_eq!(state.game_state.system_signals.last_music_control(), 6);
}

#[test]
fn resumed_caller_audio_write_precedes_the_following_nmi_sample() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_music_control(0);
    state.set_current_music_control(0);
    state.set_last_music_control(0xf1);
    state.dungeon_landing_entry_started_after_leading_nmi = true;
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
        1,
    );
    state.game_execution_scheduler.begin_host_frame();

    // The final scheduled slice still owns the interrupted C stack. Its
    // caller suffix must run before the following NMI samples audio commands.
    state.prepare_audio_nmi_for_main_boundary(state.game_state.frame);
    assert!(!state.audio_nmi_processed_before_main);
    let audio_follows_host_publication =
        state.resumed_dungeon_caller_audio_follows_host_publication();
    assert!(audio_follows_host_publication);

    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
        ))
    );
    assert!(state
        .game_execution_scheduler
        .resumed_call_stack_is_before_nmi());
    state
        .game_execution_scheduler
        .mark_audio_nmi_after_host_publication();

    // Model the resumed C suffix's write. The host audio batch has already
    // been published when its following NMI becomes reachable.
    state.set_music_control(0x10);
    state.zelda_push_apu_state();
    assert!(state
        .game_execution_scheduler
        .take_audio_nmi_after_host_publication());
    state.interrupt_nmi_audio_parts();
    state.audio_nmi_processed_before_main = true;

    assert!(state.audio_nmi_processed_before_main);
    assert_eq!(state.game_state.system_signals.music_control(), 0);
    assert_eq!(
        state.game_state.system_signals.current_music_control(),
        0x10
    );
    assert_eq!(state.game_state.system_signals.last_music_control(), 0x10);
    assert_eq!(state.zelda_debug_apu_write_ports()[0], 0x10);
    assert!(!state
        .game_execution_scheduler
        .take_audio_nmi_after_host_publication());
}

#[test]
fn landing_entry_before_leading_nmi_keeps_ordinary_audio_publication() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_music_control(0x10);
    state.set_current_music_control(0);
    state.set_last_music_control(0xf1);
    state.dungeon_landing_entry_started_after_leading_nmi = false;
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
        1,
    );
    state.game_execution_scheduler.begin_host_frame();

    assert!(!state.resumed_dungeon_caller_audio_follows_host_publication());
    state.prepare_audio_nmi_for_main_boundary(state.game_state.frame);

    assert!(state.audio_nmi_processed_before_main);
    assert_eq!(state.game_state.system_signals.music_control(), 0);
    assert_eq!(state.game_state.system_signals.last_music_control(), 0x10);
}

#[test]
fn name_player_tilemap_finishes_after_the_intervening_nmi_slice() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(0xa680);
    state.set_main_module(4);
    state.set_submodule(1);

    state.module_name_player_1();

    assert_eq!(state.game_state.frame.submodule, 1);
    assert!(
        state.pre_main_caller_continuation_is(PreMainCallerContinuation::NamePlayerTilemapUpload)
    );
    assert!(state.rom_load_partial_nmi_this_frame);
    assert_eq!(state.ram[NMI_LOAD_BG_FROM_VRAM], 0);

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.submodule, 2);
    assert!(state
        .game_execution_scheduler
        .pre_main_caller_continuation()
        .is_none());
    assert_eq!(state.ram[NMI_LOAD_BG_FROM_VRAM], 0);
    let terminator = state.game_state.display.vram_upload_buffer_base()
        + 4
        + select_file::SELECT_FILE_CHECKERBOARD_TILE_COUNT * 2;
    assert_eq!(read_le_u16(&state.ram, terminator), 0xffff);
}

#[test]
fn penultimate_landing_palette_zero_suspends_its_pre_completion_caller_return() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(14);
    state.set_dungeon_room_index(0x41);
    state.set_countdown_word(1);
    state.set_darkening_or_lightening_screen_word(2);
    state.set_mosaic_target_level_word(0);
    state.dungeon_torch_mut().set_lights_out_request(1);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedAfterModule,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 14,
        palette_countdown: 0,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });

    state.Module07_02_FadedFilter();

    assert_eq!(state.game_state.display.palette_filter.countdown(), 0);
    assert_eq!(state.game_state.frame.subsubmodule, 14);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::FadedFilterPreCompletionCallerReturn,
        })
    );
    assert_eq!(
        state.dungeon_faded_filter_palette_completion_host_frame,
        Some(state.frame_ctr_dbg)
    );
}

#[test]
fn full_tilemap_upload_publishes_vram_at_the_following_nmi() {
    assert!(rom_full_tilemap_scanout_retains_uploaded_region(true, 0));
    assert!(!rom_full_tilemap_scanout_retains_uploaded_region(true, 1));
    assert!(!rom_full_tilemap_scanout_retains_uploaded_region(false, 0));

    let mut state = ZeldaState::new();
    state.set_pending_nmi_subroutine(1);
    state.set_nmi_load_target_page(14);
    let (tilemap_start, tilemap_words) =
        full_tilemap_nmi_vram_region(14).expect("valid NMI tilemap destination");
    let outside_tilemap = tilemap_start + tilemap_words;
    state.ppu.vram[tilemap_start] = 0x1111;
    state.ppu.vram[outside_tilemap] = 0xaaaa;
    state.capture_display_snapshot();
    state.ppu.vram[tilemap_start] = 0x2222;
    state.ppu.vram[outside_tilemap] = 0xbbbb;
    assert_eq!(
        state.with_display_snapshot(|display| [
            display.ppu.vram[tilemap_start],
            display.ppu.vram[outside_tilemap],
        ]),
        [0x1111, 0xbbbb],
        "only the pending tilemap DMA destination retains its pre-NMI words"
    );

    state.nmi_forced_blank_scanlines_pending = 1;
    state.ppu.vram[tilemap_start] = 0x3333;
    state.capture_display_snapshot();
    state.ppu.vram[tilemap_start] = 0x4444;
    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[tilemap_start]),
        0x4444
    );
}

#[test]
fn explicit_force_blank_event_owns_the_active_display_suffix() {
    // C WorldMap_FadeOut calls EnableForceBlank from the main thread. The ROM
    // reaches that routine at V=49 on the standard route, so the direct $2100
    // write owns output row 48 onward even though the previously published PPU
    // generation was not blank.
    assert!(live_forced_blank_for_scanout(false, None, Some(48), false));
    assert!(!live_forced_blank_for_scanout(false, None, Some(48), true));
    assert_eq!(
        resolve_active_display_blanking_scanout(false, Some(30), true),
        ActiveDisplayBlankingScanout {
            suffix_start_scanline: Some(30),
            retain_prior_surface: true,
        }
    );
    assert_eq!(
        resolve_active_display_blanking_scanout(true, None, false),
        ActiveDisplayBlankingScanout {
            suffix_start_scanline: None,
            retain_prior_surface: true,
        }
    );
}

#[test]
fn bg_scroll_scanout_replays_the_nmi_register_write_order() {
    let mut ppu = snes::ppu::PpuState::default();
    ppu.scroll_prev = 0x91;
    ppu.scroll_prev2 = 0x35;
    let register_bytes = [
        [0x91, 0x24, 0x00, 0x41],
        [0x18, 0x02, 0x00, 0x02],
        [0x00, 0x00, 0x00, 0x00],
    ];
    let predicted = BgScrollRegisterScanout::after_nmi_writes(&ppu, register_bytes);

    for (layer, [h_low, h_high, v_low, v_high]) in register_bytes.into_iter().enumerate() {
        let h_register = 0x0d + (layer as u8) * 2;
        let v_register = h_register + 1;
        ppu.write(h_register, h_low);
        ppu.write(h_register, h_high);
        ppu.write(v_register, v_low);
        ppu.write(v_register, v_high);
    }

    assert_eq!(predicted, BgScrollRegisterScanout::capture(&ppu));
    assert_eq!(predicted.offsets[0], [0x2491, 0x4100]);
    assert_eq!(predicted.offsets[1], [0x0218, 0x0200]);
}

#[test]
fn completed_scroll_can_start_the_source_next_iteration_after_its_captured_boundary() {
    let mut state = ZeldaState::new();
    state.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::BeforeNextVblank,
    );
    state.finish_dialogue_scroll_remaining_pixels();
    state.stage_early_dialogue_scroll_completion(DialogueTextScanout {
        vram: vec![0; crate::PresentedDialogueText::WORD_COUNT],
        ..DialogueTextScanout::default()
    });
    state.advance_dialogue_scroll_display_boundary();
    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionStagedAfterFrozenScanout,
    );
    publish_staged_dialogue_text_dma(&mut state);

    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletedScroll,
    );
    state.begin_dialogue_scroll(
        DialogueTextGeneration::CurrentRenderBuffer,
        DialogueScrollCompletionTiming::BeforeNextVblank,
    );
    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CopyingRemainingPixels {
            completion_timing: DialogueScrollCompletionTiming::BeforeNextVblank,
        },
    );
}

#[test]
fn live_scroll_preserves_the_caller_through_two_two_one_source_copies() {
    let mut state = ZeldaState::new();
    state.set_main_module(14);
    state.set_submodule(2);
    state.messaging_state_mut().set_module(1);
    state.messaging_state_mut().set_text_render_state(3);
    state.messaging_state_mut().set_dialogue_scroll_speed(4);
    for index in 0..crate::PresentedDialogueText::WORD_COUNT {
        state.set_messaging_render_buffer_word(index, index as u16);
    }
    let mut atomic = state.clone();
    assert!(!atomic.render_text_scroll_pixels(5));
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    let install = |state: &mut ZeldaState, entered, copies, returned| {
        state.original_timing_semantic_receipts = Some(
            OriginalTimingHostReceipts::new(0, 0, vec![]).with_dialogue_scroll_progress(vec![
                crate::DialogueScrollProgressReceipt {
                    entered,
                    completed_pixel_passes: copies,
                    returned,
                },
            ]),
        );
    };
    install(&mut state, true, 2, false);
    assert!(!state.RenderText_Draw_Scroll(0, u64::MAX));
    assert!(!state.dialogue_scroll_cpu_is_idle());
    install(&mut state, false, 2, false);
    state.zelda_run_game_loop_body_with_dialogue_text_dma(
        Some(crate::MainLoopProgress::CallStackContinued),
        &mut None,
        None,
        None,
    );
    assert!(state.dialogue_scroll_is_copying_remaining_pixels());
    assert_eq!(
        state.game_state.messaging.runtime.dialogue_msg_read_pos(),
        0
    );
    install(&mut state, false, 1, true);
    state.zelda_run_game_loop_body_with_dialogue_text_dma(
        Some(crate::MainLoopProgress::CallStackContinued),
        &mut None,
        None,
        None,
    );
    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::ReturnOnly
    );
    assert_eq!(
        state.game_state.messaging.render_buffer,
        atomic.game_state.messaging.render_buffer
    );
    assert_eq!(
        state.game_state.messaging.dialogue_source_offset,
        atomic.game_state.messaging.dialogue_source_offset
    );
}

#[test]
fn graphics_dma_plan_separates_operands_from_visible_scanout() {
    assert!(!rom_display_oam_publication_is_deferred(
        7, 0, 0, false, false
    ));
    // The overworld transition pipeline (submodules 1..=8) holds the main loop
    // across its load slices, so the OAM scanout keeps the retained boundary
    // cadence (measured at route frames 6913/6931/6944/6948).
    assert_eq!(
        rom_graphics_dma_plan(9, 1),
        GraphicsDmaPlan {
            oam_operands: GraphicsDmaGeneration::LiveAfterMain,
            oam_scanout: OamScanoutSource::RetainCapturedBeforeNmi,
            link_obj_scanout: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_operands: GraphicsDmaGeneration::LiveAfterMain,
            animated_bg_operands: GraphicsDmaGeneration::LiveAfterMain,
            animated_bg_scanout: AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi,
        },
    );
    assert_eq!(
        rom_graphics_dma_plan(9, 5),
        GraphicsDmaPlan {
            oam_operands: GraphicsDmaGeneration::LiveAfterMain,
            oam_scanout: OamScanoutSource::RetainCapturedBeforeNmi,
            link_obj_scanout: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_operands: GraphicsDmaGeneration::LiveAfterMain,
            animated_bg_operands: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            animated_bg_scanout: AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi,
        },
    );
    assert_eq!(
        rom_graphics_dma_plan(0x11, 7),
        GraphicsDmaPlan {
            oam_operands: GraphicsDmaGeneration::LiveAfterMain,
            oam_scanout: OamScanoutSource::RetainCapturedBeforeNmi,
            link_obj_scanout: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_operands: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            animated_bg_operands: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            animated_bg_scanout: AnimatedBgScanoutGeneration::LiveAfterNmi,
        },
    );
    for submodule in [8, 0x10] {
        let plan = rom_graphics_dma_plan(7, submodule);
        assert_eq!(plan.oam_scanout, OamScanoutSource::RetainCapturedBeforeNmi);
        assert_eq!(plan.link_obj_scanout, GraphicsDmaGeneration::LiveAfterMain);
        assert_eq!(
            plan.link_obj_operands,
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        );
    }
    let shutter_plan = rom_graphics_dma_plan(7, 5);
    assert_eq!(
        shutter_plan.oam_operands,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(
        shutter_plan.oam_scanout,
        OamScanoutSource::ComposeLiveAfterNmi
    );
    assert_eq!(
        shutter_plan.link_obj_operands,
        GraphicsDmaGeneration::LiveAfterMain
    );
    assert_eq!(
        rom_graphics_dma_plan(9, 0x0a),
        GraphicsDmaPlan {
            oam_operands: GraphicsDmaGeneration::LiveAfterMain,
            oam_scanout: OamScanoutSource::RetainCapturedBeforeNmi,
            link_obj_scanout: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_operands: GraphicsDmaGeneration::LiveAfterMain,
            animated_bg_operands: GraphicsDmaGeneration::LiveAfterMain,
            animated_bg_scanout: AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi,
        },
    );
    assert_eq!(
        rom_graphics_dma_plan(14, 7),
        GraphicsDmaPlan {
            oam_operands: GraphicsDmaGeneration::LiveAfterMain,
            oam_scanout: OamScanoutSource::ComposeLiveAfterNmi,
            link_obj_scanout: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_operands: GraphicsDmaGeneration::LiveAfterMain,
            animated_bg_operands: GraphicsDmaGeneration::LiveAfterMain,
            animated_bg_scanout: AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi,
        },
    );
    let mut subtile_landing = crate::game_state::FrameState::default();
    subtile_landing.main_module = 7;
    subtile_landing.submodule = 1;
    subtile_landing.subsubmodule = 4;
    assert_eq!(
        rom_graphics_dma_plan_at_host_boundary(subtile_landing),
        GraphicsDmaPlan {
            oam_operands: GraphicsDmaGeneration::LiveAfterMain,
            oam_scanout: OamScanoutSource::RetainResidentPpuOam,
            link_obj_scanout: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_operands: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            animated_bg_operands: GraphicsDmaGeneration::LiveAfterMain,
            animated_bg_scanout: AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi,
        },
    );
    subtile_landing.subsubmodule = 5;
    assert_eq!(
        rom_graphics_dma_plan_at_host_boundary(subtile_landing),
        rom_graphics_dma_plan_at_host_boundary(crate::game_state::FrameState {
            subsubmodule: 4,
            ..subtile_landing
        }),
    );
    subtile_landing.subsubmodule = 6;
    assert_eq!(
        rom_graphics_dma_plan_at_host_boundary(subtile_landing),
        rom_graphics_dma_plan_at_host_boundary(crate::game_state::FrameState {
            subsubmodule: 5,
            ..subtile_landing
        }),
    );
    subtile_landing.subsubmodule = 7;
    assert_eq!(
        rom_graphics_dma_plan_at_host_boundary(subtile_landing),
        rom_graphics_dma_plan_at_host_boundary(crate::game_state::FrameState {
            subsubmodule: 6,
            ..subtile_landing
        }),
    );
    assert!(rom_display_oam_publication_is_deferred(
        4, 3, 0, true, false
    ));
    assert!(rom_display_oam_publication_is_deferred(
        4, 3, 0, false, true
    ));
    assert!(rom_display_oam_publication_is_deferred(
        4, 3, 0, false, false
    ));
    assert!(rom_display_oam_publication_is_deferred(
        14, 7, 0, false, false
    ));
    assert!(!rom_display_oam_publication_is_deferred(
        4, 2, 0, false, false
    ));
    assert!(!rom_display_memory_publication_is_deferred(7, 0, 0, false));
    assert!(rom_display_memory_publication_is_deferred(14, 2, 3, false));
    assert!(!rom_display_memory_publication_is_deferred(14, 2, 4, false));
    assert!(rom_dungeon_exit_entry_crosses_nmi_boundary(
        0x0f, 0, 0x0f, 1, false
    ));
    assert!(rom_dungeon_exit_entry_crosses_nmi_boundary(
        0x0f, 0, 0x0f, 0, true
    ));
    assert!(!rom_dungeon_exit_entry_crosses_nmi_boundary(
        0x0f, 0, 0x0f, 0, false
    ));
    assert!(!rom_dungeon_exit_entry_crosses_nmi_boundary(
        0x0f, 1, 0x0f, 1, true
    ));
    assert_eq!(
        GraphicsDmaGeneration::HostBoundaryBeforeMain.resolve_live_override(false),
        GraphicsDmaGeneration::HostBoundaryBeforeMain,
    );
    assert_eq!(
        GraphicsDmaGeneration::HostBoundaryBeforeMain.resolve_live_override(true),
        GraphicsDmaGeneration::LiveAfterMain,
    );

    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.ppu.vram[0] = 0x1111;
    state.ppu.vram[0x4000] = 0x4444;
    state.ppu.oam[0] = 0x2222;
    state.ppu.cgram[0] = 0x3333;
    state.capture_display_snapshot();
    state.ppu.vram[0] = 0xaaaa;
    state.ppu.vram[0x4000] = 0xdddd;
    state.ppu.oam[0] = 0xbbbb;
    state.ppu.cgram[0] = 0xcccc;

    let captured = state.with_display_snapshot(|display| {
        (
            display.ppu.vram[0],
            display.ppu.vram[0x4000],
            display.ppu.oam[0],
            display.ppu.cgram[0],
        )
    });

    assert_eq!(captured, (0xaaaa, 0xdddd, 0xbbbb, 0xcccc));
}

#[test]
fn retained_display_memory_resolves_link_obj_from_its_snapshot_dma_words() {
    let mut link_graphics = vec![0; 0x7000];
    link_graphics[0x4d80..0x4d82].copy_from_slice(&[0x22, 0x23]);
    link_graphics[0x4dc0..0x4dc2].copy_from_slice(&[0x33, 0x34]);
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 58];
    put_test_asset(&mut data, &mut ranges, 57, link_graphics);

    let mut state = ZeldaState::new();
    state.assets = Some(AssetPack::from_data_ranges(data, ranges));
    state.set_main_module(7);
    state.set_submodule(0);
    write_le_u16(&mut state.ram, DMA_SOURCE_ADDR_4, 0xcd80);
    state.ppu.vram[0x4020] = 0x1111;
    state.capture_display_snapshot();
    state.display_snapshot.as_mut().unwrap().vram_generation =
        DisplayVramGeneration::RetainCapturedBeforeNmi;

    // The coarse following slice can skip NMI_DoUpdates after latching long
    // work. Its PPU is stale and its CPU source has already advanced again;
    // the retained scanout owns the source word captured with the snapshot.
    write_le_u16(&mut state.ram, DMA_SOURCE_ADDR_4, 0xcdc0);
    state.ppu.vram[0x4020] = 0x1111;
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishItemReceiptGraphics {
            continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                gfx: 0x14,
                ground_apress_tail: None,
            },
        },
        ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES,
    );

    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[0x4020]),
        0x2322,
    );
}

#[test]
fn display_snapshot_keeps_link_chr_identity_with_its_vram_generation() {
    let mut state = ZeldaState::new();
    state.set_main_module(9);
    state.set_submodule(6);
    let link_slot = 0x4000 / 16;
    let background_slot = 0x2000 / 16;
    let record = |table: &mut crate::chr_source::VramChrSourceTable, slot, hash| {
        table.record_tile_content_hash(slot, crate::chr_source::CHR_KIND_LINK_CONTENT, hash);
    };

    state.ppu.vram[0x4000] = 0x1111;
    record(&mut state.vram_chr_source, link_slot, 0x1111_1111);
    record(&mut state.vram_chr_preview_source, link_slot, 0x2222_2222);
    state.capture_display_snapshot();

    state.ppu.vram[0x4000] = 0xaaaa;
    record(&mut state.vram_chr_source, link_slot, 0xaaaa_aaaa);
    record(&mut state.vram_chr_preview_source, link_slot, 0xbbbb_bbbb);
    record(&mut state.vram_chr_source, background_slot, 0xcccc_cccc);

    let visible = state.with_display_snapshot(|display| {
        (
            display.ppu.vram[0x4000],
            display.vram_chr_source.get(link_slot),
            display.vram_chr_preview_source.get(link_slot),
            display.vram_chr_source.get(background_slot),
        )
    });
    assert_eq!(
        (
            visible.0,
            visible.1.pack,
            visible.1.tile_off,
            visible.2.pack,
            visible.2.tile_off,
            visible.3.pack,
            visible.3.tile_off,
        ),
        (0x1111, 0x1111, 0x1111, 0x2222, 0x2222, 0xcccc, 0xcccc),
    );
    assert_eq!(state.ppu.vram[0x4000], 0xaaaa);
    assert_eq!(state.vram_chr_source.get(link_slot).pack, 0xaaaa);
}

#[test]
fn trailing_nmi_force_blank_preserves_the_completed_fields_visible_rows() {
    let mut state = ZeldaState::new();
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();
    state.set_screen_brightness(0x0f);
    state.ppu.brightness = 0x0f;
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::PublishCaptured);
    state.schedule_dungeon_exit_spotlight_goal_caller(SpotlightIteration::closing(
        SpotlightIterationPhase::WholeTableAfterTablePublication,
    ));
    state.set_screen_brightness(0x80);
    state.ppu.forced_blank = true;
    state.ppu.brightness = 0;
    state.capture_display_snapshot_with_override(Some(DisplaySnapshotPublication::PublishCaptured));
    let scanout = state.with_display_snapshot(|display| {
        (
            display.ppu.forced_blank,
            display.ppu.forced_blank_from_scanline,
            display.ppu.brightness,
            display.ppu.scanout_brightness_override,
        )
    });

    assert_eq!(
        scanout,
        (true, Some(TRAILING_NMI_FORCE_BLANK_SCANLINE), 0, Some(15),)
    );

    // The same two-slice C continuation is still scheduled, but the late
    // INIDISP event was already attached to the retiring scanout above. A new
    // capture owns the live forced-blank register without replaying that
    // one-shot brightness generation.
    state.capture_display_snapshot_with_override(Some(DisplaySnapshotPublication::PublishCaptured));
    let following = state.with_display_snapshot(|display| {
        (
            display.ppu.forced_blank,
            display.ppu.brightness,
            display.ppu.scanout_brightness_override,
        )
    });
    assert_eq!(following, (true, 0, None));
}

#[test]
fn nmi_operand_consumption_preserves_the_scanout_plan() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(1);
    state.set_subsubmodule(6);
    let entry_plan = rom_graphics_dma_plan_at_host_boundary(state.game_state.frame);
    state.pre_main_graphics_dma = Some(PreMainGraphicsDma {
        entry_frame: state.game_state.frame,
        entry_plan,
        entry_link_handler_state: 0,
        animated_tile: None,
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        obj_vram: state.ppu.vram.clone(),
        oam_shadow: state.sprite_oam_shadow_buffer().to_vec(),
    });

    state.nmi_do_updates();

    assert_eq!(
        state
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.entry_plan),
        Some(entry_plan),
    );
}

#[test]
fn leading_nmi_uses_the_captured_link_high_plane_staging_buffers() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.ram[LINK_DMA_EXPANDED_HIGH_PLANES_START
        ..LINK_DMA_EXPANDED_HIGH_PLANES_START + LINK_DMA_EXPANDED_HIGH_PLANES_HALF_LEN]
        .fill(0x12);
    state.ram[LINK_DMA_EXPANDED_HIGH_PLANES_START + LINK_DMA_EXPANDED_HIGH_PLANES_HALF_LEN
        ..LINK_DMA_EXPANDED_HIGH_PLANES_START + LINK_DMA_EXPANDED_HIGH_PLANES_LEN]
        .fill(0x34);
    let entry_plan = rom_graphics_dma_plan_at_host_boundary(state.game_state.frame);
    state.pre_main_graphics_dma = Some(PreMainGraphicsDma {
        entry_frame: state.game_state.frame,
        entry_plan,
        entry_link_handler_state: 0,
        animated_tile: None,
        link_operands: PreMainLinkDmaOperands::capture(&state.ram),
        obj_vram: state.ppu.vram.clone(),
        oam_shadow: state.sprite_oam_shadow_buffer().to_vec(),
    });

    state.ram[LINK_DMA_EXPANDED_HIGH_PLANES_START
        ..LINK_DMA_EXPANDED_HIGH_PLANES_START + LINK_DMA_EXPANDED_HIGH_PLANES_LEN]
        .fill(0x56);
    state.nmi_do_updates();

    assert!(state.ppu.vram[0x4240..0x4260]
        .iter()
        .all(|&word| word == 0x1212));
    assert!(state.ppu.vram[0x4340..0x4360]
        .iter()
        .all(|&word| word == 0x3434));
}

#[test]
fn subtile_palette_filter_schedules_only_instruction_level_interruptions() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();

    // The terminal lightening pass has already toggled back to darkening and
    // advanced state 1 -> 2, but the common Module 7 return is still in flight
    // when NMI arrives. Countdown/direction cannot identify this boundary.
    state.set_subsubmodule(2);
    state.set_countdown(0);
    state.set_darkening_or_lightening_screen(0);
    state.dungeon_palette_cpu_advance_pending = Some(DungeonPaletteCpuAdvance {
        work: CpuWorkAdvance::ReachedBoundary {
            boundary: CpuRasterBoundary::CpuNmiAcceptance,
            remaining_work_master_cycles: 0,
        },
        pc: 0x0d_fcd1,
        subsubmodule: 2,
        palette_countdown: 0,
    });
    state.suspend_dungeon_subtile_palette_filter_if_cpu_interrupted();
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonSubtilePaletteFilter)
    );
    assert!(
        state
            .game_execution_scheduler
            .work_suspends_translated_call_stack(),
        "the interrupted palette loop must retain Module07's caller suffix until its return"
    );

    state.game_execution_scheduler.finish_work();
    state.set_subsubmodule(1);
    state.set_countdown(2);
    state.dungeon_palette_cpu_advance_pending = Some(DungeonPaletteCpuAdvance {
        work: CpuWorkAdvance::Complete,
        pc: 0x00_8036,
        subsubmodule: 1,
        palette_countdown: 2,
    });
    state.suspend_dungeon_subtile_palette_filter_if_cpu_interrupted();
    assert!(!state.game_execution_scheduler.work_is_pending());

    assert_eq!(
        dungeon_subtile_palette_filter_return_obj_scanout(),
        ObjScanoutGenerations {
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
        },
        "the return keeps resident OAM/raw CHR while its NMI publishes a newer decoded Link cache"
    );
}

#[test]
fn hud_tilemap_upload_publishes_at_the_following_scanout() {
    let mut state = ZeldaState::new();
    state.set_message_dma_destination_address(0x60b9);
    state.ppu.vram[0] = 0x1111;
    state.ppu.vram[0x60b9] = 0x2222;
    state.increment_hud_update_flag();
    state.capture_display_snapshot();

    state.ppu.vram[0] = 0xaaaa;
    state.ppu.vram[0x60b9] = 0xbbbb;

    let captured =
        state.with_display_snapshot(|display| (display.ppu.vram[0], display.ppu.vram[0x60b9]));

    assert_eq!(captured, (0xaaaa, 0x2222));
}

#[test]
fn nmi_phase_classifier_preserves_same_host_and_carry_in_acceptance_ownership() {
    assert_eq!(
        classify_original_timing_nmi_phases_with_ownership(
            false,
            &[
                OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld),
                OriginalTimingNmiPhase::HandlerCompleted,
            ],
        ),
        OriginalTimingNmiPhaseClassification {
            handler_completion: OriginalTimingNmiHandlerCompletionOwner::AcceptedInThisSequence,
            publication_pending_at_exit: false,
        },
    );
    assert_eq!(
        classify_original_timing_nmi_phases_with_ownership(
            true,
            &[OriginalTimingNmiPhase::HandlerCompleted],
        ),
        OriginalTimingNmiPhaseClassification {
            handler_completion: OriginalTimingNmiHandlerCompletionOwner::PendingAtSequenceEntry,
            publication_pending_at_exit: false,
        },
    );
}

#[test]
fn carry_in_handler_rejects_missing_or_closed_display_before_timeline_consumption() {
    for timeline_kind in ["return", "uninterrupted", "interrupted"] {
        for close_existing_snapshot in [false, true] {
            let mut state = ZeldaState::new();
            state.original_timing_owner = OriginalTimingOwnerState::Live;
            state.original_timing_nmi_publication_pending = true;
            state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
            state.pending_main_loop_common_suffix =
                Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
            let mut semantic = vec![OriginalTimingSemanticReceipt::NmiHandlerCompleted];
            semantic.push(OriginalTimingSemanticReceipt::MainLoopProgress(
                match timeline_kind {
                    "return" => crate::MainLoopProgress::CallStackContinued,
                    "uninterrupted" | "interrupted" => crate::MainLoopProgress::IterationStarted,
                    _ => unreachable!(),
                },
            ));
            match timeline_kind {
                "return" => {
                    semantic.push(OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted)
                }
                "interrupted" => semantic.push(OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    crate::MainLoopInterruption::LinkOam,
                )),
                "uninterrupted" => {}
                _ => unreachable!(),
            }
            state.original_timing_semantic_receipts =
                Some(OriginalTimingHostReceipts::new(806, 0x0008, semantic));
            if close_existing_snapshot {
                state.capture_display_snapshot();
                state
                    .display_snapshot
                    .as_mut()
                    .unwrap()
                    .accepts_nmi_dma_receipts = false;
            }
            let semantic_before = state
                .original_timing_semantic_receipts
                .as_ref()
                .unwrap()
                .semantic
                .clone();

            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match timeline_kind {
                    "return" => {
                        state.take_original_timing_main_loop_return_timeline();
                    }
                    "uninterrupted" => {
                        state.take_original_timing_uninterrupted_main_loop_timeline(
                            crate::MainLoopProgress::IterationStarted,
                        );
                    }
                    "interrupted" => {
                        state.take_original_timing_main_loop_interruption_timeline(Some(
                            crate::MainLoopInterruption::LinkOam,
                        ));
                    }
                    _ => unreachable!(),
                }));

            assert!(result.is_err());
            assert_eq!(
                state
                    .original_timing_semantic_receipts
                    .as_ref()
                    .unwrap()
                    .semantic,
                semantic_before,
            );
            assert!(state.original_timing_nmi_publication_pending);
            assert_eq!(
                state.original_timing_pending_nmi_update_gate,
                Some(NmiUpdateGate::LatchHeld),
            );
            assert_eq!(
                state.pending_main_loop_common_suffix,
                Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
            );
        }
    }
}

#[test]
fn trailing_acceptance_publishes_sparse_obj_continuity_without_replacing_other_domains() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(0);
    state.set_submodule(5);
    state.set_frame_counter(79);
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state.clear_pending_polyhedral_update();
    state.sync_native_game_state_from_ram();

    state.ppu.vram[0x5800..0x5c00].fill(0x1357);
    state.ppu.obj_tile_adr1 = 0x4000;
    state.ppu.obj_tile_adr2 = 0x5000;
    let mut stale_accepted_obj = vec![0x2222; state.ppu.vram.len()];
    stale_accepted_obj[0x5a20] = 0x2223;
    state.set_obj_vram_latch_traced(Some(stale_accepted_obj));
    state.capture_display_snapshot();
    state.ppu.vram[0x5800..0x5c00].fill(0x2468);
    let mut live_obj = vec![0x3333; state.ppu.vram.len()];
    live_obj[0x5a20] = 0x3334;
    state.set_obj_vram_latch_traced(Some(live_obj.clone()));
    state.ppu.oam[0] = 0xabcd;
    state.ppu.cgram[7] = 0x1357;
    state.set_screen_brightness(6);
    state.ppu.brightness = 6;
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    let presentation = crate::PresentedObjTiles::new(
        vec![0x5a20],
        vec![1; crate::PresentedObjTiles::PIXELS_PER_TILE],
    )
    .unwrap();
    state
        .install_original_timing_host_receipts(
            OriginalTimingHostReceipts::new(
                807,
                0x0008,
                vec![
                    OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                    OriginalTimingSemanticReceipt::MainLoopProgress(
                        crate::MainLoopProgress::CallStackContinued,
                    ),
                    OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                ],
            )
            .with_presented_obj_tiles(presentation),
        )
        .unwrap();

    state.run_frame_internal(0x0008, crate::RUN_MAIN);

    assert_eq!(
        state.display_snapshot.as_ref().unwrap().ppu.vram[0x5800..0x5c00],
        [0x2468; 0x400],
        "the trailing acceptance must capture the post-return generation instead of reusing the completed carry-in scanout",
    );
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::Open),
    );
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(!state.game_state.display.nmi_update_is_latched());
    let presented = state.with_display_snapshot(|display| {
        let obj = display.ppu.obj_vram_latch.as_ref().unwrap();
        (
            obj[0x5a20],
            obj[0x5a27],
            obj[0x5a28],
            obj[0x5a2f],
            obj[0x5a30],
            display.ppu.oam[0],
            display.ppu.cgram[7],
            display.ppu.brightness,
        )
    });
    assert_eq!(
        presented,
        (0x00ff, 0x00ff, 0, 0, 0x3333, 0xabcd, 0x1357, 0),
        "a latch-held handler suppresses conditional DMA, but its unconditional WritePpuRegisters still uses the acceptance snapshot's operands",
    );
    assert_eq!(state.ppu.obj_vram_latch.as_ref(), Some(&live_obj));
}

#[test]
fn second_acceptance_replaces_the_same_host_completed_handler_scanout_before_carrying() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(0);
    state.set_submodule(5);
    state.set_frame_counter(79);
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state.clear_pending_polyhedral_update();
    state.sync_native_game_state_from_ram();

    state.ppu.vram[0x5800..0x5c00].fill(0x1357);
    state.capture_display_snapshot();
    state.ppu.vram[0x5800..0x5c00].fill(0x2468);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            808,
            0x0008,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                    high: 0,
                    low: 0,
                    high_filtered: 0,
                    low_filtered: 0,
                }),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            ],
        ))
        .unwrap();

    state.run_frame_internal(0x0008, crate::RUN_MAIN);

    assert_eq!(
        state.display_snapshot.as_ref().unwrap().ppu.vram[0x5800..0x5c00],
        [0x2468; 0x400],
    );
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(state.game_state.display.nmi_update_is_latched());
}

#[test]
fn carried_open_handler_refines_only_its_receptive_scanout_with_completed_nmi_effects() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_main_module(0);
    state.set_submodule(5);
    state.set_animated_tile_data_source_address(1);

    let expected_oam = (0..0x220)
        .map(|index| (index as u8).wrapping_mul(13).wrapping_add(7))
        .collect::<Vec<_>>();
    state.ram[OAM_BUF..OAM_BUF + expected_oam.len()].copy_from_slice(&expected_oam);
    state.ppu.oam.fill(0x1111);

    state.set_pending_nmi_subroutine(1);
    state.set_nmi_load_target_page(14);
    write_le_u16(
        &mut state.ram,
        crate::game_state::constants::nmi::TILEMAP_UPLOAD_BUFFER,
        0x4567,
    );
    let (tilemap_start, _) = full_tilemap_nmi_vram_region(14).unwrap();
    state.ppu.vram[tilemap_start] = 0x2222;

    state.set_main_color(7, 0x1357);
    state.ppu.cgram[7] = 0x3333;
    state.increment_cgram_update_flag();
    state.set_screen_brightness(6);
    state.ppu.brightness = 3;
    state.set_main_screen_layers(0x10);
    state.set_sub_screen_layers(0x05);
    state.set_bg1_v_copy(0x1234);
    state.set_bg3_h_copy2(0x5678);
    state.set_bg3_v_copy2(0x9abc);
    state.ppu.screen_enabled = [0x10, 0x05];
    state.ppu.scroll_prev = 0x91;
    state.ppu.scroll_prev2 = 0x82;
    state.ppu.m7_prev = 0x73;

    state.capture_display_snapshot();
    let exact_acceptance_operands =
        nmi_ppu_register_operands_from_snapshot(state.display_snapshot.as_ref().unwrap());
    // The translated caller has advanced these native mirrors by the time the
    // atomic host wrapper captures the already-accepted handler. The source
    // handler still owns the typed register operands sampled at acceptance,
    // not this post-interruption RAM generation.
    state.set_main_screen_layers(0x15);
    state.set_sub_screen_layers(0x00);
    state.ppu.scroll_prev = 0x19;
    state.ppu.scroll_prev2 = 0x28;
    state.ppu.m7_prev = 0x37;
    state.capture_display_snapshot();
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
    state.original_timing_pending_nmi_ppu_register_operands = Some(exact_acceptance_operands);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            809,
            0x0008,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                    high: 0xa5,
                    low: 0x5a,
                    high_filtered: 0x81,
                    low_filtered: 0x18,
                }),
            ],
        ))
        .unwrap();

    let owns_dispatch = state.begin_original_timing_host_dispatch(0x0008);
    assert!(owns_dispatch);
    let nmi_phases = state.take_original_timing_nmi_phases();
    assert_eq!(nmi_phases, vec![OriginalTimingNmiPhase::HandlerCompleted],);
    let classification = classify_original_timing_nmi_phases_with_ownership(true, &nmi_phases);
    assert_eq!(
        classification.handler_completion,
        OriginalTimingNmiHandlerCompletionOwner::PendingAtSequenceEntry,
    );
    assert!(state
        .complete_original_timing_nmi_handler_for_active_scanout(
            classification.handler_completion,
            0x0008,
            None,
        )
        .completed());
    let snapshot = state.display_snapshot.as_ref().unwrap();
    assert_eq!(snapshot.ppu.oam[0], 0x1111);
    assert_eq!(snapshot.ppu.vram[tilemap_start], 0x2222);
    assert_eq!(snapshot.ppu.cgram[7], 0x3333);
    assert_eq!(snapshot.ppu.brightness, 3);
    let presented = state.with_display_snapshot(|display| {
        (
            display.ppu.oam[0],
            display.ppu.vram[tilemap_start],
            display.ppu.cgram[7],
            display.ppu.brightness,
            display.ppu.screen_enabled,
            display.ppu.scroll_prev,
            display.ppu.scroll_prev2,
            display.ppu.m7_prev,
        )
    });
    assert_eq!(
        presented,
        (
            u16::from_le_bytes([expected_oam[0], expected_oam[1]]),
            0x4567,
            0x1357,
            6,
            [0x10, 0x05],
            0x9a,
            0x56,
            0x12,
        ),
        "the carried Open handler's completed OAM, VRAM, CGRAM, and register writes must refine the acceptance-host scanout",
    );

    // A trailing acceptance captures resident hardware after this completion.
    // It must inherit the same acceptance-owned register result instead of
    // resurrecting the caller's later software mirrors.
    state.capture_display_snapshot();
    assert_eq!(
        state.with_display_snapshot(|display| {
            (
                display.ppu.screen_enabled,
                display.ppu.scroll_prev,
                display.ppu.scroll_prev2,
                display.ppu.m7_prev,
            )
        }),
        ([0x10, 0x05], 0x9a, 0x56, 0x12),
    );

    let link = &state.game_state.player.follower_link;
    assert_eq!(link.joypad1h_last(), 0xa5);
    assert_eq!(link.joypad1l_last(), 0x5a);
    assert_eq!(link.filtered_joypad_h(), 0x81);
    assert_eq!(link.filtered_joypad_l(), 0x18);
    state.finish_original_timing_host_dispatch(owns_dispatch);
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(809));
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn acceptance_register_reconstruction_retains_mode7_terminal_write_latches() {
    let mut state = ZeldaState::new();
    state.set_bg_mode(7);
    state.set_bg1_v_copy(0x1234);
    state.set_bg3_h_copy2(0x5678);
    state.set_bg3_v_copy2(0x9abc);
    state.set_mode7_center_y(0xdef0);
    state.capture_display_snapshot();

    let registers = nmi_ppu_register_scanout_from_acceptance_snapshot(
        state.display_snapshot.as_ref().unwrap(),
        None,
    );
    assert_eq!(registers.scroll_prev, 0x9a);
    assert_eq!(registers.scroll_prev2, 0x56);
    assert_eq!(registers.m7_prev, 0xde);

    state.ppu.scroll_prev = 0x19;
    state.ppu.scroll_prev2 = 0x28;
    state.ppu.m7_prev = 0x37;
    registers.publish_to(&mut state.ppu);
    assert_eq!(state.ppu.scroll_prev, 0x9a);
    assert_eq!(state.ppu.scroll_prev2, 0x56);
    assert_eq!(state.ppu.m7_prev, 0xde);
}

#[test]
fn hdma_setup_and_simple_hdma_line_write_ppu() {
    let mut state = ZeldaState::new();
    state.hdma_setup(0x0cfa87, 0x0cfa94, 0, 0, 0, 0);

    assert_eq!(state.dma.channel[6].a_adr, 0xfa87);
    assert_eq!(state.dma.channel[6].a_bank, 0x0c);
    assert_eq!(state.dma.channel[6].b_adr, 0);
    assert_eq!(state.dma.channel[7].a_adr, 0xfa94);

    state.dma.channel[6].hdma_active = true;
    let mut hdma = SimpleHdma::default();
    state.simple_hdma_init(&mut hdma, &state.dma.channel[6]);
    state.simple_hdma_do_line(&mut hdma);

    assert!(state.ppu.forced_blank);
    assert_eq!(state.ppu.brightness, 0x0f);
    assert_eq!(hdma.rep_count, 0x1f);
}

#[test]
fn retiring_window_hdma_scanout_commits_its_final_ppu_latches() {
    let mut state = ZeldaState::new();
    state.ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + 4].copy_from_slice(&[1, 0, 255, 0]);
    state.hdma_setup(0, 0x001b00, 1, 0, 0x26, 0);
    state.set_hdma_enable_mask(1 << 7);
    state.ppu.window1_left = 1;
    state.ppu.window1_right = 0;

    state.capture_display_snapshot();
    state.set_hdma_enable_mask(0);
    state.capture_display_snapshot();

    assert_eq!((state.ppu.window1_left, state.ppu.window1_right), (0, 255));
}

#[test]
fn retained_window_hdma_scanout_does_not_retire_twice() {
    let mut state = ZeldaState::new();
    state.ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + 4].copy_from_slice(&[1, 0, 255, 0]);
    state.hdma_setup(0, 0x001b00, 1, 0, 0x26, 0);
    state.set_hdma_enable_mask(1 << 7);
    state.capture_display_snapshot();

    state.set_hdma_enable_mask(0);
    state.ppu.window1_left = 7;
    state.ppu.window1_right = 8;
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::RetainPublished);

    assert_eq!((state.ppu.window1_left, state.ppu.window1_right), (7, 8));
}

#[test]
fn cgram_capture_preserves_dma_state() {
    let mut state = ZeldaState::new();
    state.set_hdma_enable_mask(0);
    state.dma.channel[1].hdma_active = true;
    state.dma.channel[6].hdma_active = true;
    let dma_before = state.dma.save_c_saveload();

    state.cgram_after_first_hdma_line();

    assert_eq!(state.dma.save_c_saveload(), dma_before);
}

#[test]
fn repeated_display_captures_preserve_dma_and_ppu_latches() {
    let mut state = ZeldaState::new();
    state.set_hdma_enable_mask(1 << 6);
    state.hdma_setup(0x0cfa87, 0, 0, 0x0d, 0, 0);
    state.dma.channel[2].hdma_active = true;
    state.dma.channel[7].hdma_active = true;
    state.ppu.scroll_prev = 0x12;
    state.ppu.scroll_prev2 = 0x34;
    let dma_before = state.dma.save_c_saveload();
    let scroll_latches_before = (state.ppu.scroll_prev, state.ppu.scroll_prev2);
    let bg_scrolls_before: [(u16, u16); 4] = std::array::from_fn(|i| {
        (
            state.ppu.bg_layer[i].h_scroll,
            state.ppu.bg_layer[i].v_scroll,
        )
    });
    let mode7_before = (state.ppu.m7_matrix, state.ppu.m7_prev);

    for _ in 0..3 {
        state.cgram_after_first_hdma_line();
        state.ppu_scanline_windows();
        assert_eq!(state.dma.save_c_saveload(), dma_before);
        assert_eq!(
            (state.ppu.scroll_prev, state.ppu.scroll_prev2),
            scroll_latches_before
        );
        assert_eq!(
            std::array::from_fn::<_, 4, _>(|i| {
                (
                    state.ppu.bg_layer[i].h_scroll,
                    state.ppu.bg_layer[i].v_scroll,
                )
            }),
            bg_scrolls_before
        );
        assert_eq!((state.ppu.m7_matrix, state.ppu.m7_prev), mode7_before);
    }
}

#[test]
fn simple_hdma_get_ptr_maps_mode7_zoom_tables() {
    let state = ZeldaState::new();

    assert_eq!(
        state.simple_hdma_get_ptr(0x0add27).unwrap()[0..4],
        [0x77, 0x01, 0x76, 0x01]
    );
    assert_eq!(
        state.simple_hdma_get_ptr(0x0ade07).unwrap()[0..4],
        [0x35, 0x01, 0x35, 0x01]
    );
    assert_eq!(
        state.simple_hdma_get_ptr(0x0adee7).unwrap()[0..4],
        [0x88, 0x00, 0x88, 0x00]
    );
    assert_eq!(
        state.simple_hdma_get_ptr(0x0adfc7).unwrap()[0..4],
        [0x70, 0x00, 0x70, 0x00]
    );
}

#[test]
fn renderer_capture_observes_pre_nmi_state_without_rewinding_live_state() {
    let mut state = ZeldaState::new();
    state.ppu.brightness = 3;
    state.ppu.vram[0] = 0x1111;
    state.ppu.vram[0x5800] = 0x3333;
    state.capture_display_snapshot();
    state.ppu.brightness = 12;
    state.ppu.vram[0] = 0x2222;
    state.ppu.vram[0x5800] = 0x4444;

    let captured = state.with_display_snapshot(|display| {
        (
            display.ppu.brightness,
            display.ppu.vram[0],
            display.ppu.vram[0x5800],
        )
    });

    assert_eq!(captured, (3, 0x2222, 0x3333));
    assert_eq!(state.ppu.brightness, 12);
    assert_eq!(state.ppu.vram[0], 0x2222);
    assert_eq!(state.ppu.vram[0x5800], 0x4444);
    assert!(state.display_snapshot.is_some());
}

#[test]
fn nmi_force_blank_gates_the_pre_nmi_display_snapshot() {
    let mut state = ZeldaState::new();
    state.ppu.forced_blank = false;
    state.capture_display_snapshot();

    // zelda3/src/nmi.c:WritePpuRegisters publishes INIDISP_copy through the
    // leading NMI. The completed register receipt, rather than the unrelated
    // future live PPU latch, owns this retiring scanout.
    state.set_screen_brightness(0x80);
    state.interrupt_nmi_for_active_scanout(0, None, false);

    assert!(state
        .display_snapshot
        .as_ref()
        .and_then(|display| display.effective_presented_dma.as_ref())
        .and_then(|receipt| receipt.completed_ppu_registers)
        .is_some_and(|registers| registers.inidisp.forced_blank));
    let captured_forced_blank = state.with_display_snapshot(|display| display.ppu.forced_blank);

    assert!(captured_forced_blank);
    assert!(state.ppu.forced_blank);
}

#[test]
fn nmi_subroutine_11_uploads_bg_char_half_to_vram() {
    let mut state = ZeldaState::new();
    write_le_u16(&mut state.ram, 0x11000, 0x1234);
    write_le_u16(&mut state.ram, 0x11002, 0xabcd);
    state.set_nmi_load_target_page(0x46);
    state.set_pending_nmi_subroutine(11);

    state.nmi_do_updates();

    assert_eq!(state.ppu.vram[0x4600], 0x1234);
    assert_eq!(state.ppu.vram[0x4601], 0xabcd);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0);
}

#[test]
fn name_file_x_scroll_both_horizontal_bits_match_c_rom_table() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_joypad1h_last(0x03);
    state.set_select_file_name_column(21);

    state.name_file_check_for_scroll_input_x();

    let select_file = &state.game_state.messaging.select_file_menu;
    assert_eq!(select_file.name_column(), 53);
    assert_eq!(select_file.name_scroll_x_step(), 1);
    assert_eq!(select_file.name_scroll_x_direction(), 2);
}

#[test]
fn rom_cpu_timing_dma_materializes_native_hdma_enable_mask() {
    let mut state = ZeldaState::new();
    state.dma.channel[0].hdma_active = true;
    state.dma.channel[7].hdma_active = false;
    state.set_hdma_enable_mask(0x80);

    let timing_dma = state.dma_with_native_hdma_enable();

    assert!(!timing_dma.channel[0].hdma_active);
    assert!(timing_dma.channel[7].hdma_active);
    assert!(state.dma.channel[0].hdma_active);
    assert!(!state.dma.channel[7].hdma_active);
}

#[test]
fn fire_debirando_dynamic_spawn_resumes_after_the_exact_source_publication() {
    let mut split = ZeldaState::new();
    let parent = 0;
    let child = 15;
    {
        let mut sprite = split.sprite_slot_view_mut(parent);
        sprite.set_state(8);
        sprite.set_sprite_type(0x64);
        sprite.set_floor(1);
        sprite.set_direction(3);
        sprite.set_head_direction(0x0e);
        sprite.set_ai_state(2);
        sprite.set_delay_main(0xfb);
        sprite.set_graphics(5);
    }
    let mut atomic = split.clone();

    split.sprite_module_initialize_properties(parent);
    split.sprite_slot_view_mut(parent).set_sprite_type(0x63);
    split.sprite_prep_load_properties(parent);
    split.sprite_prep_fire_debirando_after_property_reload_before_spawn(parent);
    let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
    split.sprite_spawn_dynamically_selected_prefix(
        parent,
        0x64,
        &mut info,
        child,
        crate::SpriteDynamicSpawnProgress::FloorPublished,
    );
    {
        let child = split.sprite_slot_view(child);
        assert_eq!(child.state(), 9);
        assert_eq!(child.sprite_type(), 0x64);
        assert_eq!(child.floor(), 1);
        assert_eq!(child.direction(), 0);
    }

    split.sprite_spawn_dynamically_selected_from(
        parent,
        &mut info,
        child,
        crate::SpriteDynamicSpawnProgress::FloorPublished,
    );
    split.sprite_prep_debirando_pit_after_spawn(parent, child, &info);
    atomic.sprite_module_initialize(parent);

    assert_eq!(
        split.game_state.sprites.sprite_slots, atomic.game_state.sprites.sprite_slots,
        "split execution must equal the atomic dynamic-spawn call and Fire Debirando suffix",
    );
}
