//! ZeldaState runtime tests — timing.
//! Split mechanically from the hub file; bodies unchanged.

use super::*;

#[test]
fn migrated_link_world_reads_use_semantic_views() {
    for (path, source) in [
        ("ancilla.rs", include_str!("../../ancilla.rs")),
        ("dungeon.rs", include_str!("../../dungeon.rs")),
        ("ending.rs", include_str!("../../ending.rs")),
        ("hud.rs", include_str!("../../hud.rs")),
        ("load_gfx.rs", include_str!("../../load_gfx.rs")),
        ("messaging.rs", include_str!("../../messaging.rs")),
        ("misc.rs", include_str!("../../misc.rs")),
        ("overlord.rs", include_str!("../../overlord.rs")),
        ("overworld.rs", include_str!("../../overworld.rs")),
        ("player.rs", include_str!("../../player.rs")),
        ("player_oam.rs", include_str!("../../player_oam.rs")),
        ("sprite.rs", include_str!("../../sprite.rs")),
        ("sprite_main.rs", include_str!("../../sprite_main.rs")),
        (
            "sprite_main_blind.rs",
            include_str!("../../sprite_main_blind.rs"),
        ),
        (
            "sprite_main_draw.rs",
            include_str!("../../sprite_main_draw.rs"),
        ),
        (
            "sprite_main_dungeon_npcs.rs",
            include_str!("../../sprite_main_dungeon_npcs.rs"),
        ),
        (
            "sprite_main_ganon.rs",
            include_str!("../../sprite_main_ganon.rs"),
        ),
        (
            "sprite_main_guard.rs",
            include_str!("../../sprite_main_guard.rs"),
        ),
        (
            "sprite_main_helmasaur_king.rs",
            include_str!("../../sprite_main_helmasaur_king.rs"),
        ),
        (
            "sprite_main_hinox_shop.rs",
            include_str!("../../sprite_main_hinox_shop.rs"),
        ),
        (
            "sprite_main_mothula.rs",
            include_str!("../../sprite_main_mothula.rs"),
        ),
        (
            "sprite_main_npcs.rs",
            include_str!("../../sprite_main_npcs.rs"),
        ),
        (
            "sprite_main_prep.rs",
            include_str!("../../sprite_main_prep.rs"),
        ),
        (
            "sprite_main_small_bosses.rs",
            include_str!("../../sprite_main_small_bosses.rs"),
        ),
        (
            "sprite_main_world.rs",
            include_str!("../../sprite_main_world.rs"),
        ),
        ("tile_detect.rs", include_str!("../../tile_detect.rs")),
    ] {
        for needle in [
            concat!("self.", "raw_", "ram().word_at(LINK_X_COORD)"),
            concat!("self.", "raw_", "ram().word_at(LINK_Y_COORD)"),
            concat!("self.", "raw_", "ram().word_at(LINK_Z_COORD)"),
            concat!("self.", "raw_", "ram().word_at(OVERWORLD_SCREEN_INDEX)"),
            concat!("self.", "raw_", "ram().word_at(DUNGEON_ROOM_INDEX)"),
        ] {
            assert!(
                !source.contains(needle),
                "{path} should use typed semantic views for {needle}"
            );
        }
    }
}

#[test]
fn migrated_player_coordinate_writes_use_semantic_views() {
    for (path, source) in [
        ("ancilla.rs", include_str!("../../ancilla.rs")),
        ("dungeon.rs", include_str!("../../dungeon.rs")),
        ("overworld.rs", include_str!("../../overworld.rs")),
        ("player.rs", include_str!("../../player.rs")),
        ("player_oam.rs", include_str!("../../player_oam.rs")),
        ("sprite.rs", include_str!("../../sprite.rs")),
        (
            "sprite_main_draw.rs",
            include_str!("../../sprite_main_draw.rs"),
        ),
        (
            "sprite_main_dungeon_npcs.rs",
            include_str!("../../sprite_main_dungeon_npcs.rs"),
        ),
        (
            "sprite_main_mothula.rs",
            include_str!("../../sprite_main_mothula.rs"),
        ),
        (
            "sprite_main_world.rs",
            include_str!("../../sprite_main_world.rs"),
        ),
        ("zelda_rtl.rs", include_str!("../../zelda_rtl.rs")),
    ] {
        for needle in [
            concat!("self.", "raw_", "ram_mut().set_word_at(", "LINK_X_COORD,"),
            concat!("self.", "raw_", "ram_mut().set_word_at(", "LINK_Y_COORD,"),
            concat!("self.", "raw_", "ram_mut().set_word_at(", "LINK_Z_COORD,"),
        ] {
            assert!(
                !source.contains(needle),
                "{path} should use typed semantic views for {needle}"
            );
        }
    }
}

#[test]
fn emu_callback_setup_syncs_whole_state_and_regions() {
    let mut state = ZeldaState::new();
    state.zelda_setup_emu_callbacks(Some(vec![0; 16]), None, Some(test_sync_all));

    state.emu_synchronize_whole_state();
    assert_eq!(state.ram[0x42], 1);

    state.ram[0x1234..0x1238].copy_from_slice(&[1, 2, 3, 4]);
    state.emu_sync_memory_region(0x1234, 4);
    let emu = state.emu_memory_ptr.as_ref().unwrap();
    assert_eq!(&emu[0x1234..0x1238], &[1, 2, 3, 4]);
}

#[test]
fn original_timing_resume_sidecar_preserves_cross_host_semantics_without_changing_zeldastate_bytes()
{
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_last_oracle_host_call = Some(15_999);
    let baseline = bincode::serialize(&state).unwrap();

    state.original_timing_dungeon_exit_spotlight_entry_return_pending = true;
    state.original_timing_pre_dungeon_return_pending =
        Some(crate::MainLoopProgress::CallStackContinued);
    state.item_receipt_completion_live_link_dma_host = Some(15_999);
    assert_eq!(
        bincode::serialize(&state).unwrap(),
        baseline,
        "backend-neutral timing continuation must remain outside positional ZeldaState bytes",
    );

    let checkpoint = state.capture_original_timing_resume_checkpoint().unwrap();
    let checkpoint = bincode::deserialize(
        &bincode::serialize(&checkpoint).expect("resume sidecar must serialize"),
    )
    .unwrap();
    let mut restored: ZeldaState = bincode::deserialize(&baseline).unwrap();
    restored.restore_live_rom_timing_after_checkpoint();
    restored
        .restore_original_timing_resume_checkpoint(checkpoint)
        .unwrap();

    assert_eq!(restored.original_timing_last_oracle_host_call, Some(15_999));
    assert!(!restored.original_timing_nmi_publication_pending);
    assert_eq!(restored.original_timing_pending_nmi_update_gate, None);
    assert!(restored.original_timing_dungeon_exit_spotlight_entry_return_pending);
    assert_eq!(
        restored.original_timing_pre_dungeon_return_pending,
        Some(crate::MainLoopProgress::CallStackContinued),
    );
    assert_eq!(
        restored.item_receipt_completion_live_link_dma_host,
        Some(15_999),
    );
    assert_eq!(
        restored.original_timing_owner(),
        OriginalTimingOwner::Unavailable(OriginalTimingUnavailableReason::CheckpointRestore),
        "the backend must not become live until its next typed receipt is installed",
    );
}

#[test]
fn original_timing_resume_sidecar_rejects_unbound_continuation_before_mutation() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    let before = state.clone();
    let malformed = crate::OriginalTimingResumeCheckpoint {
        schema: crate::OriginalTimingResumeCheckpoint::SCHEMA,
        last_consumed_host_call: None,
        nmi_publication_pending: true,
        pending_nmi_update_gate: Some(NmiUpdateGate::Open),
        dungeon_exit_spotlight_entry_return_pending: false,
        pre_dungeon_return_pending: None,
        item_receipt_live_link_dma_host: None,
    };
    assert_eq!(
        state.restore_original_timing_resume_checkpoint(malformed),
        Err(OriginalTimingResumeCheckpointError::MissingHostCallProvenance),
    );
    assert_eq!(
        state.original_timing_nmi_publication_pending,
        before.original_timing_nmi_publication_pending,
    );
    assert_eq!(
        state.original_timing_last_oracle_host_call,
        before.original_timing_last_oracle_host_call,
    );
}

#[test]
fn interactive_cleanup_keeps_current_slot_live_until_pickup_test() {
    for slot in 0..6 {
        let mut state = ZeldaState::new();
        for i in 0..6 {
            state.ancilla_slot_view_mut(i).set_ancilla_type(0x2c);
        }
        let mut atomic = state.clone();
        atomic.ancilla_terminate_select_interactives(0);
        state.ancilla_interactive_cleanup_before_pickup(slot);
        for i in 0..6 {
            assert_eq!(
                state.ancilla_slot_view(i).ancilla_type(),
                if i > usize::from(slot) { 0 } else { 0x2c }
            );
        }
        state.ancilla_interactive_cleanup_after_pickup(slot);
        assert_eq!(state.game_state, atomic.game_state);
    }
}

#[test]
fn interactive_cleanup_type_clear_preserves_completed_pickup_test() {
    for slot in 0..6 {
        let mut state = ZeldaState::new();
        for i in 0..6 {
            state.ancilla_slot_view_mut(i).set_ancilla_type(0x2c);
        }
        state
            .follower_link_state_mut()
            .set_ancilla_pickup_flag(slot + 1);
        let mut atomic = state.clone();
        atomic.ancilla_terminate_select_interactives(0);
        state.ancilla_interactive_cleanup_before_type_clear(slot);
        assert_eq!(
            state.game_state.player.follower_link.ancilla_pickup_flag(),
            0
        );
        assert_eq!(
            state.ancilla_slot_view(usize::from(slot)).ancilla_type(),
            0x2c
        );
        state.ancilla_interactive_cleanup_at_type_clear(slot);
        assert_eq!(state.game_state, atomic.game_state);
        assert_eq!(state.ram, atomic.ram);
    }
}

#[test]
fn cached_restore_publishes_stores_between_host_return_and_acceptance() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.ram[0x0de2] = 1;
    state.sync_native_game_state_from_ram();
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
        vec![cached_sprite_progress_receipt(
            crate::CachedSpriteExecutionProgress::Restoring {
                slot: 2,
                live_fields: 11,
            },
            OriginalTimingBoundary::NmiAccepted,
        )],
    ));
    state.latch_nmi_update();
    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    state
        .complete_original_timing_nmi_handler_for_active_scanout(
            OriginalTimingNmiHandlerCompletionOwner::AcceptedInThisSequence,
            0,
            None,
        )
        .assert_no_unclaimed_dialogue_text_dma();
    assert_eq!(state.ram[0x0de2], 0);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonCachedSpriteMain {
            boundary: CachedSpriteCpuInterruption::Restoring {
                slot: 2,
                live_fields: 11
            },
            ..
        })
    ));
    assert_eq!(
        state
            .game_execution_scheduler
            .scheduled_work_slices_remaining(),
        Some(1)
    );
}

#[test]
fn run4788_checkpoint_then_run4789_terminal_build_completes_cpu_and_suffix_once() {
    let mut state = run4789_terminal_spotlight_build_state();
    let epoch_before = state.display_snapshot_epoch;
    let active_radius_before = read_le_u16(
        &state
            .display_snapshot
            .as_ref()
            .expect("run4788 lost its receptive Held snapshot")
            .ram,
        SPOTLIGHT_WINDOW_RADIUS,
    );
    assert_eq!(active_radius_before, 112);

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 105);
    assert_eq!(state.display_snapshot_epoch, epoch_before + 1);
    let snapshot = state
        .display_snapshot
        .as_ref()
        .expect("run4789 lost its retained active display snapshot");
    assert_eq!(
        read_le_u16(&snapshot.ram, SPOTLIGHT_WINDOW_RADIUS),
        112,
        "RetainPublished must keep run4788's active table generation",
    );
    assert!(!snapshot.accepts_nmi_dma_receipts);
    assert!(state.deferred_display_snapshot.is_none());
    assert!(matches!(
        snapshot.hdma_table_generation,
        DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout {
            live_tail_start: 224,
            ..
        },
    ));
    assert_eq!(state.next_display_obj_scanout_generation, None);
    assert_eq!(state.pending_main_loop_common_suffix, None);
    assert!(state.main_loop_sprite_preparation_completed);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn run4790_checkpoint_then_run4791_terminal_build_carries_its_trailing_open_once() {
    let mut state = run4791_terminal_spotlight_build_state();
    let epoch_before = state.display_snapshot_epoch;
    let active_radius_before = read_le_u16(
        &state
            .display_snapshot
            .as_ref()
            .expect("run4790 lost its active display generation")
            .ram,
        SPOTLIGHT_WINDOW_RADIUS,
    );
    assert_eq!(active_radius_before, 105);

    // The same-host/no-trailing member of the typed 2x2 grammar proves the
    // retained run4790 generation receives its table-tail projection before
    // a trailing acceptance can replace it.
    let mut no_trailing = state.clone();
    no_trailing
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .remove(4);
    no_trailing.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    no_trailing.original_timing_expected_nmi_ppu_register_operands = vec![None];
    no_trailing.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(no_trailing.display_snapshot_epoch, epoch_before + 2);
    let retained = no_trailing
        .display_snapshot
        .as_ref()
        .expect("same-host Build lost its retained active generation");
    assert_eq!(read_le_u16(&retained.ram, SPOTLIGHT_WINDOW_RADIUS), 105,);
    assert!(!retained.accepts_nmi_dma_receipts);
    let DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout {
        live_tail_start, ..
    } = retained.hdma_table_generation
    else {
        panic!("run4791 did not project the retained table tail")
    };
    assert_eq!(live_tail_start, 221);

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 98);
    assert_eq!(
        state.display_snapshot_epoch,
        epoch_before + 3,
        "run4791 owns one Held acceptance, one typed RetainPublished completion, and one trailing Open acceptance",
    );
    let carried = state
        .display_snapshot
        .as_ref()
        .expect("run4791 lost its receptive trailing-Open generation");
    assert_eq!(
        read_le_u16(&carried.ram, SPOTLIGHT_WINDOW_RADIUS),
        98,
        "the trailing Open acceptance must capture the completed live Build",
    );
    assert!(carried.accepts_nmi_dma_receipts);
    assert!(state.deferred_display_snapshot.is_none());
    assert_eq!(state.pending_main_loop_common_suffix, None);
    assert!(state.main_loop_sprite_preparation_completed);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::Open),
    );
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert!(state.original_timing_semantic_receipts.is_none());
    assert_eq!(state.next_display_obj_scanout_generation, None);
}

#[test]
fn run4794_projection_interrupt_then_run4795_flagged_terminal_build_completes_cpu_and_suffix_once()
{
    let mut state = run4795_flagged_terminal_spotlight_build_state();
    let epoch_before = state.display_snapshot_epoch;

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 84);
    assert_eq!(
        state.display_snapshot_epoch,
        epoch_before + 1,
        "run4795 owns exactly its carried Held handler's publication",
    );
    let retained = state
        .display_snapshot
        .as_ref()
        .expect("run4795 lost its retained active display snapshot");
    assert_eq!(
        read_le_u16(&retained.ram, SPOTLIGHT_WINDOW_RADIUS),
        91,
        "RetainPublished must keep run4794's active table generation",
    );
    assert!(!retained.accepts_nmi_dma_receipts);
    assert!(matches!(
        retained.hdma_table_generation,
        DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout { .. },
    ));
    assert!(state.deferred_display_snapshot.is_none());
    assert_eq!(state.pending_main_loop_common_suffix, None);
    assert!(state.main_loop_sprite_preparation_completed);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn run4797_flagged_terminal_build_carries_its_trailing_open_once() {
    let mut state = run4797_flagged_trailing_open_spotlight_build_state();
    let epoch_before = state.display_snapshot_epoch;

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 77);
    assert_eq!(
        state.display_snapshot_epoch,
        epoch_before + 2,
        "run4797 owns its carried Held publication and one trailing Open acceptance",
    );
    let carried = state
        .display_snapshot
        .as_ref()
        .expect("run4797 lost its receptive trailing-Open generation");
    assert_eq!(
        read_le_u16(&carried.ram, SPOTLIGHT_WINDOW_RADIUS),
        77,
        "the trailing Open acceptance must capture the completed live Build",
    );
    assert!(carried.accepts_nmi_dma_receipts);
    assert!(state.deferred_display_snapshot.is_none());
    assert_eq!(state.pending_main_loop_common_suffix, None);
    assert!(state.main_loop_sprite_preparation_completed);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::Open),
    );
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn run4799_terminal_build_consumes_its_projection_recheckpoint_once() {
    let mut state = run4799_recheckpointed_terminal_spotlight_build_state();
    let epoch_before = state.display_snapshot_epoch;

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 70);
    assert_eq!(
        state.display_snapshot_epoch,
        epoch_before + 3,
        "run4799 owns its same-host Held publication, its typed RetainPublished completion, and one trailing Open acceptance",
    );
    assert_eq!(state.pending_main_loop_common_suffix, None);
    assert!(state.main_loop_sprite_preparation_completed);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::Open),
    );
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert!(
        state.original_timing_semantic_receipts.is_none(),
        "run4799 must consume its re-checkpoint claim with its terminal return",
    );
}

