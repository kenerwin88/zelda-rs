//! ZeldaState runtime tests — dialogue.
//! Split mechanically from the hub file; bodies unchanged.

use super::*;

#[test]
fn dialogue_ir_for_decoded_bytes_uses_runtime_dialogue_flags() {
    let state = ZeldaState::new();

    let ops = state.dialogue_ir_for_decoded_bytes(&[
        0,
        TEXT_COMMAND_START_US + TEXT_CMD_2,
        TEXT_COMMAND_START_US + TEXT_CMD_WAIT,
        3,
    ]);

    assert_eq!(ops[0].kind, DialogueIrKind::Glyph { code: 0 });
    assert_eq!(ops[1].kind, DialogueIrKind::Line { line: 2 });
    assert_eq!(ops[2].kind, DialogueIrKind::Wait { duration: 3 });
}

#[test]
fn current_dialogue_message_id_setter_updates_native_state_and_ram() {
    let mut state = ZeldaState::new();

    state.set_current_dialogue_message_id(0x00c8);

    assert_eq!(state.current_dialogue_message_id(), 0x00c8);
    assert_eq!(read_le_u16(&state.ram, DIALOGUE_MESSAGE_INDEX), 0x00c8);
}

#[test]
fn source_dialogue_ir_uses_authored_message_before_runtime_substitution() {
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 96];
    put_test_asset(
        &mut data,
        &mut ranges,
        95,
        dialogue_source_sidecar_asset(&[vec![
            0,
            TEXT_COMMAND_START_US + TEXT_CMD_NAME,
            TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE,
        ]]),
    );
    let mut state = ZeldaState::new();
    state.assets = Some(asset_pack_with_named_assets(
        data,
        ranges,
        &[(95, DIALOGUE_SOURCE_SIDECAR_ASSET_NAME)],
    ));
    state.messaging_text_mut().load_decoded_dialogue(&[
        0,
        1,
        TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE,
    ]);

    let source_ir = state.current_source_dialogue_ir();
    let rendered_ir = state.current_dialogue_ir();

    assert_eq!(state.current_dialogue_message_id(), 0);
    assert_eq!(source_ir[0].kind, DialogueIrKind::Glyph { code: 0 });
    assert_eq!(source_ir[1].kind, DialogueIrKind::PlayerName);
    assert_eq!(source_ir[2].kind, DialogueIrKind::EndMessage);
    assert_eq!(rendered_ir[0].kind, DialogueIrKind::Glyph { code: 0 });
    assert_eq!(rendered_ir[1].kind, DialogueIrKind::Glyph { code: 1 });
    assert_eq!(rendered_ir[2].kind, DialogueIrKind::EndMessage);
}

#[test]
fn source_dialogue_ir_requires_named_semantic_sidecar_asset() {
    let legacy_dictionary = pack_test_memblk_arrays(&[vec![]]);
    let legacy_messages =
        pack_test_memblk_arrays(&[vec![0, TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE]]);
    let legacy_asset = pack_test_memblk_arrays(&[legacy_dictionary, legacy_messages]);
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 96];
    put_test_asset(&mut data, &mut ranges, 94, legacy_asset);
    put_test_asset(
        &mut data,
        &mut ranges,
        95,
        dialogue_source_sidecar_asset(&[vec![1, TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE]]),
    );
    let mut state = ZeldaState::new();
    state.assets = Some(AssetPack::from_data_ranges(data, ranges));

    let source_ir = state.current_source_dialogue_ir();

    assert!(source_ir.is_empty());
}

#[test]
fn source_dialogue_ir_reads_named_semantic_sidecar_asset() {
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 96];
    put_test_asset(
        &mut data,
        &mut ranges,
        95,
        dialogue_source_sidecar_asset(&[vec![1, TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE]]),
    );
    let mut state = ZeldaState::new();
    state.assets = Some(asset_pack_with_named_assets(
        data,
        ranges,
        &[(95, DIALOGUE_SOURCE_SIDECAR_ASSET_NAME)],
    ));

    let source_ir = state.current_source_dialogue_ir();

    assert_eq!(source_ir[0].kind, DialogueIrKind::Glyph { code: 1 });
    assert_eq!(source_ir[1].kind, DialogueIrKind::EndMessage);
}

#[test]
fn deserialized_asset_pack_caches_dialogue_semantic_sidecar_after_first_read() {
    let sidecar =
        dialogue_source_sidecar_asset(&[vec![1, TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE]]);
    let bytes = test_asset_pack_bytes(&[(DIALOGUE_SOURCE_SIDECAR_ASSET_NAME, sidecar)]);
    let asset_pack = AssetPack::parse(&bytes).unwrap();
    let restored: AssetPack =
        bincode::deserialize(&bincode::serialize(&asset_pack).unwrap()).unwrap();

    assert!(restored.dialogue_source_ir_table.get().is_none());

    let source_ir = restored.source_dialogue_ir_for_message(0).unwrap();

    assert_eq!(source_ir[0].kind, DialogueIrKind::Glyph { code: 1 });
    assert_eq!(source_ir[1].kind, DialogueIrKind::EndMessage);
    assert!(restored.dialogue_source_ir_table.get().is_some());
}

#[test]
fn asset_pack_parse_requires_named_dialogue_semantic_sidecar_when_kdialogue_exists() {
    let err = AssetPack::parse(&test_asset_pack_bytes(&[("kDialogue", vec![0])]))
        .err()
        .unwrap();

    assert!(err.contains("missing required kDialogueSourceSemantic"));
}

#[test]
fn asset_pack_parse_rejects_malformed_named_dialogue_semantic_sidecar() {
    let err = AssetPack::parse(&test_asset_pack_bytes(&[
        ("kDialogue", vec![0]),
        ("kDialogueSourceSemantic", b"not semantic".to_vec()),
    ]))
    .err()
    .unwrap();

    assert!(err.contains("invalid semantic sidecar magic"));
}

#[test]
fn source_render_dialogue_ir_expands_runtime_number_commands() {
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 96];
    put_test_asset(
        &mut data,
        &mut ranges,
        95,
        dialogue_source_sidecar_asset(&[vec![
            TEXT_COMMAND_START_US + TEXT_CMD_NUMBER,
            1,
            TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE,
        ]]),
    );
    let mut state = ZeldaState::new();
    state.assets = Some(asset_pack_with_named_assets(
        data,
        ranges,
        &[(95, DIALOGUE_SOURCE_SIDECAR_ASSET_NAME)],
    ));
    state
        .game_state
        .messaging
        .dialogue_number
        .set_packed_digits(0x42, 0);

    let source_ir = state.current_source_dialogue_ir();
    let render_ir = state.current_source_render_dialogue_ir();

    assert_eq!(source_ir[0].kind, DialogueIrKind::Number { slot: 1 });
    assert_eq!(render_ir[0].kind, DialogueIrKind::Glyph { code: 0x38 });
    assert_eq!(render_ir[1].kind, DialogueIrKind::EndMessage);
}

#[test]
fn visible_source_render_dialogue_ir_tracks_live_read_position() {
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 96];
    put_test_asset(
        &mut data,
        &mut ranges,
        95,
        dialogue_source_sidecar_asset(&[vec![
            TEXT_COMMAND_START_US + TEXT_CMD_COLOR,
            2,
            0,
            1,
            TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE,
        ]]),
    );
    let mut state = ZeldaState::new();
    state.assets = Some(asset_pack_with_named_assets(
        data,
        ranges,
        &[(95, DIALOGUE_SOURCE_SIDECAR_ASSET_NAME)],
    ));
    state.messaging_text_mut().load_decoded_dialogue(&[
        0,
        1,
        TEXT_COMMAND_START_US + TEXT_CMD_END_MESSAGE,
    ]);
    state.messaging_state_mut().set_dialogue_msg_read_pos(1);

    let render_ir = state.current_visible_source_render_dialogue_ir();
    let visible_kinds = render_ir.iter().map(|op| &op.kind).collect::<Vec<_>>();

    assert_eq!(
        visible_kinds,
        vec![
            &DialogueIrKind::Color { color: 2 },
            &DialogueIrKind::Glyph { code: 0 },
        ]
    );
    assert_eq!(
        render_ir.iter().map(|op| op.offset).collect::<Vec<_>>(),
        vec![0, 0]
    );
}

#[test]
fn live_dialogue_close_uses_the_native_saved_module_transition() {
    let mut state = ZeldaState::new();
    state.set_main_module(14);
    state.set_submodule(2);
    state.frame_state_mut().set_saved_module_for_menu(9);
    state.messaging_state_mut().set_module(1);
    state.messaging_state_mut().set_text_render_state(3);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::DialogueClosed,
        ],
    ));
    state.game_execution_scheduler.begin_host_frame();

    state.zelda_run_game_loop();

    assert_eq!(state.game_state.frame.main_module, 9);
    assert_eq!(state.game_state.frame.submodule, 0);
    assert_eq!(state.game_state.messaging.runtime.module(), 0);
    assert_eq!(state.game_state.messaging.runtime.text_render_state(), 4);
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn live_dialogue_resume_receipt_suppresses_only_an_already_completed_native_prefix() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(14);
    state.messaging_state_mut().set_dialogue_msg_read_pos(0x37);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::DialogueExecutionProgress(
            crate::DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                message_read_position: 0x37,
            },
        )],
    ));

    assert_eq!(
        state.take_original_timing_dialogue_message_endpoint(),
        Some(0x37),
    );

    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));

    // A semantic endpoint the native owner has not reached remains shadow
    // evidence only; it cannot force or overwrite translated state.
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        1,
        0,
        vec![OriginalTimingSemanticReceipt::DialogueExecutionProgress(
            crate::DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                message_read_position: 0x39,
            },
        )],
    ));
    assert_eq!(
        state.take_original_timing_dialogue_message_endpoint(),
        Some(0x39),
    );
}

#[test]
fn continued_dialogue_endpoints_preserve_one_common_suffix_until_terminal_return() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(14);
    state.set_submodule(2);
    state.messaging_state_mut().set_module(1);
    state.messaging_state_mut().set_text_render_state(3);
    state.messaging_state_mut().set_dialogue_msg_read_pos(0x37);

    for host in [2331, 2332, 2333] {
        state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
            host,
            0,
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
        ));

        assert_eq!(
            state.original_timing_dialogue_message_endpoint(),
            Some(0x37)
        );

        state.zelda_run_game_loop_body(Some(crate::MainLoopProgress::CallStackContinued));

        assert_eq!(
            state.pending_main_loop_common_suffix,
            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
            "each nonterminal VWF host must retain the same one-shot suffix owner",
        );
        assert_eq!(
            state
                .original_timing_semantic_receipts
                .as_ref()
                .unwrap()
                .semantic(),
            &[OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            )],
            "only the dialogue endpoint belongs to the resumed native caller",
        );
    }

    state.main_loop_sprite_preparation_completed = true;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        2333,
        0,
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
    ));
    let receipts_before = state.original_timing_semantic_receipts.clone();
    let suffix_before = state.pending_main_loop_common_suffix;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.zelda_run_game_loop_body(Some(crate::MainLoopProgress::CallStackContinued));
    }));
    assert!(result.is_err());
    assert_eq!(state.original_timing_semantic_receipts, receipts_before);
    assert_eq!(state.pending_main_loop_common_suffix, suffix_before);
}

