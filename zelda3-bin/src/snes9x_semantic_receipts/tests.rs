use super::*;

const NMI_HANDLER_COMPLETE_PC: u32 = NMI_HANDLER_COMPLETE_PCS[0];

#[test]
fn desert_prayer_iris_exports_source_statement_progress() {
    assert_eq!(
        desert_prayer_iris_interruption(
            0x07eb31,
            Some(0x0e),
            Some(5),
            Some(2),
            Some(0x26),
            Some(22),
            Some(0),
            Some(3472),
            Some(3374),
            Some(130),
            Some(131),
            Some(255),
        )
        .unwrap(),
        Some(MainLoopInterruption::DesertPrayerIris {
            source_subsubmodule: 2,
            palette_countdown: 0,
            radius: 0x26,
            progress: zelda3::DesertPrayerIrisProgress::AfterPrimaryTableWrite {
                table_word: 87,
                y_buffer: 22,
            },
        }),
    );
    assert_eq!(
        desert_prayer_iris_interruption(
            0x07ed1d,
            Some(0x0e),
            Some(5),
            Some(4),
            Some(187),
            Some(69),
            Some(0),
            Some(3472),
            Some(3374),
            Some(0),
            Some(80),
            Some(255),
        )
        .unwrap(),
        Some(MainLoopInterruption::DesertPrayerIris {
            source_subsubmodule: 4,
            palette_countdown: 0,
            radius: 187,
            progress: zelda3::DesertPrayerIrisProgress::BeforePrimaryTableWrite {
                table_word: 40,
                y_buffer: 69,
            },
        }),
    );
    assert_eq!(
        desert_prayer_iris_interruption(
            0x07ea6b,
            Some(0x0e),
            Some(5),
            Some(4),
            Some(115),
            Some(1),
            Some(0),
            Some(3472),
            Some(3374),
            Some(0xfffb),
            Some(352),
            Some(0xff),
        )
        .unwrap(),
        Some(MainLoopInterruption::DesertPrayerIris {
            source_subsubmodule: 4,
            palette_countdown: 0,
            radius: 115,
            progress: zelda3::DesertPrayerIrisProgress::BeforeIteration { scanline: 0xfffb },
        }),
    );
    assert_eq!(
        desert_prayer_iris_interruption(
            0x07eb03,
            Some(0x0e),
            Some(5),
            Some(3),
            Some(0x26),
            Some(1),
            Some(11),
            Some(3472),
            Some(3374),
            Some(0xffff),
            Some(444),
            Some(255),
        )
        .unwrap(),
        Some(MainLoopInterruption::DesertPrayerIris {
            source_subsubmodule: 3,
            palette_countdown: 11,
            radius: 0x26,
            progress: zelda3::DesertPrayerIrisProgress::BeforePrimaryTableWrite {
                table_word: 222,
                y_buffer: 1,
            },
        }),
    );
    assert_eq!(
        desert_prayer_iris_interruption(
            0x07ea66,
            Some(0x0e),
            Some(5),
            Some(4),
            Some(0x26),
            Some(34),
            Some(0),
            Some(3472),
            Some(3374),
            Some(0x0100),
            Some(0x011a),
            Some(0x00ff),
        )
        .unwrap(),
        Some(MainLoopInterruption::DesertPrayerIris {
            source_subsubmodule: 4,
            palette_countdown: 0,
            radius: 0x26,
            progress: zelda3::DesertPrayerIrisProgress::BeforeIteration { scanline: 105 },
        }),
    );
    assert_eq!(
        desert_prayer_iris_interruption(
            0x07eaa1,
            Some(0x0e),
            Some(5),
            Some(4),
            Some(0x26),
            Some(35),
            Some(0),
            Some(3472),
            Some(3374),
            Some(0),
            Some(117),
            Some(0),
        )
        .unwrap(),
        Some(MainLoopInterruption::DesertPrayerIris {
            source_subsubmodule: 4,
            palette_countdown: 0,
            radius: 0x26,
            progress: zelda3::DesertPrayerIrisProgress::BeforePrimaryTableWrite {
                table_word: 74,
                y_buffer: 35,
            },
        }),
    );
    assert_eq!(
        desert_prayer_iris_interruption(
            0x07ea9e,
            Some(0x0e),
            Some(5),
            Some(4),
            Some(0x26),
            Some(35),
            Some(0),
            Some(3472),
            Some(3374),
            Some(0),
            Some(117),
            Some(0),
        )
        .unwrap(),
        Some(MainLoopInterruption::DesertPrayerIris {
            source_subsubmodule: 4,
            palette_countdown: 0,
            radius: 0x26,
            progress: zelda3::DesertPrayerIrisProgress::BeforeIteration { scanline: 106 },
        }),
    );
    assert_eq!(
        desert_prayer_iris_interruption(
            0x07ea77,
            Some(0x0e),
            Some(5),
            Some(4),
            Some(51),
            Some(40),
            Some(0),
            Some(3472),
            Some(3374),
            Some(98),
            Some(294),
            Some(200),
        )
        .unwrap(),
        Some(MainLoopInterruption::DesertPrayerIris {
            source_subsubmodule: 4,
            palette_countdown: 0,
            radius: 51,
            progress: zelda3::DesertPrayerIrisProgress::BeforeIteration { scanline: 98 },
        }),
    );
    assert!(desert_prayer_iris_interruption(
        0x07eb31,
        Some(0x0e),
        Some(5),
        Some(2),
        Some(0x25),
        Some(22),
        Some(0),
        Some(3472),
        Some(3374),
        Some(130),
        Some(131),
        Some(255),
    )
    .unwrap_err()
    .contains("expected $26"));
}

#[test]
fn desert_prayer_palette_filter_exports_next_source_color() {
    assert_eq!(
        desert_prayer_palette_filter_interruption(
            0x00e9e4,
            Some(0x0e),
            Some(5),
            Some(3),
            Some(0),
            Some(0x01aa),
        )
        .unwrap(),
        Some(MainLoopInterruption::DesertPrayerPaletteFilterBeforeColor {
            countdown: 0,
            next_color: 213,
        }),
    );
    assert!(desert_prayer_palette_filter_interruption(
        0x00e9e4,
        Some(0x0e),
        Some(5),
        Some(3),
        None,
        Some(0x01aa),
    )
    .unwrap_err()
    .contains("omitted source countdown"));
}

#[test]
fn semantic_trace_configuration_observes_acceptance_publication_and_context_resume() {
    assert!(REQUIRED_TRACE_EVENTS.contains(&"nmi"));
    assert!(REQUIRED_TRACE_EVENTS.contains(&"nmi-resume"));
    assert_eq!(NMI_UPDATE_LATCH, 0x0012);
    assert_eq!(ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC, 0x00805f);
    assert_eq!(NMI_HANDLER_COMPLETE_PCS, [0x0000_8225, 0x0000_82c7]);
    assert_eq!(
        append_csv(Some("dma,nmi"), REQUIRED_TRACE_EVENTS),
        "dma,nmi,frame,nmi-resume,wram,rom-rng,pc",
    );
}

#[test]
fn nmi_publication_emits_exact_zelda_joypad_bytes() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    source
        .consume_event(raw("nmi", Some(0x008036), None, None), &mut receipts)
        .unwrap();
    let mut completion = raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f8);
    completion.joypad_high = Some(0x05);
    completion.joypad_low = Some(0x80);
    completion.joypad_high_filtered = Some(0x04);
    completion.joypad_low_filtered = Some(0x80);
    source.consume_event(completion, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                high: 0x05,
                low: 0x80,
                high_filtered: 0x04,
                low_filtered: 0x80,
            }),
        ]
    );
}

#[test]
fn partial_nmi_joypad_publication_fails_before_committing_completion() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    source
        .consume_event(raw("nmi", Some(0x008036), None, None), &mut receipts)
        .unwrap();
    let mut completion = raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f8);
    completion.joypad_high = Some(0x05);
    completion.joypad_low = None;
    completion.joypad_high_filtered = None;
    completion.joypad_low_filtered = None;

    assert!(source
        .consume_event(completion, &mut receipts)
        .unwrap_err()
        .contains("omitted part"));
    assert!(source.nmi_publication_pending);
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open
        )]
    );
}

#[test]
fn semantic_trace_checkpoint_preserves_cross_host_nmi_ownership() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    source
        .consume_event(raw("nmi", Some(0x008036), None, None), &mut receipts)
        .unwrap();
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open
        )]
    );

    let bytes = serde_json::to_vec(&source.checkpoint()).unwrap();
    let checkpoint = serde_json::from_slice(&bytes).unwrap();
    let mut resumed = empty_semantic_tracker();
    resumed.restore_checkpoint(checkpoint).unwrap();
    let mut resumed_receipts = Vec::new();
    publish_nmi(&mut resumed, &mut resumed_receipts);

    assert_eq!(
        resumed_receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                high: 0,
                low: 0,
                high_filtered: 0,
                low_filtered: 0,
            }),
        ]
    );
    assert!(!resumed.nmi_publication_pending);
    assert_eq!(resumed.nmi_resume_targets, source.nmi_resume_targets);
}

#[test]
fn cross_host_latch_held_nmi_completes_without_joypad_publication() {
    let mut source = empty_semantic_tracker();
    let mut accepted = raw("nmi", Some(0x0c_ce3c), None, None);
    accepted.nmi_latch = Some(1);
    let mut receipts = Vec::new();
    source.consume_event(accepted, &mut receipts).unwrap();
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::LatchHeld,
        )],
    );

    let checkpoint =
        serde_json::from_slice(&serde_json::to_vec(&source.checkpoint()).unwrap()).unwrap();
    let mut resumed = empty_semantic_tracker();
    resumed.restore_checkpoint(checkpoint).unwrap();
    let mut completion = raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f8);
    // The trace event still carries Zelda's unchanged joypad bytes. A held
    // `$12` gate proves NMI_ReadJoypads did not publish them this handler.
    completion.joypad_high = Some(0xa5);
    completion.joypad_low = Some(0x5a);
    completion.joypad_high_filtered = Some(0x81);
    completion.joypad_low_filtered = Some(0x42);
    let mut resumed_receipts = Vec::new();
    resumed
        .consume_event(completion, &mut resumed_receipts)
        .unwrap();

    assert_eq!(
        resumed_receipts,
        vec![OriginalTimingSemanticReceipt::NmiHandlerCompleted],
    );
    assert!(!resumed.nmi_publication_pending);
    assert!(resumed.pending_nmi_update_gate.is_none());
}

#[test]
fn semantic_trace_checkpoint_preserves_native_mode_nmi_stack_context() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    source
        .consume_event(raw_at("nmi", 0x09fe65, 0x1f34), &mut receipts)
        .unwrap();
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open
        )]
    );

    let bytes = serde_json::to_vec(&source.checkpoint()).unwrap();
    let checkpoint = serde_json::from_slice(&bytes).unwrap();
    let mut resumed = empty_semantic_tracker();
    resumed.restore_checkpoint(checkpoint).unwrap();

    let mut resumed_receipts = Vec::new();
    publish_nmi(&mut resumed, &mut resumed_receipts);
    resumed
        .consume_event(
            raw_at("nmi-resume", 0x09fe65, 0x1f34),
            &mut resumed_receipts,
        )
        .unwrap();

    assert_eq!(
        resumed_receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                high: 0,
                low: 0,
                high_filtered: 0,
                low_filtered: 0,
            }),
        ]
    );
    assert!(resumed.nmi_resume_targets.is_empty());
}

#[test]
fn abandoned_poly_thread_nmi_context_is_retired_at_the_next_main_stack_acceptance() {
    // Route runs 965..967: the intro's parked poly context is interrupted at
    // `$09:FE65/S=$1F34`, module 1 begins on the main stack, and the next
    // NMI is accepted at the main-loop wait with no poly resume between.
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();
    tracker
        .consume_event(raw_at("nmi", 0x09fe65, 0x1f34), &mut receipts)
        .unwrap();
    assert_eq!(tracker.nmi_resume_targets, vec![(0x09fe65, 0x1f34)]);
    publish_nmi(&mut tracker, &mut receipts);
    receipts.clear();

    tracker
        .consume_event(raw_at("nmi", 0x008036, 0x01f8), &mut receipts)
        .unwrap();
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open
        )]
    );
    assert_eq!(tracker.nmi_resume_targets, vec![(0x008036, 0x01f8)]);
    publish_nmi(&mut tracker, &mut receipts);
    tracker
        .consume_event(raw_at("nmi-resume", 0x008036, 0x01f8), &mut receipts)
        .unwrap();
    assert!(tracker.nmi_resume_targets.is_empty());
}

#[test]
fn live_poly_thread_nmi_context_survives_its_own_resume_cycle() {
    // While the poly thread lives its parked context resumes before the
    // next acceptance; a poly-stack acceptance must not retire another.
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();
    tracker
        .consume_event(raw_at("nmi", 0x09fd81, 0x1f36), &mut receipts)
        .unwrap();
    publish_nmi(&mut tracker, &mut receipts);
    tracker
        .consume_event(raw_at("nmi-resume", 0x09fd81, 0x1f36), &mut receipts)
        .unwrap();
    assert!(tracker.nmi_resume_targets.is_empty());
    tracker
        .consume_event(raw_at("nmi", 0x09f81d, 0x1f3e), &mut receipts)
        .unwrap();
    assert_eq!(tracker.nmi_resume_targets, vec![(0x09f81d, 0x1f3e)]);
}

#[test]
fn sprite_main_nmi_exports_only_the_last_completed_source_slot() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(15), None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_SLOT_RETURN_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(14), None),
            &mut receipts,
        )
        .unwrap();

    // Persisting between the source slot return and interrupt must retain
    // the semantic loop cursor without exporting CPU state to gameplay.
    let checkpoint =
        serde_json::from_slice(&serde_json::to_vec(&source.checkpoint()).unwrap()).unwrap();
    let mut resumed = empty_semantic_tracker();
    resumed.restore_checkpoint(checkpoint).unwrap();
    resumed
        .consume_event(raw("nmi", Some(0x06_f80f), Some(14), None), &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainAfterSlot(15),
            ),
        ],
    );
}

#[test]
fn nmi_acceptance_decodes_complete_ppu_register_operands_and_rejects_absence() {
    let mut event = raw("nmi", None, None, None);
    event.nmi_ppu_register_operands = Some(std::array::from_fn(|index| index as u8));
    let operands = event.nmi_ppu_register_operands().unwrap();
    assert_eq!(operands.window_selection, [0, 1, 2]);
    assert_eq!(operands.fixed_color, [5, 6, 7]);
    assert_eq!(operands.screen_layers, [8, 9, 10, 11]);
    assert_eq!(
        operands.bg_scroll,
        [0x0d0c, 0x0f0e, 0x1110, 0x1312, 0x1514, 0x1716]
    );
    assert_eq!(operands.screen_brightness, 24);
    assert_eq!(operands.mosaic, 25);
    assert_eq!(operands.bg_mode, 26);
    assert_eq!(operands.mode7_center, [0x1c1b, 0x1e1d]);

    event.nmi_ppu_register_operands = None;
    assert!(event.nmi_ppu_register_operands().is_err());
}

#[test]
fn resumed_entry_nmi_preserves_sprite_progress_without_duplicate_interruption() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    for (pc, x) in [
        (SPRITE_MAIN_ENTRY_PC, None),
        (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(15)),
        (SPRITE_SLOT_RETURN_PC, None),
        (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(14)),
    ] {
        source
            .consume_event(raw("pc", Some(pc), x, None), &mut receipts)
            .unwrap();
    }

    source
        .consume_event(raw_at("nmi", 0x06_f80f, 0x01ff), &mut receipts)
        .unwrap();
    source
        .consume_event(raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f3), &mut receipts)
        .unwrap();
    source
        .consume_event(raw_at("nmi-resume", 0x06_f80f, 0x01ff), &mut receipts)
        .unwrap();

    source
        .consume_event(
            raw("pc", Some(SPRITE_SLOT_RETURN_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(13), None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(raw_at("nmi", 0x06_f80f, 0x01ff), &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::SpriteMainProgressed(SpriteMainProgress::AfterSlot(15),),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            zero_joypad_publication(),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainAfterSlot(14),
            ),
        ],
    );
}

#[test]
fn cross_host_nmi_resume_emits_progress_from_the_persistent_sprite_loop() {
    let mut source = empty_semantic_tracker();
    let mut previous_host = Vec::new();
    for (pc, x) in [
        (SPRITE_MAIN_ENTRY_PC, None),
        (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(15)),
        (SPRITE_SLOT_RETURN_PC, None),
        (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(14)),
    ] {
        source
            .consume_event(raw("pc", Some(pc), x, None), &mut previous_host)
            .unwrap();
    }
    source
        .consume_event(raw_at("nmi", 0x06_f80f, 0x01ff), &mut previous_host)
        .unwrap();
    assert_eq!(
        source
            .sprite_main_execution
            .map(|execution| execution.progress()),
        Some(SpriteMainProgress::AfterSlot(15)),
    );

    let checkpoint =
        serde_json::from_slice(&serde_json::to_vec(&source.checkpoint()).unwrap()).unwrap();
    let mut resumed = empty_semantic_tracker();
    resumed.restore_checkpoint(checkpoint).unwrap();
    assert_eq!(
        resumed
            .sprite_main_execution
            .map(|execution| execution.progress()),
        Some(SpriteMainProgress::AfterSlot(15)),
    );
    let mut current_host = Vec::new();
    resumed
        .consume_event(
            raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f3),
            &mut current_host,
        )
        .unwrap();
    assert_eq!(resumed.nmi_resume_targets, vec![(0x06_f80f, 0x01ff)]);
    assert_eq!(resumed.synthesized_nmi_resume, None);
    assert_eq!(
        resumed
            .sprite_main_execution
            .map(|execution| execution.progress()),
        Some(SpriteMainProgress::AfterSlot(15)),
    );
    resumed
        .consume_event(raw_at("nmi-resume", 0x06_f80f, 0x01ff), &mut current_host)
        .unwrap();

    assert_eq!(
        current_host,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            zero_joypad_publication(),
            OriginalTimingSemanticReceipt::SpriteMainProgressed(SpriteMainProgress::AfterSlot(15),),
        ],
    );
}

#[test]
fn host_return_publishes_the_active_sprite_main_checkpoint_without_an_nmi() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    for (pc, x) in [
        (SPRITE_MAIN_ENTRY_PC, None),
        (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(4)),
        (SPRITE_SLOT_RETURN_PC, None),
        (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(3)),
    ] {
        source
            .consume_event(raw("pc", Some(pc), x, None), &mut receipts)
            .unwrap();
    }

    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::AfterSlot(4),
        )],
    );
    assert_eq!(
        source
            .sprite_main_execution
            .map(|execution| execution.progress()),
        Some(SpriteMainProgress::AfterSlot(4)),
    );
}

#[test]
fn zero_hit_timer_clear_precedes_priority_store() {
    for pc in 0x06_8499..0x06_849c {
        let mut execution = SpriteMainExecutionTracker {
            current_slot: Some(12),
            last_completed_slot: Some(13),
            primary_timer_decrements_slot: Some(12),
            ..Default::default()
        };
        execution
            .observe_zero_hit_timer_clear(&raw("nmi", Some(pc), Some(12), None))
            .unwrap();
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::AfterZeroHitTimerClear(12)
        );
        execution
            .observe_hit_timer(&raw("nmi", Some(0x06_849c), Some(12), None))
            .unwrap();
        assert_eq!(execution.progress(), SpriteMainProgress::AfterHitTimer(12));
    }
}

#[test]
fn nmi_inside_aux1_load_publishes_only_main_countdown() {
    let mut execution = SpriteMainExecutionTracker {
        current_slot: Some(12),
        last_completed_slot: Some(13),
        ..Default::default()
    };
    let nmi = raw("nmi", Some(0x06_8429), Some(12), None);
    execution.observe_main_timer_decrement(&nmi).unwrap();
    execution.observe_zero_hit_timer_clear(&nmi).unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::AfterMainTimerDecrement(12)
    );
    assert_eq!(
        execution.interruption(),
        MainLoopInterruption::SpriteMainAfterMainTimerDecrement(12)
    );
    execution
        .observe_main_and_aux1_timer_decrements(&raw("nmi", Some(0x06_8432), Some(12), None))
        .unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::AfterMainAndAux1TimerDecrements(12)
    );
}

#[test]
fn nmi_inside_aux2_load_publishes_main_and_aux1_countdowns() {
    let mut execution = SpriteMainExecutionTracker {
        current_slot: Some(0),
        last_completed_slot: Some(1),
        ..SpriteMainExecutionTracker::default()
    };
    let nmi = raw(
        "nmi",
        Some(SPRITE_MAIN_AND_AUX1_TIMER_DECREMENTS_COMPLETE_START_PC + 2),
        Some(0),
        None,
    );

    execution
        .observe_main_and_aux1_timer_decrements(&nmi)
        .unwrap();

    assert_eq!(
        execution.progress(),
        SpriteMainProgress::AfterMainAndAux1TimerDecrements(0),
    );
    assert_eq!(
        execution.interruption(),
        MainLoopInterruption::SpriteMainAfterMainAndAux1TimerDecrements(0),
    );
}

#[test]
fn host_return_inside_hit_timer_load_publishes_primary_countdowns() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    for (pc, x) in [
        (SPRITE_MAIN_ENTRY_PC, None),
        (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(0)),
    ] {
        source
            .consume_event(raw("pc", Some(pc), x, None), &mut receipts)
            .unwrap();
    }
    let returned = raw(
        "frame",
        Some(SPRITE_PRIMARY_TIMER_DECREMENTS_COMPLETE_START_PC + 4),
        Some(0),
        None,
    );
    source
        .sprite_main_execution
        .as_mut()
        .unwrap()
        .observe_primary_timer_decrements(&returned)
        .unwrap();

    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::AfterPrimaryTimerDecrements(0),
        )],
    );
}

#[test]
fn host_return_coalesces_resumed_sprite_progress_to_the_latest_checkpoint() {
    let mut source = empty_semantic_tracker();
    source.sprite_main_execution = Some(SpriteMainExecutionTracker {
        current_slot: Some(2),
        last_completed_slot: Some(3),
        timers_and_oam_slot: None,
        timers_and_oam_dispatch_state: None,
        initialize_active_main_calls: 0,
        hog_spear_active_body: false,
        buzzblob_movement: None,
        trinexx_head_draw: None,
        trinexx_segment_counter: None,
        trinexx_head_draw_setup: None,
        trinexx_breath_tile_collision: None,
        handler_returned_slot: None,
        trinexx_final_phase_case0: None,
        trinexx_final_phase_tile_collision: None,
        helmasaur_hard_hat_tile_collision: None,
        trinexx_d_draw_counter: None,
        trinexx_d_draw_active: None,
        trinexx_final_phase_draw: None,
        sidenexx_neck_target: None,
        trinexx_front_part: None,
        guard_prep_parry_hitbox: None,
        guard_prep_patrol_delay: None,
        guard_prep_tile_collision_return: None,
        guard_animation_checkpoint: None,
        hog_spear_body_graphics_pending: None,
        absorbable_body_active: false,
        absorbable_horizontal_lookup: None,
        absorbable_vertical_lookup: None,
        absorbable_vertical_attribute_loaded: None,
        swamola_segment: None,
        dispatch_trampoline_return: None,
        vitreous_minions_seen: false,
        moblin_collision_started: false,
        moblin_collision_geometry: None,
        moblin_attribute_loaded: None,
        vitreous_player_damage_pending: None,
        vitreous_ai_pending: None,
        mini_moldorm_ai_pending: None,
        vitreous_damage_pending: None,
        swamola_head_prepared: false,
        swamola_head_draw_completed: None,
        swamola_head_draw: None,
        swamola_segment_draw: None,
        pengator_slide_pending: None,
        antifairy_bounce_pending: None,
        kholdstare_subtype_decremented: false,
        kholdstare_damage_pending: None,
        initialize_prep_pending: None,
        initialize_prep_move_y: None,
        guard_animation_pose_slot: None,
        guard_prep_weapon_flags_pending_slot: None,
        mini_moldorm_history: None,
        initialize_reset_properties: None,
        initialize_load_properties: None,
        fire_debirando_property_reload: false,
        fire_debirando_before_spawn_slot: None,
        fire_debirando_spawn: None,
        trinexx_death_spawn: None,
        agahnim_motion_blur_spawn: None,
        antfairy_subtype2_increment_slot: None,
        lanmola_subtype2_increment_slot: None,
        lanmola_draw_prefix: None,
        helmasaur_hard_hat_beetle_subtype2_increment_slot: None,
        timer_decrements_slot: None,
        primary_timer_decrements_slot: None,
        hit_timer_slot: None,
        main_and_aux1_timer_decrements_slot: None,
        main_timer_decrement_slot: None,
        zero_hit_timer_clear_slot: None,
        bari_before_random_slot: None,
        throwable_scenery_state_clear_slot: None,
        cucco_subtype_increments: None,
        cucco_helper_ordinal: 0,
        cucco_flee_movement: None,
        active_cucco_movement: None,
        active_cucco_x_publications: 0,
        active_cucco_y_subpixel: None,
        master_sword_light_beam_movement: None,
        boulder_movement: None,
        zora_fireball: None,
        laser_eye_draw_prologue: None,
        master_sword_light_beam_spawn: None,
        cucco_animation_slot: None,
        big_key_drop_graphics_slot: None,
        king_zora_flippers_graphics_slot: None,
        happiness_pond_rupee_graphics_slot: None,
        catfish_medallion_graphics_slot: None,
        waterfall_gt_cutscene_graphics_slot: None,
        bonk_item_graphics_slot: None,
        wish_pond_tossed_item_graphics_slot: None,
        single_small_draw_position_slot: None,
        probe_after_oam_coordinates_slot: None,
        wallmaster_reset_prefix_slot: None,
        wallmaster_reset_cleared_bytes: None,
        zazak_graphics_slot: None,
        follower_graphics: None,
    });
    let mut receipts = vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
        SpriteMainProgress::AfterSlot(4),
    )];

    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::AfterSlot(3),
        )],
    );
}

#[test]
fn big_key_type_publication_becomes_a_typed_partial_slot_checkpoint() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(2), None),
            &mut receipts,
        )
        .unwrap();
    let mut publication = raw(
        "wram-write",
        Some(BIG_KEY_DROP_TYPE_PUBLICATION_PC),
        Some(2),
        Some(SPRITE_TYPE_BASE + 2),
    );
    publication.value = Some(BIG_KEY_DROP_SPRITE_TYPE);
    source.consume_event(publication, &mut receipts).unwrap();

    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::BigKeyDropGraphicsStarted(2),
        )],
    );
    assert_eq!(
        source
            .sprite_main_execution
            .map(|execution| execution.interruption()),
        Some(MainLoopInterruption::SpriteMainBigKeyDropGraphicsStarted(2)),
    );
}

#[test]
fn king_zora_flippers_decoder_entry_becomes_a_typed_partial_slot_checkpoint() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(14), None),
            &mut receipts,
        )
        .unwrap();
    let mut graphics_entry = raw("pc", Some(DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC), None, None);
    graphics_entry.return_address = Some(ZORA_FLIPPERS_GRAPHICS_RETURN_ADDRESS);
    source.consume_event(graphics_entry, &mut receipts).unwrap();

    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::KingZoraFlippersGraphicsStarted(14),
        )],
    );
    assert_eq!(
        source
            .sprite_main_execution
            .map(|execution| execution.interruption()),
        Some(MainLoopInterruption::SpriteMainKingZoraFlippersGraphicsStarted(14)),
    );
}

#[test]
fn mini_moldorm_dispatch_checkpoint_ends_at_the_ai_target() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    for event in [
        raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(1), None),
        raw("pc", Some(0x06_9853), Some(1), None),
    ] {
        source.consume_event(event, &mut receipts).unwrap();
    }
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::MiniMoldormAiPending(1)
        )]
    );
    source
        .consume_event(raw("pc", Some(0x06_9860), Some(1), None), &mut receipts)
        .unwrap();
    assert_eq!(
        source
            .sprite_main_execution
            .unwrap()
            .mini_moldorm_ai_pending,
        None
    );
}

#[test]
fn pre_dungeon_disable_loop_keeps_its_sprite_reset_owner() {
    let mut event = raw("nmi", Some(0x09_c234), Some(4), None);
    event.main = Some(6);
    event.return_address = Some(0x09_c451);
    let mut receipts = Vec::new();
    assert!(publish_pre_dungeon_sprite_reset_progress(
        &event,
        OriginalTimingBoundary::NmiAccepted,
        false,
        &mut receipts
    )
    .unwrap());
    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::PreDungeonSpriteDisableThrough {
                slot: 4,
                boundary: OriginalTimingBoundary::NmiAccepted
            }
        ]
    );
    event.return_address = Some(0x09_c146);
    receipts.clear();
    assert!(!publish_pre_dungeon_sprite_reset_progress(
        &event,
        OriginalTimingBoundary::NmiAccepted,
        false,
        &mut receipts
    )
    .unwrap());
    assert!(receipts.is_empty());
}

#[test]
fn pre_dungeon_garnish_loop_keeps_its_reset_owner_and_store_cursor() {
    for (pc, x, slot) in [
        (0x09_c288, 29, 30),
        (0x09_c288, 14, 15),
        (0x09_c28c, 14, 14),
        (0x09_c28d, 13, 14),
        (0x09_c28d, 255, 0),
    ] {
        let mut event = raw("nmi", Some(pc), Some(x), None);
        event.main = Some(6);
        event.return_address = Some(0x09_c451);
        event.stack4 = Some(0x4b);
        let mut receipts = Vec::new();
        assert!(publish_pre_dungeon_sprite_reset_progress(
            &event,
            OriginalTimingBoundary::NmiAccepted,
            false,
            &mut receipts
        )
        .unwrap());
        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::PreDungeonGarnishDisableThrough {
                    slot,
                    boundary: OriginalTimingBoundary::NmiAccepted
                }
            ]
        );
        event.stack4 = Some(0x22);
        receipts.clear();
        assert!(!publish_pre_dungeon_sprite_reset_progress(
            &event,
            OriginalTimingBoundary::NmiAccepted,
            false,
            &mut receipts
        )
        .unwrap());
    }
}

#[test]
fn trinexx_final_phase_draw_checkpoints_name_segment_and_stage() {
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(0);
    let mut head = raw("nmi", Some(0x1d_b581), Some(0), None);
    head.y = Some(4);
    head.return_address = Some(0xd9_b069);
    head.stack4 = Some(0xad);
    tracker.observe_trinexx_final_phase_draw(&head).unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxFinalPhaseDraw {
            slot: 0,
            segment: 4,
            stage: 7,
        }
    );
    for (pc, x, y, ret, stack4, segment, stage) in [
        (0x1d_afa9, 0x13, 2, 0xad_d900, 0x1f, 2, 0),
        (0x1d_afd6, 0, 6, 0xad_d903, 0x1f, 3, 0),
        (0x1d_b01d, 0, 6, 0xad_d903, 0x1f, 3, 1),
        (0x1d_b02b, 0, 6, 0xad_d903, 0x1f, 3, 2),
        (0x1d_b034, 0, 3, 0x1f_add9, 0xc2, 3, 3),
        (0x1d_b03b, 0, 3, 0x1f_add9, 0xc2, 3, 4),
        (0x1d_b055, 0, 3, 0x1f_add9, 0xc2, 3, 5),
        (0x1d_b05c, 0, 1, 0x1f_add9, 0xc2, 1, 7),
        (0x1d_b060, 0, 1, 0x1f_add9, 0xc2, 2, 0),
        (0x1d_b06d, 0, 5, 0x1f_add9, 0xc2, 6, 0),
        (0x1d_b078, 0, 5, 0x1f_add9, 0xc2, 6, 0),
    ] {
        let mut event = raw("frame", Some(pc), Some(x), None);
        event.y = Some(y);
        event.return_address = Some(ret);
        event.stack4 = Some(stack4);
        tracker.observe_trinexx_final_phase_draw(&event).unwrap();
        assert_eq!(
            tracker.progress(),
            SpriteMainProgress::TrinexxFinalPhaseDraw {
                slot: 0,
                segment,
                stage,
            },
            "{pc:#x}"
        );
    }
    let mut counter = raw("wram-write", Some(0x1d_b06d), Some(0), Some(0x0fb6));
    counter.value = Some(5);
    tracker.observe_trinexx_final_phase_draw(&counter).unwrap();
    let mut loop_head = raw("frame", Some(0x1d_af94), Some(0), None);
    loop_head.y = Some(4);
    loop_head.return_address = Some(0x1f_add9);
    loop_head.stack4 = Some(0xc2);
    tracker
        .observe_trinexx_final_phase_draw(&loop_head)
        .unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxFinalPhaseDraw {
            slot: 0,
            segment: 5,
            stage: 0,
        }
    );
    let mut damage = raw("frame", Some(0x1d_b004), Some(0), None);
    damage.y = Some(6);
    damage.return_address = Some(0xad_d903);
    damage.stack4 = Some(0x1f);
    assert!(tracker.observe_trinexx_final_phase_draw(&damage).is_err());
    for (pc, y, ret, stack4, stage) in [
        (0x1d_b079, 4, 0x00_b045, 0x00, 16),
        (0x1d_b08f, 4, 0x15_8300, 0x4e, 17),
        (0x1d_b0a9, 4, 0x15_8300, 0x4e, 22),
        (0x1d_b0b7, 0, 0x83_084e, 0x45, 25),
        (0x1d_b0bc, 0, 0x45_4e08, 0xb0, 26),
        (0x1d_b0c0, 0, 0xb0_454e, 0x00, 27),
        (0x1d_b0c7, 0, 0x00_b045, 0x00, 29),
    ] {
        let mut event = raw("nmi", Some(pc), Some(0), None);
        event.y = Some(y);
        event.return_address = Some(ret);
        event.stack4 = Some(stack4);
        tracker.observe_trinexx_final_phase_draw(&event).unwrap();
        assert_eq!(
            tracker.progress(),
            SpriteMainProgress::TrinexxFinalPhaseDraw {
                slot: 0,
                segment: 4,
                stage,
            },
            "{pc:#x}"
        );
    }
    let mut inside = raw("nmi", Some(0x1d_b0aa), Some(0), None);
    inside.y = Some(4);
    assert!(tracker.observe_trinexx_final_phase_draw(&inside).is_err());
    // The single-large prep helper is only attributed once the loop's
    // `$0FB6 = 0` store proved this slot is inside Sprite_TrinexxD_Draw.
    let mut prep = raw("frame", Some(0x06_e423), Some(0), None);
    prep.y = Some(0);
    prep.return_address = Some(0xf5_dc12);
    prep.stack4 = Some(0xdb);
    tracker.observe_trinexx_final_phase_draw(&prep).unwrap();
    assert_eq!(tracker.trinexx_final_phase_draw, None);
    let mut start = raw("wram-write", Some(0x1d_af94), Some(0), Some(0x0fb6));
    start.value = Some(0);
    tracker.observe_trinexx_final_phase_draw(&start).unwrap();
    let mut advance = raw("wram-write", Some(0x1d_b06d), Some(0), Some(0x0fb6));
    advance.value = Some(2);
    tracker.observe_trinexx_final_phase_draw(&advance).unwrap();
    tracker.observe_trinexx_final_phase_draw(&prep).unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxFinalPhaseDraw {
            slot: 0,
            segment: 2,
            stage: 7,
        }
    );
    let mut wrapper = raw("frame", Some(0x06_dbf0), Some(0), None);
    wrapper.y = Some(2);
    wrapper.return_address = Some(0x1d_b05f);
    wrapper.stack4 = Some(0xd9);
    tracker.observe_trinexx_final_phase_draw(&wrapper).unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxFinalPhaseDraw {
            slot: 0,
            segment: 2,
            stage: 7,
        }
    );
    head.stack4 = Some(0xc2);
    assert!(tracker.observe_trinexx_final_phase_draw(&head).is_err());
}

#[test]
fn helmasaur_hard_hat_tile_collision_checkpoints_follow_the_probe_stack() {
    use SpriteTileCollisionStage as S;
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(0);
    let mut event = raw("frame", Some(0x06_e812), Some(0), None);
    event.return_address = Some(0x03_e5f0);
    event.stack4 = Some(0xe5);
    tracker
        .observe_helmasaur_hard_hat_tile_collision(&event)
        .unwrap();
    assert_eq!(tracker.helmasaur_hard_hat_tile_collision, None);
    tracker.helmasaur_hard_hat_beetle_subtype2_increment_slot = Some(0);
    for (pc, ret, stack4, stage) in [
        (0x06_e4ab, 0xa6_a491, 0x83, S::Entered),
        (0x06_e4ae, 0xa6_a491, 0x83, S::Cleared),
        (0x06_e501, 0xa6_a491, 0x83, S::Cleared),
        (0x06_e812, 0x03_e5f0, 0xe5, S::Cleared),
        (0x06_e890, 0xf0_e7a0, 0xe5, S::Cleared),
        (0x06_e50e, 0xa6_a491, 0x83, S::VerticalProbeDone),
        (0x06_e5b8, 0x91_e510, 0xa4, S::VerticalProbeDone),
        (0x06_e812, 0x10_e5ba, 0xe5, S::VerticalProbeDone),
        (0x06_e532, 0xa6_a491, 0x83, S::ProbesCompleted),
        (0x06_e890, 0x34_e7a0, 0xe5, S::ProbesCompleted),
        (0x06_e813, 0xe5_f000, 0x03, S::Cleared),
    ] {
        let mut event = raw("frame", Some(pc), Some(0), None);
        event.return_address = Some(ret);
        event.stack4 = Some(stack4);
        tracker
            .observe_helmasaur_hard_hat_tile_collision(&event)
            .unwrap();
        assert_eq!(
            tracker.progress(),
            SpriteMainProgress::HelmasaurHardHatTileCollision { slot: 0, stage },
            "{pc:#x}"
        );
    }
    let mut event = raw("frame", Some(0x06_e5fc), Some(0), None);
    event.return_address = Some(0x91_e503);
    event.stack4 = Some(0xa4);
    assert!(tracker
        .observe_helmasaur_hard_hat_tile_collision(&event)
        .is_err());
}