#[test]
fn run4805_carried_suffix_only_terminal_completes_held_handler_and_suffix_once() {
    let mut state = run4805_carried_suffix_only_spotlight_state();
    let epoch_before = state.display_snapshot_epoch;

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 49);
    assert_eq!(
        state.display_snapshot_epoch, epoch_before,
        "run4805's carried handler published at run4804's acceptance; its suffix-only completion projects into the retained generation",
    );
    assert_eq!(state.pending_main_loop_common_suffix, None);
    assert!(state.main_loop_sprite_preparation_completed);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn run4807_suffix_only_terminal_carries_its_trailing_open_once() {
    let mut state = run4807_trailing_open_suffix_only_spotlight_state();
    let epoch_before = state.display_snapshot_epoch;

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 42);
    assert_eq!(
        state.display_snapshot_epoch,
        epoch_before + 2,
        "run4807 owns its same-host Held publication and one trailing Open acceptance",
    );
    assert_eq!(state.pending_main_loop_common_suffix, None);
    assert!(state.main_loop_sprite_preparation_completed);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::Open),
    );
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn run4819_final_suffix_only_terminal_returns_without_module_qualified_token() {
    let mut state = run4819_tokenless_final_spotlight_state();
    let epoch_before = state.display_snapshot_epoch;

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.pending_main_loop_common_suffix, None);
    assert!(state.main_loop_sprite_preparation_completed);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::Open),
    );
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert!(state.original_timing_semantic_receipts.is_none());
    assert_eq!(
        state.display_snapshot_epoch,
        epoch_before + 1,
        "run4819's carried handler published at run4818's acceptance; only its trailing Open acceptance publishes here",
    );
    assert_ne!(
        (
            state.game_state.frame.main_module,
            state.game_state.frame.submodule,
        ),
        (0x0f, 1),
        "the final spotlight caller returns outside Module0F",
    );
}

#[test]
fn run4799_and_run4819_terminal_preflight_is_failure_atomic() {
    fn assert_rejected(label: &str, mut state: ZeldaState) {
        let receipts_before = state.original_timing_semantic_receipts.clone();
        let scheduler_before = state.game_execution_scheduler;
        let suffix_before = state.pending_main_loop_common_suffix;
        let game_before = state.game_state.clone();
        let ram_before = state.ram.clone();
        let sidecars_before = (
            state.original_timing_nmi_publication_pending,
            state.original_timing_pending_nmi_update_gate,
            state.original_timing_expected_nmi_update_gates.clone(),
            state.original_timing_sprite_main_return_claims_remaining,
            state.main_loop_sprite_preparation_completed,
            state.display_snapshot_epoch,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal(0, crate::RUN_MAIN);
        }));
        assert!(result.is_err(), "{label} unexpectedly reached execution");
        assert_eq!(
            state.original_timing_semantic_receipts, receipts_before,
            "{label}"
        );
        assert_eq!(state.game_execution_scheduler, scheduler_before, "{label}");
        assert_eq!(
            state.pending_main_loop_common_suffix, suffix_before,
            "{label}"
        );
        assert_eq!(state.game_state, game_before, "{label}");
        assert_eq!(state.ram, ram_before, "{label}");
        assert_eq!(
            (
                state.original_timing_nmi_publication_pending,
                state.original_timing_pending_nmi_update_gate,
                state.original_timing_expected_nmi_update_gates.clone(),
                state.original_timing_sprite_main_return_claims_remaining,
                state.main_loop_sprite_preparation_completed,
                state.display_snapshot_epoch,
            ),
            sidecars_before,
            "{label}"
        );
    }

    // A re-checkpoint may refine the scheduled ProjectionCopy forward (the
    // exact interrupting acceptance supersedes the estimate, route host
    // 50632) but can never rewind it.
    let mut wrong_copied_words = run4799_recheckpointed_terminal_spotlight_build_state();
    wrong_copied_words
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic[1] = run48xx_spotlight_projection_claim(116, OriginalTimingBoundary::NmiAccepted);
    assert_rejected(
        "re-checkpoint rewinding the copy position",
        wrong_copied_words,
    );

    // The re-checkpoint is only published at its interrupting acceptance.
    let mut wrong_boundary = run4799_recheckpointed_terminal_spotlight_build_state();
    wrong_boundary
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic[1] = run48xx_spotlight_projection_claim(117, OriginalTimingBoundary::HostReturn);
    assert_rejected("re-checkpoint at a HostReturn boundary", wrong_boundary);

    // A suffix-only terminal carrying an acceptance-boundary re-checkpoint is
    // no longer a rejected shape: route host 40982 proved the interrupting
    // acceptance can restate the separately-suspended recurring build's
    // state, which the plan validates and consumes as corroboration.

    // A caller still inside Module0F must publish its qualified return token.
    let mut missing_token = run4807_trailing_open_suffix_only_spotlight_state();
    missing_token
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .pop();
    assert_rejected("Module0F return without its qualified token", missing_token);

    // A caller that already left Module0F cannot also publish the token.
    let mut stale_token = run4819_tokenless_final_spotlight_state();
    stale_token
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .push(OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait);
    assert_rejected("token after the module body left Module0F", stale_token);

    // The sprite-preparation estimate stays fail-closed for suffix-only work.
    let mut flagged_suffix_only = run4805_carried_suffix_only_spotlight_state();
    let Some(GameWorkContinuation::FinishSpotlightIteration { iteration }) =
        flagged_suffix_only.game_execution_scheduler.current_work()
    else {
        panic!("run4805 fixture lost its suffix-only owner")
    };
    flagged_suffix_only.game_execution_scheduler.finish_work();
    flagged_suffix_only.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishSpotlightIteration {
            iteration: iteration.with_main_loop_sprite_preparation_before_second_nmi(),
        },
        1,
    );
    assert_rejected(
        "sprite-preparation estimate on suffix-only work",
        flagged_suffix_only,
    );
}

#[test]
fn main_loop_common_suffix_completion_installation_is_failure_atomic() {
    let mut missing_progress = ZeldaState::new();
    missing_progress.set_rom_startup_timing(true);
    assert_eq!(
        missing_progress.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted],
        ),),
        Err(OriginalTimingReceiptInstallError::InvalidMainLoopIterationReturn),
    );
    assert!(missing_progress.original_timing_semantic_receipts.is_none());
    assert_eq!(
        missing_progress.last_consumed_original_timing_host_call(),
        None
    );

    let mut continued_without_owner = ZeldaState::new();
    continued_without_owner.set_rom_startup_timing(true);
    assert_eq!(
        continued_without_owner.install_original_timing_host_receipts(
            OriginalTimingHostReceipts::new(
                0,
                0,
                vec![
                    OriginalTimingSemanticReceipt::MainLoopProgress(
                        crate::MainLoopProgress::CallStackContinued,
                    ),
                    OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                ],
            ),
        ),
        Err(OriginalTimingReceiptInstallError::InvalidMainLoopIterationReturn),
    );
    assert!(continued_without_owner
        .original_timing_semantic_receipts
        .is_none());
    assert!(continued_without_owner
        .pending_main_loop_common_suffix
        .is_none());

    let mut completion_before_progress = ZeldaState::new();
    completion_before_progress.set_rom_startup_timing(true);
    assert_eq!(
        completion_before_progress.install_original_timing_host_receipts(
            OriginalTimingHostReceipts::new(
                0,
                0,
                vec![
                    OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                    OriginalTimingSemanticReceipt::MainLoopProgress(
                        crate::MainLoopProgress::IterationStarted,
                    ),
                ],
            ),
        ),
        Err(OriginalTimingReceiptInstallError::InvalidMainLoopIterationReturn),
    );
    assert!(completion_before_progress
        .original_timing_semantic_receipts
        .is_none());
    assert_eq!(
        completion_before_progress.last_consumed_original_timing_host_call(),
        None
    );

    let mut fresh_with_stale_owner = ZeldaState::new();
    fresh_with_stale_owner.set_rom_startup_timing(true);
    fresh_with_stale_owner.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    assert_eq!(
        fresh_with_stale_owner.install_original_timing_host_receipts(
            OriginalTimingHostReceipts::new(
                0,
                0,
                vec![
                    OriginalTimingSemanticReceipt::MainLoopProgress(
                        crate::MainLoopProgress::IterationStarted,
                    ),
                    OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                ],
            ),
        ),
        Err(OriginalTimingReceiptInstallError::InvalidMainLoopIterationReturn),
    );
    assert_eq!(
        fresh_with_stale_owner.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert!(fresh_with_stale_owner
        .original_timing_semantic_receipts
        .is_none());

    let mut interrupted_completion = ZeldaState::new();
    interrupted_completion.set_rom_startup_timing(true);
    assert_eq!(
        interrupted_completion.install_original_timing_host_receipts(
            OriginalTimingHostReceipts::new(
                0,
                0,
                vec![
                    OriginalTimingSemanticReceipt::MainLoopProgress(
                        crate::MainLoopProgress::IterationStarted,
                    ),
                    OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                    OriginalTimingSemanticReceipt::MainLoopInterrupted(
                        crate::MainLoopInterruption::LinkOam,
                    ),
                ],
            ),
        ),
        Err(OriginalTimingReceiptInstallError::InvalidMainLoopIterationReturn),
    );
    assert!(interrupted_completion
        .original_timing_semantic_receipts
        .is_none());

    let mut unsupported_post_completion_phases = ZeldaState::new();
    unsupported_post_completion_phases.set_rom_startup_timing(true);
    unsupported_post_completion_phases.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    assert_eq!(
        unsupported_post_completion_phases.install_original_timing_host_receipts(
            OriginalTimingHostReceipts::new(
                0,
                0,
                vec![
                    OriginalTimingSemanticReceipt::MainLoopProgress(
                        crate::MainLoopProgress::CallStackContinued,
                    ),
                    OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                    OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                    OriginalTimingSemanticReceipt::JoypadPublication(crate::JoypadPublication {
                        high: 0,
                        low: 0,
                        high_filtered: 0,
                        low_filtered: 0,
                    }),
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                    OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                    OriginalTimingSemanticReceipt::JoypadPublication(crate::JoypadPublication {
                        high: 0,
                        low: 0,
                        high_filtered: 0,
                        low_filtered: 0,
                    }),
                ],
            ),
        ),
        Err(OriginalTimingReceiptInstallError::InvalidMainLoopIterationReturn),
    );
    assert_eq!(
        unsupported_post_completion_phases.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert!(unsupported_post_completion_phases
        .original_timing_semantic_receipts
        .is_none());
    assert_eq!(
        unsupported_post_completion_phases.last_consumed_original_timing_host_call(),
        None,
    );
}

#[test]
fn live_throwable_scenery_state_clear_applies_the_partial_slot_prefix() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(7);
    state.set_submodule(0);
    state.sprite_slot_view_mut(6).set_state(6);
    state.sprite_slot_view_mut(6).set_sprite_type(0xec);
    state.sprite_slot_view_mut(6).set_delay_main(31);
    state.sprite_slot_view_mut(6).set_c(1);
    state.sprite_slot_view_mut(6).set_x(0x0ef8);
    state.sprite_slot_view_mut(6).set_y(0x0518);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        164_140,
        0,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            crate::SpriteMainProgress::AfterThrowableSceneryStateClear(6),
        )],
    ));

    state.run_module07_sprite_main_caller(DungeonSpriteMainReturn {
        link_oam: None,
        bg2_x: 0,
        bg2_y: 0,
        bg1_x: 0,
        bg1_y: 0,
    });

    assert_eq!(state.sprite_slot_view(6).state(), 0);
    assert_eq!(state.sprite_slot_view(6).delay_main(), 30);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterThrowableSceneryStateClear(6),
            caller: SpriteMainCpuCaller::DungeonModule07Live {
                boundary: crate::OriginalTimingBoundary::HostReturn,
            },
        }),
    );
    assert!(state.active_dungeon_sprite_main_return.is_some());
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn live_single_small_draw_position_applies_prefix_and_resumes_without_replaying_timers() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(7);
    state.set_submodule(0);
    state.sprite_slot_view_mut(15).set_state(9);
    state.sprite_slot_view_mut(15).set_sprite_type(0x23);
    state.sprite_slot_view_mut(15).set_delay_main(0x7f);
    state.sprite_slot_view_mut(15).set_delay_aux1(2);
    state.sprite_slot_view_mut(15).set_c(1);
    state.sprite_slot_view_mut(15).set_graphics(3);
    state.sprite_slot_view_mut(15).set_flags2(4);
    state.sprite_slot_view_mut(15).set_flags3(0x33);
    state.sprite_slot_view_mut(15).set_x(0x0120);
    state.sprite_slot_view_mut(15).set_y(0x0080);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        660_588,
        0,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            crate::SpriteMainProgress::AfterSingleSmallDrawPosition(15),
        )],
    ));

    state.run_module07_sprite_main_caller(DungeonSpriteMainReturn {
        link_oam: None,
        bg2_x: 0,
        bg2_y: 0,
        bg1_x: 0,
        bg1_y: 0,
    });

    assert_eq!(state.sprite_slot_view(15).delay_main(), 0x7e);
    assert_eq!(state.sprite_slot_view(15).delay_aux1(), 1);
    let boundary = match state.game_execution_scheduler.current_work() {
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary:
                SpriteMainCpuBoundary::AfterSingleSmallDrawPosition {
                    slot: 15,
                    continuation: Some(continuation),
                },
            caller:
                SpriteMainCpuCaller::DungeonModule07Live {
                    boundary: crate::OriginalTimingBoundary::HostReturn,
                },
        }) => SpriteMainCpuBoundary::AfterSingleSmallDrawPosition {
            slot: 15,
            continuation: Some(continuation),
        },
        other => panic!("unexpected single-small draw continuation: {other:?}"),
    };

    state.complete_sprite_main_after_cpu_boundary(boundary);

    assert_eq!(state.sprite_slot_view(15).delay_main(), 0x7e);
    assert_eq!(state.sprite_slot_view(15).delay_aux1(), 1);
}

#[test]
fn live_timer_decrement_boundary_finishes_priority_and_dispatch_without_replaying_countdowns() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(7);
    state.set_submodule(0);
    // State 2 has no native countdown write of its own while the timer is
    // nonzero, so the assertion below isolates replay of Sprite_TimersAndOam.
    state.sprite_slot_view_mut(0).set_state(2);
    state.sprite_slot_view_mut(0).set_delay_main(0xa0);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        676_340,
        0,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            crate::SpriteMainProgress::AfterTimerDecrements(0),
        )],
    ));

    state.run_module07_sprite_main_caller(DungeonSpriteMainReturn {
        link_oam: None,
        bg2_x: 0,
        bg2_y: 0,
        bg1_x: 0,
        bg1_y: 0,
    });

    assert_eq!(state.sprite_slot_view(0).delay_main(), 0x9f);
    let boundary = match state.game_execution_scheduler.current_work() {
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary:
                SpriteMainCpuBoundary::AfterTimerDecrements {
                    slot: 0,
                    state: Some(2),
                },
            caller:
                SpriteMainCpuCaller::DungeonModule07Live {
                    boundary: crate::OriginalTimingBoundary::HostReturn,
                },
        }) => SpriteMainCpuBoundary::AfterTimerDecrements {
            slot: 0,
            state: Some(2),
        },
        other => panic!("unexpected timer decrement continuation: {other:?}"),
    };

    state.complete_sprite_main_after_cpu_boundary(boundary);

    assert_eq!(state.sprite_slot_view(0).delay_main(), 0x9f);
}

#[test]
fn live_hit_timer_checkpoint_leaves_aux4_for_the_resumed_call() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(7);
    state.set_submodule(0);
    // State 2 has no native countdown write of its own while the timer is
    // nonzero, so the assertion below isolates replay of Sprite_TimersAndOam.
    state.sprite_slot_view_mut(0).set_state(2);
    state.sprite_slot_view_mut(0).set_delay_main(0xa0);
    state.sprite_slot_view_mut(0).set_hit_timer(3);
    state.sprite_slot_view_mut(0).set_delay_aux4(3);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        676_340,
        0,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            crate::SpriteMainProgress::AfterHitTimer(0),
        )],
    ));

    state.run_module07_sprite_main_caller(DungeonSpriteMainReturn {
        link_oam: None,
        bg2_x: 0,
        bg2_y: 0,
        bg1_x: 0,
        bg1_y: 0,
    });

    assert_eq!(state.sprite_slot_view(0).delay_main(), 0x9f);
    assert_eq!(state.sprite_slot_view(0).hit_timer(), 2);
    assert_eq!(state.sprite_slot_view(0).delay_aux4(), 3);
    let boundary = match state.game_execution_scheduler.current_work() {
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary:
                SpriteMainCpuBoundary::AfterHitTimer {
                    slot: 0,
                    state: Some(2),
                },
            caller:
                SpriteMainCpuCaller::DungeonModule07Live {
                    boundary: crate::OriginalTimingBoundary::HostReturn,
                },
        }) => SpriteMainCpuBoundary::AfterHitTimer {
            slot: 0,
            state: Some(2),
        },
        other => panic!("unexpected timer decrement continuation: {other:?}"),
    };

    state.complete_sprite_main_after_cpu_boundary(boundary);
    assert_eq!(state.sprite_slot_view(0).hit_timer(), 2);
    assert_eq!(state.sprite_slot_view(0).delay_aux4(), 2);

    assert_eq!(state.sprite_slot_view(0).delay_main(), 0x9f);
}

