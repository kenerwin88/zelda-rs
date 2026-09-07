//! ZeldaState runtime tests — audio.
//! Split mechanically from the hub file; bodies unchanged.

use super::*;

#[test]
fn asset_pack_resolves_spc_driver_timing_program_by_name() {
    let pack = AssetPack::parse(&test_asset_pack_bytes(&[
        ("kUnused", vec![0]),
        ("kSpcDriverTimingProgram", vec![0x20, 0xcd, 0xcf, 0xbd]),
    ]))
    .unwrap();

    assert_eq!(
        pack.asset_by_name("kSpcDriverTimingProgram"),
        Some(&[0x20, 0xcd, 0xcf, 0xbd][..])
    );
    assert_eq!(pack.asset_by_name("missing"), None);
}

#[test]
fn live_audio_presentation_runs_native_shadow_then_publishes_exactly_once() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    let authority = vec![12_345, -12_345, 23_456, -23_456];
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_host_dispatch_active = true;
    state.original_timing_semantic_receipts = Some(
        OriginalTimingHostReceipts::new(0, 0, Vec::new())
            .with_presented_audio(crate::PresentedAudio::new(authority.clone()).unwrap()),
    );

    state.finish_original_timing_host_dispatch(true);
    assert_eq!(
        state.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            1,
            0,
            Vec::new(),
        )),
        Err(OriginalTimingReceiptInstallError::UnconsumedPresentedAudio),
        "a completed audio presentation cannot roll into a later host call",
    );

    let mut output = vec![0; authority.len()];
    state.zelda_render_audio(&mut output, 2, 2);
    assert_eq!(output, authority);
    let shadow = state
        .last_original_timing_audio_shadow_result()
        .expect("native renderer shadows every authoritative presentation");
    assert_eq!(shadow.sample_frames, 2);
    assert!(shadow.mismatched_interleaved_samples > 0);
    assert!(shadow.first_mismatch_interleaved.is_some());

    let mut next = vec![0; authority.len()];
    state.zelda_render_audio(&mut next, 2, 2);
    assert_ne!(next, authority, "the authority receipt must never replay");
    assert_eq!(state.last_original_timing_audio_shadow_result(), None);
}

#[test]
fn live_audio_presentation_shape_mismatch_fails_closed() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_host_dispatch_active = true;
    state.original_timing_semantic_receipts = Some(
        OriginalTimingHostReceipts::new(0, 0, Vec::new())
            .with_presented_audio(crate::PresentedAudio::new(vec![1, 2, 3, 4]).unwrap()),
    );
    state.finish_original_timing_host_dispatch(true);

    let mut mono = vec![0; 2];
    state.zelda_render_audio(&mut mono, 2, 1);

    assert_eq!(
        state.original_timing_owner(),
        OriginalTimingOwner::Unavailable(
            OriginalTimingUnavailableReason::AuthorityAudioShapeMismatch,
        ),
    );
    assert_eq!(state.last_original_timing_audio_shadow_result(), None);
}

#[test]
fn checkpoint_resume_restores_live_timing_without_rephasing_audio() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    let encoded = bincode::serialize(&state).expect("serialize ZeldaState checkpoint");
    let mut restored: ZeldaState =
        bincode::deserialize(&encoded).expect("deserialize ZeldaState checkpoint");

    assert!(!restored.rom_startup_timing());
    assert_eq!(
        restored.original_timing_owner(),
        OriginalTimingOwner::Unavailable(OriginalTimingUnavailableReason::CheckpointRestore)
    );
    let audio_before = restored.zelda_audio_snapshot_bytes();

    restored.restore_live_rom_timing_after_checkpoint();

    assert!(restored.rom_startup_timing());
    assert_eq!(
        restored.original_timing_owner(),
        OriginalTimingOwner::Unavailable(OriginalTimingUnavailableReason::CheckpointRestore)
    );
    assert_eq!(restored.zelda_audio_snapshot_bytes(), audio_before);
}

#[test]
fn dark_world_selected_load_waits_past_the_compatibility_audio_boundary() {
    let mut state = live_selected_game_load_state_before_pre_dungeon_audio();
    state.game_execution_scheduler.reset();
    state
        .game_execution_scheduler
        .schedule_selected_game_load(SelectedGameLoadDestination::DarkWorldOverworld);
    for _ in 1..SELECTED_GAME_LOAD_BEFORE_PRE_DUNGEON_AUDIO_NMI_SLICES {
        assert_eq!(
            state.game_execution_scheduler.advance_startup_sequence(),
            Some(StartupSequenceStep::SelectedGameLoadWaiting)
        );
    }
    let remaining = state
        .game_execution_scheduler
        .selected_game_load_remaining_nmi_slices();
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            422632,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
            ],
        ))
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.game_state.frame.main_module, 5);
    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_after_pre_dungeon_audio_sprite_reset(),
        None
    );
    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_remaining_nmi_slices(),
        remaining
    );
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_none_or(|receipts| receipts.semantic.is_empty()));
    for _ in 0..100 {
        assert_eq!(
            state
                .game_execution_scheduler
                .advance_selected_game_load_from_source(None, false),
            Some(StartupSequenceStep::SelectedGameLoadWaiting)
        );
    }
    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_remaining_nmi_slices(),
        remaining
    );
}