#[test]
fn trinexx_final_phase_tile_collision_checkpoints_follow_the_probe_stack() {
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(0);
    let mut attribute = raw("frame", Some(0x06_e883), Some(0), None);
    attribute.return_address = Some(0x34_e7a0);
    attribute.stack4 = Some(0xe5);
    tracker
        .observe_trinexx_final_phase_tile_collision(&attribute)
        .unwrap();
    assert_eq!(tracker.trinexx_final_phase_tile_collision, None);
    let mut store = raw("wram-write", Some(0x1d_ae77), Some(0), Some(0x0d90));
    store.value = Some(127);
    tracker
        .observe_trinexx_final_phase_tile_collision(&store)
        .unwrap();
    tracker
        .observe_trinexx_final_phase_tile_collision(&attribute)
        .unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxFinalPhaseTileCollision {
            slot: 0,
            probes_completed: true,
        }
    );
    let mut entry = raw("nmi", Some(0x06_e4ab), Some(0), None);
    entry.return_address = Some(0x1d_e49b);
    entry.stack4 = Some(0x97);
    tracker
        .observe_trinexx_final_phase_tile_collision(&entry)
        .unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxFinalPhaseTileCollision {
            slot: 0,
            probes_completed: false,
        }
    );
    let mut probe = raw("frame", Some(0x06_e79d), Some(0), None);
    probe.return_address = Some(0x9b_e534);
    probe.stack4 = Some(0xe4);
    tracker
        .observe_trinexx_final_phase_tile_collision(&probe)
        .unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxFinalPhaseTileCollision {
            slot: 0,
            probes_completed: true,
        }
    );
    attribute.stack4 = Some(0xe4);
    assert!(tracker
        .observe_trinexx_final_phase_tile_collision(&attribute)
        .is_err());
}

#[test]
fn trinexx_death_explosion_spawn_tracks_the_dynamic_spawn_progress() {
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(0);
    let mut store = raw(
        "wram-write",
        Some(SPRITE_SPAWN_DYNAMICALLY_FIRST_TYPE_STORE_PC),
        Some(0),
        Some(SPRITE_TYPE_BASE + 13),
    );
    store.y = Some(13);
    store.value = Some(0);
    store.return_address = Some(0x1d_dc35);
    store.stack4 = Some(0xaa);
    tracker.observe_trinexx_death_spawn_write(&store).unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxDeathExplosionSpawn {
            slot: 0,
            spawned_slot: 13,
            progress: SpriteDynamicSpawnProgress::TypePublished,
        }
    );
    let mut state = raw(
        "wram-write",
        Some(SPRITE_SPAWN_DYNAMICALLY_STATE_STORE_PC),
        Some(0),
        Some(SPRITE_STATE_BASE + 13),
    );
    state.y = Some(13);
    state.value = Some(9);
    tracker.observe_trinexx_death_spawn_write(&state).unwrap();
    let mut boundary = raw("nmi", Some(0x0d_b835), Some(13), None);
    boundary.return_address = Some(0xa6_0d1d);
    tracker
        .observe_trinexx_death_spawn_boundary(&boundary)
        .unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxDeathExplosionSpawn {
            slot: 0,
            spawned_slot: 13,
            progress: SpriteDynamicSpawnProgress::LoadProperties {
                completed_stores: 3
            },
        }
    );
    let mut other = SpriteMainExecutionTracker::default();
    other.current_slot = Some(0);
    store.stack4 = Some(0x12);
    other.observe_trinexx_death_spawn_write(&store).unwrap();
    assert_eq!(other.trinexx_death_spawn, None);
}

#[test]
fn trinexx_neck_delta_checkpoint_reads_the_traced_segment_counter() {
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(1);
    let mut event = raw("frame", Some(0x1d_bc3c), Some(1), None);
    event.return_address = Some(0x1f_b8dc);
    event.stack4 = Some(0xc2);
    assert!(tracker.observe_trinexx_head_draw(&event).is_err());
    let mut store = raw("wram-write", Some(0x1d_bc74), Some(1), Some(0x0fb5));
    store.value = Some(8);
    tracker.observe_trinexx_head_draw(&store).unwrap();
    tracker.observe_trinexx_head_draw(&event).unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxHeadDraw {
            slot: 1,
            segment: 8
        }
    );
    event.pc = Some(0x1d_bc07);
    tracker.observe_trinexx_head_draw(&event).unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxHeadDraw {
            slot: 1,
            segment: 8
        }
    );
    event.pc = Some(0x1d_bc37);
    assert!(tracker.observe_trinexx_head_draw(&event).is_err());
    store.value = Some(0);
    tracker.observe_trinexx_head_draw(&store).unwrap();
    tracker.observe_trinexx_head_draw(&event).unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxHeadDraw {
            slot: 1,
            segment: 0
        }
    );
}

#[test]
fn sidenexx_neck_target_checkpoints_count_loop_steps() {
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(1);
    for (pc, x, step) in [
        (0x1d_ba07, 9, 0),
        (0x1d_ba11, 12, 18),
        (0x1d_ba14, 12, 19),
        (0x1d_ba1d, 12, 20),
        (0x1d_ba31, 12, 21),
        (0x1d_ba39, 12, 22),
        (0x1d_ba46, 12, 23),
        (0x1d_ba4f, 12, 24),
        (0x1d_ba50, 16, 42),
        (0x1d_ba53, 18, 54),
        (0x1d_ba55, 18, 54),
    ] {
        let mut event = raw("frame", Some(pc), Some(x), None);
        event.return_address = Some(0xc2_1f01);
        event.stack4 = Some(0x06);
        tracker.observe_sidenexx_neck_target(&event).unwrap();
        assert_eq!(
            tracker.progress(),
            SpriteMainProgress::SidenexxNeckTargetLoop { slot: 1, step },
            "{pc:#x}"
        );
    }
    for (pc, step) in [(0x1d_ba56, 54), (0x1d_ba5a, 54), (0x1d_ba5d, 55)] {
        let mut event = raw("frame", Some(pc), Some(1), None);
        event.return_address = Some(0x06_c21f);
        tracker.observe_sidenexx_neck_target(&event).unwrap();
        assert_eq!(
            tracker.progress(),
            SpriteMainProgress::SidenexxNeckTargetLoop { slot: 1, step },
            "{pc:#x}"
        );
    }
    let mut event = raw("frame", Some(0x1d_ba50), Some(8), None);
    event.return_address = Some(0xc2_1f01);
    event.stack4 = Some(0x06);
    assert!(tracker.observe_sidenexx_neck_target(&event).is_err());
    event.x = Some(16);
    event.return_address = Some(0xc2_1f02);
    assert!(tracker.observe_sidenexx_neck_target(&event).is_err());
}

#[test]
fn trinexx_first_part_loop_checkpoints_count_the_entry_stores() {
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(2);
    for (pc, x, completed) in [
        (0x1d_bca6, 2, 0),
        (0x1d_bca8, 0, 0),
        (0x1d_bcab, 0, 0),
        (0x1d_bcb4, 3, 15),
        (0x1d_bcba, 3, 16),
        (0x1d_bcd1, 1, 7),
        (0x1d_bcd7, 4, 23),
        (0x1d_bcdf, 4, 24),
        (0x1d_bce9, 4, 25),
        (0x1d_bceb, 2, 10),
        (0x1d_bcef, 5, 25),
    ] {
        let mut event = raw("frame", Some(pc), Some(x), None);
        event.return_address = Some(0xbc_3902);
        event.stack4 = Some(0xdc);
        tracker.observe_trinexx_head_draw(&event).unwrap();
        assert_eq!(
            tracker.progress(),
            SpriteMainProgress::TrinexxHeadFrontPart {
                slot: 2,
                completed_stores: completed,
            },
            "{pc:#x}"
        );
    }
    let mut event = raw("frame", Some(0x1d_bca0), Some(2), None);
    event.return_address = Some(0xdc_bc39);
    event.stack4 = Some(0xb8);
    tracker.observe_trinexx_head_draw(&event).unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxHeadFrontPart {
            slot: 2,
            completed_stores: 0,
        }
    );
    let mut event = raw("frame", Some(0x1d_bcb4), Some(3), None);
    event.return_address = Some(0xbc_3903);
    event.stack4 = Some(0xdc);
    assert!(tracker.observe_trinexx_head_draw(&event).is_err());
    event.return_address = Some(0xbc_3902);
    event.x = Some(5);
    assert!(tracker.observe_trinexx_head_draw(&event).is_err());
}

#[test]
fn trinexx_first_part_tail_checkpoints_count_the_position_stores() {
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(2);
    for (pc, completed) in [
        (0x1d_bcf0, 25),
        (0x1d_bcf9, 26),
        (0x1d_bd08, 27),
        (0x1d_bd14, 28),
        (0x1d_bd1e, 29),
        (0x1d_bd25, 30),
    ] {
        let mut event = raw("nmi", Some(pc), Some(2), None);
        event.return_address = Some(0xdc_bc39);
        event.stack4 = Some(0xb8);
        tracker.observe_trinexx_head_draw(&event).unwrap();
        assert_eq!(
            tracker.progress(),
            SpriteMainProgress::TrinexxHeadFrontPart {
                slot: 2,
                completed_stores: completed,
            },
            "{pc:#x}"
        );
    }
    let mut event = raw("nmi", Some(0x1d_bd14), Some(2), None);
    event.return_address = Some(0xdc_bc39);
    event.stack4 = Some(0xbc);
    assert!(tracker.observe_trinexx_head_draw(&event).is_err());
}

#[test]
fn trinexx_neck_checkpoint_covers_the_whole_angle_calculation_block() {
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(1);
    let mut event = raw("frame", Some(0x1d_bbbf), Some(1), None);
    event.y = Some(9);
    event.return_address = Some(0xb8_dc01);
    tracker.observe_trinexx_head_draw(&event).unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxHeadDraw {
            slot: 1,
            segment: 0
        }
    );
    event.pc = Some(0x1d_bbdf);
    event.y = Some(13);
    tracker.observe_trinexx_head_draw(&event).unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxHeadDraw {
            slot: 1,
            segment: 4
        }
    );
    event.pc = Some(0x1d_bbe1);
    tracker.observe_trinexx_head_draw(&event).unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxHeadDraw {
            slot: 1,
            segment: 4
        }
    );
    // After PLX the cursor left Y; without a traced segment counter the
    // boundary is refused rather than guessed.
    event.pc = Some(0x1d_bbe2);
    event.return_address = Some(0x1f_b8dc);
    event.stack4 = Some(0xc2);
    assert!(tracker.observe_trinexx_head_draw(&event).is_err());
}

#[test]
fn trinexx_breath_tile_collision_checkpoint_spans_the_return_chain() {
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(12);
    let mut event = raw("frame", Some(0x06_e5b7), Some(12), None);
    event.y = Some(3);
    event.return_address = Some(0x1d_e49b);
    event.stack4 = Some(0x97);
    tracker
        .observe_trinexx_breath_tile_collision(&event)
        .unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxBreathTileCollisionReturned(12)
    );
    // A foreign caller on the shared collision helper is not the breath
    // checkpoint (cold route frame 378813).
    event.stack4 = Some(0xc9);
    tracker
        .observe_trinexx_breath_tile_collision(&event)
        .unwrap();
    assert_eq!(tracker.trinexx_breath_tile_collision, None);
    let mut event = raw("frame", Some(0x06_e49d), Some(12), None);
    event.return_address = Some(0x1d_8097);
    tracker
        .observe_trinexx_breath_tile_collision(&event)
        .unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxBreathTileCollisionReturned(12)
    );
    event.return_address = Some(0x05_b890);
    tracker
        .observe_trinexx_breath_tile_collision(&event)
        .unwrap();
    assert_eq!(tracker.trinexx_breath_tile_collision, None);
    let mut event = raw("frame", Some(0x1d_bd5f), Some(11), None);
    event.return_address = Some(0x1d_bdd5);
    assert!(tracker
        .observe_trinexx_breath_tile_collision(&event)
        .is_err());
    let event = raw("frame", Some(0x1d_bd64), Some(12), None);
    tracker
        .observe_trinexx_breath_tile_collision(&event)
        .unwrap();
    assert_ne!(
        tracker.progress(),
        SpriteMainProgress::TrinexxBreathTileCollisionReturned(12)
    );
}

#[test]
fn trinexx_head_draw_setup_checkpoint_requires_the_sidenexx_caller() {
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(2);
    let mut event = raw("frame", Some(0x06_84c0), Some(2), None);
    event.y = Some(1);
    event.return_address = Some(0x1d_bb8b);
    event.stack4 = Some(0xdc);
    tracker.observe_trinexx_head_draw(&event).unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxHeadDrawSetup(2)
    );
    event.return_address = Some(0x1d_bb6b);
    assert!(tracker.observe_trinexx_head_draw(&event).is_err());
    event.return_address = Some(0x1d_bb8b);
    event.x = Some(3);
    assert!(tracker.observe_trinexx_head_draw(&event).is_err());
    let mut event = raw("frame", Some(0x1d_bb8c), Some(2), None);
    event.return_address = Some(0x1f_b8dc);
    tracker.observe_trinexx_head_draw(&event).unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxHeadDrawSetup(2)
    );
}

#[test]
fn trinexx_front_part_checkpoint_proves_flags_written_and_extension_pending() {
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(2);
    let mut event = raw("frame", Some(0x1d_bce0), Some(3), None);
    event.y = Some(15);
    event.return_address = Some(0x39_020f);
    event.stack4 = Some(0xbc);
    tracker.observe_trinexx_head_draw(&event).unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxHeadFrontPart {
            slot: 2,
            completed_stores: 19
        }
    );
    event.stack4 = Some(0xbb);
    assert!(tracker.observe_trinexx_head_draw(&event).is_err());
}

#[test]
fn trinexx_neck_checkpoint_uses_saved_slot_and_cached_segment_index() {
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(2);
    let mut event = raw("frame", Some(0x1d_bbcf), Some(232), None);
    event.return_address = Some(0xb8_dc02);
    event.y = Some(22);
    tracker.observe_trinexx_head_draw(&event).unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxHeadDraw {
            slot: 2,
            segment: 4
        }
    );
    event.return_address = Some(0xb8_dc01);
    assert!(tracker.observe_trinexx_head_draw(&event).is_err());
    event.pc = Some(0x1d_bc7c);
    event.x = Some(2);
    event.a = Some(5);
    event.return_address = Some(0x1f_b8dc);
    tracker.observe_trinexx_head_draw(&event).unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::TrinexxHeadDraw {
            slot: 2,
            segment: 5
        }
    );
}

#[test]
fn link_velocity_clear_receipt_counts_completed_source_stores() {
    for (pc, completed) in [(0x07_e2cc, 1), (0x07_e2ce, 2), (0x07_e2d0, 3)] {
        assert_eq!(
            main_loop_interruption_for_source_state(pc, Some(0x0f), Some(1), None),
            Some(MainLoopInterruption::LinkVelocityClearProgress { completed })
        );
    }
}

#[test]
fn overworld_iris_reset_host_return_preserves_partial_stores() {
    assert_eq!(
        main_loop_interruption_for_source_state(0x00_f43e, Some(0x10), Some(1), Some(42)),
        Some(MainLoopInterruption::SpotlightGoalResetTable {
            completed_stores: 75
        })
    );
    let mut host = HostFrameWindow::default();
    host.observe(&frame_with_sub("entry", 1, 0x10, 1)).unwrap();
    let mut returned = frame_with_sub("return", 1, 0x10, 1);
    returned.pc = Some(0x00_f43e);
    returned.x = Some(42);
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();
    host.finish(&mut receipts, None, true).unwrap();
    assert!(
        receipts.contains(&OriginalTimingSemanticReceipt::MainLoopInterrupted(
            MainLoopInterruption::SpotlightGoalResetTable {
                completed_stores: 75
            }
        ))
    );
}
#[test]
fn happiness_pond_rupee_decoder_entry_becomes_a_typed_partial_slot_checkpoint() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(14), None),
            &mut receipts,
        )
        .unwrap();
    let mut graphics_entry = raw(
        "pc",
        Some(DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC),
        Some(4),
        None,
    );
    graphics_entry.return_address = Some(0x09_8b05);
    source.consume_event(graphics_entry, &mut receipts).unwrap();

    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::HappinessPondRupeeGraphicsStarted(14),
        )],
    );
    assert_eq!(
        source
            .sprite_main_execution
            .map(|execution| execution.interruption()),
        Some(MainLoopInterruption::SpriteMainHappinessPondRupeeGraphicsStarted(14)),
    );
}

#[test]
fn gt_cutscene_decode_becomes_a_waterfall_graphics_checkpoint() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(13), None),
            &mut receipts,
        )
        .unwrap();
    let mut decode = raw(
        "pc",
        Some(DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC),
        Some(255),
        None,
    );
    decode.return_address = Some(0x09_9bd5);
    source.consume_event(decode, &mut receipts).unwrap();
    assert_eq!(
        source.sprite_main_execution.as_ref().unwrap().progress(),
        SpriteMainProgress::WaterfallGtCutsceneGraphicsStarted(13)
    );
}

#[test]
fn catfish_medallion_decode_preserves_the_spawning_caller_slot() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(11), None),
            &mut receipts,
        )
        .unwrap();
    let mut decode = raw(
        "pc",
        Some(DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC),
        Some(11),
        None,
    );
    decode.return_address = Some(0x1d_e1a6);
    source.consume_event(decode, &mut receipts).unwrap();
    assert_eq!(
        source.sprite_main_execution.as_ref().unwrap().progress(),
        SpriteMainProgress::CatfishMedallionGraphicsStarted(11)
    );
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(10), None),
            &mut receipts,
        )
        .unwrap();
    assert_eq!(
        source
            .sprite_main_execution
            .as_ref()
            .unwrap()
            .catfish_medallion_graphics_slot,
        None
    );
}

#[test]
fn bonk_item_decoder_entry_becomes_a_typed_partial_slot_checkpoint() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(0), None),
            &mut receipts,
        )
        .unwrap();
    let mut graphics_entry = raw("pc", Some(DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC), None, None);
    graphics_entry.return_address = Some(BONK_ITEM_GRAPHICS_RETURN_ADDRESS);
    source.consume_event(graphics_entry, &mut receipts).unwrap();

    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::BonkItemGraphicsStarted(0),
        )],
    );
    assert_eq!(
        source
            .sprite_main_execution
            .map(|execution| execution.interruption()),
        Some(MainLoopInterruption::SpriteMainBonkItemGraphicsStarted(0)),
    );
}

#[test]
fn wish_pond_tossed_item_decoder_entry_retains_the_spawned_prefix() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(0), None),
            &mut receipts,
        )
        .unwrap();
    let mut graphics_entry = raw("pc", Some(DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC), None, None);
    graphics_entry.return_address = Some(WISH_POND_TOSSED_ITEM_GRAPHICS_RETURN_ADDRESS);
    source.consume_event(graphics_entry, &mut receipts).unwrap();

    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::WishPondTossedItemGraphicsStarted(0),
        )],
    );
    assert_eq!(
        source
            .sprite_main_execution
            .map(|execution| execution.interruption()),
        Some(MainLoopInterruption::SpriteMainWishPondTossedItemGraphicsStarted(0)),
    );
}

#[test]
fn shared_animated_decode_without_king_zora_return_address_is_not_flippers_progress() {
    let mut source = empty_semantic_tracker();
    source.sprite_main_execution = Some(SpriteMainExecutionTracker {
        current_slot: Some(14),
        ..SpriteMainExecutionTracker::default()
    });
    let mut receipts = Vec::new();
    let mut graphics_entry = raw("pc", Some(DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC), None, None);
    graphics_entry.return_address = Some(0x1d_e000);

    source.consume_event(graphics_entry, &mut receipts).unwrap();

    assert_eq!(
        source
            .sprite_main_execution
            .map(|execution| execution.progress()),
        Some(SpriteMainProgress::BeforeFirstSlot),
    );
    assert!(receipts.is_empty());
}

#[test]
fn ordinary_enemy_drop_type_does_not_enter_the_big_key_receipt_domain() {
    let mut source = empty_semantic_tracker();
    source.sprite_main_execution = Some(SpriteMainExecutionTracker {
        current_slot: Some(1),
        ..SpriteMainExecutionTracker::default()
    });
    let mut receipts = Vec::new();
    let mut publication = raw(
        "wram-write",
        Some(BIG_KEY_DROP_TYPE_PUBLICATION_PC),
        Some(1),
        Some(SPRITE_TYPE_BASE + 1),
    );
    publication.value = Some(0xd8);

    source.consume_event(publication, &mut receipts).unwrap();

    assert_eq!(
        source
            .sprite_main_execution
            .map(|execution| execution.progress()),
        Some(SpriteMainProgress::BeforeFirstSlot),
    );
    assert!(receipts.is_empty());
}

#[test]
fn sprite_main_slot_zero_then_common_return_closes_the_tracker_once() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    for (pc, x) in [
        (SPRITE_MAIN_ENTRY_PC, None),
        (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(0)),
        (SPRITE_SLOT_RETURN_PC, None),
        (SPRITE_MAIN_RETURN_PC, None),
    ] {
        source
            .consume_event(raw("pc", Some(pc), x, None), &mut receipts)
            .unwrap();
    }

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainReturned],
    );
    assert!(source.sprite_main_execution.is_none());
}

#[test]
fn cached_sprite_execute_single_after_sprite_main_does_not_reopen_the_loop() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    for (pc, x) in [
        (SPRITE_MAIN_ENTRY_PC, None),
        (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(0)),
        (SPRITE_SLOT_RETURN_PC, None),
        // ExecuteCachedSprites invokes the leaf directly after the
        // descending Sprite_Main loop has closed.
        (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(9)),
    ] {
        source
            .consume_event(raw("pc", Some(pc), x, None), &mut receipts)
            .unwrap();
    }

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainReturned],
    );
    assert!(source.sprite_main_execution.is_none());
}

#[test]
fn fresh_sprite_main_entry_proves_prior_item_receipt_caller_returned() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw(
                "pc",
                Some(BOTTLE_VENDOR_ITEM_RECEIPT_CALL_PC),
                Some(0),
                None,
            ),
            &mut receipts,
        )
        .unwrap();

    // A later top-level Sprite_Main entry cannot coexist with the prior
    // synchronous item call on the source CPU stack. It is therefore a
    // stronger semantic return proof than any caller-specific PC marker.
    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                ItemReceiptGraphicsProgressReceipt {
                    caller: ItemReceiptGraphicsCaller::SpriteMain { slot: 0 },
                    progress: SourceCallProgress::Returned,
                },
            ),
            OriginalTimingSemanticReceipt::SpriteMainReturned,
        ],
    );
    assert!(source.item_receipt_caller.is_none());
}

#[test]
fn sprite_main_nmi_exports_cucco_animation_publication_before_lift_tail_returns() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(6), None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw(
                "wram-write",
                Some(CUCCO_ANIMATION_PUBLICATION_PC),
                Some(6),
                Some(SPRITE_GRAPHICS_BASE + 6),
            ),
            &mut receipts,
        )
        .unwrap();

    // The semantic cursor is part of the paired oracle checkpoint. A
    // resume between the publication and the NMI must not regress to the
    // prior fully-returned slot.
    let checkpoint =
        serde_json::from_slice(&serde_json::to_vec(&source.checkpoint()).unwrap()).unwrap();
    let mut resumed = empty_semantic_tracker();
    resumed.restore_checkpoint(checkpoint).unwrap();
    resumed
        .consume_event(raw("nmi", Some(0x06_f80f), Some(6), None), &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainAfterCuccoGraphicsPublication {
                    slot: 6,
                    helper_ordinal: 0,
                },
            ),
        ],
    );
}

#[test]
fn sprite_main_nmi_after_cucco_flee_movement_keeps_the_subtype_helper_pending() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(2), None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(CUCCO_FLEE_SUBTYPE_HELPER_CALL_PC), Some(2), None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(raw("nmi", Some(0x06_a724), Some(2), None), &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainAfterCuccoFleeMovement {
                    slot: 2,
                    helper_ordinal: 0,
                },
            ),
        ],
    );
}

#[test]
fn sprite_main_host_return_after_active_cucco_x_keeps_y_movement_pending() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    for event in [
        raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(1), None),
        raw("pc", Some(ACTIVE_CUCCO_MOVEMENT_CALL_PC), Some(1), None),
        raw(
            "wram-write",
            Some(0x06_e94e),
            Some(0x11),
            Some(SPRITE_X_SUBPIXEL_BASE + 1),
        ),
        raw(
            "wram-write",
            Some(0x06_e964),
            Some(0x11),
            Some(SPRITE_X_LOW_BASE + 1),
        ),
        raw(
            "wram-write",
            Some(0x06_e96b),
            Some(0x11),
            Some(SPRITE_X_HIGH_BASE + 1),
        ),
    ] {
        source.consume_event(event, &mut receipts).unwrap();
    }
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::AfterActiveCuccoX {
                slot: 1,
                helper_ordinal: 0,
            },
        )],
    );
}

#[test]
fn master_sword_light_beam_exports_every_move_xy_assignment_prefix() {
    let checkpoints = [
        SpriteMoveXYCheckpoint::BeforeMovement,
        SpriteMoveXYCheckpoint::AfterXSubpixel,
        SpriteMoveXYCheckpoint::AfterXLow,
        SpriteMoveXYCheckpoint::AfterXHigh,
        SpriteMoveXYCheckpoint::AfterYSubpixel,
        SpriteMoveXYCheckpoint::AfterYLow,
        SpriteMoveXYCheckpoint::AfterYHigh,
    ];
    let addresses = [
        SPRITE_X_SUBPIXEL_BASE + 2,
        SPRITE_X_LOW_BASE + 2,
        SPRITE_X_HIGH_BASE + 2,
        SPRITE_Y_SUBPIXEL_BASE + 2,
        SPRITE_Y_LOW_BASE + 2,
        SPRITE_Y_HIGH_BASE + 2,
    ];
    for (completed, checkpoint) in checkpoints.into_iter().enumerate() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        source
            .consume_event(
                raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(2), None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(
                raw(
                    "pc",
                    Some(MASTER_SWORD_LIGHT_BEAM_MOVEMENT_CALL_PC),
                    Some(2),
                    None,
                ),
                &mut receipts,
            )
            .unwrap();
        for &address in &addresses[..completed] {
            source
                .consume_event(
                    raw("wram-write", Some(0x05_fa00), Some(2), Some(address)),
                    &mut receipts,
                )
                .unwrap();
        }
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::MasterSwordLightBeamMovement {
                    slot: 2,
                    checkpoint,
                },
            )],
        );
    }
}

#[test]
fn master_sword_light_beam_zero_x_velocity_starts_at_y_assignment() {
    let y_addresses = [
        SPRITE_Y_SUBPIXEL_BASE + 10,
        SPRITE_Y_LOW_BASE + 10,
        SPRITE_Y_HIGH_BASE + 10,
    ];
    let checkpoints = [
        SpriteMoveXYCheckpoint::AfterYSubpixel,
        SpriteMoveXYCheckpoint::AfterYLow,
        SpriteMoveXYCheckpoint::AfterYHigh,
    ];

    for (completed, checkpoint) in (1..=3).zip(checkpoints) {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        for event in [
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(10), None),
            raw(
                "pc",
                Some(MASTER_SWORD_LIGHT_BEAM_MOVEMENT_CALL_PC),
                Some(10),
                None,
            ),
        ] {
            source.consume_event(event, &mut receipts).unwrap();
        }
        for &address in &y_addresses[..completed] {
            source
                .consume_event(
                    raw("wram-write", Some(0x05_fa00), Some(10), Some(address)),
                    &mut receipts,
                )
                .unwrap();
        }
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::MasterSwordLightBeamMovement {
                    slot: 10,
                    checkpoint,
                },
            )],
        );
    }
}

#[test]
fn master_sword_replacement_spawn_exports_shared_helper_prefix() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    for event in [
        raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(5), None),
        raw(
            "pc",
            Some(MASTER_SWORD_LIGHT_BEAM_MOVEMENT_CALL_PC),
            Some(5),
            None,
        ),
    ] {
        source.consume_event(event, &mut receipts).unwrap();
    }

    let mut type_write = raw(
        "wram-write",
        Some(SPRITE_SPAWN_DYNAMICALLY_FIRST_TYPE_STORE_PC),
        Some(5),
        Some(SPRITE_TYPE_BASE + 3),
    );
    type_write.y = Some(3);
    type_write.value = Some(0x62);
    source.consume_event(type_write, &mut receipts).unwrap();

    let mut state_write = raw(
        "wram-write",
        Some(SPRITE_SPAWN_DYNAMICALLY_STATE_STORE_PC),
        Some(5),
        Some(SPRITE_STATE_BASE + 3),
    );
    state_write.y = Some(3);
    state_write.value = Some(9);
    source.consume_event(state_write, &mut receipts).unwrap();

    source
        .consume_event(
            raw("wram-write", Some(0x0d_b877), Some(3), Some(0x0e93)),
            &mut receipts,
        )
        .unwrap();
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::MasterSwordLightBeamSpawn {
                slot: 5,
                spawned_slot: 3,
                progress: SpriteDynamicSpawnProgress::ResetProperties {
                    completed_stores: 2,
                },
            },
        )],
    );
}

#[test]
fn dynamic_spawn_outdoor_identity_publishes_after_atomic_word_store() {
    let mut tracker = Some((5, 4, SpriteDynamicSpawnProgress::StatePublished));
    let mut low = raw(
        "wram-write",
        Some(SPRITE_SPAWN_DYNAMICALLY_IDENTITY_STORE_PC),
        Some(4),
        Some(SPRITE_N_BASE + 8),
    );
    low.value = Some(0xff);
    observe_dynamic_spawn_progress_write(&mut tracker, &low, "test").unwrap();
    assert_eq!(
        tracker,
        Some((5, 4, SpriteDynamicSpawnProgress::StatePublished)),
    );

    let mut high = raw(
        "wram-write",
        Some(SPRITE_SPAWN_DYNAMICALLY_IDENTITY_STORE_PC),
        Some(4),
        Some(SPRITE_N_BASE + 9),
    );
    high.value = Some(0xff);
    observe_dynamic_spawn_progress_write(&mut tracker, &high, "test").unwrap();
    assert_eq!(
        tracker,
        Some((5, 4, SpriteDynamicSpawnProgress::IdentityPublished)),
    );
}

#[test]
fn single_small_draw_nmi_exports_the_published_position_prefix() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    for event in [
        raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(15), None),
        raw(
            "nmi",
            Some(SPRITE_SINGLE_SMALL_AFTER_POSITION_PC),
            Some(15),
            None,
        ),
    ] {
        source.consume_event(event, &mut receipts).unwrap();
    }
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainAfterSingleSmallDrawPosition(15),
            ),
            OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::AfterSingleSmallDrawPosition(15),
            ),
        ],
    );
}

#[test]
fn guard_probe_nmi_exports_the_completed_oam_coordinate_prefix() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    for event in [
        raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(8), None),
        raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(8), None),
        raw(
            "nmi",
            Some(SPRITE_PROBE_AFTER_OAM_COORDINATES_PC),
            Some(8),
            None,
        ),
    ] {
        source.consume_event(event, &mut receipts).unwrap();
    }
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainProbeAfterOamCoordinates(8),
            ),
            OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::ProbeAfterOamCoordinates(8),
            ),
        ],
    );
}

#[test]
fn state8_property_reset_nmi_exports_the_exact_completed_store_prefix() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(1), None),
            &mut receipts,
        )
        .unwrap();
    let mut timers_return = raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(1), None);
    timers_return.stack1 = Some(8);
    source.consume_event(timers_return, &mut receipts).unwrap();
    let mut nmi = raw("nmi", Some(0x0db8ad), Some(1), None);
    nmi.return_address = Some(SPRITE_PREP_LOAD_PROPERTIES_AFTER_RESET_RETURN_ADDRESS);
    source.consume_event(nmi, &mut receipts).unwrap();
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainInitializeResetProperties {
                    slot: 1,
                    phase: SpriteInitializeResetPropertiesPhase::InitialPropertyLoad,
                    completed_stores: 20,
                },
            ),
            OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::InitializeResetProperties {
                    slot: 1,
                    phase: SpriteInitializeResetPropertiesPhase::InitialPropertyLoad,
                    completed_stores: 20,
                },
            ),
        ],
    );
}

#[test]
fn fire_debirando_nested_property_reset_exports_its_source_phase() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(1), None),
            &mut receipts,
        )
        .unwrap();
    let mut timers_return = raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(1), None);
    timers_return.stack1 = Some(8);
    source.consume_event(timers_return, &mut receipts).unwrap();
    let mut conversion = raw(
        "wram-write",
        Some(SPRITE_PREP_FIRE_DEBIRANDO_TYPE_STORE_PC),
        Some(1),
        Some(SPRITE_TYPE_BASE + 1),
    );
    conversion.value = Some(0x63);
    source.consume_event(conversion, &mut receipts).unwrap();
    let mut nmi = raw("nmi", Some(0x0db874), Some(1), None);
    nmi.return_address = Some(SPRITE_PREP_LOAD_PROPERTIES_AFTER_RESET_RETURN_ADDRESS);
    source.consume_event(nmi, &mut receipts).unwrap();
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainInitializeResetProperties {
                    slot: 1,
                    phase: SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion,
                    completed_stores: 1,
                },
            ),
            OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::InitializeResetProperties {
                    slot: 1,
                    phase: SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion,
                    completed_stores: 1,
                },
            ),
        ],
    );
}

#[test]
fn fire_debirando_property_load_exports_its_source_store_cursor() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(0), None),
            &mut receipts,
        )
        .unwrap();
    let mut timers_return = raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(0), None);
    timers_return.stack1 = Some(8);
    source.consume_event(timers_return, &mut receipts).unwrap();
    let mut conversion = raw(
        "wram-write",
        Some(SPRITE_PREP_FIRE_DEBIRANDO_TYPE_STORE_PC),
        Some(0),
        Some(SPRITE_TYPE_BASE),
    );
    conversion.value = Some(0x63);
    source.consume_event(conversion, &mut receipts).unwrap();
    let returned = raw(
        "frame",
        Some(SPRITE_PREP_LOAD_PROPERTIES_AFTER_FLAGS3_PC),
        Some(0),
        None,
    );
    source
        .sprite_main_execution
        .as_mut()
        .unwrap()
        .observe_initialize_load_properties(&returned)
        .unwrap();
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::InitializeLoadProperties {
                slot: 0,
                phase: SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion,
                completed_stores: 9,
            },
        )],
    );
}

#[test]
fn fire_debirando_spawn_scan_exports_the_completed_initializer_prefix() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(0), None),
            &mut receipts,
        )
        .unwrap();
    let mut timers_return = raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(0), None);
    timers_return.stack1 = Some(8);
    source.consume_event(timers_return, &mut receipts).unwrap();
    let mut conversion = raw(
        "wram-write",
        Some(SPRITE_PREP_FIRE_DEBIRANDO_TYPE_STORE_PC),
        Some(0),
        Some(SPRITE_TYPE_BASE),
    );
    conversion.value = Some(0x63);
    source.consume_event(conversion, &mut receipts).unwrap();
    let mut returned = raw(
        "frame",
        Some(SPRITE_SPAWN_DYNAMICALLY_FIRST_TYPE_STORE_PC),
        Some(0),
        None,
    );
    returned.return_address = Some(SPRITE_PREP_FIRE_DEBIRANDO_SPAWN_RETURN_ADDRESS);
    source
        .sprite_main_execution
        .as_mut()
        .unwrap()
        .observe_fire_debirando_before_spawn(&returned)
        .unwrap();
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::FireDebirandoBeforeSpawn(0),
        )],
    );
}

#[test]
fn fire_debirando_dynamic_spawn_exports_the_exact_source_publication() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(0), None),
            &mut receipts,
        )
        .unwrap();
    let mut timers_return = raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(0), None);
    timers_return.stack1 = Some(8);
    source.consume_event(timers_return, &mut receipts).unwrap();
    let mut conversion = raw(
        "wram-write",
        Some(SPRITE_PREP_FIRE_DEBIRANDO_TYPE_STORE_PC),
        Some(0),
        Some(SPRITE_TYPE_BASE),
    );
    conversion.value = Some(0x63);
    source.consume_event(conversion, &mut receipts).unwrap();

    let mut child_type = raw(
        "wram-write",
        Some(SPRITE_SPAWN_DYNAMICALLY_FIRST_TYPE_STORE_PC),
        Some(0),
        Some(SPRITE_TYPE_BASE + 15),
    );
    child_type.y = Some(15);
    child_type.value = Some(0x64);
    source.consume_event(child_type, &mut receipts).unwrap();
    let mut child_state = raw(
        "wram-write",
        Some(SPRITE_SPAWN_DYNAMICALLY_STATE_STORE_PC),
        Some(0),
        Some(SPRITE_STATE_BASE + 15),
    );
    child_state.y = Some(15);
    child_state.value = Some(9);
    source.consume_event(child_state, &mut receipts).unwrap();
    let mut child_floor = raw(
        "wram-write",
        Some(SPRITE_SPAWN_DYNAMICALLY_FLOOR_STORE_PC),
        Some(0),
        Some(SPRITE_FLOOR_BASE + 15),
    );
    child_floor.y = Some(15);
    source.consume_event(child_floor, &mut receipts).unwrap();
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::FireDebirandoSpawn {
                slot: 0,
                spawned_slot: 15,
                progress: SpriteDynamicSpawnProgress::FloorPublished,
            },
        )],
    );
}

#[test]
fn timer_oam_return_nmi_exports_the_generic_completed_prefix() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    for event in [
        raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(10), None),
        raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(10), None),
        raw("nmi", Some(0x069276), Some(10), None),
    ] {
        source.consume_event(event, &mut receipts).unwrap();
    }
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainAfterTimersAndOam(10),
            ),
            OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::AfterTimersAndOam(10),
            ),
        ],
    );
}