#[test]
fn active_cucco_x_checkpoint_resumes_without_replaying_movement() {
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
        state.sprite_system_mut().set_cur_object_index(1);
        let mut sprite = state.sprite_slot_view_mut(1);
        sprite.set_state(9);
        sprite.set_sprite_type(0x0b);
        sprite.set_ai_state(2);
        sprite.set_c(1);
        sprite.set_x(0x01a1);
        sprite.set_y(0x094b);
        sprite.set_x_velocity(0x20);
        sprite.set_y_velocity(7);
        sprite.set_x_subpixel(0);
        sprite.set_y_subpixel(0x20);
        sprite.set_subtype2(0xfc);
        sprite.set_graphics(1);
        state
    }

    let mut atomic = configured_state();
    atomic.sprite_0_b_cucco(1);

    let mut resumed = configured_state();
    let original_y = resumed.sprite_slot_view(1).y();
    let original_y_subpixel = resumed.sprite_slot_view(1).y_subpixel();
    resumed.sprite_main_cpu_boundary = Some(SpriteMainCpuBoundary::AfterActiveCuccoX {
        slot: 1,
        helper_ordinal: 0,
    });
    resumed.sprite_0_b_cucco(1);

    assert_eq!(resumed.sprite_slot_view(1).x(), 0x01a3);
    assert_eq!(resumed.sprite_slot_view(1).y(), original_y);
    assert_eq!(
        resumed.sprite_slot_view(1).y_subpixel(),
        original_y_subpixel
    );
    assert_eq!(
        resumed.sprite_main_cpu_boundary,
        Some(SpriteMainCpuBoundary::AfterActiveCuccoX {
            slot: 1,
            helper_ordinal: 0,
        }),
    );

    resumed.complete_active_cucco_after_x(1, 0);
    resumed.sprite_main_cpu_boundary = None;

    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(resumed.game_state, atomic.game_state);
}

#[test]
fn active_cucco_y_subpixel_checkpoint_resumes_without_replaying_movement() {
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
        sprite.set_c(1);
        sprite.set_x(0x012e);
        sprite.set_y(0x094b);
        sprite.set_x_velocity(0x20);
        sprite.set_y_velocity(7);
        sprite.set_y_subpixel(0x20);
        sprite.set_subtype2(0xfc);
        sprite.set_graphics(1);
        state
    }

    let mut atomic = configured_state();
    atomic.sprite_0_b_cucco(5);

    let mut resumed = configured_state();
    let original_y_low = resumed.sprite_slot_view(5).y_low();
    let original_y_high = resumed.sprite_slot_view(5).y_high();
    resumed.sprite_main_cpu_boundary = Some(SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
        slot: 5,
        helper_ordinal: 0,
        y_low: None,
        y_high: None,
    });
    resumed.sprite_0_b_cucco(5);

    assert_eq!(resumed.sprite_slot_view(5).x(), 0x0130);
    assert_eq!(resumed.sprite_slot_view(5).y_subpixel(), 0x90);
    assert_eq!(resumed.sprite_slot_view(5).y_low(), original_y_low);
    assert_eq!(resumed.sprite_slot_view(5).y_high(), original_y_high);
    let Some(SpriteMainCpuBoundary::AfterActiveCuccoYSubpixel {
        slot: 5,
        helper_ordinal: 0,
        y_low: Some(y_low),
        y_high: Some(y_high),
    }) = resumed.sprite_main_cpu_boundary
    else {
        panic!("native Cucco movement did not bind the pending Y coordinate bytes")
    };

    resumed.complete_active_cucco_after_y_subpixel(5, 0, y_low, y_high);
    resumed.sprite_main_cpu_boundary = None;

    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(resumed.game_state, atomic.game_state);
}

#[test]
fn master_sword_light_beam_move_xy_checkpoints_resume_without_replaying_stores() {
    fn configured_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.set_main_module(9);
        state.game_state.frame.frame_counter = 1;
        state.oam_reset_region_bases();
        state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
        state
            .oam_state_mut()
            .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
        state.sprite_system_mut().set_cur_object_index(2);
        let mut sprite = state.sprite_slot_view_mut(2);
        sprite.set_state(9);
        sprite.set_sprite_type(0x62);
        sprite.set_subtype2(2);
        sprite.set_a(1);
        sprite.set_b(3);
        sprite.set_x(0x0133);
        sprite.set_y(0x024b);
        sprite.set_x_velocity(0xd0);
        sprite.set_y_velocity(0x27);
        sprite.set_x_subpixel(0x20);
        sprite.set_y_subpixel(0x90);
        state
    }

    let checkpoints = [
        crate::SpriteMoveXYCheckpoint::BeforeMovement,
        crate::SpriteMoveXYCheckpoint::AfterXSubpixel,
        crate::SpriteMoveXYCheckpoint::AfterXLow,
        crate::SpriteMoveXYCheckpoint::AfterXHigh,
        crate::SpriteMoveXYCheckpoint::AfterYSubpixel,
        crate::SpriteMoveXYCheckpoint::AfterYLow,
        crate::SpriteMoveXYCheckpoint::AfterYHigh,
    ];
    let mut atomic = configured_state();
    atomic.sprite_master_sword_light_beam(2);

    for checkpoint in checkpoints {
        let mut resumed = configured_state();
        resumed.sprite_main_cpu_boundary =
            Some(SpriteMainCpuBoundary::MasterSwordLightBeamMovement {
                slot: 2,
                checkpoint,
                continuation: None,
            });
        resumed.sprite_master_sword_light_beam(2);
        let Some(SpriteMainCpuBoundary::MasterSwordLightBeamMovement {
            slot: 2,
            checkpoint: observed,
            continuation: Some(continuation),
        }) = resumed.sprite_main_cpu_boundary
        else {
            panic!("master-sword movement did not bind its coordinate continuation")
        };
        assert_eq!(observed, checkpoint);
        resumed.complete_master_sword_light_beam_movement(2, checkpoint, continuation);
        resumed.sprite_main_cpu_boundary = None;
        assert_eq!(resumed.ram, atomic.ram, "checkpoint {checkpoint:?}");
        assert_eq!(
            resumed.game_state, atomic.game_state,
            "checkpoint {checkpoint:?}"
        );
    }
}

#[test]
fn master_sword_replacement_spawn_resumes_after_the_exact_helper_prefix() {
    fn configured_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.set_main_module(9);
        state.game_state.frame.frame_counter = 0;
        state.oam_reset_region_bases();
        state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
        state
            .oam_state_mut()
            .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
        for slot in 4..16 {
            let mut sprite = state.sprite_slot_view_mut(slot);
            sprite.set_state(9);
            sprite.set_sprite_type(1);
        }
        state.sprite_system_mut().set_cur_object_index(5);
        let mut sprite = state.sprite_slot_view_mut(5);
        sprite.set_state(9);
        sprite.set_sprite_type(0x62);
        sprite.set_subtype2(2);
        sprite.set_a(1);
        sprite.set_b(3);
        sprite.set_graphics(3);
        sprite.set_oam_flags(0x24);
        sprite.set_x(0x0133);
        sprite.set_y(0x024b);
        state
    }

    let progress = crate::SpriteDynamicSpawnProgress::ResetProperties {
        completed_stores: 1,
    };
    let mut atomic = configured_state();
    atomic.sprite_master_sword_light_beam(5);

    let mut resumed = configured_state();
    resumed.sprite_main_cpu_boundary = Some(SpriteMainCpuBoundary::MasterSwordLightBeamSpawn {
        slot: 5,
        spawned_slot: 3,
        progress,
    });
    resumed.sprite_master_sword_light_beam(5);
    assert_eq!(resumed.sprite_slot_view(3).sprite_type(), 0x62);
    assert_eq!(resumed.sprite_slot_view(3).state(), 9);

    let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
    resumed.sprite_spawn_dynamically_selected_from(5, &mut info, 3, progress);
    resumed.master_sword_finish_replacement_light_beam_spawn(5, 3, info.r0_x, info.r2_y);
    let new_b = resumed.sprite_slot_view(5).b().wrapping_sub(1);
    resumed.sprite_slot_view_mut(5).set_b(new_b);
    if new_b == 0 {
        resumed.sprite_slot_view_mut(5).set_state(0);
    }
    resumed.sprite_main_cpu_boundary = None;

    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(resumed.game_state, atomic.game_state);
}

#[test]
fn live_auxiliary_graphics_does_not_publish_on_a_bare_continued_host() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(1);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    state.capture_display_snapshot();
    state.dungeon_room_load_cpu_schedule = Some(DungeonRoomLoadCpuSchedule::default());
    let work = GameWorkContinuation::FinishDungeonSupertileTransition {
        work: DungeonSupertileTransitionWork::AuxiliarySpriteGraphics,
    };
    state.game_execution_scheduler.schedule_work(work, 1);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        711_826,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
        ],
    ));

    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert_eq!(state.game_execution_scheduler.current_work(), Some(work));
    assert_eq!(
        state
            .game_execution_scheduler
            .scheduled_work_slices_remaining(),
        Some(0),
    );
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn host_close_captures_and_carries_one_exact_terminal_acceptance() {
    for gate in [NmiUpdateGate::Open, NmiUpdateGate::LatchHeld] {
        let mut state = ZeldaState::new();
        state.set_rom_startup_timing(true);
        match gate {
            NmiUpdateGate::Open => state.clear_nmi_update_latch(),
            NmiUpdateGate::LatchHeld => state.latch_nmi_update(),
        }
        state
            .install_original_timing_host_receipts(
                OriginalTimingHostReceipts::new(
                    0,
                    0,
                    vec![OriginalTimingSemanticReceipt::NmiAccepted(gate)],
                )
                .with_presented_inidisp(crate::PresentedInidisp::new(7, 0, None).unwrap()),
            )
            .unwrap();
        let owns_dispatch = state.begin_original_timing_host_dispatch(0);
        let epoch_before = state.display_snapshot_epoch;

        state.finish_original_timing_host_dispatch(owns_dispatch);

        assert_eq!(state.display_snapshot_epoch, epoch_before + 1);
        assert!(state
            .display_snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
        assert_eq!(
            state
                .display_snapshot
                .as_ref()
                .and_then(|snapshot| snapshot.presented_inidisp_override),
            Some(crate::PresentedInidisp::new(7, 0, None).unwrap()),
        );
        assert!(state.original_timing_nmi_publication_pending);
        assert_eq!(state.original_timing_pending_nmi_update_gate, Some(gate));
        assert!(state.original_timing_semantic_receipts.is_none());
        assert!(!state.original_timing_host_dispatch_active);
    }
}

#[test]
fn host_close_accepts_an_explicitly_consumed_pending_acceptance() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.latch_nmi_update();
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::LatchHeld,
            )],
        ))
        .unwrap();
    let owns_dispatch = state.begin_original_timing_host_dispatch(0);
    assert_eq!(
        state.take_original_timing_nmi_phases(),
        [OriginalTimingNmiPhase::Accepted(NmiUpdateGate::LatchHeld)],
    );
    state.capture_and_carry_original_timing_nmi_publication_at_host_return();
    let acceptance_epoch = state.display_snapshot_epoch;

    state.finish_original_timing_host_dispatch(owns_dispatch);

    assert_eq!(state.display_snapshot_epoch, acceptance_epoch);
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn host_close_rejects_every_other_semantic_shape_before_mutation() {
    let joypad = OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
        high: 1,
        low: 2,
        high_filtered: 3,
        low_filtered: 4,
    });
    let invalid = vec![
        vec![OriginalTimingSemanticReceipt::NmiHandlerCompleted],
        vec![joypad],
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::IterationStarted,
        )],
        vec![OriginalTimingSemanticReceipt::MainLoopIterationReturnedToWait],
        vec![OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted],
        vec![OriginalTimingSemanticReceipt::MainLoopInterrupted(
            crate::MainLoopInterruption::LinkOam,
        )],
        vec![OriginalTimingSemanticReceipt::DmaPublicationCompleted { channel_mask: 1 }],
        vec![OriginalTimingSemanticReceipt::DialogueClosed],
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        ],
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            joypad,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::DmaPublicationCompleted { channel_mask: 2 },
        ],
    ];

    for semantic in invalid {
        let mut state = ZeldaState::new();
        state.set_rom_startup_timing(true);
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        state.original_timing_host_dispatch_active = true;
        state.clear_nmi_update_latch();
        state.capture_display_snapshot();
        state.pending_main_loop_common_suffix =
            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
        state.original_timing_semantic_receipts = Some(
            OriginalTimingHostReceipts::new(0, 0, semantic)
                .with_presented_inidisp(crate::PresentedInidisp::new(7, 0, None).unwrap()),
        );
        state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
        state.follower_link_state_mut().set_joypad1h_last(0xa5);

        let receipts_before = state.original_timing_semantic_receipts.clone();
        let gates_before = state.original_timing_expected_nmi_update_gates.clone();
        let pending_before = (
            state.original_timing_nmi_publication_pending,
            state.original_timing_pending_nmi_update_gate,
        );
        let suffix_before = state.pending_main_loop_common_suffix;
        let sprite_preparation_before = state.main_loop_sprite_preparation_completed;
        let epoch_before = state.display_snapshot_epoch;
        let snapshot_before = state.display_snapshot.as_ref().map(|snapshot| {
            (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.presented_inidisp_override,
                snapshot.ppu.vram[0x7c00],
            )
        });
        let latch_before = state.game_state.display.nmi_update_is_latched();
        let joypad_before = state.game_state.player.follower_link.joypad1h_last();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.finish_original_timing_host_dispatch(true);
        }));

        assert!(result.is_err());
        assert_eq!(state.original_timing_semantic_receipts, receipts_before);
        assert_eq!(
            state.original_timing_expected_nmi_update_gates,
            gates_before
        );
        assert_eq!(
            (
                state.original_timing_nmi_publication_pending,
                state.original_timing_pending_nmi_update_gate,
            ),
            pending_before,
        );
        assert_eq!(state.pending_main_loop_common_suffix, suffix_before);
        assert_eq!(
            state.main_loop_sprite_preparation_completed,
            sprite_preparation_before,
        );
        assert_eq!(state.display_snapshot_epoch, epoch_before);
        assert_eq!(
            state.display_snapshot.as_ref().map(|snapshot| (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.presented_inidisp_override,
                snapshot.ppu.vram[0x7c00],
            )),
            snapshot_before,
        );
        assert_eq!(
            state.game_state.display.nmi_update_is_latched(),
            latch_before
        );
        assert_eq!(
            state.game_state.player.follower_link.joypad1h_last(),
            joypad_before,
        );
        assert!(state.original_timing_host_dispatch_active);
    }
}