#[test]
fn carried_dialogue_handler_completes_before_terminal_caller_and_suffix_without_recapture() {
    let mut state = live_dialogue_terminal_return_state(0x37);
    state
        .install_original_timing_host_receipts(dialogue_terminal_return_receipts(2334))
        .unwrap();

    // The acceptance host presents before the carried handler returns.
    state.with_display_snapshot(|_| ());
    state.advance_display_publication_history();
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
    let acceptance_host_vram = state.display_snapshot.as_ref().unwrap().ppu.vram.clone();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(
        state.display_snapshot.as_ref().unwrap().ppu.vram,
        acceptance_host_vram,
        "the latch-held handler cannot publish core VRAM DMA into its acceptance-host scanout",
    );
    assert_eq!(
        state.display_snapshot.as_ref().unwrap().ppu.vram[0x5a20],
        0x1357,
        "the carry-in handler must refine the acceptance-host scanout instead of recapturing live state",
    );
    assert_eq!(state.ppu.vram[0x5a20], 0x2468);
    assert_eq!(
        state.game_state.player.follower_link.joypad1h_last(),
        0xa5,
        "a latch-held handler must not publish controller input",
    );
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 2);
    assert_eq!(state.game_state.display.core_update_disable_flag, 2);
    assert_eq!(
        (
            state.game_state.display.ppu_scroll_copy.bg2_h_copy(),
            state.game_state.display.ppu_scroll_copy.bg2_v_copy(),
            state.game_state.display.ppu_scroll_copy.bg1_h_copy(),
            state.game_state.display.ppu_scroll_copy.bg1_v_copy(),
        ),
        (0x1013, 0x2025, 0x3033, 0x4045),
        "the resumed Module0E caller must publish its exact scroll-copy suffix",
    );
    assert_eq!(
        state.game_state.messaging.runtime.dialogue_msg_read_pos(),
        0,
        "the resumed Text_LoadCharacterBuffer caller must clear its carry cursor",
    );
    assert_eq!(
        state.game_state.system_signals.last_ambient_sound_effect(),
        3,
        "the host audio sampler must consume the command once; the held handler cannot replay it",
    );
    assert!(state.game_execution_scheduler.is_idle());
    assert!(state
        .game_execution_scheduler
        .returned_main_is_waiting_for_nmi());
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn terminal_dialogue_return_accepts_all_four_source_nmi_ownership_cells() {
    for publication_pending_at_entry in [false, true] {
        for trailing_open_acceptance in [false, true] {
            let mut state = live_dialogue_terminal_return_state(0x37);
            if !publication_pending_at_entry {
                state.original_timing_nmi_publication_pending = false;
                state.original_timing_pending_nmi_update_gate = None;
                state
                    .display_snapshot
                    .as_mut()
                    .unwrap()
                    .accepts_nmi_dma_receipts = false;
            }
            let entry_epoch = state.display_snapshot_epoch;
            state
                .install_original_timing_host_receipts(
                    dialogue_terminal_return_receipts_for_nmi_cell(
                        3812,
                        publication_pending_at_entry,
                        trailing_open_acceptance,
                    ),
                )
                .unwrap();

            state.run_frame_internal(0, crate::RUN_MAIN);

            assert_eq!(
                state.display_snapshot_epoch - entry_epoch,
                u64::from(!publication_pending_at_entry) + u64::from(trailing_open_acceptance),
                "each source acceptance must own exactly one display capture",
            );
            assert_eq!(
                state.original_timing_nmi_publication_pending,
                trailing_open_acceptance,
            );
            assert_eq!(
                state.original_timing_pending_nmi_update_gate,
                trailing_open_acceptance.then_some(NmiUpdateGate::Open),
            );
            assert!(state.original_timing_expected_nmi_update_gates.is_empty());
            assert!(state.pending_main_loop_common_suffix.is_none());
            assert!(state.main_loop_sprite_preparation_completed);
            assert!(!state.game_state.display.nmi_update_is_latched());
            assert_eq!(state.game_state.player.follower_link.joypad1h_last(), 0xa5);
            assert_eq!(
                state.game_state.messaging.runtime.dialogue_msg_read_pos(),
                0,
                "the terminal CPU closure must run exactly once for every NMI ownership cell",
            );
            assert!(state.game_execution_scheduler.is_idle());
            assert!(state
                .game_execution_scheduler
                .returned_main_is_waiting_for_nmi());
            assert!(state.original_timing_semantic_receipts.is_none());
            if trailing_open_acceptance {
                assert!(state
                    .display_snapshot
                    .as_ref()
                    .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
            }
        }
    }
}

#[test]
fn poly_thread_dialogue_after_current_waits_for_the_wire_terminal_return() {
    let mut state = live_dialogue_terminal_return_state(0x37);
    let endpoint_before = state.game_state.messaging.runtime.dialogue_msg_read_pos();
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            414029,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
            ],
        ))
        .unwrap();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDialogueInitializationCallerReturn),
        "the source continued-call host must retain the interrupted dialogue caller",
    );
    assert_eq!(
        state.game_state.messaging.runtime.dialogue_msg_read_pos(),
        endpoint_before,
        "a nonterminal host cannot execute the caller-return CPU suffix",
    );
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn idle_dialogue_iteration_after_terminal_initializer_uses_ordinary_main_loop_owner() {
    let mut state = live_dialogue_terminal_return_state(0x37);
    state.set_frame_counter(143);
    state
        .install_original_timing_host_receipts(dialogue_terminal_return_receipts(2334))
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert!(state
        .game_execution_scheduler
        .returned_main_is_waiting_for_nmi());
    assert_eq!(state.game_execution_scheduler.current_work(), None);
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(!state.game_state.display.nmi_update_is_latched());

    state
        .install_original_timing_host_receipts(ordinary_dialogue_iteration_receipts(2335))
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(
        state.game_state.frame.frame_counter, 144,
        "the ordinary host must begin exactly one fresh ZeldaRunGameLoop iteration",
    );
    assert_eq!(state.game_state.player.follower_link.joypad1h_last(), 0x40);
    assert_eq!(state.game_state.player.follower_link.joypad1l_last(), 0x02);
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::Open),
    );
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        None,
        "a fresh dialogue iteration must not replay the retired text-initializer caller",
    );
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn run3818_terminal_dialogue_endpoint_resumes_before_the_one_common_suffix() {
    let endpoint = 25;
    let mut state = live_idle_terminal_dialogue_endpoint_state(endpoint);
    state.messaging_state_mut().set_dialogue_msg_read_pos(18);
    state.messaging_text_mut().load_decoded_dialogue(&[0; 26]);

    let mut endpoint_probe = state.clone();
    let transition =
        endpoint_probe.advance_suspended_vwf_to_authoritative_endpoint(endpoint, false);
    assert_eq!(transition.slice_count(), 1);
    assert_eq!(
        endpoint_probe
            .game_state
            .messaging
            .runtime
            .dialogue_msg_read_pos(),
        endpoint,
    );
    assert!(endpoint_probe.dialogue_fast_forward_hold_pending);
    assert_eq!(
        endpoint_probe.dialogue_live_message_read_position_target,
        Some(endpoint),
    );

    // Run3817's source endpoint advances the suspended native VWF caller from
    // translated cursor 18 through source cursor 25, but does not yet reach
    // the character epilogue, Module0E scroll suffix, or $805f.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            3_817,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                    crate::DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                        message_read_position: endpoint,
                    },
                ),
            ],
        ))
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert_eq!(
        state.game_state.messaging.runtime.dialogue_msg_read_pos(),
        endpoint,
    );
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(state.dialogue_fast_forward_hold_active);
    assert!(!state.dialogue_fast_forward_hold_pending);
    assert_eq!(state.dialogue_live_message_read_position_target, None);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0);
    assert_eq!(state.game_state.display.core_update_disable_flag, 0);

    let mut terminal_endpoint_probe = state.clone();
    let terminal_transition =
        terminal_endpoint_probe.advance_suspended_vwf_to_authoritative_endpoint(endpoint, false);
    assert_eq!(
        terminal_transition.slice_count(),
        0,
        "the terminal repeated endpoint must not replay native VWF work",
    );

    // Run3818 repeats endpoint 25, then returns through $805f. Native VWF is
    // already caught up, so this host owns only the character epilogue,
    // Module0E scroll suffix, post-module bookkeeping, and common suffix.
    let frame_counter = state.game_state.frame.frame_counter;
    let epoch_before_terminal = state.display_snapshot_epoch;
    state
        .install_original_timing_host_receipts(idle_terminal_dialogue_endpoint_receipts(
            3_818, endpoint,
        ))
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.frame_counter, frame_counter);
    assert_eq!(
        state.game_state.messaging.runtime.dialogue_msg_read_pos(),
        endpoint,
        "the repeated endpoint cannot replay or overwrite the native VWF cursor",
    );
    assert_eq!(
        (
            state.game_state.display.ppu_scroll_copy.bg2_h_copy(),
            state.game_state.display.ppu_scroll_copy.bg2_v_copy(),
            state.game_state.display.ppu_scroll_copy.bg1_h_copy(),
            state.game_state.display.ppu_scroll_copy.bg1_v_copy(),
        ),
        (0x1013, 0x2025, 0x3033, 0x4045),
        "the terminal endpoint must execute Module0E's post-RunInterface scroll suffix before $805f",
    );
    assert_eq!(state.display_snapshot_epoch, epoch_before_terminal + 1);
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(state.main_loop_sprite_preparation_completed);
    assert!(!state.dialogue_fast_forward_hold_active);
    assert!(!state.dialogue_fast_forward_hold_pending);
    assert_eq!(state.dialogue_live_message_read_position_target, None);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 2);
    assert_eq!(state.game_state.display.core_update_disable_flag, 2);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert!(state.original_timing_semantic_receipts.is_none());
    assert!(state.game_execution_scheduler.is_idle());
}

#[test]
fn run4652_terminal_dialogue_endpoint_carries_only_the_post_suffix_open_nmi() {
    // The preserved raw host pins this lifecycle but not the receipt payload;
    // use one valid resident endpoint to exercise the generic typed claim.
    let endpoint = 25;
    let mut state = live_idle_terminal_dialogue_endpoint_state(endpoint);
    state.messaging_text_mut().load_decoded_dialogue(&[0; 26]);
    let epoch_before_terminal = state.display_snapshot_epoch;

    // Run4652 completes its same-host Held handler, returns through the
    // character and Module0E postludes, clears $12 at $805f, then accepts one
    // Open NMI before HostFrameWindow appends the endpoint receipt.
    state
        .install_original_timing_host_receipts(
            idle_terminal_dialogue_endpoint_with_trailing_open_receipts(4_652, endpoint),
        )
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.display_snapshot_epoch, epoch_before_terminal + 2);
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
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 2);
    assert_eq!(state.game_state.display.core_update_disable_flag, 2);
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert!(state.original_timing_semantic_receipts.is_none());

    // The next host completes the carried Open handler on run4652's receptive
    // capture, then accepts one new Held NMI after Sprite_Main. Only that new
    // acceptance may advance the display epoch.
    let epoch_before_completion = state.display_snapshot_epoch;
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            4_653,
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
fn run3822_endpoint_then_run3823_dless_terminal_retires_vwf_before_run3824_sprite_main() {
    let endpoint = 25;
    let mut state = live_idle_terminal_dialogue_endpoint_state(endpoint);
    state.messaging_text_mut().load_decoded_dialogue(&[0; 27]);
    state.messaging_state_mut().set_vwf_line_speed_cur(3);
    state.dialogue_fast_forward_hold_active = true;

    // Run3822 reaches the already-resident endpoint, then accepts the Held
    // NMI whose handler is carried into the D-less terminal host.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            3_822,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                    crate::DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                        message_read_position: endpoint,
                    },
                ),
            ],
        ))
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert!(state.dialogue_fast_forward_hold_active);
    assert!(!state.dialogue_fast_forward_hold_pending);
    assert_eq!(state.dialogue_live_message_read_position_target, None);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0);
    assert_eq!(state.game_state.display.core_update_disable_flag, 0);

    let scroll_before = (
        state.game_state.display.ppu_scroll_copy.bg2_h_copy(),
        state.game_state.display.ppu_scroll_copy.bg2_v_copy(),
        state.game_state.display.ppu_scroll_copy.bg1_h_copy(),
        state.game_state.display.ppu_scroll_copy.bg1_v_copy(),
    );
    let frame_counter = state.game_state.frame.frame_counter;
    let mut completion_probe = state.clone();
    let completion = completion_probe.advance_suspended_vwf_to_handler_completion();
    assert_eq!(completion.slice_count(), 1);

    // Run3823 publishes no repeated D receipt. Its carried Held completion and
    // typed common suffix close the VWF caller represented by run3822.
    state
        .install_original_timing_host_receipts(idle_terminal_suspended_vwf_receipts(3_823))
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.game_state.frame.frame_counter, frame_counter);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 2);
    assert_eq!(state.game_state.display.core_update_disable_flag, 2);
    assert!(!state.dialogue_fast_forward_hold_active);
    assert!(!state.dialogue_fast_forward_hold_pending);
    assert_eq!(state.dialogue_live_message_read_position_target, None);
    assert_ne!(
        (
            state.game_state.display.ppu_scroll_copy.bg2_h_copy(),
            state.game_state.display.ppu_scroll_copy.bg2_v_copy(),
            state.game_state.display.ppu_scroll_copy.bg1_h_copy(),
            state.game_state.display.ppu_scroll_copy.bg1_v_copy(),
        ),
        scroll_before,
        "the D-less terminal must execute Module0E's scroll-register postlude",
    );
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(state.main_loop_sprite_preparation_completed);
    assert!(!state.original_timing_nmi_publication_pending);
    assert!(state.original_timing_semantic_receipts.is_none());

    // Run3824 begins a fresh Module0E iteration. Retiring the cross-host hold
    // in run3823 is what permits its source Sprite_Main return to be consumed.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            3_824,
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
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(
        state.original_timing_sprite_main_return_claims_remaining,
        None,
    );
    assert!(state.original_timing_semantic_receipts.is_none());
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
}