#[test]
fn mini_moldorm_nmi_exports_exact_history_store_progress() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(0), None),
            &mut receipts,
        )
        .unwrap();
    let mut timers_return = raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(0), None);
    timers_return.stack1 = Some(8);
    source.consume_event(timers_return, &mut receipts).unwrap();
    let mut nmi = raw(
        "nmi",
        Some(SPRITE_PREP_MINI_MOLDORM_HISTORY_X_HIGH_LOAD_PC),
        Some(24),
        None,
    );
    nmi.y = Some(0);
    source.consume_event(nmi, &mut receipts).unwrap();
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    let progress = SpriteMainProgress::MiniMoldormHistory {
        slot: 0,
        completed_stores: 99,
    };
    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainMiniMoldormHistory {
                    slot: 0,
                    completed_stores: 99,
                },
            ),
            OriginalTimingSemanticReceipt::SpriteMainProgressed(progress),
        ],
    );
}

#[test]
fn guard_prep_host_return_exports_the_first_active_calls_pending_weapon_flags() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(12), None),
            &mut receipts,
        )
        .unwrap();
    let mut timers_return = raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(12), None);
    timers_return.stack1 = Some(8);
    source.consume_event(timers_return, &mut receipts).unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_ACTIVE_MAIN_ENTRY_PC), Some(12), None),
            &mut receipts,
        )
        .unwrap();
    let mut weapon_flags_store = raw(
        "wram-write",
        Some(GUARD_ANIMATE_WEAPON_FLAGS_STORE_PC),
        Some(0),
        Some(0x0803),
    );
    weapon_flags_store.sub = Some(0);
    source
        .consume_event(weapon_flags_store, &mut receipts)
        .unwrap();

    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::GuardPrepWeaponFlagsPending(12),
        )],
    );
}

#[test]
fn active_guard_weapon_nmi_exports_the_unfinished_entry() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(10), None),
            &mut receipts,
        )
        .unwrap();
    let mut timers_return = raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(10), None);
    timers_return.stack1 = Some(9);
    source.consume_event(timers_return, &mut receipts).unwrap();
    let mut nmi = raw("nmi", Some(0x05_cbaa), Some(34), None);
    let mut pose = raw(
        "wram-write",
        Some(0x05_c240),
        Some(10),
        Some(SPRITE_GRAPHICS_BASE + 10),
    );
    pose.value = Some(8);
    source.consume_event(pose, &mut receipts).unwrap();
    nmi.y = Some(21);
    source.consume_event(nmi, &mut receipts).unwrap();
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);
    assert!(
        receipts.contains(&OriginalTimingSemanticReceipt::MainLoopInterrupted(
            MainLoopInterruption::SpriteMainGuardAnimation {
                slot: 10,
                checkpoint: zelda3::GuardAnimationCheckpoint::WeaponCoordinates { entry: 1 }
            }
        ))
    );
    assert!(
        receipts.contains(&OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::GuardAnimation {
                slot: 10,
                checkpoint: zelda3::GuardAnimationCheckpoint::WeaponCoordinates { entry: 1 }
            }
        ))
    );
}

#[test]
fn guard_head_flags_boundary_requires_the_temporary_pose_caller() {
    let mut tracker = SpriteMainExecutionTracker {
        current_slot: Some(11),
        timers_and_oam_dispatch_state: Some(9),
        ..Default::default()
    };
    let mut event = raw("nmi", Some(0x05_c71c), Some(0), None);
    event.y = Some(2);
    tracker.observe_guard_animation_checkpoint(&event).unwrap();
    assert_eq!(tracker.guard_animation_checkpoint, None);
    tracker.guard_animation_pose_slot = Some(11);
    tracker.observe_guard_animation_checkpoint(&event).unwrap();
    assert_eq!(
        tracker.guard_animation_checkpoint,
        Some((11, zelda3::GuardAnimationCheckpoint::HeadFlagsPending))
    );
}

#[test]
fn hog_spear_body_return_requires_nested_initializer_authority() {
    for active_call in 1..=2 {
        let mut tracker = SpriteMainExecutionTracker {
            current_slot: Some(12),
            timers_and_oam_dispatch_state: Some(8),
            initialize_active_main_calls: active_call,
            ..Default::default()
        };
        let mut event = raw("frame", Some(0x05_cab0), Some(0), None);
        event.sub = Some(0);
        event.return_address = Some(0xc6880c);
        tracker.observe_guard_animation_checkpoint(&event).unwrap();
        assert_eq!(tracker.guard_animation_checkpoint, None);
        tracker
            .observe_guard_animation_checkpoint(&raw("pc", Some(0x05_cbe0), Some(12), None))
            .unwrap();
        tracker.observe_guard_animation_checkpoint(&event).unwrap();
        assert_eq!(
            tracker.guard_animation_checkpoint,
            Some((
                12,
                zelda3::GuardAnimationCheckpoint::HogSpearInitializerBodyReturned { active_call }
            ))
        );
        event.return_address = Some(0xc6880b);
        assert!(tracker.observe_guard_animation_checkpoint(&event).is_err());
    }
}

#[test]
fn buzzblob_subpixel_boundary_requires_its_normal_movement_call() {
    let mut tracker = SpriteMainExecutionTracker {
        current_slot: Some(11),
        ..Default::default()
    };
    let boundary = raw("nmi", Some(0x06_e94e), Some(27), None);
    tracker.observe_buzzblob_movement(&boundary).unwrap();
    assert_eq!(tracker.buzzblob_movement, None);
    tracker
        .observe_buzzblob_movement(&raw("pc", Some(0x06_d8e2), Some(11), None))
        .unwrap();
    tracker.observe_buzzblob_movement(&boundary).unwrap();
    assert_eq!(tracker.buzzblob_movement, Some((11, true)));
    let mut wrong_axis = boundary;
    wrong_axis.x = Some(11);
    assert!(tracker.observe_buzzblob_movement(&wrong_axis).is_err());
    wrong_axis.event = "wram-write".into();
    tracker.observe_buzzblob_movement(&wrong_axis).unwrap();
    tracker
        .observe_buzzblob_movement(&raw("pc", Some(0x06_d8e5), Some(11), None))
        .unwrap();
    assert_eq!(tracker.buzzblob_movement, None);
}

#[test]
fn pengator_slide_entry_proves_movement_before_the_sparkle_rng() {
    let mut tracker = SpriteMainExecutionTracker {
        current_slot: Some(6),
        timers_and_oam_dispatch_state: Some(9),
        ..Default::default()
    };
    let mut event = raw("nmi", Some(0x1e_a279), None, None);
    event.x = Some(6);
    tracker.observe_pengator_slide_pending(&event).unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::PengatorSlidePending(6)
    );
    event.x = Some(5);
    assert!(tracker.observe_pengator_slide_pending(&event).is_err());
}

#[test]
fn kholdstare_damage_checkpoint_requires_body_and_hitbox_caller() {
    let mut tracker = SpriteMainExecutionTracker {
        current_slot: Some(4),
        timers_and_oam_dispatch_state: Some(9),
        ..Default::default()
    };
    let mut endpoint = raw("frame", Some(0x06_f839), None, None);
    endpoint.x = Some(1);
    endpoint.stack1 = Some(4);
    endpoint.return_address = Some(0xf2_d004);
    tracker
        .observe_kholdstare_damage_pending(&endpoint)
        .unwrap();
    assert_eq!(tracker.kholdstare_damage_pending, None);
    let mut decrement = raw("wram-write", Some(0x1e_953a), None, None);
    decrement.x = Some(4);
    decrement.address = Some(0x0e84);
    tracker
        .observe_kholdstare_damage_pending(&decrement)
        .unwrap();
    tracker
        .observe_kholdstare_damage_pending(&endpoint)
        .unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::KholdstareDamagePending(4)
    );
    endpoint.stack1 = Some(3);
    assert!(tracker
        .observe_kholdstare_damage_pending(&endpoint)
        .is_err());
}

#[test]
fn antifairy_bounce_checkpoint_supersedes_draw_progress_for_its_caller_only() {
    let mut tracker = SpriteMainExecutionTracker {
        current_slot: Some(0),
        timers_and_oam_dispatch_state: Some(9),
        antfairy_subtype2_increment_slot: Some(0),
        ..Default::default()
    };
    let mut event = raw("nmi", Some(0x1d_c778), None, None);
    event.x = Some(0);
    event.return_address = Some(0x06_a53d);
    tracker.observe_antifairy_bounce_pending(&event).unwrap();
    assert_eq!(tracker.antifairy_bounce_pending, None);
    event.return_address = Some(0x06_a53e);
    tracker.observe_antifairy_bounce_pending(&event).unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::AntifairyBouncePending(0)
    );
    event.x = Some(1);
    assert!(tracker.observe_antifairy_bounce_pending(&event).is_err());
}

#[test]
fn guard_draw_return_retains_pose_without_requiring_a_pose_store() {
    let mut tracker = SpriteMainExecutionTracker {
        current_slot: Some(10),
        timers_and_oam_dispatch_state: Some(9),
        ..Default::default()
    };
    let mut event = raw("nmi", Some(0x05_c243), None, None);
    event.x = Some(10);
    tracker.observe_guard_animation_checkpoint(&event).unwrap();
    assert_eq!(
        tracker.guard_animation_checkpoint,
        Some((10, zelda3::GuardAnimationCheckpoint::DrawReturned))
    );
    event.x = Some(9);
    assert!(tracker.observe_guard_animation_checkpoint(&event).is_err());
}

#[test]
fn guard_draw_cursors_distinguish_body_and_weapon_store_prefixes() {
    use zelda3::GuardAnimationCheckpoint as Stage;
    for (pc, x, y, expected) in [
        (0x05_c711, 0, 1, Stage::HeadCharacterPending),
        (0x05_c713, 0, 1, Stage::HeadCharacterPending),
        (0x05_ca71, 33, 13, Stage::BodyCoordinates { entry: 1 }),
        (0x05_ca74, 33, 13, Stage::BodyCoordinates { entry: 1 }),
        (0x05_ca77, 35, 6, Stage::BodyFlagsPending { entry: 3 }),
        (0x05_ca8e, 34, 10, Stage::BodyFlagsPending { entry: 2 }),
        (0x05_ca91, 34, 10, Stage::BodyFlagsPending { entry: 2 }),
        (
            0x05_cb8c,
            32,
            24,
            Stage::WeaponBeforeCoordinates { entry: 0 },
        ),
        (0x05_c721, 0, 3, Stage::HeadExtendedPending),
        (0x05_c724, 0, 3, Stage::HeadExtendedPending),
        (0x05_c725, 0, 0, Stage::HeadExtendedPending),
        (0x05_c729, 0, 0, Stage::HeadExtendedPending),
        (0x05_c717, 0, 2, Stage::HeadCharacterPending),
        (0x05_ca9e, 35, 6, Stage::BodyFlagsPending { entry: 3 }),
        (0x05_ca29, 33, 12, Stage::BodyBeforeEntry { entry: 1 }),
        (0x05_ca43, 66, 12, Stage::BodyBeforeEntry { entry: 1 }),
        (0x05_ca4b, 66, 12, Stage::BodyBeforeEntry { entry: 1 }),
        (0x05_ca6b, 66, 13, Stage::BodyCoordinates { entry: 1 }),
        (0x05_ca96, 34, 10, Stage::BodyFlagsPending { entry: 2 }),
        (0x05_ca9f, 35, 7, Stage::BodyFlagsPending { entry: 3 }),
        (
            0x05_cb86,
            32,
            24,
            Stage::WeaponBeforeCoordinates { entry: 0 },
        ),
        (0x05_cbaa, 34, 21, Stage::WeaponCoordinates { entry: 1 }),
    ] {
        let mut tracker = SpriteMainExecutionTracker {
            current_slot: Some(11),
            timers_and_oam_dispatch_state: Some(9),
            guard_animation_pose_slot: Some(11),
            ..Default::default()
        };
        let mut event = raw("nmi", Some(pc), Some(x), None);
        event.y = Some(y);
        tracker.observe_guard_animation_checkpoint(&event).unwrap();
        assert_eq!(tracker.guard_animation_checkpoint, Some((11, expected)));
    }
}

#[test]
fn guard_initializer_parry_nmi_preserves_the_nested_call_ordinal() {
    // Pinned-ROM host 279816: slot 10 dispatches state 8, reaches
    // $069271 twice, and accepts NMI at $06EB94 during call two.
    assert_eq!(GUARD_PARRY_HITBOX_COMPARE_PC, 0x06eb94);
    for (active_call, pc) in [(1, 0x06eb92), (2, 0x06eb92), (1, 0x06eb94), (2, 0x06eb94)] {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        for event in [
            raw("pc", Some(0x068328), None, None),
            raw("pc", Some(0x0684e2), Some(10), None),
        ] {
            source.consume_event(event, &mut receipts).unwrap();
        }
        let mut timers = raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(10), None);
        timers.stack1 = Some(8);
        source.consume_event(timers, &mut receipts).unwrap();
        for _ in 0..active_call {
            source
                .consume_event(raw("pc", Some(0x069271), Some(10), None), &mut receipts)
                .unwrap();
        }
        source
            .consume_event(raw("nmi", Some(pc), Some(10), None), &mut receipts)
            .unwrap();
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);
        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainGuardPrepParryHitbox {
                        slot: 10,
                        active_call
                    }
                ),
                OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    SpriteMainProgress::GuardPrepParryHitbox {
                        slot: 10,
                        active_call
                    }
                ),
            ]
        );
    }
}

#[test]
fn timer_decrement_nmi_exports_the_completed_countdown_prefix() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    for event in [
        raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(0), None),
        // The route frontier returns from retro_run at `$06:84A8`, after
        // countdowns and before the suffix's conditional floor branch.
        raw(
            "nmi",
            Some(SPRITE_TIMER_DECREMENTS_COMPLETE_START_PC + 4),
            Some(0),
            None,
        ),
    ] {
        source.consume_event(event, &mut receipts).unwrap();
    }
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainAfterTimerDecrements(0),
            ),
            OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::AfterTimerDecrements(0),
            ),
        ],
    );
}

#[test]
fn hit_timer_nmi_retains_the_pending_aux4_update() {
    for pc in [0x06_849c, 0x06_849f, 0x06_84a1] {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        for event in [
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(0), None),
            // The aux4 load follows the completed hit-timer statement.
            raw("nmi", Some(pc), Some(0), None),
        ] {
            source.consume_event(event, &mut receipts).unwrap();
        }
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainAfterHitTimer(0),
                ),
                OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    SpriteMainProgress::AfterHitTimer(0),
                ),
            ],
        );
    }
}

#[test]
fn selected_game_entrance_return_requires_its_module05_caller() {
    for (main, caller, expected) in [
        (5, Some(0x00_8059), true),
        (7, Some(0x00_8059), false),
        (5, Some(0x02_85ad), false),
        (5, None, false),
    ] {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut event = raw("pc", Some(0x02_824d), None, None);
        event.main = Some(main);
        event.sub = Some(0);
        event.return_address = caller;
        source.consume_event(event, &mut receipts).unwrap();
        assert_eq!(
            receipts,
            if expected {
                vec![OriginalTimingSemanticReceipt::SelectedGameEntranceReturned]
            } else {
                vec![]
            }
        );
    }
}

#[test]
fn zero_hit_timer_branch_nmi_exports_the_primary_countdown_prefix() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    for event in [
        raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(14), None),
        // The zero branch skipped the linear hit-timer interval and the
        // host ended while fetching the first clear instruction.
        raw(
            "nmi",
            Some(SPRITE_PRIMARY_TIMER_DECREMENTS_ZERO_HIT_STORE_START_PC),
            Some(14),
            None,
        ),
    ] {
        source.consume_event(event, &mut receipts).unwrap();
    }
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainAfterPrimaryTimerDecrements(14),
            ),
            OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::AfterPrimaryTimerDecrements(14),
            ),
        ],
    );
}

#[test]
fn wallmaster_reset_nmi_exports_the_fixed_reset_prefix() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(12), None),
            &mut receipts,
        )
        .unwrap();
    let mut nmi = raw(
        "nmi",
        Some(WALLMASTER_RESET_AFTER_FIXED_PREFIX_PC),
        Some(0xfff),
        None,
    );
    nmi.return_address = Some(WALLMASTER_AFTER_SPRITE_RESET_PC);
    source.consume_event(nmi, &mut receipts).unwrap();
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainAfterWallmasterResetPrefix(12),
            ),
            OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::AfterWallmasterResetPrefix(12),
            ),
        ],
    );
}

#[test]
fn wallmaster_reset_nmi_exports_the_completed_descending_clear() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(12), None),
            &mut receipts,
        )
        .unwrap();
    let mut nmi = raw("nmi", Some(0x09_c47f), Some(0x039e), None);
    nmi.return_address = Some(WALLMASTER_AFTER_SPRITE_RESET_PC);
    source.consume_event(nmi, &mut receipts).unwrap();
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainWallmasterResetClear {
                    slot: 12,
                    cleared_bytes: 3170
                },
            ),
            OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::WallmasterResetClear {
                    slot: 12,
                    cleared_bytes: 3170
                },
            ),
        ],
    );
}

#[test]
fn sprite_main_host_return_exports_throwable_scenery_state_clear() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    for event in [
        raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(7), None),
        raw("pc", Some(SPRITE_SLOT_RETURN_PC), Some(7), None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(6), None),
        raw(
            "wram-write",
            Some(THROWABLE_SCENERY_STATE_CLEAR_PC),
            Some(6),
            Some(SPRITE_STATE_BASE + 6),
        ),
    ] {
        source.consume_event(event, &mut receipts).unwrap();
    }

    let checkpoint =
        serde_json::from_slice(&serde_json::to_vec(&source.checkpoint()).unwrap()).unwrap();
    let mut resumed = empty_semantic_tracker();
    resumed.restore_checkpoint(checkpoint).unwrap();
    resumed.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::AfterThrowableSceneryStateClear(6),
        )],
    );
}

#[test]
fn sprite_main_nmi_exports_throwable_scenery_state_clear() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    for event in [
        raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(6), None),
        raw(
            "wram-write",
            Some(THROWABLE_SCENERY_STATE_CLEAR_PC),
            Some(6),
            Some(SPRITE_STATE_BASE + 6),
        ),
        raw("nmi", Some(0x06_e465), Some(6), None),
    ] {
        source.consume_event(event, &mut receipts).unwrap();
    }

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainAfterThrowableSceneryStateClear(6),
            ),
        ],
    );
}

#[test]
fn cached_sprite_nmi_exports_antfairy_subtype_increment() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    for &address in &CACHED_SPRITE_LIVE_FIELDS {
        source
            .consume_event(
                raw(
                    "wram-write",
                    Some(UNCACHE_SPRITE_START_PC),
                    Some(1),
                    Some(address + 1),
                ),
                &mut receipts,
            )
            .unwrap();
    }
    source
        .consume_event(
            raw(
                "wram-write",
                Some(ANTFAIRY_SUBTYPE2_INCREMENT_PC),
                Some(1),
                Some(SPRITE_SUBTYPE2_BASE + 1),
            ),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("nmi", Some(0x05_dfb5), Some(0x08e8), None),
            &mut receipts,
        )
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(
                CachedSpriteExecutionProgressReceipt {
                    progress: CachedSpriteExecutionProgress::Executing {
                        slot: 1,
                        progress: CachedSpriteExecutionBodyProgress::AfterAntfairySubtype2Increment,
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    );
}

#[test]
fn sprite_main_nmi_exports_antfairy_subtype_increment() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    for event in [
        raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(1), None),
        raw(
            "wram-write",
            Some(ANTFAIRY_SUBTYPE2_INCREMENT_PC),
            Some(1),
            Some(SPRITE_SUBTYPE2_BASE + 1),
        ),
        raw("nmi", Some(0x06_e465), Some(1), None),
    ] {
        source.consume_event(event, &mut receipts).unwrap();
    }
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainAfterAntfairySubtype2Increment(1),
            ),
            OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::AfterAntfairySubtype2Increment(1),
            ),
        ],
    );
}

#[test]
fn sprite_main_nmi_exports_lanmola_subtype_increment() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    for event in [
        raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(2), None),
        raw(
            "wram-write",
            Some(LANMOLA_SUBTYPE2_INCREMENT_PC),
            Some(2),
            Some(SPRITE_SUBTYPE2_BASE + 2),
        ),
        raw("nmi", Some(0x05_a6c0), Some(2), None),
    ] {
        source.consume_event(event, &mut receipts).unwrap();
    }
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainAfterLanmolaSubtype2Increment(2),
            ),
            OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::AfterLanmolaSubtype2Increment(2),
            ),
        ],
    );
}

#[test]
fn agahnim_motion_blur_spawn_tracker_follows_the_shared_helper() {
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(1);
    let mut store = raw(
        "wram-write",
        Some(SPRITE_SPAWN_DYNAMICALLY_FIRST_TYPE_STORE_PC),
        Some(1),
        Some(SPRITE_TYPE_BASE + 3),
    );
    store.y = Some(3);
    store.value = Some(0xc1);
    store.return_address = Some(0x1dd39f);
    tracker
        .observe_agahnim_motion_blur_spawn_write(&store)
        .unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::AgahnimMotionBlurSpawn {
            slot: 1,
            spawned_slot: 3,
            progress: SpriteDynamicSpawnProgress::TypePublished,
        }
    );
    let mut state = raw(
        "wram-write",
        Some(SPRITE_SPAWN_DYNAMICALLY_STATE_STORE_PC),
        Some(1),
        Some(SPRITE_STATE_BASE + 3),
    );
    state.y = Some(3);
    state.value = Some(9);
    tracker
        .observe_agahnim_motion_blur_spawn_write(&state)
        .unwrap();
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::AgahnimMotionBlurSpawn {
            slot: 1,
            spawned_slot: 3,
            progress: SpriteDynamicSpawnProgress::StatePublished,
        }
    );
    // Another caller's spawn of the same type is not the checkpoint.
    let mut other = SpriteMainExecutionTracker::default();
    other.current_slot = Some(1);
    store.return_address = Some(0x1dd2a0);
    other
        .observe_agahnim_motion_blur_spawn_write(&store)
        .unwrap();
    assert_eq!(other.agahnim_motion_blur_spawn, None);
}

#[test]
fn initializer_prep_move_y_boundaries_name_the_completed_assignment() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    for event in [
        raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(6), None),
    ] {
        source.consume_event(event, &mut receipts).unwrap();
    }
    let execution = source.sprite_main_execution.as_mut().unwrap();
    execution.timers_and_oam_dispatch_state = Some(8);
    // f1525509: STA y_high pending inside Sprite_MoveY from SpritePrep_Spike.
    let mut event = raw("nmi", Some(0x06_e968), Some(6), None);
    event.return_address = Some(0xa691e3);
    execution.observe_initialize_prep_move_y(&event).unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::InitializePrepMoveY {
            slot: 6,
            checkpoint: SpriteMoveXYCheckpoint::AfterYLow,
        }
    );
    event.pc = Some(0x06_e955);
    event.return_address = Some(0x91e330);
    execution.observe_initialize_prep_move_y(&event).unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::InitializePrepMoveY {
            slot: 6,
            checkpoint: SpriteMoveXYCheckpoint::AfterYSubpixel,
        }
    );
    event.pc = Some(0x06_91e4);
    event.return_address = Some(0x00_83a6);
    execution.observe_initialize_prep_move_y(&event).unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::InitializePrepMoveY {
            slot: 6,
            checkpoint: SpriteMoveXYCheckpoint::AfterYHigh,
        }
    );
    // Sprite_MoveY from a state-9 handler is not the prep checkpoint.
    execution.timers_and_oam_dispatch_state = Some(9);
    event.pc = Some(0x06_e968);
    event.return_address = Some(0xa691e3);
    execution.observe_initialize_prep_move_y(&event).unwrap();
    assert_eq!(execution.initialize_prep_move_y, None);
}

#[test]
fn initializer_dispatch_accepts_bounce_trampoline_prologues() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    for event in [
        raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(4), None),
    ] {
        source.consume_event(event, &mut receipts).unwrap();
    }
    let execution = source.sprite_main_execution.as_mut().unwrap();
    execution.timers_and_oam_dispatch_state = Some(8);
    // SpritePrep_LaserEye_bounce ($06:8B03 JSL $1E:A4E7), interrupted
    // with the JSR pending after PHB/PHK/PLB (f1520004).
    let mut event = raw("nmi", Some(0x1e_a4ea), Some(4), None);
    event.return_address = Some(0x8b0606);
    event.stack4 = Some(0x06);
    execution.observe_initialize_prep_pending(&event).unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::InitializePrepPending(4)
    );
    // PHB pending right after the JSL.
    execution.initialize_prep_pending = None;
    event.pc = Some(0x1e_a4e7);
    event.return_address = Some(0x06_8b06);
    event.stack4 = None;
    execution.observe_initialize_prep_pending(&event).unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::InitializePrepPending(4)
    );
    // PLB pending with K above B.
    execution.initialize_prep_pending = None;
    event.pc = Some(0x1e_a4e9);
    event.return_address = Some(0x06_061e);
    event.stack4 = Some(0x8b);
    execution.observe_initialize_prep_pending(&event).unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::InitializePrepPending(4)
    );
    // A different caller's return is not the prep checkpoint.
    execution.initialize_prep_pending = None;
    event.pc = Some(0x1e_a4ea);
    event.return_address = Some(0x8b0706);
    event.stack4 = Some(0x06);
    execution.observe_initialize_prep_pending(&event).unwrap();
    assert_eq!(execution.initialize_prep_pending, None);
    // The prep body itself is past the trampoline.
    event.pc = Some(0x1e_a4f1);
    event.return_address = Some(0x8b0606);
    execution.observe_initialize_prep_pending(&event).unwrap();
    assert_eq!(execution.initialize_prep_pending, None);
}

#[test]
fn initializer_dispatch_jump_table_requires_its_source_caller() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    for event in [
        raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(1), None),
    ] {
        source.consume_event(event, &mut receipts).unwrap();
    }
    let execution = source.sprite_main_execution.as_mut().unwrap();
    execution.timers_and_oam_dispatch_state = Some(8);
    let mut event = raw("nmi", Some(0x008781), Some(1), None);
    event.return_address = Some(0x06865a);
    execution.observe_initialize_prep_pending(&event).unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::InitializePrepPending(1)
    );
    execution.initialize_prep_pending = None;
    event.return_address = Some(0x068659);
    execution.observe_initialize_prep_pending(&event).unwrap();
    assert_eq!(execution.initialize_prep_pending, None);
    event.pc = Some(0x06_91b4);
    event.return_address = Some(0x00_83a6);
    execution.observe_initialize_prep_pending(&event).unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::InitializePrepPending(1)
    );
    execution.initialize_prep_pending = None;
    event.pc = Some(0x00_8797);
    event.y = Some(0xb5);
    execution.observe_initialize_prep_pending(&event).unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::InitializePrepPending(1)
    );
    execution.initialize_prep_pending = None;
    event.return_address = Some(0x00_83a5);
    execution.observe_initialize_prep_pending(&event).unwrap();
    assert_eq!(execution.initialize_prep_pending, None);
    // Any type's prep routine entry under Sprite_ExecuteSingle's return is
    // the same checkpoint.
    for entry in [0x06_91ae, 0x06_91ba, 0x06_91c5] {
        execution.initialize_prep_pending = None;
        event.pc = Some(entry);
        event.return_address = Some(0x00_83a6);
        execution.observe_initialize_prep_pending(&event).unwrap();
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::InitializePrepPending(1),
            "{entry:#x}"
        );
    }
    event.pc = Some(0x00_8797);
    event.y = Some(0xb5);
    // Sprite_ExecuteSingle's state-8 dispatch shares the jump-table
    // instruction and stack but targets SpriteModule_Initialize itself.
    execution.initialize_prep_pending = None;
    event.return_address = Some(0x00_83a6);
    event.y = Some(17);
    event.a = Some(0x864d);
    execution.observe_initialize_prep_pending(&event).unwrap();
    assert_eq!(execution.initialize_prep_pending, None);
    event.a = Some(0x8d2c);
    execution.observe_initialize_prep_pending(&event).unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::InitializePrepPending(1)
    );
}

#[test]
fn falling_palette_direction_checkpoint_is_bound_to_its_caller() {
    for (caller, expected) in [(0x02_8ea4, true), (0x02_8ea3, false)] {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut event = raw("nmi", Some(0x00_e9bc), Some(480), None);
        event.main = Some(7);
        event.sub = Some(7);
        event.subsub = Some(15);
        event.return_address = Some(caller);
        event.nmi_latch = Some(1);
        source.consume_event(event, &mut receipts).unwrap();
        assert_eq!(
            receipts.contains(
                &OriginalTimingSemanticReceipt::DungeonFallingFadeInPaletteDirectionToggled
            ),
            expected
        );
    }
}

#[test]
fn guard_patrol_endpoint_retains_the_initializer_active_call() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    for event in [
        raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(9), None),
    ] {
        source.consume_event(event, &mut receipts).unwrap();
    }
    let execution = source.sprite_main_execution.as_mut().unwrap();
    execution.timers_and_oam_dispatch_state = Some(8);
    let event = raw("nmi", Some(0x05c415), Some(9), None);
    for active_call in 1..=2 {
        execution.initialize_active_main_calls = active_call;
        execution.observe_guard_prep_patrol_delay(&event).unwrap();
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::GuardPrepPatrolDelay {
                slot: 9,
                active_call
            }
        );
    }
    execution.initialize_active_main_calls = 0;
    assert!(execution.observe_guard_prep_patrol_delay(&event).is_err());
    execution.timers_and_oam_dispatch_state = Some(9);
    execution.guard_prep_patrol_delay = None;
    execution.observe_guard_prep_patrol_delay(&event).unwrap();
    assert_eq!(execution.guard_prep_patrol_delay, None);
}

#[test]
fn hog_spear_animation_endpoint_requires_the_exact_two_byte_caller() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    for event in [
        raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(13), None),
    ] {
        source.consume_event(event, &mut receipts).unwrap();
    }
    let execution = source.sprite_main_execution.as_mut().unwrap();
    let mut event = raw("frame", Some(0x05c469), Some(13), None);
    event.return_address = Some(0xc8cc3a);
    execution
        .observe_hog_spear_body_graphics_pending(&event)
        .unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::HogSpearBodyGraphicsPending(13)
    );
    execution.hog_spear_body_graphics_pending = None;
    execution.absorbable_body_active = false;
    execution.absorbable_horizontal_lookup = None;
    execution.absorbable_vertical_lookup = None;
    execution.absorbable_vertical_attribute_loaded = None;
    execution.dispatch_trampoline_return = None;
    execution.moblin_collision_started = false;
    execution.moblin_attribute_loaded = None;
    execution.moblin_collision_geometry = None;
    execution.vitreous_minions_seen = false;
    execution.vitreous_player_damage_pending = None;
    execution.vitreous_ai_pending = None;
    execution.mini_moldorm_ai_pending = None;
    execution.vitreous_damage_pending = None;
    execution.swamola_segment = None;
    execution.swamola_head_prepared = false;
    execution.swamola_head_draw_completed = None;
    execution.swamola_head_draw = None;
    execution.swamola_segment_draw = None;
    execution.pengator_slide_pending = None;
    execution.antifairy_bounce_pending = None;
    execution.kholdstare_subtype_decremented = false;
    execution.kholdstare_damage_pending = None;
    execution.initialize_prep_pending = None;
    event.return_address = Some(0xc8cc3b);
    execution
        .observe_hog_spear_body_graphics_pending(&event)
        .unwrap();
    assert_eq!(execution.hog_spear_body_graphics_pending, None);
}

#[test]
fn guard_initializer_tile_collision_requires_its_nested_call_and_wrapper() {
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();
    tracker
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    tracker
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(13), None),
            &mut receipts,
        )
        .unwrap();
    let execution = tracker.sprite_main_execution.as_mut().unwrap();
    execution.timers_and_oam_dispatch_state = Some(8);
    let mut event = raw("frame", Some(0x06_e49d), Some(13), None);
    event.return_address = Some(0x05_b890);
    for active_call in 1..=2 {
        execution.initialize_active_main_calls = active_call;
        execution
            .observe_guard_prep_tile_collision_return(&event)
            .unwrap();
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::GuardPrepTileCollisionReturned {
                slot: 13,
                active_call
            }
        );
    }
    execution.initialize_active_main_calls = 0;
    assert!(execution
        .observe_guard_prep_tile_collision_return(&event)
        .is_err());
    event.return_address = Some(0);
    execution.guard_prep_tile_collision_return = None;
    execution
        .observe_guard_prep_tile_collision_return(&event)
        .unwrap();
    assert_eq!(execution.guard_prep_tile_collision_return, None);
}

#[test]
fn absorbable_lookup_checkpoint_requires_body_and_horizontal_direction() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(3), None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(raw("pc", Some(0x06_d051), Some(3), None), &mut receipts)
        .unwrap();
    let execution = source.sprite_main_execution.as_mut().unwrap();
    let mut event = raw("nmi", Some(0x00_8872), Some(170), None);
    event.return_address = Some(0x06_e8cd);
    event.y = Some(6);
    execution.observe_absorbable_tile_lookup(&event).unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::AbsorbableHorizontalTileLookup(3)
    );
    execution.absorbable_horizontal_lookup = None;
    execution.absorbable_vertical_lookup = None;
    execution.absorbable_vertical_attribute_loaded = None;
    execution.dispatch_trampoline_return = None;
    execution.moblin_collision_started = false;
    execution.moblin_attribute_loaded = None;
    execution.moblin_collision_geometry = None;
    execution.vitreous_minions_seen = false;
    execution.vitreous_player_damage_pending = None;
    execution.vitreous_ai_pending = None;
    execution.mini_moldorm_ai_pending = None;
    execution.vitreous_damage_pending = None;
    execution.swamola_segment = None;
    execution.swamola_head_prepared = false;
    execution.swamola_head_draw_completed = None;
    execution.swamola_head_draw = None;
    execution.swamola_segment_draw = None;
    execution.pengator_slide_pending = None;
    execution.antifairy_bounce_pending = None;
    execution.kholdstare_subtype_decremented = false;
    execution.kholdstare_damage_pending = None;
    event.y = Some(0);
    execution.observe_absorbable_tile_lookup(&event).unwrap();
    assert_eq!(execution.absorbable_horizontal_lookup, None);
    assert_eq!(execution.absorbable_vertical_lookup, Some(3));
    execution.absorbable_vertical_lookup = None;
    execution.absorbable_vertical_attribute_loaded = None;
    execution.dispatch_trampoline_return = None;
    execution.moblin_collision_started = false;
    execution.moblin_attribute_loaded = None;
    execution.moblin_collision_geometry = None;
    execution.vitreous_minions_seen = false;
    execution.vitreous_player_damage_pending = None;
    execution.vitreous_ai_pending = None;
    execution.mini_moldorm_ai_pending = None;
    execution.vitreous_damage_pending = None;
    execution.swamola_segment = None;
    execution.swamola_head_prepared = false;
    execution.swamola_head_draw_completed = None;
    execution.swamola_head_draw = None;
    execution.swamola_segment_draw = None;
    event.pc = Some(0x06_e782);
    event.return_address = Some(0x03_e5f0);
    event.x = Some(3);
    event.y = Some(8);
    execution.observe_absorbable_tile_lookup(&event).unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::AbsorbableVerticalTileLookup(3)
    );
    execution.absorbable_vertical_lookup = None;
    execution.absorbable_vertical_attribute_loaded = None;
    execution.dispatch_trampoline_return = None;
    execution.moblin_collision_started = false;
    execution.moblin_attribute_loaded = None;
    execution.moblin_collision_geometry = None;
    execution.vitreous_minions_seen = false;
    execution.vitreous_player_damage_pending = None;
    execution.vitreous_ai_pending = None;
    execution.mini_moldorm_ai_pending = None;
    execution.vitreous_damage_pending = None;
    execution.swamola_segment = None;
    execution.swamola_head_prepared = false;
    execution.swamola_head_draw_completed = None;
    execution.swamola_head_draw = None;
    execution.swamola_segment_draw = None;
    event.return_address = Some(0x03_e5f1);
    execution.observe_absorbable_tile_lookup(&event).unwrap();
    assert_eq!(execution.absorbable_vertical_lookup, None);
    event.pc = Some(0x06_e812);
    event.return_address = Some(0x03_e5f0);
    execution.observe_absorbable_tile_lookup(&event).unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::AbsorbableVerticalTileAttributeLoaded(3)
    );
    execution.absorbable_vertical_attribute_loaded = None;
    execution.dispatch_trampoline_return = None;
    execution.moblin_collision_started = false;
    execution.moblin_attribute_loaded = None;
    execution.moblin_collision_geometry = None;
    execution.vitreous_minions_seen = false;
    execution.vitreous_player_damage_pending = None;
    execution.vitreous_ai_pending = None;
    execution.mini_moldorm_ai_pending = None;
    execution.vitreous_damage_pending = None;
    execution.swamola_segment = None;
    execution.swamola_head_prepared = false;
    execution.swamola_head_draw_completed = None;
    execution.swamola_head_draw = None;
    execution.swamola_segment_draw = None;
    event.pc = Some(0x06_e883);
    event.return_address = Some(0xba_e7a0);
    event.y = Some(14);
    execution.observe_absorbable_tile_lookup(&event).unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::AbsorbableHorizontalTileLookup(3)
    );
}

#[test]
fn sprite_main_host_return_exports_helmasaur_hard_hat_subtype_increment() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    for event in [
        raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
        raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(4), None),
        raw(
            "wram-write",
            Some(HELMASAUR_HARD_HAT_BEETLE_SUBTYPE2_INCREMENT_PC),
            Some(4),
            Some(SPRITE_SUBTYPE2_BASE + 4),
        ),
    ] {
        source.consume_event(event, &mut receipts).unwrap();
    }
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::AfterHelmasaurHardHatBeetleSubtype2Increment(4),
        )],
    );
}