#[test]
fn host_close_rejects_invalid_pending_and_gate_state_before_mutation() {
    #[derive(Clone, Copy, Debug)]
    enum InvalidState {
        AcceptanceBehindPending,
        AcceptanceMissingGate,
        AcceptanceWrongGate,
        AcceptanceNativeMismatch,
        ExhaustedGateWithoutPending,
        PendingMissingSnapshot,
        PendingClosedSnapshot,
        PendingNativeMismatch,
    }

    for case in [
        InvalidState::AcceptanceBehindPending,
        InvalidState::AcceptanceMissingGate,
        InvalidState::AcceptanceWrongGate,
        InvalidState::AcceptanceNativeMismatch,
        InvalidState::ExhaustedGateWithoutPending,
        InvalidState::PendingMissingSnapshot,
        InvalidState::PendingClosedSnapshot,
        InvalidState::PendingNativeMismatch,
    ] {
        let mut state = ZeldaState::new();
        state.set_rom_startup_timing(true);
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        state.original_timing_host_dispatch_active = true;
        state.clear_nmi_update_latch();
        state.capture_display_snapshot();
        state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
            0,
            0,
            match case {
                InvalidState::AcceptanceBehindPending
                | InvalidState::AcceptanceMissingGate
                | InvalidState::AcceptanceWrongGate
                | InvalidState::AcceptanceNativeMismatch => {
                    vec![OriginalTimingSemanticReceipt::NmiAccepted(
                        NmiUpdateGate::Open,
                    )]
                }
                _ => Vec::new(),
            },
        ));
        match case {
            InvalidState::AcceptanceBehindPending => {
                state.original_timing_nmi_publication_pending = true;
                state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
                state.original_timing_expected_nmi_update_gates =
                    vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::Open];
            }
            InvalidState::AcceptanceMissingGate => {}
            InvalidState::AcceptanceWrongGate => {
                state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
            }
            InvalidState::AcceptanceNativeMismatch => {
                state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
                state.latch_nmi_update();
            }
            InvalidState::ExhaustedGateWithoutPending => {
                state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
            }
            InvalidState::PendingMissingSnapshot => {
                state.original_timing_nmi_publication_pending = true;
                state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
                state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
                state.display_snapshot = None;
            }
            InvalidState::PendingClosedSnapshot => {
                state.original_timing_nmi_publication_pending = true;
                state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
                state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
                state
                    .display_snapshot
                    .as_mut()
                    .unwrap()
                    .accepts_nmi_dma_receipts = false;
            }
            InvalidState::PendingNativeMismatch => {
                state.original_timing_nmi_publication_pending = true;
                state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
                state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
                state.latch_nmi_update();
            }
        }

        let receipts_before = state.original_timing_semantic_receipts.clone();
        let gates_before = state.original_timing_expected_nmi_update_gates.clone();
        let pending_before = (
            state.original_timing_nmi_publication_pending,
            state.original_timing_pending_nmi_update_gate,
        );
        let snapshot_before = state.display_snapshot.as_ref().map(|snapshot| {
            (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
            )
        });
        let epoch_before = state.display_snapshot_epoch;
        let latch_before = state.game_state.display.nmi_update_is_latched();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.finish_original_timing_host_dispatch(true);
        }));

        assert!(
            result.is_err(),
            "invalid host-close case {case:?} was accepted"
        );
        assert_eq!(state.original_timing_semantic_receipts, receipts_before);
        assert_eq!(
            state.original_timing_expected_nmi_update_gates,
            gates_before
        );
        assert_eq!(
            (
                state.original_timing_nmi_publication_pending,
                state.original_timing_pending_nmi_update_gate,
            ),
            pending_before,
        );
        assert_eq!(
            state.display_snapshot.as_ref().map(|snapshot| (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
            )),
            snapshot_before,
        );
        assert_eq!(state.display_snapshot_epoch, epoch_before);
        assert_eq!(
            state.game_state.display.nmi_update_is_latched(),
            latch_before
        );
        assert!(state.original_timing_host_dispatch_active);
    }
}

#[test]
fn repeated_idle_continued_host_retains_the_one_source_common_suffix() {
    let mut state = live_idle_dialogue_main_loop_state();
    state.capture_display_snapshot();
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            3_815,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                    high: 0x12,
                    low: 0x34,
                    high_filtered: 0x56,
                    low_filtered: 0x78,
                }),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::IterationStarted,
                ),
                OriginalTimingSemanticReceipt::SpriteMainReturned,
            ],
        ))
        .unwrap();

    // Source run3815 begins one ZeldaRunGameLoop at $8053, completes
    // Sprite_Main's slot-zero return at $06:83a7, then returns from Module0E at
    // $0e:c9fd without reaching $805f. This host owns the one ordinary caller
    // suffix for every following Continued slice.
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert_eq!(state.game_state.player.follower_link.joypad1h_last(), 0x12,);
    let frame_counter = state.game_state.frame.frame_counter;
    let epoch_before_continued = state.display_snapshot_epoch;
    let snapshot_before_continued = state.display_snapshot.as_ref().unwrap().ppu.vram.clone();
    state.ram[OAM_BUF] = 0x6a;

    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            3_816,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
            ],
        ))
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.frame_counter, frame_counter);
    assert_eq!(
        state.ram[OAM_BUF], 0x6a,
        "CallStackContinued must not replay ZeldaRunGameLoop's ClearOamBuffer prefix",
    );
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
        "run3816 must retain run3815's one-shot suffix rather than install another owner",
    );
    assert_eq!(state.display_snapshot_epoch, epoch_before_continued + 1);
    assert_eq!(
        state.display_snapshot.as_ref().unwrap().ppu.vram,
        snapshot_before_continued,
        "the Held handler may refine its acceptance capture but cannot publish core VRAM DMA",
    );
    assert_eq!(
        state.game_state.player.follower_link.joypad1h_last(),
        0x12,
        "the same-host Held handler must not publish Joypad input",
    );
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn fresh_uninterrupted_iteration_retires_one_pre_main_timing_shadow() {
    let mut state = live_idle_dialogue_main_loop_state();
    state.capture_display_snapshot();
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
    state
        .game_execution_scheduler
        .schedule_pre_main_nmi_resume(PreMainNmiResume::DungeonSupertileQuadrantUploads);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            3_815,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                    high: 0x12,
                    low: 0x34,
                    high_filtered: 0x56,
                    low_filtered: 0x78,
                }),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::IterationStarted,
                ),
                OriginalTimingSemanticReceipt::SpriteMainReturned,
            ],
        ))
        .unwrap();

    let frame_counter = state.game_state.frame.frame_counter;
    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.frame_counter, frame_counter + 1);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_execution_scheduler.pre_main_nmi_resume(), None);
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn suspended_fresh_idle_iteration_can_carry_one_trailing_held_acceptance() {
    let mut state = live_idle_dialogue_main_loop_state();
    state.latch_nmi_update();
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            3_815,
            0,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::IterationStarted,
                ),
                OriginalTimingSemanticReceipt::SpriteMainReturned,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            ],
        ))
        .unwrap();
    let epoch_before = state.display_snapshot_epoch;

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert_eq!(state.display_snapshot_epoch, epoch_before + 1);
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn idle_main_loop_plan_rejects_malformed_authority_before_mutation() {
    fn assert_rejected(mut state: ZeldaState, semantic: Vec<OriginalTimingSemanticReceipt>) {
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        state.original_timing_host_dispatch_active = true;
        state.original_timing_semantic_receipts =
            Some(OriginalTimingHostReceipts::new(3_816, 0, semantic));
        let receipts_before = state.original_timing_semantic_receipts.clone();
        let scheduler_before = state.game_execution_scheduler;
        let suffix_before = state.pending_main_loop_common_suffix;
        let frame_before = state.game_state.frame;
        let pending_before = (
            state.original_timing_nmi_publication_pending,
            state.original_timing_pending_nmi_update_gate,
        );
        let gates_before = state.original_timing_expected_nmi_update_gates.clone();
        let latch_before = state.game_state.display.nmi_update_is_latched();
        let epoch_before = state.display_snapshot_epoch;
        let sprite_main_claims_before = state.original_timing_sprite_main_return_claims_remaining;
        let snapshot_before = state.display_snapshot.as_ref().map(|snapshot| {
            (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.vram.clone(),
            )
        });
        let audio_before = (
            state.game_state.system_signals.ambient_sound_effect(),
            state.game_state.system_signals.last_ambient_sound_effect(),
            state.zelda_debug_apu_write_ports(),
            state.audio_nmi_processed_before_main,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);
        }));
        assert!(result.is_err());
        assert_eq!(state.original_timing_semantic_receipts, receipts_before);
        assert_eq!(state.game_execution_scheduler, scheduler_before);
        assert_eq!(state.pending_main_loop_common_suffix, suffix_before);
        assert_eq!(state.game_state.frame, frame_before);
        assert_eq!(
            (
                state.original_timing_nmi_publication_pending,
                state.original_timing_pending_nmi_update_gate,
            ),
            pending_before,
        );
        assert_eq!(
            state.original_timing_expected_nmi_update_gates,
            gates_before
        );
        assert_eq!(
            state.game_state.display.nmi_update_is_latched(),
            latch_before
        );
        assert_eq!(state.display_snapshot_epoch, epoch_before);
        assert_eq!(
            state.original_timing_sprite_main_return_claims_remaining,
            sprite_main_claims_before,
        );
        assert_eq!(
            state.display_snapshot.as_ref().map(|snapshot| (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.vram.clone(),
            )),
            snapshot_before,
        );
        assert_eq!(
            (
                state.game_state.system_signals.ambient_sound_effect(),
                state.game_state.system_signals.last_ambient_sound_effect(),
                state.zelda_debug_apu_write_ports(),
                state.audio_nmi_processed_before_main,
            ),
            audio_before,
        );
    }

    let continued = vec![
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ),
    ];
    let mut missing_suffix = live_idle_dialogue_main_loop_state();
    missing_suffix.latch_nmi_update();
    missing_suffix.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    assert_rejected(missing_suffix, continued.clone());

    let mut stale_sprite_main_scope = live_idle_dialogue_main_loop_state();
    stale_sprite_main_scope.original_timing_sprite_main_return_claims_remaining = Some(0);
    assert_rejected(
        stale_sprite_main_scope,
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::IterationStarted,
        )],
    );

    let mut specialized_suffix = live_idle_dialogue_main_loop_state();
    specialized_suffix.latch_nmi_update();
    specialized_suffix.pending_main_loop_common_suffix = Some(
        MainLoopCommonSuffixContinuation::ResumeSpritePreparationExtendedOamPackingAndClearNmiLatch {
            next_group_start: 4,
        },
    );
    specialized_suffix.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    assert_rejected(specialized_suffix, continued.clone());

    let mut wrong_native_gate = live_idle_dialogue_main_loop_state();
    wrong_native_gate.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    wrong_native_gate.clear_nmi_update_latch();
    wrong_native_gate.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    assert_rejected(wrong_native_gate, continued.clone());

    let mut wrong_gate_queue = live_idle_dialogue_main_loop_state();
    wrong_gate_queue.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    wrong_gate_queue.latch_nmi_update();
    wrong_gate_queue.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
    assert_rejected(wrong_gate_queue, continued.clone());

    let mut extra = live_idle_dialogue_main_loop_state();
    extra.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    extra.latch_nmi_update();
    extra.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    let mut extra_semantic = continued.clone();
    extra_semantic.insert(
        2,
        OriginalTimingSemanticReceipt::DmaPublicationCompleted { channel_mask: 1 },
    );
    assert_rejected(extra, extra_semantic);

    let mut reordered_joypad = live_idle_dialogue_main_loop_state();
    reordered_joypad.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
    assert_rejected(
        reordered_joypad,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                high: 1,
                low: 2,
                high_filtered: 3,
                low_filtered: 4,
            }),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
        ],
    );

    let mut arm_with_open = live_idle_dialogue_main_loop_state();
    arm_with_open.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
    assert_rejected(
        arm_with_open,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    );

    let mut in_module_completion = live_idle_dialogue_main_loop_state();
    in_module_completion.latch_nmi_update();
    in_module_completion.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    assert_rejected(
        in_module_completion,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    );

    let mut held_after_suffix = live_idle_dialogue_main_loop_state();
    held_after_suffix.latch_nmi_update();
    held_after_suffix.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    assert_rejected(
        held_after_suffix,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        ],
    );

    let mut completed_trailing_held = live_idle_dialogue_main_loop_state();
    completed_trailing_held.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::Open, NmiUpdateGate::LatchHeld];
    assert_rejected(
        completed_trailing_held,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                high: 1,
                low: 2,
                high_filtered: 3,
                low_filtered: 4,
            }),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        ],
    );

    let mut stale_iteration = live_idle_dialogue_main_loop_state();
    stale_iteration.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    stale_iteration.latch_nmi_update();
    assert_rejected(
        stale_iteration,
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::IterationStarted,
        )],
    );

    let mut closed_while_continued = live_idle_dialogue_main_loop_state();
    closed_while_continued.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    closed_while_continued.latch_nmi_update();
    assert_rejected(
        closed_while_continued,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::DialogueClosed,
        ],
    );

    let mut endpoint_outside_dialogue = live_idle_dialogue_main_loop_state();
    endpoint_outside_dialogue.set_main_module(13);
    endpoint_outside_dialogue.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    endpoint_outside_dialogue.latch_nmi_update();
    assert_rejected(
        endpoint_outside_dialogue,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                crate::DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                    message_read_position: 0x37,
                },
            ),
        ],
    );

    let sprite_return_before_iteration = live_idle_dialogue_main_loop_state();
    assert_rejected(
        sprite_return_before_iteration,
        vec![
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
        ],
    );

    let mut interleaved_sprite_returns = live_idle_dialogue_main_loop_state();
    interleaved_sprite_returns.latch_nmi_update();
    interleaved_sprite_returns.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::LatchHeld];
    assert_rejected(
        interleaved_sprite_returns,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::SpriteMainReturned,
        ],
    );

    let mut continued_sprite_return_after_progress = live_idle_dialogue_main_loop_state();
    continued_sprite_return_after_progress.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    continued_sprite_return_after_progress.latch_nmi_update();
    assert_rejected(
        continued_sprite_return_after_progress,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::SpriteMainReturned,
        ],
    );

    let mut continued_sprite_return_before_progress = live_idle_dialogue_main_loop_state();
    continued_sprite_return_before_progress.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    continued_sprite_return_before_progress.latch_nmi_update();
    assert_rejected(
        continued_sprite_return_before_progress,
        vec![
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
        ],
    );

    let spotlight_outside_its_module = live_idle_dialogue_main_loop_state();
    assert_rejected(
        spotlight_outside_its_module,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                crate::SpotlightTableBuildProgressReceipt {
                    progress: crate::SpotlightTableBuildProgress {
                        completed_iterations: 1,
                        checkpoint:
                            crate::SpotlightTableBuildCheckpoint::BeforeIterationInitialization,
                    },
                    boundary: OriginalTimingBoundary::HostReturn,
                },
            ),
        ],
    );

    let run4782_state = || {
        let mut state = live_idle_dialogue_main_loop_state();
        state.set_main_module(0x0f);
        state.set_submodule(0);
        state.set_subsubmodule(0);
        state.original_timing_expected_nmi_update_gates =
            vec![NmiUpdateGate::Open, NmiUpdateGate::LatchHeld];
        state
    };

    let mut progress_before_held_acceptance = run4782_spotlight_semantic();
    progress_before_held_acceptance.swap(5, 6);
    assert_rejected(run4782_state(), progress_before_held_acceptance);

    let mut receipt_separated_from_held_acceptance = run4782_spotlight_semantic();
    receipt_separated_from_held_acceptance.insert(
        6,
        OriginalTimingSemanticReceipt::DmaPublicationCompleted { channel_mask: 1 },
    );
    assert_rejected(run4782_state(), receipt_separated_from_held_acceptance);

    let mut open_at_spotlight_boundary = run4782_spotlight_semantic();
    open_at_spotlight_boundary[5] = OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open);
    let mut open_boundary_state = run4782_state();
    open_boundary_state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::Open, NmiUpdateGate::Open];
    assert_rejected(open_boundary_state, open_at_spotlight_boundary);

    let mut completed_spotlight_boundary = run4782_spotlight_semantic();
    completed_spotlight_boundary.insert(6, OriginalTimingSemanticReceipt::NmiHandlerCompleted);
    completed_spotlight_boundary.insert(
        7,
        OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
            high: 1,
            low: 2,
            high_filtered: 3,
            low_filtered: 4,
        }),
    );
    assert_rejected(run4782_state(), completed_spotlight_boundary);

    let mut duplicate_spotlight_claim = run4782_spotlight_semantic();
    duplicate_spotlight_claim.push(run4782_spotlight_progress_receipt());
    assert_rejected(run4782_state(), duplicate_spotlight_claim);

    let mut wrong_boundary = run4782_spotlight_semantic();
    let OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(mut wrong_boundary_claim) =
        run4782_spotlight_progress_receipt()
    else {
        unreachable!();
    };
    wrong_boundary_claim.boundary = OriginalTimingBoundary::HostReturn;
    wrong_boundary[6] =
        OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(wrong_boundary_claim);
    assert_rejected(run4782_state(), wrong_boundary);

    let mut wrong_spotlight_gate_queue = run4782_state();
    wrong_spotlight_gate_queue.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::Open, NmiUpdateGate::Open];
    assert_rejected(wrong_spotlight_gate_queue, run4782_spotlight_semantic());

    let mut wrong_spotlight_native_latch = run4782_state();
    wrong_spotlight_native_latch.latch_nmi_update();
    assert_rejected(wrong_spotlight_native_latch, run4782_spotlight_semantic());

    // A LinkPositionBeforeCoordinates interruption on a fresh idle iteration
    // is no longer a rejected shape: route host 50635 proved it on the wire
    // (Module0F's spotlight-close Link movement consumes the boundary inside
    // the module body), so the interrupted-idle plan forwards it instead of
    // failing closed.

    let reordered_interrupted_sprite_return = live_idle_dialogue_main_loop_state();
    assert_rejected(
        reordered_interrupted_sprite_return,
        vec![
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                crate::MainLoopInterruption::LinkOam,
            ),
        ],
    );

    let mut non_timing_pre_main = live_idle_dialogue_main_loop_state();
    non_timing_pre_main
        .game_execution_scheduler
        .schedule_pre_main_nmi_resume(PreMainNmiResume::OverworldAuxGraphicsReturn);
    assert_rejected(
        non_timing_pre_main,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                crate::MainLoopInterruption::LinkOam,
            ),
        ],
    );

    let mut timing_shadow_without_handler = live_idle_dialogue_main_loop_state();
    timing_shadow_without_handler
        .game_execution_scheduler
        .schedule_pre_main_nmi_resume(PreMainNmiResume::DungeonSupertileQuadrantUploads);
    assert_rejected(
        timing_shadow_without_handler,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                crate::MainLoopInterruption::LinkOam,
            ),
        ],
    );

    let mut timing_shadow_with_attached_work = live_idle_dialogue_main_loop_state();
    timing_shadow_with_attached_work.capture_display_snapshot();
    timing_shadow_with_attached_work.original_timing_nmi_publication_pending = true;
    timing_shadow_with_attached_work.original_timing_pending_nmi_update_gate =
        Some(NmiUpdateGate::Open);
    timing_shadow_with_attached_work.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::Open];
    timing_shadow_with_attached_work
        .game_execution_scheduler
        .schedule_pre_main_nmi_resume(PreMainNmiResume::DungeonSupertileQuadrantUploads);
    assert!(timing_shadow_with_attached_work
        .game_execution_scheduler
        .schedule_after_pending_pre_main_nmi(
            GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn,
        ));
    assert_rejected(
        timing_shadow_with_attached_work,
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
        ],
    );

    let mut unsupported_scheduled_predecessor = live_idle_dialogue_main_loop_state();
    unsupported_scheduled_predecessor
        .game_execution_scheduler
        .schedule_work(
            GameWorkContinuation::FinishDialogueInitializationPrefix {
                caller_nmi_crossings: 0,
            },
            1,
        );
    assert_rejected(
        unsupported_scheduled_predecessor,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                crate::MainLoopInterruption::LinkOam,
            ),
        ],
    );

    let scheduled_vector = |sprite_main_returns: usize,
                            interruption: crate::MainLoopInterruption| {
        let mut semantic = vec![
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
        ];
        semantic.extend(std::iter::repeat_n(
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            sprite_main_returns,
        ));
        semantic.push(OriginalTimingSemanticReceipt::MainLoopInterrupted(
            interruption,
        ));
        semantic
    };

    let scheduled_state13 = |nmi_slices| {
        let mut state = live_idle_dialogue_main_loop_state();
        state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
        state.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::State13CallerReturn,
            },
            nmi_slices,
        );
        state
    };
    assert_rejected(
        scheduled_state13(2),
        scheduled_vector(1, crate::MainLoopInterruption::SpritePreparation),
    );
    assert_rejected(
        scheduled_state13(1),
        scheduled_vector(0, crate::MainLoopInterruption::SpritePreparation),
    );
    assert_rejected(
        scheduled_state13(1),
        scheduled_vector(2, crate::MainLoopInterruption::SpritePreparation),
    );

    let mut spotlight_with_plain_sprite_preparation = live_idle_dialogue_main_loop_state();
    spotlight_with_plain_sprite_preparation.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::Open];
    spotlight_with_plain_sprite_preparation.schedule_spotlight_iteration_return(
        SpotlightIteration::closing(SpotlightIterationPhase::MixedTailAfterReturn),
    );
    assert_rejected(
        spotlight_with_plain_sprite_preparation,
        scheduled_vector(1, crate::MainLoopInterruption::SpritePreparation),
    );
}