#[test]
fn source_dialogue_endpoint_folds_multiple_translated_vwf_slices_without_publishing_callers() {
    let endpoint = 19;
    let mut state = live_idle_terminal_dialogue_endpoint_state(endpoint);
    state.messaging_state_mut().set_dialogue_msg_read_pos(18);
    state.messaging_text_mut().load_decoded_dialogue(&[0; 20]);
    // One more cycle than a complete resumed NTSC frame forces the first
    // translated raster-budget slice to remain inside the same glyph. The D19
    // receipt supersedes that replaceable timing shadow and owns both slices.
    state.dialogue_vwf_glyph_cpu_phase = messaging::VwfGlyphCpuPhase::Drawing {
        remaining_master_cycles: 262 * 341 * 4 + 1,
    };
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            3_817,
            0,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                    crate::DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                        message_read_position: endpoint,
                    },
                ),
            ],
        ))
        .unwrap();

    let mut probe = state.clone();
    let transition = probe.advance_suspended_vwf_to_authoritative_endpoint(endpoint, false);
    assert_eq!(transition.slice_count(), 2);

    let scroll_before = (
        state.game_state.display.ppu_scroll_copy.bg2_h_copy(),
        state.game_state.display.ppu_scroll_copy.bg2_v_copy(),
        state.game_state.display.ppu_scroll_copy.bg1_h_copy(),
        state.game_state.display.ppu_scroll_copy.bg1_v_copy(),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(
        state.game_state.messaging.runtime.dialogue_msg_read_pos(),
        endpoint,
    );
    assert_eq!(
        (
            state.game_state.display.ppu_scroll_copy.bg2_h_copy(),
            state.game_state.display.ppu_scroll_copy.bg2_v_copy(),
            state.game_state.display.ppu_scroll_copy.bg1_h_copy(),
            state.game_state.display.ppu_scroll_copy.bg1_v_copy(),
        ),
        scroll_before,
        "a nonterminal endpoint cannot publish Module0E's scroll-register suffix",
    );
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0);
    assert_eq!(state.game_state.display.core_update_disable_flag, 0);
    assert!(state.dialogue_fast_forward_hold_active);
    assert!(!state.dialogue_fast_forward_hold_pending);
    assert_eq!(state.dialogue_live_message_read_position_target, None);
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert!(state.original_timing_semantic_receipts.is_none());
    assert!(state.game_execution_scheduler.is_idle());
}

#[test]
fn idle_terminal_continued_return_without_dialogue_keeps_the_generic_owner() {
    let mut state = live_idle_terminal_dialogue_endpoint_state(25);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            3_818,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            ],
        ))
        .unwrap();

    let scroll_before = (
        state.game_state.display.ppu_scroll_copy.bg2_h_copy(),
        state.game_state.display.ppu_scroll_copy.bg2_v_copy(),
        state.game_state.display.ppu_scroll_copy.bg1_h_copy(),
        state.game_state.display.ppu_scroll_copy.bg1_v_copy(),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(
        (
            state.game_state.display.ppu_scroll_copy.bg2_h_copy(),
            state.game_state.display.ppu_scroll_copy.bg2_v_copy(),
            state.game_state.display.ppu_scroll_copy.bg1_h_copy(),
            state.game_state.display.ppu_scroll_copy.bg1_v_copy(),
        ),
        scroll_before,
        "the generic non-dialogue terminal owner must retain its empty CPU closure",
    );
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(state.main_loop_sprite_preparation_completed);
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn carried_terminal_return_without_a_suspended_vwf_keeps_the_empty_cpu_closure() {
    let mut state = live_idle_terminal_suspended_vwf_state();
    state.dialogue_fast_forward_hold_active = false;
    let scroll_before = (
        state.game_state.display.ppu_scroll_copy.bg2_h_copy(),
        state.game_state.display.ppu_scroll_copy.bg2_v_copy(),
        state.game_state.display.ppu_scroll_copy.bg1_h_copy(),
        state.game_state.display.ppu_scroll_copy.bg1_v_copy(),
    );

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(
        (
            state.game_state.display.ppu_scroll_copy.bg2_h_copy(),
            state.game_state.display.ppu_scroll_copy.bg2_v_copy(),
            state.game_state.display.ppu_scroll_copy.bg1_h_copy(),
            state.game_state.display.ppu_scroll_copy.bg1_v_copy(),
        ),
        scroll_before,
        "a generic carried terminal return cannot execute the VWF postlude",
    );
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0);
    assert_eq!(state.game_state.display.core_update_disable_flag, 0);
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn dless_suspended_vwf_terminal_preflight_is_failure_atomic() {
    fn assert_rejected(mut state: ZeldaState) {
        let scheduler_before = state.game_execution_scheduler;
        let ram_before = state.ram.clone();
        let receipts_before = state.original_timing_semantic_receipts.clone();
        let gates_before = state.original_timing_expected_nmi_update_gates.clone();
        let suffix_before = state.pending_main_loop_common_suffix;
        let sprite_preparation_before = state.main_loop_sprite_preparation_completed;
        let publication_before = (
            state.original_timing_nmi_publication_pending,
            state.original_timing_pending_nmi_update_gate,
        );
        let snapshot_before = state.display_snapshot.as_ref().map(|snapshot| {
            (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.oam.clone(),
                snapshot.ppu.vram.clone(),
                snapshot.ppu.cgram.clone(),
            )
        });
        let epoch_before = state.display_snapshot_epoch;
        let live_ppu_before = (
            state.ppu.oam.clone(),
            state.ppu.vram.clone(),
            state.ppu.cgram.clone(),
        );
        let latch_before = state.game_state.display.nmi_update_is_latched();
        let audio_before = state.audio_nmi_processed_before_main;
        let hold_before = (
            state.dialogue_fast_forward_hold_active,
            state.dialogue_fast_forward_hold_pending,
            state.dialogue_live_message_read_position_target,
            state.dialogue_vwf_glyph_cpu_phase,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal(0, crate::RUN_MAIN);
        }));
        assert!(result.is_err());
        assert_eq!(state.game_execution_scheduler, scheduler_before);
        assert_eq!(state.ram, ram_before);
        assert_eq!(state.original_timing_semantic_receipts, receipts_before);
        assert_eq!(
            state.original_timing_expected_nmi_update_gates,
            gates_before
        );
        assert_eq!(state.pending_main_loop_common_suffix, suffix_before);
        assert_eq!(
            state.main_loop_sprite_preparation_completed,
            sprite_preparation_before,
        );
        assert_eq!(
            (
                state.original_timing_nmi_publication_pending,
                state.original_timing_pending_nmi_update_gate,
            ),
            publication_before,
        );
        assert_eq!(
            state.display_snapshot.as_ref().map(|snapshot| (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.oam.clone(),
                snapshot.ppu.vram.clone(),
                snapshot.ppu.cgram.clone(),
            )),
            snapshot_before,
        );
        assert_eq!(state.display_snapshot_epoch, epoch_before);
        assert_eq!(
            (
                state.ppu.oam.clone(),
                state.ppu.vram.clone(),
                state.ppu.cgram.clone(),
            ),
            live_ppu_before,
        );
        assert_eq!(
            state.game_state.display.nmi_update_is_latched(),
            latch_before
        );
        assert_eq!(state.audio_nmi_processed_before_main, audio_before);
        assert_eq!(
            (
                state.dialogue_fast_forward_hold_active,
                state.dialogue_fast_forward_hold_pending,
                state.dialogue_live_message_read_position_target,
                state.dialogue_vwf_glyph_cpu_phase,
            ),
            hold_before,
        );
    }

    // A fast-forward hold paired with the resumed extended-OAM-packing
    // suffix is no longer malformed: the run-67132 calibration proved that
    // lifecycle at host 7323 (the interrupted suffix resumes and returns
    // while the source keeps rendering at its own endpoint pace), and the
    // suffix variant now routes such a host to the ordinary continued-return
    // action instead of the D-less VWF completion.

    let mut missing_suffix = live_idle_terminal_suspended_vwf_state();
    missing_suffix.pending_main_loop_common_suffix = None;
    assert_rejected(missing_suffix);

    let mut wrong_module = live_idle_terminal_suspended_vwf_state();
    wrong_module.set_main_module(13);
    assert_rejected(wrong_module);

    let mut wrong_render_state = live_idle_terminal_suspended_vwf_state();
    wrong_render_state
        .messaging_state_mut()
        .set_text_render_state(2);
    assert_rejected(wrong_render_state);

    let mut stale_target = live_idle_terminal_suspended_vwf_state();
    stale_target.dialogue_live_message_read_position_target = Some(25);
    assert_rejected(stale_target);

    let mut pending_hold = live_idle_terminal_suspended_vwf_state();
    pending_hold.dialogue_fast_forward_hold_pending = true;
    assert_rejected(pending_hold);

    let mut wrong_gate = live_idle_terminal_suspended_vwf_state();
    wrong_gate.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
    wrong_gate.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
    assert_rejected(wrong_gate);

    let mut wrong_native_latch = live_idle_terminal_suspended_vwf_state();
    wrong_native_latch.clear_nmi_update_latch();
    assert_rejected(wrong_native_latch);

    let mut nonreceptive_snapshot = live_idle_terminal_suspended_vwf_state();
    nonreceptive_snapshot
        .display_snapshot
        .as_mut()
        .unwrap()
        .accepts_nmi_dma_receipts = false;
    assert_rejected(nonreceptive_snapshot);

    let mut stale_character_epilogue = live_idle_terminal_suspended_vwf_state();
    stale_character_epilogue.set_pending_nmi_subroutine(1);
    assert_rejected(stale_character_epilogue);

    let mut extra_receipt = live_idle_terminal_suspended_vwf_state();
    extra_receipt.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        3_823,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    ));
    assert_rejected(extra_receipt);

    let mut trailing_held = live_idle_terminal_suspended_vwf_state();
    trailing_held.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::LatchHeld];
    trailing_held.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        3_827,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        ],
    ));
    assert_rejected(trailing_held);

    let mut trailing_completion = live_idle_terminal_suspended_vwf_state();
    trailing_completion.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::Open];
    trailing_completion.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        3_827,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
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
        ],
    ));
    assert_rejected(trailing_completion);

    let mut trailing_joypad = live_idle_terminal_suspended_vwf_state();
    trailing_joypad.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::Open];
    trailing_joypad.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        3_827,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                high: 0,
                low: 0,
                high_filtered: 0,
                low_filtered: 0,
            }),
        ],
    ));
    assert_rejected(trailing_joypad);

    let mut multiple_acceptances = live_idle_terminal_suspended_vwf_state();
    multiple_acceptances.original_timing_expected_nmi_update_gates = vec![
        NmiUpdateGate::LatchHeld,
        NmiUpdateGate::Open,
        NmiUpdateGate::Open,
    ];
    multiple_acceptances.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        3_827,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    ));
    assert_rejected(multiple_acceptances);
}

#[test]
fn dless_suspended_vwf_terminal_folds_multiple_inner_timing_slices() {
    let mut state = live_idle_terminal_suspended_vwf_state();
    let mut decoded = vec![0; 29];
    // US command $7a is [Speed]; after the resumed glyph completes, this
    // two-byte command ends the restart loop without another synthetic host.
    decoded[26] = 0x7a;
    decoded[27] = 1;
    state.messaging_text_mut().load_decoded_dialogue(&decoded);
    state.messaging_state_mut().set_vwf_line_speed_cur(0);
    state.dialogue_vwf_glyph_cpu_phase = messaging::VwfGlyphCpuPhase::Drawing {
        remaining_master_cycles: 262 * 341 * 4 + 1,
    };

    let mut probe = state.clone();
    let transition = probe.advance_suspended_vwf_to_handler_completion();
    assert_eq!(transition.slice_count(), 2);

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(!state.dialogue_fast_forward_hold_active);
    assert!(!state.dialogue_fast_forward_hold_pending);
    assert_eq!(state.dialogue_live_message_read_position_target, None);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 2);
    assert_eq!(state.game_state.display.core_update_disable_flag, 2);
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn terminal_dialogue_endpoint_retires_stale_translated_hold_bookkeeping() {
    let mut state = live_idle_terminal_dialogue_endpoint_state(25);
    state.dialogue_fast_forward_hold_active = true;
    state.dialogue_live_message_read_position_target = Some(25);
    state
        .install_original_timing_host_receipts(idle_terminal_dialogue_endpoint_receipts(3_818, 25))
        .unwrap();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(!state.dialogue_fast_forward_hold_active);
    assert!(!state.dialogue_fast_forward_hold_pending);
    assert_eq!(state.dialogue_live_message_read_position_target, None);
    assert!(state.pending_main_loop_common_suffix.is_none());
}