#[test]
fn sprite_main_host_return_after_active_cucco_y_subpixel_keeps_coordinate_suffix_pending() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(2), None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(ACTIVE_CUCCO_MOVEMENT_CALL_PC), Some(2), None),
            &mut receipts,
        )
        .unwrap();
    for address in [
        SPRITE_X_SUBPIXEL_BASE + 2,
        SPRITE_X_LOW_BASE + 2,
        SPRITE_X_HIGH_BASE + 2,
    ] {
        source
            .consume_event(
                raw("wram-write", Some(0x06_e94e), Some(0x12), Some(address)),
                &mut receipts,
            )
            .unwrap();
    }
    source
        .consume_event(
            raw(
                "wram-write",
                Some(0x06_e94e),
                Some(0x12),
                Some(SPRITE_Y_SUBPIXEL_BASE + 2),
            ),
            &mut receipts,
        )
        .unwrap();
    source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::AfterActiveCuccoYSubpixel {
                slot: 2,
                helper_ordinal: 0,
            },
        )],
    );
}

#[test]
fn sprite_main_nmi_after_three_shared_cucco_increments_keeps_the_helper_pending() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(5), None),
            &mut receipts,
        )
        .unwrap();
    for pc in CUCCO_SUBTYPE_INCREMENT_PUBLICATION_PCS.iter().take(3) {
        source
            .consume_event(
                raw(
                    "wram-write",
                    Some(*pc),
                    Some(5),
                    Some(SPRITE_SUBTYPE2_BASE + 5),
                ),
                &mut receipts,
            )
            .unwrap();
    }
    source
        .consume_event(raw("nmi", Some(0x06_a6eb), Some(5), None), &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainAfterCuccoSubtypeIncrements {
                    slot: 5,
                    helper_ordinal: 0,
                    completed: 3,
                },
            ),
        ],
    );
}

#[test]
fn sprite_main_nmi_before_the_first_slot_is_not_a_cucco_publication() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    source
        .consume_event(
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    source
        .consume_event(raw("nmi", Some(0x06_f80f), None, None), &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainBeforeFirstSlot,
            ),
        ],
    );
}

#[test]
fn preemptive_poly_render_start_is_module_scoped_source_authority() {
    for main in [0x07, 0x0e, 0x19] {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut event = raw("pc", Some(POLYHEDRAL_RENDER_START_PC), None, None);
        event.main = Some(main);
        source.consume_event(event, &mut receipts).unwrap();
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::PreemptivePolyhedralRenderStarted],
        );
    }

    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    let mut title_event = raw("pc", Some(POLYHEDRAL_RENDER_START_PC), None, None);
    title_event.main = Some(0x00);
    source.consume_event(title_event, &mut receipts).unwrap();
    assert!(receipts.is_empty());
}

#[test]
fn dungeon_reset_caller_pc_maps_to_the_last_published_source_statement() {
    for pc in [0x09_c163, 0x09_c166] {
        let mut event = raw("nmi", Some(pc), None, None);
        event.a = Some(0xffff);
        assert_eq!(
            dungeon_reset_sprites_caller_progress(&event),
            Some(DungeonResetSpritesCpuProgress::LoadBeforeOrigin)
        );
        event.a = Some(0x0123);
        assert_eq!(dungeon_reset_sprites_caller_progress(&event), None);
    }
    let cases = [
        (
            DUNGEON_RESET_SPRITES_AFTER_DISABLE_PC,
            DungeonResetSpritesCpuProgress::SpritesDisabled,
        ),
        (
            DUNGEON_RESET_SPRITES_COLLISION_Y_STORE_PC,
            DungeonResetSpritesCpuProgress::CollisionXSizeSet,
        ),
        (
            DUNGEON_RESET_SPRITES_HISTORY_SEARCH_START_PC,
            DungeonResetSpritesCpuProgress::RoomHistorySearchStarted,
        ),
        (
            0x09_c137,
            DungeonResetSpritesCpuProgress::RoomHistorySearchStarted,
        ),
        (
            DUNGEON_RESET_SPRITES_HISTORY_FOUND_PC,
            DungeonResetSpritesCpuProgress::RoomHistorySearchStarted,
        ),
    ];
    for (pc, expected) in cases {
        let event = raw("nmi", Some(pc), None, None);
        assert_eq!(
            dungeon_reset_sprites_caller_progress(&event),
            Some(expected)
        );
    }
    assert_eq!(
        dungeon_reset_sprites_caller_progress(&raw(
            "nmi",
            Some(DUNGEON_RESET_SPRITES_HISTORY_FIRST_MUTATION_PC),
            None,
            None,
        )),
        None,
    );
}

#[test]
fn falling_entrance_control_writes_publish_source_stages_without_cpu_provenance() {
    let cases = [
        (
            FALLING_ENTRANCE_ROOM_PARSER_SUBSUB_CLEAR_PC,
            SUBSUBMODULE_INDEX,
            0,
            DungeonFallingEntranceProgress::RoomParserClearedSubsubmodule,
        ),
        (
            FALLING_ENTRANCE_SUBSUB_ADVANCE_PC,
            SUBSUBMODULE_INDEX,
            3,
            DungeonFallingEntranceProgress::RoomLoadAdvancedSubsubmodule,
        ),
        (
            FALLING_ENTRANCE_SONG_BANK_TAIL_PC,
            SUBMODULE_INDEX,
            7,
            DungeonFallingEntranceProgress::SongBankTailEntered,
        ),
    ];

    for (pc, address, value, expected) in cases {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut event = raw("wram-write", Some(pc), None, Some(address));
        event.main = Some(0x11);
        event.value = Some(value);
        source.consume_event(event, &mut receipts).unwrap();
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::DungeonFallingEntranceProgress(expected,)],
        );
    }

    let mut source = empty_semantic_tracker();
    let mut event = raw(
        "wram-write",
        Some(FALLING_ENTRANCE_ROOM_PARSER_SUBSUB_CLEAR_PC),
        None,
        Some(SUBSUBMODULE_INDEX),
    );
    event.main = Some(6);
    let mut receipts = Vec::new();
    source.consume_event(event, &mut receipts).unwrap();
    assert!(
        receipts.is_empty(),
        "Module_PreDungeon's shared room-parser clear is not a falling-entrance publication",
    );
}

#[test]
fn rescued_maiden_nmi_publishes_exact_source_order_tilemap_clear_prefix() {
    let cases = [
        (RESCUED_MAIDEN_TILEMAP_CLEAR_FIRST_STORE_PC, 0x03b4, 3792),
        (RESCUED_MAIDEN_TILEMAP_CLEAR_SIXTH_STORE_PC, 0x03b4, 3797),
        (RESCUED_MAIDEN_TILEMAP_CLEAR_FIRST_INX_PC, 0x03b4, 3800),
        (RESCUED_MAIDEN_TILEMAP_CLEAR_SECOND_INX_PC, 0x03b5, 3800),
        (RESCUED_MAIDEN_TILEMAP_CLEAR_COMPARE_PC, 0x03b6, 3800),
        (RESCUED_MAIDEN_TILEMAP_CLEAR_BRANCH_PC, 0x0800, 8192),
    ];

    for (pc, x, completed_stores) in cases {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut event = raw("nmi", Some(pc), Some(x), None);
        event.main = Some(7);
        event.sub = Some(0x18);
        event.subsub = Some(0);
        event.nmi_latch = Some(1);
        source.consume_event(event, &mut receipts).unwrap();
        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::RescuedMaidenTilemapClearProgress(
                    RescuedMaidenTilemapClearProgressReceipt {
                        completed_stores,
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
            ],
        );
    }

    let mut source = empty_semantic_tracker();
    let mut event = raw(
        "nmi",
        Some(RESCUED_MAIDEN_TILEMAP_CLEAR_SIXTH_STORE_PC),
        Some(0x03b4),
        None,
    );
    event.main = Some(7);
    event.sub = Some(0x17);
    event.subsub = Some(0);
    assert!(source.consume_event(event, &mut Vec::new()).is_err());
}

#[test]
fn rescued_maiden_host_return_preserves_the_incomplete_store_loop() {
    let mut event = raw(
        "frame",
        Some(RESCUED_MAIDEN_TILEMAP_CLEAR_SIXTH_STORE_PC),
        Some(0x03b4),
        None,
    );
    event.main = Some(7);
    event.sub = Some(0x18);
    event.subsub = Some(0);
    assert_eq!(
        rescued_maiden_tilemap_clear_progress(&event, OriginalTimingBoundary::HostReturn).unwrap(),
        Some(RescuedMaidenTilemapClearProgressReceipt {
            completed_stores: 3797,
            boundary: OriginalTimingBoundary::HostReturn,
        }),
    );
}

#[test]
fn rescued_maiden_follower_graphics_tracks_exact_sheet_cursors() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();

    let mut load = raw(
        "pc",
        Some(RESCUED_MAIDEN_LOAD_FOLLOWER_GRAPHICS_ENTRY_PC),
        None,
        None,
    );
    load.main = Some(7);
    load.sub = Some(0x18);
    load.subsub = Some(10);
    source.consume_event(load, &mut receipts).unwrap();

    let mut first_entry = raw(
        "pc",
        Some(RESCUED_MAIDEN_FIRST_FOLLOWER_SHEET_ENTRY_PC),
        None,
        None,
    );
    first_entry.y = Some(0x66);
    source.consume_event(first_entry, &mut receipts).unwrap();

    let mut first_boundary = raw("nmi", Some(0x00_e843), None, None);
    first_boundary.main = Some(7);
    first_boundary.sub = Some(0x18);
    first_boundary.subsub = Some(10);
    first_boundary.nmi_latch = Some(1);
    first_boundary.y = Some(1027);
    source.consume_event(first_boundary, &mut receipts).unwrap();
    assert_eq!(
        source
            .rescued_maiden_initialization
            .unwrap()
            .host_return_receipt()
            .unwrap(),
        RescuedMaidenInitializationProgressReceipt {
            stage: RescuedMaidenInitializationStage::FirstFollowerSheet {
                completed_bytes: 1027,
            },
            boundary: OriginalTimingBoundary::HostReturn,
        },
    );

    let mut tracker = RescuedMaidenInitializationTracker::first_sheet();
    tracker.begin_second_sheet().unwrap();
    let mut second_boundary = raw("frame", Some(0x00_e851), None, None);
    second_boundary.y = Some(189);
    tracker.observe_boundary(&second_boundary).unwrap();
    assert_eq!(
        tracker.host_return_receipt().unwrap().stage,
        RescuedMaidenInitializationStage::SecondFollowerSheet {
            completed_bytes: 189,
        },
    );

    let mut after_store = raw("frame", Some(0x00_e7f4), None, None);
    after_store.y = Some(1343);
    tracker.observe_boundary(&after_store).unwrap();
    assert_eq!(
        tracker.host_return_receipt().unwrap().stage,
        RescuedMaidenInitializationStage::SecondFollowerSheet {
            completed_bytes: 1344,
        },
    );
}

#[test]
fn lanmola_draw_prologue_checkpoints_count_the_trail_stores() {
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(2);
    for (pc, x, ret, stack4, completed) in [
        (0x05_a66d, 2, 0x00_a3a8, 0x00, 0),
        (0x05_a670, 2, 0x00_a3a8, 0x00, 1),
        (0x05_a676, 2, 0xa3_a802, 0x00, 1),
        (0x05_a692, 40, 0x70_6802, 0x02, 1),
        (0x05_a697, 40, 0x68_02a8, 0xa8, 2),
        (0x05_a69c, 40, 0x68_02a8, 0xa3, 3),
        (0x05_a6a1, 40, 0xa3_a802, 0xc8, 4),
        (0x05_a6a5, 40, 0xa3_a802, 0xc8, 5),
    ] {
        let mut event = raw("nmi", Some(pc), Some(x), None);
        event.return_address = Some(ret);
        event.stack4 = Some(stack4);
        tracker.observe_lanmola_draw_prefix(&event).unwrap();
        assert_eq!(
            tracker.progress(),
            SpriteMainProgress::LanmolaDrawPrefix {
                slot: 2,
                completed_stores: completed,
            },
            "{pc:#x}"
        );
    }
    let mut event = raw("nmi", Some(0x05_a6a1), Some(40), None);
    event.return_address = Some(0xa3_a803);
    assert!(tracker.observe_lanmola_draw_prefix(&event).is_err());
}

#[test]
fn bank_1d_dispatcher_return_completes_the_active_slot() {
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(4);
    let mut event = raw("nmi", Some(0x1d_c221), Some(4), None);
    event.return_address = Some(0x06_bff7);
    event.stack4 = Some(0xa6);
    tracker.observe_sprite_handler_returned(&event).unwrap();
    assert_eq!(tracker.progress(), SpriteMainProgress::AfterSlot(4));
    let mut event = raw("frame", Some(0x06_bff7), Some(4), None);
    event.return_address = Some(0x00_84a6);
    tracker.observe_sprite_handler_returned(&event).unwrap();
    assert_eq!(tracker.progress(), SpriteMainProgress::AfterSlot(4));
    event.return_address = Some(0x00_84a7);
    tracker.observe_sprite_handler_returned(&event).unwrap();
    assert_eq!(tracker.handler_returned_slot, None);
}

#[test]
fn laser_eye_draw_prologue_tracker_reports_the_pending_draw() {
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(8);
    tracker.laser_eye_draw_prologue = Some(8);
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::LaserEyeDrawPrologue(8)
    );
    tracker.observe_laser_eye_draw_prologue(&raw("nmi", Some(0x05_dfa9), Some(0x0884), None));
    assert_eq!(tracker.laser_eye_draw_prologue, Some(8));
    tracker.observe_laser_eye_draw_prologue(&raw("nmi", Some(0x1e_a5af), Some(8), None));
    assert_eq!(tracker.laser_eye_draw_prologue, None);
}

#[test]
fn zora_fireball_tracker_maps_source_stores_to_move_xy_checkpoints() {
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(13);
    tracker.zora_fireball = Some((13, 0, false));
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::ZoraFireballMovement {
            slot: 13,
            checkpoint: SpriteMoveXYCheckpoint::BeforeMovement,
        }
    );
    tracker.zora_fireball = Some((13, 1, false));
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::ZoraFireballMovement {
            slot: 13,
            checkpoint: SpriteMoveXYCheckpoint::AfterXSubpixel,
        }
    );
    tracker.zora_fireball = Some((13, 3, true));
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::ZoraFireballMovement {
            slot: 13,
            checkpoint: SpriteMoveXYCheckpoint::AfterYHigh,
        }
    );
}

#[test]
fn boulder_movement_tracker_counts_source_stores_after_the_z_move() {
    let mut tracker = SpriteMainExecutionTracker::default();
    tracker.current_slot = Some(12);
    tracker.boulder_movement = Some((12, 0, true));
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::BoulderMovement {
            slot: 12,
            checkpoint: SpriteMoveXYCheckpoint::BeforeMovement,
        }
    );
    tracker.boulder_movement = Some((12, 3, true));
    assert_eq!(
        tracker.progress(),
        SpriteMainProgress::BoulderMovement {
            slot: 12,
            checkpoint: SpriteMoveXYCheckpoint::AfterXHigh,
        }
    );
}

#[test]
fn push_block_handled_checkpoint_sits_in_the_lamp_cone_guard() {
    let mut event = raw("nmi", Some(0x00_f56a), Some(10), None);
    event.main = Some(7);
    event.return_address = Some(0x02_87e4);
    assert!(dungeon_push_blocks_handled(&event));
    event.pc = Some(0x00_f572);
    assert!(!dungeon_push_blocks_handled(&event));
    event.pc = Some(0x00_f56a);
    event.return_address = Some(0x02_87e5);
    assert!(!dungeon_push_blocks_handled(&event));
}

#[test]
fn push_block_loop_checkpoint_names_the_next_misc_object() {
    let mut event = raw("nmi", Some(0x01_d818), Some(10), None);
    event.main = Some(7);
    event.y = Some(6);
    event.return_address = Some(0x02_87b5);
    assert_eq!(dungeon_push_blocks_in_progress(&event), Some(8));
    event.pc = Some(0x01_d7c8);
    assert_eq!(dungeon_push_blocks_in_progress(&event), Some(6));
    event.pc = Some(0x01_d823);
    event.y = Some(8);
    assert_eq!(dungeon_push_blocks_in_progress(&event), Some(8));
    event.pc = Some(0x01_d7d2);
    assert_eq!(dungeon_push_blocks_in_progress(&event), None);
    event.pc = Some(0x01_d818);
    event.return_address = Some(0x02_87b6);
    assert_eq!(dungeon_push_blocks_in_progress(&event), None);
}

#[test]
fn push_block_checkpoint_requires_the_module07_caller() {
    let mut event = raw("frame", Some(0x07_f0b2), None, None);
    event.main = Some(7);
    event.return_address = Some(0x88_3d00);
    assert!(dungeon_push_blocks_pending(&event));
    event.return_address = Some(0x88_3e00);
    assert!(!dungeon_push_blocks_pending(&event));
    event.return_address = Some(0x88_3d00);
    event.main = Some(9);
    assert!(!dungeon_push_blocks_pending(&event));
}

#[test]
fn vitreous_damage_checkpoint_requires_its_minion_caller() {
    let mut source = empty_semantic_tracker();
    source.sprite_main_execution = Some(SpriteMainExecutionTracker::default());
    source.sprite_main_execution.as_mut().unwrap().current_slot = Some(0);
    let mut returned = raw("frame", Some(0x06_f2ab), Some(0), None);
    returned.return_address = Some(0xc2_141d);
    source
        .sprite_main_execution
        .as_mut()
        .unwrap()
        .observe_vitreous_damage_pending(&returned);
    assert_eq!(
        source
            .sprite_main_execution
            .as_ref()
            .unwrap()
            .vitreous_damage_pending,
        None
    );
    let mut write = raw("wram-write", Some(0x1d_e5dd), Some(0), None);
    write.address = Some(0x0e80);
    write.value = Some(72);
    source.consume_event(write, &mut Vec::new()).unwrap();
    let execution = source.sprite_main_execution.as_mut().unwrap();
    execution.observe_vitreous_damage_pending(&returned);
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::VitreousDamagePending(0)
    );
    execution.vitreous_player_damage_pending = None;
    execution.vitreous_ai_pending = None;
    execution.mini_moldorm_ai_pending = None;
    execution.vitreous_damage_pending = None;
    returned.return_address = Some(0xc2_151d);
    execution.observe_vitreous_damage_pending(&returned);
    assert_eq!(execution.vitreous_damage_pending, None);
    returned.pc = Some(0x06_f82d);
    returned.return_address = Some(0xf2_cd01);
    execution.observe_vitreous_damage_pending(&returned);
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::VitreousDamagePending(0)
    );
    execution.vitreous_damage_pending = None;
    returned.pc = Some(0x06_f5e3);
    returned.return_address = Some(0xaf_f2ca);
    execution.observe_vitreous_damage_pending(&returned);
    assert_eq!(execution.vitreous_damage_pending, Some(0));
    execution.vitreous_damage_pending = None;
    returned.pc = Some(0x06_f600);
    returned.x = Some(9);
    returned.return_address = Some(0xf2_ca00);
    execution.observe_vitreous_damage_pending(&returned);
    assert_eq!(execution.vitreous_damage_pending, Some(0));
    execution.vitreous_damage_pending = None;
    returned.pc = Some(0x06_f645);
    execution.observe_vitreous_damage_pending(&returned);
    assert_eq!(execution.vitreous_damage_pending, None);
    returned.x = Some(0);
    returned.pc = Some(0x00_8788);
    returned.y = Some(0xe4);
    returned.return_address = Some(0x1f_1de4);
    execution.observe_vitreous_damage_pending(&returned);
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::VitreousAiPending(0)
    );
    execution.vitreous_ai_pending = None;
    execution.mini_moldorm_ai_pending = None;
    returned.pc = Some(0x06_f145);
    returned.return_address = Some(0x1d_f126);
    execution.observe_vitreous_damage_pending(&returned);
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::VitreousPlayerDamagePending(0)
    );
}

#[test]
fn dispatcher_rts_supersedes_body_progress_only_for_the_slot_loop_caller() {
    for slot in [0, 1, 15] {
        let mut execution = SpriteMainExecutionTracker {
            current_slot: Some(slot),
            timers_and_oam_slot: Some(slot),
            timers_and_oam_dispatch_state: Some(9),
            ..Default::default()
        };
        let mut event = raw("nmi", Some(0x06_bff8), Some(u16::from(slot)), None);
        event.return_address = Some(0x0083a5);
        execution
            .observe_dispatch_trampoline_return(&event)
            .unwrap();
        assert_eq!(execution.dispatch_trampoline_return, None);
        event.return_address = Some(0x0083a6);
        execution
            .observe_dispatch_trampoline_return(&event)
            .unwrap();
        assert_eq!(execution.progress(), SpriteMainProgress::AfterSlot(slot));
    }
}

#[test]
fn swamola_segment_checkpoint_requires_the_source_loop_and_draw_caller() {
    let mut source = empty_semantic_tracker();
    source.sprite_main_execution = Some(SpriteMainExecutionTracker::default());
    source.sprite_main_execution.as_mut().unwrap().current_slot = Some(4);
    let mut write = raw("wram-write", Some(0x1d_a034), Some(4), None);
    write.address = Some(0x0fb6);
    write.value = Some(1);
    source.consume_event(write, &mut Vec::new()).unwrap();
    let execution = source.sprite_main_execution.as_mut().unwrap();
    let mut returned = raw("frame", Some(0x06_e442), Some(4), None);
    returned.return_address = Some(0xf5_dc12);
    execution.observe_swamola_segment_draw(&returned).unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::SwamolaSegmentDraw {
            slot: 4,
            segment: 1
        }
    );
    execution.swamola_head_prepared = false;
    execution.swamola_head_draw_completed = None;
    execution.swamola_head_draw = None;
    execution.swamola_segment_draw = None;
    returned.return_address = Some(0xf5_dc13);
    execution.observe_swamola_segment_draw(&returned).unwrap();
    assert_eq!(execution.swamola_segment_draw, None);
    returned.pc = Some(0x06_dbf0);
    returned.return_address = Some(0x1d_9f8b);
    execution.observe_swamola_segment_draw(&returned).unwrap();
    assert_eq!(execution.progress(), SpriteMainProgress::SwamolaHeadDraw(4));
    execution.swamola_head_prepared = false;
    execution.swamola_head_draw_completed = None;
    execution.swamola_head_draw = None;
    returned.return_address = Some(0x1d_9f8c);
    execution.observe_swamola_segment_draw(&returned).unwrap();
    assert_eq!(execution.swamola_head_draw, None);
    returned.pc = Some(0x06_e492);
    returned.return_address = Some(0xdb_f5dc);
    execution.observe_swamola_segment_draw(&returned).unwrap();
    assert_eq!(execution.swamola_head_draw_completed, None);
    execution.swamola_head_prepared = true;
    execution.observe_swamola_segment_draw(&returned).unwrap();
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::SwamolaHeadDrawCompleted(4)
    );
}

#[test]
fn moblin_geometry_requires_its_collision_call_and_vertical_probe() {
    let mut execution = SpriteMainExecutionTracker::default();
    execution.current_slot = Some(11);
    let mut event = raw("nmi", Some(0x06_e790), Some(11), None);
    event.y = Some(2);
    event.return_address = Some(0x03_e5f0);
    execution.observe_moblin_collision_geometry(&event);
    assert_eq!(execution.moblin_collision_geometry, None);
    execution.moblin_collision_started = true;
    execution.observe_moblin_collision_geometry(&event);
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::MoblinCollisionGeometry(11)
    );
    execution.moblin_attribute_loaded = None;
    execution.moblin_collision_geometry = None;
    event.return_address = Some(0x03_e5f1);
    execution.observe_moblin_collision_geometry(&event);
    assert_eq!(execution.moblin_collision_geometry, None);
    event.pc = Some(0x06_e8d0);
    event.x = Some(139);
    event.return_address = Some(0xa0_0b02);
    execution.observe_moblin_collision_geometry(&event);
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::MoblinCollisionGeometry(11)
    );
    event.pc = Some(0x06_e818);
    event.x = Some(72);
    event.y = Some(72);
    event.return_address = Some(0xe5_f00b);
    execution.observe_moblin_collision_geometry(&event);
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::MoblinAttributeLoaded(11)
    );
    execution.moblin_attribute_loaded = None;
    event.pc = Some(0x00_8850);
    execution.moblin_collision_geometry = None;
    event.x = Some(1454);
    event.y = Some(2);
    event.return_address = Some(0x06_e8cd);
    execution.observe_moblin_collision_geometry(&event);
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::MoblinCollisionGeometry(11)
    );
    event.pc = Some(0x06_e72f);
    execution.moblin_collision_geometry = None;
    event.x = Some(11);
    event.y = Some(1);
    event.return_address = Some(0x03_e5f0);
    execution.observe_moblin_collision_geometry(&event);
    assert_eq!(
        execution.progress(),
        SpriteMainProgress::MoblinCollisionGeometry(11)
    );
}

#[test]
fn super_bomb_purchase_owns_its_follower_graphics_sheet() {
    let mut source = empty_semantic_tracker();
    let mut execution = SpriteMainExecutionTracker::default();
    execution.current_slot = Some(14);
    source.sprite_main_execution = Some(execution);
    let mut event = raw(
        "pc",
        Some(RESCUED_MAIDEN_LOAD_FOLLOWER_GRAPHICS_ENTRY_PC),
        Some(14),
        None,
    );
    event.return_address = Some(0x1e_e201);
    source.consume_event(event, &mut Vec::new()).unwrap();
    assert_eq!(
        source
            .sprite_main_execution
            .as_ref()
            .unwrap()
            .follower_graphics
            .unwrap()
            .0,
        SpriteFollowerGraphicsCaller::SuperBomb
    );
    let mut sheet = raw(
        "pc",
        Some(RESCUED_MAIDEN_FIRST_FOLLOWER_SHEET_ENTRY_PC),
        Some(14),
        None,
    );
    sheet.y = Some(0x58);
    source.consume_event(sheet, &mut Vec::new()).unwrap();
}

#[test]
fn purple_chest_follower_graphics_uses_its_exact_body_caller() {
    let mut source = empty_semantic_tracker();
    let mut execution = SpriteMainExecutionTracker::default();
    execution.current_slot = Some(8);
    source.sprite_main_execution = Some(execution);
    let mut event = raw(
        "pc",
        Some(RESCUED_MAIDEN_LOAD_FOLLOWER_GRAPHICS_ENTRY_PC),
        Some(8),
        None,
    );
    event.return_address = Some(SPRITE_PURPLE_CHEST_FOLLOWER_GRAPHICS_RETURN_PC);
    source.consume_event(event, &mut Vec::new()).unwrap();
    assert_eq!(
        source
            .sprite_main_execution
            .as_ref()
            .unwrap()
            .follower_graphics
            .unwrap()
            .0,
        SpriteFollowerGraphicsCaller::PurpleChest
    );
    let mut sheet = raw(
        "pc",
        Some(RESCUED_MAIDEN_FIRST_FOLLOWER_SHEET_ENTRY_PC),
        Some(8),
        None,
    );
    sheet.y = Some(0x58);
    source
        .consume_event(sheet.clone(), &mut Vec::new())
        .unwrap();
    sheet.y = Some(0x66);
    assert!(source.consume_event(sheet, &mut Vec::new()).is_err());
}

#[test]
fn blind_maiden_body_follower_graphics_uses_its_exact_caller() {
    let mut source = empty_semantic_tracker();
    let mut execution = SpriteMainExecutionTracker::default();
    execution.current_slot = Some(0);
    source.sprite_main_execution = Some(execution);

    let mut load = raw(
        "pc",
        Some(RESCUED_MAIDEN_LOAD_FOLLOWER_GRAPHICS_ENTRY_PC),
        Some(0),
        None,
    );
    load.return_address = Some(SPRITE_BLIND_MAIDEN_BODY_FOLLOWER_GRAPHICS_RETURN_PC);
    source.consume_event(load, &mut Vec::new()).unwrap();

    assert_eq!(
        source
            .sprite_main_execution
            .unwrap()
            .follower_graphics
            .unwrap()
            .0,
        SpriteFollowerGraphicsCaller::BlindMaidenBody,
    );
}

#[test]
fn old_man_follower_graphics_uses_its_exact_prep_return() {
    let mut source = empty_semantic_tracker();
    let mut execution = SpriteMainExecutionTracker::default();
    execution.current_slot = Some(8);
    source.sprite_main_execution = Some(execution);

    let mut load = raw(
        "pc",
        Some(RESCUED_MAIDEN_LOAD_FOLLOWER_GRAPHICS_ENTRY_PC),
        Some(8),
        None,
    );
    load.return_address = Some(SPRITE_PREP_OLD_MAN_FOLLOWER_GRAPHICS_RETURN_PC);
    source.consume_event(load, &mut Vec::new()).unwrap();

    let mut boundary = raw("nmi", Some(0x00_e845), None, None);
    boundary.main = Some(7);
    boundary.sub = Some(15);
    boundary.subsub = Some(1);
    boundary.nmi_latch = Some(1);
    boundary.y = Some(1102);
    let mut receipts = Vec::new();
    source.consume_event(boundary, &mut receipts).unwrap();

    assert!(
        receipts.contains(&OriginalTimingSemanticReceipt::MainLoopInterrupted(
            MainLoopInterruption::SpriteMainFollowerGraphics {
                slot: 8,
                caller: SpriteFollowerGraphicsCaller::OldMan,
                stage: RescuedMaidenInitializationStage::FirstFollowerSheet {
                    completed_bytes: 1102,
                },
            },
        ))
    );
}

#[test]
fn zelda_follower_graphics_uses_the_pinned_sprite_prep_return() {
    let mut source = empty_semantic_tracker();
    let mut execution = SpriteMainExecutionTracker::default();
    execution.current_slot = Some(1);
    source.sprite_main_execution = Some(execution);

    let mut load = raw(
        "pc",
        Some(RESCUED_MAIDEN_LOAD_FOLLOWER_GRAPHICS_ENTRY_PC),
        Some(1),
        None,
    );
    load.return_address = Some(0x05_ebf5);
    source.consume_event(load, &mut Vec::new()).unwrap();

    assert_eq!(
        source
            .sprite_main_execution
            .unwrap()
            .follower_graphics
            .unwrap()
            .0,
        SpriteFollowerGraphicsCaller::Zelda,
    );
}

fn raw(event: &str, pc: Option<u32>, x: Option<u16>, address: Option<u16>) -> RawTraceEvent {
    let pc = pc.or_else(|| matches!(event, "nmi" | "nmi-resume").then_some(0x008000));
    RawTraceEvent {
        event: event.to_string(),
        stage: None,
        run: None,
        pc,
        s: matches!(event, "nmi" | "nmi-resume").then_some(0x01ff),
        return_address: None,
        stack1: None,
        stack4: None,
        a: None,
        main: None,
        sub: None,
        subsub: None,
        room: None,
        frame_counter: None,
        nmi_latch: matches!(event, "nmi").then_some(0),
        link_y: None,
        bg2_v: None,
        bg2_h: None,
        spotlight_radius: None,
        spotlight_var4_low: None,
        palette_countdown: None,
        spotlight_lower_cursor: None,
        joypad_high: None,
        joypad_low: None,
        joypad_high_filtered: None,
        joypad_low_filtered: None,
        x,
        y: None,
        address,
        value: address.map(|_| 0),
        nmi_ppu_register_operands: matches!(event, "nmi").then_some([0; 31]),
    }
}

#[test]
fn triforce_case2_palette_progress_uses_source_word_and_return_boundaries() {
    let mut partial = raw(
        "frame",
        Some(PALETTE_LOAD_MULTIPLE_BEFORE_WORD_COPY_PC),
        Some(0x00bc),
        None,
    );
    partial.main = Some(0x19);
    partial.sub = Some(0);
    partial.subsub = Some(2);
    partial.room = Some(0x0109);
    assert_eq!(
        triforce_room_case2_palette_progress(&partial, OriginalTimingBoundary::HostReturn,)
            .unwrap(),
        Some(TriforceRoomCase2PaletteProgressReceipt {
            completed_ow_bg2_words: 5,
            boundary: OriginalTimingBoundary::HostReturn,
        }),
    );

    let mut completed = raw(
        "nmi",
        Some(OVERWORLD_PARSE_MAP32_DEFINITION_SECOND_WORD_PC),
        None,
        None,
    );
    completed.main = Some(0x19);
    completed.sub = Some(0);
    completed.subsub = Some(2);
    completed.room = Some(0x0189);
    assert_eq!(
        triforce_room_case2_palette_progress(&completed, OriginalTimingBoundary::NmiAccepted,)
            .unwrap(),
        Some(TriforceRoomCase2PaletteProgressReceipt {
            completed_ow_bg2_words: 21,
            boundary: OriginalTimingBoundary::NmiAccepted,
        }),
    );
}

#[test]
fn credits_scene_progress_preserves_the_scene_advance_and_text_prefix() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    let mut scene_return = raw(
        "wram-write",
        Some(CREDITS_SCENE_OVERWORLD_SUBSUBMODULE_INCREMENT_PC),
        None,
        Some(SUBSUBMODULE_INDEX),
    );
    scene_return.main = Some(0x1a);
    scene_return.value = Some(1);
    source.consume_event(scene_return, &mut receipts).unwrap();
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::CreditsSceneLoadProgress(
            CreditsSceneLoadProgressReceipt {
                progress: CreditsSceneLoadProgress::SceneLoadCompleted,
                boundary: OriginalTimingBoundary::HostReturn,
            },
        )],
    );

    let mut text = raw(
        "frame",
        Some(CREDITS_ENDING_TEXT_BEFORE_TILE_COPY_PC),
        Some(50),
        None,
    );
    text.main = Some(0x1a);
    assert_eq!(
        credits_scene_load_boundary_progress(&text, OriginalTimingBoundary::HostReturn).unwrap(),
        Some(CreditsSceneLoadProgressReceipt {
            progress: CreditsSceneLoadProgress::EndingTextPayloadBytes(50),
            boundary: OriginalTimingBoundary::HostReturn,
        }),
    );
}

#[test]
fn credits_finale_save_progress_decodes_the_completed_checksum_words() {
    let mut event = raw(
        "frame",
        Some(CREDITS_END_SEQUENCE_32_SAVE_CHECKSUM_LOOP_PC),
        Some(0x016e),
        None,
    );
    event.main = Some(0x1a);
    event.sub = Some(0x21);
    event.subsub = Some(0);
    assert_eq!(
        credits_end_sequence_32_boundary_progress(&event, OriginalTimingBoundary::HostReturn,)
            .unwrap(),
        Some(CreditsEndSequence32ProgressReceipt {
            completed_checksum_words: 183,
            boundary: OriginalTimingBoundary::HostReturn,
        }),
    );
}

#[test]
fn peg_attribute_flip_pc_decodes_to_source_bank_progress() {
    let mut event = raw(
        "frame",
        Some(DUNGEON_PEG_FLIP_BANK_C_PC),
        Some(0x0594),
        None,
    );
    event.main = Some(7);
    event.sub = Some(0x16);
    event.subsub = Some(0x10);

    assert_eq!(
        dungeon_peg_attribute_flip_progress(&event, OriginalTimingBoundary::HostReturn,).unwrap(),
        Some(DungeonPegAttributeFlipProgressReceipt {
            index: 0x0594,
            completed_banks: 2,
            boundary: OriginalTimingBoundary::HostReturn,
        }),
    );

    event.sub = Some(2);
    event.subsub = Some(8);
    assert_eq!(
        dungeon_peg_attribute_flip_progress(&event, OriginalTimingBoundary::HostReturn,).unwrap(),
        Some(DungeonPegAttributeFlipProgressReceipt {
            index: 0x0594,
            completed_banks: 2,
            boundary: OriginalTimingBoundary::HostReturn,
        }),
        "the shared source helper must expose the same cursor to its selectable caller",
    );
}

fn raw_at(event: &str, pc: u32, s: u16) -> RawTraceEvent {
    let mut event = raw(event, Some(pc), None, None);
    event.s = Some(s);
    if event.event == "pc" && NMI_HANDLER_COMPLETE_PCS.contains(&(pc & 0x00ff_ffff)) {
        event.joypad_high = Some(0);
        event.joypad_low = Some(0);
        event.joypad_high_filtered = Some(0);
        event.joypad_low_filtered = Some(0);
    }
    event
}

fn empty_semantic_tracker() -> Snes9xOracleSemanticTrace {
    Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    }
}

fn publish_nmi(
    tracker: &mut Snes9xOracleSemanticTrace,
    receipts: &mut Vec<OriginalTimingSemanticReceipt>,
) {
    let mut completion = raw_at("pc", NMI_HANDLER_COMPLETE_PCS[0], 0x01f3);
    completion.joypad_high = Some(0);
    completion.joypad_low = Some(0);
    completion.joypad_high_filtered = Some(0);
    completion.joypad_low_filtered = Some(0);
    tracker.consume_event(completion, receipts).unwrap();
}

fn zero_joypad_publication() -> OriginalTimingSemanticReceipt {
    OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
        high: 0,
        low: 0,
        high_filtered: 0,
        low_filtered: 0,
    })
}

fn frame(stage: &str, run: u64, main: u8, frame_counter: u8) -> RawTraceEvent {
    RawTraceEvent {
        event: "frame".to_string(),
        stage: Some(stage.to_string()),
        run: Some(run),
        pc: Some(0),
        s: Some(0x01ff),
        return_address: None,
        stack1: None,
        stack4: None,
        a: None,
        main: Some(main),
        sub: Some(0),
        subsub: Some(0),
        room: None,
        frame_counter: Some(frame_counter),
        nmi_latch: Some(0),
        link_y: None,
        bg2_v: None,
        bg2_h: None,
        spotlight_radius: None,
        spotlight_var4_low: None,
        palette_countdown: None,
        spotlight_lower_cursor: None,
        joypad_high: None,
        joypad_low: None,
        joypad_high_filtered: None,
        joypad_low_filtered: None,
        x: None,
        y: None,
        address: None,
        value: None,
        nmi_ppu_register_operands: None,
    }
}