#[test]
fn callback_calling_public_internal_does_not_double_advance_live_timing() {
    let mut state = ZeldaState::new();
    state.set_rom(&exact_timing_test_rom(0x18));
    state.set_rom_startup_timing(true);
    state.zelda_setup_emu_callbacks(None, Some(test_run_frame_callback_calls_internal), None);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0x1234,
            Vec::new(),
        ))
        .unwrap();

    state.zelda_run_frame_with_replay_input_override(0x1234, None);

    assert_eq!(state.ram[0x43], 1);
    assert_eq!(state.original_timing_owner(), OriginalTimingOwner::Live);
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(0));
    assert!(!state.original_timing_host_dispatch_active);
}

#[test]
fn state_recorder_records_input_edges_like_c() {
    let mut sr = StateRecorder {
        last_inputs: 0xffff,
        frames_since_last: 99,
        total_frames: 88,
        replay_mode: true,
        log: ByteArray { data: vec![1, 2] },
        base_snapshot: ByteArray { data: vec![3] },
        ..StateRecorder::default()
    };
    ZeldaState::state_recorder_init(&mut sr);
    assert_eq!(sr, StateRecorder::default());

    ZeldaState::state_recorder_record(&mut sr, 0x0001);
    ZeldaState::state_recorder_record(&mut sr, 0x0001);
    ZeldaState::state_recorder_record(&mut sr, 0x0003);

    assert_eq!(sr.last_inputs, 0x0003);
    assert_eq!(sr.frames_since_last, 1);
    assert_eq!(sr.total_frames, 3);
    assert_eq!(sr.log.data, vec![0x00, 0x12]);
}

#[test]
fn state_recorder_records_long_waits_and_patch_bytes() {
    let mut sr = StateRecorder {
        frames_since_last: 20,
        ..StateRecorder::default()
    };
    ZeldaState::state_recorder_record_cmd(&mut sr, 0x00);
    assert_eq!(sr.frames_since_last, 0);
    assert_eq!(sr.log.data, vec![0x0f, 5]);

    ZeldaState::state_recorder_record_patch_byte(
        &mut sr,
        0x10020,
        &[0xaa, 0xbb, 0xcc, 0xdd, 0xee],
        5,
    );
    assert_eq!(
        sr.log.data,
        vec![0x0f, 5, 0xce, 1, 0x00, 0x20, 0xaa, 0xbb, 0xcc, 0xdd, 0xee]
    );
}

#[test]
fn state_recorder_replays_input_edges_like_c() {
    let mut state = ZeldaState::new();
    let mut sr = StateRecorder {
        replay_mode: true,
        total_frames: 3,
        log: ByteArray {
            data: vec![0x00, 0x12],
        },
        ..StateRecorder::default()
    };

    assert_eq!(state.state_recorder_read_next_replay_state(&mut sr), 0x0001);
    assert!(sr.replay_mode);
    assert_eq!(state.state_recorder_read_next_replay_state(&mut sr), 0x0001);
    assert!(sr.replay_mode);
    assert_eq!(state.state_recorder_read_next_replay_state(&mut sr), 0x0003);
    assert!(!sr.replay_mode);
}

#[test]
fn state_recorder_replay_input_override_advances_replay_but_substitutes_input() {
    let mut state = ZeldaState::new();
    let mut sr = StateRecorder {
        replay_mode: true,
        total_frames: 3,
        log: ByteArray {
            data: vec![0x00, 0x12],
        },
        ..StateRecorder::default()
    };

    assert_eq!(
        state.state_recorder_read_next_replay_state_with_input_override(&mut sr, None),
        0x0001
    );
    assert!(sr.replay_mode);
    assert_eq!(
        state.state_recorder_read_next_replay_state_with_input_override(&mut sr, Some(0x0080)),
        0x0080
    );
    assert!(sr.replay_mode);
    assert_eq!(
        state.state_recorder_read_next_replay_state_with_input_override(&mut sr, None),
        0x0003
    );
    assert!(!sr.replay_mode);
}

#[test]
fn state_recorder_replays_patch_bytes_and_can_stop() {
    let mut state = ZeldaState::new();
    let mut sr = StateRecorder {
        replay_mode: true,
        total_frames: 1,
        log: ByteArray {
            data: vec![0xce, 1, 0x00, 0x20, 0xaa, 0xbb, 0xcc, 0xdd, 0xee],
        },
        ..StateRecorder::default()
    };

    assert_eq!(state.state_recorder_read_next_replay_state(&mut sr), 0);
    assert_eq!(
        &state.ram[0x10020..0x10025],
        &[0xaa, 0xbb, 0xcc, 0xdd, 0xee]
    );
    assert!(!sr.replay_mode);

    sr.replay_mode = true;
    sr.replay_frame_counter = 7;
    sr.replay_pos_last_complete = 3;
    ZeldaState::state_recorder_stop_replay(&mut sr);
    assert!(!sr.replay_mode);
    assert_eq!(sr.total_frames, 7);
    assert_eq!(sr.log.data, vec![0xce, 1, 0x00]);
}

#[test]
fn state_recorder_replays_snapshot_boundary_commands() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    assert_eq!(
        state.original_timing_owner(),
        OriginalTimingOwner::PendingColdStart
    );
    state.ram[0x1234] = 0x5a;
    state.sram[0x234] = 0x6b;
    state.ppu.cgram[3] = 0x1357;

    let mut snapshot = ByteArray::default();
    let mut save = SaveLoadFunc::Save(&mut snapshot);
    state.save_snes_state(&mut save);

    state.ram[0x1234] = 0;
    state.sram[0x234] = 0;
    state.ppu.cgram[3] = 0;

    let mut log = ByteArray::default();
    ByteArray_AppendByte(&mut log, 0xd0);
    ZeldaState::byte_array_append_vl(&mut log, snapshot.size() as u32);
    ByteArray_AppendData(&mut log, &snapshot.data);

    let mut sr = StateRecorder {
        replay_mode: true,
        total_frames: 1,
        last_inputs: 0xffff,
        log,
        ..StateRecorder::default()
    };

    assert_eq!(state.state_recorder_read_next_replay_state(&mut sr), 0);
    assert_eq!(state.ram[0x1234], 0x5a);
    assert_eq!(state.sram[0x234], 0x6b);
    assert_eq!(state.ppu.cgram[3], 0x1357);
    assert_eq!(
        state.original_timing_owner(),
        OriginalTimingOwner::Unavailable(OriginalTimingUnavailableReason::CheckpointRestore)
    );
    assert_eq!(sr.last_inputs, 0);
    assert!(!sr.replay_mode);
}

#[test]
fn state_recorder_clear_key_log_rebases_snapshot_and_active_inputs() {
    let mut state = ZeldaState::new();
    state.ram[0x100] = 0x56;
    let mut sr = StateRecorder {
        last_inputs: 0x0003,
        frames_since_last: 5,
        total_frames: 12,
        log: ByteArray {
            data: vec![0xaa, 0xbb],
        },
        ..StateRecorder::default()
    };

    state.state_recorder_clear_key_log(&mut sr);

    assert!(!sr.base_snapshot.data.is_empty());
    assert_eq!(sr.log.data, vec![0x00, 0x10]);
    assert_eq!(sr.frames_since_last, 0);
    assert_eq!(sr.total_frames, 0);
}

#[test]
fn zelda_run_frame_sanitizes_inputs_and_records_features() {
    let mut state = ZeldaState::new();
    state.wanted_zelda_features = 0x1000;
    state.set_animated_tile_data_source_address(1);

    let was_replay = state.zelda_run_frame(0x30 | 0xc0 | 1);

    assert!(!was_replay);
    assert_eq!(state.frame_ctr_dbg, 1);
    assert_eq!(state.state_recorder.last_inputs, 0x00a1);
    assert_eq!(state.ram[RAM_BUGS_FIXED], BUGFIX_LATEST);
    assert_eq!(
        state.debug_compatibility_ram_u32(ENHANCED_FEATURES0),
        0x1000
    );
}

#[test]
fn perform_dash_sets_start_dash_state_and_tagalong_timeout() {
    let mut state = ZeldaState::new();
    state
        .follower_link_state_mut()
        .set_somaria_platform_state(1);
    state.link_perform_dash();
    assert_eq!(state.game_state.player.follower_link.handler_state(), 0);

    state
        .follower_link_state_mut()
        .set_somaria_platform_state(0);
    state
        .follower_link_state_mut()
        .set_y_button_action_flags(0xff);
    state.follower_link_state_mut().set_button_mask_b_y(0x7f);
    state.follower_link_state_mut().set_state_bits(0x7f);
    state.follower_link_state_mut().set_item_in_hand(3);
    state.ram[PLAYER_DEFENSE_FLAGS] = 0xff;
    set_link_test_byte(&mut state, LINK_MOVING_AGAINST_DIAG_TILE, 0xff);
    state.follower_link_state_mut().set_speed_setting(5);
    state.follower_state_mut().set_indicator(2);

    state.link_perform_dash();

    assert_eq!(
        state
            .game_state
            .player
            .follower_link
            .y_button_action_flags(),
        0
    );
    assert_eq!(link_test_byte(&state, LINK_COUNTDOWN_FOR_DASH), 29);
    assert_eq!(link_test_byte(&state, LINK_DASH_CTR), 64);
    assert_eq!(state.game_state.player.follower_link.handler_state(), 17);
    assert_eq!(state.game_state.player.follower_link.running_state(), 1);
    assert_eq!(state.game_state.player.follower_link.button_mask_b_y(), 0);
    assert_eq!(state.game_state.player.follower_link.state_bits(), 0);
    assert_eq!(state.game_state.player.follower_link.item_in_hand(), 0);
    assert_eq!(state.ram[PLAYER_DEFENSE_FLAGS], 0);
    assert_eq!(link_test_byte(&state, LINK_MOVING_AGAINST_DIAG_TILE), 0);
    assert_eq!(state.game_state.player.follower_link.speed_setting(), 0);
    assert_eq!(read_le_u16(&state.ram, TIMER_TAGALONG_REACQUIRE), 64);
}

#[test]
fn dash_repel_applies_tile_rebound_state() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_running_state(1);
    set_link_test_byte(&mut state, LINK_DASH_CTR, 32);
    set_link_test_byte(&mut state, LINK_FLAG_MOVING, 2);
    set_link_test_byte(&mut state, LINK_LAST_DIRECTION_MOVED_TOWARDS, 3);

    state.repel_dash();

    assert_eq!(
        state.game_state.player.follower_link.actual_x_velocity(),
        0u8.wrapping_sub(24)
    );
    assert_eq!(state.game_state.player.follower_link.actual_y_velocity(), 0);
    assert_eq!(
        state.game_state.player.follower_link.actual_z_velocity(),
        36
    );
    assert_eq!(
        state.game_state.player.follower_link.incapacitated_timer(),
        24
    );
    assert_eq!(state.game_state.player.follower_link.auxiliary_state(), 1);
    assert_eq!(link_test_byte(&state, LINK_WANT_MAKE_NOISE_WHEN_DASHED), 1);
    assert_eq!(link_test_byte(&state, LINK_DIRECTION), 1);
    assert_eq!(
        state.game_state.player.swim_acceleration.acceleration(2),
        256
    );
}

#[test]
fn flag67_with_directions_derives_direction_from_actual_velocity() {
    let mut state = ZeldaState::new();
    set_link_test_byte(&mut state, LINK_DIRECTION, 0xff);
    state.follower_link_state_mut().set_actual_y_velocity(0xf0);
    state.follower_link_state_mut().set_actual_x_velocity(2);

    state.flag67_with_directions();

    assert_eq!(link_test_byte(&state, LINK_DIRECTION), 9);
}

#[test]
fn link_velocity_coordinate_boundary_resumes_to_the_atomic_c_result() {
    fn configured_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.set_main_module(0x0f);
        state.set_submodule(1);
        set_link_test_word(&mut state, LINK_X_COORD, 0x0100);
        set_link_test_word(&mut state, LINK_Y_COORD, 0x0200);
        set_link_test_byte(&mut state, LINK_DIRECTION, 8);
        set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 8);
        state.follower_link_state_mut().set_speed_setting(6);
        state
    }

    let mut atomic = configured_state();
    atomic.link_handle_velocity();

    let mut resumed = configured_state();
    let position_return = resumed
        .link_handle_velocity_until_position_integrated()
        .expect("ordinary Module0F movement must reach Player_MovePosition1_");

    // Player_MovePosition1_ has integrated coordinates, but the C calls to
    // moving-floor, conveyor, and drag/velocity-delta handling are still on
    // the suspended stack.
    assert_eq!(link_test_byte(&resumed, LINK_Y_VEL), 0);
    resumed.complete_link_move_position_after_coordinates(position_return);

    assert_eq!(resumed.ram, atomic.ram);
}

#[test]
fn swimming_handler_without_flippers_only_clears_action_state() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_button_mask_b_y(0xff);
    state.follower_link_state_mut().set_button_b_frames(9);
    set_link_test_byte(&mut state, LINK_DELAY_TIMER_SPIN_ATTACK, 7);
    set_link_test_byte(&mut state, LINK_SPIN_ATTACK_STEP_COUNTER, 6);
    state.follower_link_state_mut().set_state_bits(5);
    state.follower_link_state_mut().set_picking_throw_state(4);
    set_link_test_byte(&mut state, LINK_ITEM_FLIPPERS, 0);

    state.player_handler_04_swimming();

    assert_eq!(state.game_state.player.follower_link.button_mask_b_y(), 0);
    assert_eq!(state.game_state.player.follower_link.button_b_frames(), 0);
    assert_eq!(link_test_byte(&state, LINK_DELAY_TIMER_SPIN_ATTACK), 0);
    assert_eq!(link_test_byte(&state, LINK_SPIN_ATTACK_STEP_COUNTER), 0);
    assert_eq!(state.game_state.player.follower_link.state_bits(), 0);
    assert_eq!(
        state.game_state.player.follower_link.picking_throw_state(),
        0
    );
}