#[test]
fn terminal_idle_dialogue_endpoint_preflight_is_failure_atomic() {
    fn assert_rejected_after_install(
        mut state: ZeldaState,
        semantic: Vec<OriginalTimingSemanticReceipt>,
        mutate_installed_state: impl FnOnce(&mut ZeldaState),
    ) {
        state
            .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
                3_818, 0, semantic,
            ))
            .unwrap();
        mutate_installed_state(&mut state);
        let scheduler_before = state.game_execution_scheduler;
        let ram_before = state.ram.clone();
        let receipts_before = state.original_timing_semantic_receipts.clone();
        let gates_before = state.original_timing_expected_nmi_update_gates.clone();
        let suffix_before = state.pending_main_loop_common_suffix;
        let sprite_preparation_before = state.main_loop_sprite_preparation_completed;
        let publication_before = (
            state.original_timing_nmi_publication_pending,
            state.original_timing_pending_nmi_update_gate,
        );
        let snapshot_before = state.display_snapshot.as_ref().map(|snapshot| {
            (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.oam.clone(),
                snapshot.ppu.vram.clone(),
            )
        });
        let epoch_before = state.display_snapshot_epoch;
        let latch_before = state.game_state.display.nmi_update_is_latched();
        let audio_before = state.audio_nmi_processed_before_main;

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal(0, crate::RUN_MAIN);
        }));
        assert!(result.is_err());
        assert_eq!(state.game_execution_scheduler, scheduler_before);
        assert_eq!(state.ram, ram_before);
        assert_eq!(state.original_timing_semantic_receipts, receipts_before);
        assert_eq!(
            state.original_timing_expected_nmi_update_gates,
            gates_before
        );
        assert_eq!(state.pending_main_loop_common_suffix, suffix_before);
        assert_eq!(
            state.main_loop_sprite_preparation_completed,
            sprite_preparation_before,
        );
        assert_eq!(
            (
                state.original_timing_nmi_publication_pending,
                state.original_timing_pending_nmi_update_gate,
            ),
            publication_before,
        );
        assert_eq!(
            state.display_snapshot.as_ref().map(|snapshot| (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.oam.clone(),
                snapshot.ppu.vram.clone(),
            )),
            snapshot_before,
        );
        assert_eq!(state.display_snapshot_epoch, epoch_before);
        assert_eq!(
            state.game_state.display.nmi_update_is_latched(),
            latch_before
        );
        assert_eq!(state.audio_nmi_processed_before_main, audio_before);
    }

    fn assert_rejected(state: ZeldaState, semantic: Vec<OriginalTimingSemanticReceipt>) {
        assert_rejected_after_install(state, semantic, |_| {});
    }

    let exact = idle_terminal_dialogue_endpoint_receipts(3_818, 25)
        .semantic()
        .to_vec();

    // An endpoint at or behind the native decoder cursor is no longer a
    // rejected shape: fresh iterations render natively at their own budget
    // and may legitimately lead the wire, so such an endpoint is an
    // already-satisfied no-op (route host 37226).

    let mut out_of_buffer_semantic = idle_terminal_dialogue_endpoint_receipts(3_818, 25)
        .semantic()
        .to_vec();
    *out_of_buffer_semantic.last_mut().unwrap() =
        OriginalTimingSemanticReceipt::DialogueExecutionProgress(
            crate::DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                message_read_position: u16::MAX,
            },
        );
    assert_rejected(
        live_idle_terminal_dialogue_endpoint_state(25),
        out_of_buffer_semantic,
    );

    let mut unreachable_endpoint = live_idle_terminal_dialogue_endpoint_state(25);
    unreachable_endpoint
        .messaging_state_mut()
        .set_vwf_line_speed_cur(2);
    let mut unreachable_endpoint_semantic = idle_terminal_dialogue_endpoint_receipts(3_818, 25)
        .semantic()
        .to_vec();
    *unreachable_endpoint_semantic.last_mut().unwrap() =
        OriginalTimingSemanticReceipt::DialogueExecutionProgress(
            crate::DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                message_read_position: 26,
            },
        );
    assert_rejected(unreachable_endpoint, unreachable_endpoint_semantic);

    let mut reordered = exact.clone();
    reordered.swap(3, 4);
    assert_rejected(live_idle_terminal_dialogue_endpoint_state(25), reordered);

    let mut extra = exact.clone();
    extra.insert(2, OriginalTimingSemanticReceipt::SpriteMainReturned);
    assert_rejected(live_idle_terminal_dialogue_endpoint_state(25), extra);

    let mut duplicate_endpoint = idle_terminal_dialogue_endpoint_receipts(3_818, 25)
        .semantic()
        .to_vec();
    duplicate_endpoint.push(OriginalTimingSemanticReceipt::DialogueExecutionProgress(
        crate::DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
            message_read_position: 25,
        },
    ));
    let mut duplicate_state = live_idle_terminal_dialogue_endpoint_state(25);
    let suffix_before_duplicate = duplicate_state.pending_main_loop_common_suffix;
    assert_eq!(
        duplicate_state.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            3_818,
            0,
            duplicate_endpoint
        ),),
        Err(OriginalTimingReceiptInstallError::DuplicateDialogueExecutionProgress),
    );
    assert!(duplicate_state.original_timing_semantic_receipts.is_none());
    assert_eq!(
        duplicate_state.pending_main_loop_common_suffix,
        suffix_before_duplicate,
    );

    let mut wrong_gate = exact;
    wrong_gate[0] = OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open);
    wrong_gate.insert(
        2,
        OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
            high: 1,
            low: 2,
            high_filtered: 3,
            low_filtered: 4,
        }),
    );
    assert_rejected(live_idle_terminal_dialogue_endpoint_state(25), wrong_gate);

    let exact_with_open = idle_terminal_dialogue_endpoint_with_trailing_open_receipts(4_652, 25)
        .semantic()
        .to_vec();

    let mut trailing_held = exact_with_open.clone();
    trailing_held[4] = OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld);
    assert_rejected(
        live_idle_terminal_dialogue_endpoint_state(25),
        trailing_held,
    );

    let mut trailing_completion = exact_with_open.clone();
    trailing_completion.insert(5, OriginalTimingSemanticReceipt::NmiHandlerCompleted);
    trailing_completion.insert(
        6,
        OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
            high: 1,
            low: 2,
            high_filtered: 3,
            low_filtered: 4,
        }),
    );
    assert_rejected(
        live_idle_terminal_dialogue_endpoint_state(25),
        trailing_completion,
    );

    assert_rejected_after_install(
        live_idle_terminal_dialogue_endpoint_state(25),
        exact_with_open.clone(),
        |state| {
            state
                .original_timing_semantic_receipts
                .as_mut()
                .unwrap()
                .semantic
                .insert(
                    5,
                    OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                        high: 5,
                        low: 6,
                        high_filtered: 7,
                        low_filtered: 8,
                    }),
                );
        },
    );

    assert_rejected_after_install(
        live_idle_terminal_dialogue_endpoint_state(25),
        exact_with_open.clone(),
        |state| {
            state
                .original_timing_semantic_receipts
                .as_mut()
                .unwrap()
                .semantic
                .insert(
                    5,
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                );
        },
    );

    let mut endpoint_before_trailing_open = exact_with_open.clone();
    endpoint_before_trailing_open.swap(4, 5);
    assert_rejected(
        live_idle_terminal_dialogue_endpoint_state(25),
        endpoint_before_trailing_open,
    );

    assert_rejected_after_install(
        live_idle_terminal_dialogue_endpoint_state(25),
        exact_with_open.clone(),
        |state| {
            state.original_timing_expected_nmi_update_gates =
                vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::LatchHeld];
        },
    );

    let mut wrong_native_latch = live_idle_terminal_dialogue_endpoint_state(25);
    wrong_native_latch.clear_nmi_update_latch();
    assert_rejected(
        wrong_native_latch,
        idle_terminal_dialogue_endpoint_receipts(3_818, 25)
            .semantic()
            .to_vec(),
    );

    let mut pending_hold = live_idle_terminal_dialogue_endpoint_state(25);
    pending_hold.dialogue_fast_forward_hold_pending = true;
    assert_rejected(
        pending_hold,
        idle_terminal_dialogue_endpoint_receipts(3_818, 25)
            .semantic()
            .to_vec(),
    );

    let mut stale_nmi_subroutine = live_idle_terminal_dialogue_endpoint_state(25);
    stale_nmi_subroutine.set_pending_nmi_subroutine(1);
    assert_rejected(
        stale_nmi_subroutine,
        idle_terminal_dialogue_endpoint_receipts(3_818, 25)
            .semantic()
            .to_vec(),
    );

    let mut stale_core_disable = live_idle_terminal_dialogue_endpoint_state(25);
    stale_core_disable.set_core_update_disable_flag(1);
    assert_rejected(
        stale_core_disable,
        idle_terminal_dialogue_endpoint_receipts(3_818, 25)
            .semantic()
            .to_vec(),
    );

    let mut legacy_suffix = idle_terminal_dialogue_endpoint_receipts(3_818, 25)
        .semantic()
        .to_vec();
    legacy_suffix[3] = OriginalTimingSemanticReceipt::MainLoopIterationReturnedToWait;
    assert_rejected(
        live_idle_terminal_dialogue_endpoint_state(25),
        legacy_suffix,
    );

    let mut specialized_suffix = live_idle_terminal_dialogue_endpoint_state(25);
    specialized_suffix.pending_main_loop_common_suffix = Some(
        MainLoopCommonSuffixContinuation::ResumeSpritePreparationExtendedOamPackingAndClearNmiLatch {
            next_group_start: 4,
        },
    );
    assert_rejected(
        specialized_suffix,
        idle_terminal_dialogue_endpoint_receipts(3_818, 25)
            .semantic()
            .to_vec(),
    );

    let mut wrong_suffix = live_idle_terminal_dialogue_endpoint_state(25);
    wrong_suffix.pending_main_loop_common_suffix = None;
    assert_eq!(
        wrong_suffix.install_original_timing_host_receipts(
            idle_terminal_dialogue_endpoint_receipts(3_818, 25),
        ),
        Err(OriginalTimingReceiptInstallError::InvalidMainLoopIterationReturn),
    );
    assert!(wrong_suffix.original_timing_semantic_receipts.is_none());
    assert_eq!(wrong_suffix.pending_main_loop_common_suffix, None);
}

#[test]
fn idle_main_loop_plan_preserves_explicit_dialogue_domain_claims() {
    let mut endpoint = live_idle_dialogue_main_loop_state();
    endpoint.messaging_state_mut().set_text_render_state(3);
    endpoint.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    endpoint.latch_nmi_update();
    endpoint
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            3_816,
            0,
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
        ))
        .unwrap();
    endpoint.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(
        endpoint.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert!(endpoint.original_timing_semantic_receipts.is_none());

    let mut closed = live_idle_dialogue_main_loop_state();
    closed.game_state.frame.set_saved_module_for_menu(9);
    closed.messaging_state_mut().set_module(1);
    closed.messaging_state_mut().set_text_render_state(3);
    closed
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            3_817,
            0,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::IterationStarted,
                ),
                OriginalTimingSemanticReceipt::SpriteMainReturned,
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                OriginalTimingSemanticReceipt::DialogueClosed,
            ],
        ))
        .unwrap();
    closed.run_frame_internal(0, crate::RUN_MAIN);
    assert_ne!(
        closed.game_state.frame.main_module, 14,
        "the claimed close must leave Module0E through the native dialogue-close path",
    );
    assert_eq!(closed.game_state.messaging.runtime.text_render_state(), 4);
    assert!(closed.pending_main_loop_common_suffix.is_none());
    assert!(closed.original_timing_semantic_receipts.is_none());
}