fn frame_with_sub(stage: &str, run: u64, main: u8, sub: u8) -> RawTraceEvent {
    let mut event = frame(stage, run, main, 0);
    event.sub = Some(sub);
    event
}

fn main_loop_start() -> RawTraceEvent {
    raw(
        "wram-write",
        Some(ZELDA_RUN_GAME_LOOP_FRAME_COUNTER_WRITE_PC),
        None,
        Some(FRAME_COUNTER),
    )
}

fn main_loop_common_suffix_completion() -> RawTraceEvent {
    let mut event = raw(
        "wram-write",
        Some(ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC),
        None,
        Some(NMI_UPDATE_LATCH),
    );
    event.value = Some(0);
    event
}

/// Encode canonical JSON events as the pinned core's Z3TRACE1 binary
/// records, exactly as the adapter reads them in production.
fn write_semantic_trace(path: &Path, events: &[serde_json::Value]) {
    let mut bytes = parity::trace_format::MAGIC.to_vec();
    for event in events {
        bytes.extend(
            parity::trace_format::TraceRecord::from_json(event)
                .unwrap()
                .encode_framed(),
        );
    }
    fs::write(path, bytes).unwrap();
}

#[test]
fn cold_boot_without_a_zelda_run_game_loop_start_emits_no_progress() {
    let path = env::temp_dir().join(format!(
        "zelda3-snes9x-cold-bootstrap-no-main-progress-{}.jsonl",
        std::process::id()
    ));
    write_semantic_trace(
        &path,
        &[
            serde_json::json!({
                "event": "frame", "stage": "entry", "run": 0,
                "pc": 0x008000, "s": 0x01ff, "main": 0x55,
                "sub": 0x55, "subsub": 0x55, "frame_counter": 0x55,
                "nmi_latch": 0x55
            }),
            serde_json::json!({
                "event": "frame", "stage": "return", "run": 0,
                "pc": 0x0088b3, "s": 0x01fa, "main": 0x55,
                "sub": 0x55, "subsub": 0x55, "frame_counter": 0x55,
                "nmi_latch": 0x55
            }),
        ],
    );
    let mut tracker = empty_semantic_tracker();
    tracker.path = path.clone();

    let receipts = tracker.read_after_host_call(None, None, None).unwrap();
    fs::remove_file(path).unwrap();

    assert!(receipts.is_empty());
    assert!(!tracker.zelda_run_game_loop_call_active);
}

#[test]
fn main_loop_call_ownership_survives_checkpoint_into_continued_suffix_host() {
    let start_path = env::temp_dir().join(format!(
        "zelda3-snes9x-main-start-before-checkpoint-{}.jsonl",
        std::process::id()
    ));
    write_semantic_trace(
        &start_path,
        &[
            serde_json::json!({
                "event": "frame", "stage": "entry", "run": 81,
                "pc": 0x008034, "s": 0x01ff, "main": 0,
                "sub": 0, "subsub": 0, "frame_counter": 0,
                "nmi_latch": 0
            }),
            serde_json::json!({
                "event": "wram-write", "run": 81,
                "pc": ZELDA_RUN_GAME_LOOP_FRAME_COUNTER_WRITE_PC,
                "s": 0x01ff, "address": FRAME_COUNTER, "value": 1
            }),
            serde_json::json!({
                "event": "frame", "stage": "return", "run": 81,
                "pc": 0x0c_c1db, "s": 0x01f4, "main": 0,
                "sub": 1, "subsub": 0, "frame_counter": 1,
                "nmi_latch": 1
            }),
        ],
    );
    let mut source = empty_semantic_tracker();
    source.path = start_path.clone();
    assert_eq!(
        source.read_after_host_call(None, None, None).unwrap(),
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            MainLoopProgress::IterationStarted,
        )],
    );
    fs::remove_file(start_path).unwrap();
    assert!(source.zelda_run_game_loop_call_active);

    let checkpoint: Snes9xOracleSemanticTraceCheckpoint =
        serde_json::from_slice(&serde_json::to_vec(&source.checkpoint()).unwrap()).unwrap();
    let suffix_path = env::temp_dir().join(format!(
        "zelda3-snes9x-main-suffix-after-checkpoint-{}.jsonl",
        std::process::id()
    ));
    write_semantic_trace(
        &suffix_path,
        &[
            serde_json::json!({
                "event": "frame", "stage": "entry", "run": 82,
                "pc": 0x0c_c1db, "s": 0x01f4, "main": 0,
                "sub": 1, "subsub": 0, "frame_counter": 1,
                "nmi_latch": 1
            }),
            serde_json::json!({
                "event": "nmi", "run": 82, "pc": 0x0c_c1df,
                "s": 0x01f4, "main": 0, "sub": 1, "nmi_latch": 1,
                "nmi_ppu_register_operands": vec![0; 31]
            }),
            serde_json::json!({
                "event": "pc", "run": 82,
                "pc": NMI_HANDLER_COMPLETE_PCS[0], "s": 0x01e8
            }),
            serde_json::json!({
                "event": "nmi-resume", "run": 82,
                "pc": 0x0c_c1df, "s": 0x01f4
            }),
            serde_json::json!({
                "event": "wram-write", "run": 82,
                "pc": ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC,
                "s": 0x01ff, "address": NMI_UPDATE_LATCH, "value": 0
            }),
            serde_json::json!({
                "event": "frame", "stage": "return", "run": 82,
                "pc": 0x008034, "s": 0x01ff, "main": 0,
                "sub": 1, "subsub": 0, "frame_counter": 1,
                "nmi_latch": 0
            }),
        ],
    );
    let mut resumed = empty_semantic_tracker();
    resumed.path = suffix_path.clone();
    resumed.restore_checkpoint(checkpoint).unwrap();

    let receipts = resumed.read_after_host_call(None, None, None).unwrap();
    fs::remove_file(suffix_path).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    );
    assert!(!resumed.zelda_run_game_loop_call_active);
}

#[test]
fn main_loop_call_ownership_rejects_double_start_and_orphan_suffix() {
    for (active, event, message) in [
        (
            true,
            main_loop_start(),
            "before the prior call completed its common suffix",
        ),
        (
            false,
            main_loop_common_suffix_completion(),
            "without an active source call",
        ),
    ] {
        let path = env::temp_dir().join(format!(
            "zelda3-snes9x-invalid-main-loop-transition-{}-{active}.jsonl",
            std::process::id()
        ));
        let mut entry = frame("entry", 1, 0, 0);
        entry.pc = Some(0x008034);
        let mut returned = frame("return", 1, 0, 0);
        returned.pc = Some(0x008034);
        write_semantic_trace(
            &path,
            &[
                serde_json::to_value(entry).unwrap(),
                serde_json::to_value(event).unwrap(),
                serde_json::to_value(returned).unwrap(),
            ],
        );
        let mut tracker = empty_semantic_tracker();
        tracker.path = path.clone();
        tracker.zelda_run_game_loop_call_active = active;

        let error = tracker.read_after_host_call(None, None, None).unwrap_err();
        fs::remove_file(path).unwrap();

        assert!(error.contains(message), "unexpected error: {error}");
    }
}

#[test]
fn completed_old_call_then_new_start_preserves_both_ordered_progress_facts() {
    let path = env::temp_dir().join(format!(
        "zelda3-snes9x-main-suffix-then-new-start-{}.jsonl",
        std::process::id()
    ));
    write_semantic_trace(
        &path,
        &[
            serde_json::to_value(frame("entry", 3, 0, 1)).unwrap(),
            serde_json::to_value(main_loop_common_suffix_completion()).unwrap(),
            serde_json::to_value(main_loop_start()).unwrap(),
            serde_json::to_value(frame("return", 3, 0, 2)).unwrap(),
        ],
    );
    let mut tracker = empty_semantic_tracker();
    tracker.path = path.clone();
    tracker.zelda_run_game_loop_call_active = true;

    let receipts = tracker.read_after_host_call(None, None, None).unwrap();
    fs::remove_file(path).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::IterationStarted),
        ],
    );
    assert!(tracker.zelda_run_game_loop_call_active);
}

#[test]
fn sprite_item_receipt_call_progress_hides_cpu_provenance_and_preserves_call_order() {
    let path = env::temp_dir().join("unused-snes9x-item-receipt-progress-test.jsonl");
    let mut tracker = Snes9xOracleSemanticTrace {
        path,
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();

    tracker
        .consume_event(
            raw(
                "pc",
                // The pinned cold trace executes the symbol's $05 LoROM
                // mirror even though source listings commonly print $85.
                Some(0x05eb1d),
                Some(12),
                None,
            ),
            &mut receipts,
        )
        .unwrap();
    tracker.flush_item_receipt_progress(&mut receipts);
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
            ItemReceiptGraphicsProgressReceipt {
                caller: ItemReceiptGraphicsCaller::SpriteMain { slot: 12 },
                progress: SourceCallProgress::Suspended,
            }
        )],
    );

    receipts.clear();
    tracker.flush_item_receipt_progress(&mut receipts);
    assert_eq!(
        receipts[0],
        OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
            ItemReceiptGraphicsProgressReceipt {
                caller: ItemReceiptGraphicsCaller::SpriteMain { slot: 12 },
                progress: SourceCallProgress::Suspended,
            }
        ),
    );

    receipts.clear();
    tracker
        .consume_event(raw("pc", Some(0x05eb21), None, None), &mut receipts)
        .unwrap();
    tracker.flush_item_receipt_progress(&mut receipts);
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
            ItemReceiptGraphicsProgressReceipt {
                caller: ItemReceiptGraphicsCaller::SpriteMain { slot: 12 },
                progress: SourceCallProgress::Returned,
            }
        )],
    );

    receipts.clear();
    tracker
        .consume_event(
            raw("pc", Some(SICK_KID_ITEM_RECEIPT_CALL_PC), Some(3), None),
            &mut receipts,
        )
        .unwrap();
    tracker.flush_item_receipt_progress(&mut receipts);
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
            ItemReceiptGraphicsProgressReceipt {
                caller: ItemReceiptGraphicsCaller::SpriteMain { slot: 3 },
                progress: SourceCallProgress::Suspended,
            }
        )],
    );

    receipts.clear();
    tracker
        .consume_event(
            raw("pc", Some(SICK_KID_ITEM_RECEIPT_RETURN_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
            ItemReceiptGraphicsProgressReceipt {
                caller: ItemReceiptGraphicsCaller::SpriteMain { slot: 3 },
                progress: SourceCallProgress::Returned,
            }
        )],
    );

    receipts.clear();
    tracker
        .consume_event(
            raw(
                "pc",
                Some(UNCLE_PASSAGE_ITEM_RECEIPT_CALL_PC),
                Some(1),
                None,
            ),
            &mut receipts,
        )
        .unwrap();
    tracker.flush_item_receipt_progress(&mut receipts);
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
            ItemReceiptGraphicsProgressReceipt {
                caller: ItemReceiptGraphicsCaller::UnclePassage { slot: 1 },
                progress: SourceCallProgress::Suspended,
            }
        )],
    );

    receipts.clear();
    tracker
        .consume_event(
            raw("pc", Some(UNCLE_PASSAGE_ITEM_RECEIPT_RETURN_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
            ItemReceiptGraphicsProgressReceipt {
                caller: ItemReceiptGraphicsCaller::UnclePassage { slot: 1 },
                progress: SourceCallProgress::Returned,
            }
        )],
    );
}

#[test]
fn direct_sprite_item_graphics_return_is_distinct_from_outer_sprite_main_return() {
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();

    for (pc, x) in [
        (SPRITE_MAIN_ENTRY_PC, None),
        (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(3)),
        (LINK_RECEIVE_ITEM_ENTRY_PC, None),
    ] {
        tracker
            .consume_event(raw("pc", Some(pc), x, None), &mut receipts)
            .unwrap();
    }
    tracker.flush_item_receipt_progress(&mut receipts);
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
            ItemReceiptGraphicsProgressReceipt {
                caller: ItemReceiptGraphicsCaller::SpriteMainDirect { slot: 3 },
                progress: SourceCallProgress::Suspended,
            },
        )],
    );

    receipts.clear();
    tracker
        .consume_event(
            raw("pc", Some(LINK_RECEIVE_ITEM_GRAPHICS_RETURN_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    tracker
        .consume_event(
            raw("pc", Some(SPRITE_SLOT_RETURN_PC), None, None),
            &mut receipts,
        )
        .unwrap();
    for slot in (0..3).rev() {
        tracker
            .consume_event(
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(slot), None),
                &mut receipts,
            )
            .unwrap();
        tracker
            .consume_event(
                raw("pc", Some(SPRITE_SLOT_RETURN_PC), None, None),
                &mut receipts,
            )
            .unwrap();
    }

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                ItemReceiptGraphicsProgressReceipt {
                    caller: ItemReceiptGraphicsCaller::SpriteMainDirect { slot: 3 },
                    progress: SourceCallProgress::Returned,
                },
            ),
            OriginalTimingSemanticReceipt::SpriteMainReturned,
        ],
    );
    assert!(tracker.item_receipt_caller.is_none());
    assert!(tracker.sprite_main_execution.is_none());
}

fn save_menu_frame(stage: &str, run: u64, subsub: u8, frame_counter: u8) -> RawTraceEvent {
    let mut event = frame_with_sub(stage, run, 14, 11);
    event.subsub = Some(subsub);
    event.frame_counter = Some(frame_counter);
    event
}

#[test]
fn save_menu_initialization_reports_source_call_progress_without_cpu_state() {
    let cases = [
        (
            save_menu_frame("entry", 11616, 0, 13),
            save_menu_frame("return", 11616, 0, 14),
            MainLoopProgress::IterationStarted,
            SaveMenuInitializationProgress::InProgress,
        ),
        (
            save_menu_frame("entry", 11621, 0, 14),
            save_menu_frame("return", 11621, 1, 14),
            MainLoopProgress::CallStackContinued,
            SaveMenuInitializationProgress::Completed,
        ),
    ];

    for (entry, returned, main_progress, save_progress) in cases {
        let mut host = HostFrameWindow::default();
        host.observe(&entry).unwrap();
        if main_progress == MainLoopProgress::IterationStarted {
            host.observe(&main_loop_start()).unwrap();
        }
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();
        host.finish(
            &mut receipts,
            None,
            main_progress == MainLoopProgress::CallStackContinued,
        )
        .unwrap();
        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(main_progress),
                OriginalTimingSemanticReceipt::SaveMenuInitializationProgress(save_progress),
            ]
        );
    }
}

#[test]
fn main_wait_return_without_exact_common_suffix_does_not_claim_completion() {
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 1923, 14, 3);
    entry.pc = Some(0x00_8034);
    entry.frame_counter = Some(220);
    let mut returned = frame_with_sub("return", 1923, 14, 3);
    returned.pc = Some(0x00_8034);
    returned.frame_counter = Some(221);

    host.observe(&entry).unwrap();
    host.observe(&main_loop_start()).unwrap();
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();
    host.finish(&mut receipts, None, false).unwrap();

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            MainLoopProgress::IterationStarted,
        )],
    );
}

#[test]
fn main_wait_nmi_without_exact_common_suffix_does_not_claim_completion() {
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 326, 14, 3);
    entry.pc = Some(0x00_80c9);
    entry.frame_counter = Some(223);
    let mut accepted = raw("nmi", Some(0x00_8036), None, None);
    accepted.run = Some(326);
    let mut returned = frame_with_sub("return", 326, 14, 3);
    returned.pc = Some(0x00_80c9);
    returned.frame_counter = Some(224);

    host.observe(&entry).unwrap();
    host.observe(&main_loop_start()).unwrap();
    host.observe(&accepted).unwrap();
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();
    host.finish(&mut receipts, None, false).unwrap();

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            MainLoopProgress::IterationStarted,
        )],
    );
}

#[test]
fn continued_wait_return_without_exact_common_suffix_does_not_claim_completion() {
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 1059, 4, 1);
    entry.pc = Some(0x0c_ce3a);
    entry.nmi_latch = Some(1);
    let mut accepted = raw("nmi", Some(0x0c_ce3c), None, None);
    accepted.run = Some(1059);
    let mut returned = frame_with_sub("return", 1059, 4, 2);
    returned.pc = Some(0x00_8034);
    returned.nmi_latch = Some(0);

    host.observe(&entry).unwrap();
    host.observe(&accepted).unwrap();
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();
    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            MainLoopProgress::CallStackContinued,
        )],
    );
}

#[test]
fn continued_wait_nmi_without_exact_common_suffix_does_not_claim_completion() {
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 84, 0, 1);
    entry.pc = Some(0x0c_c1d3);
    entry.nmi_latch = Some(1);
    let mut interrupted = raw("nmi", Some(0x0c_c1d7), None, None);
    interrupted.run = Some(84);
    interrupted.nmi_latch = Some(1);
    let mut following = raw("nmi", Some(0x00_8034), None, None);
    following.run = Some(84);
    following.nmi_latch = Some(0);
    let mut returned = frame_with_sub("return", 84, 0, 1);
    returned.pc = Some(0x00_80c9);
    returned.nmi_latch = Some(0);

    assert_eq!(host.observe(&entry).unwrap(), None);
    assert_eq!(host.observe(&interrupted).unwrap(), None);
    let mut receipts = vec![
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
    ];
    assert_eq!(host.observe(&following).unwrap(), None);
    receipts.push(OriginalTimingSemanticReceipt::NmiAccepted(
        NmiUpdateGate::Open,
    ));
    assert_eq!(host.observe(&returned).unwrap(), None);
    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
        ],
    );
}

#[test]
fn continued_iteration_still_inside_module_omits_terminal_completion() {
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 1058, 4, 1);
    entry.pc = Some(0x0c_ce38);
    entry.nmi_latch = Some(1);
    let mut returned = frame_with_sub("return", 1058, 4, 1);
    returned.pc = Some(0x0c_ce3a);
    returned.nmi_latch = Some(1);

    host.observe(&entry).unwrap();
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();
    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            MainLoopProgress::CallStackContinued,
        )],
    );
}

#[test]
fn leading_nmi_at_main_wait_does_not_complete_the_later_iteration() {
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 327, 14, 3);
    entry.pc = Some(0x00_8034);
    let leading = raw("nmi", Some(0x00_8036), None, None);
    let mut returned = frame_with_sub("return", 327, 14, 3);
    returned.pc = Some(LINK_OAM_START_PC);

    host.observe(&entry).unwrap();
    host.observe(&leading).unwrap();
    host.observe(&main_loop_start()).unwrap();
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();
    host.finish(&mut receipts, None, false).unwrap();

    assert!(!receipts.contains(&OriginalTimingSemanticReceipt::MainLoopIterationReturnedToWait,));
}

#[test]
fn dialogue_scroll_copy_counts_follow_source_hosts_until_return() {
    // Source hosts 20765-20767 finish the same five-pass call as 2+2+1.
    // The third host alone reaches the scroll RTS. Register values are
    // deliberately absent: this receipt counts source operations only.
    for (entered, copies, returned) in [(true, 2, false), (false, 2, false), (false, 1, true)] {
        let mut host = DialogueScrollHostWindow::default();
        if entered {
            host.observe(&raw("pc", Some(DIALOGUE_SCROLL_ENTRY_PC), None, None))
                .unwrap();
        }
        for _ in 0..copies {
            host.observe(&raw(
                "pc",
                Some(DIALOGUE_SCROLL_PIXEL_COMPLETED_PC),
                None,
                None,
            ))
            .unwrap();
        }
        if returned {
            host.observe(&raw("pc", Some(DIALOGUE_SCROLL_RETURN_PC), None, None))
                .unwrap();
        }
        assert_eq!(
            host.finish(),
            vec![zelda3::DialogueScrollProgressReceipt {
                entered,
                completed_pixel_passes: copies,
                returned,
            }]
        );
    }
}

#[test]
fn dialogue_scroll_observation_does_not_invent_a_completed_copy() {
    let mut host = DialogueScrollHostWindow::default();
    host.observe(&raw(
        "frame",
        Some(DIALOGUE_SCROLL_PIXEL_COMPLETED_PC),
        None,
        None,
    ))
    .unwrap();
    host.observe(&raw(
        "nmi",
        Some(DIALOGUE_SCROLL_PIXEL_COMPLETED_PC),
        None,
        None,
    ))
    .unwrap();
    assert!(host.finish().is_empty());
}

#[test]
fn dialogue_scroll_calls_keep_their_order_within_one_host() {
    let mut host = DialogueScrollHostWindow::default();
    host.observe(&raw(
        "pc",
        Some(DIALOGUE_SCROLL_PIXEL_COMPLETED_PC),
        None,
        None,
    ))
    .unwrap();
    host.observe(&raw("pc", Some(DIALOGUE_SCROLL_RETURN_PC), None, None))
        .unwrap();
    host.observe(&raw("pc", Some(DIALOGUE_SCROLL_ENTRY_PC), None, None))
        .unwrap();
    assert_eq!(
        host.finish(),
        vec![
            zelda3::DialogueScrollProgressReceipt {
                entered: false,
                completed_pixel_passes: 1,
                returned: true
            },
            zelda3::DialogueScrollProgressReceipt {
                entered: true,
                completed_pixel_passes: 0,
                returned: false
            },
        ]
    );
    let mut duplicate = DialogueScrollHostWindow::default();
    duplicate
        .observe(&raw("pc", Some(DIALOGUE_SCROLL_ENTRY_PC), None, None))
        .unwrap();
    assert!(duplicate
        .observe(&raw("pc", Some(DIALOGUE_SCROLL_ENTRY_PC), None, None))
        .is_err());
}

#[test]
fn carried_glyph_resume_preserves_the_endpoint_before_scroll_entry() {
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 218, 14, 2);
    entry.pc = Some(NMI_HANDLER_ENTRY_PC);
    host.observe(&entry).unwrap();
    host.observe(&raw("nmi-resume", Some(0x0e_cca5), None, None))
        .unwrap();
    host.observe(&raw("pc", Some(DIALOGUE_SCROLL_ENTRY_PC), None, None))
        .unwrap();
    host.observe(&raw("nmi", Some(0x0e_d031), None, None))
        .unwrap();
    let mut returned = frame_with_sub("return", 218, 14, 2);
    returned.pc = Some(NMI_HANDLER_ENTRY_PC);
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();
    host.finish(&mut receipts, Some(65), true).unwrap();
    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued),
            OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                    message_read_position: 65
                },
            ),
        ]
    );
}

#[test]
fn dialogue_terminal_caller_supersedes_the_decoder_endpoint() {
    // Cold host 20440 resumes a glyph at $0E:CC2F, executes END's
    // countdown decrement, returns through the common suffix, and enters
    // the next NMI. The read cursor remains 108 at the END command.
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 440, 14, 2);
    entry.pc = Some(0x0e_cc2e);
    host.observe(&entry).unwrap();
    host.observe(&raw("nmi", Some(0x0e_cc2f), None, None))
        .unwrap();
    let mut returned = frame_with_sub("return", 440, 14, 2);
    returned.pc = Some(0x00_80c9);
    host.observe(&returned).unwrap();
    let terminal = vec![
        OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued),
        OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
    ];
    let mut receipts = terminal.clone();
    host.finish(&mut receipts, Some(108), true).unwrap();
    assert_eq!(receipts, terminal);
}

#[test]
fn unchanged_dialogue_iteration_with_vwf_nmi_becomes_a_semantic_hold() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame("entry", 36018, 14, 0x35)).unwrap();
    host.observe(&raw(
        "nmi",
        Some(VWF_RENDER_SINGLE_START_PC + 0x11),
        None,
        None,
    ))
    .unwrap();
    host.observe(&frame("return", 36018, 14, 0x35)).unwrap();
    let mut receipts = Vec::new();
    host.finish(&mut receipts, Some(0x0037), true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                    message_read_position: 0x0037,
                },
            ),
        ],
    );
}

#[test]
fn dialogue_return_inside_current_glyph_preserves_the_committed_prefix() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame("entry", 7327, 14, 0x35)).unwrap();
    host.observe(&raw("nmi", Some(VWF_RENDER_SINGLE_END_PC - 1), None, None))
        .unwrap();
    let mut returned = frame("return", 7327, 14, 0x35);
    returned.pc = Some(VWF_RENDER_SINGLE_BODY_START_PC);
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();
    host.finish(&mut receipts, Some(0x003a), true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                DialogueExecutionProgress::ResumedRenderingWithCurrentGlyphStarted {
                    message_read_position: 0x003a,
                },
            ),
        ],
    );
}

#[test]
fn dialogue_return_at_nmi_entry_preserves_the_interrupted_glyph_prefix() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame("entry", 7332, 14, 0xbf)).unwrap();
    host.observe(&raw(
        "nmi",
        Some(VWF_RENDER_SINGLE_BODY_START_PC + 0x166),
        None,
        None,
    ))
    .unwrap();
    let mut returned = frame("return", 7332, 14, 0xbf);
    returned.pc = Some(NMI_HANDLER_ENTRY_PC);
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();
    host.finish(&mut receipts, Some(0x0056), true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                DialogueExecutionProgress::ResumedRenderingWithCurrentGlyphStarted {
                    message_read_position: 0x0056,
                },
            ),
        ],
    );
}

#[test]
fn dialogue_caller_return_pc_outside_vwf_range_omits_semantic_hold() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame("entry", 2333, 14, 0x8f)).unwrap();
    host.observe(&raw("nmi", Some(0x0e_c58b), None, None))
        .unwrap();
    host.observe(&frame("return", 2333, 14, 0x8f)).unwrap();
    let mut receipts = Vec::new();
    host.finish(&mut receipts, Some(0x0037), true).unwrap();

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            MainLoopProgress::CallStackContinued,
        )],
        "$0e:c58b is Text_LoadCharacterBuffer's caller, not the VWF render loop",
    );
}

#[test]
fn dialogue_terminal_common_suffix_keeps_progress_without_vwf_endpoint() {
    let path = env::temp_dir().join(format!(
        "zelda3-snes9x-dialogue-terminal-common-suffix-{}.jsonl",
        std::process::id()
    ));
    let events = [
        serde_json::json!({
            "event": "frame", "stage": "entry", "run": 2334,
            "pc": NMI_HANDLER_ENTRY_PC, "s": 0x01ee, "main": 14,
            "sub": 2, "subsub": 0, "frame_counter": 0x8f,
            "nmi_latch": 1
        }),
        serde_json::json!({
            "event": "pc", "run": 2334,
            "pc": NMI_HANDLER_COMPLETE_PCS[0], "s": 0x01e5
        }),
        serde_json::json!({
            "event": "nmi-resume", "run": 2334,
            "pc": 0x0e_c58b, "s": 0x01f2
        }),
        serde_json::json!({
            "event": "wram-write", "run": 2334,
            "pc": ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC,
            "s": 0x01ff, "address": NMI_UPDATE_LATCH, "value": 0
        }),
        serde_json::json!({
            "event": "frame", "stage": "return", "run": 2334,
            "pc": 0x00_8034, "s": 0x01ff, "main": 14, "sub": 2,
            "subsub": 0, "frame_counter": 0x8f, "nmi_latch": 0
        }),
    ];
    write_semantic_trace(&path, &events);
    let mut tracker = empty_semantic_tracker();
    tracker.path = path.clone();
    tracker.zelda_run_game_loop_call_active = true;
    tracker.nmi_publication_pending = true;
    tracker.pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    tracker.nmi_resume_targets.push((0x0e_c58b, 0x01f2));

    let receipts = tracker
        .read_after_host_call(Some(0x0037), None, None)
        .unwrap();
    fs::remove_file(path).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    );
    assert!(!tracker.nmi_publication_pending);
    assert!(tracker.pending_nmi_update_gate.is_none());
    assert!(tracker.nmi_resume_targets.is_empty());
}

#[test]
fn pre_overworld_register_transitions_become_source_stage_receipts() {
    let cases = [
        (8, 0, 8, 1, PreOverworldStageCompletion::PropertiesReturned),
        (8, 1, 8, 2, PreOverworldStageCompletion::OverlaysReturned),
        (
            8,
            2,
            16,
            0,
            PreOverworldStageCompletion::ScreenBuildReturned,
        ),
    ];
    for (entry_main, entry_sub, return_main, return_sub, expected) in cases {
        let mut host = HostFrameWindow::default();
        host.observe(&frame_with_sub("entry", 7, entry_main, entry_sub))
            .unwrap();
        host.observe(&frame_with_sub("return", 7, return_main, return_sub))
            .unwrap();
        let mut receipts = Vec::new();
        host.finish(&mut receipts, None, true).unwrap();
        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::PreOverworldStageCompleted(expected),
            ]
        );
    }
}

#[test]
fn overworld_sprite_reload_return_becomes_a_backend_neutral_completion_receipt() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame_with_sub("entry", 38517, 9, 4)).unwrap();
    host.observe(&frame_with_sub("return", 38517, 9, 5))
        .unwrap();
    let mut receipts = Vec::new();

    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::ReloadReturned,
            ),
        ],
    );
}

#[test]
fn link_oam_equipment_selector_publishes_its_native_prefix() {
    assert_eq!(
        link_oam_stair_progress(0x0d_a8b6, Some(18)),
        Some(zelda3::LinkOamStairProgress::ShadowSelection)
    );
    assert_eq!(
        link_oam_stair_progress(0x0d_a992, Some(18)),
        Some(zelda3::LinkOamStairProgress::BodySelection)
    );
    assert_eq!(
        link_oam_stair_progress(0x0d_a47e, Some(18)),
        Some(zelda3::LinkOamStairProgress::PoseSelected)
    );
    assert_eq!(link_oam_stair_progress(0x0d_a47b, Some(18)), None);
    assert_eq!(link_oam_stair_progress(0x0d_a47e, Some(1)), None);
    let mut host = HostFrameWindow::default();
    host.observe(&frame_with_sub("entry", 1, 7, 18)).unwrap();
    let mut returned = frame_with_sub("return", 1, 7, 18);
    returned.pc = Some(0x0d_a61a);
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();
    host.finish(&mut receipts, None, true).unwrap();
    assert!(
        receipts.contains(&OriginalTimingSemanticReceipt::MainLoopInterrupted(
            MainLoopInterruption::LinkOam,
        ))
    );
    assert!(
        receipts.contains(&OriginalTimingSemanticReceipt::LinkOamStairProgress(
            zelda3::LinkOamStairProgress::EquipmentSelection
        ))
    );
}

#[test]
fn overworld_map_quadrant_publication_becomes_a_backend_neutral_receipt() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame_with_sub("entry", 39261, 9, 3)).unwrap();
    host.observe(&frame_with_sub("return", 39261, 9, 4))
        .unwrap();
    let mut receipts = Vec::new();

    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::OverworldMapQuadrantsPublished,
        ],
    );
}

#[test]
fn world_map_ambient_map8_return_becomes_a_backend_neutral_receipt() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame_with_sub("entry", 6173, 9, 0x21))
        .unwrap();
    host.observe(&frame_with_sub("return", 6173, 9, 0x22))
        .unwrap();
    let mut receipts = Vec::new();

    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::WorldMapAmbientMap8Returned,
        ],
    );
}

#[test]
fn special_exit_mosaic_return_becomes_a_backend_neutral_receipt() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame_with_sub("entry", 7426, 0x0b, 0x24))
        .unwrap();
    host.observe(&frame_with_sub("return", 7426, 0x0b, 0x25))
        .unwrap();
    let mut receipts = Vec::new();

    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::OverworldSpecialExitMosaicReturned,
        ],
    );
}

#[test]
fn special_exit_second_decode_entry_proves_the_restore_prefix() {
    let mut event = raw("pc", Some(DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC), None, None);
    event.return_address = Some(SPECIAL_EXIT_MOSAIC_SECOND_DECODE_RETURN_ADDRESS);
    event.main = Some(0x0b);
    event.sub = Some(0x24);

    assert!(special_exit_mosaic_restore_checkpoint(&event).unwrap());

    event.sub = Some(0x23);
    assert!(special_exit_mosaic_restore_checkpoint(&event).is_err());
}

#[test]
fn special_exit_terminal_return_supersedes_the_restore_checkpoint() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame_with_sub("entry", 7426, 0x0b, 0x24))
        .unwrap();
    host.observe(&frame_with_sub("return", 7426, 0x0b, 0x25))
        .unwrap();
    let mut receipts = vec![OriginalTimingSemanticReceipt::OverworldSpecialExitMosaicRestored];

    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::OverworldSpecialExitMosaicReturned,
        ],
    );
}

#[test]
fn world_map_overlay_reload_return_becomes_a_backend_neutral_receipt() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame_with_sub("entry", 6168, 9, 0x20))
        .unwrap();
    host.observe(&frame_with_sub("return", 6168, 9, 0x21))
        .unwrap();
    let mut receipts = Vec::new();

    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::WorldMapOverlayReloadReturned,
        ],
    );
}

#[test]
fn dungeon_exit_spotlight_entry_return_becomes_a_backend_neutral_receipt() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame_with_sub("entry", 39630, 15, 0))
        .unwrap();
    host.observe(&frame_with_sub("return", 39630, 15, 1))
        .unwrap();
    let mut receipts = Vec::new();

    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned,
        ],
    );
}

#[test]
fn dungeon_exit_spotlight_entry_preserves_its_partial_link_movement() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame_with_sub("entry", 24_193, 0x0f, 0))
        .unwrap();
    let mut returned = frame_with_sub("return", 24_193, 0x0f, 1);
    returned.pc = Some(0x07_e3c5);
    returned.x = Some(0);
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();

    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::LinkPositionAfterSubpixel { pass: 0 },
            ),
            OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned,
        ],
    );
}

#[test]
fn recurring_spotlight_caller_return_to_main_wait_is_a_semantic_receipt() {
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 4789, 15, 1);
    entry.pc = Some(NMI_HANDLER_ENTRY_PC);
    let mut returned = frame_with_sub("return", 4789, 15, 1);
    returned.pc = Some(0x00_8036);
    host.observe(&entry).unwrap();
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();

    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
        ],
    );

    let mut idle = HostFrameWindow::default();
    let mut idle_entry = frame_with_sub("entry", 4790, 15, 1);
    idle_entry.pc = Some(0x00_8036);
    let mut idle_return = frame_with_sub("return", 4790, 15, 1);
    idle_return.pc = Some(0x00_8036);
    idle.observe(&idle_entry).unwrap();
    idle.observe(&idle_return).unwrap();
    let mut idle_receipts = Vec::new();
    idle.finish(&mut idle_receipts, None, false).unwrap();
    assert!(
        idle_receipts.is_empty(),
        "an idle main-wait host invented a suspended C caller",
    );
}

#[test]
fn recurring_spotlight_caller_return_before_following_nmi_is_a_semantic_receipt() {
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 39634, 15, 1);
    entry.pc = Some(0x00_f3bf);
    entry.nmi_latch = Some(1);
    let mut returned = frame_with_sub("return", 39634, 15, 1);
    returned.pc = Some(NMI_HANDLER_ENTRY_PC);
    returned.nmi_latch = Some(0);
    host.observe(&entry).unwrap();
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();

    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
        ],
        "ZeldaRunGameLoop clears nmi_boolean only after Module0F and NMI_PrepareSprites return",
    );

    let mut interrupted = HostFrameWindow::default();
    let mut interrupted_entry = frame_with_sub("entry", 39635, 15, 1);
    interrupted_entry.pc = Some(0x00_f3bf);
    interrupted_entry.nmi_latch = Some(1);
    let mut interrupted_return = frame_with_sub("return", 39635, 15, 1);
    interrupted_return.pc = Some(NMI_HANDLER_ENTRY_PC);
    interrupted_return.nmi_latch = Some(1);
    interrupted.observe(&interrupted_entry).unwrap();
    interrupted.observe(&interrupted_return).unwrap();
    let mut interrupted_receipts = Vec::new();
    interrupted
        .finish(&mut interrupted_receipts, None, true)
        .unwrap();
    assert_eq!(
        interrupted_receipts,
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            MainLoopProgress::CallStackContinued,
        )],
        "an NMI accepted while Module0F remains active cannot fabricate a caller return",
    );
}

#[test]
fn overworld_sprite_writes_become_backend_neutral_publication_receipts() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();
    let slot = 13u8;
    let writes = [
        (SPRITE_N_WORD_BASE + u16::from(slot) * 2, 0x98),
        (SPRITE_N_WORD_BASE + u16::from(slot) * 2 + 1, 0x01),
        (SPRITE_TYPE_BASE + u16::from(slot), 0xac),
        (SPRITE_STATE_BASE + u16::from(slot), 8),
        (SPRITE_DIE_ACTION_BASE + u16::from(slot), 0),
    ];
    for (index, (address, value)) in writes.into_iter().enumerate() {
        let mut event = raw(
            "wram-write",
            Some(OVERWORLD_LOAD_SINGLE_SPRITE_START_PC + 1),
            Some(if index < 2 {
                u16::from(slot) * 2
            } else {
                u16::from(slot)
            }),
            Some(address),
        );
        event.main = Some(8);
        event.sub = Some(0);
        event.value = Some(value);
        tracker.consume_event(event, &mut receipts).unwrap();
    }

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::PresencePublished,
            ),
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::SpriteActivated {
                    block: 0x0198,
                    slot,
                    sprite_type: 0xac,
                },
            ),
        ],
    );
}