#[test]
fn handle_toss_clears_a_press_state_when_throwing() {
    let mut state = ZeldaState::new();
    state
        .follower_link_state_mut()
        .set_y_button_action_flags(0x80);
    state.follower_link_state_mut().set_filtered_joypad_l(0x80);
    state.follower_link_state_mut().set_item_action_step_var(7);
    state.follower_link_state_mut().set_throw_oam_state_index(8);
    state.follower_link_state_mut().set_y_button_action_step(9);
    set_link_test_byte(&mut state, LINK_CANT_CHANGE_DIRECTION, 0xff);

    assert!(state.link_handle_toss());

    assert_eq!(
        state.game_state.player.follower_link.item_action_step_var(),
        0
    );
    assert_eq!(
        state
            .game_state
            .player
            .follower_link
            .throw_oam_state_index(),
        0
    );
    assert_eq!(
        state.game_state.player.follower_link.y_button_action_step(),
        0
    );
    assert_eq!(
        state
            .game_state
            .player
            .follower_link
            .y_button_action_flags(),
        0
    );
    assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 0);

    state
        .follower_link_state_mut()
        .set_y_button_action_flags(0x80);
    state.follower_link_state_mut().set_filtered_joypad_l(0x80);
    state.follower_link_state_mut().set_picking_throw_state(1);
    assert!(!state.link_handle_toss());
    assert_eq!(
        state
            .game_state
            .player
            .follower_link
            .y_button_action_flags(),
        0x80
    );
}

#[test]
fn cape_item_activation_and_no_magic_prompt_match_c_state() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_filtered_joypad_h(0x40);
    set_link_test_byte(&mut state, LINK_MAGIC_POWER, 10);
    set_link_test_byte(&mut state, LINK_MAGIC_CONSUMPTION, 1);

    state.link_item_cape();

    assert_eq!(link_test_byte(&state, LINK_CAPE_MODE), 1);
    assert_eq!(state.ram[CAPE_DECREMENT_COUNTER], 8);
    assert_eq!(link_test_byte(&state, LINK_BUNNY_TRANSFORM_TIMER), 20);
    assert_eq!(state.game_state.system_signals.sound_effect_1() & 0x3f, 20);
    assert_eq!(
        state.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );

    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_filtered_joypad_h(0x40);
    state.link_item_cape();

    assert_eq!(link_test_byte(&state, LINK_CAPE_MODE), 0);
    assert_eq!(state.game_state.system_signals.sound_effect_1() & 0x3f, 60);
    assert_eq!(
        state.game_state.messaging.dialogue_message_index.value(),
        123
    );
    assert_eq!(state.game_state.frame.main_module, 14);
}

#[test]
fn lamp_powder_and_shovel_item_handlers_match_core_state() {
    let mut lamp = ZeldaState::new();
    lamp.follower_link_state_mut().set_filtered_joypad_h(0x40);
    lamp.inventory_items_mut().set_inventory_item(10, 1);
    lamp.follower_link_state_mut().set_magic_power(32);
    set_link_test_byte(&mut lamp, LINK_CANT_CHANGE_DIRECTION, 1);
    lamp.follower_link_state_mut().set_button_b_frames(9);
    lamp.link_item_lamp();
    assert_eq!(link_test_byte(&lamp, LINK_MAGIC_POWER), 28);
    assert_eq!(lamp.game_state.player.follower_link.button_mask_b_y(), 0);
    assert_eq!(lamp.game_state.player.follower_link.button_b_frames(), 0);
    assert_eq!(link_test_byte(&lamp, LINK_CANT_CHANGE_DIRECTION), 0);
    assert_eq!(lamp.ancilla_slot_view(4).ancilla_type(), 0x1a);
    assert_eq!(lamp.ancilla_slot_view(3).ancilla_type(), 0x2f);

    let mut powder = ZeldaState::new();
    powder.follower_link_state_mut().set_filtered_joypad_h(0x40);
    powder.inventory_items_mut().set_mushroom(2);
    powder.follower_link_state_mut().set_magic_power(16);
    powder.link_item_powder();
    assert_eq!(link_test_byte(&powder, LINK_MAGIC_POWER), 8);
    assert_eq!(powder.game_state.player.follower_link.item_in_hand(), 0x40);
    assert_eq!(link_test_byte(&powder, LINK_DELAY_TIMER_SPIN_ATTACK), 1);
    assert_eq!(link_test_byte(&powder, LINK_DIRECTION), 0);

    let mut shovel = ZeldaState::new();
    shovel.follower_link_state_mut().set_filtered_joypad_h(0x40);
    shovel.link_item_shovel();
    assert_eq!(shovel.game_state.player.follower_link.position_mode(), 1);
    assert_eq!(link_test_byte(&shovel, LINK_CANT_CHANGE_DIRECTION) & 1, 1);
    assert_eq!(link_test_byte(&shovel, LINK_DELAY_TIMER_SPIN_ATTACK), 6);

    set_link_test_byte(&mut shovel, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
    shovel.follower_link_state_mut().set_item_action_step_var(2);
    shovel.link_item_shovel();
    assert_eq!(
        shovel
            .game_state
            .player
            .follower_link
            .item_action_step_var(),
        0
    );
    assert_eq!(shovel.ram[PLAYER_HANDLER_TIMER], 0);
    assert_eq!(
        shovel.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );
    assert_eq!(shovel.game_state.player.follower_link.position_mode(), 0);
    assert_eq!(link_test_byte(&shovel, LINK_CANT_CHANGE_DIRECTION) & 1, 0);
}

#[test]
fn medallion_item_start_and_state_progression_match_core_state() {
    let mut ether = ZeldaState::new();
    ether.follower_link_state_mut().set_filtered_joypad_h(0x40);
    ether.inventory_items_mut().set_sword_type(1);
    ether.follower_link_state_mut().set_magic_power(64);
    ether.link_item_ether();
    assert_eq!(link_test_byte(&ether, LINK_MAGIC_POWER), 32);
    assert_eq!(ether.game_state.player.follower_link.handler_state(), 8);
    assert_eq!(link_test_byte(&ether, LINK_CANT_CHANGE_DIRECTION) & 1, 1);
    assert_eq!(link_test_byte(&ether, LINK_DELAY_TIMER_SPIN_ATTACK), 5);
    assert_eq!(ether.ram[STEP_COUNTER_FOR_SPIN_ATTACK], 0);
    assert_eq!(ether.game_state.system_signals.sound_effect_2() & 0x3f, 35);

    ether
        .follower_link_state_mut()
        .set_spin_attack_delay_timer(0);
    ether.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 9;
    ether.link_state_using_ether();
    assert_eq!(ether.ram[STEP_COUNTER_FOR_SPIN_ATTACK], 10);
    assert_eq!(ether.ram[SPIN_ATTACK_SOUND_LATCH], 1);
    assert_eq!(ether.ancilla_slot_view(4).ancilla_type(), 24);

    let mut quake = ZeldaState::new();
    quake.follower_link_state_mut().set_filtered_joypad_h(0x40);
    quake.inventory_items_mut().set_sword_type(1);
    quake.follower_link_state_mut().set_magic_power(64);
    quake.link_item_quake();
    assert_eq!(quake.game_state.player.follower_link.handler_state(), 10);
    assert_eq!(link_test_byte(&quake, LINK_ACTUAL_VEL_Z_MIRROR), 40);
    assert_eq!(link_test_byte(&quake, LINK_ACTUAL_VEL_Z_COPY_MIRROR), 40);
    assert_eq!(link_test_byte(&quake, LINK_Z_COORD_MIRROR), 0);

    let mut blocked = ZeldaState::new();
    blocked
        .follower_link_state_mut()
        .set_filtered_joypad_h(0x40);
    blocked.follower_link_state_mut().set_magic_power(64);
    blocked.link_item_bombos();
    assert_eq!(blocked.game_state.player.follower_link.handler_state(), 0);
    assert_eq!(
        blocked.game_state.system_signals.sound_effect_1() & 0x3f,
        60
    );
}

#[test]
fn hookshot_item_and_timeout_state_match_core_state() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_filtered_joypad_h(0x40);
    state.follower_link_state_mut().set_facing(4);
    set_link_test_word(&mut state, LINK_X_COORD, 0x0100);
    set_link_test_word(&mut state, LINK_Y_COORD, 0x0200);
    state.swim_acceleration_mut().set_speed_active_flag(0, 1);

    state.link_item_hookshot();

    assert_eq!(state.game_state.player.follower_link.handler_state(), 19);
    assert_eq!(state.game_state.player.follower_link.position_mode(), 4);
    assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 1);
    assert_eq!(link_test_byte(&state, LINK_DELAY_TIMER_SPIN_ATTACK), 7);
    assert_eq!(state.ancilla_slot_view(4).ancilla_type(), 0x1f);
    assert_eq!(
        state.game_state.messaging.runtime.game_over_letter_cursor(),
        4
    );
    assert_eq!(state.ram[ANCILLA_X_VEL + 4], 0xc0);
    assert_eq!(read_le_u16(&state.ram, ANCILLA_X_LO + 4), 0x00fc);

    state.ancilla_slot_view_mut(4).set_ancilla_type(0);
    set_link_test_byte(&mut state, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
    state.follower_link_state_mut().set_button_b_frames(12);
    state.link_state_hookshotting();
    assert_eq!(state.game_state.player.follower_link.handler_state(), 0);
    assert!(!state.game_state.player.follower_link.position_mode_has(4));
    assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 0);
    assert_eq!(
        state.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );
    assert_eq!(state.game_state.player.follower_link.button_b_frames(), 9);
}

#[test]
fn cane_of_byrna_start_and_finish_match_timer_state() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_filtered_joypad_h(0x40);
    set_link_test_byte(&mut state, LINK_MAGIC_POWER, 40);
    set_link_test_byte(&mut state, LINK_MAGIC_CONSUMPTION, 0);

    state.link_item_cane_of_byrna();

    assert_eq!(link_test_byte(&state, LINK_MAGIC_POWER), 24);
    assert_eq!(state.ancilla_slot_view(4).ancilla_type(), 0x30);
    assert_eq!(state.game_state.player.follower_link.position_mode(), 8);
    assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 1);
    assert_eq!(link_test_byte(&state, LINK_DELAY_TIMER_SPIN_ATTACK), 18);

    state.ancilla_slot_view_mut(4).set_ancilla_type(0);
    state.follower_link_state_mut().set_button_mask_b_y(0x40);
    set_link_test_byte(&mut state, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
    state.ram[PLAYER_HANDLER_TIMER] = 2;
    state.link_item_cane_of_byrna();

    assert_eq!(state.ram[PLAYER_HANDLER_TIMER], 0);
    assert_eq!(
        state.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );
    assert_eq!(state.game_state.player.follower_link.position_mode(), 0);
    assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 0);
}

#[test]
fn zapped_state_advances_timer_and_finishes_on_eighth_pulse() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_handler_state(7);
    set_link_test_byte(&mut state, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
    state.ram[PLAYER_HANDLER_TIMER] = 7;
    set_link_test_byte(&mut state, LINK_DISABLE_SPRITE_DAMAGE, 1);
    set_link_test_byte(&mut state, LINK_ELECTROCUTE_ON_TOUCH, 1);
    state.follower_link_state_mut().set_auxiliary_state(1);
    state.set_mosaic_level(0x20);
    state.set_mosaic_direction(1);

    state.link_state_zapped();

    assert_eq!(state.ram[PLAYER_HANDLER_TIMER], 0);
    assert_eq!(state.game_state.player.follower_link.handler_state(), 0);
    assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 0);
    assert_eq!(link_test_byte(&state, LINK_ELECTROCUTE_ON_TOUCH), 0);
    assert_eq!(state.game_state.player.follower_link.auxiliary_state(), 0);
    assert_eq!(state.game_state.display.mosaic_level, 0);
    assert_eq!(state.game_state.display.mosaic_copy, 3);
    assert_eq!(state.game_state.display.bg_mode, 9);
}

#[test]
fn tile_main_handler_icy_floor_starts_sliding_state() {
    let mut state = ZeldaState::new();
    state.set_indoor_flag(1);
    set_link_test_byte(&mut state, LINK_DIRECTION, 4);
    set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 8);
    state
        .tile_detect_position_mut()
        .set_location_calc_mask(0x01ff);
    state
        .dungeon_bg2_attributes_mut()
        .set_bg2_attr(16 * 8 + 1, 0x0e);

    state.tile_detect_main_handler(0);

    assert_eq!(link_test_byte(&state, LINK_FLAG_MOVING), 1);
    assert_eq!(
        state.game_state.player.follower_link.swim_direction_flags(),
        8
    );
    assert_eq!(
        state
            .game_state
            .player
            .follower_link
            .water_ripple_or_grass_state(),
        0
    );
}

#[test]
fn original_timing_owner_is_absent_from_serde_and_bincode_bytes() {
    let disabled = ZeldaState::new();
    let expected = bincode::serialize(&disabled).expect("serialize disabled timing owner");

    for owner in [
        OriginalTimingOwner::PendingColdStart,
        OriginalTimingOwner::Live,
        OriginalTimingOwner::Unavailable(OriginalTimingUnavailableReason::ProgressedState),
        OriginalTimingOwner::Unavailable(OriginalTimingUnavailableReason::CheckpointRestore),
    ] {
        let mut state = disabled.clone();
        state.original_timing_owner = match owner {
            OriginalTimingOwner::Disabled => OriginalTimingOwnerState::Disabled,
            OriginalTimingOwner::PendingColdStart => OriginalTimingOwnerState::PendingColdStart,
            OriginalTimingOwner::Live => OriginalTimingOwnerState::Live,
            OriginalTimingOwner::Unavailable(reason) => {
                OriginalTimingOwnerState::Unavailable(reason)
            }
        };
        state.original_timing_cold_start_eligible = false;
        assert_eq!(
            bincode::serialize(&state).expect("serialize runtime-only timing owner"),
            expected,
            "runtime-only owner {owner:?} changed positional checkpoint bytes"
        );
    }

    let mut oracle_state = disabled.clone();
    oracle_state.original_timing_owner = OriginalTimingOwnerState::Live;
    oracle_state.original_timing_cold_start_eligible = false;
    oracle_state.original_timing_host_dispatch_active = true;
    oracle_state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        41,
        0x1234,
        vec![OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open,
        )],
    ));
    oracle_state.original_timing_presented_audio =
        Some(crate::PresentedAudio::new(vec![1, -1]).unwrap());
    oracle_state.original_timing_audio_shadow_result =
        Some(crate::OriginalTimingAudioShadowResult {
            sample_frames: 1,
            mismatched_interleaved_samples: 2,
            first_mismatch_interleaved: Some(0),
        });
    assert_eq!(
        bincode::serialize(&oracle_state).expect("serialize oracle runtime-only receipt"),
        expected,
        "oracle owner and semantic receipt changed positional checkpoint bytes",
    );

    let restored: ZeldaState =
        bincode::deserialize(&expected).expect("deserialize unchanged ZeldaState bytes");
    assert_eq!(
        restored.original_timing_owner(),
        OriginalTimingOwner::Unavailable(OriginalTimingUnavailableReason::CheckpointRestore)
    );
    assert!(!restored.original_timing_cold_start_eligible);
}

#[test]
fn restored_frame_zero_cannot_be_reenabled_as_a_fresh_cold_start() {
    let encoded = bincode::serialize(&ZeldaState::new()).expect("serialize frame-zero checkpoint");
    let mut restored: ZeldaState =
        bincode::deserialize(&encoded).expect("deserialize frame-zero checkpoint");
    assert_eq!(restored.frame_ctr_dbg, 0);

    restored.set_rom_startup_timing(false);
    assert_eq!(
        restored.original_timing_owner(),
        OriginalTimingOwner::Disabled
    );
    let audio_before_reenable = restored.zelda_audio_snapshot_bytes();
    restored.set_rom_startup_timing(true);

    assert_eq!(
        restored.original_timing_owner(),
        OriginalTimingOwner::Unavailable(OriginalTimingUnavailableReason::ProgressedState)
    );
    assert_eq!(restored.zelda_audio_snapshot_bytes(), audio_before_reenable);
}

#[test]
fn paired_resume_requires_every_execution_continuation_to_be_idle() {
    let mut state = ZeldaState::new();
    assert!(state.paired_resume_cpu_boundary_is_quiescent());

    state
        .game_execution_scheduler
        .schedule_work(GameWorkContinuation::FinishAttractThroneRoom, 1);
    assert!(!state.paired_resume_cpu_boundary_is_quiescent());
    state.game_execution_scheduler.reset();

    state
        .game_execution_scheduler
        .schedule_pre_main_nmi_resume(PreMainNmiResume::OverworldAuxGraphicsReturn);
    assert!(!state.paired_resume_cpu_boundary_is_quiescent());
    state.game_execution_scheduler.reset();

    state
        .game_execution_scheduler
        .schedule_pre_main_caller_continuation(PreMainCallerContinuation::DialogueVwfReturn);
    assert!(!state.paired_resume_cpu_boundary_is_quiescent());
    state.game_execution_scheduler.reset();

    state
        .game_execution_scheduler
        .schedule_file_select_graphics();
    assert!(!state.paired_resume_cpu_boundary_is_quiescent());
    state.game_execution_scheduler.reset();

    state
        .game_execution_scheduler
        .schedule_selected_game_load(SelectedGameLoadDestination::Dungeon);
    assert!(!state.paired_resume_cpu_boundary_is_quiescent());
}