#[test]
fn terminal_dialogue_return_preflight_rejects_malformed_ownership_without_consumption() {
    fn assert_rejected_without_ownership_mutation(mut state: ZeldaState) {
        let receipts_before = state.original_timing_semantic_receipts.clone();
        let snapshot_before = state.display_snapshot.as_ref().map(|snapshot| {
            (
                snapshot.accepts_nmi_dma_receipts,
                snapshot.publication_host_frame,
                snapshot.ppu.vram.clone(),
            )
        });
        let suffix_before = state.pending_main_loop_common_suffix;
        let latch_before = state.game_state.display.nmi_update_is_latched();
        let pending_before = state.original_timing_nmi_publication_pending;
        let gate_before = state.original_timing_pending_nmi_update_gate;
        let expected_gates_before = state.original_timing_expected_nmi_update_gates.clone();
        let endpoint_before = state.game_state.messaging.runtime.dialogue_msg_read_pos();
        let mut scheduler_after_host_entry = state.game_execution_scheduler;
        scheduler_after_host_entry.begin_host_frame();
        let work_before = state.game_execution_scheduler.current_work();
        let slices_before = state
            .game_execution_scheduler
            .scheduled_work_slices_remaining();
        let audio_before = (
            state.game_state.system_signals.ambient_sound_effect(),
            state.game_state.system_signals.last_ambient_sound_effect(),
            state.game_state.system_signals.music_control(),
            state.game_state.system_signals.current_music_control(),
            state.game_state.system_signals.last_music_control(),
            state.game_state.system_signals.sound_effect_1(),
            state.game_state.system_signals.sound_effect_2(),
            state.zelda_debug_apu_write_ports(),
            state.audio_nmi_processed_before_main,
            state.initialized,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal(0, crate::RUN_MAIN);
        }));
        assert!(result.is_err());
        assert_eq!(state.original_timing_semantic_receipts, receipts_before);
        assert_eq!(
            state.display_snapshot.as_ref().map(|snapshot| (
                snapshot.accepts_nmi_dma_receipts,
                snapshot.publication_host_frame,
                snapshot.ppu.vram.clone(),
            )),
            snapshot_before,
        );
        assert_eq!(state.pending_main_loop_common_suffix, suffix_before);
        assert_eq!(
            state.game_state.display.nmi_update_is_latched(),
            latch_before
        );
        assert_eq!(
            state.original_timing_nmi_publication_pending,
            pending_before
        );
        assert_eq!(state.original_timing_pending_nmi_update_gate, gate_before);
        assert_eq!(
            state.original_timing_expected_nmi_update_gates,
            expected_gates_before,
        );
        assert_eq!(
            state.game_state.messaging.runtime.dialogue_msg_read_pos(),
            endpoint_before,
        );
        assert_eq!(
            state.game_execution_scheduler, scheduler_after_host_entry,
            "only the shared host-entry phase normalization may precede terminal dialogue validation",
        );
        assert_eq!(state.game_execution_scheduler.current_work(), work_before);
        assert_eq!(
            state
                .game_execution_scheduler
                .scheduled_work_slices_remaining(),
            slices_before,
        );
        assert_eq!(
            (
                state.game_state.system_signals.ambient_sound_effect(),
                state.game_state.system_signals.last_ambient_sound_effect(),
                state.game_state.system_signals.music_control(),
                state.game_state.system_signals.current_music_control(),
                state.game_state.system_signals.last_music_control(),
                state.game_state.system_signals.sound_effect_1(),
                state.game_state.system_signals.sound_effect_2(),
                state.zelda_debug_apu_write_ports(),
                state.audio_nmi_processed_before_main,
                state.initialized,
            ),
            audio_before,
            "dialogue terminal authority must fail before audio sampling or initialization",
        );
    }

    let mut missing_authority = live_dialogue_terminal_return_state(0x37);
    missing_authority
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(2334, 0, vec![]))
        .unwrap();
    assert_rejected_without_ownership_mutation(missing_authority);

    let mut missing_suffix = live_dialogue_terminal_return_state(0x37);
    missing_suffix.pending_main_loop_common_suffix = None;
    let receipts_before = missing_suffix.original_timing_semantic_receipts.clone();
    let snapshot_before = missing_suffix
        .display_snapshot
        .as_ref()
        .map(|snapshot| snapshot.ppu.vram.clone());
    assert!(missing_suffix
        .install_original_timing_host_receipts(dialogue_terminal_return_receipts(2334))
        .is_err());
    assert_eq!(
        missing_suffix.original_timing_semantic_receipts,
        receipts_before,
    );
    assert_eq!(
        missing_suffix
            .display_snapshot
            .as_ref()
            .map(|snapshot| snapshot.ppu.vram.clone()),
        snapshot_before,
    );

    let mut closed_snapshot = live_dialogue_terminal_return_state(0x37);
    closed_snapshot
        .display_snapshot
        .as_mut()
        .unwrap()
        .accepts_nmi_dma_receipts = false;
    closed_snapshot
        .install_original_timing_host_receipts(dialogue_terminal_return_receipts(2334))
        .unwrap();
    assert_rejected_without_ownership_mutation(closed_snapshot);

    let mut malformed_after = live_dialogue_terminal_return_state(0x37);
    // Bypass the installer so the runtime preflight itself proves that a
    // corrupted post-suffix completion cannot mutate the active owner.
    malformed_after.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        2334,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        ],
    ));
    malformed_after.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::Open];
    assert_rejected_without_ownership_mutation(malformed_after);

    let mut reordered_same_host = live_dialogue_terminal_return_state(0x37);
    reordered_same_host.original_timing_nmi_publication_pending = false;
    reordered_same_host.original_timing_pending_nmi_update_gate = None;
    reordered_same_host.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        3812,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    ));
    reordered_same_host.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    assert_rejected_without_ownership_mutation(reordered_same_host);

    let mut held_with_joypad = live_dialogue_terminal_return_state(0x37);
    held_with_joypad.original_timing_nmi_publication_pending = false;
    held_with_joypad.original_timing_pending_nmi_update_gate = None;
    held_with_joypad.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        3812,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                high: 0,
                low: 0,
                high_filtered: 0,
                low_filtered: 0,
            }),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    ));
    held_with_joypad.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    assert_rejected_without_ownership_mutation(held_with_joypad);

    let mut wrong_same_host_gate = live_dialogue_terminal_return_state(0x37);
    wrong_same_host_gate.original_timing_nmi_publication_pending = false;
    wrong_same_host_gate.original_timing_pending_nmi_update_gate = None;
    wrong_same_host_gate
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            3812,
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
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            ],
        ))
        .unwrap();
    assert_rejected_without_ownership_mutation(wrong_same_host_gate);

    let mut wrong_progress = live_dialogue_terminal_return_state(0x37);
    // Bypass the public installer to exercise the runtime's own fail-closed
    // guard against a corrupted transient which gives an active suspended
    // initializer a fresh-iteration terminal fact.
    wrong_progress.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        2334,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    ));
    assert_rejected_without_ownership_mutation(wrong_progress);

    let mut spurious_endpoint = live_dialogue_terminal_return_state(0x37);
    spurious_endpoint
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2334,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                    crate::DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                        message_read_position: 0x37,
                    },
                ),
            ],
        ))
        .unwrap();
    assert_rejected_without_ownership_mutation(spurious_endpoint);

    let mut completed_sprite_preparation = live_dialogue_terminal_return_state(0x37);
    completed_sprite_preparation.main_loop_sprite_preparation_completed = true;
    completed_sprite_preparation
        .install_original_timing_host_receipts(dialogue_terminal_return_receipts(2334))
        .unwrap();
    assert_rejected_without_ownership_mutation(completed_sprite_preparation);

    let mut wrong_gate = live_dialogue_terminal_return_state(0x37);
    wrong_gate.clear_nmi_update_latch();
    wrong_gate.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
    wrong_gate
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2334,
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
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            ],
        ))
        .unwrap();
    assert_rejected_without_ownership_mutation(wrong_gate);

    let mut wrong_scheduler_owner = live_dialogue_terminal_return_state(0x37);
    wrong_scheduler_owner.game_execution_scheduler = GameExecutionScheduler::default();
    wrong_scheduler_owner
        .game_execution_scheduler
        .schedule_post_trailing_nmi(GameWorkContinuation::FinishDialogueInitializationCallerReturn);
    wrong_scheduler_owner
        .install_original_timing_host_receipts(dialogue_terminal_return_receipts(2334))
        .unwrap();
    assert_rejected_without_ownership_mutation(wrong_scheduler_owner);
}

#[test]
fn room_61_all_rescue_follower_trigger_zones_open_the_message() {
    for (x, y, event_bit) in [
        (0x025b, 0x0cf8, 1),
        (0x039d, 0x0cf8, 2),
        (0x0238, 0x0c78, 4),
    ] {
        let mut state = ZeldaState::new();
        state.set_main_module(7);
        state.set_submodule(0);
        state.set_rom_startup_timing(true);
        state.set_indoor_flag(1);
        state.set_dungeon_room(0x0061);
        state.follower_state_mut().set_indicator(1);
        state.follower_link_state_mut().set_x(x);
        state.follower_link_state_mut().set_y(y);

        state.follower_handle_trigger();

        assert_eq!(state.game_state.frame.main_module, 14);
        assert_eq!(state.game_state.frame.submodule, 2);
        assert_eq!(
            state.game_state.messaging.dialogue_message_index.value(),
            0x21
        );
        assert_eq!(
            state.game_state.sprites.follower_runtime.event_flags(),
            event_bit
        );
        assert!(state.next_display_obj_memory_generation.is_none());
        assert_eq!(
            state.next_display_obj_scanout_generation.is_some(),
            event_bit == 2
        );
        if event_bit == 2 {
            assert_eq!(
                state.next_display_obj_scanout_generation,
                Some(ObjScanoutGenerations {
                    oam: OamScanoutSource::ComposeLiveAfterNmiWithHostBoundaryLink,
                    link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                })
            );
        }
    }
}

#[test]
fn rescue_follower_message_obj_selector_handles_rollover_and_rejects_neighbors() {
    let frame = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0,
        ..Default::default()
    };

    assert_eq!(
        rescue_follower_message_obj_scanout(frame, 0x61, 2, 0x21),
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposeLiveAfterNmiWithHostBoundaryLink,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        })
    );
    assert_eq!(
        rescue_follower_message_obj_scanout(
            crate::game_state::FrameState {
                frame_counter: 0xff,
                ..frame
            },
            0x61,
            2,
            0x21
        ),
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposeLiveAfterNmiWithHostBoundaryLink,
            link_obj: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
        })
    );
    assert_eq!(
        rescue_follower_message_obj_scanout(frame, 0x61, 1, 0x21),
        None
    );
    assert_eq!(
        rescue_follower_message_obj_scanout(frame, 0x61, 2, 0x22),
        None
    );
    assert_eq!(
        rescue_follower_message_obj_scanout(frame, 0x52, 2, 0x21),
        None
    );
    assert_eq!(
        rescue_follower_message_obj_scanout(
            crate::game_state::FrameState {
                main_module: 14,
                submodule: 2,
                ..Default::default()
            },
            0x61,
            2,
            0x21
        ),
        None
    );
}

#[test]
fn rescue_follower_message_rollover_stages_live_link_graphics_only() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.game_state.frame.frame_counter = 0xff;
    state.set_rom_startup_timing(true);
    state.set_indoor_flag(1);
    state.set_dungeon_room(0x0061);
    state.follower_state_mut().set_indicator(1);
    state.follower_link_state_mut().set_x(0x039d);
    state.follower_link_state_mut().set_y(0x0cf8);
    state.follower_handle_trigger();

    assert!(state.next_display_obj_memory_generation.is_none());
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposeLiveAfterNmiWithHostBoundaryLink,
            link_obj: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
        })
    );
}

#[test]
fn dialogue_scroll_checkpoint_projection_preserves_stable_phases_and_normalizes_transients() {
    fn round_trip(state: &ZeldaState) -> ZeldaState {
        bincode::deserialize(
            &bincode::serialize(state).expect("serialize dialogue-scroll checkpoint"),
        )
        .expect("deserialize dialogue-scroll checkpoint")
    }

    fn scrolling_state(completion_timing: DialogueScrollCompletionTiming) -> ZeldaState {
        let mut state = ZeldaState::new();
        state.ppu.vram[0x7c00] = 0x1234;
        state.begin_dialogue_scroll(DialogueTextGeneration::PublishedDisplay, completion_timing);
        state
    }

    let copying = round_trip(&scrolling_state(
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    ));
    assert_eq!(
        copying.dialogue_scroll_phase(),
        DialogueScrollPhase::CopyingRemainingPixels {
            completion_timing: DialogueScrollCompletionTiming::AfterReturnBoundary,
        }
    );
    assert_eq!(
        copying
            .dialogue_scroll_frozen_scanout
            .as_ref()
            .unwrap()
            .vram[0],
        0x1234
    );

    let mut return_only = scrolling_state(DialogueScrollCompletionTiming::AfterReturnBoundary);
    return_only.finish_dialogue_scroll_remaining_pixels();
    let return_only = round_trip(&return_only);
    assert_eq!(
        return_only.dialogue_scroll_phase(),
        DialogueScrollPhase::ReturnOnly
    );

    let mut pending = scrolling_state(DialogueScrollCompletionTiming::BeforeNextVblank);
    pending.finish_dialogue_scroll_remaining_pixels();
    let pending = round_trip(&pending);
    assert_eq!(
        pending.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionPendingPublication
    );

    let mut staged = scrolling_state(DialogueScrollCompletionTiming::BeforeNextVblank);
    staged.finish_dialogue_scroll_remaining_pixels();
    staged.stage_early_dialogue_scroll_completion(DialogueTextScanout::default());
    let mut staged = round_trip(&staged);
    staged.restore_live_rom_timing_after_checkpoint();
    assert_eq!(
        staged.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionStagedAfterFrozenScanout
    );

    let mut completed = scrolling_state(DialogueScrollCompletionTiming::BeforeNextVblank);
    completed.finish_dialogue_scroll_remaining_pixels();
    completed.stage_early_dialogue_scroll_completion(DialogueTextScanout {
        vram: vec![0; crate::PresentedDialogueText::WORD_COUNT],
        ..DialogueTextScanout::default()
    });
    completed.advance_dialogue_scroll_display_boundary();
    publish_staged_dialogue_text_dma(&mut completed);
    let mut completed = round_trip(&completed);
    completed.restore_live_rom_timing_after_checkpoint();
    assert_eq!(
        completed.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletedScroll
    );

    let mut staged_after_return =
        scrolling_state(DialogueScrollCompletionTiming::AfterReturnBoundary);
    staged_after_return.finish_dialogue_scroll_remaining_pixels();
    staged_after_return.finish_dialogue_scroll_return();
    staged_after_return
        .stage_dialogue_scroll_completion_after_return(DialogueTextScanout::default());
    let mut staged_after_return = round_trip(&staged_after_return);
    staged_after_return.restore_live_rom_timing_after_checkpoint();
    assert_eq!(
        staged_after_return.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionStagedAfterSnapshot
    );
}

#[test]
fn restored_staged_dialogue_completion_consumes_the_next_ordinary_open_nmi() {
    let mut state = ZeldaState::new();
    state.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::BeforeNextVblank,
    );
    state.finish_dialogue_scroll_remaining_pixels();
    for index in 0..crate::PresentedDialogueText::WORD_COUNT {
        state.set_messaging_render_buffer_word(index, 0x4600 | index as u16);
    }
    let staged = state.dialogue_text_scanout_from_render_buffer();
    state.stage_early_dialogue_scroll_completion(staged);
    let mut state: ZeldaState = bincode::deserialize(
        &bincode::serialize(&state).expect("serialize staged dialogue checkpoint"),
    )
    .expect("restore staged dialogue checkpoint");
    state.restore_live_rom_timing_after_checkpoint();
    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionStagedAfterFrozenScanout,
    );

    state.capture_display_snapshot();
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
        0x4600,
    );
}

#[test]
fn dialogue_character_tiles_publish_at_the_following_nmi() {
    assert!(rom_display_memory_publication_is_deferred(14, 2, 3, false));
    assert!(rom_display_memory_publication_is_deferred(4, 3, 0, true));
    assert!(!rom_display_memory_publication_is_deferred(4, 3, 0, false));
    assert!(!rom_display_memory_publication_is_deferred(14, 1, 0, false));
}