#[test]
fn overworld_load_overlays_call_identity_owns_cross_host_sprite_publication_and_return() {
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();
    let mut entry = raw(
        "pc",
        Some(OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_ENTRY_PC),
        None,
        None,
    );
    entry.return_address = Some(OVERWORLD_LOAD_OVERLAYS_AFTER_SPRITE_RELOAD_PC);
    entry.main = Some(0x0b);
    entry.sub = Some(0x25);
    tracker.consume_event(entry, &mut receipts).unwrap();
    assert!(tracker.overworld_load_overlays_sprite_reload_active);

    let slot = 13u8;
    let writes = [
        (SPRITE_N_WORD_BASE + u16::from(slot) * 2, 0x98),
        (SPRITE_N_WORD_BASE + u16::from(slot) * 2 + 1, 0x01),
        (SPRITE_TYPE_BASE + u16::from(slot), 0xac),
        (SPRITE_STATE_BASE + u16::from(slot), 8),
        (SPRITE_DIE_ACTION_BASE + u16::from(slot), 0),
    ];
    for (index, (address, value)) in writes.into_iter().enumerate() {
        let mut event = raw(
            "wram-write",
            Some(OVERWORLD_LOAD_SINGLE_SPRITE_START_PC + 1),
            Some(if index < 2 {
                u16::from(slot) * 2
            } else {
                u16::from(slot)
            }),
            Some(address),
        );
        event.main = Some(0x0b);
        // The source module bytes do not prove this inner call. Use a
        // different direct-dispatch submodule to ensure the entry/return
        // identity, not the old Module0B/$18 special case, owns it.
        event.sub = Some(0x25);
        event.value = Some(value);
        tracker.consume_event(event, &mut receipts).unwrap();
    }

    let mut returned = raw(
        "pc",
        Some(OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_RETURN_PC),
        None,
        None,
    );
    returned.main = Some(0x0b);
    returned.sub = Some(0x25);
    tracker.consume_event(returned, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::SpriteActivated {
                    block: 0x0198,
                    slot,
                    sprite_type: 0xac,
                },
            ),
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::GenerationReturned,
            ),
        ],
    );
    assert!(!tracker.overworld_load_overlays_sprite_reload_active);
}

#[test]
fn module09_scroll_prefix_is_distinct_from_sprite_main_entry() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame_with_sub("entry", 1, 9, 4)).unwrap();
    let mut returned = frame_with_sub("return", 1, 9, 5);
    returned.pc = Some(0x02_a4aa);
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();
    host.finish(&mut receipts, None, true).unwrap();
    assert!(receipts.contains(&OriginalTimingSemanticReceipt::Module09FinalScrollPairPending));
    assert!(receipts.contains(
        &OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
            OverworldSpriteReloadProgress::ReloadReturned
        )
    ));
    assert!(!receipts.contains(&OriginalTimingSemanticReceipt::SpriteMainReturned));
}

#[test]
fn sprite_main_proximity_helper_does_not_publish_reload_authority() {
    let mut source = empty_semantic_tracker();
    source.sprite_main_execution = Some(SpriteMainExecutionTracker::default());
    let mut event = raw("nmi", Some(0x09_c5fc), Some(0), None);
    event.main = Some(9);
    event.sub = Some(4);
    event.bg2_h = Some(0x08a8);
    let mut receipts = Vec::new();
    assert!(!source.publish_overworld_presence_at_scan_boundary(&event, &mut receipts));
    assert!(receipts.is_empty());
    source.sprite_main_execution = None;
    assert!(source.publish_overworld_presence_at_scan_boundary(&event, &mut receipts));
    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::PresencePublished
            )
        ]
    );
}

#[test]
fn mirror_portal_reset_requires_the_private_spawn_caller() {
    for caller in [0x09_afa5, 0x06_8b5f] {
        let mut host = HostFrameWindow::default();
        host.observe(&frame_with_sub("entry", 1, 9, 0x23)).unwrap();
        let mut spawn = raw(
            "wram-write",
            Some(SPRITE_SPAWN_DYNAMICALLY_FIRST_TYPE_STORE_PC),
            Some(0xff),
            Some(15),
        );
        spawn.return_address = Some(caller);
        spawn.y = Some(15);
        spawn.address = Some(SPRITE_TYPE_BASE + 15);
        spawn.value = Some(0x6c);
        host.observe(&spawn).unwrap();
        let mut returned = frame_with_sub("return", 1, 9, 0x23);
        returned.pc = Some(0x0d_b889);
        returned.x = Some(15);
        returned.return_address = Some(SPRITE_PREP_LOAD_PROPERTIES_AFTER_RESET_RETURN_ADDRESS);
        host.observe(&returned).unwrap();
        let mut receipts = vec![
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::GenerationReturned,
            ),
        ];
        host.finish(&mut receipts, None, true).unwrap();
        let expected = if caller == 0x09_afa5 {
            OverworldSpriteReloadProgress::GenerationReturnedAtPortalReset {
                slot: 15,
                completed_stores: 8,
            }
        } else {
            OverworldSpriteReloadProgress::GenerationReturned
        };
        assert!(receipts
            .contains(&OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(expected)));
    }
}

#[test]
fn mirror_cleanup_boundary_retains_generation_return_and_pending_slot() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame_with_sub("entry", 1, 9, 0x23)).unwrap();
    let mut returned = frame_with_sub("return", 1, 9, 0x23);
    returned.pc = Some(0x09_ac9c);
    returned.x = Some(4);
    host.observe(&returned).unwrap();
    let mut receipts = vec![
        OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
            OverworldSpriteReloadProgress::GenerationReturned,
        ),
    ];
    host.finish(&mut receipts, None, true).unwrap();
    assert!(receipts.contains(
        &OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
            OverworldSpriteReloadProgress::GenerationReturnedAtInteractiveCleanup { slot: 4 }
        )
    ));
    assert!(!receipts.contains(
        &OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
            OverworldSpriteReloadProgress::GenerationReturned
        )
    ));
}

#[test]
fn mirror_portal_property_load_is_owned_by_the_suspended_spawn() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame_with_sub("entry", 1, 9, 0x23)).unwrap();
    let mut spawn = raw(
        "wram-write",
        Some(SPRITE_SPAWN_DYNAMICALLY_FIRST_TYPE_STORE_PC),
        Some(0xff),
        None,
    );
    spawn.y = Some(15);
    spawn.return_address = Some(0x09_afa5);
    spawn.address = Some(SPRITE_TYPE_BASE + 15);
    spawn.value = Some(0x6c);
    host.observe(&spawn).unwrap();
    let mut nmi = raw("nmi", Some(0x0d_b844), Some(15), None);
    nmi.return_address = Some(0xa60f09);
    host.observe(&nmi).unwrap();
    assert_eq!(host.mirror_portal_load_progress, Some(5));
    let mut returned = frame_with_sub("return", 1, 9, 0x23);
    returned.pc = Some(0x0080c9);
    host.observe(&returned).unwrap();
    let mut receipts = vec![
        OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
            OverworldSpriteReloadProgress::GenerationReturned,
        ),
    ];
    host.finish(&mut receipts, None, true).unwrap();
    assert!(receipts.contains(
        &OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
            OverworldSpriteReloadProgress::GenerationReturnedAtPortalLoadProperties {
                slot: 15,
                completed_stores: 5
            }
        )
    ));
    nmi.event = "nmi-resume".into();
    let mut resumed_host = HostFrameWindow {
        mirror_portal_load_progress: Some(5),
        ..Default::default()
    };
    resumed_host.observe(&nmi).unwrap();
    assert_eq!(resumed_host.mirror_portal_load_progress, None);
}

#[test]
fn mirror_type_clear_boundary_defers_portal_spawn() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame_with_sub("entry", 1, 9, 0x23)).unwrap();
    let mut returned = frame_with_sub("return", 1, 9, 0x23);
    returned.pc = Some(0x09_aca6);
    returned.x = Some(1);
    host.observe(&returned).unwrap();
    let mut receipts = vec![
        OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
            OverworldSpriteReloadProgress::GenerationReturned,
        ),
    ];
    host.finish(&mut receipts, None, true).unwrap();
    assert!(receipts.contains(
        &OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
            OverworldSpriteReloadProgress::GenerationReturnedAtInteractiveTypeClear { slot: 1 }
        )
    ));
    assert!(!receipts.contains(
        &OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
            OverworldSpriteReloadProgress::GenerationReturned
        )
    ));
}

#[test]
fn mirror_warp_call_identity_owns_module09_23_scan_and_generation_return() {
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();
    let mut entry = raw(
        "pc",
        Some(OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_ENTRY_PC),
        None,
        None,
    );
    entry.return_address = Some(MIRROR_WARP_AFTER_SPRITE_RELOAD_PC);
    entry.main = Some(9);
    entry.sub = Some(0x23);
    tracker.consume_event(entry, &mut receipts).unwrap();
    assert!(tracker.overworld_load_overlays_sprite_reload_active);

    let mut nmi = raw(
        "nmi",
        Some(OVERWORLD_SPRITE_SCAN_START_PC + 0x1b0),
        None,
        None,
    );
    nmi.main = Some(9);
    nmi.sub = Some(0x23);
    nmi.nmi_latch = Some(1);
    nmi.bg2_h = Some(0x01fa);
    tracker.consume_event(nmi, &mut receipts).unwrap();
    assert!(receipts.contains(
        &OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
            OverworldSpriteReloadProgress::PresencePublished,
        ),
    ));
    assert!(receipts.contains(
        &OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
            OverworldSpriteReloadProgress::ProximityScanSuspended { bg2_h: 0x01fa },
        ),
    ));

    publish_nmi(&mut tracker, &mut receipts);
    tracker
        .consume_event(
            raw(
                "nmi-resume",
                Some(OVERWORLD_SPRITE_SCAN_START_PC + 0x1b0),
                None,
                None,
            ),
            &mut receipts,
        )
        .unwrap();
    let mut returned = raw(
        "pc",
        Some(OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_RETURN_PC),
        None,
        None,
    );
    returned.main = Some(9);
    returned.sub = Some(0x23);
    tracker.consume_event(returned, &mut receipts).unwrap();
    assert!(receipts.contains(
        &OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
            OverworldSpriteReloadProgress::GenerationReturned,
        ),
    ));
    assert!(!tracker.overworld_load_overlays_sprite_reload_active);
}

#[test]
fn save_quit_intro_return_is_superseded_by_terminal_reset_state() {
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();
    let mut event = raw("wram-write", Some(0x0c_c25b), None, None);
    event.address = Some(0x11);
    event.value = Some(2);
    event.main = Some(0x17);
    event.sub = Some(1);
    event.return_address = Some(0x0c_f0e8);
    tracker.consume_event(event, &mut receipts).unwrap();
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SaveQuitIntroMemoryReturned]
    );
    let mut returned = raw(
        "pc",
        Some(SAVE_QUIT_RESET_DUNGEON_INFO_CLEAR_ENTRY_PC),
        None,
        None,
    );
    returned.main = Some(0);
    returned.sub = Some(10);
    returned.subsub = Some(10);
    tracker.consume_event(returned, &mut receipts).unwrap();
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SaveQuitResetStatePublished]
    );
}

#[test]
fn save_quit_reset_dungeon_info_clear_entry_publishes_typed_state_boundary() {
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();
    let mut event = raw(
        "pc",
        Some(SAVE_QUIT_RESET_DUNGEON_INFO_CLEAR_ENTRY_PC),
        None,
        None,
    );
    event.main = Some(0);
    event.sub = Some(10);
    event.subsub = Some(10);

    tracker.consume_event(event, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SaveQuitResetStatePublished],
    );
}

#[test]
fn file_select_graphics_low_wram_store_reports_exact_source_prefix() {
    let mut event = raw(
        "pc",
        Some(FILE_SELECT_GRAPHICS_LOW_WRAM_CLEAR_AFTER_PAGE_D_PC),
        None,
        None,
    );
    event.main = Some(1);
    event.sub = Some(1);
    event.subsub = Some(0xd6);
    event.x = Some(0x00fe);

    assert_eq!(
        file_select_graphics_low_wram_clear_progress(&event).unwrap(),
        Some(FileSelectGraphicsLowWramClearProgress {
            word_offset: 0xfe,
            completed_page_stores: 1,
        }),
    );

    event.pc = Some(FILE_SELECT_GRAPHICS_LOW_WRAM_CLEAR_AFTER_PAGE_F_PC);
    event.x = Some(0x00fc);
    assert_eq!(
        file_select_graphics_low_wram_clear_progress(&event).unwrap(),
        Some(FileSelectGraphicsLowWramClearProgress {
            word_offset: 0xfc,
            completed_page_stores: 3,
        }),
    );
}

#[test]
fn file_select_graphics_low_wram_clear_return_publishes_typed_boundary() {
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();
    let mut event = raw(
        "pc",
        Some(FILE_SELECT_GRAPHICS_LOW_WRAM_CLEAR_RETURN_PC),
        None,
        None,
    );
    event.main = Some(1);
    event.sub = Some(1);
    event.subsub = Some(0xd6);

    tracker.consume_event(event, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::FileSelectGraphicsLowWramCleared],
    );
}

#[test]
fn module05_show_text_message_return_publishes_typed_interface_boundary() {
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();
    let mut event = raw(
        "pc",
        Some(SELECTED_GAME_LOAD_MESSAGE_INTERFACE_RETURN_PC),
        None,
        None,
    );
    event.return_address = Some(MODULE05_AFTER_SHOW_TEXT_MESSAGE_PC);
    event.main = Some(14);
    event.sub = Some(2);

    tracker.consume_event(event, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SelectedGameLoadMessageInterfacePublished,],
    );
}

#[test]
fn show_text_message_return_from_an_unrelated_caller_is_not_module05_publication() {
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();
    let mut event = raw(
        "pc",
        Some(SELECTED_GAME_LOAD_MESSAGE_INTERFACE_RETURN_PC),
        None,
        None,
    );
    event.return_address = Some(MODULE05_AFTER_SHOW_TEXT_MESSAGE_PC + 1);
    event.main = Some(14);
    event.sub = Some(2);

    tracker.consume_event(event, &mut receipts).unwrap();

    assert!(receipts.is_empty());
}

#[test]
fn overworld_load_overlays_call_identity_survives_semantic_checkpoint() {
    let mut source = empty_semantic_tracker();
    let mut receipts = Vec::new();
    let mut entry = raw(
        "pc",
        Some(OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_ENTRY_PC),
        None,
        None,
    );
    entry.return_address = Some(OVERWORLD_LOAD_OVERLAYS_AFTER_SPRITE_RELOAD_PC);
    source.consume_event(entry, &mut receipts).unwrap();

    let checkpoint = source.checkpoint();
    let mut resumed = empty_semantic_tracker();
    resumed.restore_checkpoint(checkpoint).unwrap();
    assert!(resumed.overworld_load_overlays_sprite_reload_active);

    resumed
        .consume_event(
            raw(
                "pc",
                Some(OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_RETURN_PC),
                None,
                None,
            ),
            &mut receipts,
        )
        .unwrap();
    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::GenerationReturned,
            ),
        ],
    );
}

#[test]
fn module0b_held_host_publishes_proximity_scan_scratch_coordinate() {
    let mut host = HostFrameWindow::default();
    host.overworld_load_overlays_sprite_reload_active = true;
    let mut entry = frame_with_sub("entry", 165775, 0x0b, 0x18);
    entry.bg2_h = Some(0x021e);
    let mut returned = frame_with_sub("return", 165775, 0x0b, 0x18);
    returned.pc = Some(OVERWORLD_SPRITE_SCAN_START_PC + 1);
    returned.bg2_h = Some(0x028e);

    host.observe(&entry).unwrap();
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();
    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::ProximityScanSuspended { bg2_h: 0x028e },
            ),
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
        ],
    );
}

#[test]
fn pre_overworld_held_host_publishes_proximity_scan_scratch_coordinate() {
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 652122, 0x08, 0);
    entry.bg2_h = Some(0x0700);
    let mut returned = frame_with_sub("return", 652122, 0x08, 0);
    returned.pc = Some(OVERWORLD_SPRITE_SCAN_START_PC + 1);
    returned.bg2_h = Some(0x0720);

    host.observe(&entry).unwrap();
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();
    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::ProximityScanSuspended { bg2_h: 0x0720 },
            ),
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
        ],
    );
}

#[test]
fn module0b_fresh_iteration_does_not_publish_held_scan_scratch() {
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 165774, 0x0b, 0x18);
    entry.bg2_h = Some(0x021e);
    let mut returned = frame_with_sub("return", 165774, 0x0b, 0x18);
    returned.bg2_h = Some(0x022e);

    host.observe(&entry).unwrap();
    let mut reload_entry = raw(
        "pc",
        Some(OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_ENTRY_PC),
        None,
        None,
    );
    reload_entry.return_address = Some(OVERWORLD_LOAD_OVERLAYS_AFTER_SPRITE_RELOAD_PC);
    host.observe(&reload_entry).unwrap();
    host.observe(&main_loop_start()).unwrap();
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();
    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            MainLoopProgress::IterationStarted,
        )],
    );
}

#[test]
fn unrelated_module09_sprite_writes_do_not_publish_reload_activation() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();
    let slot = 13u8;
    let writes = [
        (SPRITE_N_WORD_BASE + u16::from(slot) * 2, 0x98),
        (SPRITE_N_WORD_BASE + u16::from(slot) * 2 + 1, 0x01),
        (SPRITE_TYPE_BASE + u16::from(slot), 0xac),
        (SPRITE_STATE_BASE + u16::from(slot), 8),
        (SPRITE_DIE_ACTION_BASE + u16::from(slot), 0),
    ];
    for (index, (address, value)) in writes.into_iter().enumerate() {
        let mut event = raw(
            "wram-write",
            Some(OVERWORLD_LOAD_SINGLE_SPRITE_START_PC + 1),
            Some(if index < 2 {
                u16::from(slot) * 2
            } else {
                u16::from(slot)
            }),
            Some(address),
        );
        event.main = Some(9);
        event.sub = Some(6);
        event.value = Some(value);
        tracker.consume_event(event, &mut receipts).unwrap();
    }

    assert!(receipts.is_empty());
    assert!(tracker.overworld_sprite_activation.is_none());
}

#[test]
fn host_return_inside_flute_scan_publishes_presence_before_scan_progress() {
    let mut tracker = empty_semantic_tracker();
    tracker.overworld_load_overlays_sprite_reload_active = true;
    let mut host = HostFrameWindow::default();
    host.overworld_load_overlays_sprite_reload_active = true;
    host.observe(&frame_with_sub("entry", 1, 0x0e, 0x0a))
        .unwrap();
    let mut returned = frame_with_sub("return", 1, 0x0e, 0x0a);
    returned.pc = Some(0x09_c723);
    returned.bg2_h = Some(128);
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();
    assert!(tracker.publish_overworld_presence_at_scan_boundary(&returned, &mut receipts));
    host.finish(&mut receipts, None, true).unwrap();
    assert_eq!(
        &receipts[..2],
        &[
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::PresencePublished
            ),
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::ProximityScanSuspended { bg2_h: 128 }
            ),
        ]
    );
    let mut later = Vec::new();
    assert!(tracker.publish_overworld_presence_at_scan_boundary(&returned, &mut later));
    assert!(later.is_empty());
}

#[test]
fn nmi_inside_overworld_sprite_scan_publishes_presence_once() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();
    for _ in 0..2 {
        let mut event = raw("nmi", Some(OVERWORLD_SPRITE_SCAN_START_PC + 1), None, None);
        event.main = Some(8);
        event.sub = Some(0);
        event.bg2_h = Some(0x0720);
        tracker.consume_event(event, &mut receipts).unwrap();
        publish_nmi(&mut tracker, &mut receipts);
    }

    assert_eq!(
        receipts
            .iter()
            .filter(|receipt| matches!(
                receipt,
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    OverworldSpriteReloadProgress::PresencePublished
                )
            ))
            .count(),
        1,
    );
    assert_eq!(
        receipts
            .iter()
            .filter(|receipt| matches!(
                receipt,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open)
            ))
            .count(),
        2,
    );
}

#[test]
fn pre_dungeon_return_is_distinct_from_the_next_main_iteration() {
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 49414, 6, 0);
    entry.frame_counter = Some(218);
    let mut returned = frame_with_sub("return", 49414, 7, 15);
    returned.frame_counter = Some(218);
    host.observe(&entry).unwrap();
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();

    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::PreDungeonModuleReturned,
        ],
    );
}

#[test]
fn dialogue_iteration_that_advanced_is_not_reported_as_a_resumed_hold() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame("entry", 36014, 14, 0x34)).unwrap();
    host.observe(&main_loop_start()).unwrap();
    host.observe(&raw("nmi", Some(VWF_RENDER_SINGLE_END_PC - 1), None, None))
        .unwrap();
    host.observe(&frame("return", 36014, 14, 0x35)).unwrap();
    let mut receipts = Vec::new();
    host.finish(&mut receipts, None, false).unwrap();

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            MainLoopProgress::IterationStarted,
        )],
    );
}

#[test]
fn dialogue_return_to_gameplay_becomes_a_backend_neutral_close_receipt() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame_with_sub("entry", 9, 14, 2)).unwrap();
    host.observe(&main_loop_start()).unwrap();
    let mut returned = frame_with_sub("return", 9, 9, 0);
    returned.frame_counter = Some(1);
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();

    host.finish(&mut receipts, None, false).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::IterationStarted,),
            OriginalTimingSemanticReceipt::DialogueClosed,
        ],
    );
}

#[test]
fn host_interval_rejects_multiple_main_loop_starts() {
    let mut host = HostFrameWindow::default();
    host.observe(&frame("entry", 7, 9, 0xfe)).unwrap();
    host.observe(&main_loop_start()).unwrap();
    host.observe(&main_loop_start()).unwrap();
    host.observe(&frame("return", 7, 9, 0x00)).unwrap();
    let mut receipts = Vec::new();

    assert_eq!(
        host.finish(&mut receipts, None, false).unwrap_err(),
        "Snes9x host call started ZeldaRunGameLoop 2 times; expected zero or one",
    );
    assert!(receipts.is_empty());
}

#[test]
fn cold_initialization_frame_counter_clear_is_not_a_main_loop_start() {
    let mut host = HostFrameWindow::default();
    let mut entry = frame("entry", 80, 0x55, 0x55);
    entry.pc = Some(0x0087cb);
    entry.nmi_latch = Some(0x55);
    host.observe(&entry).unwrap();
    host.observe(&raw(
        "wram-write",
        Some(0x0087ce),
        None,
        Some(FRAME_COUNTER),
    ))
    .unwrap();
    let mut returned = frame("return", 80, 0, 0);
    returned.pc = Some(0x008034);
    returned.nmi_latch = Some(0);
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();

    host.finish(&mut receipts, None, false).unwrap();

    assert!(receipts.is_empty());
}

#[test]
fn nmi_inside_common_sprite_preparation_becomes_a_backend_neutral_receipt() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();

    tracker
        .consume_event(raw("nmi", Some(0x00_8751), None, None), &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpritePreparation,
            ),
        ],
    );
}

#[test]
fn nmi_inside_extended_oam_pack_exports_only_the_resumable_source_cursor() {
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();
    let mut event = raw("nmi", Some(0x00_860f), None, None);
    event.y = Some(4);
    event.x = Some(16);
    event.nmi_latch = Some(1);

    tracker.consume_event(event, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpritePreparationExtendedOamPacking {
                    next_group_start: 4,
                },
            ),
        ],
    );
}

#[test]
fn extended_oam_first_store_opcode_is_still_an_unpublished_group_boundary() {
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();
    let mut event = raw("nmi", Some(0x00_8614), None, None);
    event.y = Some(4);
    event.x = Some(16);
    event.nmi_latch = Some(1);

    tracker.consume_event(event, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpritePreparationExtendedOamPacking {
                    next_group_start: 4,
                },
            ),
        ],
    );
}

#[test]
fn extended_oam_pack_receipt_rejects_a_missing_cursor_before_mutation() {
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();
    let mut event = raw("nmi", Some(0x00_860f), None, None);
    event.x = Some(16);
    event.nmi_latch = Some(1);

    let error = tracker.consume_event(event, &mut receipts).unwrap_err();

    assert!(error.contains("omitted source cursor Y"));
    assert!(receipts.is_empty());
    assert!(!tracker.nmi_publication_pending);
    assert_eq!(tracker.pending_nmi_update_gate, None);
    assert!(tracker.nmi_resume_targets.is_empty());
}

#[test]
fn extended_oam_pack_receipt_rejects_an_invalid_cursor_before_mutation() {
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();
    let mut event = raw("nmi", Some(0x00_860f), None, None);
    event.y = Some(6);
    event.x = Some(24);
    event.nmi_latch = Some(1);

    let error = tracker.consume_event(event, &mut receipts).unwrap_err();

    assert!(error.contains("invalid group cursor 6"));
    assert!(receipts.is_empty());
    assert!(!tracker.nmi_publication_pending);
    assert_eq!(tracker.pending_nmi_update_gate, None);
    assert!(tracker.nmi_resume_targets.is_empty());
}

#[test]
fn extended_oam_pack_receipt_rejects_disagreeing_source_cursors_before_mutation() {
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();
    let mut event = raw("nmi", Some(0x00_860f), None, None);
    event.y = Some(4);
    event.x = Some(12);
    event.nmi_latch = Some(1);

    let error = tracker.consume_event(event, &mut receipts).unwrap_err();

    assert!(error.contains("cursors disagreed: y=4, x=12"));
    assert!(receipts.is_empty());
    assert!(!tracker.nmi_publication_pending);
    assert_eq!(tracker.pending_nmi_update_gate, None);
    assert!(tracker.nmi_resume_targets.is_empty());
}

#[test]
fn extended_oam_pack_receipt_requires_a_held_latch_before_mutation() {
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();
    let mut event = raw("nmi", Some(0x00_860f), None, None);
    event.y = Some(4);
    event.x = Some(16);
    event.nmi_latch = Some(0);

    let error = tracker.consume_event(event, &mut receipts).unwrap_err();

    assert!(error.contains("observed an open Zelda NMI latch"));
    assert!(receipts.is_empty());
    assert!(!tracker.nmi_publication_pending);
    assert_eq!(tracker.pending_nmi_update_gate, None);
    assert!(tracker.nmi_resume_targets.is_empty());
}

#[test]
fn shared_jump_table_inside_the_symbol_gap_keeps_the_active_source_receipt() {
    let mut tracker = empty_semantic_tracker();
    tracker.sprite_main_execution = Some(SpriteMainExecutionTracker {
        current_slot: Some(0),
        last_completed_slot: Some(1),
        timers_and_oam_slot: None,
        timers_and_oam_dispatch_state: None,
        initialize_active_main_calls: 0,
        hog_spear_active_body: false,
        buzzblob_movement: None,
        trinexx_head_draw: None,
        trinexx_segment_counter: None,
        trinexx_head_draw_setup: None,
        trinexx_breath_tile_collision: None,
        handler_returned_slot: None,
        trinexx_final_phase_case0: None,
        trinexx_final_phase_tile_collision: None,
        helmasaur_hard_hat_tile_collision: None,
        trinexx_d_draw_counter: None,
        trinexx_d_draw_active: None,
        trinexx_final_phase_draw: None,
        sidenexx_neck_target: None,
        trinexx_front_part: None,
        guard_prep_parry_hitbox: None,
        guard_prep_patrol_delay: None,
        guard_prep_tile_collision_return: None,
        guard_animation_checkpoint: None,
        hog_spear_body_graphics_pending: None,
        absorbable_body_active: false,
        absorbable_horizontal_lookup: None,
        absorbable_vertical_lookup: None,
        absorbable_vertical_attribute_loaded: None,
        swamola_segment: None,
        dispatch_trampoline_return: None,
        vitreous_minions_seen: false,
        moblin_collision_started: false,
        moblin_collision_geometry: None,
        moblin_attribute_loaded: None,
        vitreous_player_damage_pending: None,
        vitreous_ai_pending: None,
        mini_moldorm_ai_pending: None,
        vitreous_damage_pending: None,
        swamola_head_prepared: false,
        swamola_head_draw_completed: None,
        swamola_head_draw: None,
        swamola_segment_draw: None,
        pengator_slide_pending: None,
        antifairy_bounce_pending: None,
        kholdstare_subtype_decremented: false,
        kholdstare_damage_pending: None,
        initialize_prep_pending: None,
        initialize_prep_move_y: None,
        guard_animation_pose_slot: None,
        guard_prep_weapon_flags_pending_slot: None,
        mini_moldorm_history: None,
        initialize_reset_properties: None,
        initialize_load_properties: None,
        fire_debirando_property_reload: false,
        fire_debirando_before_spawn_slot: None,
        fire_debirando_spawn: None,
        trinexx_death_spawn: None,
        agahnim_motion_blur_spawn: None,
        antfairy_subtype2_increment_slot: None,
        lanmola_subtype2_increment_slot: None,
        lanmola_draw_prefix: None,
        helmasaur_hard_hat_beetle_subtype2_increment_slot: None,
        timer_decrements_slot: None,
        primary_timer_decrements_slot: None,
        hit_timer_slot: None,
        main_and_aux1_timer_decrements_slot: None,
        main_timer_decrement_slot: None,
        zero_hit_timer_clear_slot: None,
        bari_before_random_slot: None,
        throwable_scenery_state_clear_slot: None,
        cucco_subtype_increments: None,
        cucco_helper_ordinal: 0,
        cucco_flee_movement: None,
        active_cucco_movement: None,
        active_cucco_x_publications: 0,
        active_cucco_y_subpixel: None,
        master_sword_light_beam_movement: None,
        boulder_movement: None,
        zora_fireball: None,
        laser_eye_draw_prologue: None,
        master_sword_light_beam_spawn: None,
        cucco_animation_slot: None,
        big_key_drop_graphics_slot: None,
        king_zora_flippers_graphics_slot: None,
        happiness_pond_rupee_graphics_slot: None,
        catfish_medallion_graphics_slot: None,
        waterfall_gt_cutscene_graphics_slot: None,
        bonk_item_graphics_slot: None,
        wish_pond_tossed_item_graphics_slot: None,
        single_small_draw_position_slot: None,
        probe_after_oam_coordinates_slot: None,
        wallmaster_reset_prefix_slot: None,
        wallmaster_reset_cleared_bytes: None,
        zazak_graphics_slot: None,
        follower_graphics: None,
    });
    let mut receipts = Vec::new();

    tracker
        .consume_event(raw("nmi", Some(0x00_8799), Some(0), None), &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainAfterSlot(1),
            ),
        ],
    );
}

#[test]
fn nmi_inside_caller_specific_item_graphics_keeps_the_current_sprite_slot_pending() {
    let mut tracker = empty_semantic_tracker();
    tracker.sprite_main_execution = Some(SpriteMainExecutionTracker {
        current_slot: Some(12),
        last_completed_slot: Some(13),
        timers_and_oam_slot: None,
        timers_and_oam_dispatch_state: None,
        initialize_active_main_calls: 0,
        hog_spear_active_body: false,
        buzzblob_movement: None,
        trinexx_head_draw: None,
        trinexx_segment_counter: None,
        trinexx_head_draw_setup: None,
        trinexx_breath_tile_collision: None,
        handler_returned_slot: None,
        trinexx_final_phase_case0: None,
        trinexx_final_phase_tile_collision: None,
        helmasaur_hard_hat_tile_collision: None,
        trinexx_d_draw_counter: None,
        trinexx_d_draw_active: None,
        trinexx_final_phase_draw: None,
        sidenexx_neck_target: None,
        trinexx_front_part: None,
        guard_prep_parry_hitbox: None,
        guard_prep_patrol_delay: None,
        guard_prep_tile_collision_return: None,
        guard_animation_checkpoint: None,
        hog_spear_body_graphics_pending: None,
        absorbable_body_active: false,
        absorbable_horizontal_lookup: None,
        absorbable_vertical_lookup: None,
        absorbable_vertical_attribute_loaded: None,
        swamola_segment: None,
        dispatch_trampoline_return: None,
        vitreous_minions_seen: false,
        moblin_collision_started: false,
        moblin_collision_geometry: None,
        moblin_attribute_loaded: None,
        vitreous_player_damage_pending: None,
        vitreous_ai_pending: None,
        mini_moldorm_ai_pending: None,
        vitreous_damage_pending: None,
        swamola_head_prepared: false,
        swamola_head_draw_completed: None,
        swamola_head_draw: None,
        swamola_segment_draw: None,
        pengator_slide_pending: None,
        antifairy_bounce_pending: None,
        kholdstare_subtype_decremented: false,
        kholdstare_damage_pending: None,
        initialize_prep_pending: None,
        initialize_prep_move_y: None,
        guard_animation_pose_slot: None,
        guard_prep_weapon_flags_pending_slot: None,
        mini_moldorm_history: None,
        initialize_reset_properties: None,
        initialize_load_properties: None,
        fire_debirando_property_reload: false,
        fire_debirando_before_spawn_slot: None,
        fire_debirando_spawn: None,
        trinexx_death_spawn: None,
        agahnim_motion_blur_spawn: None,
        antfairy_subtype2_increment_slot: None,
        lanmola_subtype2_increment_slot: None,
        lanmola_draw_prefix: None,
        helmasaur_hard_hat_beetle_subtype2_increment_slot: None,
        timer_decrements_slot: None,
        primary_timer_decrements_slot: None,
        hit_timer_slot: None,
        main_and_aux1_timer_decrements_slot: None,
        main_timer_decrement_slot: None,
        zero_hit_timer_clear_slot: None,
        bari_before_random_slot: None,
        throwable_scenery_state_clear_slot: None,
        cucco_subtype_increments: None,
        cucco_helper_ordinal: 0,
        cucco_flee_movement: None,
        active_cucco_movement: None,
        active_cucco_x_publications: 0,
        active_cucco_y_subpixel: None,
        master_sword_light_beam_movement: None,
        boulder_movement: None,
        zora_fireball: None,
        laser_eye_draw_prologue: None,
        master_sword_light_beam_spawn: None,
        cucco_animation_slot: None,
        big_key_drop_graphics_slot: None,
        king_zora_flippers_graphics_slot: None,
        happiness_pond_rupee_graphics_slot: None,
        catfish_medallion_graphics_slot: None,
        waterfall_gt_cutscene_graphics_slot: None,
        bonk_item_graphics_slot: None,
        wish_pond_tossed_item_graphics_slot: None,
        single_small_draw_position_slot: None,
        probe_after_oam_coordinates_slot: None,
        wallmaster_reset_prefix_slot: None,
        wallmaster_reset_cleared_bytes: None,
        zazak_graphics_slot: None,
        follower_graphics: None,
    });
    tracker.item_receipt_caller = Some(ItemReceiptGraphicsCaller::SpriteMain { slot: 12 });
    let mut receipts = Vec::new();

    tracker
        .consume_event(raw("nmi", Some(0x07_99f0), Some(12), None), &mut receipts)
        .unwrap();
    tracker.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);
    tracker.flush_item_receipt_progress(&mut receipts);

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainItemReceiptGraphicsStarted(12),
            ),
            OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                ItemReceiptGraphicsProgressReceipt {
                    caller: ItemReceiptGraphicsCaller::SpriteMain { slot: 12 },
                    progress: SourceCallProgress::Suspended,
                },
            ),
        ],
    );
}

#[test]
fn nmi_inside_link_oam_becomes_a_backend_neutral_receipt() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();

    tracker
        .consume_event(raw("nmi", Some(0x0d_a9d0), None, None), &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(MainLoopInterruption::LinkOam,),
        ],
    );
}

#[test]
fn link_velocity_running_test_boundary_is_before_coordinates_only_on_the_beq_path() {
    let mut event = raw("frame", Some(0x07_e29e), Some(0), None);
    event.main = Some(0x0f);
    event.sub = Some(1);
    event.a = Some(0x2f00);
    assert_eq!(
        main_loop_interruption_for_event(&event).unwrap(),
        Some(MainLoopInterruption::LinkPositionBeforeCoordinates)
    );
    event.a = Some(0x10);
    assert!(main_loop_interruption_for_event(&event).is_err());
    for pc in [0x07_e282, 0x07_e288, 0x07_e28d, 0x07_e290, 0x07_e292] {
        assert_eq!(
            main_loop_interruption_for_source_state(pc, Some(0x0f), Some(1), None),
            Some(MainLoopInterruption::LinkPositionBeforeCoordinates),
            "{pc:#x}"
        );
    }
    assert_eq!(
        main_loop_interruption_for_source_state(0x07_e294, Some(0x0f), Some(1), None),
        None
    );
}

#[test]
fn nmi_before_link_coordinate_publication_becomes_a_backend_neutral_receipt() {
    for pc in [
        0x07_e275, 0x07_e27d, 0x07_e27f, 0x07_e28d, 0x07_e292, 0x07_e2ca, 0x07_e381,
    ] {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
            host_dialogue_scroll_progress: Vec::new(),
        };
        let mut event = raw("nmi", Some(pc), None, None);
        event.main = Some(0x0f);
        event.sub = Some(1);
        let mut receipts = Vec::new();

        tracker.consume_event(event, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::LinkPositionBeforeCoordinates,
                ),
            ],
            "source PC {pc:06x}",
        );
    }
}

#[test]
fn nmi_after_spotlight_submodule_return_preserves_the_link_suffix() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut event = raw(
        "nmi",
        Some(MODULE0F_AFTER_SUBMODULE_DISPATCH_PC),
        None,
        None,
    );
    event.main = Some(0x0f);
    event.sub = Some(1);
    let mut receipts = Vec::new();

    tracker.consume_event(event, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::DungeonExitSpotlightAfterSubmodule,
            ),
        ],
    );
}

#[test]
fn recurring_spotlight_table_tail_retains_control_and_link_caller() {
    for pc in [0x00_f423, 0x00_f425, 0x00_f426] {
        assert_eq!(
            main_loop_interruption_for_source_state(pc, Some(0x0f), Some(1), Some(0)),
            Some(MainLoopInterruption::DungeonExitSpotlightTableCompleted)
        );
        assert_ne!(
            main_loop_interruption_for_source_state(pc, Some(0x0f), Some(0), Some(0)),
            Some(MainLoopInterruption::DungeonExitSpotlightTableCompleted)
        );
        assert_ne!(
            main_loop_interruption_for_source_state(pc, Some(6), Some(0), Some(0)),
            Some(MainLoopInterruption::DungeonExitSpotlightTableCompleted)
        );
    }
}