#[test]
fn paired_resume_rejects_every_runtime_only_cpu_schedule() {
    let mut state = ZeldaState::new();
    assert!(state.paired_resume_cpu_boundary_is_quiescent());

    state.dungeon_room_load_cpu_schedule = Some(DungeonRoomLoadCpuSchedule::default());
    assert!(!state.paired_resume_cpu_boundary_is_quiescent());
    state.dungeon_room_load_cpu_schedule = None;

    state.dungeon_submodule_cpu_schedule = Some(DungeonSubmoduleCpuSchedule::default());
    assert!(!state.paired_resume_cpu_boundary_is_quiescent());
    state.dungeon_submodule_cpu_schedule = None;

    state.module09_cpu_schedule = Some(Module09CpuSchedule::default());
    assert!(!state.paired_resume_cpu_boundary_is_quiescent());
    state.module09_cpu_schedule = None;

    state.intro_poly_thread_initialization_phase = 1;
    assert!(!state.paired_resume_cpu_boundary_is_quiescent());
    state.intro_poly_thread_initialization_phase = 0;

    state.pending_main_loop_common_suffix = Some(
        MainLoopCommonSuffixContinuation::ResumeSpritePreparationExtendedOamPackingAndClearNmiLatch {
            next_group_start: 4,
        },
    );
    assert!(!state.paired_resume_cpu_boundary_is_quiescent());
    state.pending_main_loop_common_suffix = None;

    assert!(state.paired_resume_cpu_boundary_is_quiescent());
}

#[test]
fn boulder_movement_checkpoints_resume_to_the_atomic_body() {
    use crate::SpriteMoveXYCheckpoint as C;
    for checkpoint in [
        C::BeforeMovement,
        C::AfterXSubpixel,
        C::AfterXLow,
        C::AfterXHigh,
        C::AfterYSubpixel,
        C::AfterYLow,
        C::AfterYHigh,
    ] {
        let mut state = ZeldaState::new();
        state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
        state.game_state.world.location.set_indoor_flag(1);
        state.sprite_slot_view_mut(12).set_sprite_type(0xc2);
        state.sprite_slot_view_mut(12).set_state(9);
        state.sprite_slot_view_mut(12).set_x_velocity(0xe4);
        state.sprite_slot_view_mut(12).set_y_velocity(0xe4);
        state.sprite_slot_view_mut(12).set_z_velocity(0xf0);
        state.sprite_slot_view_mut(12).set_x_low(0x2a);
        state.sprite_slot_view_mut(12).set_x_high(0x18);
        state.sprite_slot_view_mut(12).set_y_low(0x4a);
        state.sprite_slot_view_mut(12).set_y_high(0x0d);
        state.sprite_slot_view_mut(12).set_x_subpixel(0x80);
        let mut atomic = state.clone();
        atomic.sprite_c2_boulder(12);
        state.sprite_main_cpu_boundary = Some(SpriteMainCpuBoundary::BoulderMovement {
            slot: 12,
            checkpoint,
            continuation: None,
        });
        state.sprite_c2_boulder(12);
        let Some(SpriteMainCpuBoundary::BoulderMovement {
            continuation: Some(continuation),
            ..
        }) = state.sprite_main_cpu_boundary.take()
        else {
            panic!("Boulder movement boundary did not bind its continuation");
        };
        state.complete_boulder_movement(12, checkpoint, continuation);
        assert_eq!(state.ram, atomic.ram, "{checkpoint:?}");
        assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
    }
}

#[test]
fn initialize_prep_move_y_checkpoints_resume_to_the_atomic_body() {
    use crate::SpriteMoveXYCheckpoint as C;
    for sprite_type in [0x8au8, 0xd3] {
        for checkpoint in [
            C::BeforeMovement,
            C::AfterYSubpixel,
            C::AfterYLow,
            C::AfterYHigh,
        ] {
            let mut state = ZeldaState::new();
            state.game_state.world.location.set_indoor_flag(1);
            state.sprite_slot_view_mut(6).set_sprite_type(sprite_type);
            state.sprite_slot_view_mut(6).set_state(8);
            state.sprite_slot_view_mut(6).set_x_low(0x40);
            state.sprite_slot_view_mut(6).set_x_high(0x1a);
            state.sprite_slot_view_mut(6).set_y_low(0xa0);
            state.sprite_slot_view_mut(6).set_y_high(0x06);
            let mut atomic = state.clone();
            atomic.sprite_module_initialize(6);
            state.sprite_module_initialize_properties(6);
            let continuation = state.sprite_prep_move_y_through_checkpoint(6, checkpoint);
            state.sprite_prep_move_y_from_checkpoint(6, checkpoint, continuation);
            assert_eq!(state.ram, atomic.ram, "{sprite_type:#x} {checkpoint:?}");
            assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
        }
    }
}

#[test]
fn laser_eye_draw_prologue_checkpoint_resumes_to_the_atomic_body() {
    for (head_direction, ai_state) in [(16u8, 0u8), (0, 0), (16, 1)] {
        let mut state = ZeldaState::new();
        state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
        state.game_state.world.location.set_indoor_flag(1);
        state.sprite_slot_view_mut(8).set_sprite_type(0x97);
        state.sprite_slot_view_mut(8).set_state(9);
        state
            .sprite_slot_view_mut(8)
            .set_head_direction(head_direction);
        state.sprite_slot_view_mut(8).set_direction(2);
        state.sprite_slot_view_mut(8).set_ai_state(ai_state);
        state.sprite_slot_view_mut(8).set_delay_main(3);
        state.sprite_slot_view_mut(8).set_x_low(0xc8);
        state.sprite_slot_view_mut(8).set_x_high(0x0a);
        state.sprite_slot_view_mut(8).set_y_low(0x20);
        state.sprite_slot_view_mut(8).set_y_high(0x15);
        let mut atomic = state.clone();
        atomic.sprite_95_laser_eye_left(8);
        assert!(state.sprite_95_laser_eye_until_draw_prologue(8));
        state.sprite_95_laser_eye_after_draw_prologue(8);
        assert_eq!(state.ram, atomic.ram, "{head_direction} {ai_state}");
        assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
    }
}

#[test]
fn zora_fireball_movement_checkpoints_resume_to_the_atomic_body() {
    use crate::SpriteMoveXYCheckpoint as C;
    for checkpoint in [
        C::BeforeMovement,
        C::AfterXSubpixel,
        C::AfterXLow,
        C::AfterXHigh,
        C::AfterYSubpixel,
        C::AfterYLow,
        C::AfterYHigh,
    ] {
        let mut state = ZeldaState::new();
        state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
        state.game_state.world.location.set_indoor_flag(1);
        state.sprite_slot_view_mut(13).set_sprite_type(0x55);
        state.sprite_slot_view_mut(13).set_e(1);
        state.sprite_slot_view_mut(13).set_state(9);
        state.sprite_slot_view_mut(13).set_x_velocity(0xf4);
        state.sprite_slot_view_mut(13).set_y_velocity(0xe8);
        state.sprite_slot_view_mut(13).set_x_low(0x55);
        state.sprite_slot_view_mut(13).set_x_high(0x18);
        state.sprite_slot_view_mut(13).set_y_low(0x6a);
        state.sprite_slot_view_mut(13).set_y_high(0x0d);
        state.sprite_slot_view_mut(13).set_x_subpixel(0xc0);
        let mut atomic = state.clone();
        atomic.sprite_fireball(13);
        state.sprite_main_cpu_boundary = Some(SpriteMainCpuBoundary::ZoraFireballMovement {
            slot: 13,
            checkpoint,
            continuation: None,
        });
        state.sprite_fireball(13);
        let Some(SpriteMainCpuBoundary::ZoraFireballMovement {
            continuation: Some(continuation),
            ..
        }) = state.sprite_main_cpu_boundary.take()
        else {
            panic!("Zora fireball movement boundary did not bind its continuation");
        };
        state.complete_zora_fireball_movement(13, checkpoint, continuation);
        assert_eq!(state.ram, atomic.ram, "{checkpoint:?}");
        assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
    }
}

#[test]
fn helmasaur_hard_hat_tile_collision_checkpoints_resume_to_the_atomic_body() {
    use crate::SpriteTileCollisionStage as S;
    for (x_velocity, y_velocity) in [(0xfdu8, 0xffu8), (0x10, 0), (0, 0x10)] {
        for stage in [
            S::Entered,
            S::Cleared,
            S::VerticalProbeDone,
            S::ProbesCompleted,
        ] {
            let mut state = ZeldaState::new();
            state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
            state.sprite_slot_view_mut(0).set_sprite_type(0x26);
            state.sprite_slot_view_mut(0).set_state(9);
            state.sprite_slot_view_mut(0).set_x_velocity(x_velocity);
            state.sprite_slot_view_mut(0).set_y_velocity(y_velocity);
            state.sprite_slot_view_mut(0).set_x_low(0x1e);
            state.sprite_slot_view_mut(0).set_x_high(0x19);
            state.sprite_slot_view_mut(0).set_y_low(0x95);
            state.sprite_slot_view_mut(0).set_y_high(0x12);
            state.sprite_slot_view_mut(0).set_subtype2(0xd2);
            state.sprite_slot_view_mut(0).set_a(0x10);
            let mut atomic = state.clone();
            atomic.sprite_26_hardhat_beetle(0);
            state.begin_helmasaur_hard_hat_tile_collision_checkpoint(0, stage);
            state.resume_helmasaur_hard_hat_tile_collision(0, stage);
            assert_eq!(
                state.ram, atomic.ram,
                "{x_velocity:#x}/{y_velocity:#x} {stage:?}"
            );
            assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
        }
    }
}

#[test]
fn sidenexx_neck_target_checkpoints_resume_to_the_atomic_loop() {
    fn head_state(reached: bool) -> ZeldaState {
        let mut state = ZeldaState::new();
        state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
        state.sprite_slot_view_mut(0).set_a(100);
        state.sprite_slot_view_mut(0).set_c(100);
        state.sprite_slot_view_mut(1).set_sprite_type(0xcc);
        state.sprite_slot_view_mut(1).set_state(9);
        state.sprite_slot_view_mut(1).set_ai_state(2);
        state.sprite_slot_view_mut(1).set_direction(1);
        state.sprite_slot_view_mut(1).set_subtype2(9);
        for j in 9..18 {
            let (angle, radius) = if reached {
                (
                    crate::zelda_rtl::sprite_main_small_bosses::TRINEXX_SIDE_HEAD_X_TARGETS[j],
                    crate::zelda_rtl::sprite_main_small_bosses::TRINEXX_SIDE_HEAD_Y_TARGETS[j],
                )
            } else {
                (40 + j as u8, if j % 2 == 0 { 30 } else { 2 })
            };
            state.cached_sprite_slot_mut(j).set_type_byte(angle);
            state.cached_sprite_slot_mut(j).set_y_high(radius);
        }
        state
    }
    for step in 0..=54u8 {
        let mut state = head_state(false);
        let mut atomic = state.clone();
        atomic.sprite_sidenexx(1);
        let continuation = state.begin_sidenexx_neck_target_checkpoint(1, step);
        assert_eq!(state.sprite_slot_view(1).ai_state(), 2);
        state.resume_sidenexx_neck_target(1, step, continuation);
        assert_eq!(state.ram, atomic.ram, "step {step}");
        assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
    }
    for step in [0u8, 27, 54, 55] {
        let mut state = head_state(true);
        let mut atomic = state.clone();
        atomic.sprite_sidenexx(1);
        assert_eq!(atomic.sprite_slot_view(1).ai_state(), 1);
        let continuation = state.begin_sidenexx_neck_target_checkpoint(1, step);
        assert_eq!(
            state.sprite_slot_view(1).ai_state(),
            if step == 55 { 1 } else { 2 }
        );
        state.resume_sidenexx_neck_target(1, step, continuation);
        assert_eq!(state.ram, atomic.ram, "reached step {step}");
        assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
    }
}

#[test]
fn filtered_state_9_interrupts_the_caller_suffix_from_cpu_work() {
    assert_eq!(
        dungeon_supertile_state_9_cpu_advance(None),
        CpuPhaseSequenceAdvance::Complete,
    );
    assert!(matches!(
        dungeon_supertile_state_9_cpu_advance(Some(168_420)),
        CpuPhaseSequenceAdvance::ReachedBoundary {
            boundary: CpuRasterBoundary::VblankPublication,
            phase_index: 1,
            ..
        }
    ));

    let entry = FrameState {
        main_module: 7,
        submodule: 2,
        subsubmodule: 9,
        ..FrameState::default()
    };
    let exit = FrameState {
        subsubmodule: 10,
        ..entry
    };
    assert_eq!(
        dungeon_supertile_state_9_caller_continuation(entry, exit, Some(168_420)),
        Some(PreMainNmiResume::DungeonSupertileCallerReturnNmi),
    );
    assert_eq!(
        PreMainNmiResume::DungeonSupertileCallerReturnNmi.nmi_latch_clear_phase(),
        Some(NmiPhase::AfterNmi),
    );
    assert_eq!(
        dungeon_supertile_state_9_caller_continuation(entry, exit, None),
        None,
    );
}

#[test]
fn filtered_state_10_interrupts_the_quadrant_body_from_cpu_work() {
    assert!(matches!(
        dungeon_supertile_state_10_cpu_advance(187_620),
        CpuPhaseSequenceAdvance::ReachedBoundary {
            boundary: CpuRasterBoundary::VblankPublication,
            phase_index: 0,
            ..
        }
    ));
}

#[test]
fn absorbable_horizontal_lookup_keeps_movement_and_defers_bounce_suffix() {
    let mut state = ZeldaState::new();
    state.set_main_module(9);
    state.set_submodule(0);
    state.follower_link_state_mut().set_position(0x300, 0x300);
    state.garnish_state_mut().set_sprcoll_x_size(0x1000);
    state.garnish_state_mut().set_sprcoll_y_size(0x1000);
    {
        let mut sprite = state.sprite_slot_view_mut(0);
        sprite.set_state(9);
        sprite.set_sprite_type(0xd9);
        sprite.set_x(0x80);
        sprite.set_y(0x80);
        sprite.set_z(1);
        sprite.set_x_velocity(24);
        sprite.set_y_velocity(0xf0);
    }
    let mut atomic = state.clone();
    atomic.sprite_absorbable_main(0);
    state.sprite_main_cpu_boundary =
        Some(SpriteMainCpuBoundary::AbsorbableHorizontalTileLookup { slot: 0 });
    state.sprite_absorbable_main(0);
    assert_eq!(state.sprite_slot_view(0).x(), 0x81);
    assert_eq!(state.sprite_slot_view(0).x_subpixel(), 0x80);
    assert_eq!(state.sprite_slot_view(0).y(), 0x7f);
    assert_eq!(state.sprite_slot_view(0).z_velocity(), 0);
    state.sprite_main_cpu_boundary = None;
    state.sprite_absorbable_after_horizontal_lookup(0);
    assert_eq!(state.game_state, atomic.game_state);
    assert_eq!(state.ram, atomic.ram);
}

#[test]
fn absorbable_vertical_lookup_keeps_movement_and_defers_bounce_suffix() {
    let mut state = ZeldaState::new();
    state.set_main_module(9);
    state.set_submodule(0);
    state.follower_link_state_mut().set_position(0x300, 0x300);
    state.garnish_state_mut().set_sprcoll_x_size(0x1000);
    state.garnish_state_mut().set_sprcoll_y_size(0x1000);
    {
        let mut sprite = state.sprite_slot_view_mut(0);
        sprite.set_state(9);
        sprite.set_sprite_type(0xd9);
        sprite.set_x(0x80);
        sprite.set_y(0x80);
        sprite.set_z(1);
        sprite.set_x_velocity(24);
        sprite.set_y_velocity(0xf0);
    }
    let mut atomic = state.clone();
    atomic.sprite_absorbable_main(0);
    state.sprite_main_cpu_boundary =
        Some(SpriteMainCpuBoundary::AbsorbableVerticalTileLookup { slot: 0 });
    state.sprite_absorbable_main(0);
    assert_eq!(state.sprite_slot_view(0).x(), 0x81);
    assert_eq!(state.sprite_slot_view(0).x_subpixel(), 0x80);
    assert_eq!(state.sprite_slot_view(0).y(), 0x7f);
    assert_eq!(state.sprite_slot_view(0).z_velocity(), 0);
    state.sprite_main_cpu_boundary = None;
    state.sprite_absorbable_after_vertical_lookup(0);
    assert_eq!(state.game_state, atomic.game_state);
    assert_eq!(state.ram, atomic.ram);
}