#[test]
fn dialogue_scroll_freezes_the_published_hardware_generation() {
    let mut state = ZeldaState::new();
    state.ppu.vram[0x7c00] = 0x1111;
    state.ram[0x10000] = 0x22;
    state.ram[0x10001] = 0x22;
    state.bg3_vwf_glyph_runs = vec![Bg3VwfGlyphRun {
        glyph_code: 2,
        ..Bg3VwfGlyphRun::default()
    }];
    state.published_bg3_vwf_glyph_runs = vec![Bg3VwfGlyphRun {
        glyph_code: 1,
        ..Bg3VwfGlyphRun::default()
    }];
    state.published_dialogue_msg_read_pos = 0x34;
    state.published_dialogue_message_id = 0x56;

    state.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );

    let frozen = state
        .dialogue_scroll_frozen_scanout
        .as_ref()
        .expect("published dialogue scanout");
    assert_eq!(frozen.vram[0], 0x1111);
    assert_eq!(frozen.glyph_runs[0].glyph_code, 1);
    assert_eq!(frozen.dialogue_msg_read_pos, 0x34);
    assert_eq!(frozen.dialogue_message_id, 0x56);
}

#[test]
fn dialogue_scroll_completion_timing_follows_measured_vblank_headroom() {
    // A call the host's remaining CPU work cannot cover returns after the
    // boundary; one it can cover returns before the next vblank.
    let one_pass = crate::cycle_models::vwf::SCROLL_PASS_MASTER_CYCLES;
    assert_eq!(
        DialogueScrollCompletionTiming::at_scroll_entry(255_000, 5 * one_pass),
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );
    assert_eq!(
        DialogueScrollCompletionTiming::at_scroll_entry(283_400, 5 * one_pass),
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );
    assert_eq!(
        DialogueScrollCompletionTiming::at_scroll_entry(283_400, 2 * one_pass),
        DialogueScrollCompletionTiming::BeforeNextVblank,
    );
    // Every call that reaches this decision copies a full five-pass group,
    // which no frame of CPU work can cover.
    // 262 scanlines of 341 dots at four master cycles per dot.
    assert!(5 * one_pass > 262 * 341 * 4);

    let mut state = ZeldaState::new();
    state.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::BeforeNextVblank,
    );
    assert_eq!(
        state.finish_dialogue_scroll_remaining_pixels(),
        DialogueScrollCompletionTiming::BeforeNextVblank
    );
    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionPendingPublication
    );
    state.stage_early_dialogue_scroll_completion(DialogueTextScanout::default());
    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionStagedAfterFrozenScanout
    );
}

#[test]
fn dialogue_scroll_completion_uses_live_main_loop_receipt_over_entry_estimate() {
    let mut source_completed_before_vblank = ZeldaState::new();
    source_completed_before_vblank.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );
    assert_eq!(
        source_completed_before_vblank
            .finish_dialogue_scroll_remaining_pixels_with_main_loop_receipt(
                crate::MainLoopProgress::IterationStarted,
            ),
        DialogueScrollCompletionTiming::BeforeNextVblank,
    );
    assert_eq!(
        source_completed_before_vblank.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionPendingPublication,
    );

    let mut source_returned_after_vblank = ZeldaState::new();
    source_returned_after_vblank.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::BeforeNextVblank,
    );
    assert_eq!(
        source_returned_after_vblank
            .finish_dialogue_scroll_remaining_pixels_with_main_loop_receipt(
                crate::MainLoopProgress::CallStackContinued,
            ),
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );
    assert_eq!(
        source_returned_after_vblank.dialogue_scroll_phase(),
        DialogueScrollPhase::ReturnOnly,
    );
}

#[test]
fn early_dialogue_completion_requires_and_consumes_the_same_host_text_dma_evidence() {
    let mut state = early_dialogue_completion_state(2);
    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletedScroll,
    );
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn early_dialogue_completion_rejects_missing_text_dma_before_acceptance_capture() {
    let mut state = early_dialogue_completion_state(1);
    let phase_before = state.dialogue_scroll_phase();
    let epoch_before = state.display_snapshot_epoch;
    let ppu_before = state.ppu.clone();
    let receipts_before = state.original_timing_semantic_receipts.clone();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.run_frame_internal(0, crate::RUN_MAIN);
    }));

    assert!(result.is_err());
    assert_eq!(state.dialogue_scroll_phase(), phase_before);
    assert_eq!(state.display_snapshot_epoch, epoch_before);
    assert_eq!(state.ppu.vram, ppu_before.vram);
    assert_eq!(state.ppu.oam, ppu_before.oam);
    assert_eq!(state.original_timing_semantic_receipts, receipts_before);
}

#[test]
fn dialogue_completion_before_vblank_uses_text_dma_sampled_at_publication() {
    let mut state = ZeldaState::new();
    state.set_main_module(14);
    state.set_submodule(2);
    state.ppu.vram[0x7c00] = 0x1111;
    state.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::BeforeNextVblank,
    );
    state.ppu.vram[0x7c00] = 0x2222;
    state.finish_dialogue_scroll_remaining_pixels();

    state.capture_display_snapshot();

    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionPendingPublication
    );
    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[0x7c00]),
        0x1111
    );

    state.stage_early_dialogue_scroll_completion(DialogueTextScanout {
        vram: vec![0x3333; 0x3f0],
        ..DialogueTextScanout::default()
    });
    state.capture_display_snapshot();
    publish_staged_dialogue_text_dma(&mut state);

    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletedScroll
    );
    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[0x7c00]),
        0x3333
    );

    // The next boundary samples the actual NMI text-DMA source instead of
    // projecting the earlier staged CPU buffer for a second scanout.
    write_le_u16(&mut state.ram, 0x10000, 0x4444);
    state.capture_display_snapshot();
    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::RetiredTextDma
    );
    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[0x7c00]),
        0x4444
    );
}

#[test]
fn dialogue_scroll_machine_has_closed_hardware_boundary_sequences() {
    let mut after_return = ZeldaState::new();
    after_return.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );
    assert_eq!(
        after_return.dialogue_scroll_phase(),
        DialogueScrollPhase::CopyingRemainingPixels {
            completion_timing: DialogueScrollCompletionTiming::AfterReturnBoundary,
        }
    );
    after_return.finish_dialogue_scroll_remaining_pixels();
    assert_eq!(
        after_return.dialogue_scroll_phase(),
        DialogueScrollPhase::ReturnOnly
    );
    after_return.finish_dialogue_scroll_return();
    assert_eq!(
        after_return.dialogue_scroll_phase(),
        DialogueScrollPhase::Idle
    );
    after_return.stage_dialogue_scroll_completion_after_return(DialogueTextScanout {
        vram: vec![0; crate::PresentedDialogueText::WORD_COUNT],
        ..DialogueTextScanout::default()
    });
    assert_eq!(
        after_return.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionStagedAfterSnapshot
    );
    after_return.advance_dialogue_scroll_display_boundary();
    assert_eq!(
        after_return.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionStagedAfterSnapshot,
        "capture alone cannot prove that the after-snapshot BG3 text DMA ran",
    );
    publish_staged_dialogue_text_dma(&mut after_return);
    assert_eq!(
        after_return.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletedScroll
    );
    let mut adjacent_scroll = after_return.clone();
    adjacent_scroll.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );
    assert_eq!(
        adjacent_scroll.dialogue_scroll_phase(),
        DialogueScrollPhase::CopyingRemainingPixels {
            completion_timing: DialogueScrollCompletionTiming::AfterReturnBoundary,
        }
    );
    after_return.advance_dialogue_scroll_display_boundary();
    assert_eq!(
        after_return.dialogue_scroll_phase(),
        DialogueScrollPhase::RetiredTextDma
    );
    after_return.advance_dialogue_scroll_display_boundary();
    assert_eq!(
        after_return.dialogue_scroll_phase(),
        DialogueScrollPhase::Idle
    );

    let mut before_vblank = ZeldaState::new();
    before_vblank.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::BeforeNextVblank,
    );
    before_vblank.finish_dialogue_scroll_remaining_pixels();
    assert_eq!(
        before_vblank.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionPendingPublication
    );
    before_vblank.advance_dialogue_scroll_display_boundary();
    assert_eq!(
        before_vblank.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionPendingPublication
    );
    before_vblank.stage_early_dialogue_scroll_completion(DialogueTextScanout {
        vram: vec![0; crate::PresentedDialogueText::WORD_COUNT],
        ..DialogueTextScanout::default()
    });
    assert_eq!(
        before_vblank.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionStagedAfterFrozenScanout
    );
    before_vblank.advance_dialogue_scroll_display_boundary();
    assert_eq!(
        before_vblank.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionStagedAfterFrozenScanout,
        "capture alone cannot prove that the after-frozen BG3 text DMA ran",
    );
    publish_staged_dialogue_text_dma(&mut before_vblank);
    assert_eq!(
        before_vblank.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletedScroll
    );
    before_vblank.advance_dialogue_scroll_display_boundary();
    assert_eq!(
        before_vblank.dialogue_scroll_phase(),
        DialogueScrollPhase::RetiredTextDma
    );
    before_vblank.advance_dialogue_scroll_display_boundary();
    assert_eq!(
        before_vblank.dialogue_scroll_phase(),
        DialogueScrollPhase::Idle
    );
}

#[test]
fn vwf_endpoint_preflights_the_following_source_scroll_before_mutation() {
    let mut state = ZeldaState::new();
    state.set_main_module(14);
    state.set_submodule(2);
    state.messaging_state_mut().set_module(1);
    state.messaging_state_mut().set_text_render_state(3);
    state.messaging_state_mut().set_dialogue_scroll_speed(4);
    state
        .messaging_text_mut()
        .load_decoded_dialogue(&[TEXT_COMMAND_START_US + 12]);
    state.dialogue_fast_forward_hold_active = true;
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(
        OriginalTimingHostReceipts::new(
            0,
            0,
            vec![OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                crate::DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                    message_read_position: 0,
                },
            )],
        )
        .with_dialogue_scroll_progress(vec![crate::DialogueScrollProgressReceipt {
            entered: true,
            completed_pixel_passes: 2,
            returned: false,
        }]),
    );
    let before = state.original_timing_semantic_receipts.clone();
    let transition = state.original_timing_suspended_vwf_endpoint_transition_plan(0, false);
    assert_eq!(state.original_timing_semantic_receipts, before);
    assert!(state.dialogue_scroll_cpu_is_idle());

    let mut invalid = state.clone();
    invalid
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .dialogue_scroll_progress[0]
        .completed_pixel_passes = 6;
    let invalid_receipts = invalid.original_timing_semantic_receipts.clone();
    let invalid_buffer = invalid.game_state.messaging.render_buffer.clone();
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        invalid.original_timing_suspended_vwf_endpoint_transition_plan(0, false);
    }))
    .is_err());
    assert_eq!(invalid.original_timing_semantic_receipts, invalid_receipts);
    assert_eq!(invalid.game_state.messaging.render_buffer, invalid_buffer);
    assert!(invalid.dialogue_scroll_cpu_is_idle());

    state.complete_original_timing_suspended_vwf_endpoint(0, false, transition);
    assert!(state.dialogue_scroll_is_copying_remaining_pixels());
    assert!(!state.dialogue_fast_forward_hold_active);
    assert_eq!(
        state.game_state.messaging.runtime.dialogue_msg_read_pos(),
        0
    );
    assert_eq!(
        state
            .game_state
            .messaging
            .dialogue_source_offset
            .bank_offset_low_nibble(),
        2
    );
}

#[test]
fn live_return_only_dialogue_uses_all_four_source_owned_nmi_grammars() {
    for publication_pending_at_entry in [false, true] {
        for trailing_open_acceptance in [false, true] {
            let mut state = return_only_dialogue_state_before_run2508();
            if !publication_pending_at_entry {
                state.original_timing_nmi_publication_pending = false;
                state.original_timing_pending_nmi_update_gate = None;
            }
            let entry_epoch = state.display_snapshot_epoch;
            let mut semantic = Vec::new();
            if !publication_pending_at_entry {
                semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
                    NmiUpdateGate::LatchHeld,
                ));
            }
            semantic.extend([
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            ]);
            if trailing_open_acceptance {
                semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
                    NmiUpdateGate::Open,
                ));
            }
            state
                .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
                    2508, 0, semantic,
                ))
                .unwrap();

            state.run_frame_internal(0, crate::RUN_MAIN);

            assert_eq!(
                state.dialogue_scroll_phase(),
                DialogueScrollPhase::CompletionStagedAfterSnapshot,
            );
            assert_eq!(
                state.display_snapshot_epoch - entry_epoch,
                u64::from(!publication_pending_at_entry) + u64::from(trailing_open_acceptance),
                "each source acceptance owns exactly one capture",
            );
            assert_eq!(
                state.original_timing_nmi_publication_pending,
                trailing_open_acceptance,
            );
            assert_eq!(
                state.original_timing_pending_nmi_update_gate,
                trailing_open_acceptance.then_some(NmiUpdateGate::Open),
            );
            assert!(state.pending_main_loop_common_suffix.is_none());
            assert!(state.main_loop_sprite_preparation_completed);
            assert!(!state.game_state.display.nmi_update_is_latched());
            assert!(state.original_timing_semantic_receipts.is_none());
            assert!(state.game_execution_scheduler.is_idle());
            assert_eq!(
                state.with_display_snapshot(|display| display.ppu.vram[0x7c00]),
                0x1111,
                "the Held handler cannot expose the staged text generation",
            );
        }
    }
}