#[test]
fn host_boundary_after_actual_x_velocity_retains_the_pending_y_component() {
    for pc in [0x07_e359, 0x07_e35d, 0x07_e35f, 0x07_e361] {
        assert_eq!(
            main_loop_interruption_for_source_state(pc, Some(0x0f), Some(1), Some(0)),
            Some(MainLoopInterruption::LinkActualVelocityCompleted),
        );
        assert_eq!(
            main_loop_interruption_for_source_state(pc, Some(0x0f), Some(1), Some(1)),
            Some(MainLoopInterruption::LinkActualVelocity {
                horizontal_resolved: Some(true)
            }),
        );
    }
    assert_eq!(
        main_loop_interruption_for_source_state(0x07_e245, Some(0x0f), Some(1), Some(0)),
        Some(MainLoopInterruption::LinkPositionBeforeCoordinates),
    );
    assert_eq!(
        main_loop_interruption_for_source_state(0x07_e245, Some(9), Some(0), Some(0)),
        None,
    );
    assert_eq!(
        main_loop_interruption_for_source_state(0x07_e2de, Some(0x0f), Some(1), Some(0)),
        Some(MainLoopInterruption::LinkActualVelocity {
            horizontal_resolved: None
        }),
    );
    assert_eq!(
        main_loop_interruption_for_source_state(
            MODULE0F_LINK_VELOCITY_CALL_PC,
            Some(0x0f),
            Some(1),
            Some(0)
        ),
        Some(MainLoopInterruption::LinkPositionBeforeCoordinates),
    );
    assert_eq!(
        main_loop_interruption_for_source_state(0x07_e352, Some(0x0f), Some(1), Some(0),),
        Some(MainLoopInterruption::LinkActualVelocity {
            horizontal_resolved: Some(true)
        }),
    );
    assert_eq!(
        main_loop_interruption_for_source_state(0x07_e352, Some(0x0f), Some(1), Some(1),),
        Some(MainLoopInterruption::LinkActualVelocity {
            horizontal_resolved: Some(false)
        }),
        "the horizontal pass has not completed while X still names it",
    );
}

#[test]
fn first_dungeon_record_inspection_proves_the_completed_reset_prefix() {
    assert_eq!(
        dungeon_reset_sprites_caller_progress(&raw("frame", Some(0x09_c2a0), Some(2), None)),
        Some(DungeonResetSpritesCpuProgress::LoadBeforeOrigin)
    );
    let mut event = raw("frame", Some(0x09_c32e), Some(19), None);
    event.y = Some(3);
    assert_eq!(
        dungeon_reset_sprites_caller_progress(&event),
        Some(DungeonResetSpritesCpuProgress::LoadStarted)
    );
    event.y = Some(6);
    assert_eq!(dungeon_reset_sprites_caller_progress(&event), None);
}

#[test]
fn pending_y_subpixel_store_retains_the_completed_x_pass() {
    for pc in [0x07_e3a4, 0x07_e3af] {
        assert_eq!(
            main_loop_interruption_for_source_state(pc, Some(0x0f), Some(1), Some(0)),
            Some(MainLoopInterruption::LinkPositionAfterCoordinates { pass: 2 })
        );
    }
    assert_eq!(
        main_loop_interruption_for_source_state(0x07_e3af, Some(0x0f), Some(1), Some(2)),
        None
    );
    assert_eq!(
        main_loop_interruption_for_source_state(0x07_e3af, Some(7), Some(1), Some(0)),
        None
    );
}

#[test]
fn resumed_link_oam_context_retires_its_intermediate_drawing_checkpoint() {
    let mut receipts = vec![
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        OriginalTimingSemanticReceipt::MainLoopInterrupted(MainLoopInterruption::LinkOam),
        OriginalTimingSemanticReceipt::LinkOamStairProgress(
            zelda3::LinkOamStairProgress::EquipmentSelection,
        ),
    ];
    retire_resumed_main_loop_interruption(&mut receipts, None).unwrap();
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::LatchHeld
        )]
    );
}

#[test]
fn link_position_after_coordinates_names_the_completed_axis() {
    assert_eq!(
        main_loop_interruption_for_source_state(
            LINK_POSITION_AFTER_COORDINATES_START_PC,
            Some(0x0f),
            Some(1),
            Some(2),
        ),
        Some(MainLoopInterruption::LinkPositionAfterCoordinates { pass: 2 }),
    );
    assert_eq!(
        main_loop_interruption_for_source_state(
            LINK_POSITION_AFTER_COORDINATES_START_PC,
            Some(0x0f),
            Some(1),
            Some(1),
        ),
        None,
        "the loop's transient first DEX value is not a completed axis",
    );
    assert_eq!(
        main_loop_interruption_for_source_state(0x07e3d3, Some(0x0f), Some(1), Some(0xfffe),),
        Some(MainLoopInterruption::LinkPositionAfterCoordinates { pass: 0 }),
        "the final loop epilogue has committed Y even though X is no longer the pass",
    );
}

#[test]
fn link_position_low_coordinate_store_is_a_distinct_source_boundary() {
    assert_eq!(
        main_loop_interruption_for_source_state(0x07_e3ca, Some(0x0f), Some(1), Some(0),),
        Some(MainLoopInterruption::LinkPositionAfterCoordinateLow { pass: 0 }),
    );
    assert_eq!(
        main_loop_interruption_for_source_state(0x07_e3cd, Some(0x0f), Some(1), Some(2),),
        Some(MainLoopInterruption::LinkPositionAfterCoordinateLow { pass: 2 }),
        "the whole interval before the high-byte store is the same source boundary",
    );
}

#[test]
fn nmi_inside_spotlight_circle_build_reports_exact_c_statement_progress() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut event = raw("nmi", Some(IRIS_SPOTLIGHT_CIRCLE_VALUE_CALL_PC), None, None);
    event.a = Some(30);
    event.main = Some(0x0f);
    event.sub = Some(0);
    event.link_y = Some(1012);
    event.bg2_v = Some(786);
    event.spotlight_radius = Some(126);
    let mut receipts = Vec::new();

    tracker.consume_event(event, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                SpotlightTableBuildProgressReceipt {
                    progress: SpotlightTableBuildProgress {
                        completed_iterations: 209,
                        checkpoint: SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                            pending_circle_input: 30,
                        },
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
        ],
    );
}

#[test]
fn host_return_after_pure_circle_helper_precedes_the_first_table_publication() {
    // Route host 682798 returns at $00:F37B: ASL has doubled the upper
    // cursor, but the guarded upper-table STA at $00:F383 is still pending.
    // Re-running the pure helper is state-equivalent; advancing to the
    // table store is not.
    let mut event = frame_with_sub("return", 17_288, 0x0f, 0);
    event.pc = Some(0x00f37b);
    event.a = Some(286);
    event.x = Some(98);
    event.link_y = Some(3060);
    event.bg2_v = Some(2833);
    event.spotlight_radius = Some(126);
    event.spotlight_var4_low = Some(96);
    event.spotlight_lower_cursor = Some(335);

    assert_eq!(
        spotlight_table_build_progress(&event, None, None).unwrap(),
        Some(SpotlightTableBuildProgress {
            completed_iterations: 143,
            checkpoint: SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                pending_circle_input: 97,
            },
        }),
    );
}

#[test]
fn nmi_before_spotlight_iteration_initialization_reports_exact_c_progress() {
    let mut tracker = empty_semantic_tracker();
    let mut interrupted = raw(
        "nmi",
        Some(IRIS_SPOTLIGHT_ITERATION_VALUE_STORE_PC),
        Some(100),
        None,
    );
    interrupted.a = Some(0x00ff);
    interrupted.main = Some(0x0f);
    interrupted.sub = Some(0);
    interrupted.link_y = Some(8209);
    interrupted.bg2_v = Some(8191);
    interrupted.spotlight_radius = Some(126);
    let mut receipts = Vec::new();

    tracker.consume_event(interrupted, &mut receipts).unwrap();
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open
        )]
    );

    let mut returned = frame_with_sub("return", 49_001, 0x0f, 0);
    returned.pc = Some(NMI_HANDLER_ENTRY_PC);
    tracker
        .finish_pending_spotlight_helper_nmi(&returned, Some(20), Some(49), None, &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                SpotlightTableBuildProgressReceipt {
                    progress: SpotlightTableBuildProgress {
                        completed_iterations: 175,
                        checkpoint: SpotlightTableBuildCheckpoint::BeforeIterationInitialization,
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
        ]
    );
}

#[test]
fn nmi_inside_pure_circle_helper_rewinds_to_its_c_call_boundary() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    // The source loop loads t=126, decrements spotlight_var4 to 125, then
    // enters the pure circle helper. The authoritative frame-40977 NMI is
    // accepted at $00:f4da after the divider operands are written but
    // before a result or either HDMA-table word is published. The generic
    // WRAM write trace does not cover this direct-mapped store, so the
    // adapter reads the current source variable from WRAM at the immediate
    // NMI-entry return instead of retaining an older host's value.
    let mut interrupted = raw("nmi", Some(0x00_f4da), Some(182), None);
    interrupted.main = Some(0x0f);
    interrupted.sub = Some(0);
    interrupted.a = Some(126);
    interrupted.link_y = Some(8180);
    interrupted.bg2_v = Some(7954);
    interrupted.spotlight_radius = Some(126);
    let mut receipts = Vec::new();

    tracker.consume_event(interrupted, &mut receipts).unwrap();
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open
        )],
    );
    let mut returned = frame("return", 40977, 0x0f, 253);
    returned.pc = Some(NMI_HANDLER_ENTRY_PC);
    tracker
        .finish_pending_spotlight_helper_nmi(&returned, Some(125), None, None, &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                SpotlightTableBuildProgressReceipt {
                    progress: SpotlightTableBuildProgress {
                        completed_iterations: 113,
                        checkpoint: SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                            pending_circle_input: 126,
                        },
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
        ],
    );
}

#[test]
fn completed_spotlight_entry_supersedes_helper_interruption_progress() {
    // Pinned frame 50186 accepts NMI inside
    // IrisSpotlight_CalculateCircleValue at $00:f52d, then resumes the
    // same Module0F call through Dungeon_PrepExitWithSpotlight's
    // submodule increment before returning inside LinkOam_Main. The C
    // submodule transition proves the entire IrisSpotlight_close call has
    // returned, so replaying its earlier pure-helper checkpoint would
    // move source execution backwards.
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 3186, 0x0f, 0);
    entry.pc = Some(0x00_f52b);
    host.observe(&entry).unwrap();

    let mut interrupted = raw("nmi", Some(0x00_f52d), None, None);
    interrupted.main = Some(0x0f);
    interrupted.sub = Some(0);
    interrupted.a = Some(0x0174);
    interrupted.link_y = Some(8692);
    interrupted.bg2_v = Some(8465);
    interrupted.spotlight_radius = Some(126);
    let mut receipts = Vec::new();
    tracker
        .consume_event(interrupted.clone(), &mut receipts)
        .unwrap();
    host.observe(&interrupted).unwrap();

    let mut returned = frame_with_sub("return", 3186, 0x0f, 1);
    returned.pc = Some(0x0d_a38c);
    host.observe(&returned).unwrap();
    tracker
        .finish_pending_spotlight_helper_nmi(
            &returned,
            Some(119),
            None,
            host.spotlight_call_completion(),
            &mut receipts,
        )
        .unwrap();
    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(MainLoopInterruption::LinkOam,),
            OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned,
        ],
    );
}

#[test]
fn completed_recurring_spotlight_call_supersedes_helper_interruption_progress() {
    // Pinned frame 50192 enters this host while the recurring Module0F
    // spotlight caller is suspended at $00:f4fd, accepts NMI at $00:f500,
    // and returns at Zelda's $00:8036 main wait. The source-owned
    // nmi_boolean/main-wait transition proves Module_MainRouting and its
    // Link/OAM + NMI_PrepareSprites suffix returned, which supersedes the
    // intermediate pure-helper checkpoint.
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 3192, 0x0f, 1);
    entry.pc = Some(0x00_f4fd);
    entry.nmi_latch = Some(1);
    host.observe(&entry).unwrap();

    let mut interrupted = raw("nmi", Some(0x00_f500), None, None);
    interrupted.main = Some(0x0f);
    interrupted.sub = Some(1);
    interrupted.a = Some(110);
    interrupted.link_y = Some(8692);
    interrupted.bg2_v = Some(8465);
    interrupted.spotlight_radius = Some(112);
    let mut receipts = Vec::new();
    tracker
        .consume_event(interrupted.clone(), &mut receipts)
        .unwrap();
    host.observe(&interrupted).unwrap();

    let mut returned = frame_with_sub("return", 3192, 0x0f, 1);
    returned.pc = Some(0x00_8036);
    returned.nmi_latch = Some(0);
    host.observe(&returned).unwrap();
    tracker
        .finish_pending_spotlight_helper_nmi(
            &returned,
            Some(105),
            None,
            host.spotlight_call_completion(),
            &mut receipts,
        )
        .unwrap();
    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
        ],
    );
}

#[test]
fn terminal_recurring_spotlight_double_nmi_suppresses_the_deferred_helper_checkpoint() {
    // Pinned run4791 accepts its leading Held NMI inside the circle helper,
    // completes that handler and the whole recurring Module0F caller, then
    // accepts one trailing Open NMI before the host returns inside its
    // handler. The deferred helper checkpoint belongs to the leading Held
    // boundary and is superseded by the stronger caller-return fact.
    let mut tracker = empty_semantic_tracker();
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 4_791, 0x0f, 1);
    entry.pc = Some(0x00_f4d4);
    entry.nmi_latch = Some(1);
    host.observe(&entry).unwrap();

    let mut leading = raw("nmi", Some(0x00_f4d7), Some(0x01f0), None);
    leading.main = Some(0x0f);
    leading.sub = Some(1);
    leading.nmi_latch = Some(1);
    leading.a = Some(105);
    leading.link_y = Some(8692);
    leading.bg2_v = Some(8466);
    leading.spotlight_radius = Some(105);
    let mut receipts = Vec::new();
    tracker
        .consume_event(leading.clone(), &mut receipts)
        .unwrap();
    host.observe(&leading).unwrap();
    publish_nmi(&mut tracker, &mut receipts);

    receipts.push(OriginalTimingSemanticReceipt::MainLoopProgress(
        MainLoopProgress::CallStackContinued,
    ));
    let suffix = main_loop_common_suffix_completion();
    assert_eq!(
        host.observe(&suffix).unwrap(),
        Some(MainLoopCompletionProof::CommonSuffixCompleted),
    );
    receipts.push(OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted);

    let mut trailing = raw("nmi", Some(0x00_8034), Some(0x01ff), None);
    trailing.main = Some(0x0f);
    trailing.sub = Some(1);
    trailing.nmi_latch = Some(0);
    tracker
        .consume_event(trailing.clone(), &mut receipts)
        .unwrap();
    host.observe(&trailing).unwrap();

    let mut returned = frame_with_sub("return", 4_791, 0x0f, 1);
    returned.pc = Some(NMI_HANDLER_ENTRY_PC);
    returned.nmi_latch = Some(0);
    host.observe(&returned).unwrap();
    assert_eq!(
        host.spotlight_call_completion(),
        Some(SpotlightCallCompletion::RecurringCallerReturnedToMainWait),
    );
    tracker
        .finish_pending_spotlight_helper_nmi(
            &returned,
            Some(9),
            Some(247),
            host.spotlight_call_completion(),
            &mut receipts,
        )
        .unwrap();
    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
        ],
    );
}

#[test]
fn nonterminal_final_handler_return_preserves_acceptance_then_deferred_helper_progress() {
    let mut tracker = empty_semantic_tracker();
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 4_790, 0x0f, 1);
    entry.pc = Some(0x00_f4d4);
    entry.nmi_latch = Some(1);
    host.observe(&entry).unwrap();

    let mut interrupted = raw("nmi", Some(0x00_f4d7), Some(0x01f0), None);
    interrupted.main = Some(0x0f);
    interrupted.sub = Some(1);
    interrupted.nmi_latch = Some(1);
    interrupted.a = Some(105);
    interrupted.link_y = Some(8692);
    interrupted.bg2_v = Some(8466);
    interrupted.spotlight_radius = Some(105);
    let mut receipts = Vec::new();
    tracker
        .consume_event(interrupted.clone(), &mut receipts)
        .unwrap();
    host.observe(&interrupted).unwrap();

    let mut returned = frame_with_sub("return", 4_790, 0x0f, 1);
    returned.pc = Some(NMI_HANDLER_ENTRY_PC);
    returned.nmi_latch = Some(1);
    host.observe(&returned).unwrap();
    assert_eq!(host.spotlight_call_completion(), None);
    tracker
        .finish_pending_spotlight_helper_nmi(&returned, Some(9), Some(247), None, &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                SpotlightTableBuildProgressReceipt {
                    progress: SpotlightTableBuildProgress {
                        completed_iterations: 229,
                        checkpoint: SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                            pending_circle_input: 10,
                        },
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
        ],
    );
}

#[test]
fn recurring_spotlight_link_oam_interruption_supersedes_helper_progress() {
    // Pinned frame 56931 enters a recurring Module0F call at $00:f516,
    // accepts NMI inside the pure table helper at $00:f518, and returns
    // after the resumed caller has reached LinkOam_Main. The helper
    // checkpoint is no longer the source boundary: the stronger semantic
    // fact is that the enclosing caller reached Link OAM and remains
    // suspended there.
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 9931, 0x0f, 1);
    entry.pc = Some(0x00_f516);
    entry.nmi_latch = Some(1);
    host.observe(&entry).unwrap();

    let mut interrupted = raw("nmi", Some(0x00_f518), None, None);
    interrupted.main = Some(0x0f);
    interrupted.sub = Some(1);
    interrupted.a = Some(0xffa2);
    interrupted.x = Some(45);
    interrupted.link_y = Some(9204);
    interrupted.bg2_v = Some(8978);
    interrupted.spotlight_radius = Some(119);
    let mut receipts = Vec::new();
    tracker
        .consume_event(interrupted.clone(), &mut receipts)
        .unwrap();
    host.observe(&interrupted).unwrap();

    let mut returned = frame_with_sub("return", 9931, 0x0f, 1);
    returned.pc = Some(0x0d_a38f);
    returned.nmi_latch = Some(1);
    host.observe(&returned).unwrap();
    tracker
        .finish_pending_spotlight_helper_nmi(
            &returned,
            Some(112),
            None,
            host.spotlight_call_completion(),
            &mut receipts,
        )
        .unwrap();
    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(MainLoopInterruption::LinkOam,),
        ],
    );
}

#[test]
fn completed_overworld_spotlight_goal_supersedes_helper_interruption_progress() {
    // Pinned frame 54309 enters in recurring Module10 at $00:f4e6,
    // accepts NMI inside the pure helper at $00:f4e8, then returns at
    // Zelda's main wait after IrisSpotlight_ConfigureTable restored the
    // saved module and OpenSpotlight_Next2 selected its source submodule.
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 7309, 0x10, 1);
    entry.pc = Some(0x00_f4e6);
    entry.nmi_latch = Some(1);
    host.observe(&entry).unwrap();

    let mut interrupted = raw("nmi", Some(0x00_f4e8), None, None);
    interrupted.main = Some(0x10);
    interrupted.sub = Some(1);
    interrupted.a = Some(9);
    interrupted.link_y = Some(2359);
    interrupted.bg2_v = Some(2278);
    interrupted.spotlight_radius = Some(119);
    let mut receipts = Vec::new();
    tracker
        .consume_event(interrupted.clone(), &mut receipts)
        .unwrap();
    host.observe(&interrupted).unwrap();

    let mut returned = frame_with_sub("return", 7309, 9, 10);
    returned.pc = Some(0x00_8036);
    returned.nmi_latch = Some(0);
    host.observe(&returned).unwrap();
    tracker
        .finish_pending_spotlight_helper_nmi(
            &returned,
            Some(126),
            None,
            host.spotlight_call_completion(),
            &mut receipts,
        )
        .unwrap();
    host.finish(&mut receipts, None, true).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::OverworldSpotlightGoalCallerReturned,
        ],
    );
}

#[test]
fn helper_interruption_rejects_unproven_non_nmi_host_return() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: Some({
            let mut event = raw("nmi", Some(0x00_f52d), None, None);
            event.main = Some(0x0f);
            event.sub = Some(0);
            event
        }),
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut returned = frame_with_sub("return", 3186, 0x0f, 0);
    returned.pc = Some(0x0d_a38c);

    assert_eq!(
        tracker
            .finish_pending_spotlight_helper_nmi(&returned, Some(119), None, None, &mut Vec::new(),)
            .unwrap_err(),
        "Snes9x spotlight helper NMI did not return at the source NMI entry: $0da38c",
    );
}

#[test]
fn host_return_before_upper_spotlight_write_rewinds_only_the_pure_helper() {
    // Pinned host call 43805 returns at $00:f383. The circle helper has
    // completed, X holds 2*r4, and spotlight_var4 has already decremented,
    // but the upper and lower HDMA-table stores are both still pending.
    // Re-entering at BeforeCircleCalculation repeats only the pure helper
    // and preserves the exact C publication boundary.
    let mut returned = raw(
        "frame",
        Some(IRIS_SPOTLIGHT_UPPER_TABLE_WRITE_PC),
        Some(424),
        None,
    );
    returned.stage = Some("return".to_string());
    returned.main = Some(0x0f);
    returned.sub = Some(0);
    returned.link_y = Some(9204);
    returned.bg2_v = Some(8978);
    returned.spotlight_radius = Some(126);
    let mut receipts = Vec::new();

    publish_spotlight_host_return_progress(&returned, Some(26), None, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
            SpotlightTableBuildProgressReceipt {
                progress: SpotlightTableBuildProgress {
                    completed_iterations: 212,
                    checkpoint: SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                        pending_circle_input: 27,
                    },
                },
                boundary: OriginalTimingBoundary::HostReturn,
            },
        )],
    );
}

#[test]
fn nmi_after_spotlight_cursor_update_resumes_at_the_next_c_iteration() {
    // Pinned cold-route frame 56,927 accepts NMI at $00:f3a0.  The false
    // loop-completion branch has already advanced both source cursors, but
    // the loop-back JMP has not begun the next C iteration. The source r6
    // cursor is 286 and spotlight_var4 is 49, which independently prove
    // exactly 190 completed iterations. X is deliberately not consulted:
    // some valid geometries execute no visible table store and retain an
    // unrelated value there.
    let mut event = raw("nmi", Some(IRIS_SPOTLIGHT_NEXT_ITERATION_PC), None, None);
    event.main = Some(0x0f);
    event.sub = Some(0);
    event.link_y = Some(9204);
    event.bg2_v = Some(8978);
    event.spotlight_radius = Some(126);
    event.spotlight_var4_low = Some(49);
    event.spotlight_lower_cursor = Some(286);

    assert_eq!(
        // Volatile host-end scratch is intentionally different; the
        // event-bound source values remain authoritative.
        spotlight_table_build_progress(&event, Some(0), Some(600)).unwrap(),
        Some(SpotlightTableBuildProgress {
            completed_iterations: 190,
            checkpoint: SpotlightTableBuildCheckpoint::BeforeIterationInitialization,
        }),
    );
}

#[test]
fn host_return_before_spotlight_iteration_value_load_reports_pending_iteration() {
    // Route host 155201 returns at $00:F361 after 172 row-pair iterations.
    // The next iteration has not initialized its local value, while the
    // lower cursor and spotlight scratch independently identify the exact
    // source loop position.
    let mut event = frame_with_sub("return", 155_201, 0x10, 1);
    event.pc = Some(IRIS_SPOTLIGHT_ITERATION_VALUE_LOAD_PC);
    event.link_y = Some(3112);
    event.bg2_v = Some(3072);
    event.spotlight_radius = Some(119);
    event.spotlight_var4_low = Some(1);
    event.spotlight_lower_cursor = Some(52);
    let mut receipts = Vec::new();

    publish_spotlight_host_return_progress(&event, None, None, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
            SpotlightTableBuildProgressReceipt {
                progress: SpotlightTableBuildProgress {
                    completed_iterations: 172,
                    checkpoint: SpotlightTableBuildCheckpoint::BeforeIterationInitialization,
                },
                boundary: OriginalTimingBoundary::HostReturn,
            },
        )],
    );
}

#[test]
fn host_return_after_lower_spotlight_store_keeps_loop_test_pending() {
    // Route host 104340 returns at $00:F398. The visible upper row for
    // cursor 211 has been stored, the lower row at 265 was clipped, and
    // the source loop-completion comparison has not executed yet.
    let mut event = frame_with_sub("return", 104_340, 0x0f, 0);
    event.pc = Some(0x00f398);
    event.x = Some(422);
    event.link_y = Some(6644);
    event.bg2_v = Some(6418);
    event.spotlight_radius = Some(126);
    event.spotlight_var4_low = Some(27);
    event.spotlight_lower_cursor = Some(265);
    let mut receipts = Vec::new();

    publish_spotlight_host_return_progress(&event, None, None, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
            SpotlightTableBuildProgressReceipt {
                progress: SpotlightTableBuildProgress {
                    completed_iterations: 211,
                    checkpoint: SpotlightTableBuildCheckpoint::BeforeLoopCompletionTest {
                        upper_cursor: 211,
                        lower_cursor: 265,
                    },
                },
                boundary: OriginalTimingBoundary::HostReturn,
            },
        )],
    );
}

#[test]
fn nmi_in_spotlight_iteration_bound_branch_rewinds_before_publication() {
    // Route host 752654 accepts NMI at $00:F36B. The iteration-local
    // value has been initialized and the upper-bound comparison has run,
    // but the radius scratch decrement, pure circle call, and both HDMA
    // table stores are still pending.
    let mut event = raw("nmi", Some(0x00f36b), None, None);
    event.main = Some(0x10);
    event.sub = Some(1);
    event.link_y = Some(1335);
    event.bg2_v = Some(1254);
    event.spotlight_radius = Some(119);
    event.spotlight_var4_low = Some(4);
    event.spotlight_lower_cursor = Some(96);

    assert_eq!(
        spotlight_table_build_progress(&event, None, None).unwrap(),
        Some(SpotlightTableBuildProgress {
            completed_iterations: 128,
            checkpoint: SpotlightTableBuildCheckpoint::BeforeIterationInitialization,
        }),
    );
}

#[test]
fn nmi_before_upper_spotlight_write_derives_input_from_source_cursor() {
    // Pinned frame 54151 accepts NMI at $00:f383 after the pure circle
    // helper returned but before the upper-table store. C has not advanced
    // r4 yet, so X=$0196 means upper cursor 203. With the first iris
    // iteration at 127 and radius 112, the pending helper input is exactly
    // 112 - (203 - 127) = 36. The accumulator already holds the helper
    // result and must not be interpreted as its input.
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut interrupted = raw(
        "nmi",
        Some(IRIS_SPOTLIGHT_UPPER_TABLE_WRITE_PC),
        Some(406),
        None,
    );
    interrupted.main = Some(0x0f);
    interrupted.sub = Some(1);
    interrupted.a = Some(0xff00);
    interrupted.link_y = Some(9204);
    interrupted.bg2_v = Some(8978);
    interrupted.spotlight_radius = Some(112);
    let mut receipts = Vec::new();

    tracker.consume_event(interrupted, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                SpotlightTableBuildProgressReceipt {
                    progress: SpotlightTableBuildProgress {
                        completed_iterations: 203,
                        checkpoint: SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                            pending_circle_input: 36,
                        },
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
        ],
    );
}

#[test]
fn nmi_between_spotlight_upper_and_lower_writes_reports_exact_c_statement_progress() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut event = raw(
        "nmi",
        Some(IRIS_SPOTLIGHT_LOWER_TABLE_WRITE_PC),
        Some(78),
        None,
    );
    event.a = Some(0xff00);
    event.main = Some(0x10);
    event.sub = Some(1);
    event.link_y = Some(1044);
    event.bg2_v = Some(1024);
    event.spotlight_radius = Some(119);
    let mut receipts = Vec::new();

    tracker.consume_event(event, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                SpotlightTableBuildProgressReceipt {
                    progress: SpotlightTableBuildProgress {
                        completed_iterations: 185,
                        checkpoint: SpotlightTableBuildCheckpoint::BeforeLowerTableWrite {
                            lower_cursor: 39,
                            circle_value: 0xff00,
                        },
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
        ],
    );
}

#[test]
fn shared_circle_helper_in_dungeon_caller_does_not_cross_receipt_domains() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    // Pinned run 2327 is inside the same pure helper, but the enclosing
    // source caller is Module 7's dungeon-landing spotlight. That caller
    // has its own native continuation and must not receive the Module0F/10
    // opening/closing receipt merely because the ROM shares this helper.
    let mut interrupted = raw("nmi", Some(0x00_f4fd), Some(2), None);
    interrupted.main = Some(7);
    interrupted.sub = Some(15);
    interrupted.a = Some(119);
    interrupted.link_y = Some(8538);
    interrupted.bg2_v = Some(8464);
    interrupted.spotlight_radius = Some(119);
    let mut receipts = Vec::new();

    tracker.consume_event(interrupted, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open
        )],
    );
    assert!(tracker.pending_spotlight_helper_nmi.is_none());
}

#[test]
fn nmi_after_spotlight_projection_store_reports_copied_word_prefix() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    // At $f3be the absolute,X store at $f3bb has completed with X=$0138.
    // Bytes 0..=$0139, or 157 words, are therefore already published.
    let mut event = raw(
        "nmi",
        Some(IRIS_SPOTLIGHT_COPY_FIRST_INCREMENT_PC),
        Some(0x0138),
        None,
    );
    event.main = Some(0x0f);
    event.sub = Some(0);
    event.link_y = Some(1704);
    event.bg2_v = Some(1610);
    event.spotlight_radius = Some(126);
    let mut receipts = Vec::new();

    tracker.consume_event(event, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                SpotlightTableBuildProgressReceipt {
                    progress: SpotlightTableBuildProgress {
                        completed_iterations: 119,
                        checkpoint: SpotlightTableBuildCheckpoint::ProjectionCopy {
                            copied_words: 157,
                        },
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
        ],
    );
}

#[test]
fn nmi_at_spotlight_loop_completion_test_reports_published_iteration() {
    // The pinned failing cold host reaches $f39a after both table stores.
    // X still holds 2*r6 (222), while the source-derived upper cursor is
    // 107. The branch/cursor tail remains pending and must not cause the
    // translated owner to replay either table publication.
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut event = raw(
        "nmi",
        Some(IRIS_SPOTLIGHT_LOOP_COMPLETION_BRANCH_PC),
        Some(222),
        None,
    );
    event.main = Some(0x0f);
    event.sub = Some(0);
    event.link_y = Some(2168);
    event.bg2_v = Some(2071);
    event.spotlight_radius = Some(126);
    let mut receipts = Vec::new();

    tracker.consume_event(event, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                SpotlightTableBuildProgressReceipt {
                    progress: SpotlightTableBuildProgress {
                        completed_iterations: 113,
                        checkpoint: SpotlightTableBuildCheckpoint::BeforeLoopCompletionTest {
                            upper_cursor: 107,
                            lower_cursor: 111,
                        },
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
        ],
    );
}

#[test]
fn spotlight_beam_wait_has_completed_rows_but_no_projection_words() {
    for pc in IRIS_SPOTLIGHT_BEAM_WAIT_PCS {
        let mut event = raw("nmi", Some(pc), Some(178), None);
        event.main = Some(0x10);
        event.sub = Some(1);
        event.link_y = Some(2838);
        event.bg2_v = Some(2761);
        event.spotlight_radius = Some(119);
        assert_eq!(
            spotlight_table_build_progress(&event, None, None).unwrap(),
            Some(SpotlightTableBuildProgress {
                completed_iterations: 136,
                checkpoint: SpotlightTableBuildCheckpoint::ProjectionCopy { copied_words: 0 },
            })
        );
    }
}

#[test]
fn clipped_spotlight_helper_index_cannot_alias_an_earlier_visible_row() {
    // At $F396, X=10 is the circle helper index for input8/radius98,
    // not the byte offset of visible row5. r6=246 binds this event to232.
    let mut event = raw("nmi", Some(0x00f396), Some(10), None);
    event.main = Some(0x0f);
    event.sub = Some(1);
    event.link_y = Some(8692);
    event.bg2_v = Some(8465);
    event.spotlight_radius = Some(98);
    event.spotlight_var4_low = Some(7);
    event.spotlight_lower_cursor = Some(246);
    assert_eq!(
        spotlight_table_build_progress(&event, None, None).unwrap(),
        Some(SpotlightTableBuildProgress {
            completed_iterations: 232,
            checkpoint: SpotlightTableBuildCheckpoint::BeforeLoopCompletionTest {
                upper_cursor: 232,
                lower_cursor: 246,
            },
        })
    );
    event.spotlight_lower_cursor = None;
    assert!(spotlight_table_build_progress(&event, None, None).is_err());
}

#[test]
fn spotlight_entry_before_upper_increment_keeps_the_loop_pending() {
    // Pinned ROM host 281498 returns at INC r4 ($00:F39C), before
    // that write: center=239, upper=217, lower=261, X=2*upper.
    let mut event = raw("frame", Some(0x00f39c), Some(434), None);
    event.main = Some(0x0f);
    event.sub = Some(0);
    event.link_y = Some(3573);
    event.bg2_v = Some(3346);
    event.spotlight_radius = Some(126);
    event.spotlight_var4_low = Some(22);
    event.spotlight_lower_cursor = Some(261);
    assert_eq!(
        spotlight_table_build_progress(&event, None, None).unwrap(),
        Some(SpotlightTableBuildProgress {
            completed_iterations: 217,
            checkpoint: SpotlightTableBuildCheckpoint::BeforeLoopCompletionTest {
                upper_cursor: 217,
                lower_cursor: 261,
            },
        })
    );
}

#[test]
fn loop_completion_uses_the_upper_cursor_when_the_lower_store_is_offscreen() {
    // Pinned cold host 37,589 reaches $f39a with r4=217 and r6=259.
    // Because r6 is offscreen, the lower store is skipped and X retains
    // the preceding upper-store byte offset 2*r4=434.
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut event = raw(
        "nmi",
        Some(IRIS_SPOTLIGHT_LOOP_COMPLETION_BRANCH_PC),
        Some(434),
        None,
    );
    event.main = Some(0x0f);
    event.sub = Some(1);
    event.link_y = Some(1012);
    event.bg2_v = Some(786);
    event.spotlight_radius = Some(112);
    let mut receipts = Vec::new();

    tracker.consume_event(event, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                SpotlightTableBuildProgressReceipt {
                    progress: SpotlightTableBuildProgress {
                        completed_iterations: 217,
                        checkpoint: SpotlightTableBuildCheckpoint::BeforeLoopCompletionTest {
                            upper_cursor: 217,
                            lower_cursor: 259,
                        },
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
        ],
    );
}

#[test]
fn lower_cursor_decrement_checkpoint_preserves_the_source_statement_boundary() {
    // Cold host 54,145 returns at $f39e after INC r4 and before DEC r6.
    // The lower row is offscreen, so X retains the upper-table byte offset
    // from the just-published iteration.
    let mut event = raw(
        "frame",
        Some(IRIS_SPOTLIGHT_LOWER_CURSOR_DECREMENT_PC),
        Some(378),
        None,
    );
    event.stage = Some("return".to_string());
    event.run = Some(54_145);
    event.main = Some(0x0f);
    event.sub = Some(0);
    event.subsub = Some(0);
    event.frame_counter = Some(155);
    event.nmi_latch = Some(1);
    event.link_y = Some(9204);
    event.bg2_v = Some(8978);
    event.spotlight_radius = Some(126);
    let mut receipts = Vec::new();

    publish_spotlight_host_return_progress(&event, Some(0), None, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
            SpotlightTableBuildProgressReceipt {
                progress: SpotlightTableBuildProgress {
                    completed_iterations: 189,
                    checkpoint: SpotlightTableBuildCheckpoint::BeforeLowerCursorDecrement {
                        upper_cursor: 190,
                        lower_cursor: 287,
                    },
                },
                boundary: OriginalTimingBoundary::HostReturn,
            },
        )],
    );
}

#[test]
fn shared_spotlight_copy_in_another_caller_does_not_cross_receipt_domains() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut event = raw(
        "nmi",
        Some(IRIS_SPOTLIGHT_COPY_FIRST_INCREMENT_PC),
        Some(0x0138),
        None,
    );
    event.main = Some(7);
    event.sub = Some(15);
    let mut receipts = Vec::new();

    tracker.consume_event(event, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open
        )]
    );
}

#[test]
fn recurring_close_projection_store_remains_pending_in_its_semantic_receipt() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    // At $f3bb the long load is complete, but the absolute,X store has
    // not executed. X=$011e therefore denotes exactly 143 copied words.
    let mut event = raw(
        "nmi",
        Some(IRIS_SPOTLIGHT_COPY_STORE_PC),
        Some(0x011e),
        None,
    );
    event.main = Some(0x0f);
    event.sub = Some(1);
    event.link_y = Some(1703);
    event.bg2_v = Some(1610);
    event.spotlight_radius = Some(119);
    let mut receipts = Vec::new();

    tracker.consume_event(event, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                SpotlightTableBuildProgressReceipt {
                    progress: SpotlightTableBuildProgress {
                        completed_iterations: 120,
                        checkpoint: SpotlightTableBuildCheckpoint::ProjectionCopy {
                            copied_words: 143,
                        },
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
        ],
    );
}