#[test]
fn absorbable_loaded_attribute_keeps_movement_and_defers_bounce_suffix() {
    let mut state = ZeldaState::new();
    state.set_main_module(9);
    state.set_submodule(0);
    state.follower_link_state_mut().set_position(0x300, 0x300);
    state.garnish_state_mut().set_sprcoll_x_size(0x1000);
    state.garnish_state_mut().set_sprcoll_y_size(0x1000);
    {
        let mut sprite = state.sprite_slot_view_mut(0);
        sprite.set_state(9);
        sprite.set_sprite_type(0xd9);
        sprite.set_x(0x80);
        sprite.set_y(0x80);
        sprite.set_z(1);
        sprite.set_x_velocity(24);
        sprite.set_y_velocity(0xf0);
    }
    state.sprite_workspace_mut().set_tile_type(0x44);
    let mut atomic = state.clone();
    atomic.sprite_absorbable_main(0);
    state.sprite_main_cpu_boundary =
        Some(SpriteMainCpuBoundary::AbsorbableVerticalTileAttributeLoaded { slot: 0 });
    state.sprite_absorbable_main(0);
    assert_eq!(state.sprite_slot_view(0).x(), 0x81);
    assert_eq!(state.sprite_slot_view(0).x_subpixel(), 0x80);
    assert_eq!(state.sprite_slot_view(0).y(), 0x7f);
    assert_eq!(state.sprite_slot_view(0).z_velocity(), 0);
    assert_eq!(state.game_state.sprites.workspace.tile_type(), 0);
    state.sprite_main_cpu_boundary = None;
    state.sprite_absorbable_after_vertical_attribute_loaded(0);
    assert_eq!(state.game_state, atomic.game_state);
    assert_eq!(state.ram, atomic.ram);
}

#[test]
fn module_7_cpu_model_starts_before_translated_state_mutations() {
    assert_eq!(DUNGEON_PALETTE_CALLER_CPU_CHECKPOINT.entry_pc, 0x00_8051);
    assert_eq!(DUNGEON_PALETTE_CALLER_CPU_CHECKPOINT.stop_pc, 0x00_8036);
}

#[test]
fn scheduled_caller_progress_is_owned_by_its_interruption_timeline_once() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                crate::MainLoopInterruption::SpritePreparation,
            ),
        ],
    ));

    let timeline = state
        .take_original_timing_main_loop_interruption_timeline(None)
        .expect("the source interruption should own the host progress receipt");
    assert_eq!(
        state.take_original_timing_scheduled_caller_progress(Some(&timeline), None),
        Some(crate::MainLoopProgress::CallStackContinued),
    );
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn module10_opening_goal_completes_before_its_caller_returns() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_main_module(16);
    state.set_submodule(1);
    state.set_saved_module_for_menu(9);
    state.follower_link_state_mut().set_facing(2);
    state.set_spotlight_window_state(2);
    state.set_spotlight_window_radius(0x77);

    // In C, IrisSpotlight_ConfigureTable clears the module indices before
    // Spotlight_ConfigureTableAndControl immediately calls OpenSpotlight_Next2.
    let (caller_interrupted, reached_spotlight_goal) =
        state.spotlight_configure_table_and_control(false);

    assert!(!caller_interrupted);
    assert!(reached_spotlight_goal);
    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        0x7e
    );
    assert_eq!(state.game_state.frame.main_module, 9);
    assert_eq!(state.game_state.frame.submodule, 0x0a);
    assert_eq!(state.game_state.frame.subsubmodule, 0);
    assert!(!state.dungeon_landing_goal_transition_pending);
}

#[test]
fn animated_bg_uses_the_snapshot_owned_host_boundary_at_either_destination() {
    for destination in [0x3b00, 0x3c00] {
        let mut state = ZeldaState::new();
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

        assert_eq!(presented_word, 0x2222);
        assert_eq!(state.ppu.vram[destination], 0x3333);
    }
}

#[test]
fn live_main_loop_progress_is_consumed_once_without_counter_or_pc_provenance() {
    let progress = crate::MainLoopProgress::CallStackContinued;
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(progress)],
    ));

    assert_eq!(
        state.take_original_timing_main_loop_progress(),
        Some(progress)
    );
    assert_eq!(state.take_original_timing_main_loop_progress(), None);
}

#[test]
fn rom_timed_main_loop_observes_current_host_frame_input() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(20);
    state.attract_scene_mut().set_state(5);
    state.set_screen_brightness(15);
    state.set_bg_mode(9);

    state.run_frame_internal(0x0008, crate::RUN_MAIN);

    assert_eq!(state.game_state.ending.attract_scene.state(), 9);
    assert_eq!(state.game_state.display.screen_brightness, 14);
}

#[test]
fn name_player_terminal_caller_uses_the_same_source_return_order() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(4);
    state.set_submodule(1);
    state.set_subsubmodule(0);
    state.set_frame_counter(220);

    state.module_name_player_1();
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            1030,
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

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.frame_counter, 220);
    assert_eq!(state.game_state.frame.submodule, 2);
    assert_eq!(state.game_state.display.bg_vram_load_mode, 1);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.main_loop_sprite_preparation_completed);
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(state
        .game_execution_scheduler
        .pre_main_caller_continuation()
        .is_none());
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::Open),
    );
    assert!(!state.rom_load_partial_nmi_this_frame);
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(1030));
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn live_pre_main_caller_without_a_return_timeline_fails_before_caller_completion() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(1);
    state.set_submodule(2);
    state.set_subsubmodule(249);

    state.module_erase_file_1();
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            1031,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
            ],
        ))
        .unwrap();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.run_frame_internal(0, crate::RUN_MAIN);
    }));

    assert!(result.is_err());
    assert_eq!(state.game_state.frame.submodule, 2);
    assert_eq!(state.game_state.display.bg_vram_load_mode, 0);
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(state
        .pre_main_caller_continuation_is(PreMainCallerContinuation::FileSelectCheckerboardUpload));
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
}

#[test]
#[should_panic(expected = "cannot schedule")]
fn pre_main_scheduler_rejects_parallel_caller_suffixes() {
    let mut state = ZeldaState::new();
    state.schedule_pre_main_caller_continuation(PreMainCallerContinuation::DialogueVwfReturn);
    state.schedule_pre_main_caller_continuation(
        PreMainCallerContinuation::FileSelectCheckerboardUpload,
    );
}

#[test]
fn resumed_caller_does_not_relabel_future_link_dma_as_presented() {
    assert_eq!(
        link_obj_dma_generations_for_cpu_phase(true, GraphicsDmaGeneration::LiveAfterMain,),
        LinkObjDmaPhaseGenerations {
            presented: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            following_nmi: GraphicsDmaGeneration::LiveAfterMain,
        },
    );
    assert_eq!(
        link_obj_dma_generations_for_cpu_phase(false, GraphicsDmaGeneration::LiveAfterMain,),
        LinkObjDmaPhaseGenerations {
            presented: GraphicsDmaGeneration::LiveAfterMain,
            following_nmi: GraphicsDmaGeneration::LiveAfterMain,
        },
    );
}

#[test]
fn cached_antfairy_body_checkpoint_publishes_then_resumes_without_reincrementing() {
    let mut base = ZeldaState::new();
    base.set_indoor_flag(1);
    base.set_submodule(2);
    {
        let mut slot = base.sprite_slot_view_mut(1);
        slot.set_state(9);
        slot.set_sprite_type(0xd1);
        slot.set_ai_state(1);
        slot.set_subtype2(0x90);
        slot.set_delay_main(10);
        slot.set_x(0x0400);
        slot.set_y(0x0500);
    }
    base.dungeon_cache_trans_sprites();
    {
        let mut slot = base.sprite_slot_view_mut(1);
        slot.set_state(9);
        slot.set_sprite_type(0x6f);
        slot.set_subtype2(0x44);
        slot.set_x(0x0380);
        slot.set_y(0x0460);
    }

    let mut atomic = base.clone();
    atomic.execute_cached_sprites();

    let mut resumed = base;
    resumed.dungeon_cached_sprite_cpu_interruption_pending = Some(
        CachedSpriteCpuInterruption::ExecutingAntfairyAfterSubtype2Increment {
            slot: 1,
            continuation: None,
        },
    );
    resumed.dungeon_cached_sprite_cpu_interruption_boundary =
        Some(OriginalTimingBoundary::NmiAccepted);
    resumed.active_dungeon_sprite_main_return = Some(DungeonSpriteMainReturn {
        link_oam: None,
        bg2_x: 0,
        bg2_y: 0,
        bg1_x: 0,
        bg1_y: 0,
    });
    resumed.execute_cached_sprites();

    assert_eq!(resumed.sprite_slot_view(1).sprite_type(), 0xd1);
    assert_eq!(resumed.sprite_slot_view(1).subtype2(), 0x91);
    let (boundary, live_slot_backup) = match resumed
        .game_execution_scheduler
        .current_work()
        .expect("the cached body checkpoint must park its caller")
    {
        GameWorkContinuation::FinishDungeonCachedSpriteMain {
            boundary,
            live_slot_backup,
            ..
        } => (boundary, live_slot_backup),
        work => panic!("unexpected cached-sprite continuation: {work:?}"),
    };
    assert_eq!(
        boundary,
        CachedSpriteCpuInterruption::ExecutingAntfairyAfterSubtype2Increment {
            slot: 1,
            continuation: Some(AntfairyDrawContinuation::BunnyBeam),
        },
    );

    resumed.complete_cached_sprite_main_after_interrupted_slot(boundary, &live_slot_backup);

    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(resumed.game_state.sprites, atomic.game_state.sprites);
}

#[test]
fn interrupted_landing_fades_suspend_their_caller_return_before_state_fifteen_main() {
    for room in [0x21, 0x22, 0x41, 0x60] {
        let mut state = ZeldaState::new();
        state.rom_startup_timing = true;
        state.set_main_module(7);
        state.set_submodule(2);
        state.set_subsubmodule(14);
        state.set_dungeon_room_index(room);
        state.set_countdown_word(30);
        state.set_mosaic_target_level(31);
        state.dungeon_torch_mut().set_lights_out_request(1);
        state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
            phase: ModuleCpuPhase::InterruptedAfterModule,
            resumed_phase: None,
            submodule_nmi_slices: 0,
            subsubmodule: 15,
            palette_countdown: 0,
            sprite_main_boundary: None,
            cached_sprite_interruption: None,
        });

        state.Module07_02_FadedFilter();

        assert_eq!(state.game_state.frame.subsubmodule, 15, "room={room:#04x}");
        assert_eq!(
            state.game_state.display.palette_filter.countdown(),
            0,
            "room={room:#04x}"
        );
        assert_eq!(
            state.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::FadedFilterCallerReturn,
            }),
            "room={room:#04x}"
        );
        assert!(state
            .game_execution_scheduler
            .work_suspends_translated_call_stack());
        assert_eq!(
            state.dungeon_faded_filter_palette_completion_host_frame,
            Some(state.frame_ctr_dbg)
        );
        assert_eq!(
            state.next_display_obj_scanout_generation,
            Some(ObjScanoutGenerations {
                oam: OamScanoutSource::ComposeHostBoundaryShadowDma,
                link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            })
        );
    }
}

#[test]
fn completed_landing_fade_without_cpu_interruption_returns_atomically() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(14);
    state.set_dungeon_room_index(0x60);
    state.set_countdown_word(30);
    state.set_mosaic_target_level(31);
    state.dungeon_torch_mut().set_lights_out_request(1);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::CompleteBeforeNmi,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 15,
        palette_countdown: 0,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });

    state.Module07_02_FadedFilter();

    assert_eq!(state.game_state.frame.subsubmodule, 15);
    assert_eq!(state.game_state.display.palette_filter.countdown(), 0);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(
        state.dungeon_faded_filter_palette_completion_host_frame,
        Some(state.frame_ctr_dbg)
    );
    assert_eq!(state.dungeon_post_landing_leading_nmi_room, None);
}

#[test]
fn landing_fade_no_request_suspends_only_the_state_fifteen_caller_return() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(14);
    state.set_dungeon_room_index(0x41);
    state.dungeon_torch_mut().set_lights_out_request(0);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedAfterModule,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 15,
        palette_countdown: 0,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });

    state.Module07_02_FadedFilter();

    assert_eq!(state.game_state.frame.subsubmodule, 15);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::FadedFilterCallerReturn,
        })
    );
    assert!(state
        .game_execution_scheduler
        .work_suspends_translated_call_stack());

    let completion =
        GameWorkStep::Complete(GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::FadedFilterCallerReturn,
        });
    assert!(scheduled_work_completion_clears_nmi_latch_after_interrupt(
        completion
    ));
    assert_eq!(
        GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::FadedFilterCallerReturn,
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
fn c_enable_force_blank_requests_the_following_field_from_row_zero() {
    let mut state = ZeldaState::new();

    // zelda3/src/load_gfx.c:EnableForceBlank assigns INIDISP_copy=$80 for
    // WritePpuRegisters at the immediately following NMI. This is distinct
    // from an arbitrary INIDISP_copy assignment made after a field began.
    state.enable_force_blank();

    assert_eq!(state.active_display_force_blank_event, Some(0));
    assert!(live_forced_blank_for_scanout(
        false,
        None,
        state.active_display_force_blank_event,
        false,
    ));
}

#[test]
fn rom_title_fade_transition_resumes_without_another_main_loop_tick() {
    let mut state = ZeldaState::new();
    state.set_main_module(0);
    state.set_submodule(5);
    state.set_frame_counter(105);
    state.intro_zelda_fade_transition_pending = true;

    state.complete_intro_zelda_fade_transition();

    assert_eq!(state.game_state.frame.frame_counter, 105);
    assert_eq!(state.game_state.frame.submodule, 6);
    assert_eq!(state.game_state.frame.subsubmodule, 42);
    assert!(!state.intro_zelda_fade_transition_pending);
}

#[test]
fn frame_input_opposites_match_snes9x_libretro_report_order() {
    assert_eq!(ZeldaState::sanitize_frame_inputs(0x0130), 0x0120);
    assert_eq!(ZeldaState::sanitize_frame_inputs(0x00c0), 0x0080);
    assert_eq!(ZeldaState::sanitize_frame_inputs(0x00f0), 0x00a0);
}

#[test]
fn frame_input_opposite_resolution_preserves_non_direction_buttons() {
    assert_eq!(ZeldaState::sanitize_frame_inputs(0x0f30), 0x0f20);
}

#[test]
fn fire_debirando_property_load_resumes_from_its_source_store_cursor() {
    let mut split = ZeldaState::new();
    let slot = 0;
    {
        let mut sprite = split.sprite_slot_view_mut(slot);
        sprite.set_state(8);
        sprite.set_sprite_type(0x64);
        sprite.set_x_velocity(6);
        sprite.set_y_velocity(0xf0);
        sprite.set_head_direction(0x0e);
        sprite.set_ai_state(2);
    }
    let mut atomic = split.clone();

    split.sprite_module_initialize_properties(slot);
    split.sprite_slot_view_mut(slot).set_sprite_type(0x63);
    split.sprite_prep_reset_properties(slot);
    split.sprite_prep_load_properties_after_reset_prefix(slot, 9);
    {
        let sprite = split.sprite_slot_view(slot);
        assert_eq!(sprite.state(), 9);
        assert_eq!(sprite.sprite_type(), 0x63);
        assert_eq!(sprite.x_velocity(), 0);
        assert_eq!(sprite.y_velocity(), 0);
        assert_eq!(sprite.head_direction(), 0);
        assert_eq!(sprite.ai_state(), 0);
    }

    split.sprite_prep_load_properties_after_reset_from(slot, 9);
    split.sprite_prep_fire_debirando_after_property_reload(slot);
    atomic.sprite_module_initialize(slot);

    assert_eq!(
        split.game_state.sprites.sprite_slots, atomic.game_state.sprites.sprite_slots,
        "split execution must equal the atomic Fire Debirando initializer",
    );
}

#[test]
fn fire_debirando_spawn_boundary_resumes_without_replaying_its_prefix() {
    let mut split = ZeldaState::new();
    let slot = 0;
    {
        let mut sprite = split.sprite_slot_view_mut(slot);
        sprite.set_state(8);
        sprite.set_sprite_type(0x64);
        sprite.set_head_direction(0x0e);
        sprite.set_ai_state(2);
        sprite.set_delay_main(0xfb);
        sprite.set_graphics(5);
    }
    let mut atomic = split.clone();

    split.sprite_module_initialize_properties(slot);
    split.sprite_slot_view_mut(slot).set_sprite_type(0x63);
    split.sprite_prep_load_properties(slot);
    split.sprite_prep_fire_debirando_after_property_reload_before_spawn(slot);
    {
        let sprite = split.sprite_slot_view(slot);
        assert_eq!(sprite.state(), 9);
        assert_eq!(sprite.sprite_type(), 0x63);
        assert_eq!(sprite.head_direction(), 0);
        assert_eq!(sprite.ai_state(), 0);
        assert_eq!(sprite.delay_main(), 0);
        assert_eq!(sprite.graphics(), 6);
    }

    split.sprite_prep_debirando_pit_after_before_spawn(slot);
    atomic.sprite_module_initialize(slot);

    assert_eq!(
        split.game_state.sprites.sprite_slots, atomic.game_state.sprites.sprite_slots,
        "split execution must equal the atomic Fire Debirando initializer",
    );
}