#[test]
fn live_return_only_dialogue_rejects_a_competing_scheduler_owner_before_mutation() {
    let mut state = return_only_dialogue_state_before_run2508();
    state
        .game_execution_scheduler
        .schedule_after_current_trailing_nmi(
            GameWorkContinuation::FinishDialogueInitializationCallerReturn,
        );
    // Match the host-entry phase which run_frame_internal will establish so
    // the failure-atomic assertion isolates the ownership preflight itself.
    state.game_execution_scheduler.begin_host_frame();
    state
        .install_original_timing_host_receipts(run2508_dialogue_receipts())
        .unwrap();

    let scheduler_before = state.game_execution_scheduler;
    let receipts_before = state.original_timing_semantic_receipts.clone();
    let gates_before = state.original_timing_expected_nmi_update_gates.clone();
    let pending_suffix_before = state.pending_main_loop_common_suffix;
    let publication_before = (
        state.original_timing_nmi_publication_pending,
        state.original_timing_pending_nmi_update_gate,
    );
    let phase_before = state.dialogue_scroll_phase();
    let snapshot_before = state.display_snapshot.as_ref().map(|snapshot| {
        (
            snapshot.publication_epoch,
            snapshot.publication_host_frame,
            snapshot.accepts_nmi_dma_receipts,
            snapshot.ppu.vram[0x7c00],
        )
    });
    let latch_before = state.game_state.display.nmi_update_is_latched();
    let audio_processed_before = state.audio_nmi_processed_before_main;

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.run_frame_internal(0, crate::RUN_MAIN);
    }));

    assert!(result.is_err());
    assert_eq!(state.game_execution_scheduler, scheduler_before);
    assert_eq!(state.original_timing_semantic_receipts, receipts_before);
    assert_eq!(
        state.original_timing_expected_nmi_update_gates,
        gates_before
    );
    assert_eq!(state.pending_main_loop_common_suffix, pending_suffix_before);
    assert_eq!(
        (
            state.original_timing_nmi_publication_pending,
            state.original_timing_pending_nmi_update_gate,
        ),
        publication_before,
    );
    assert_eq!(state.dialogue_scroll_phase(), phase_before);
    assert_eq!(
        state.display_snapshot.as_ref().map(|snapshot| (
            snapshot.publication_epoch,
            snapshot.publication_host_frame,
            snapshot.accepts_nmi_dma_receipts,
            snapshot.ppu.vram[0x7c00],
        )),
        snapshot_before,
    );
    assert_eq!(
        state.game_state.display.nmi_update_is_latched(),
        latch_before,
    );
    assert_eq!(
        state.audio_nmi_processed_before_main,
        audio_processed_before,
    );
}

#[test]
fn live_return_only_dialogue_rejects_generic_and_pre_main_scheduler_owners() {
    fn assert_rejected(mut state: ZeldaState) {
        state.game_execution_scheduler.begin_host_frame();
        state
            .install_original_timing_host_receipts(run2508_dialogue_receipts())
            .unwrap();
        let scheduler_before = state.game_execution_scheduler;
        let receipts_before = state.original_timing_semantic_receipts.clone();
        let suffix_before = state.pending_main_loop_common_suffix;
        let phase_before = state.dialogue_scroll_phase();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal(0, crate::RUN_MAIN);
        }));
        assert!(result.is_err());
        assert_eq!(state.game_execution_scheduler, scheduler_before);
        assert_eq!(state.original_timing_semantic_receipts, receipts_before);
        assert_eq!(state.pending_main_loop_common_suffix, suffix_before);
        assert_eq!(state.dialogue_scroll_phase(), phase_before);
    }

    let mut scheduled = return_only_dialogue_state_before_run2508();
    scheduled.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDialogueInitializationCallerReturn,
        1,
    );
    assert_rejected(scheduled);

    let mut pre_main = return_only_dialogue_state_before_run2508();
    pre_main
        .game_execution_scheduler
        .schedule_pre_main_caller_continuation(PreMainCallerContinuation::DialogueVwfReturn);
    assert_rejected(pre_main);
}

#[test]
fn live_return_only_dialogue_rejects_extra_or_reordered_receipts_before_mutation() {
    fn assert_rejected(semantic: Vec<OriginalTimingSemanticReceipt>) {
        let mut state = return_only_dialogue_state_before_run2508();
        state
            .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
                2508, 0, semantic,
            ))
            .unwrap();
        let receipts_before = state.original_timing_semantic_receipts.clone();
        let gates_before = state.original_timing_expected_nmi_update_gates.clone();
        let suffix_before = state.pending_main_loop_common_suffix;
        let phase_before = state.dialogue_scroll_phase();
        let snapshot_before = state.display_snapshot.as_ref().map(|snapshot| {
            (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.vram[0x7c00],
            )
        });
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal(0, crate::RUN_MAIN);
        }));
        assert!(result.is_err());
        assert_eq!(state.original_timing_semantic_receipts, receipts_before);
        assert_eq!(
            state.original_timing_expected_nmi_update_gates,
            gates_before
        );
        assert_eq!(state.pending_main_loop_common_suffix, suffix_before);
        assert_eq!(state.dialogue_scroll_phase(), phase_before);
        assert_eq!(
            state.display_snapshot.as_ref().map(|snapshot| (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.vram[0x7c00],
            )),
            snapshot_before,
        );
    }

    let mut extra = run2508_dialogue_receipts().semantic().to_vec();
    extra.push(OriginalTimingSemanticReceipt::DialogueExecutionProgress(
        crate::DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
            message_read_position: 0,
        },
    ));
    assert_rejected(extra);

    assert_rejected(vec![
        OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ),
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
    ]);
}

#[test]
fn carried_open_dialogue_handler_publishes_staged_text_before_the_adjacent_scroll() {
    let mut state = return_only_dialogue_state_before_run2508();
    state
        .install_original_timing_host_receipts(run2508_dialogue_receipts())
        .unwrap();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionStagedAfterSnapshot,
    );
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::Open),
    );
    let run2508_publication = state
        .display_snapshot
        .as_ref()
        .map(|snapshot| {
            (
                snapshot.publication_host_frame,
                snapshot.ppu.vram[0x7c00],
                snapshot.accepts_nmi_dma_receipts,
            )
        })
        .unwrap();
    assert_eq!(run2508_publication.1, 0x1111);
    assert!(run2508_publication.2);

    // The runner renders and advances publication history between source host
    // calls. Neither observation may expose or retire the staged completion;
    // its BG3 DMA has not executed yet.
    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[0x7c00]),
        0x1111,
    );
    state.advance_display_publication_history();
    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[0x7c00]),
        0x1111,
    );
    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletionStagedAfterSnapshot,
    );
    assert_eq!(
        state
            .display_snapshot
            .as_ref()
            .map(|snapshot| snapshot.publication_host_frame),
        Some(run2508_publication.0),
        "presenting run 2508 must not recapture its acceptance-owned scanout",
    );

    state.frame_ctr_dbg = 2509;
    state
        .install_original_timing_host_receipts(run2509_dialogue_receipts())
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(matches!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CopyingRemainingPixels { .. }
    ));
    let frozen = state
        .dialogue_scroll_frozen_scanout
        .as_ref()
        .expect("the adjacent source scroll must freeze the completed BG3 generation");
    assert_eq!(frozen.vram[0], 0x3000);
    assert_eq!(frozen.vram[0x3ef], 0x33ef);
    let run2509_first = state.with_display_snapshot(|display| {
        (
            display.ppu.vram[0x7c00],
            display.published_bg3_vwf_glyph_runs().to_vec(),
        )
    });
    let run2509_second = state.with_display_snapshot(|display| {
        (
            display.ppu.vram[0x7c00],
            display.published_bg3_vwf_glyph_runs().to_vec(),
        )
    });
    assert_eq!(run2509_first.0, 0x3000);
    assert_eq!(run2509_second, run2509_first);
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
}

#[test]
fn staged_dialogue_carry_rejects_held_or_wrong_subroutine_before_mutation() {
    fn assert_rejected_without_mutation(mut state: ZeldaState) {
        let phase_before = state.dialogue_scroll_phase();
        let staged_before = state.dialogue_scroll_completion_staged.clone();
        let snapshot_before = state.display_snapshot.as_ref().map(|snapshot| {
            (
                snapshot.ppu.vram.clone(),
                snapshot.accepts_nmi_dma_receipts,
                snapshot.effective_presented_dma.is_some(),
            )
        });
        let receipts_before = state.original_timing_semantic_receipts.clone();
        let gates_before = state.original_timing_expected_nmi_update_gates.clone();
        let pending_before = state.original_timing_nmi_publication_pending;
        let latch_before = state.game_state.display.nmi_update_is_latched();
        let ppu_before = state.ppu.vram.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = state.complete_original_timing_nmi_handler_for_active_scanout(
                OriginalTimingNmiHandlerCompletionOwner::PendingAtSequenceEntry,
                0,
                None,
            );
        }));
        assert!(result.is_err());
        assert_eq!(state.dialogue_scroll_phase(), phase_before);
        assert_eq!(state.dialogue_scroll_completion_staged, staged_before);
        assert_eq!(
            state.display_snapshot.as_ref().map(|snapshot| (
                snapshot.ppu.vram.clone(),
                snapshot.accepts_nmi_dma_receipts,
                snapshot.effective_presented_dma.is_some(),
            )),
            snapshot_before,
        );
        assert_eq!(state.original_timing_semantic_receipts, receipts_before);
        assert_eq!(
            state.original_timing_expected_nmi_update_gates,
            gates_before
        );
        assert_eq!(
            state.original_timing_nmi_publication_pending,
            pending_before
        );
        assert_eq!(
            state.game_state.display.nmi_update_is_latched(),
            latch_before,
        );
        assert_eq!(state.ppu.vram, ppu_before);
    }

    assert_rejected_without_mutation(staged_dialogue_carry_state(NmiUpdateGate::LatchHeld));

    let mut wrong_subroutine = staged_dialogue_carry_state(NmiUpdateGate::Open);
    wrong_subroutine.set_pending_nmi_subroutine(1);
    assert_rejected_without_mutation(wrong_subroutine);

    let mut wrong_disable = staged_dialogue_carry_state(NmiUpdateGate::Open);
    wrong_disable.set_core_update_disable_flag(1);
    assert_rejected_without_mutation(wrong_disable);

    let mut wrong_native_latch = staged_dialogue_carry_state(NmiUpdateGate::Open);
    wrong_native_latch.latch_nmi_update();
    assert_rejected_without_mutation(wrong_native_latch);
}

#[test]
fn staged_dialogue_carry_requires_a_receptive_snapshot_before_mutation() {
    let mut state = staged_dialogue_carry_state(NmiUpdateGate::Open);
    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .accepts_nmi_dma_receipts = false;
    let phase_before = state.dialogue_scroll_phase();
    let receipts_before = state.original_timing_semantic_receipts.clone();
    let gates_before = state.original_timing_expected_nmi_update_gates.clone();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = state.complete_original_timing_nmi_handler_for_active_scanout(
            OriginalTimingNmiHandlerCompletionOwner::PendingAtSequenceEntry,
            0,
            None,
        );
    }));
    assert!(result.is_err());
    assert_eq!(state.dialogue_scroll_phase(), phase_before);
    assert_eq!(state.original_timing_semantic_receipts, receipts_before);
    assert_eq!(
        state.original_timing_expected_nmi_update_gates,
        gates_before
    );
    assert!(state.original_timing_nmi_publication_pending);
}

#[test]
fn dialogue_text_dma_evidence_cannot_cross_snapshot_epochs_or_text_generations() {
    fn staged_state(word: u16) -> ZeldaState {
        let mut state = ZeldaState::new();
        state.capture_display_snapshot();
        state.stage_dialogue_scroll_completion_after_return(DialogueTextScanout {
            vram: vec![word; crate::PresentedDialogueText::WORD_COUNT],
            ..DialogueTextScanout::default()
        });
        state
    }

    let mut stale_epoch = staged_state(0x1234);
    let stale_token = DialogueTextDmaPublicationToken {
        snapshot_epoch: stale_epoch
            .display_snapshot
            .as_ref()
            .unwrap()
            .publication_epoch,
        words: vec![0x1234; crate::PresentedDialogueText::WORD_COUNT],
        metadata: PublishedDialogueMetadata::from_scanout(
            stale_epoch
                .dialogue_scroll_completion_staged
                .as_ref()
                .unwrap(),
        ),
    };
    stale_epoch.capture_display_snapshot();
    let stale_phase = stale_epoch.dialogue_scroll_phase();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        stale_epoch.complete_dialogue_scroll_after_text_dma_publication(stale_token);
    }));
    assert!(result.is_err());
    assert_eq!(stale_epoch.dialogue_scroll_phase(), stale_phase);

    let mut wrong_generation = staged_state(0x2345);
    let wrong_token = DialogueTextDmaPublicationToken {
        snapshot_epoch: wrong_generation
            .display_snapshot
            .as_ref()
            .unwrap()
            .publication_epoch,
        words: vec![0x3456; crate::PresentedDialogueText::WORD_COUNT],
        metadata: PublishedDialogueMetadata::from_scanout(
            wrong_generation
                .dialogue_scroll_completion_staged
                .as_ref()
                .unwrap(),
        ),
    };
    let wrong_phase = wrong_generation.dialogue_scroll_phase();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        wrong_generation.complete_dialogue_scroll_after_text_dma_publication(wrong_token);
    }));
    assert!(result.is_err());
    assert_eq!(wrong_generation.dialogue_scroll_phase(), wrong_phase);
}