#[test]
fn host_return_replaces_earlier_spotlight_progress_with_latest_c_statement() {
    // The authoritative run returns at $f3bf after the second increment.
    // X=$01b7 means bytes 0..=$01b7, or 220 words, are already copied.
    let mut returned = raw(
        "frame",
        Some(IRIS_SPOTLIGHT_COPY_SECOND_INCREMENT_PC),
        Some(0x01b7),
        None,
    );
    returned.stage = Some("return".to_string());
    returned.main = Some(0x0f);
    returned.sub = Some(1);
    returned.link_y = Some(1703);
    returned.bg2_v = Some(1610);
    returned.spotlight_radius = Some(112);
    let mut receipts = vec![
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
            SpotlightTableBuildProgressReceipt {
                progress: SpotlightTableBuildProgress {
                    completed_iterations: 120,
                    checkpoint: SpotlightTableBuildCheckpoint::ProjectionCopy { copied_words: 143 },
                },
                boundary: OriginalTimingBoundary::NmiAccepted,
            },
        ),
    ];

    publish_spotlight_host_return_progress(&returned, None, None, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                SpotlightTableBuildProgressReceipt {
                    progress: SpotlightTableBuildProgress {
                        completed_iterations: 120,
                        checkpoint: SpotlightTableBuildCheckpoint::ProjectionCopy {
                            copied_words: 220,
                        },
                    },
                    boundary: OriginalTimingBoundary::HostReturn,
                },
            ),
        ],
    );
}

#[test]
fn game_over_sprite_reset_requires_its_exact_death_caller() {
    let mut event = frame_with_sub("return", 609, 0x12, 9);
    event.pc = Some(0x09_c47f);
    event.return_address = Some(0x09_f58b);
    let mut receipts = Vec::new();
    assert!(publish_pre_dungeon_sprite_reset_progress(
        &event,
        OriginalTimingBoundary::NmiAccepted,
        false,
        &mut receipts
    )
    .unwrap());
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteResetAllProgress(
            SpriteResetAllProgressReceipt {
                progress: SpriteResetAllProgress::SpriteDisableAllCompleted,
                boundary: OriginalTimingBoundary::NmiAccepted
            },
        )]
    );
    event.return_address = Some(0x09_f58c);
    assert!(!publish_pre_dungeon_sprite_reset_progress(
        &event,
        OriginalTimingBoundary::NmiAccepted,
        false,
        &mut Vec::new()
    )
    .unwrap());
    event.return_address = Some(0x09_f58b);
    event.sub = Some(8);
    assert!(!publish_pre_dungeon_sprite_reset_progress(
        &event,
        OriginalTimingBoundary::NmiAccepted,
        false,
        &mut Vec::new()
    )
    .unwrap());
}

#[test]
fn pre_dungeon_sprite_reset_return_reports_completed_disable_semantically() {
    let mut returned = frame_with_sub("return", 39_722, 6, 0);
    returned.pc = Some(0x09_c47f);
    returned.return_address = Some(MODULE_PRE_DUNGEON_AFTER_SPRITE_RESET_PC);
    let mut receipts = Vec::new();

    assert!(publish_pre_dungeon_sprite_reset_progress(
        &returned,
        OriginalTimingBoundary::HostReturn,
        false,
        &mut receipts,
    )
    .unwrap());

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::SpriteResetAllProgress(
            SpriteResetAllProgressReceipt {
                progress: SpriteResetAllProgress::SpriteDisableAllCompleted,
                boundary: OriginalTimingBoundary::HostReturn,
            },
        )],
    );

    returned.return_address = Some(MODULE_PRE_DUNGEON_AFTER_SPRITE_RESET_PC + 4);
    let mut wrong_caller = Vec::new();
    assert!(!publish_pre_dungeon_sprite_reset_progress(
        &returned,
        OriginalTimingBoundary::HostReturn,
        false,
        &mut wrong_caller,
    )
    .unwrap());
    assert!(wrong_caller.is_empty());
}

#[test]
fn pre_dungeon_sprite_reset_caller_proof_is_independent_of_entry_module() {
    for main in [5, 6, 27] {
        let mut returned = frame_with_sub("return", 2_291, main, 0);
        returned.pc = Some(0x09_c47f);
        returned.return_address = Some(MODULE_PRE_DUNGEON_AFTER_SPRITE_RESET_PC);
        let mut receipts = Vec::new();

        assert!(publish_pre_dungeon_sprite_reset_progress(
            &returned,
            OriginalTimingBoundary::HostReturn,
            false,
            &mut receipts,
        )
        .unwrap());
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteResetAllProgress(
                SpriteResetAllProgressReceipt {
                    progress: SpriteResetAllProgress::SpriteDisableAllCompleted,
                    boundary: OriginalTimingBoundary::HostReturn,
                },
            )],
        );
    }
}

#[test]
fn pre_dungeon_nmi_inside_reset_suffix_routes_completed_disable_to_its_source_caller() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: Some(DungeonResetSpritesCpuProgress::SpritesDisabled),
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    // Pinned Snes9x frame 48,536 accepts NMI at $09:c47b inside
    // Sprite_ResetAll_noDisable with the innermost return address $02:834b,
    // the Module_PreDungeon statement immediately after Sprite_ResetAll.
    let mut event = raw("nmi", Some(0x09_c47b), Some(0x0ed7), None);
    event.return_address = Some(MODULE_PRE_DUNGEON_AFTER_SPRITE_RESET_PC);
    event.main = Some(6);
    event.sub = Some(0);
    let mut receipts = Vec::new();

    tracker.consume_event(event, &mut receipts).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::SpriteResetAllProgress(SpriteResetAllProgressReceipt {
                progress: SpriteResetAllProgress::SpriteDisableAllCompleted,
                boundary: OriginalTimingBoundary::NmiAccepted,
            },),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    );
    assert_eq!(tracker.pending_reset_progress, None);
}

#[test]
fn host_return_inside_link_oam_becomes_a_backend_neutral_receipt() {
    let mut host = HostFrameWindow::default();
    let mut entry = frame_with_sub("entry", 11_561, 7, 15);
    entry.subsub = Some(0);
    entry.frame_counter = Some(186);
    entry.pc = Some(0x00_8036);
    let mut returned = frame_with_sub("return", 11_561, 7, 15);
    returned.subsub = Some(1);
    returned.frame_counter = Some(187);
    returned.pc = Some(0x0d_a49b);

    host.observe(&entry).unwrap();
    host.observe(&main_loop_start()).unwrap();
    host.observe(&returned).unwrap();
    let mut receipts = Vec::new();
    host.finish(&mut receipts, None, false).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::IterationStarted,),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(MainLoopInterruption::LinkOam,),
        ],
    );
}

#[test]
fn nmi_acceptance_and_publication_remain_distinct_ordered_receipts() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();

    tracker
        .consume_event(raw("nmi", None, None, None), &mut receipts)
        .unwrap();
    tracker
        .consume_event(raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f3), &mut receipts)
        .unwrap();
    tracker
        .consume_event(raw_at("nmi", 0x008010, 0x01f0), &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            zero_joypad_publication(),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    );
}

#[test]
fn ordinary_and_threaded_nmi_paths_share_one_publication_receipt() {
    for &publication_pc in &NMI_HANDLER_COMPLETE_PCS {
        let mut tracker = empty_semantic_tracker();
        let mut receipts = Vec::new();

        tracker
            .consume_event(raw("nmi", None, None, None), &mut receipts)
            .unwrap();
        tracker
            .consume_event(raw_at("pc", publication_pc, 0x01f3), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                zero_joypad_publication(),
            ],
        );
        assert!(!tracker.nmi_publication_pending);
    }
}

#[test]
fn frame_boundary_context_resume_is_private_and_deduplicates_the_direct_marker() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();

    tracker
        .consume_event(raw_at("nmi", 0x008123, 0x01f8), &mut receipts)
        .unwrap();
    tracker
        .consume_event(raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f3), &mut receipts)
        .unwrap();
    tracker
        .consume_event(raw_at("frame", 0x008123, 0x01f8), &mut receipts)
        .unwrap();
    tracker
        .consume_event(raw_at("nmi-resume", 0x008123, 0x01f8), &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            zero_joypad_publication(),
        ]
    );
    assert!(tracker.nmi_resume_targets.is_empty());
    assert_eq!(tracker.synthesized_nmi_resume, None);
}

#[test]
fn new_nmi_at_restored_context_validates_the_old_resume_without_republishing_it() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();

    tracker
        .consume_event(raw_at("nmi", 0x008123, 0x01f8), &mut receipts)
        .unwrap();
    tracker
        .consume_event(raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f3), &mut receipts)
        .unwrap();
    tracker
        .consume_event(raw_at("nmi", 0x008123, 0x01f8), &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            zero_joypad_publication(),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ]
    );
    assert_eq!(tracker.nmi_resume_targets, vec![(0x008123, 0x01f8)]);
}

#[test]
fn nested_nmi_contexts_resume_in_stack_order_without_gameplay_receipts() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();

    tracker
        .consume_event(raw_at("nmi", 0x008123, 0x01f8), &mut receipts)
        .unwrap();
    tracker
        .consume_event(raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f3), &mut receipts)
        .unwrap();
    tracker
        .consume_event(raw_at("nmi", 0x0080d0, 0x01f0), &mut receipts)
        .unwrap();
    tracker
        .consume_event(raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f3), &mut receipts)
        .unwrap();
    tracker
        .consume_event(raw_at("nmi-resume", 0x0080d0, 0x01f0), &mut receipts)
        .unwrap();
    tracker
        .consume_event(raw_at("pc", 0x008123, 0x01f8), &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            zero_joypad_publication(),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            zero_joypad_publication(),
        ]
    );
    assert!(tracker.nmi_resume_targets.is_empty());
}

#[test]
fn mismatched_direct_nmi_completion_fails_closed_without_losing_target() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();

    tracker
        .consume_event(raw_at("nmi", 0x008123, 0x01f8), &mut receipts)
        .unwrap();
    tracker
        .consume_event(raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f3), &mut receipts)
        .unwrap();
    let error = tracker
        .consume_event(raw_at("nmi-resume", 0x008124, 0x01f8), &mut receipts)
        .unwrap_err();

    assert!(error.contains("did not match the active target"));
    assert_eq!(tracker.nmi_resume_targets, vec![(0x008123, 0x01f8)]);
    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            zero_joypad_publication(),
        ]
    );
}

#[test]
fn publication_without_acceptance_fails_closed() {
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();

    let error = tracker
        .consume_event(raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f3), &mut receipts)
        .unwrap_err();

    assert!(error.contains("without an accepted NMI"));
    assert!(receipts.is_empty());
    assert!(!tracker.nmi_publication_pending);
}

#[test]
fn second_acceptance_before_publication_fails_without_losing_context() {
    let mut tracker = empty_semantic_tracker();
    let mut receipts = Vec::new();
    tracker
        .consume_event(raw_at("nmi", 0x008123, 0x01f8), &mut receipts)
        .unwrap();

    let error = tracker
        .consume_event(raw_at("nmi", 0x008123, 0x01f8), &mut receipts)
        .unwrap_err();

    assert!(error.contains("before the first published"));
    assert_eq!(tracker.nmi_resume_targets, vec![(0x008123, 0x01f8)]);
    assert!(tracker.nmi_publication_pending);
    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open
        )]
    );
}

#[test]
fn live_trace_preserves_main_loop_start_between_leading_and_trailing_nmi_phases() {
    let path = env::temp_dir().join(format!(
        "zelda3-snes9x-ordered-main-loop-receipt-{}.jsonl",
        std::process::id()
    ));
    let events = [
        serde_json::json!({
            "event": "frame", "stage": "entry", "run": 7,
            "pc": 0x008123, "s": 0x01f8, "main": 15, "sub": 1,
            "subsub": 0, "frame_counter": 9, "nmi_latch": 1
        }),
        serde_json::json!({
            "event": "nmi", "run": 7, "pc": 0x008123, "s": 0x01f8,
            "main": 15, "sub": 1, "nmi_latch": 0,
            "nmi_ppu_register_operands": vec![0; 31]
        }),
        serde_json::json!({
            "event": "pc", "run": 7, "pc": NMI_HANDLER_COMPLETE_PC,
            "s": 0x01f3,
            "joypad_high": 0, "joypad_low": 0,
            "joypad_high_filtered": 0, "joypad_low_filtered": 0
        }),
        serde_json::json!({
            "event": "wram-write", "run": 7,
            "pc": ZELDA_RUN_GAME_LOOP_FRAME_COUNTER_WRITE_PC,
            "s": 0x01f8, "address": FRAME_COUNTER, "value": 10
        }),
        serde_json::json!({
            "event": "nmi-resume", "run": 7, "pc": 0x008123, "s": 0x01f8
        }),
        serde_json::json!({
            "event": "nmi", "run": 7, "pc": 0x009000, "s": 0x01f7,
            "main": 15, "sub": 1, "nmi_latch": 1,
                "nmi_ppu_register_operands": vec![0; 31]
        }),
        serde_json::json!({
            "event": "frame", "stage": "return", "run": 7,
            "pc": NMI_HANDLER_ENTRY_PC, "s": 0x01f3, "main": 15, "sub": 1,
            "subsub": 0, "frame_counter": 10, "nmi_latch": 1
        }),
    ];
    write_semantic_trace(&path, &events);
    let mut tracker = Snes9xOracleSemanticTrace {
        path: path.clone(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };

    let receipts = tracker.read_after_host_call(None, None, None).unwrap();
    fs::remove_file(path).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            zero_joypad_publication(),
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::IterationStarted,),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        ]
    );
}

#[test]
fn live_trace_publishes_exact_common_suffix_before_poly_nmi_and_across_next_host() {
    let path = env::temp_dir().join(format!(
        "zelda3-snes9x-poly-common-suffix-receipt-{}.jsonl",
        std::process::id()
    ));
    let first_host = [
        serde_json::json!({
            "event": "frame", "stage": "entry", "run": 889,
            "pc": 0x008034, "s": 0x01ff, "main": 0, "sub": 7,
            "subsub": 0, "frame_counter": 147, "nmi_latch": 0
        }),
        serde_json::json!({
            "event": "nmi", "run": 889, "pc": 0x008036, "s": 0x01ff,
            "main": 0, "sub": 7, "nmi_latch": 0,
                "nmi_ppu_register_operands": vec![0; 31]
        }),
        serde_json::json!({
            "event": "pc", "run": 889, "pc": NMI_HANDLER_COMPLETE_PC,
            "s": 0x01f2,
            "joypad_high": 0, "joypad_low": 0,
            "joypad_high_filtered": 0, "joypad_low_filtered": 0
        }),
        serde_json::json!({
            "event": "nmi-resume", "run": 889, "pc": 0x008036, "s": 0x01ff
        }),
        serde_json::json!({
            "event": "wram-write", "run": 889,
            "pc": ZELDA_RUN_GAME_LOOP_FRAME_COUNTER_WRITE_PC,
            "s": 0x01ff, "address": FRAME_COUNTER, "value": 148
        }),
        serde_json::json!({
            "event": "wram-write", "run": 889,
            "pc": ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC,
            "s": 0x01ff, "address": NMI_UPDATE_LATCH, "value": 0
        }),
        serde_json::json!({
            "event": "nmi", "run": 889, "pc": 0x09fd18, "s": 0x1f39,
            "main": 0, "sub": 7, "nmi_latch": 0,
                "nmi_ppu_register_operands": vec![0; 31]
        }),
        serde_json::json!({
            "event": "frame", "stage": "return", "run": 889,
            "pc": NMI_HANDLER_ENTRY_PC, "s": 0x1f35, "main": 0, "sub": 7,
            "subsub": 0, "frame_counter": 148, "nmi_latch": 0
        }),
    ];
    write_semantic_trace(&path, &first_host);
    let mut tracker = Snes9xOracleSemanticTrace {
        path: path.clone(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };

    let receipts = tracker.read_after_host_call(None, None, None).unwrap();
    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            zero_joypad_publication(),
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::IterationStarted,),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    );
    assert!(tracker.nmi_publication_pending);
    assert_eq!(tracker.take_host_nmi_ppu_register_operands().len(), 2);

    let second_host = [
        serde_json::json!({
            "event": "frame", "stage": "entry", "run": 890,
            "pc": NMI_HANDLER_ENTRY_PC, "s": 0x1f35, "main": 0, "sub": 7,
            "subsub": 0, "frame_counter": 148, "nmi_latch": 0
        }),
        serde_json::json!({
            "event": "pc", "run": 890, "pc": NMI_HANDLER_COMPLETE_PCS[1],
            "s": 0x1f2c,
            "joypad_high": 0, "joypad_low": 0,
            "joypad_high_filtered": 0, "joypad_low_filtered": 0
        }),
        serde_json::json!({
            "event": "wram-write", "run": 890,
            "pc": ZELDA_RUN_GAME_LOOP_FRAME_COUNTER_WRITE_PC,
            "s": 0x01ff, "address": FRAME_COUNTER, "value": 149
        }),
        serde_json::json!({
            "event": "wram-write", "run": 890,
            "pc": ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC,
            "s": 0x01ff, "address": NMI_UPDATE_LATCH, "value": 0
        }),
        serde_json::json!({
            "event": "nmi-resume", "run": 890, "pc": 0x09fd18, "s": 0x1f39
        }),
        serde_json::json!({
            "event": "frame", "stage": "return", "run": 890,
            "pc": 0x09fe63, "s": 0x1f34, "main": 0, "sub": 7,
            "subsub": 0, "frame_counter": 149, "nmi_latch": 0
        }),
    ];
    // A later host appends framed records after the existing ones.
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    for event in second_host {
        use std::io::Write as _;
        file.write_all(
            &parity::trace_format::TraceRecord::from_json(&event)
                .unwrap()
                .encode_framed(),
        )
        .unwrap();
    }
    drop(file);

    let receipts = tracker.read_after_host_call(None, None, None).unwrap();
    fs::remove_file(path).unwrap();
    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            zero_joypad_publication(),
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::IterationStarted,),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    );
    assert!(!tracker.nmi_publication_pending);
    assert!(tracker.nmi_resume_targets.is_empty());
}

#[test]
fn continued_trace_publishes_progress_and_exact_suffix_before_following_open_nmi() {
    let path = env::temp_dir().join(format!(
        "zelda3-snes9x-continued-common-suffix-receipt-{}.jsonl",
        std::process::id()
    ));
    let events = [
        serde_json::json!({
            "event": "frame", "stage": "entry", "run": 1059,
            "pc": 0x0cce3a, "s": 0x01f3, "main": 4, "sub": 1,
            "subsub": 0, "frame_counter": 1, "nmi_latch": 1
        }),
        serde_json::json!({
            "event": "nmi", "run": 1059, "pc": 0x0cce3c,
            "s": 0x01f3, "main": 4, "sub": 1, "nmi_latch": 1,
                "nmi_ppu_register_operands": vec![0; 31]
        }),
        serde_json::json!({
            "event": "pc", "run": 1059, "pc": NMI_HANDLER_COMPLETE_PC,
            "s": 0x01f2
        }),
        serde_json::json!({
            "event": "nmi-resume", "run": 1059, "pc": 0x0cce3c,
            "s": 0x01f3
        }),
        serde_json::json!({
            "event": "wram-write", "run": 1059,
            "pc": ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC,
            "s": 0x01ff, "address": NMI_UPDATE_LATCH, "value": 0
        }),
        serde_json::json!({
            "event": "nmi", "run": 1059, "pc": 0x008034,
            "s": 0x01ff, "main": 4, "sub": 2, "nmi_latch": 0,
                "nmi_ppu_register_operands": vec![0; 31]
        }),
        serde_json::json!({
            "event": "frame", "stage": "return", "run": 1059,
            "pc": NMI_HANDLER_ENTRY_PC, "s": 0x01fb, "main": 4,
            "sub": 2, "subsub": 0, "frame_counter": 1, "nmi_latch": 0
        }),
    ];
    write_semantic_trace(&path, &events);
    let mut tracker = Snes9xOracleSemanticTrace {
        path: path.clone(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: true,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };

    let receipts = tracker.read_after_host_call(None, None, None).unwrap();
    fs::remove_file(path).unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued,),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    );
    assert!(tracker.nmi_publication_pending);
    assert_eq!(tracker.pending_nmi_update_gate, Some(NmiUpdateGate::Open));
}

#[test]
fn common_suffix_receipt_requires_exact_post_write_pc_value_and_is_unique() {
    let mut host = HostFrameWindow::default();
    let mut wrong_pc = main_loop_common_suffix_completion();
    wrong_pc.pc = Some(ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC - 1);
    assert_eq!(host.observe(&wrong_pc).unwrap(), None);

    let mut wrong_value = main_loop_common_suffix_completion();
    wrong_value.value = Some(1);
    assert!(host
        .observe(&wrong_value)
        .unwrap_err()
        .contains("invalid $12 value"));
    assert!(!host.main_loop_common_suffix_completed);

    assert_eq!(
        host.observe(&main_loop_common_suffix_completion()).unwrap(),
        Some(MainLoopCompletionProof::CommonSuffixCompleted),
    );
    assert!(host
        .observe(&main_loop_common_suffix_completion())
        .unwrap_err()
        .contains("common suffix twice"));
}

#[test]
fn cached_sprite_load_and_restore_writes_become_semantic_progress() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();
    for &base in &CACHED_SPRITE_LIVE_FIELDS[..4] {
        tracker
            .consume_event(
                raw(
                    "wram-write",
                    Some(UNCACHE_SPRITE_START_PC + 0x20),
                    None,
                    Some(base + 7),
                ),
                &mut receipts,
            )
            .unwrap();
    }
    tracker
        .consume_event(raw("nmi", None, None, None), &mut receipts)
        .unwrap();
    publish_nmi(&mut tracker, &mut receipts);

    for &base in CACHED_SPRITE_LIVE_FIELDS.iter().rev().take(4) {
        tracker
            .consume_event(
                raw(
                    "wram-write",
                    Some(UNCACHE_SPRITE_RESTORE_START_PC),
                    None,
                    Some(base + 7),
                ),
                &mut receipts,
            )
            .unwrap();
    }
    tracker
        .consume_event(raw_at("nmi", 0x008010, 0x01f0), &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(
                CachedSpriteExecutionProgressReceipt {
                    progress: CachedSpriteExecutionProgress::Loading {
                        slot: 7,
                        copied_fields: 4,
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                high: 0,
                low: 0,
                high_filtered: 0,
                low_filtered: 0,
            }),
            OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(
                CachedSpriteExecutionProgressReceipt {
                    progress: CachedSpriteExecutionProgress::Restoring {
                        slot: 7,
                        live_fields: 20,
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    );
}

#[test]
fn cached_sprite_progress_at_scan_keys_return_keeps_host_return_ownership() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();
    tracker
        .consume_event(
            raw(
                "wram-write",
                Some(UNCACHE_SPRITE_RESTORE_START_PC),
                None,
                Some(CACHED_SPRITE_LIVE_FIELDS[7] + 2),
            ),
            &mut receipts,
        )
        .unwrap();

    tracker.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(
                CachedSpriteExecutionProgressReceipt {
                    progress: CachedSpriteExecutionProgress::Restoring {
                        slot: 2,
                        live_fields: 7,
                    },
                    boundary: OriginalTimingBoundary::HostReturn,
                },
            ),
        ],
    );
    assert_eq!(tracker.cached_sprite_execution, None);
}

#[test]
fn source_y_high_then_nmi_becomes_one_typed_progress_receipt() {
    let path = env::temp_dir().join("unused-snes9x-semantic-test.jsonl");
    let mut tracker = Snes9xOracleSemanticTrace {
        path,
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();
    tracker
        .consume_event(
            raw(
                "wram-write",
                Some(DUNGEON_RESET_SPRITES_CLEAR_PC),
                Some(0),
                Some(0x0dd0),
            ),
            &mut receipts,
        )
        .unwrap();
    for slot in [0, 1] {
        tracker
            .consume_event(
                raw(
                    "wram-write",
                    Some(DUNGEON_LOAD_SINGLE_SPRITE_STATE_PC),
                    Some(slot),
                    Some(SPRITE_STATE_BASE + slot),
                ),
                &mut receipts,
            )
            .unwrap();
        tracker
            .consume_event(
                raw(
                    "wram-write",
                    Some(DUNGEON_LOAD_SINGLE_SPRITE_Y_HIGH_PC),
                    Some(slot),
                    Some(SPRITE_Y_HIGH_BASE + slot),
                ),
                &mut receipts,
            )
            .unwrap();
        if slot == 0 {
            tracker
                .consume_event(
                    raw(
                        "wram-write",
                        Some(0x09c3b6),
                        Some(slot),
                        Some(0x0d10 + slot),
                    ),
                    &mut receipts,
                )
                .unwrap();
        }
    }
    tracker
        .consume_event(raw("nmi", None, None, None), &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
                DungeonResetSpritesProgressReceipt {
                    progress: DungeonResetSpritesCpuProgress::Load(DungeonLoadSpritesCpuProgress {
                        normal_load_ordinal: 1,
                        slot: 1,
                        checkpoint: DungeonSpriteLoadCheckpoint::YHigh,
                    },),
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    );
}

#[test]
fn completed_dungeon_sprite_record_survives_to_host_return() {
    let mut tracker = empty_semantic_tracker();
    tracker.normal_load_ordinal = Some(5);
    let mut receipts = Vec::new();
    let slot = 6u16;
    let writes = [
        (
            DUNGEON_LOAD_SINGLE_SPRITE_STATE_PC,
            SPRITE_STATE_BASE + slot,
        ),
        (DUNGEON_LOAD_SINGLE_SPRITE_TEMP_Y_PC, DUNGEON_LOAD_TEMP_Y),
        (
            DUNGEON_LOAD_SINGLE_SPRITE_FLOOR_PC,
            SPRITE_FLOOR_BASE + slot,
        ),
        (
            DUNGEON_LOAD_SINGLE_SPRITE_Y_LOW_PC,
            SPRITE_Y_LOW_BASE + slot,
        ),
        (
            DUNGEON_LOAD_SINGLE_SPRITE_Y_HIGH_PC,
            SPRITE_Y_HIGH_BASE + slot,
        ),
        (
            DUNGEON_LOAD_SINGLE_SPRITE_SHARED_X_PC,
            DUNGEON_LOAD_SHARED_X,
        ),
        (
            DUNGEON_LOAD_SINGLE_SPRITE_X_LOW_PC,
            SPRITE_X_LOW_BASE + slot,
        ),
        (
            DUNGEON_LOAD_SINGLE_SPRITE_X_HIGH_PC,
            SPRITE_X_HIGH_BASE + slot,
        ),
        (DUNGEON_LOAD_SINGLE_SPRITE_TYPE_PC, SPRITE_TYPE_BASE + slot),
        (
            DUNGEON_LOAD_SINGLE_SPRITE_SUBTYPE_CLEAR_PC,
            SPRITE_SUBTYPE_BASE + slot,
        ),
        (
            DUNGEON_LOAD_SINGLE_SPRITE_TEMP_SUBTYPE_PC,
            DUNGEON_LOAD_TEMP_Y,
        ),
        (
            DUNGEON_LOAD_SINGLE_SPRITE_SUBTYPE_FINAL_PC,
            SPRITE_SUBTYPE_BASE + slot,
        ),
        (
            DUNGEON_LOAD_SINGLE_SPRITE_SPAWN_INDEX_PC,
            SPRITE_N_WORD_BASE + slot,
        ),
        (
            DUNGEON_LOAD_SINGLE_SPRITE_COMPLETE_PC,
            SPRITE_DIE_ACTION_BASE + slot,
        ),
    ];
    for (pc, address) in writes {
        tracker
            .consume_event(
                raw("wram-write", Some(pc), Some(slot), Some(address)),
                &mut receipts,
            )
            .unwrap();
    }
    tracker.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
            DungeonResetSpritesProgressReceipt {
                progress: DungeonResetSpritesCpuProgress::Load(DungeonLoadSpritesCpuProgress {
                    normal_load_ordinal: 6,
                    slot: 6,
                    checkpoint: DungeonSpriteLoadCheckpoint::Complete,
                },),
                boundary: OriginalTimingBoundary::HostReturn,
            },
        )],
    );
}

#[test]
fn cache_short_branch_then_nmi_becomes_a_typed_field_receipt() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();
    for &(field, base) in &CACHE_FIELD_WRITES[..=6] {
        tracker
            .consume_event(
                raw(
                    "wram-write",
                    Some(DUNGEON_CACHE_TRANS_SPRITES_START_PC + 9),
                    Some(15),
                    Some(base + 15),
                ),
                &mut receipts,
            )
            .unwrap_or_else(|error| panic!("failed to consume {field:?}: {error}"));
    }
    tracker
        .consume_event(raw("nmi", None, None, None), &mut receipts)
        .unwrap();
    publish_nmi(&mut tracker, &mut receipts);
    tracker
        .consume_event(
            raw(
                "wram-write",
                Some(DUNGEON_CACHE_TRANS_SPRITES_START_PC + 9),
                Some(14),
                Some(0x1d0e),
            ),
            &mut receipts,
        )
        .unwrap();
    tracker
        .consume_event(raw_at("nmi", 0x008010, 0x01f0), &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
                DungeonResetSpritesProgressReceipt {
                    progress: DungeonResetSpritesCpuProgress::Cache {
                        slot: 15,
                        field: CachedSpriteCacheField::YHigh,
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                high: 0,
                low: 0,
                high_filtered: 0,
                low_filtered: 0,
            }),
            OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
                DungeonResetSpritesProgressReceipt {
                    progress: DungeonResetSpritesCpuProgress::Cache {
                        slot: 14,
                        field: CachedSpriteCacheField::StateClear,
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    );
}

#[test]
fn host_return_without_nmi_publishes_the_completed_sprite_disable_prefix() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();
    tracker
        .consume_event(
            raw(
                "wram-write",
                Some(SPRITE_DISABLE_ALL_FINAL_GARNISH_PC),
                Some(0),
                Some(GARNISH_TYPE_SLOT_ZERO),
            ),
            &mut receipts,
        )
        .unwrap();
    assert!(receipts.is_empty());

    tracker.flush_reset_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
            DungeonResetSpritesProgressReceipt {
                progress: DungeonResetSpritesCpuProgress::SpritesDisabled,
                boundary: OriginalTimingBoundary::HostReturn,
            },
        )],
    );
    let mut next_host = Vec::new();
    tracker.flush_reset_progress(&mut next_host, OriginalTimingBoundary::HostReturn);
    assert!(next_host.is_empty(), "host-return receipts are one-shot");
}

#[test]
fn leading_nmi_does_not_republish_the_same_reset_checkpoint() {
    let progress = DungeonResetSpritesCpuProgress::RoomHistorySearchStarted;
    let mut tracker = empty_semantic_tracker();
    tracker.pending_reset_progress = Some(progress);
    let mut first_host = Vec::new();
    tracker.flush_reset_progress(&mut first_host, OriginalTimingBoundary::HostReturn);
    assert_eq!(first_host.len(), 1);
    let checkpoint = tracker.checkpoint();
    let mut resumed = empty_semantic_tracker();
    resumed.restore_checkpoint(checkpoint).unwrap();
    resumed.pending_reset_progress = Some(progress);
    let mut next_host = Vec::new();
    resumed.flush_reset_progress(&mut next_host, OriginalTimingBoundary::NmiAccepted);
    assert!(next_host.is_empty());

    tracker.pending_reset_progress = Some(DungeonResetSpritesCpuProgress::LoadBeforeOrigin);
    tracker.flush_reset_progress(&mut next_host, OriginalTimingBoundary::NmiAccepted);
    assert_eq!(
        next_host,
        vec![OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
            DungeonResetSpritesProgressReceipt {
                progress: DungeonResetSpritesCpuProgress::LoadBeforeOrigin,
                boundary: OriginalTimingBoundary::NmiAccepted,
            },
        )]
    );
}

#[test]
fn sprite_disable_progress_refines_across_host_return_then_nmi() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut first_host = Vec::new();
    tracker
        .consume_event(
            raw(
                "wram-write",
                Some(DUNGEON_RESET_SPRITES_CLEAR_PC),
                Some(0),
                Some(SPRITE_STATE_BASE),
            ),
            &mut first_host,
        )
        .unwrap();
    for slot in (0..10).rev() {
        tracker
            .consume_event(
                raw(
                    "wram-write",
                    Some(DUNGEON_RESET_SPRITES_CLEAR_PC + 5),
                    Some(slot),
                    Some(ANCILLA_TYPE_BASE + slot),
                ),
                &mut first_host,
            )
            .unwrap();
    }
    tracker
        .consume_event(
            raw(
                "wram-write",
                Some(DUNGEON_RESET_SPRITES_CLEAR_PC + 0x0e),
                Some(0xff),
                Some(ANCILLA_PICKUP_FLAG),
            ),
            &mut first_host,
        )
        .unwrap();
    tracker.flush_reset_progress(&mut first_host, OriginalTimingBoundary::HostReturn);
    assert_eq!(
        first_host,
        vec![OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
            DungeonResetSpritesProgressReceipt {
                progress: DungeonResetSpritesCpuProgress::Disable(
                    DungeonSpriteDisableCpuProgress::AncillaPickupFlagCleared,
                ),
                boundary: OriginalTimingBoundary::HostReturn,
            },
        )],
    );

    let mut second_host = Vec::new();
    tracker
        .consume_event(
            raw(
                "wram-write",
                Some(DUNGEON_RESET_SPRITES_CLEAR_PC + 0x11),
                Some(0xff),
                Some(SPRITE_LIMIT_INSTANCE),
            ),
            &mut second_host,
        )
        .unwrap();
    tracker
        .consume_event(raw("nmi", None, None, None), &mut second_host)
        .unwrap();
    assert_eq!(
        second_host,
        vec![
            OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
                DungeonResetSpritesProgressReceipt {
                    progress: DungeonResetSpritesCpuProgress::Disable(
                        DungeonSpriteDisableCpuProgress::SpriteLimitInstanceCleared,
                    ),
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    );
}

#[test]
fn pinned_route_receipt_decodes_to_normal_load_one_slot_one_y_high() {
    let mut trace = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();
    for line in include_str!(
        "../../../external/snes9x-libretro/fixtures/zelda3-dungeon-reset-sprites-yhigh-nmi.jsonl"
    )
    .lines()
    {
        let mut event: RawTraceEvent = serde_json::from_str(line).unwrap();
        if event.event == "nmi" && event.s.is_none() {
            // This older reduced fixture ends at acceptance and predates
            // preservation of the already-present source stack and update-
            // gate fields. A later full trace of this same route boundary
            // independently preserves the omitted source values: S=$01f2
            // and Zelda's `$12` latch held. This fixture cannot exercise
            // completion ownership; restore only those corroborated fields
            // to drive its terminal acceptance through the stricter adapter.
            event.s = Some(0x01f2);
            event.nmi_latch = Some(1);
            event.nmi_ppu_register_operands = Some([0; 31]);
        }
        trace.consume_event(event, &mut receipts).unwrap();
    }

    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
                DungeonResetSpritesProgressReceipt {
                    progress: DungeonResetSpritesCpuProgress::Load(DungeonLoadSpritesCpuProgress {
                        normal_load_ordinal: 1,
                        slot: 1,
                        checkpoint: DungeonSpriteLoadCheckpoint::YHigh,
                    },),
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        ],
    );
}

#[test]
fn later_source_write_refines_the_y_high_candidate() {
    let path = env::temp_dir().join("unused-snes9x-semantic-test.jsonl");
    let mut tracker = Snes9xOracleSemanticTrace {
        path,
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: Some(0),
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();
    tracker
        .consume_event(
            raw(
                "wram-write",
                Some(DUNGEON_LOAD_SINGLE_SPRITE_Y_HIGH_PC),
                Some(0),
                Some(SPRITE_Y_HIGH_BASE),
            ),
            &mut receipts,
        )
        .unwrap();
    tracker
        .consume_event(
            raw("wram-write", Some(0x09c3b6), Some(0), Some(0x0d10)),
            &mut receipts,
        )
        .unwrap();
    tracker
        .consume_event(raw("nmi", None, None, None), &mut receipts)
        .unwrap();
    assert_eq!(
        receipts,
        vec![
            OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
                DungeonResetSpritesProgressReceipt {
                    progress: DungeonResetSpritesCpuProgress::Load(DungeonLoadSpritesCpuProgress {
                        normal_load_ordinal: 0,
                        slot: 0,
                        checkpoint: DungeonSpriteLoadCheckpoint::XLow,
                    },),
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ]
    );
}

#[test]
fn later_source_write_invalidates_the_sprite_disable_candidate() {
    let mut tracker = Snes9xOracleSemanticTrace {
        path: PathBuf::new(),
        offset: 0,
        cache_write_progress: None,
        normal_load_ordinal: None,
        pending_reset_progress: None,
        last_host_return_reset_progress: None,
        cached_sprite_execution: None,
        overworld_presence_published: false,
        overworld_sprite_activation: None,
        overworld_load_overlays_sprite_reload_active: false,
        overworld_sprite_reload_reset_published: false,
        rescued_maiden_initialization: None,
        pending_spotlight_helper_nmi: None,
        pending_spotlight_helper_nmi_acceptance_index: None,
        seed_warmup_active: false,
        item_receipt_caller: None,
        sprite_main_execution: None,
        zelda_run_game_loop_call_active: false,
        nmi_publication_pending: false,
        pending_nmi_update_gate: None,
        nmi_resume_targets: Vec::new(),
        synthesized_nmi_resume: None,
        host_nmi_ppu_register_operands: Vec::new(),
        host_dialogue_scroll_progress: Vec::new(),
    };
    let mut receipts = Vec::new();
    tracker
        .consume_event(
            raw(
                "wram-write",
                Some(SPRITE_DISABLE_ALL_FINAL_GARNISH_PC),
                Some(0),
                Some(GARNISH_TYPE_SLOT_ZERO),
            ),
            &mut receipts,
        )
        .unwrap();
    // The pinned route immediately continues into post-disable
    // bookkeeping at $09:C12C before loading sprites and consuming RNG.
    tracker
        .consume_event(
            raw("wram-write", Some(0x09c12c), Some(0xff), Some(0x0fba)),
            &mut receipts,
        )
        .unwrap();
    tracker
        .consume_event(raw("nmi", None, None, None), &mut receipts)
        .unwrap();

    assert_eq!(
        receipts,
        vec![OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open
        )]
    );
}

#[test]
fn csv_extension_is_deduplicated_and_preserves_existing_domains() {
    assert_eq!(
        append_csv(Some("frame,wram"), &["nmi", "wram"]),
        "frame,wram,nmi"
    );
}