#[test]
fn same_host_dialogue_acceptance_uses_capture_promotion_only_once() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(14);
    state.set_submodule(2);
    for index in 0..crate::PresentedDialogueText::WORD_COUNT {
        state.set_messaging_render_buffer_word(index, 0x5555);
    }
    let staged = state.dialogue_text_scanout_from_render_buffer();
    state.stage_dialogue_scroll_completion_after_return(staged);
    state.clear_nmi_update_latch();
    state.set_pending_nmi_subroutine(2);
    state.set_core_update_disable_flag(2);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            1,
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
            ],
        ))
        .unwrap();
    assert_eq!(
        state.take_original_timing_nmi_phases(),
        [
            OriginalTimingNmiPhase::Accepted(NmiUpdateGate::Open),
            OriginalTimingNmiPhase::HandlerCompleted,
        ],
    );
    state.begin_original_timing_host_dispatch(0);

    assert!(state
        .complete_original_timing_nmi_handler_for_active_scanout(
            OriginalTimingNmiHandlerCompletionOwner::AcceptedInThisSequence,
            0,
            None,
        )
        .completed());

    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletedScroll,
        "same-host acceptance capture must not be followed by a second carried-handler promotion",
    );
}

#[test]
fn carried_open_dialogue_completion_promotes_once_without_recapturing() {
    let mut state = staged_dialogue_carry_state(NmiUpdateGate::Open);
    let acceptance_publication = state
        .display_snapshot
        .as_ref()
        .map(|snapshot| (snapshot.publication_host_frame, snapshot.ppu.vram[0x7c00]))
        .unwrap();

    assert!(state
        .complete_original_timing_nmi_handler_for_active_scanout(
            OriginalTimingNmiHandlerCompletionOwner::PendingAtSequenceEntry,
            0,
            None,
        )
        .completed());

    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletedScroll,
    );
    let active = state.display_snapshot.as_ref().unwrap();
    assert_eq!(active.publication_host_frame, acceptance_publication.0);
    assert_eq!(active.ppu.vram[0x7c00], acceptance_publication.1);
    assert!(active.effective_presented_dma.is_some());
    let first = state.with_display_snapshot(|display| display.ppu.vram[0x7c00]);
    let second = state.with_display_snapshot(|display| display.ppu.vram[0x7c00]);
    assert_eq!((first, second), (0x4444, 0x4444));
    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletedScroll,
        "repeated presentation is observation, not another dialogue display boundary",
    );
}

#[test]
fn game_over_text_uploads_run_on_successive_leading_nmi_boundaries() {
    let mut frame = crate::game_state::FrameState::default();
    frame.main_module = 0x12;
    frame.submodule = 5;
    assert!(game_over_upload_pipeline_runs_after_leading_nmi(
        frame, 0, 31, 0,
    ));
    assert!(!game_over_upload_pipeline_runs_after_leading_nmi(
        frame, 1, 31, 0,
    ));
    frame.submodule = 6;
    assert!(game_over_upload_pipeline_runs_after_leading_nmi(
        frame, 12, 32, 22,
    ));
    frame.submodule = 7;
    assert!(game_over_upload_pipeline_runs_after_leading_nmi(
        frame, 12, 32, 11,
    ));
    assert!(!game_over_upload_pipeline_runs_after_leading_nmi(
        frame, 12, 32, 0,
    ));
}

#[test]
fn game_over_text_draw_authors_oam_without_rescheduling_the_chr_upload() {
    let mut state = ZeldaState::new();
    state.minigame_state_mut().set_flag_boomerang_in_place(1);
    state.ancilla_slot_view_mut(0).set_x_low(0x68);
    state.ancilla_slot_view_mut(1).set_x_low(0x98);
    state.set_pending_nmi_subroutine(0x77);

    state.GameOverText_Draw();

    assert_eq!(
        &state.sprite_oam_shadow_buffer()[..16],
        &[
            0x98, 0x57, 0x41, 0x3c, 0x98, 0x5f, 0x51, 0x3c, 0x68, 0x57, 0x40, 0x3c, 0x68, 0x5f,
            0x50, 0x3c,
        ]
    );
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0x77);
}

#[test]
fn game_over_text_outer_loop_defers_the_menu_until_all_five_calls_return() {
    let mut state = ZeldaState::new();
    state.set_main_module(0x12);
    state.set_submodule(8);
    state.game_over_text_render_calls_remaining = 5;

    for remaining_after_return in (1..=4).rev() {
        state.game_over_text_render_call_in_flight = true;
        state.finish_game_over_text_render_call();
        assert_eq!(state.game_state.frame.submodule, 8);
        assert_eq!(
            state.game_over_text_render_calls_remaining,
            remaining_after_return
        );
    }

    state.game_over_text_render_call_in_flight = true;
    state.finish_game_over_text_render_call();
    assert_eq!(state.game_state.frame.submodule, 9);
    assert_eq!(state.game_state.messaging.runtime.menu_animation_timer(), 2);
    assert_eq!(state.game_state.system_signals.music_control(), 11);
}

#[test]
fn game_over_text_cpu_slice_hides_stale_gameplay_oam_before_drawing_letters() {
    let mut state = ZeldaState::new();
    state
        .oam_state_mut()
        .write_indexed_entry_with_extended(102, 48, 117, 0, 0x1e, 2);
    state.prepare_game_over_text_oam();

    assert_eq!(state.sprite_oam_shadow_buffer()[102 * 4 + 1], 0xf0);
}

#[test]
fn dialogue_exit_bg_packet_waits_for_the_following_nmi() {
    let mut state = ZeldaState::new();
    state.set_bg_mode(9);
    state.write_vram_upload_buffer_word(0, 0xffff);
    state.set_bg_vram_load_mode(1);

    state.interrupt_nmi(0, None, true);
    assert_eq!(state.ram[NMI_LOAD_BG_FROM_VRAM], 1);

    state.clear_nmi_update_latch();
    state.interrupt_nmi(0, None, false);
    assert_eq!(state.ram[NMI_LOAD_BG_FROM_VRAM], 0);
}

#[test]
fn renderer_publication_exposes_consumed_dialogue_clear_without_advancing_menu_stripes() {
    let mut ordinary = ZeldaState::new();
    ordinary.ppu.vram[0] = 0x1111;
    ordinary.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    ordinary.capture_display_snapshot();
    ordinary.ppu.vram[0] = 0x2222;
    ordinary.ram[NMI_LOAD_BG_FROM_VRAM] = 0;

    assert_eq!(
        ordinary.with_display_snapshot(|display| display.ppu.vram[0]),
        0x1111
    );

    let mut dialogue_clear = ZeldaState::new();
    dialogue_clear.ppu.vram[0] = 0x3333;
    dialogue_clear.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    dialogue_clear.ram[crate::game_state::constants::VRAM_UPLOAD_DATA..][..8]
        .copy_from_slice(&[0x62, 0x44, 0x42, 0x2e, 0x7f, 0x38, 0xff, 0xff]);
    dialogue_clear.capture_display_snapshot();
    dialogue_clear.ppu.vram[0] = 0x4444;
    dialogue_clear.ram[NMI_LOAD_BG_FROM_VRAM] = 0;

    assert_eq!(
        dialogue_clear.with_display_snapshot(|display| display.ppu.vram[0]),
        0x4444
    );
}

#[test]
fn retained_display_memory_keeps_dialogue_metadata_with_its_vram_generation() {
    let mut state = ZeldaState::new();
    state.set_main_module(14);
    state.set_submodule(2);
    let pre_nmi_run = Bg3VwfGlyphRun {
        glyph_code: 0x41,
        origin_tile_number: 0x180,
        x: 4,
        y: -5,
        width: 3,
    };
    state.ppu.vram[0x7c00] = 0x1111;
    state.published_bg3_vwf_glyph_runs = vec![pre_nmi_run];
    state.published_bg3_vwf_glyph_run_dialogue_offsets = vec![0x2d];
    state.published_dialogue_msg_read_pos = 0x2d;
    state.published_dialogue_message_id = 32;
    state.capture_display_snapshot();

    let post_nmi_run = Bg3VwfGlyphRun {
        y: 0,
        ..pre_nmi_run
    };
    state.ppu.vram[0x7c00] = 0x2222;
    state.published_bg3_vwf_glyph_runs = vec![post_nmi_run];
    state.published_bg3_vwf_glyph_run_dialogue_offsets = vec![0x2e];
    state.published_dialogue_msg_read_pos = 0x2e;

    let captured = state.with_display_snapshot(|display| {
        (
            display.ppu.vram[0x7c00],
            display.published_bg3_vwf_glyph_runs().to_vec(),
            display
                .published_bg3_vwf_glyph_run_dialogue_offsets()
                .to_vec(),
            display.published_dialogue_msg_read_pos,
        )
    });

    assert_eq!(captured, (0x1111, vec![pre_nmi_run], vec![0x2d], 0x2d));
    assert_eq!(state.ppu.vram[0x7c00], 0x2222);
    assert_eq!(state.published_bg3_vwf_glyph_runs, vec![post_nmi_run]);
}

#[test]
fn recomposed_display_memory_publishes_post_nmi_dialogue_metadata_with_vram() {
    let mut state = ZeldaState::new();
    state.set_main_module(6);
    state.set_submodule(0);
    let pre_nmi_run = Bg3VwfGlyphRun {
        glyph_code: 0x41,
        origin_tile_number: 0x180,
        x: 4,
        y: -5,
        width: 3,
    };
    state.ppu.vram[0x7c00] = 0x1111;
    state.published_bg3_vwf_glyph_runs = vec![pre_nmi_run];
    state.published_bg3_vwf_glyph_run_dialogue_offsets = vec![0x2d];
    state.published_dialogue_msg_read_pos = 0x2d;
    state.published_dialogue_message_id = 32;
    state.capture_display_snapshot();

    let post_nmi_run = Bg3VwfGlyphRun {
        y: 0,
        ..pre_nmi_run
    };
    state.ppu.vram[0x7c00] = 0x2222;
    state.published_bg3_vwf_glyph_runs = vec![post_nmi_run];
    state.published_bg3_vwf_glyph_run_dialogue_offsets = vec![0x2e];
    state.published_dialogue_msg_read_pos = 0x2e;

    let captured = state.with_display_snapshot(|display| {
        (
            display.ppu.vram[0x7c00],
            display.published_bg3_vwf_glyph_runs().to_vec(),
            display
                .published_bg3_vwf_glyph_run_dialogue_offsets()
                .to_vec(),
            display.published_dialogue_msg_read_pos,
        )
    });

    assert_eq!(captured, (0x2222, vec![post_nmi_run], vec![0x2e], 0x2e));
    assert_eq!(state.ppu.vram[0x7c00], 0x2222);
    assert_eq!(state.published_bg3_vwf_glyph_runs, vec![post_nmi_run]);
}

#[test]
fn dialogue_scroll_completion_pairs_retired_dma_with_completion_metadata() {
    let mut state = ZeldaState::new();
    state.set_main_module(14);
    state.set_submodule(2);
    state.capture_display_snapshot();
    let scroll_run = Bg3VwfGlyphRun {
        glyph_code: 0x41,
        origin_tile_number: 0x180,
        x: 4,
        y: -5,
        width: 3,
    };
    state.stage_dialogue_scroll_completion_after_return(DialogueTextScanout {
        vram: vec![0x3333; 0x3f0],
        glyph_runs: vec![scroll_run],
        glyph_run_dialogue_offsets: vec![0x2d],
        dialogue_msg_read_pos: 0x2d,
        dialogue_message_id: 32,
    });
    state.capture_display_snapshot();
    publish_staged_dialogue_text_dma(&mut state);
    let staged = state.with_display_snapshot(|display| display.ppu.vram[0x7c00]);
    assert_eq!(staged, 0x3333);

    write_le_u16(&mut state.ram, 0x10000, 0x4444);
    state.bg3_vwf_glyph_runs = vec![scroll_run];
    state.bg3_vwf_glyph_run_dialogue_offsets = vec![0x2d];
    state.bg3_vwf_glyph_run_dialogue_message_id = 32;
    state.messaging_state_mut().set_dialogue_msg_read_pos(0x2d);
    state.capture_display_snapshot();

    let captured = state.with_display_snapshot(|display| {
        (
            display.ppu.vram[0x7c00],
            display.published_bg3_vwf_glyph_runs().to_vec(),
            display
                .published_bg3_vwf_glyph_run_dialogue_offsets()
                .to_vec(),
            display.published_dialogue_msg_read_pos,
            display.published_dialogue_message_id,
        )
    });

    assert_eq!(captured, (0x4444, vec![scroll_run], vec![0x2d], 0x2d, 32));
}
