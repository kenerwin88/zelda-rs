//! ZeldaState runtime tests — spotlight.
//! Split mechanically from the hub file; bodies unchanged.

use super::*;

#[test]
fn interrupted_spotlight_entry_keeps_oam_from_before_the_held_nmi() {
    // Original run 11444 is interrupted at $F38F, V=225/C=16 with $12=1.
    // $805D returns at V=29/C=370 in the following field; only the NMI
    // ending that field can upload its new shadow. The original OAM keeps
    // entry 13 at (64,135) and entry 102 at (116,93), matching the preceding
    // scanout, rather than the following DMA's (63,135) and (116,92).
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_main_module(0x0f);
    state.set_submodule(0);
    state.follower_link_state_mut().set_position(2040, 1784);
    state.set_bg2_v_copy2(1690);
    state.set_bg2_h_copy2(1924);
    state.set_spotlight_window_state(0);
    state.set_spotlight_window_radius(126);
    let table = state.begin_iris_spotlight_configure_table(113);
    state.complete_dungeon_exit_spotlight_entry_returned(
        table,
        SpotlightIteration::closing(SpotlightIterationPhase::CloseEntryBeforeTablePublication),
    );
    let mut resident = state.ppu.oam.clone();
    resident[13 * 2] = 0x8740;
    resident[102 * 2] = 0x5d74;
    let mut following_dma = resident.clone();
    following_dma[13 * 2] = 0x873f;
    following_dma[102 * 2] = 0x5c74;
    state.last_presented_oam = Some(resident.clone());
    state.ppu.oam.clone_from(&following_dma);
    state.capture_display_snapshot_with_override(Some(
        DisplaySnapshotPublication::PublishCaptured,
    ));
    let snapshot = state.display_snapshot.as_ref().unwrap().clone();
    let plan = DisplayPublicationPlan::resolve(
        &snapshot,
        DisplayPublicationSignals {
            dungeon_exit_crosses_nmi_boundary: true,
            ..DisplayPublicationSignals::default()
        },
    );
    assert_eq!(
        plan.oam_scanout_source,
        OamScanoutSource::RetainPreviousPresented,
    );
    let cpu_ram = state.ram.clone();
    state.compose_display_oam(&snapshot, &plan);
    assert_eq!(state.ppu.oam, resident);
    assert_eq!(state.ram, cpu_ram, "scanout must not rewind CPU work");
    assert_eq!(snapshot.ppu.oam, following_dma, "the next DMA remains intact");
    assert!(
        state.next_display_obj_scanout_generation.is_none(),
        "publication is consumed once",
    );
}

#[test]
fn unfinished_spotlight_rows_do_not_replace_the_published_hardware_table() {
    // Original runs 4784-4786 display the completed radius-126 table while
    // the next radius-119 table is being built. $F383/$F392 only author the
    // work buffer; $F3B7-$F3C3 is the later hardware-table copy.
    for authoritative_scanout in [false, true] {
        let mut state = ZeldaState::new();
        state.set_rom_startup_timing(true);
        state.set_main_module(0x0f);
        state.set_submodule(1);
        state.follower_link_state_mut().set_y(226);
        state.follower_link_state_mut().set_x(120);
        state.set_bg2_v_copy2(0);
        state.set_bg2_h_copy2(0);
        state.IrisSpotlight_close();
        let published = state.hdma_dynamic_table_bytes();
        // Source window-1 rows at center (128,238), captured from Snes9x.
        assert_eq!(&published[226..232], &[255, 0, 84, 172, 74, 182]);
        let table_build = state.begin_iris_spotlight_configure_table(218);
        let working = state.hdma_dynamic_table_bytes();
        assert_ne!(working, published);
        state.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                table_build,
                projection_completed: false,
                iteration: SpotlightIteration::closing(SpotlightIterationPhase::WholeTable),
            },
            1,
        );
        // An instruction-timed scanout, when present, must retain priority
        // over the whole-table fallback even during an unfinished build.
        let exact_words = [0xa040; SPOTLIGHT_VISIBLE_SCANLINES];
        if authoritative_scanout {
            state.next_display_spotlight_scanout = Some(
                LiveSpotlightScanout::capture(&state)
                    .with_authoritative_rom_hdma_words(&exact_words),
            );
        }
        state.capture_display_snapshot_with_override(Some(DisplaySnapshotPublication::PublishCaptured));
        let displayed = state.display_snapshot.as_ref().unwrap().effective_spotlight_hdma_tables();
        let expected = if authoritative_scanout {
            exact_words.iter().flat_map(|word| word.to_le_bytes()).collect::<Vec<_>>()
        } else {
            published[..448].to_vec()
        };
        assert_eq!(&displayed[0][..448], &expected);
        assert_eq!(state.hdma_dynamic_table_bytes(), working, "presentation must not undo CPU work");
        assert_eq!(
            &state.ram[RESERVED_HDMA_TABLE..RESERVED_HDMA_TABLE + 448],
            &published[..448],
        );
    }
}

#[test]
fn retained_spotlight_snapshot_consumes_the_current_field_hdma_receipt() {
    // Original run 37590 retains the earlier OAM/VRAM generation, but its
    // channel-7 reads still belong to that field: the circle begins at row
    // 121, not the following field's row 128. Retaining the snapshot used to
    // discard these measured reads and project the completed next table.
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.ppu.oam[0] = 0x1234;
    state.ppu.vram[0] = 0x5678;
    state.capture_display_snapshot_with_override(Some(DisplaySnapshotPublication::PublishCaptured));
    state.display_snapshot.as_mut().unwrap().hdma_table_generation =
        DisplayHdmaTableGeneration::SpotlightPublishedAheadOfSnapshot {
            active_table: vec![0; ZeldaState::HDMA_DYNAMIC_TABLE_LEN],
        };
    state.ppu.oam[0] = 0xabcd;
    state.ppu.vram[0] = 0xef01;
    let mut active_words = [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES];
    active_words[121] = 0xa858;
    let mut following_words = [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES];
    following_words[128] = 0xa65a;
    state.next_display_spotlight_scanout = Some(
        LiveSpotlightScanout::capture(&state).with_authoritative_rom_hdma_words(&active_words),
    );
    state.spotlight_scanout_after_active_field = Some(
        LiveSpotlightScanout::capture(&state).with_authoritative_rom_hdma_words(&following_words),
    );
    let cpu_ram = state.ram.clone();
    state.capture_display_snapshot_with_override(Some(DisplaySnapshotPublication::RetainPublished));
    let snapshot = state.display_snapshot.as_ref().unwrap();
    assert_eq!(snapshot.ppu.oam[0], 0x1234);
    assert_eq!(snapshot.ppu.vram[0], 0x5678);
    let active_bytes: Vec<_> = active_words.iter().flat_map(|word| word.to_le_bytes()).collect();
    assert_eq!(&snapshot.effective_spotlight_hdma_tables()[0][..448], active_bytes);
    assert_eq!(state.ram, cpu_ram);
    assert!(!snapshot.accepts_nmi_dma_receipts);
    state.capture_display_snapshot_with_override(Some(DisplaySnapshotPublication::PublishCaptured));
    let following_bytes: Vec<_> = following_words.iter().flat_map(|word| word.to_le_bytes()).collect();
    assert_eq!(
        &state.display_snapshot.as_ref().unwrap().effective_spotlight_hdma_tables()[0][..448],
        following_bytes,
    );
    assert!(state.next_display_spotlight_scanout.is_none());
}

#[test]
fn spotlight_cpu_checkpoints_preserve_the_source_frame_counter_phase() {
    // Cold Snes9x runs 4782 and 11443 enter $02:9982 with $1A=176 and
    // 168 respectively, after INC $1A at $00:8051. Module10's checkpoint
    // starts at that INC instead. Include wraparound to distinguish replay
    // of the instruction from a saturating adjustment of the host counter.
    let mut state = ZeldaState::new();
    for counter in [176, 168, 0] {
        state.set_frame_counter(counter);
        let original = state.ram.clone();
        for (checkpoint, expected_counter) in [
            (DUNGEON_EXIT_SPOTLIGHT_CPU_CHECKPOINT, counter),
            (OVERWORLD_SPOTLIGHT_CPU_CHECKPOINT, counter.wrapping_sub(1)),
        ] {
            let seeded = spotlight_cpu_timing_ram(&state, checkpoint);
            assert_eq!(seeded[0x1a], expected_counter);
            assert_eq!(state.ram, original, "shadow seeding must not mutate gameplay");
            // The source buffers are 224 visible words at $7E:1B00 and
            // $7F:7000. No other caller state belongs to the relocation.
            for address in 0..original.len() {
                if address == 0x1a
                    || (0x1b00..0x1cc0).contains(&address)
                    || (0x17000..0x171c0).contains(&address)
                {
                    continue;
                }
                assert_eq!(seeded[address], original[address], "unowned ${address:05x}");
            }
        }
    }
}

#[test]
#[ignore = "executes the local pinned Zelda ROM to verify the spotlight loop checkpoint"]
fn spotlight_cpu_row_checkpoint_counts_original_rom_pairs() {
    let mut state = ZeldaState::new();
    let rom_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../saves/zelda3.sfc");
    state.set_rom(&std::fs::read(rom_path).expect("read the local pinned Zelda ROM"));
    state.follower_link_state_mut().set_y(94);
    state.set_bg2_v_copy2(0);
    state.set_spotlight_window_radius(126);
    let checkpoint = RomCpuCheckpoint {
        entry_pc: 0x00_f312,
        stop_pc: 0x00_f3a3,
        ..DUNGEON_EXIT_SPOTLIGHT_CPU_CHECKPOINT
    };
    let mut run = RomCpuTimingRun::new(
        &state.rom,
        &state.ram,
        &state.sram,
        &state.ppu,
        &state.dma,
        state.zelda_audio_apu_output_ports(),
        checkpoint,
    )
    .expect("start original IrisSpotlight_ConfigureTable");
    let mut row_pairs = 0;
    for _ in 0..100_000 {
        if run.is_complete() {
            break;
        }
        if run.pc() == IRIS_SPOTLIGHT_ROW_PAIR_COMPLETED_PC {
            row_pairs += 1;
        }
        run.step();
    }
    // Source $F312-$F3A0: center=94+12=106; lower=max(212,224)=224,
    // upper=212-224=-12. The inclusive loop visits -12..106: 119 pairs.
    // Every lower cursor is inside center+radius=232, decrementing $067A
    // from 126 to 7. Stop before the hardware V-counter wait and copy.
    assert!(run.is_complete(), "original table loop must reach its V-counter wait");
    assert_eq!(row_pairs, 119, "count executable row-pair boundaries, not operands");
    assert_eq!((run.ram_byte(0x04), run.ram_byte(0x05)), (106, 0));
    assert_eq!((run.ram_byte(0x06), run.ram_byte(0x07)), (106, 0));
    assert_eq!((run.ram_byte(0x067a), run.ram_byte(0x067b)), (7, 0));
}

#[test]
fn desert_prayer_iris_checkpoint_preserves_primary_write_and_resumes_full_loop() {
    fn configure(state: &mut ZeldaState) {
        state.set_main_module(0x0e);
        state.set_submodule(5);
        state.set_subsubmodule(2);
        state.follower_link_state_mut().set_x(299);
        state.follower_link_state_mut().set_y(3472);
        state.set_bg2_h_copy2(173);
        state.set_bg2_v_copy2(3374);
    }

    let mut resumed = ZeldaState::new();
    configure(&mut resumed);
    resumed.CleanUpAndPrepDesertPrayerHDMA();
    resumed.set_spotlight_window_radius_byte(0x26);
    resumed.set_spotlight_window_state_byte(0);
    resumed.begin_desert_prayer_iris(
        crate::DesertPrayerIrisProgress::AfterPrimaryTableWrite {
            table_word: 87,
            y_buffer: 22,
        },
        DesertPrayerIrisCaller::InitializeCase2,
    );

    assert_eq!(
        resumed
            .game_state
            .display
            .spotlight_hdma
            .window_y_buffer_byte(),
        22,
    );
    assert_ne!(resumed.spotlight_hdma_table_dynamic_entry(87), 0);
    assert_eq!(resumed.spotlight_hdma_table_dynamic_entry(130), 0);

    resumed.complete_desert_prayer_iris(DesertPrayerIrisCaller::InitializeCase2);

    let mut uninterrupted = ZeldaState::new();
    configure(&mut uninterrupted);
    uninterrupted.DesertPrayer_InitializeIrisHDMA();
    for row in 0..240 {
        assert_eq!(
            resumed.spotlight_hdma_table_dynamic_entry(row),
            uninterrupted.spotlight_hdma_table_dynamic_entry(row),
            "Desert Prayer iris row {row} changed across continuation",
        );
    }
    assert_eq!(resumed.game_state.frame.subsubmodule, 3);
}

#[test]
fn desert_prayer_iris_before_primary_checkpoint_preserves_the_unpublished_word() {
    fn configure(state: &mut ZeldaState) {
        state.set_main_module(0x0e);
        state.set_submodule(5);
        state.set_subsubmodule(3);
        state.follower_link_state_mut().set_x(299);
        state.follower_link_state_mut().set_y(3472);
        state.set_bg2_h_copy2(173);
        state.set_bg2_v_copy2(3374);
        state.set_spotlight_window_radius_byte(0x26);
        state.set_spotlight_window_state_byte(0);
    }

    let mut resumed = ZeldaState::new();
    configure(&mut resumed);
    resumed.begin_desert_prayer_iris(
        crate::DesertPrayerIrisProgress::BeforePrimaryTableWrite {
            table_word: 222,
            y_buffer: 1,
        },
        DesertPrayerIrisCaller::PaletteFilterCase3,
    );
    assert_eq!(resumed.spotlight_hdma_table_dynamic_entry(222), 0);

    resumed.complete_desert_prayer_iris(DesertPrayerIrisCaller::PaletteFilterCase3);
    let mut uninterrupted = ZeldaState::new();
    configure(&mut uninterrupted);
    uninterrupted.DesertPrayer_BuildIrisHDMATable();
    for row in 0..240 {
        assert_eq!(
            resumed.spotlight_hdma_table_dynamic_entry(row),
            uninterrupted.spotlight_hdma_table_dynamic_entry(row),
            "Desert Prayer iris row {row} changed across a pre-primary continuation",
        );
    }
}

#[test]
fn live_spotlight_entry_return_receipt_prevents_early_native_completion() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(15);
    state.set_submodule(0);
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonExitSpotlightEntry {
            table_build: SpotlightTableBuildContinuation::default(),
            iteration: SpotlightIteration::closing(
                SpotlightIterationPhase::CloseEntryBeforeTablePublication,
            ),
        },
        1,
    );

    assert_eq!(
        state
            .game_execution_scheduler
            .advance_work_one_nmi_slice_with_authoritative_completion(false),
        Some(GameWorkStep::Waiting),
    );
    assert_eq!(state.game_state.frame.submodule, 0);

    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned],
    ));
    let returned = state.take_or_defer_original_timing_dungeon_exit_spotlight_entry_returned(true);
    assert!(matches!(
        state
            .game_execution_scheduler
            .advance_work_one_nmi_slice_with_authoritative_completion(returned),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishDungeonExitSpotlightEntry { .. }
        ))
    ));
    assert_eq!(state.game_state.frame.submodule, 0);
    assert!(state.game_execution_scheduler.is_idle());
}

#[test]
fn live_spotlight_entry_return_waits_for_its_native_owner_then_completes_once() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(15);
    state.set_submodule(0);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned],
    ));
    // The pinned call has returned, but the translated call has not reached
    // the point where it creates its entry continuation. Preserve the
    // semantic completion without inventing native work.
    assert!(!state.take_or_defer_original_timing_dungeon_exit_spotlight_entry_returned(false));
    assert!(state.game_execution_scheduler.is_idle());
    state.original_timing_semantic_receipts = None;

    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonExitSpotlightEntry {
            table_build: SpotlightTableBuildContinuation::default(),
            iteration: SpotlightIteration::closing(
                SpotlightIterationPhase::CloseEntryBeforeTablePublication,
            ),
        },
        1,
    );
    let returned = state.take_or_defer_original_timing_dungeon_exit_spotlight_entry_returned(true);
    assert!(returned);
    assert!(matches!(
        state
            .game_execution_scheduler
            .advance_work_one_nmi_slice_with_authoritative_completion(returned),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishDungeonExitSpotlightEntry { .. }
        ))
    ));
    assert!(
        !state.take_or_defer_original_timing_dungeon_exit_spotlight_entry_returned(false),
        "the bound source return must be consumed exactly once",
    );
}

#[test]
fn run4782_fresh_spotlight_claim_is_wire_ordered_after_its_held_acceptance() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_main_module(0x0f);
    state.set_submodule(0);
    state.set_subsubmodule(0);
    state.set_saved_module_for_menu(7);
    state.set_indoor_flag(0);
    state.follower_link_state_mut().set_x(2424);
    state.follower_link_state_mut().set_y(8692);
    state.set_bg2_h_copy2(2304);
    state.set_bg2_v_copy2(8466);
    state.set_spotlight_window_radius(126);
    state.set_spotlight_window_state(2);
    state.set_frame_counter(175);
    state.set_animated_tile_data_source_address(1);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    let OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(spotlight_claim) =
        run4782_spotlight_progress_receipt()
    else {
        unreachable!();
    };
    let mut expected_table_state = state.clone();
    let expected_table_build = expected_table_state
        .begin_iris_spotlight_configure_table_at_progress(spotlight_claim.progress);
    let receipts = OriginalTimingHostReceipts::new(4_782, 0, run4782_spotlight_semantic());
    let wire_receipts: OriginalTimingHostReceipts = bincode::deserialize(
        &bincode::serialize(&receipts).expect("serialize run4782 source receipts"),
    )
    .expect("deserialize run4782 source receipts");
    assert_eq!(
        wire_receipts.semantic(),
        run4782_spotlight_semantic().as_slice(),
        "the source ledger must preserve A(Held) before its deferred spotlight claim",
    );
    state
        .install_original_timing_host_receipts(wire_receipts)
        .unwrap();
    let epoch_before = state.display_snapshot_epoch;

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.frame_counter, 176);
    assert_eq!(state.display_snapshot_epoch, epoch_before + 2);
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    let Some(GameWorkContinuation::FinishDungeonExitSpotlightEntry {
        table_build,
        iteration,
    }) = state.game_execution_scheduler.current_work()
    else {
        panic!("run4782 lost its exact suspended spotlight-table owner");
    };
    assert_eq!(table_build, expected_table_build);
    assert_eq!(iteration.direction, SpotlightDirection::Closing);
    assert_eq!(
        iteration.phase,
        SpotlightIterationPhase::CloseEntryBeforeTablePublication,
    );
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert!(!state.main_loop_sprite_preparation_completed);
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
    assert_eq!(
        state.original_timing_sprite_main_return_claims_remaining,
        None,
    );
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn run4784_terminal_spotlight_caller_completes_held_handler_and_ordinary_suffix_once() {
    let mut state = run4784_terminal_spotlight_state();
    let epoch_before = state.display_snapshot_epoch;
    let frame_counter_before = state.game_state.frame.frame_counter;

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(state.main_loop_sprite_preparation_completed);
    assert_eq!(state.game_state.frame.frame_counter, frame_counter_before);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert_eq!(
        state.original_timing_sprite_main_return_claims_remaining,
        None,
    );
    assert!(state.original_timing_semantic_receipts.is_none());
    assert_eq!(state.display_snapshot_epoch, epoch_before + 1);
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| !snapshot.accepts_nmi_dma_receipts));
    assert!(state.display_snapshot.as_ref().is_some_and(|snapshot| {
        !matches!(
            snapshot.hdma_table_generation,
            DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout { .. }
        )
    }));
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(ObjScanoutGenerations::coherent(
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
        )),
    );
}

#[test]
fn run4787_terminal_recurring_spotlight_projects_once_then_completes_ordinary_suffix() {
    let mut state = run4787_terminal_recurring_spotlight_state();
    let epoch_before = state.display_snapshot_epoch;
    let frame_counter_before = state.game_state.frame.frame_counter;
    let radius_before = state.game_state.display.spotlight_hdma.window_radius();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(state.main_loop_sprite_preparation_completed);
    assert_eq!(state.game_state.frame.frame_counter, frame_counter_before);
    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        radius_before,
        "the terminal return must not replay the recurring table body",
    );
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert_eq!(
        state.original_timing_sprite_main_return_claims_remaining,
        None,
    );
    assert!(state.original_timing_semantic_receipts.is_none());
    assert_eq!(state.display_snapshot_epoch, epoch_before + 1);
    let snapshot = state
        .display_snapshot
        .as_ref()
        .expect("run4787 lost its one Held-handler capture");
    assert!(!snapshot.accepts_nmi_dma_receipts);
    assert!(matches!(
        snapshot.hdma_table_generation,
        DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout {
            live_tail_start: 224,
            ..
        },
    ));
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(ObjScanoutGenerations::coherent(
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
        )),
    );
}

#[test]
fn terminal_spotlight_projection_is_derived_for_every_closing_phase() {
    let cases = [
        (
            SpotlightIterationPhase::CloseEntryBeforeTablePublication,
            false,
            false,
        ),
        (
            SpotlightIterationPhase::CloseEntryAfterTablePublication,
            false,
            false,
        ),
        (SpotlightIterationPhase::WholeTable, true, false),
        (
            SpotlightIterationPhase::WholeTableAfterTablePublication,
            true,
            false,
        ),
        (SpotlightIterationPhase::MixedTailAfterReturn, true, true),
    ];

    for (phase, projects, uses_published_prefix) in cases {
        let iteration = SpotlightIteration::closing(phase);
        assert_eq!(
            iteration.projects_following_table_tail_on_completion(),
            projects,
        );
        assert_eq!(
            iteration.projection_uses_published_prefix(),
            uses_published_prefix,
        );
        let mut state = terminal_recurring_spotlight_state(iteration);
        state.set_spotlight_hdma_table_dynamic_entry(0, 0x1111);
        state.next_display_spotlight_scanout = Some(LiveSpotlightScanout::capture(&state));
        state.set_spotlight_hdma_table_dynamic_entry(0, 0x2222);
        let epoch_before = state.display_snapshot_epoch;

        state.run_frame_internal(0, crate::RUN_MAIN);

        assert!(state.game_execution_scheduler.is_idle(), "{phase:?}");
        assert!(state.pending_main_loop_common_suffix.is_none(), "{phase:?}");
        assert!(state.main_loop_sprite_preparation_completed, "{phase:?}");
        assert_eq!(state.display_snapshot_epoch, epoch_before + 1, "{phase:?}");
        let snapshot = state
            .display_snapshot
            .as_ref()
            .expect("terminal spotlight lost its Held-handler capture");
        match (&snapshot.hdma_table_generation, projects) {
            (
                DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout {
                    before_projection,
                    live_tail_start: 224,
                    ..
                },
                true,
            ) => {
                let projected_prefix =
                    u16::from_le_bytes(before_projection[0][0..2].try_into().unwrap());
                assert_eq!(
                    projected_prefix,
                    if uses_published_prefix {
                        0x1111
                    } else {
                        0x2222
                    },
                    "{phase:?}",
                );
            }
            (DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout { .. }, false) => {
                panic!("{phase:?} projected an unsupported following tail")
            }
            (_, true) => panic!("{phase:?} lost its typed following-tail projection"),
            (_, false) => {}
        }
        assert!(
            state.original_timing_semantic_receipts.is_none(),
            "{phase:?}"
        );
    }
}

#[test]
fn terminal_spotlight_caller_preflight_is_failure_atomic() {
    fn assert_rejected(label: &str, mut state: ZeldaState) {
        state.original_timing_host_dispatch_active = true;
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        let receipts_before = state.original_timing_semantic_receipts.clone();
        let scheduler_before = state.game_execution_scheduler;
        let suffix_before = state.pending_main_loop_common_suffix;
        let ram_before = state.ram.clone();
        let ppu_before = (
            state.ppu.vram.clone(),
            state.ppu.oam.clone(),
            state.ppu.cgram.clone(),
        );
        let snapshot_before = state.display_snapshot.as_ref().map(|snapshot| {
            (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.vram.clone(),
            )
        });
        let epoch_before = state.display_snapshot_epoch;
        let publication_before = (
            state.original_timing_nmi_publication_pending,
            state.original_timing_pending_nmi_update_gate,
            state.original_timing_expected_nmi_update_gates.clone(),
            state.game_state.display.nmi_update_is_latched(),
        );
        let sidecars_before = (
            state.main_loop_sprite_preparation_completed,
            state.original_timing_sprite_main_return_claims_remaining,
            state.next_display_obj_scanout_generation,
            state.oam_law_pending.clone(),
            state.original_timing_dungeon_exit_spotlight_entry_return_pending,
        );
        let audio_before = (
            state.game_state.system_signals.ambient_sound_effect(),
            state.game_state.system_signals.last_ambient_sound_effect(),
            state.zelda_debug_apu_write_ports(),
            state.audio_nmi_processed_before_main,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);
        }));
        assert!(result.is_err(), "{label} unexpectedly reached execution");
        assert_eq!(state.original_timing_semantic_receipts, receipts_before);
        assert_eq!(state.game_execution_scheduler, scheduler_before);
        assert_eq!(state.pending_main_loop_common_suffix, suffix_before);
        assert_eq!(state.ram, ram_before);
        assert_eq!(
            (
                state.ppu.vram.clone(),
                state.ppu.oam.clone(),
                state.ppu.cgram.clone(),
            ),
            ppu_before,
        );
        assert_eq!(
            state.display_snapshot.as_ref().map(|snapshot| (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.vram.clone(),
            )),
            snapshot_before,
        );
        assert_eq!(state.display_snapshot_epoch, epoch_before);
        assert_eq!(
            (
                state.original_timing_nmi_publication_pending,
                state.original_timing_pending_nmi_update_gate,
                state.original_timing_expected_nmi_update_gates.clone(),
                state.game_state.display.nmi_update_is_latched(),
            ),
            publication_before,
        );
        assert_eq!(
            (
                state.main_loop_sprite_preparation_completed,
                state.original_timing_sprite_main_return_claims_remaining,
                state.next_display_obj_scanout_generation,
                state.oam_law_pending.clone(),
                state.original_timing_dungeon_exit_spotlight_entry_return_pending,
            ),
            sidecars_before,
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

    let mut missing_domain = run4784_terminal_spotlight_state();
    missing_domain
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .pop();
    assert_rejected("missing caller-return domain fact", missing_domain);

    let mut duplicate_domain = run4784_terminal_spotlight_state();
    duplicate_domain
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .push(OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait);
    assert_rejected("duplicate caller-return domain fact", duplicate_domain);

    let mut reordered_domain = run4784_terminal_spotlight_state();
    reordered_domain
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .swap(3, 4);
    assert_rejected("reordered caller-return domain fact", reordered_domain);

    let mut unexpected_sprite_return = run4784_terminal_spotlight_state();
    unexpected_sprite_return
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .insert(2, OriginalTimingSemanticReceipt::SpriteMainReturned);
    assert_rejected("unexpected Sprite_Main return", unexpected_sprite_return);

    let mut wrong_gate = run4784_terminal_spotlight_state();
    wrong_gate.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
    assert_rejected("wrong gate queue", wrong_gate);

    let mut wrong_native_latch = run4784_terminal_spotlight_state();
    wrong_native_latch.clear_nmi_update_latch();
    assert_rejected("wrong native latch", wrong_native_latch);

    let mut specialized_suffix = run4784_terminal_spotlight_state();
    specialized_suffix.pending_main_loop_common_suffix = Some(
        MainLoopCommonSuffixContinuation::ResumeSpritePreparationExtendedOamPackingAndClearNmiLatch {
            next_group_start: 4,
        },
    );
    assert_rejected("specialized suffix", specialized_suffix);

    let mut carried_handler = run4784_terminal_spotlight_state();
    carried_handler.original_timing_nmi_publication_pending = true;
    carried_handler.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    carried_handler.capture_display_snapshot();
    assert_rejected("carried handler", carried_handler);

    let mut stale_entry_return = run4784_terminal_spotlight_state();
    stale_entry_return.original_timing_dungeon_exit_spotlight_entry_return_pending = true;
    assert_rejected("stale entry-return token", stale_entry_return);

    let closing = SpotlightIteration::closing(SpotlightIterationPhase::WholeTable);
    assert_eq!(
        GameWorkContinuation::FinishSpotlightIteration { iteration: closing }
            .terminal_spotlight_suffix_only_iteration(),
        Some(closing),
    );
    assert_eq!(
        GameWorkContinuation::FinishDungeonExitSpotlightLinkOam { iteration: closing }
            .terminal_spotlight_suffix_only_iteration(),
        Some(closing),
    );

    {
        let (label, iteration) = ("opening work", SpotlightIteration::opening());
        let mut wrong_work = run4784_terminal_spotlight_state();
        wrong_work.game_execution_scheduler.finish_work();
        wrong_work.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishSpotlightIteration { iteration },
            1,
        );
        assert_rejected(label, wrong_work);
    }

    // The Link-movement CPU owner is no longer a rejected shape: route host
    // 50636 proved its wire-owned terminal return, so the scheduled-caller
    // return lane now consumes it (with the Module0F caller-return token)
    // instead of failing closed.
    for (label, work) in [
        (
            "Link-velocity CPU owner",
            GameWorkContinuation::FinishDungeonExitSpotlightLinkVelocity {
                position_return: LinkMovePositionReturn { old_x: 0, old_y: 0 },
                iteration: closing,
            },
        ),
        (
            "overworld Link-OAM owner",
            GameWorkContinuation::FinishOverworldSpotlightLinkOam { iteration: closing },
        ),
    ] {
        assert_eq!(
            work.terminal_spotlight_suffix_only_iteration(),
            None,
            "{label}"
        );
        let mut wrong_work = run4784_terminal_spotlight_state();
        wrong_work.game_execution_scheduler.finish_work();
        wrong_work.game_execution_scheduler.schedule_work(work, 1);
        assert_rejected(label, wrong_work);
    }

    let mut following_field = run4784_terminal_spotlight_state();
    following_field.game_execution_scheduler.finish_work();
    following_field.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishSpotlightIteration {
            iteration: SpotlightIteration::closing(
                SpotlightIterationPhase::WholeTableAfterTablePublication,
            )
            .with_rom_following_field_receipt(
                [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
                SpotlightFollowingFieldPublication::WithCompletionCapture,
            ),
        },
        1,
    );
    assert_rejected("authoritative following-field sidecar", following_field);

    let mut earlier_sprite_preparation = run4784_terminal_spotlight_state();
    earlier_sprite_preparation
        .game_execution_scheduler
        .finish_work();
    earlier_sprite_preparation
        .game_execution_scheduler
        .schedule_work(
            GameWorkContinuation::FinishSpotlightIteration {
                iteration: SpotlightIteration::closing(
                    SpotlightIterationPhase::MixedTailAfterReturn,
                )
                .with_main_loop_sprite_preparation_before_second_nmi(),
            },
            1,
        );
    assert_rejected(
        "earlier sprite-preparation sidecar",
        earlier_sprite_preparation,
    );
}

#[test]
fn live_fresh_dungeon_exit_spotlight_iteration_stops_at_the_source_nmi_checkpoint() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_main_module(0x0f);
    state.set_submodule(0);
    state.set_subsubmodule(0);
    state.set_saved_module_for_menu(7);
    state.set_indoor_flag(0);
    state.follower_link_state_mut().set_x(1272);
    state.follower_link_state_mut().set_y(1012);
    state.set_bg2_h_copy2(1152);
    state.set_bg2_v_copy2(786);
    state.set_spotlight_window_radius(126);
    state.set_spotlight_window_state(0);
    state.set_frame_counter(0xe8);
    state.set_animated_tile_data_source_address(1);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::LatchHeld];
    state.latch_nmi_update();
    state.capture_display_snapshot();
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                crate::SpotlightTableBuildProgressReceipt {
                    progress: crate::SpotlightTableBuildProgress {
                        completed_iterations: 209,
                        checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                            pending_circle_input: 30,
                        },
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
        ],
    ));
    assert_eq!(state.game_state.frame.main_module, 0x0f);
    assert_eq!(state.ram[crate::game_state::constants::MAIN_MODULE], 0x0f);

    let mut timeline_probe = state.clone();
    assert!(timeline_probe
        .take_original_timing_uninterrupted_main_loop_timeline(
            crate::MainLoopProgress::IterationStarted,
        )
        .is_some());

    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.main_module, 0x0f);
    assert_eq!(state.game_state.frame.submodule, 0);
    assert!(state.original_timing_nmi_publication_pending);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonExitSpotlightEntry { .. })
    ));
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn live_fresh_dungeon_exit_spotlight_iteration_stops_before_iteration_initialization() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_main_module(0x0f);
    state.set_submodule(0);
    state.set_subsubmodule(0);
    state.set_saved_module_for_menu(7);
    state.set_indoor_flag(0);
    state.follower_link_state_mut().set_x(1656);
    state.follower_link_state_mut().set_y(8209);
    state.set_bg2_h_copy2(1536);
    state.set_bg2_v_copy2(8191);
    state.set_spotlight_window_radius(126);
    state.set_spotlight_window_state(2);
    state.set_frame_counter(0xe8);
    state.set_animated_tile_data_source_address(1);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::LatchHeld];
    state.latch_nmi_update();
    state.capture_display_snapshot();
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                crate::SpotlightTableBuildProgressReceipt {
                    progress: crate::SpotlightTableBuildProgress {
                        completed_iterations: 175,
                        checkpoint:
                            crate::SpotlightTableBuildCheckpoint::BeforeIterationInitialization,
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
        ],
    ));

    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.main_module, 0x0f);
    assert_eq!(state.game_state.frame.submodule, 0);
    assert!(state.original_timing_nmi_publication_pending);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonExitSpotlightEntry { .. })
    ));
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn live_spotlight_entry_return_is_already_owned_by_advanced_native_caller() {
    for phase in [
        SpotlightIterationPhase::CloseEntryBeforeTablePublication,
        SpotlightIterationPhase::CloseEntryAfterTablePublication,
    ] {
        let mut state = ZeldaState::new();
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        state.set_main_module(15);
        state.set_submodule(1);
        state.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishSpotlightIteration {
                iteration: SpotlightIteration::closing(phase),
            },
            1,
        );
        state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned],
        ));

        assert!(!state.take_or_defer_original_timing_dungeon_exit_spotlight_entry_returned(false));
        assert!(!state.original_timing_dungeon_exit_spotlight_entry_return_pending);
        assert!(matches!(
            state.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishSpotlightIteration { .. })
        ));
    }
}

#[test]
fn carried_terminal_spotlight_build_can_capture_one_trailing_open() {
    let mut state = run4789_terminal_spotlight_build_state();
    state
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .insert(
            3,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        );
    state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::Open];
    state.original_timing_expected_nmi_ppu_register_operands = vec![None, None];
    let epoch_before = state.display_snapshot_epoch;

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 105);
    assert_eq!(
        state.display_snapshot_epoch,
        epoch_before + 2,
        "the carried handler must not recapture; only Build publication and trailing Open do",
    );
    let carried = state
        .display_snapshot
        .as_ref()
        .expect("the terminal Build lost its trailing Open generation");
    assert_eq!(read_le_u16(&carried.ram, SPOTLIGHT_WINDOW_RADIUS), 105,);
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
fn spotlight_recheckpoint_accepts_forward_progress_within_one_iteration() {
    let build = SpotlightTableBuildContinuation {
        source_progress: Some(crate::SpotlightTableBuildProgress {
            completed_iterations: 212,
            checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                pending_circle_input: 27,
            },
        }),
        ..SpotlightTableBuildContinuation::default()
    };

    build.assert_recheckpoint_not_behind(crate::SpotlightTableBuildProgress {
        completed_iterations: 212,
        checkpoint: crate::SpotlightTableBuildCheckpoint::AfterUpperTableWrite {
            lower_cursor: 264,
        },
    });
}

#[test]
fn spotlight_recheckpoint_accepts_the_next_iteration_after_cursor_decrement() {
    let build = SpotlightTableBuildContinuation {
        source_progress: Some(crate::SpotlightTableBuildProgress {
            completed_iterations: 97,
            checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeLowerCursorDecrement {
                upper_cursor: 98,
                lower_cursor: 143,
            },
        }),
        ..SpotlightTableBuildContinuation::default()
    };

    build.assert_recheckpoint_not_behind(crate::SpotlightTableBuildProgress {
        completed_iterations: 98,
        checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeIterationInitialization,
    });
}

#[test]
fn run4789_terminal_spotlight_build_preflight_is_failure_atomic() {
    fn replace_work(
        state: &mut ZeldaState,
        transform: impl FnOnce(GameWorkContinuation) -> GameWorkContinuation,
        slices: u8,
    ) {
        let work = state
            .game_execution_scheduler
            .current_work()
            .expect("run4789 fixture lost its Build owner");
        state.game_execution_scheduler.finish_work();
        state
            .game_execution_scheduler
            .schedule_work(transform(work), slices);
    }

    fn assert_rejected(label: &str, mut state: ZeldaState) {
        let receipts_before = state.original_timing_semantic_receipts.clone();
        let scheduler_before = state.game_execution_scheduler;
        let suffix_before = state.pending_main_loop_common_suffix;
        let game_before = state.game_state.clone();
        let ram_before = state.ram.clone();
        let dma_before = bincode::serialize(&state.dma).unwrap();
        let ppu_before = (
            state.ppu.vram.clone(),
            state.ppu.oam.clone(),
            state.ppu.cgram.clone(),
        );
        let snapshot_fingerprint = |snapshot: Option<&Box<DisplaySnapshot>>| {
            snapshot.map(|snapshot| {
                (
                    snapshot.publication_epoch,
                    snapshot.accepts_nmi_dma_receipts,
                    snapshot.ram.clone(),
                    snapshot.ppu.vram.clone(),
                    snapshot.ppu.oam.clone(),
                    snapshot.ppu.cgram.clone(),
                    snapshot.hdma_table_generation.clone(),
                    snapshot.oam_scanout_source,
                )
            })
        };
        let scanout_fingerprint = |scanout: Option<&LiveSpotlightScanout>| {
            scanout.map(|scanout| {
                (
                    scanout.windowsel,
                    scanout.screen_windowed,
                    scanout.hdma_enable_mask,
                    scanout.hdma_tables.clone(),
                    scanout.authoritative_rom_hdma_receipt,
                )
            })
        };
        let snapshots_before = (
            snapshot_fingerprint(state.display_snapshot.as_ref()),
            snapshot_fingerprint(state.deferred_display_snapshot.as_ref()),
            state.display_snapshot_epoch,
        );
        let sidecars_before = (
            state.original_timing_nmi_publication_pending,
            state.original_timing_pending_nmi_update_gate,
            state.original_timing_expected_nmi_update_gates.clone(),
            state.original_timing_scheduled_nmi_accepted_at_host_return,
            state.original_timing_sprite_main_return_claims_remaining,
            state.original_timing_dungeon_exit_spotlight_entry_return_pending,
            state.main_loop_sprite_preparation_completed,
            state.next_display_obj_scanout_generation,
            scanout_fingerprint(state.next_display_spotlight_scanout.as_ref()),
            scanout_fingerprint(state.spotlight_scanout_after_active_field.as_ref()),
        );
        let audio_before = (
            state.game_state.system_signals.ambient_sound_effect(),
            state.game_state.system_signals.last_ambient_sound_effect(),
            state.zelda_debug_apu_write_ports(),
            state.audio_nmi_processed_before_main,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal(0, crate::RUN_MAIN);
        }));
        assert!(result.is_err(), "{label} unexpectedly reached execution");
        assert_eq!(state.original_timing_semantic_receipts, receipts_before);
        assert_eq!(state.game_execution_scheduler, scheduler_before);
        assert_eq!(state.pending_main_loop_common_suffix, suffix_before);
        assert_eq!(state.game_state, game_before);
        assert_eq!(state.ram, ram_before);
        assert_eq!(bincode::serialize(&state.dma).unwrap(), dma_before);
        assert_eq!(
            (
                state.ppu.vram.clone(),
                state.ppu.oam.clone(),
                state.ppu.cgram.clone(),
            ),
            ppu_before,
        );
        assert_eq!(
            (
                snapshot_fingerprint(state.display_snapshot.as_ref()),
                snapshot_fingerprint(state.deferred_display_snapshot.as_ref()),
                state.display_snapshot_epoch,
            ),
            snapshots_before,
        );
        assert_eq!(
            (
                state.original_timing_nmi_publication_pending,
                state.original_timing_pending_nmi_update_gate,
                state.original_timing_expected_nmi_update_gates.clone(),
                state.original_timing_scheduled_nmi_accepted_at_host_return,
                state.original_timing_sprite_main_return_claims_remaining,
                state.original_timing_dungeon_exit_spotlight_entry_return_pending,
                state.main_loop_sprite_preparation_completed,
                state.next_display_obj_scanout_generation,
                scanout_fingerprint(state.next_display_spotlight_scanout.as_ref()),
                scanout_fingerprint(state.spotlight_scanout_after_active_field.as_ref()),
            ),
            sidecars_before,
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

    let mut reordered = run4789_terminal_spotlight_build_state();
    reordered
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .swap(1, 2);
    assert_rejected("reordered progress and suffix", reordered);

    let mut extra = run4789_terminal_spotlight_build_state();
    extra
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .insert(1, OriginalTimingSemanticReceipt::SpriteMainReturned);
    assert_rejected("extra Sprite_Main return", extra);

    let mut wrong_gate = run4789_terminal_spotlight_build_state();
    wrong_gate.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
    assert_rejected("wrong carried gate queue", wrong_gate);

    let mut wrong_latch = run4789_terminal_spotlight_build_state();
    wrong_latch.clear_nmi_update_latch();
    assert_rejected("wrong native latch", wrong_latch);

    let mut missing_carry = run4789_terminal_spotlight_build_state();
    missing_carry.original_timing_nmi_publication_pending = false;
    missing_carry.original_timing_pending_nmi_update_gate = None;
    assert_rejected("missing carried Held owner", missing_carry);

    let mut nonreceptive = run4789_terminal_spotlight_build_state();
    nonreceptive
        .display_snapshot
        .as_mut()
        .unwrap()
        .accepts_nmi_dma_receipts = false;
    assert_rejected("nonreceptive carried snapshot", nonreceptive);

    // A deferred display generation is no longer a rejection: a wire-held
    // close entry publishes `AdvanceStaged` one host before a Build terminal
    // (route host 76634), so the Build promotes the staged generation through
    // the ordinary pipeline instead of failing the preflight.

    let mut specialized_suffix = run4789_terminal_spotlight_build_state();
    specialized_suffix.pending_main_loop_common_suffix = Some(
        MainLoopCommonSuffixContinuation::ResumeSpritePreparationExtendedOamPackingAndClearNmiLatch {
            next_group_start: 4,
        },
    );
    assert_rejected("specialized suffix", specialized_suffix);

    let mut missing_suffix = run4789_terminal_spotlight_build_state();
    missing_suffix.pending_main_loop_common_suffix = None;
    assert_rejected("missing suffix", missing_suffix);

    let mut stale_acceptance = run4789_terminal_spotlight_build_state();
    stale_acceptance.original_timing_scheduled_nmi_accepted_at_host_return = true;
    assert_rejected("stale staged acceptance", stale_acceptance);

    let mut stale_entry = run4789_terminal_spotlight_build_state();
    stale_entry.original_timing_dungeon_exit_spotlight_entry_return_pending = true;
    assert_rejected("stale entry-return owner", stale_entry);

    let mut stale_sprite_claim = run4789_terminal_spotlight_build_state();
    stale_sprite_claim.original_timing_sprite_main_return_claims_remaining = Some(1);
    assert_rejected("stale Sprite_Main claim", stale_sprite_claim);

    let mut opening = run4789_terminal_spotlight_build_state();
    replace_work(
        &mut opening,
        |work| {
            let GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                table_build,
                projection_completed,
                ..
            } = work
            else {
                unreachable!()
            };
            GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                table_build,
                projection_completed,
                iteration: SpotlightIteration::opening(),
            }
        },
        1,
    );
    assert_rejected("opening Build", opening);

    let mut following = run4789_terminal_spotlight_build_state();
    replace_work(
        &mut following,
        |work| {
            let GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                table_build,
                projection_completed,
                iteration,
            } = work
            else {
                unreachable!()
            };
            GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                table_build,
                projection_completed,
                iteration: iteration.with_rom_following_field_receipt(
                    [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
                    SpotlightFollowingFieldPublication::WithCompletionCapture,
                ),
            }
        },
        1,
    );
    assert_rejected("following-field sidecar", following);

    let mut prepared = run4789_terminal_spotlight_build_state();
    replace_work(
        &mut prepared,
        |work| {
            let GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                table_build,
                projection_completed,
                iteration,
            } = work
            else {
                unreachable!()
            };
            GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                table_build,
                projection_completed,
                iteration: iteration.with_main_loop_sprite_preparation_before_second_nmi(),
            }
        },
        1,
    );
    assert_rejected("earlier sprite-preparation sidecar", prepared);

    let mut noncanonical = run4789_terminal_spotlight_build_state();
    replace_work(
        &mut noncanonical,
        |work| {
            let GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                table_build,
                projection_completed,
                ..
            } = work
            else {
                unreachable!()
            };
            GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                table_build,
                projection_completed,
                iteration: SpotlightIteration::game_over_closing(
                    SpotlightIterationPhase::WholeTable,
                    false,
                ),
            }
        },
        1,
    );
    assert_rejected("noncanonical closing display policy", noncanonical);

    // A carried-Held suffix-only continuation is a valid terminal lifecycle
    // of its own (run4805), so downgrading the Build owner to LinkOam is no
    // longer distinguishable at preflight. An Entry owner still has no
    // terminal grammar and must stay fail-closed.
    let mut wrong_owner = run4789_terminal_spotlight_build_state();
    replace_work(
        &mut wrong_owner,
        |work| {
            let GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                table_build,
                iteration,
                ..
            } = work
            else {
                unreachable!()
            };
            GameWorkContinuation::FinishDungeonExitSpotlightEntry {
                table_build,
                iteration,
            }
        },
        1,
    );
    // FinishDungeonExitSpotlightEntry became a scheduled-return-lane owner at
    // route host 39630, so this rejection now happens inside the lane's
    // vector validation — after the ordinary per-host normalizations
    // (RAM-to-native sync, audio-NMI staging). The atomicity contract for
    // this case therefore covers the lane's own authority: receipts,
    // scheduler, and the pending suffix stay untouched on rejection.
    {
        let mut wrong_owner = wrong_owner;
        let receipts_before = wrong_owner.original_timing_semantic_receipts.clone();
        let scheduler_before = wrong_owner.game_execution_scheduler;
        let suffix_before = wrong_owner.pending_main_loop_common_suffix;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            wrong_owner.run_frame_internal(0, crate::RUN_MAIN);
        }));
        assert!(
            result.is_err(),
            "entry owner with carried Build grammar unexpectedly reached execution"
        );
        assert_eq!(
            wrong_owner.original_timing_semantic_receipts,
            receipts_before
        );
        assert_eq!(wrong_owner.game_execution_scheduler, scheduler_before);
        assert_eq!(wrong_owner.pending_main_loop_common_suffix, suffix_before);
    }

    let mut invalid_table = run4789_terminal_spotlight_build_state();
    replace_work(
        &mut invalid_table,
        |work| {
            let GameWorkContinuation::FinishDungeonExitSpotlightBuild { iteration, .. } = work
            else {
                unreachable!()
            };
            GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                table_build: SpotlightTableBuildContinuation {
                    completed: true,
                    projection_words_copied: 225,
                    ..SpotlightTableBuildContinuation::default()
                },
                projection_completed: false,
                iteration,
            }
        },
        1,
    );
    assert_rejected("invalid stored table callback", invalid_table);

    let mut trailing_held = run4791_terminal_spotlight_build_state();
    trailing_held
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic[4] = OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld);
    trailing_held.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::LatchHeld];
    assert_rejected("trailing Held acceptance", trailing_held);

    let mut post_return_completion = run4791_terminal_spotlight_build_state();
    post_return_completion
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .insert(5, OriginalTimingSemanticReceipt::NmiHandlerCompleted);
    post_return_completion
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .insert(
            6,
            OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                high: 1,
                low: 2,
                high_filtered: 3,
                low_filtered: 4,
            }),
        );
    assert_rejected("post-return completion and Joypad", post_return_completion);

    let mut stray_joypad = run4791_terminal_spotlight_build_state();
    stray_joypad
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
    assert_rejected("stray post-return Joypad", stray_joypad);

    let mut duplicate_open = run4791_terminal_spotlight_build_state();
    duplicate_open
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .insert(
            5,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        );
    duplicate_open
        .original_timing_expected_nmi_update_gates
        .push(NmiUpdateGate::Open);
    assert_rejected("duplicate trailing Open acceptance", duplicate_open);

    let mut wire_last_reordered = run4791_terminal_spotlight_build_state();
    wire_last_reordered
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .swap(4, 5);
    assert_rejected(
        "caller-return token before trailing Open",
        wire_last_reordered,
    );

    let mut missing_same_host_acceptance = run4791_terminal_spotlight_build_state();
    missing_same_host_acceptance
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .remove(0);
    assert_rejected(
        "same-host Build without its Held acceptance",
        missing_same_host_acceptance,
    );

    let mut wrong_same_host_gate = run4791_terminal_spotlight_build_state();
    wrong_same_host_gate
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic[0] = OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open);
    wrong_same_host_gate.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::Open, NmiUpdateGate::Open];
    assert_rejected("same-host Open instead of Held", wrong_same_host_gate);

    let mut wrong_trailing_gate_queue = run4791_terminal_spotlight_build_state();
    wrong_trailing_gate_queue.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::LatchHeld];
    assert_rejected("wrong trailing gate queue", wrong_trailing_gate_queue);

    let mut unexpected_carry = run4791_terminal_spotlight_build_state();
    unexpected_carry.original_timing_nmi_publication_pending = true;
    unexpected_carry.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    unexpected_carry
        .display_snapshot
        .as_mut()
        .unwrap()
        .accepts_nmi_dma_receipts = true;
    assert_rejected("carried owner with same-host vector", unexpected_carry);

    let mut same_host_wrong_latch = run4791_terminal_spotlight_build_state();
    same_host_wrong_latch.clear_nmi_update_latch();
    assert_rejected(
        "same-host Build with open native latch",
        same_host_wrong_latch,
    );
}

#[test]
fn spotlight_link_oam_publishes_later_table_stores_before_its_accepting_nmi() {
    let mut state = run4786_spotlight_build_link_oam_state();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    let prior = SpotlightTableBuildProgress {
        completed_iterations: 214,
        checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
            pending_circle_input: 25,
        },
    };
    let accepted = SpotlightTableBuildProgress {
        completed_iterations: 214,
        checkpoint: crate::SpotlightTableBuildCheckpoint::AfterUpperTableWrite {
            lower_cursor: 262,
        },
    };
    let table_build = state.begin_iris_spotlight_configure_table_at_progress(prior);
    state.game_execution_scheduler.finish_work();
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonExitSpotlightBuild {
            table_build,
            projection_completed: false,
            iteration: SpotlightIteration::closing(SpotlightIterationPhase::WholeTable),
        },
        1,
    );
    state
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .insert(
            1,
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                crate::SpotlightTableBuildProgressReceipt {
                    progress: accepted,
                    boundary: crate::OriginalTimingBoundary::NmiAccepted,
                },
            ),
        );
    let game_before = state.game_state.clone();
    let plan = state
        .original_timing_spotlight_build_link_oam_plan()
        .unwrap();
    assert_eq!(plan.rebuild_progress, Some(accepted));
    assert_eq!(
        state.game_state, game_before,
        "preflight must remain immutable"
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonExitSpotlightLinkOam { .. })
    ));
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 112);
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn recurring_spotlight_build_binds_an_entry_recheckpoint_before_handler_completion() {
    for advances_upper_cursor in [false, true] {
        let mut state = run4786_spotlight_build_link_oam_state();
        let mut progress = crate::SpotlightTableBuildProgress {
            completed_iterations: 105,
            checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeIterationInitialization,
        };
        if advances_upper_cursor {
            let work = state.game_execution_scheduler.current_work().unwrap();
            let GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                table_build,
                projection_completed,
                iteration,
            } = work
            else {
                unreachable!()
            };
            let source = crate::SpotlightTableBuildProgress {
                completed_iterations: 105,
                checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeLoopCompletionTest {
                    upper_cursor: table_build.upper_cursor,
                    lower_cursor: table_build.lower_cursor,
                },
            };
            let table_build = state.begin_iris_spotlight_configure_table_at_progress(source);
            progress.checkpoint =
                crate::SpotlightTableBuildCheckpoint::BeforeLowerCursorDecrement {
                    upper_cursor: table_build.upper_cursor.wrapping_add(1),
                    lower_cursor: table_build.lower_cursor,
                };
            state.game_execution_scheduler.refine_scheduled_work(
                work,
                GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                    table_build,
                    projection_completed,
                    iteration,
                },
            );
        }
        state.original_timing_expected_nmi_update_gates =
            vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::LatchHeld];
        state.original_timing_expected_nmi_ppu_register_operands = vec![None, None];
        state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
            4_786,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                    crate::SpotlightTableBuildProgressReceipt {
                        progress,
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    crate::MainLoopInterruption::LinkOam,
                ),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
            ],
        ));

        state.run_frame_internal(0, crate::RUN_MAIN);

        assert!(matches!(
            state.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDungeonExitSpotlightLinkOam { .. })
        ));
        assert!(state.original_timing_semantic_receipts.is_none());
    }
}

#[test]
fn run4786_spotlight_build_link_oam_preflight_is_failure_atomic() {
    fn replace_iteration(state: &mut ZeldaState, iteration: SpotlightIteration) {
        let Some(GameWorkContinuation::FinishDungeonExitSpotlightBuild {
            table_build,
            projection_completed,
            ..
        }) = state.game_execution_scheduler.current_work()
        else {
            panic!("run4786 fixture lost its Build owner")
        };
        state.game_execution_scheduler.finish_work();
        state.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishDungeonExitSpotlightBuild {
                table_build,
                projection_completed,
                iteration,
            },
            1,
        );
    }

    fn assert_rejected(label: &str, mut state: ZeldaState) {
        let receipts_before = state.original_timing_semantic_receipts.clone();
        let scheduler_before = state.game_execution_scheduler;
        let suffix_before = state.pending_main_loop_common_suffix;
        let game_before = state.game_state.clone();
        let ram_before = state.ram.clone();
        let dma_before = bincode::serialize(&state.dma).unwrap();
        let ppu_before = (
            state.ppu.vram.clone(),
            state.ppu.oam.clone(),
            state.ppu.cgram.clone(),
        );
        let snapshot_fingerprint = |snapshot: Option<&Box<DisplaySnapshot>>| {
            snapshot.map(|snapshot| {
                (
                    snapshot.publication_epoch,
                    snapshot.accepts_nmi_dma_receipts,
                    snapshot.ram.clone(),
                    snapshot.ppu.vram.clone(),
                    snapshot.ppu.oam.clone(),
                    snapshot.ppu.cgram.clone(),
                    snapshot.hdma_table_generation.clone(),
                    snapshot.oam_scanout_source,
                )
            })
        };
        let scanout_fingerprint = |scanout: Option<&LiveSpotlightScanout>| {
            scanout.map(|scanout| {
                (
                    scanout.windowsel,
                    scanout.screen_windowed,
                    scanout.hdma_enable_mask,
                    scanout.hdma_tables.clone(),
                    scanout.authoritative_rom_hdma_receipt,
                )
            })
        };
        let snapshots_before = (
            snapshot_fingerprint(state.display_snapshot.as_ref()),
            snapshot_fingerprint(state.deferred_display_snapshot.as_ref()),
            state.display_snapshot_epoch,
        );
        let publication_before = (
            state.original_timing_nmi_publication_pending,
            state.original_timing_pending_nmi_update_gate,
            state.original_timing_expected_nmi_update_gates.clone(),
            state.original_timing_scheduled_nmi_accepted_at_host_return,
            state.original_timing_sprite_main_return_claims_remaining,
            state.next_display_obj_scanout_generation,
            scanout_fingerprint(state.next_display_spotlight_scanout.as_ref()),
            scanout_fingerprint(state.spotlight_scanout_after_active_field.as_ref()),
        );
        let audio_before = (
            state.game_state.system_signals.ambient_sound_effect(),
            state.game_state.system_signals.last_ambient_sound_effect(),
            state.zelda_debug_apu_write_ports(),
            state.audio_nmi_processed_before_main,
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal(0, crate::RUN_MAIN);
        }));
        assert!(result.is_err(), "{label} unexpectedly reached execution");
        assert_eq!(state.original_timing_semantic_receipts, receipts_before);
        assert_eq!(state.game_execution_scheduler, scheduler_before);
        assert_eq!(state.pending_main_loop_common_suffix, suffix_before);
        assert_eq!(state.game_state, game_before);
        assert_eq!(state.ram, ram_before);
        assert_eq!(bincode::serialize(&state.dma).unwrap(), dma_before);
        assert_eq!(
            (
                state.ppu.vram.clone(),
                state.ppu.oam.clone(),
                state.ppu.cgram.clone(),
            ),
            ppu_before,
        );
        assert_eq!(
            (
                snapshot_fingerprint(state.display_snapshot.as_ref()),
                snapshot_fingerprint(state.deferred_display_snapshot.as_ref()),
                state.display_snapshot_epoch,
            ),
            snapshots_before,
        );
        assert_eq!(
            (
                state.original_timing_nmi_publication_pending,
                state.original_timing_pending_nmi_update_gate,
                state.original_timing_expected_nmi_update_gates.clone(),
                state.original_timing_scheduled_nmi_accepted_at_host_return,
                state.original_timing_sprite_main_return_claims_remaining,
                state.next_display_obj_scanout_generation,
                scanout_fingerprint(state.next_display_spotlight_scanout.as_ref()),
                scanout_fingerprint(state.spotlight_scanout_after_active_field.as_ref()),
            ),
            publication_before,
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

    let mut reordered = run4786_spotlight_build_link_oam_state();
    reordered
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .swap(2, 3);
    assert_rejected("reordered progress and LinkOam", reordered);

    let mut extra = run4786_spotlight_build_link_oam_state();
    extra
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .insert(2, OriginalTimingSemanticReceipt::SpriteMainReturned);
    assert_rejected("extra Sprite_Main return", extra);

    let mut wrong_interruption = run4786_spotlight_build_link_oam_state();
    *wrong_interruption
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .last_mut()
        .unwrap() = OriginalTimingSemanticReceipt::MainLoopInterrupted(
        crate::MainLoopInterruption::SpritePreparation,
    );
    assert_rejected("wrong interruption", wrong_interruption);

    let mut wrong_gate = run4786_spotlight_build_link_oam_state();
    wrong_gate.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
    assert_rejected("wrong gate", wrong_gate);

    let mut wrong_latch = run4786_spotlight_build_link_oam_state();
    wrong_latch.clear_nmi_update_latch();
    assert_rejected("wrong native latch", wrong_latch);

    let mut carried = run4786_spotlight_build_link_oam_state();
    carried.original_timing_nmi_publication_pending = true;
    carried.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    carried.capture_display_snapshot();
    assert_rejected("carried handler", carried);

    let mut missing_suffix = run4786_spotlight_build_link_oam_state();
    missing_suffix.pending_main_loop_common_suffix = None;
    assert_rejected("missing suffix", missing_suffix);

    let mut stale_acceptance = run4786_spotlight_build_link_oam_state();
    stale_acceptance.original_timing_scheduled_nmi_accepted_at_host_return = true;
    assert_rejected("stale staged acceptance", stale_acceptance);

    let mut opening = run4786_spotlight_build_link_oam_state();
    replace_iteration(&mut opening, SpotlightIteration::opening());
    assert_rejected("opening iteration", opening);

    let mut following = run4786_spotlight_build_link_oam_state();
    replace_iteration(
        &mut following,
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTable)
            .with_rom_following_field_receipt(
                [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
                SpotlightFollowingFieldPublication::WithCompletionCapture,
            ),
    );
    assert_rejected("following-field sidecar", following);

    let mut prepared = run4786_spotlight_build_link_oam_state();
    replace_iteration(
        &mut prepared,
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTable)
            .with_main_loop_sprite_preparation_before_second_nmi(),
    );
    assert_rejected("earlier sprite-preparation sidecar", prepared);

    let mut invalid_table = run4786_spotlight_build_link_oam_state();
    invalid_table.game_execution_scheduler.finish_work();
    invalid_table.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonExitSpotlightBuild {
            table_build: SpotlightTableBuildContinuation {
                completed: true,
                projection_words_copied: 225,
                ..SpotlightTableBuildContinuation::default()
            },
            projection_completed: false,
            iteration: SpotlightIteration::closing(SpotlightIterationPhase::WholeTable),
        },
        1,
    );
    assert_rejected("invalid stored table callback", invalid_table);
}

#[test]
fn spotlight_link_oam_does_not_complete_a_non_build_caller() {
    let mut state = run4786_spotlight_build_link_oam_state();
    let continuation = GameWorkContinuation::FinishSpotlightIteration {
        iteration: SpotlightIteration::closing(SpotlightIterationPhase::WholeTable),
    };
    state.game_execution_scheduler.finish_work();
    state
        .game_execution_scheduler
        .schedule_work(continuation, 1);

    let scheduler_before = state.game_execution_scheduler;
    let receipts_before = state.original_timing_semantic_receipts.clone();
    let ram_before = state.ram.clone();
    let game_before = state.game_state.clone();
    let ppu_before = (
        state.ppu.vram.clone(),
        state.ppu.oam.clone(),
        state.ppu.cgram.clone(),
    );
    let epoch_before = state.display_snapshot_epoch;
    assert!(state
        .original_timing_spotlight_build_link_oam_plan()
        .is_none());
    assert_eq!(state.game_execution_scheduler, scheduler_before);
    assert_eq!(state.original_timing_semantic_receipts, receipts_before);
    assert_eq!(state.ram, ram_before);
    assert_eq!(state.game_state, game_before);
    assert_eq!(
        (
            state.ppu.vram.clone(),
            state.ppu.oam.clone(),
            state.ppu.cgram.clone(),
        ),
        ppu_before,
    );
    assert_eq!(state.display_snapshot_epoch, epoch_before);

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(continuation)
    );
    assert_eq!(
        state
            .game_execution_scheduler
            .scheduled_work_slices_remaining(),
        Some(0),
        "LinkOam authority must not complete a non-Build spotlight caller",
    );
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
#[should_panic(
    expected = "dungeon-exit spotlight entry returned outside native Module15 entry transition"
)]
fn live_spotlight_entry_return_outside_module15_fails_closed() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(7);
    state.set_submodule(15);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned],
    ));

    state.take_or_defer_original_timing_dungeon_exit_spotlight_entry_returned(false);
}

#[test]
fn live_spotlight_progress_suspends_module0f_before_submodule_advance() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(0x0f);
    state.set_submodule(0);
    state.set_subsubmodule(0);
    state.set_indoor_flag(0);
    state.follower_link_state_mut().set_x(1272);
    state.follower_link_state_mut().set_y(1012);
    state.set_bg2_h_copy2(1152);
    state.set_bg2_v_copy2(786);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                crate::SpotlightTableBuildProgressReceipt {
                    progress: crate::SpotlightTableBuildProgress {
                        completed_iterations: 209,
                        checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                            pending_circle_input: 30,
                        },
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
        ],
    ));
    state.game_execution_scheduler.begin_host_frame();

    state.zelda_run_game_loop();

    assert_eq!(state.game_state.frame.main_module, 0x0f);
    assert_eq!(state.game_state.frame.submodule, 0);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonExitSpotlightEntry { .. })
    ));
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn fresh_spotlight_entry_return_coexists_with_its_link_velocity_checkpoint() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(0x0f);
    state.set_submodule(0);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        520749,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                high: 8,
                low: 0,
                high_filtered: 0,
                low_filtered: 0,
            }),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                crate::MainLoopInterruption::LinkActualVelocity {
                    horizontal_resolved: Some(true),
                },
            ),
            OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned,
        ],
    ));
    assert!(state
        .original_timing_interrupted_idle_main_loop_plan()
        .is_some());
    state.set_submodule(1);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
        || state.original_timing_interrupted_idle_main_loop_plan()
    ))
    .is_err());
}

#[test]
fn terminal_dungeon_spotlight_caller_returns_once_and_carries_host2294_open_nmi() {
    let mut state = live_dungeon_spotlight_caller_before_terminal_return(1);
    state
        .install_original_timing_host_receipts(dungeon_spotlight_terminal_return_receipts(2294))
        .unwrap();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::Open),
    );
    assert!(!state.original_timing_scheduled_nmi_accepted_at_host_return);
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
    assert!(state.original_timing_semantic_receipts.is_none());

    // The trailing Open acceptance belongs to the next source host. Complete
    // it through the real dispatch owner and prove that the carried handler
    // (including its joypad publication) executes exactly once.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2295,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                    high: 0x80,
                    low: 0x01,
                    high_filtered: 0x80,
                    low_filtered: 0x01,
                }),
            ],
        ))
        .unwrap();
    let owns_dispatch = state.begin_original_timing_host_dispatch(0);
    assert!(owns_dispatch);
    let phases = state.take_original_timing_nmi_phases();
    assert_eq!(phases, [OriginalTimingNmiPhase::HandlerCompleted]);
    let classification = classify_original_timing_nmi_phases_with_ownership(true, &phases);
    assert_eq!(
        classification.handler_completion,
        OriginalTimingNmiHandlerCompletionOwner::PendingAtSequenceEntry,
    );
    assert!(state
        .complete_original_timing_nmi_handler_for_active_scanout(
            classification.handler_completion,
            0,
            None,
        )
        .completed());
    state.finish_original_timing_host_dispatch(owns_dispatch);
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.game_state.player.follower_link.joypad1h_last(), 0x80,);
    assert_eq!(state.game_state.player.follower_link.joypad1l_last(), 0x01,);
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn terminal_dungeon_spotlight_caller_host2296_stops_at_wait_without_a_trailing_nmi() {
    let mut state = live_dungeon_spotlight_caller_before_terminal_return(1);
    state.set_frame_counter(126);
    state
        .install_original_timing_host_receipts(
            dungeon_spotlight_terminal_return_without_trailing_nmi_receipts(2296),
        )
        .unwrap();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert!(state
        .game_execution_scheduler
        .returned_main_is_waiting_for_nmi());
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(!state.original_timing_scheduled_nmi_accepted_at_host_return);
    assert!(state.display_snapshot.is_some());
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn terminal_dungeon_spotlight_caller_completes_host2299_carried_held_nmi_in_host2300() {
    let mut state = live_dungeon_spotlight_caller_before_terminal_return(1);
    state.set_frame_counter(128);

    // Source host 2299 presents and accepts the held NMI at the same raster
    // position, then returns from inside its handler. Preserve that acceptance
    // generation through frontend presentation before host 2300 resumes it.
    carry_dungeon_spotlight_nmi_from_acceptance_host(&mut state, 2299, NmiUpdateGate::LatchHeld);
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
    state.with_display_snapshot(|_| ());
    state.advance_display_publication_history();
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));

    state
        .install_original_timing_host_receipts(dungeon_spotlight_carried_terminal_return_receipts(
            2300,
        ))
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert!(state
        .game_execution_scheduler
        .returned_main_is_waiting_for_nmi());
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(!state.original_timing_scheduled_nmi_accepted_at_host_return);
    assert!(state.display_snapshot.is_some());
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn run2318_terminal_dungeon_spotlight_resumes_outer_sprite_main_before_suffix() {
    let mut state = live_dungeon_spotlight_caller_before_terminal_return(1);
    state.set_submodule(0x0f);
    state.set_subsubmodule(1);
    state.set_frame_counter(137);
    // This regression owns the terminal receipt/caller boundary. Seed the
    // following iteration's independent ROM-timing probe so the focused test
    // does not require the local Zelda ROM asset.
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::CompleteBeforeNmi,
        resumed_phase: None,
        submodule_nmi_slices: 0,
        subsubmodule: 1,
        palette_countdown: 0,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });

    carry_dungeon_spotlight_nmi_from_acceptance_host(&mut state, 2_317, NmiUpdateGate::LatchHeld);
    let acceptance_epoch = state.display_snapshot_epoch;
    state
        .install_original_timing_host_receipts(dungeon_spotlight_carried_terminal_return_receipts(
            2_318,
        ))
        .unwrap();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert_eq!(
        state.original_timing_sprite_main_return_claims_remaining, None,
        "the outer Sprite_Main return claim must retire before the common suffix",
    );
    assert_eq!(
        state.display_snapshot_epoch,
        acceptance_epoch + 1,
        "the carried handler must refine its acceptance snapshot before the terminal host captures its one new video boundary",
    );
    assert!(!state.original_timing_nmi_publication_pending);
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn terminal_dungeon_spotlight_caller_without_trailing_nmi_allows_retained_scanout() {
    let mut state = live_dungeon_spotlight_caller_before_terminal_return(1);
    state.dungeon_landing_goal_display_handoff =
        DungeonLandingGoalDisplayHandoff::RetainCallerReturn;
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::RetainPublished);
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| !snapshot.accepts_nmi_dma_receipts));
    state
        .install_original_timing_host_receipts(
            dungeon_spotlight_terminal_return_without_trailing_nmi_receipts(2296),
        )
        .unwrap();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert!(state
        .game_execution_scheduler
        .returned_main_is_waiting_for_nmi());
    assert_eq!(
        state.dungeon_landing_goal_display_handoff,
        DungeonLandingGoalDisplayHandoff::None,
    );
    assert!(!state.original_timing_nmi_publication_pending);
    assert!(!state.original_timing_scheduled_nmi_accepted_at_host_return);
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn terminal_dungeon_spotlight_caller_preflight_is_failure_atomic() {
    fn assert_rejected_without_mutation(mut state: ZeldaState) {
        let work_before = state.game_execution_scheduler.current_work();
        let slices_before = state
            .game_execution_scheduler
            .scheduled_work_slices_remaining();
        let ram_before = state.ram.clone();
        let receipts_before = state.original_timing_semantic_receipts.clone();
        let snapshot_before = state.display_snapshot.as_ref().map(|snapshot| {
            (
                snapshot.accepts_nmi_dma_receipts,
                snapshot.publication_host_frame,
                snapshot.ram.clone(),
                snapshot.ppu.oam.clone(),
                snapshot.ppu.vram.clone(),
            )
        });
        let suffix_before = state.pending_main_loop_common_suffix;
        let latch_before = state.game_state.display.nmi_update_is_latched();
        let pending_before = state.original_timing_nmi_publication_pending;
        let pending_gate_before = state.original_timing_pending_nmi_update_gate;
        let expected_gates_before = state.original_timing_expected_nmi_update_gates.clone();
        let staged_before = state.original_timing_scheduled_nmi_accepted_at_host_return;
        let goal_transition_before = state.dungeon_landing_goal_transition_pending;
        let goal_handoff_before = state.dungeon_landing_goal_display_handoff;
        let sprite_main_claims_before = state.original_timing_sprite_main_return_claims_remaining;

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal(0, crate::RUN_MAIN);
        }));
        assert!(result.is_err());
        assert_eq!(state.game_execution_scheduler.current_work(), work_before);
        assert_eq!(
            state
                .game_execution_scheduler
                .scheduled_work_slices_remaining(),
            slices_before,
        );
        assert_eq!(state.ram, ram_before);
        assert_eq!(state.original_timing_semantic_receipts, receipts_before);
        assert_eq!(
            state.display_snapshot.as_ref().map(|snapshot| (
                snapshot.accepts_nmi_dma_receipts,
                snapshot.publication_host_frame,
                snapshot.ram.clone(),
                snapshot.ppu.oam.clone(),
                snapshot.ppu.vram.clone(),
            )),
            snapshot_before,
        );
        assert_eq!(state.pending_main_loop_common_suffix, suffix_before);
        assert_eq!(
            state.game_state.display.nmi_update_is_latched(),
            latch_before,
        );
        assert_eq!(
            state.original_timing_nmi_publication_pending,
            pending_before
        );
        assert_eq!(
            state.original_timing_pending_nmi_update_gate,
            pending_gate_before,
        );
        assert_eq!(
            state.original_timing_expected_nmi_update_gates,
            expected_gates_before,
        );
        assert_eq!(
            state.original_timing_scheduled_nmi_accepted_at_host_return,
            staged_before,
        );
        assert_eq!(
            state.dungeon_landing_goal_transition_pending,
            goal_transition_before,
        );
        assert_eq!(
            state.dungeon_landing_goal_display_handoff,
            goal_handoff_before,
        );
        assert_eq!(
            state.original_timing_sprite_main_return_claims_remaining,
            sprite_main_claims_before,
        );
    }

    fn assert_install_rejected_without_mutation(
        mut state: ZeldaState,
        receipts: OriginalTimingHostReceipts,
    ) {
        let scheduler_before = state.game_execution_scheduler;
        let ram_before = state.ram.clone();
        let receipts_before = state.original_timing_semantic_receipts.clone();
        let snapshot_before = state.display_snapshot.as_ref().map(|snapshot| {
            (
                snapshot.accepts_nmi_dma_receipts,
                snapshot.publication_host_frame,
                snapshot.ram.clone(),
                snapshot.ppu.oam.clone(),
                snapshot.ppu.vram.clone(),
            )
        });
        let pending_before = state.original_timing_nmi_publication_pending;
        let pending_gate_before = state.original_timing_pending_nmi_update_gate;
        let expected_gates_before = state.original_timing_expected_nmi_update_gates.clone();

        assert!(state
            .install_original_timing_host_receipts(receipts)
            .is_err());
        assert_eq!(state.game_execution_scheduler, scheduler_before);
        assert_eq!(state.ram, ram_before);
        assert_eq!(state.original_timing_semantic_receipts, receipts_before);
        assert_eq!(
            state.display_snapshot.as_ref().map(|snapshot| (
                snapshot.accepts_nmi_dma_receipts,
                snapshot.publication_host_frame,
                snapshot.ram.clone(),
                snapshot.ppu.oam.clone(),
                snapshot.ppu.vram.clone(),
            )),
            snapshot_before,
        );
        assert_eq!(
            state.original_timing_nmi_publication_pending,
            pending_before,
        );
        assert_eq!(
            state.original_timing_pending_nmi_update_gate,
            pending_gate_before,
        );
        assert_eq!(
            state.original_timing_expected_nmi_update_gates,
            expected_gates_before,
        );
    }

    let bare_completion = live_dungeon_spotlight_caller_before_terminal_return(1);
    assert_install_rejected_without_mutation(
        bare_completion,
        dungeon_spotlight_carried_terminal_return_receipts(2300),
    );

    let mut missing_acceptance_snapshot = live_dungeon_spotlight_caller_before_terminal_return(1);
    carry_dungeon_spotlight_nmi_from_acceptance_host(
        &mut missing_acceptance_snapshot,
        2299,
        NmiUpdateGate::LatchHeld,
    );
    missing_acceptance_snapshot.display_snapshot = None;
    missing_acceptance_snapshot
        .install_original_timing_host_receipts(dungeon_spotlight_carried_terminal_return_receipts(
            2300,
        ))
        .unwrap();
    assert_rejected_without_mutation(missing_acceptance_snapshot);

    let mut closed_acceptance_snapshot = live_dungeon_spotlight_caller_before_terminal_return(1);
    carry_dungeon_spotlight_nmi_from_acceptance_host(
        &mut closed_acceptance_snapshot,
        2299,
        NmiUpdateGate::LatchHeld,
    );
    closed_acceptance_snapshot
        .display_snapshot
        .as_mut()
        .unwrap()
        .accepts_nmi_dma_receipts = false;
    closed_acceptance_snapshot
        .install_original_timing_host_receipts(dungeon_spotlight_carried_terminal_return_receipts(
            2300,
        ))
        .unwrap();
    assert_rejected_without_mutation(closed_acceptance_snapshot);

    let mut carried_open = live_dungeon_spotlight_caller_before_terminal_return(1);
    carry_dungeon_spotlight_nmi_from_acceptance_host(&mut carried_open, 2299, NmiUpdateGate::Open);
    assert_install_rejected_without_mutation(
        carried_open,
        dungeon_spotlight_carried_terminal_return_receipts(2300),
    );

    let mut duplicate_acceptance = live_dungeon_spotlight_caller_before_terminal_return(1);
    carry_dungeon_spotlight_nmi_from_acceptance_host(
        &mut duplicate_acceptance,
        2299,
        NmiUpdateGate::LatchHeld,
    );
    assert_install_rejected_without_mutation(
        duplicate_acceptance,
        dungeon_spotlight_terminal_return_without_trailing_nmi_receipts(2300),
    );

    let mut missing_suffix = live_dungeon_spotlight_caller_before_terminal_return(1);
    missing_suffix.pending_main_loop_common_suffix = None;
    let scheduler_before = missing_suffix.game_execution_scheduler;
    let receipts_before = missing_suffix.original_timing_semantic_receipts.clone();
    assert!(missing_suffix
        .install_original_timing_host_receipts(dungeon_spotlight_terminal_return_receipts(2294))
        .is_err());
    assert_eq!(missing_suffix.game_execution_scheduler, scheduler_before);
    assert_eq!(
        missing_suffix.original_timing_semantic_receipts,
        receipts_before
    );

    let mut malformed = live_dungeon_spotlight_caller_before_terminal_return(1);
    let scheduler_before = malformed.game_execution_scheduler;
    assert!(malformed
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2294,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            ],
        ))
        .is_err());
    assert_eq!(malformed.game_execution_scheduler, scheduler_before);
    assert!(malformed.original_timing_semantic_receipts.is_none());

    let mut trailing_held = live_dungeon_spotlight_caller_before_terminal_return(1);
    trailing_held
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2296,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            ],
        ))
        .unwrap();
    assert_rejected_without_mutation(trailing_held);

    let mut trailing_completion = live_dungeon_spotlight_caller_before_terminal_return(1);
    let scheduler_before = trailing_completion.game_execution_scheduler;
    let receipts_before = trailing_completion
        .original_timing_semantic_receipts
        .clone();
    assert!(trailing_completion
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2296,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            ],
        ))
        .is_err());
    assert_eq!(
        trailing_completion.game_execution_scheduler,
        scheduler_before,
    );
    assert_eq!(
        trailing_completion.original_timing_semantic_receipts,
        receipts_before,
    );

    let mut trailing_open_completion = live_dungeon_spotlight_caller_before_terminal_return(1);
    trailing_open_completion
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2296,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
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
        ))
        .unwrap();
    assert_rejected_without_mutation(trailing_open_completion);

    for semantic in [
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    ] {
        let mut malformed_sprite_return = live_dungeon_spotlight_caller_before_terminal_return(1);
        malformed_sprite_return
            .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
                2_318, 0, semantic,
            ))
            .unwrap();
        assert_rejected_without_mutation(malformed_sprite_return);
    }

    let mut stale_sprite_main_scope = live_dungeon_spotlight_caller_before_terminal_return(1);
    stale_sprite_main_scope
        .install_original_timing_host_receipts(dungeon_spotlight_terminal_return_receipts(2_318))
        .unwrap();
    stale_sprite_main_scope.original_timing_sprite_main_return_claims_remaining = Some(0);
    assert_rejected_without_mutation(stale_sprite_main_scope);

    let mut unsupported = live_dungeon_spotlight_caller_before_terminal_return(1);
    unsupported.game_execution_scheduler = GameExecutionScheduler::default();
    unsupported.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn,
        1,
    );
    assert!(unsupported
        .game_execution_scheduler
        .work_suspends_translated_call_stack());
    unsupported
        .install_original_timing_host_receipts(dungeon_spotlight_terminal_return_receipts(2294))
        .unwrap();
    assert_rejected_without_mutation(unsupported);

    let mut wrong_gate = live_dungeon_spotlight_caller_before_terminal_return(1);
    wrong_gate
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2294,
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
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            ],
        ))
        .unwrap();
    assert_rejected_without_mutation(wrong_gate);

    // A retained landing-goal caller-return image combined with a trailing
    // acceptance is no longer a rejected shape: route host 39759 proved it on
    // the wire, and the carried acceptance now takes a fresh receptive
    // capture when the retained image is non-receptive
    // (`carry_original_timing_scheduled_caller_host_return_from_active_capture`).
}

#[test]
fn live_spotlight_link_position_boundary_resumes_the_complete_c_leaf_once() {
    fn configured_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.set_rom_startup_timing(true);
        state.set_main_module(0x0f);
        state.set_submodule(1);
        state.set_indoor_flag(0);
        state.set_overworld_screen(0x0f);
        state.follower_link_state_mut().set_x(680);
        state.follower_link_state_mut().set_y(2166);
        state
            .follower_link_state_mut()
            .set_direction_and_last_direction(8);
        state.follower_link_state_mut().set_actual_y_velocity(0xf8);
        state
    }

    for caller_returned in [false, true] {
        let iteration = SpotlightIteration::closing(SpotlightIterationPhase::WholeTable);
        let mut atomic = configured_state();
        atomic.module0f_spotlight_close_link_and_oam();

        let mut resumed = configured_state();
        resumed.original_timing_owner = OriginalTimingOwnerState::Live;
        resumed.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
            50_635,
            0,
            vec![OriginalTimingSemanticReceipt::MainLoopInterrupted(
                crate::MainLoopInterruption::LinkPositionBeforeCoordinates,
            )],
        ));
        let entry_x = resumed.game_state.player.follower_link.x();
        let entry_y = resumed.game_state.player.follower_link.y();

        assert!(resumed.begin_module0f_spotlight_close_link_and_oam(None, iteration));
        assert_eq!(resumed.game_state.player.follower_link.x(), entry_x);
        assert_eq!(resumed.game_state.player.follower_link.y(), entry_y);
        assert_eq!(
            resumed
                .game_state
                .player
                .follower_link
                .water_ripple_or_grass_state(),
            1,
            "Module0F's source prefix must publish before the Link velocity interruption",
        );
        assert_eq!(resumed.game_state.player.follower_link.speed_setting(), 6,);
        assert!(matches!(
            resumed.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDungeonExitSpotlightLinkMovement {
                iteration: pending,
                ..
            }) if pending == iteration
        ));
        assert!(resumed
            .original_timing_semantic_receipts
            .as_ref()
            .is_some_and(|receipts| receipts.semantic().is_empty()));

        let Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishDungeonExitSpotlightLinkMovement { iteration },
        )) = resumed
            .game_execution_scheduler
            .advance_work_one_nmi_slice()
        else {
            panic!("Link movement did not resume at the following accepted NMI");
        };
        resumed.complete_dungeon_exit_spotlight_link_movement(iteration, caller_returned);

        assert_eq!(resumed.ram, atomic.ram);
        assert_eq!(
            matches!(
                resumed.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishSpotlightIteration { .. })
            ),
            !caller_returned
        );
    }
}

#[test]
fn live_spotlight_actual_velocity_boundaries_do_not_replay_velocity_selection() {
    fn configured_state(direction: u8) -> ZeldaState {
        let mut state = ZeldaState::new();
        state.set_rom_startup_timing(true);
        state.set_main_module(0x0f);
        state.set_submodule(1);
        state.set_indoor_flag(0);
        state.set_overworld_screen(0x00);
        state.follower_link_state_mut().set_x(0x0e60);
        state.follower_link_state_mut().set_y(0x0209);
        state
            .follower_link_state_mut()
            .set_direction_and_last_direction(direction);
        state.follower_link_state_mut().set_actual_y_velocity(0x14);
        state.follower_link_state_mut().set_actual_x_velocity(0x43);
        state
            .follower_link_state_mut()
            .set_page_movement_deltas(7, 9);
        state
    }

    // Original ROM $07:E2DE (host 340155) has cleared velocity but has
    // not adjusted speed; $07:E346 reaches the later per-axis loop.
    for checkpoint in [
        LinkActualVelocityCheckpoint::Clearing { completed: 1 },
        LinkActualVelocityCheckpoint::Clearing { completed: 2 },
        LinkActualVelocityCheckpoint::Clearing { completed: 3 },
        LinkActualVelocityCheckpoint::BeforeSelection,
        LinkActualVelocityCheckpoint::BeforeX,
        LinkActualVelocityCheckpoint::BeforeY,
        LinkActualVelocityCheckpoint::AfterBoth,
    ] {
        for direction in [8, 2, 10] {
            let iteration = SpotlightIteration::closing(SpotlightIterationPhase::WholeTable);
            let mut atomic = configured_state(direction);
            atomic.module0f_spotlight_close_link_and_oam();

            let mut resumed = configured_state(direction);
            resumed.original_timing_owner = OriginalTimingOwnerState::Live;
            resumed.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
                179_604,
                0,
                vec![OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    match checkpoint {
                        LinkActualVelocityCheckpoint::Clearing { completed } => {
                            crate::MainLoopInterruption::LinkVelocityClearProgress { completed }
                        }
                        LinkActualVelocityCheckpoint::AfterBoth => {
                            crate::MainLoopInterruption::LinkActualVelocityCompleted
                        }
                        _ => crate::MainLoopInterruption::LinkActualVelocity {
                            horizontal_resolved: match checkpoint {
                                LinkActualVelocityCheckpoint::BeforeSelection => None,
                                LinkActualVelocityCheckpoint::BeforeX => Some(false),
                                LinkActualVelocityCheckpoint::BeforeY => Some(true),
                                _ => unreachable!(),
                            },
                        },
                    },
                )],
            ));
            let entry_x = resumed.game_state.player.follower_link.x();
            let entry_y = resumed.game_state.player.follower_link.y();

            assert!(resumed.begin_module0f_spotlight_close_link_and_oam(None, iteration));
            assert_eq!(resumed.game_state.player.follower_link.x(), entry_x);
            assert_eq!(resumed.game_state.player.follower_link.y(), entry_y);
            assert_eq!(
                resumed.game_state.player.follower_link.actual_x_velocity(),
                if matches!(
                    checkpoint,
                    LinkActualVelocityCheckpoint::BeforeY | LinkActualVelocityCheckpoint::AfterBoth
                ) {
                    atomic.game_state.player.follower_link.actual_x_velocity()
                } else if checkpoint == (LinkActualVelocityCheckpoint::Clearing { completed: 1 }) {
                    0x43
                } else {
                    0
                },
                "only a source-completed horizontal pass may publish its velocity",
            );
            if let LinkActualVelocityCheckpoint::Clearing { completed } = checkpoint {
                assert_eq!(
                    resumed
                        .game_state
                        .player
                        .follower_link
                        .y_page_movement_delta(),
                    if completed >= 3 { 0 } else { 7 }
                );
                assert_eq!(
                    resumed
                        .game_state
                        .player
                        .follower_link
                        .x_page_movement_delta(),
                    9
                );
            }
            assert_eq!(
                resumed.game_state.player.follower_link.actual_y_velocity(),
                if checkpoint == LinkActualVelocityCheckpoint::AfterBoth {
                    atomic.game_state.player.follower_link.actual_y_velocity()
                } else {
                    0
                },
                "only a completed vertical pass publishes its velocity",
            );
            let Some(GameWorkStep::Complete(
                GameWorkContinuation::FinishDungeonExitSpotlightActualVelocity {
                    velocity_return,
                    iteration,
                },
            )) = resumed
                .game_execution_scheduler
                .advance_work_one_nmi_slice()
            else {
                panic!("actual Link velocity did not retain its typed continuation");
            };
            assert_eq!(
                velocity_return.pending_actual_y,
                (matches!(
                    checkpoint,
                    LinkActualVelocityCheckpoint::BeforeX | LinkActualVelocityCheckpoint::BeforeY
                ) && direction & 12 != 0)
                    .then_some(atomic.game_state.player.follower_link.actual_y_velocity())
            );
            assert_eq!(
                velocity_return.pending_actual_x,
                (checkpoint == LinkActualVelocityCheckpoint::BeforeX && direction & 3 != 0)
                    .then_some(atomic.game_state.player.follower_link.actual_x_velocity())
            );
            resumed.complete_dungeon_exit_spotlight_link_actual_velocity(
                velocity_return,
                iteration,
                false,
            );

            assert_eq!(resumed.ram, atomic.ram);
            assert!(resumed
                .original_timing_semantic_receipts
                .as_ref()
                .is_some_and(|receipts| receipts.semantic().is_empty()));
        }
    }
}

#[test]
fn live_spotlight_entry_return_stops_at_its_exact_link_subpixel_boundary() {
    fn configured_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.set_rom_startup_timing(true);
        state.set_main_module(0x0f);
        state.set_submodule(0);
        state.set_indoor_flag(0);
        state.set_overworld_screen(0x0f);
        state.follower_link_state_mut().set_x(0x0e60);
        state.follower_link_state_mut().set_y(0x0218);
        state
            .follower_link_state_mut()
            .set_direction_and_last_direction(8);
        state.follower_link_state_mut().set_actual_y_velocity(0x1e);
        state.set_bg2_v_copy2(0x0200);
        state.set_spotlight_window_radius(0x77);
        state.set_spotlight_window_state(0);
        state
    }

    let iteration =
        SpotlightIteration::closing(SpotlightIterationPhase::CloseEntryBeforeTablePublication);
    let mut atomic = configured_state();
    atomic.complete_dungeon_exit_spotlight_entry(
        SpotlightTableBuildContinuation::default(),
        iteration,
    );

    let mut resumed = configured_state();
    resumed.original_timing_owner = OriginalTimingOwnerState::Live;
    resumed.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        179_577,
        0,
        vec![OriginalTimingSemanticReceipt::MainLoopInterrupted(
            crate::MainLoopInterruption::LinkPositionAfterSubpixel { pass: 0 },
        )],
    ));
    let entry_y = resumed.game_state.player.follower_link.y();

    resumed.complete_dungeon_exit_spotlight_entry(
        SpotlightTableBuildContinuation::default(),
        iteration,
    );

    assert_eq!(resumed.game_state.frame.submodule, 1);
    assert_eq!(resumed.game_state.player.follower_link.y(), entry_y);
    let Some(GameWorkStep::Complete(
        GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterSubpixel {
            iteration,
            pass,
            pending_pixel_delta,
            old_x,
            old_y,
        },
    )) = resumed
        .game_execution_scheduler
        .advance_work_one_nmi_slice()
    else {
        panic!("entry Link movement did not retain its partial source continuation");
    };
    assert_eq!(pass, 0);
    resumed.complete_dungeon_exit_spotlight_link_movement_after_subpixel(
        iteration,
        LinkMovePositionPartialReturn {
            partial: LinkMovePositionPartial {
                axis: crate::game_state::PlayerAxis::from_rom_pass(pass),
                pending_pixel_delta,
            },
            old_x,
            old_y,
        },
        false,
    );

    assert_eq!(resumed.ram, atomic.ram);
    assert!(resumed
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn live_spotlight_coordinate_low_boundary_preserves_pending_high_byte_carry() {
    fn configured_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.set_rom_startup_timing(true);
        state.set_main_module(0x0f);
        state.set_submodule(1);
        state.set_indoor_flag(0);
        state.set_overworld_screen(0x0f);
        state.follower_link_state_mut().set_x(0x0cf0);
        state.follower_link_state_mut().set_y(0x0200);
        state
            .follower_link_state_mut()
            .set_direction_and_last_direction(8);
        state.follower_link_state_mut().set_actual_y_velocity(0xf8);
        state
    }

    let iteration = SpotlightIteration::closing(SpotlightIterationPhase::WholeTable);
    let mut atomic = configured_state();
    atomic.module0f_spotlight_close_link_and_oam();
    assert_eq!(atomic.game_state.player.follower_link.y(), 0x01fe);

    let mut resumed = configured_state();
    resumed.original_timing_owner = OriginalTimingOwnerState::Live;
    resumed.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        179_583,
        0,
        vec![OriginalTimingSemanticReceipt::MainLoopInterrupted(
            crate::MainLoopInterruption::LinkPositionAfterCoordinateLow { pass: 0 },
        )],
    ));

    assert!(resumed.begin_module0f_spotlight_close_link_and_oam(None, iteration));
    assert_eq!(
        resumed.game_state.player.follower_link.y(),
        0x02fe,
        "the low-byte boundary must retain the old high byte until the source high-byte store",
    );
    let Some(GameWorkStep::Complete(
        GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinateLow {
            iteration,
            pass,
            pending_coordinate_high,
            old_x,
            old_y,
        },
    )) = resumed
        .game_execution_scheduler
        .advance_work_one_nmi_slice()
    else {
        panic!("coordinate-low Link movement did not retain its high-byte continuation");
    };
    assert_eq!(pass, 0);
    assert_eq!(pending_coordinate_high, 0x01);
    resumed.complete_dungeon_exit_spotlight_link_movement_after_coordinate_low(
        iteration,
        LinkMovePositionAfterCoordinateLowReturn {
            old_x,
            old_y,
            axis: crate::game_state::PlayerAxis::from_rom_pass(pass),
            pending_coordinate_high,
        },
        false,
    );

    assert_eq!(resumed.ram, atomic.ram);
    assert!(resumed
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn live_spotlight_post_coordinate_boundary_resumes_only_later_axes() {
    fn configured_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.set_rom_startup_timing(true);
        state.set_main_module(0x0f);
        state.set_submodule(1);
        state.set_indoor_flag(0);
        state.set_overworld_screen(0x0f);
        state.follower_link_state_mut().set_x(0x0cf0);
        state.follower_link_state_mut().set_y(0x0735);
        state
            .follower_link_state_mut()
            .set_direction_and_last_direction(8);
        state.follower_link_state_mut().set_actual_y_velocity(0xf8);
        state
    }

    let iteration = SpotlightIteration::closing(SpotlightIterationPhase::WholeTable);
    let mut atomic = configured_state();
    atomic.module0f_spotlight_close_link_and_oam();

    let mut resumed = configured_state();
    resumed.original_timing_owner = OriginalTimingOwnerState::Live;
    resumed.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        71_903,
        0,
        vec![OriginalTimingSemanticReceipt::MainLoopInterrupted(
            crate::MainLoopInterruption::LinkPositionAfterCoordinates { pass: 2 },
        )],
    ));
    let entry_y = resumed.game_state.player.follower_link.y();
    let entry_y_subpixel = link_test_byte(&resumed, LINK_SUBPIXEL_Y);

    assert!(resumed.begin_module0f_spotlight_close_link_and_oam(None, iteration));
    assert_eq!(resumed.game_state.player.follower_link.y(), entry_y);
    assert_eq!(link_test_byte(&resumed, LINK_SUBPIXEL_Y), entry_y_subpixel);
    assert!(matches!(
        resumed.game_execution_scheduler.current_work(),
        Some(
            GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinates {
                pass: 2,
                ..
            }
        )
    ));

    let Some(GameWorkStep::Complete(
        GameWorkContinuation::FinishDungeonExitSpotlightLinkMovementAfterCoordinates {
            iteration,
            pass,
            old_x,
            old_y,
        },
    )) = resumed
        .game_execution_scheduler
        .advance_work_one_nmi_slice()
    else {
        panic!("post-coordinate Link movement did not resume at the following accepted NMI");
    };
    resumed.complete_dungeon_exit_spotlight_link_movement_after_coordinates(
        iteration,
        LinkMovePositionAfterCoordinatesReturn {
            old_x,
            old_y,
            axis: crate::game_state::PlayerAxis::from_rom_pass(pass),
        },
        false,
    );

    assert_eq!(resumed.ram, atomic.ram);
}

#[test]
fn fresh_overworld_iris_goal_retains_its_partial_reset() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_main_module(0x10);
    state.set_submodule(1);
    state.set_saved_module_for_menu(9);
    state.set_spotlight_window_radius(119);
    state.set_spotlight_window_state(2);
    state.follower_link_state_mut().set_position(120, 140);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    let phase = crate::MainLoopInterruption::SpotlightGoalResetTable {
        completed_stores: 75,
    };
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            1212391,
            0,
            vec![OriginalTimingSemanticReceipt::MainLoopInterrupted(phase)],
        ))
        .unwrap();
    assert_eq!(state.original_timing_main_loop_interruption(), Some(phase));
    state.Module10_SpotlightOpen();
    assert_eq!(state.game_state.frame.main_module, 0x10);
    assert_eq!(state.game_state.frame.submodule, 1);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(
            GameWorkContinuation::FinishOverworldSpotlightGoalResetTable {
                completed_stores: 75,
                ..
            }
        )
    ));
    state.complete_overworld_spotlight_goal_reset_table(75);
    assert_eq!(state.game_state.frame.main_module, 9);
}

#[test]
fn completed_spotlight_entry_does_not_schedule_a_second_return() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(0x0f);
    state.set_submodule(0);
    state.follower_link_state_mut().set_position(100, 584);
    state.set_bg2_v_copy2(360);
    state.set_spotlight_window_state(0);
    state.set_spotlight_window_radius(126);
    let iteration =
        SpotlightIteration::closing(SpotlightIterationPhase::CloseEntryBeforeTablePublication);
    let table = state.begin_iris_spotlight_configure_table(0);
    let mut deferred = state.clone();
    deferred.complete_dungeon_exit_spotlight_entry(table, iteration);
    state.complete_dungeon_exit_spotlight_entry_returned(table, iteration);
    assert_eq!(state.game_state, deferred.game_state);
    assert_eq!(state.ram, deferred.ram);
    assert!(state.game_execution_scheduler.is_idle());
    assert!(matches!(
        deferred.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpotlightIteration { .. })
    ));
}

#[test]
fn dungeon_exit_spotlight_models_measured_circle_and_suffix_boundaries() {
    // 189-row calibration center (dungeon landing): $70 publishes in-slice.
    assert!(rom_dungeon_exit_spotlight_table_needs_entry_slice(0x7e, 36));
    assert!(rom_dungeon_exit_spotlight_table_needs_entry_slice(0x77, 36));
    assert!(!rom_dungeon_exit_spotlight_table_needs_entry_slice(
        0x70, 36
    ));
    // Maximal 239-row table (Link's-house entrance, vertical center 238): the
    // $70 build still crosses vblank.
    assert!(rom_dungeon_exit_spotlight_table_needs_entry_slice(
        0x70, 238
    ));
    assert!(!rom_dungeon_exit_spotlight_table_needs_entry_slice(
        0x69, 238
    ));
    let cpu_plan = |interrupted_pc| DungeonExitSpotlightCpuPlan {
        interrupted_pc,
        interrupted_return_address: 0,
        iterations_before_nmi: 0,
        link_position_integrated_before_first_nmi: false,
        returned_to_main_wait_before_first_nmi: false,
        main_loop_sprite_preparation_completed_before_second_nmi: false,
        active_window_words: [0; SPOTLIGHT_VISIBLE_SCANLINES],
        following_window_words: [0; SPOTLIGHT_VISIBLE_SCANLINES],
        next_entry_earliest: Some(DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_EARLIEST),
        next_entry_latest: Some(DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_LATEST),
        successor_entry_earliest: None,
        successor_entry_latest: None,
    };
    assert!(cpu_plan(0x00_f38d).interrupted_during_table_build_or_copy());
    assert!(!cpu_plan(0x00_f38d).interrupted_during_table_copy());
    assert!(cpu_plan(0x00_f3be).interrupted_during_table_build_or_copy());
    assert!(cpu_plan(0x00_f3be).interrupted_during_table_copy());
    assert!(!cpu_plan(0x00_f3c5).interrupted_during_table_build_or_copy());
    assert_eq!(
        rom_display_snapshot_publication(0x0f, 0),
        DisplaySnapshotPublication::AdvanceStaged
    );
    assert_eq!(
        rom_display_snapshot_publication(0x0f, 1),
        DisplaySnapshotPublication::PublishCaptured
    );
    assert_eq!(
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTable).completion_publication(),
        DisplaySnapshotPublication::RetainPublished
    );
    assert_eq!(
        SpotlightIteration::closing(SpotlightIterationPhase::CloseEntryBeforeTablePublication)
            .completion_publication(),
        DisplaySnapshotPublication::AdvanceStaged
    );
    let entry_with_rom_following_field =
        SpotlightIteration::closing(SpotlightIterationPhase::CloseEntryBeforeTablePublication)
            .with_rom_following_field_after_staged_active([0x00ff; SPOTLIGHT_VISIBLE_SCANLINES]);
    let entry_after_rom_following_field =
        entry_with_rom_following_field.after_rom_following_field_was_staged();
    assert!(entry_after_rom_following_field
        .rom_following_field_receipt()
        .is_none());
    assert_eq!(
        entry_after_rom_following_field.completion_publication(),
        DisplaySnapshotPublication::PublishCaptured
    );
    assert_eq!(
        SpotlightIteration::closing(SpotlightIterationPhase::CloseEntryAfterTablePublication)
            .completion_publication(),
        DisplaySnapshotPublication::PublishCaptured
    );
    assert_eq!(
        SpotlightIteration::closing(SpotlightIterationPhase::MixedTailAfterReturn)
            .completion_publication(),
        DisplaySnapshotPublication::AdvanceStaged
    );
    assert_eq!(
        SpotlightIteration::opening().in_flight_publication(),
        DisplaySnapshotPublication::AdvanceStaged
    );
    assert_eq!(
        SpotlightIteration::game_over_closing(
            SpotlightIterationPhase::CloseEntryBeforeTablePublication,
            true,
        )
        .in_flight_publication(),
        DisplaySnapshotPublication::RetainPublished
    );
    assert_eq!(
        SpotlightIteration::game_over_closing(SpotlightIterationPhase::WholeTable, false)
            .in_flight_publication(),
        DisplaySnapshotPublication::AdvanceStaged
    );
    assert_eq!(
        SpotlightIteration::game_over_closing(SpotlightIterationPhase::WholeTable, false)
            .completion_publication(),
        DisplaySnapshotPublication::AdvanceStaged
    );
    assert!(
        SpotlightIteration::game_over_closing(SpotlightIterationPhase::WholeTable, false)
            .projects_following_table_tail_on_completion()
    );
    assert!(
        SpotlightIteration::game_over_closing(SpotlightIterationPhase::WholeTable, false)
            .projection_uses_published_prefix()
    );
    assert!(
        !SpotlightIteration::game_over_closing(SpotlightIterationPhase::WholeTable, false)
            .after_game_over_build()
            .projects_following_table_tail_on_completion()
    );
    assert!(
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTableAfterTablePublication)
            .publishes_completed_hdma_table_to_active_scanout()
    );
    assert!(
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTable)
            .publishes_completed_hdma_table_to_active_scanout()
    );
    assert!(
        !SpotlightIteration::closing(SpotlightIterationPhase::MixedTailAfterReturn)
            .publishes_completed_hdma_table_to_active_scanout()
    );
    assert!(
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTable)
            .projects_following_table_tail_on_completion()
    );
    assert!(
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTableAfterTablePublication)
            .projects_following_table_tail_on_completion()
    );
    assert!(
        SpotlightIteration::closing(SpotlightIterationPhase::MixedTailAfterReturn)
            .projects_following_table_tail_on_completion()
    );
    assert!(!SpotlightIteration::opening().publishes_completed_hdma_table_to_active_scanout());
    assert!(!rom_display_memory_publication_is_deferred(7, 15, 0, false));
    assert_eq!(
        SpotlightIteration::opening().completion_publication(),
        DisplaySnapshotPublication::AdvanceStaged
    );
    assert_eq!(
        SpotlightIterationPhase::for_close_iteration(1, 0x3f, 0),
        SpotlightIterationPhase::MixedTailAfterReturn
    );
    assert_eq!(
        SpotlightIterationPhase::for_close_iteration(1, 0x3f, 42),
        SpotlightIterationPhase::WholeTableAfterTablePublication
    );
    assert_eq!(
        SpotlightIterationPhase::for_close_iteration(1, 0x3f, 41),
        SpotlightIterationPhase::MixedTailAfterReturn
    );
    assert_eq!(
        SpotlightIterationPhase::for_close_iteration(1, 0x38, 0),
        SpotlightIterationPhase::MixedTailAfterReturn
    );
    assert_eq!(
        SpotlightIterationPhase::for_close_iteration(1, 0x38, 42),
        SpotlightIterationPhase::WholeTableAfterTablePublication
    );
    assert_eq!(
        SpotlightIterationPhase::for_close_iteration(1, 0x07, 0),
        SpotlightIterationPhase::MixedTailAfterReturn
    );
    assert_eq!(
        SpotlightIterationPhase::for_close_iteration(1, 0, 0),
        SpotlightIterationPhase::WholeTable
    );
    assert_eq!(DUNGEON_EXIT_SPOTLIGHT_GOAL_CALLER_NMI_SLICES, 2);
    assert_eq!(TRAILING_NMI_FORCE_BLANK_SCANLINE, 224);
    assert_eq!(
        rom_graphics_dma_plan(6, 0).oam_scanout,
        OamScanoutSource::ComposeLiveAfterNmi,
    );
    assert_eq!(
        rom_graphics_dma_plan(6, 0).link_obj_scanout,
        GraphicsDmaGeneration::LiveAfterMain,
    );
    assert_eq!(spotlight_mixed_scanout_live_tail_start(36, 0x38), 221);
    assert_eq!(spotlight_mixed_scanout_live_tail_start(238, 0x38), 224);
    assert_eq!(spotlight_mixed_scanout_live_tail_start(238, 0x31), 221);

    let closing_iteration = SpotlightIteration::closing(SpotlightIterationPhase::WholeTable);
    let mut work = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishSpotlightIteration {
            iteration: closing_iteration,
        },
        SPOTLIGHT_ITERATION_SUFFIX_NMI_SLICES,
    );
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(GameWorkContinuation::FinishSpotlightIteration {
            iteration: closing_iteration,
        })
    );
    let goal_iteration =
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTableAfterTablePublication);
    let goal_continuation = GameWorkContinuation::FinishDungeonExitSpotlightGoalCaller {
        iteration: goal_iteration,
    };
    let mut goal_work = ScheduledGameWork::schedule(
        goal_continuation,
        DUNGEON_EXIT_SPOTLIGHT_GOAL_CALLER_NMI_SLICES,
    );
    assert!(goal_work.suspends_translated_call_stack());
    assert_eq!(goal_work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    assert_eq!(
        goal_work.advance_one_nmi_slice(),
        GameWorkStep::Complete(goal_continuation)
    );
    assert!(rom_dungeon_landing_goal_transition_waits_for_caller_return(
        7, 15
    ));
    assert!(!rom_dungeon_landing_goal_transition_waits_for_caller_return(16, 1));
}

#[test]
fn interrupted_spotlight_suffix_matches_c_writes_and_stages_the_rom_receipt() {
    let mut state = ZeldaState::new();
    let measured = std::array::from_fn(|row| 0x00ff_u16.wrapping_add(row as u16));
    let ram_before = state.ram.clone();

    state.spotlight_internal_after_table_during_active_rom_field(&measured);

    // zelda3/src/load_gfx.c:SpotlightInternal writes exactly these two RAM
    // mirrors after IrisSpotlight_ConfigureTable returns. Display-generation
    // provenance is hardware state and must not manufacture another RAM write.
    let changed_ram = ram_before
        .iter()
        .zip(&state.ram)
        .enumerate()
        .filter_map(|(address, (&before, &after))| (before != after).then_some((address, after)))
        .collect::<Vec<_>>();
    assert_eq!(
        changed_ram,
        vec![
            (crate::game_state::constants::INIDISP_COPY, 0x0f),
            (crate::game_state::constants::HDMAEN_COPY, 0x80),
        ]
    );

    let staged = state
        .spotlight_scanout_after_active_field
        .as_ref()
        .expect("ROM following field");
    assert!(staged.authoritative_rom_hdma_receipt);
    for (row, word) in measured.into_iter().enumerate() {
        let offset = row * 2;
        assert_eq!(
            u16::from_le_bytes(
                staged.hdma_tables[0][offset..offset + 2]
                    .try_into()
                    .unwrap()
            ),
            word
        );
    }
}

#[test]
fn spotlight_table_completion_defers_control_clears_and_link_movement() {
    fn configured() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.set_main_module(0x0f);
        state.set_submodule(1);
        state.set_indoor_flag(0);
        state.follower_link_state_mut().set_x(1272);
        state.follower_link_state_mut().set_y(1012);
        state
            .follower_link_state_mut()
            .set_direction_and_last_direction(8);
        state.set_bg2_h_copy2(1152);
        state.set_bg2_v_copy2(786);
        state.set_spotlight_window_radius(126);
        state.set_spotlight_window_state(0);
        state.activate_nmi_thread();
        state.request_polyhedral_nmi_update();
        state
    }
    let mut atomic = configured();
    assert_eq!(
        atomic.spotlight_configure_table_and_control(false),
        (false, false)
    );
    atomic.module0f_spotlight_close_link_and_oam();

    let mut resumed = configured();
    let build = resumed.begin_iris_spotlight_configure_table_at_progress(
        crate::SpotlightTableBuildProgress {
            completed_iterations: 0,
            checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeIterationInitialization,
        },
    );
    resumed.complete_dungeon_exit_spotlight_build_until_control(
        build,
        false,
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTable),
    );
    assert_eq!(resumed.ram[0x12a], 1);
    assert_eq!(resumed.ram[0x1f0c], 0xff);
    assert_eq!(resumed.game_state.player.follower_link.y(), 1012);
    assert_eq!(
        resumed.game_state.display.spotlight_hdma.window_radius(),
        119
    );
    assert!(matches!(
        resumed.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonExitSpotlightControl { .. })
    ));
    resumed.complete_dungeon_exit_spotlight_control();
    let Some(GameWorkStep::Complete(GameWorkContinuation::FinishDungeonExitSpotlightControl {
        iteration,
    })) = resumed
        .game_execution_scheduler
        .advance_work_one_nmi_slice()
    else {
        panic!("control suffix did not retain its continuation");
    };
    resumed.complete_dungeon_exit_spotlight_link_and_oam(iteration, true);
    assert_eq!(resumed.game_execution_scheduler.current_work(), None);
    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(resumed.game_state, atomic.game_state);
}

#[test]
fn semantic_spotlight_circle_checkpoint_resumes_to_the_atomic_c_endpoint() {
    fn source_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.follower_link_state_mut().set_x(1272);
        state.follower_link_state_mut().set_y(1012);
        state.set_bg2_h_copy2(1152);
        state.set_bg2_v_copy2(786);
        state.set_spotlight_window_radius(126);
        state.set_spotlight_window_state(0);
        state
    }

    let mut atomic = source_state();
    assert!(!atomic.iris_spotlight_configure_table());

    let mut resumed = source_state();
    let build = resumed.begin_iris_spotlight_configure_table_at_progress(
        crate::SpotlightTableBuildProgress {
            completed_iterations: 209,
            checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                pending_circle_input: 30,
            },
        },
    );
    assert_eq!(
        resumed.game_state.display.spotlight_hdma.window_y_buffer(),
        29,
        "the interrupted C iteration decrements spotlight_var4 before its circle call",
    );
    assert!(!resumed.complete_iris_spotlight_configure_table(build));

    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(resumed.game_state, atomic.game_state);
}

#[test]
fn semantic_spotlight_iteration_start_checkpoint_resumes_to_the_atomic_c_endpoint() {
    fn source_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.follower_link_state_mut().set_x(1656);
        state.follower_link_state_mut().set_y(8209);
        state.set_bg2_h_copy2(1536);
        state.set_bg2_v_copy2(8191);
        state.set_spotlight_window_radius(126);
        state.set_spotlight_window_state(2);
        state
    }

    let mut atomic = source_state();
    let atomic_completed = atomic.iris_spotlight_configure_table();

    let mut resumed = source_state();
    let build = resumed.begin_iris_spotlight_configure_table_at_progress(
        crate::SpotlightTableBuildProgress {
            completed_iterations: 175,
            checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeIterationInitialization,
        },
    );
    assert_eq!(
        resumed.game_state.display.spotlight_hdma.window_y_buffer(),
        20,
        "175 completed C iterations leave the next circle input unconsumed",
    );
    let resumed_completed = resumed.complete_iris_spotlight_configure_table(build);

    assert_eq!(resumed_completed, atomic_completed);
    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(resumed.game_state, atomic.game_state);
}

#[test]
fn semantic_spotlight_loop_test_checkpoint_resumes_to_the_atomic_c_endpoint() {
    fn source_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.follower_link_state_mut().set_x(824);
        state.follower_link_state_mut().set_y(2168);
        state.set_bg2_h_copy2(690);
        state.set_bg2_v_copy2(2071);
        state.set_spotlight_window_radius(126);
        state.set_spotlight_window_state(0);
        state
    }

    let mut atomic = source_state();
    let atomic_completed = atomic.iris_spotlight_configure_table();

    let mut resumed = source_state();
    let build = resumed.begin_iris_spotlight_configure_table_at_progress(
        crate::SpotlightTableBuildProgress {
            completed_iterations: 113,
            checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeLoopCompletionTest {
                upper_cursor: 107,
                lower_cursor: 111,
            },
        },
    );
    let resumed_completed = resumed.complete_iris_spotlight_configure_table(build);

    assert_eq!(resumed_completed, atomic_completed);
    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(resumed.game_state, atomic.game_state);
}

#[test]
fn semantic_spotlight_loop_test_with_offscreen_lower_row_resumes_to_c_endpoint() {
    fn source_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.follower_link_state_mut().set_x(1272);
        state.follower_link_state_mut().set_y(1012);
        state.set_bg2_h_copy2(1152);
        state.set_bg2_v_copy2(786);
        state.set_spotlight_window_radius(112);
        state.set_spotlight_window_state(0);
        state
    }

    let mut atomic = source_state();
    let atomic_completed = atomic.iris_spotlight_configure_table();

    let mut resumed = source_state();
    let build = resumed.begin_iris_spotlight_configure_table_at_progress(
        crate::SpotlightTableBuildProgress {
            completed_iterations: 217,
            checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeLoopCompletionTest {
                upper_cursor: 217,
                lower_cursor: 259,
            },
        },
    );
    let resumed_completed = resumed.complete_iris_spotlight_configure_table(build);

    assert_eq!(resumed_completed, atomic_completed);
    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(resumed.game_state, atomic.game_state);
}

#[test]
fn semantic_spotlight_lower_cursor_checkpoint_resumes_to_the_atomic_c_endpoint() {
    fn source_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.follower_link_state_mut().set_x(8056);
        state.follower_link_state_mut().set_y(9204);
        state.set_bg2_h_copy2(7936);
        state.set_bg2_v_copy2(8978);
        state.set_spotlight_window_radius(126);
        state.set_spotlight_window_state(2);
        state
    }

    let mut atomic = source_state();
    let atomic_completed = atomic.iris_spotlight_configure_table();

    let mut resumed = source_state();
    let build = resumed.begin_iris_spotlight_configure_table_at_progress(
        crate::SpotlightTableBuildProgress {
            completed_iterations: 189,
            checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeLowerCursorDecrement {
                upper_cursor: 190,
                lower_cursor: 287,
            },
        },
    );
    let resumed_completed = resumed.complete_iris_spotlight_configure_table(build);

    assert_eq!(resumed_completed, atomic_completed);
    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(resumed.game_state, atomic.game_state);
}

#[test]
fn semantic_spotlight_lower_write_checkpoint_resumes_without_replaying_the_upper_write() {
    fn source_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.follower_link_state_mut().set_x(1880);
        state.follower_link_state_mut().set_y(1044);
        state.set_bg2_h_copy2(1758);
        state.set_bg2_v_copy2(1024);
        state.set_spotlight_window_radius(119);
        state.set_spotlight_window_state(2);
        state
    }

    let mut atomic = source_state();
    assert!(atomic.iris_spotlight_configure_table());

    let mut resumed = source_state();
    resumed.set_spotlight_hdma_table_dynamic_entry(39, 0x1234);
    let build = resumed.begin_iris_spotlight_configure_table_at_progress(
        crate::SpotlightTableBuildProgress {
            completed_iterations: 185,
            checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeLowerTableWrite {
                lower_cursor: 39,
                circle_value: 0xff00,
            },
        },
    );
    assert_eq!(
        resumed
            .game_state
            .display
            .spotlight_hdma
            .hdma_table_dynamic_entry(25),
        0xff00,
        "the source upper write is already published at this checkpoint",
    );
    assert_eq!(
        resumed
            .game_state
            .display
            .spotlight_hdma
            .hdma_table_dynamic_entry(39),
        0x1234,
        "the interrupted lower write must remain pending",
    );
    assert!(resumed.complete_iris_spotlight_configure_table(build));

    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(resumed.game_state, atomic.game_state);
}

#[test]
fn semantic_spotlight_projection_checkpoint_resumes_without_replaying_its_prefix() {
    fn source_state() -> ZeldaState {
        let mut state = ZeldaState::new();
        state.follower_link_state_mut().set_x(680);
        state.follower_link_state_mut().set_y(1704);
        state.set_bg2_h_copy2(554);
        state.set_bg2_v_copy2(1610);
        state.set_spotlight_window_radius(126);
        state.set_spotlight_window_state(0);
        for index in 0..224 {
            write_le_u16(&mut state.ram, RESERVED_HDMA_TABLE + index * 2, 0x5a5a);
        }
        state
    }

    let mut atomic = source_state();
    assert!(!atomic.iris_spotlight_configure_table());

    let mut resumed = source_state();
    let build = resumed.begin_iris_spotlight_configure_table_at_progress(
        crate::SpotlightTableBuildProgress {
            completed_iterations: 119,
            checkpoint: crate::SpotlightTableBuildCheckpoint::ProjectionCopy { copied_words: 157 },
        },
    );
    assert_eq!(
        read_le_u16(&resumed.ram, RESERVED_HDMA_TABLE + 156 * 2),
        resumed.spotlight_hdma_table_dynamic_entry(156),
        "the last source-published projection word is visible at the checkpoint",
    );
    assert_eq!(
        read_le_u16(&resumed.ram, RESERVED_HDMA_TABLE + 157 * 2),
        0x5a5a,
        "the first unexecuted projection word remains pending",
    );
    assert!(!resumed.complete_iris_spotlight_configure_table(build));

    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(resumed.game_state, atomic.game_state);
}

#[test]
fn authoritative_recurring_spotlight_return_finishes_the_existing_c_caller_once() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_indoor_flag(0);
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.follower_link_state_mut().set_x(680);
    state.follower_link_state_mut().set_y(1703);
    state
        .follower_link_state_mut()
        .set_direction_and_last_direction(8);
    state.set_bg2_h_copy2(554);
    state.set_bg2_v_copy2(1610);
    state.set_spotlight_window_radius(112);
    state.set_spotlight_window_state(0);
    write_le_u16(&mut state.ram, LINK_DMA_COUNTDOWN, 9);
    state.set_bg_tile_animation_countdown(7);
    let iteration = SpotlightIteration::closing(SpotlightIterationPhase::WholeTable);

    assert!(state.begin_dungeon_exit_spotlight_build(
        None,
        Some(crate::SpotlightTableBuildProgress {
            completed_iterations: 105,
            checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                pending_circle_input: 15,
            },
        }),
        iteration,
    ));
    assert!(!state
        .game_execution_scheduler
        .spotlight_iteration()
        .expect("table continuation must retain its iteration")
        .prepares_main_loop_sprites_before_second_nmi());

    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait],
    ));
    let caller_returned = state.take_original_timing_dungeon_exit_spotlight_caller_returned();
    assert!(caller_returned);
    let step = state
        .game_execution_scheduler
        .advance_work_one_nmi_slice_with_authoritative_completion(caller_returned);
    let Some(GameWorkStep::Complete(GameWorkContinuation::FinishDungeonExitSpotlightBuild {
        table_build,
        projection_completed,
        iteration,
    })) = step
    else {
        panic!("authoritative caller return did not complete its existing table continuation");
    };
    state.complete_dungeon_exit_spotlight_build(
        table_build,
        projection_completed,
        iteration,
        caller_returned,
        false,
    );

    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.player.follower_link.y(), 1702);
    assert_eq!(state.game_state.player.follower_link.y_velocity(), 0);
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 105);
    assert_eq!(read_le_u16(&state.ram, LINK_DMA_COUNTDOWN), 8);
    assert_eq!(state.game_state.display.bg_tile_animation_countdown, 6);
}

#[test]
fn authoritative_recurring_spotlight_link_oam_receipt_retains_only_caller_suffix() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_indoor_flag(0);
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.follower_link_state_mut().set_x(680);
    state.follower_link_state_mut().set_y(1703);
    state
        .follower_link_state_mut()
        .set_direction_and_last_direction(8);
    state.set_bg2_h_copy2(554);
    state.set_bg2_v_copy2(1610);
    state.set_spotlight_window_radius(112);
    state.set_spotlight_window_state(0);
    write_le_u16(&mut state.ram, LINK_DMA_COUNTDOWN, 9);
    state.set_bg_tile_animation_countdown(7);
    let iteration = SpotlightIteration::closing(SpotlightIterationPhase::WholeTable);

    assert!(state.begin_dungeon_exit_spotlight_build(
        None,
        Some(crate::SpotlightTableBuildProgress {
            completed_iterations: 105,
            checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                pending_circle_input: 15,
            },
        }),
        iteration,
    ));
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::MainLoopInterrupted(
            crate::MainLoopInterruption::LinkOam,
        )],
    ));
    let interrupted =
        state.take_original_timing_main_loop_interruption(crate::MainLoopInterruption::LinkOam);
    assert!(interrupted);
    let step = state
        .game_execution_scheduler
        .advance_work_one_nmi_slice_with_authoritative_completion(interrupted);
    let Some(GameWorkStep::Complete(GameWorkContinuation::FinishDungeonExitSpotlightBuild {
        table_build,
        projection_completed,
        iteration,
    })) = step
    else {
        panic!("LinkOam receipt did not complete the interrupted table continuation");
    };
    state.complete_dungeon_exit_spotlight_build(
        table_build,
        projection_completed,
        iteration,
        false,
        true,
    );

    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonExitSpotlightLinkOam {
            iteration: pending,
        }) if pending == iteration
    ));
    assert_eq!(state.game_state.player.follower_link.y(), 1702);
    assert_eq!(state.game_state.player.follower_link.y_velocity(), 0);
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 105);
    assert_eq!(read_le_u16(&state.ram, LINK_DMA_COUNTDOWN), 9);
    assert_eq!(state.game_state.display.bg_tile_animation_countdown, 7);
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn authoritative_recurring_spotlight_link_oam_then_return_finishes_caller_once() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_indoor_flag(0);
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.follower_link_state_mut().set_x(680);
    state.follower_link_state_mut().set_y(1703);
    state
        .follower_link_state_mut()
        .set_direction_and_last_direction(8);
    state.set_bg2_h_copy2(554);
    state.set_bg2_v_copy2(1610);
    state.set_spotlight_window_radius(112);
    state.set_spotlight_window_state(0);
    write_le_u16(&mut state.ram, LINK_DMA_COUNTDOWN, 9);
    state.set_bg_tile_animation_countdown(7);

    assert!(state.begin_dungeon_exit_spotlight_build(
        None,
        Some(crate::SpotlightTableBuildProgress {
            completed_iterations: 105,
            checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                pending_circle_input: 15,
            },
        }),
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTable),
    ));
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
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
            OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
        ],
    ));

    let timeline = state
        .take_original_timing_main_loop_interruption_timeline(Some(
            crate::MainLoopInterruption::LinkOam,
        ))
        .expect("source host must retain its ordered LinkOam interruption");
    assert_eq!(
        state.take_original_timing_scheduled_caller_progress(Some(&timeline), None),
        Some(crate::MainLoopProgress::CallStackContinued),
    );
    let caller_returned = state.take_original_timing_dungeon_exit_spotlight_caller_returned();
    assert!(caller_returned);
    assert!(!spotlight_caller_remains_interrupted_in_link_oam(
        caller_returned,
        true,
    ));
    let step = state
        .game_execution_scheduler
        .advance_work_one_nmi_slice_with_authoritative_completion(caller_returned);
    let Some(GameWorkStep::Complete(GameWorkContinuation::FinishDungeonExitSpotlightBuild {
        table_build,
        projection_completed,
        iteration,
    })) = step
    else {
        panic!("later caller return did not complete the interrupted table continuation");
    };
    state.complete_dungeon_exit_spotlight_build(
        table_build,
        projection_completed,
        iteration,
        caller_returned,
        false,
    );

    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.player.follower_link.y(), 1702);
    assert_eq!(state.game_state.player.follower_link.y_velocity(), 0);
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 105);
    assert_eq!(read_le_u16(&state.ram, LINK_DMA_COUNTDOWN), 8);
    assert_eq!(state.game_state.display.bg_tile_animation_countdown, 6);
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn authoritative_spotlight_return_then_new_iteration_runs_both_in_source_order() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_subsubmodule(0);
    state.set_saved_module_for_menu(8);
    state.set_indoor_flag(0);
    state.follower_link_state_mut().set_x(680);
    state.follower_link_state_mut().set_y(1703);
    state.set_bg2_h_copy2(554);
    state.set_bg2_v_copy2(1610);
    state.set_spotlight_window_radius(7);
    state.set_spotlight_window_state(0);
    state.set_frame_counter(0xba);
    state.set_animated_tile_data_source_address(1);
    state.project_native_game_state_to_ram();
    let prior_iteration =
        SpotlightIteration::closing(SpotlightIterationPhase::MixedTailAfterReturn);
    state.schedule_spotlight_iteration_return(prior_iteration);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::IterationStarted,
        )],
    ));
    assert_eq!(state.game_state.frame.frame_counter, 0xba);
    assert_eq!(state.ram[crate::game_state::constants::FRAME_COUNTER], 0xba);
    assert!(state.initialized);

    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    // The fresh source iteration proves the old suspended caller returned
    // first. It must not be discarded merely because both events occurred in
    // one host call: the final close iteration ticks once, reaches radius zero,
    // and restores the saved overworld module.
    assert_eq!(state.game_state.frame.frame_counter, 0xbb);
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 0);
    assert_eq!(state.game_state.frame.main_module, 8);
    assert_eq!(state.game_state.frame.submodule, 0);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpotlightIteration { .. })
    ));
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn authoritative_spotlight_return_then_interrupted_iteration_runs_fresh_main_once() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_subsubmodule(0);
    state.set_saved_module_for_menu(8);
    state.set_indoor_flag(0);
    state.follower_link_state_mut().set_x(680);
    state.follower_link_state_mut().set_y(1703);
    state.set_bg2_h_copy2(554);
    state.set_bg2_v_copy2(1610);
    state.set_spotlight_window_radius(14);
    state.set_spotlight_window_state(0);
    state.set_frame_counter(0x79);
    state.set_animated_tile_data_source_address(1);
    write_le_u16(&mut state.ram, LINK_DMA_COUNTDOWN, 3);
    state.set_bg_tile_animation_countdown(3);
    state.project_native_game_state_to_ram();
    state.schedule_spotlight_iteration_return(SpotlightIteration::closing(
        SpotlightIterationPhase::MixedTailAfterReturn,
    ));
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::Open, NmiUpdateGate::LatchHeld];
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
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                crate::MainLoopInterruption::LinkOam,
            ),
        ],
    ));

    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    // The old suspended call completes before the source begins exactly one
    // fresh ZeldaRunGameLoop iteration. That new call reaches LinkOam and owns
    // the only replacement continuation; replaying the iteration would tick
    // FRAME_COUNTER twice and collide with this continuation.
    assert_eq!(state.game_state.frame.frame_counter, 0x7a);
    assert_eq!(state.ram[crate::game_state::constants::FRAME_COUNTER], 0x7a);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpotlightIteration { .. })
    ));
    assert!(state.original_timing_nmi_publication_pending);
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn authoritative_recurring_spotlight_preserves_leading_and_trailing_nmi_order() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_subsubmodule(0);
    state.set_saved_module_for_menu(8);
    state.set_indoor_flag(0);
    state.follower_link_state_mut().set_x(680);
    state.follower_link_state_mut().set_y(1703);
    state.set_bg2_h_copy2(554);
    state.set_bg2_v_copy2(1610);
    state.set_spotlight_window_radius(14);
    state.set_spotlight_window_state(0);
    state.set_frame_counter(0x79);
    state.set_animated_tile_data_source_address(1);
    write_le_u16(&mut state.ram, LINK_DMA_COUNTDOWN, 3);
    state.set_bg_tile_animation_countdown(3);
    state.sync_native_game_state_from_ram();
    assert_eq!(state.game_state.frame.frame_counter, 0x79);
    assert_eq!(state.ram[crate::game_state::constants::FRAME_COUNTER], 0x79);
    assert_eq!(state.game_state.frame.main_module, 0x0f);
    assert_eq!(state.ram[crate::game_state::constants::MAIN_MODULE], 0x0f);
    assert_eq!(state.game_state.frame.submodule, 1);
    assert_eq!(state.ram[crate::game_state::constants::SUBMODULE], 1);
    state.display_snapshot = None;
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
    let mut accepts_next_nmi_at_interruption = state.clone();
    let mut uninterrupted_iteration = state.clone();

    // The source host first accepts the NMI that returned the preceding caller
    // to Zelda's wait loop, then begins a new Module0F iteration and returns
    // while that iteration is suspended in LinkOam. There is no trailing NMI.
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
                crate::MainLoopInterruption::LinkOam,
            ),
        ],
    ));
    let mut run_poly_only = state.clone();
    run_poly_only.run_frame_internal_after_original_timing(0, crate::RUN_POLY);
    assert!(matches!(
        run_poly_only.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpotlightIteration { .. })
    ));
    assert!(run_poly_only
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.frame_counter, 0x7a);
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 7);
    // LinkOam interrupted the C call before ZeldaRunGameLoop could reach
    // NMI_PrepareSprites, so neither source countdown advances on this host.
    assert_eq!(read_le_u16(&state.ram, LINK_DMA_COUNTDOWN), 3);
    assert_eq!(state.game_state.display.bg_tile_animation_countdown, 3);
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpotlightIteration { .. })
    ));

    // A host can also complete its leading handler, begin the fresh main-loop
    // iteration, and accept the following NMI without hitting a separately
    // reported semantic interruption point. The final publication remains pending.
    uninterrupted_iteration.original_timing_semantic_receipts =
        Some(OriginalTimingHostReceipts::new(
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
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            ],
        ));
    uninterrupted_iteration.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::Open, NmiUpdateGate::LatchHeld];
    uninterrupted_iteration.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);
    assert!(uninterrupted_iteration.original_timing_nmi_publication_pending);
    assert!(uninterrupted_iteration
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));

    // The same fresh source iteration can span far enough to accept the next
    // NMI at the LinkOam interruption after completing its leading handler.
    // The accepted NMI publication belongs to the following host interval.
    accepts_next_nmi_at_interruption.original_timing_semantic_receipts =
        Some(OriginalTimingHostReceipts::new(
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
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    crate::MainLoopInterruption::LinkOam,
                ),
            ],
        ));
    accepts_next_nmi_at_interruption.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::Open, NmiUpdateGate::LatchHeld];
    accepts_next_nmi_at_interruption.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);
    assert!(accepts_next_nmi_at_interruption.original_timing_nmi_publication_pending);
    assert!(accepts_next_nmi_at_interruption
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));

    accepts_next_nmi_at_interruption.original_timing_semantic_receipts =
        Some(OriginalTimingHostReceipts::new(
            1,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
            ],
        ));
    accepts_next_nmi_at_interruption.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);
    assert!(!accepts_next_nmi_at_interruption.original_timing_nmi_publication_pending);

    // The next source host accepts the NMI which interrupts LinkOam while the
    // latch is still set, resumes the saved caller through sprite preparation,
    // then accepts the ordinary trailing NMI after the latch is cleared.
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        1,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                crate::MainLoopInterruption::LinkOam,
            ),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
        ],
    ));
    state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::Open];
    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.frame_counter, 0x7a);
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 7);
    // The resumed caller reaches NMI_PrepareSprites exactly once after the
    // held NMI and before the ordinary trailing NMI.
    assert_eq!(read_le_u16(&state.ram, LINK_DMA_COUNTDOWN), 2);
    assert_eq!(state.game_state.display.bg_tile_animation_countdown, 2);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_nmi_publication_pending);
    assert!(state.game_execution_scheduler.is_idle());
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn live_recurring_overworld_spotlight_preserves_a_same_host_fresh_iteration() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_main_module(0x10);
    state.set_submodule(1);
    state.set_saved_module_for_menu(9);
    state.set_indoor_flag(0);
    state.follower_link_state_mut().set_x(1880);
    state.follower_link_state_mut().set_y(1044);
    state.follower_link_state_mut().set_facing(2);
    state.set_bg2_h_copy2(1758);
    state.set_bg2_v_copy2(1024);
    state.set_spotlight_window_radius(112);
    state.set_spotlight_window_state(2);
    state.set_frame_counter(0xd4);
    state.set_animated_tile_data_source_address(1);
    state.sync_native_game_state_from_ram();
    state.latch_nmi_update();
    state.original_timing_owner = OriginalTimingOwnerState::Live;

    // Begin from a real suspended C table build whose loop has completed but
    // whose projection, radius write, and caller suffix remain pending.
    let table_build = state.begin_iris_spotlight_configure_table(usize::MAX);
    state.schedule_overworld_spotlight_build(
        table_build,
        OverworldSpotlightBuildPhase::Recurring,
        false,
        SpotlightIteration::opening(),
    );
    assert_eq!(state.game_state.frame.frame_counter, 0xd4);
    assert_eq!(state.ram[crate::game_state::constants::FRAME_COUNTER], 0xd4);

    // The source completes that saved call, enters ZeldaRunGameLoop again in
    // the same host interval, then accepts NMI while the next table build is
    // suspended. The new progress receipt must remain available to Module10;
    // it does not belong to the completed prior call.
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                crate::SpotlightTableBuildProgressReceipt {
                    progress: crate::SpotlightTableBuildProgress {
                        completed_iterations: 185,
                        checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeLowerTableWrite {
                            lower_cursor: 39,
                            circle_value: 0xff00,
                        },
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
        ],
    ));
    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.frame_counter, 0xd5);
    assert_eq!(state.game_state.frame.main_module, 0x10);
    assert_eq!(state.game_state.frame.submodule, 1);
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 119);
    assert!(state.original_timing_nmi_publication_pending);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishOverworldSpotlightBuild { .. })
    ));
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));

    // The following host publishes the accepted handler, resumes the second
    // build through the goal transition, then accepts the ordinary next NMI.
    // No additional ZeldaRunGameLoop entry occurs, so the frame counter moves
    // exactly once across the pair.
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        1,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::OverworldSpotlightGoalCallerReturned,
        ],
    ));
    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.frame_counter, 0xd5);
    assert_eq!(state.game_state.frame.main_module, 9);
    assert_eq!(state.game_state.frame.submodule, 10);
    assert_eq!(state.game_state.frame.subsubmodule, 0);
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 126);
    assert!(state.original_timing_nmi_publication_pending);
    assert!(state.game_execution_scheduler.is_idle());
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn live_overworld_spotlight_link_oam_receipt_owns_the_atomic_leaf_once() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_main_module(0x10);
    state.set_submodule(1);
    state.set_saved_module_for_menu(9);
    state.set_indoor_flag(0);
    state.follower_link_state_mut().set_x(1880);
    state.follower_link_state_mut().set_y(1044);
    state.follower_link_state_mut().set_facing(2);
    state.set_bg2_h_copy2(1758);
    state.set_bg2_v_copy2(1024);
    state.set_spotlight_window_radius(7);
    state.set_spotlight_window_state(2);
    state.set_frame_counter(0x11);
    state.set_animated_tile_data_source_address(1);
    state.sync_native_game_state_from_ram();
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
                crate::MainLoopInterruption::LinkOam,
            ),
        ],
    ));

    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.frame_counter, 0x12);
    assert_eq!(state.ram[crate::game_state::constants::FRAME_COUNTER], 0x12);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpotlightIteration { .. })
    ));
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn live_recurring_overworld_spotlight_continuation_completes_a_non_goal_build() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_main_module(0x10);
    state.set_submodule(1);
    state.set_saved_module_for_menu(9);
    state.set_indoor_flag(0);
    state.follower_link_state_mut().set_x(1880);
    state.follower_link_state_mut().set_y(1044);
    state.set_bg2_h_copy2(1758);
    state.set_bg2_v_copy2(1024);
    state.set_spotlight_window_radius(112);
    state.set_spotlight_window_state(2);
    state.set_frame_counter(0xd4);
    state.set_animated_tile_data_source_address(1);
    state.sync_native_game_state_from_ram();
    state.latch_nmi_update();
    state.original_timing_owner = OriginalTimingOwnerState::Live;

    let table_build = state.begin_iris_spotlight_configure_table(usize::MAX);
    state.schedule_overworld_spotlight_build(
        table_build,
        OverworldSpotlightBuildPhase::Recurring,
        false,
        SpotlightIteration::opening(),
    );
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
        ],
    ));

    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.frame_counter, 0xd4);
    assert_eq!(state.game_state.frame.main_module, 0x10);
    assert_eq!(state.game_state.frame.submodule, 1);
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 119);
    assert!(!state.original_timing_nmi_publication_pending);
    assert!(state.game_execution_scheduler.is_idle());
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn live_overworld_spotlight_goal_receipt_matches_native_c_endpoint() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_main_module(16);
    state.set_submodule(1);
    state.set_saved_module_for_menu(9);
    state.follower_link_state_mut().set_facing(2);
    state.set_spotlight_window_state(2);
    state.set_spotlight_window_radius(0x77);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        7309,
        0,
        vec![OriginalTimingSemanticReceipt::OverworldSpotlightGoalCallerReturned],
    ));

    state.Module10_SpotlightOpen();

    // Pinned C completes IrisSpotlight_ConfigureTable, restores module 9, and
    // then OpenSpotlight_Next2 selects submodule 10 from Link's facing.
    assert_eq!(state.game_state.frame.main_module, 9);
    assert_eq!(state.game_state.frame.submodule, 10);
    assert_eq!(state.game_state.frame.subsubmodule, 0);
    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        0x7e
    );
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn spotlight_reset_field_is_published_only_when_the_caller_crosses_one_nmi() {
    let mut one_crossing_rows = [Some(true); SPOTLIGHT_VISIBLE_SCANLINES];
    one_crossing_rows[..4].fill(Some(false));

    // The pinned Snes9x frame-2328 trace retains rows 0..3 from the final C
    // circle and consumes IrisSpotlight_ResetTable below it. Its translated
    // caller crosses exactly one NMI before returning.
    assert_eq!(
        spotlight_reset_prefix_scanlines(&one_crossing_rows, 1),
        Some(4)
    );

    let mut two_crossing_rows = [Some(true); SPOTLIGHT_VISIBLE_SCANLINES];
    two_crossing_rows[21..32].fill(Some(false));
    two_crossing_rows[53..64].fill(Some(false));

    // At frame 11,595 the same C reset work crosses two NMIs. Snes9x presents
    // the final circle through 11,596 and the fully reset following field at
    // 11,597; the discarded in-flight store pattern owns neither publication.
    assert_eq!(
        spotlight_reset_prefix_scanlines(&two_crossing_rows, 2),
        None
    );
}

#[test]
fn spotlight_reset_prefix_expires_with_its_cpu_advance() {
    let mut state = ZeldaState::new();
    let advance = DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInSubmodule,
        resumed_phase: Some(ModuleCpuPhase::CompleteBeforeNmi),
        submodule_nmi_slices: 1,
        subsubmodule: 1,
        palette_countdown: 0,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    };
    state.dungeon_landing_cpu_advance_pending = Some(advance);
    state.dungeon_landing_spotlight_reset_prefix_scanlines = Some(4);

    assert_eq!(state.take_dungeon_landing_cpu_advance(), Some(advance));
    assert_eq!(
        state.active_dungeon_landing_spotlight_reset_prefix_scanlines,
        Some(4)
    );
    assert_eq!(state.dungeon_landing_spotlight_reset_prefix_scanlines, None);

    // The next two-NMI timing result has no presented reset field. Consuming
    // that exact continuation must replace, not retain, the earlier prefix.
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        submodule_nmi_slices: 2,
        ..advance
    });
    state.dungeon_landing_spotlight_reset_prefix_scanlines = None;
    state.take_dungeon_landing_cpu_advance();
    assert_eq!(
        state.active_dungeon_landing_spotlight_reset_prefix_scanlines,
        None
    );
}

#[test]
fn landing_copy_stores_race_their_own_hdma_rows() {
    // Pinned Snes9x $00:F3BE checkpoints, immediately after STA $1B00,X.
    // Comparison 39742 ($3f->$46) presents new rows 221..223; comparison
    // 39744 ($46->$4d) presents none. A radius cutoff cannot express this
    // source contract. Earlier row stores lose their HDMA race in both runs.
    for (stores, expected_tail) in [
        ([(220, 1286), (221, 90), (221, 258), (221, 426), (221, 634)], 221),
        ([(224, 400), (224, 608), (224, 776), (224, 944), (224, 1154)], 224),
    ] {
        let mut copied = [false; SPOTLIGHT_VISIBLE_SCANLINES];
        for (row, (line, cycle)) in (219..224).zip(stores) {
            copied[row] = spotlight_copy_store_precedes_hdma(row, CpuRasterPosition::new(line, cycle));
        }
        let before = [vec![0x11; 480], vec![0x22; 480]];
        let after = [vec![0xaa; 480], vec![0xbb; 480]];
        let composed = spotlight_copy_scanout_tables(&before, &after, &copied);
        for row in 0..SPOTLIGHT_VISIBLE_SCANLINES {
            for table in 0..2 {
                assert_eq!(composed[table][row * 2],
                    if row >= expected_tail { after[table][row * 2] } else { before[table][row * 2] });
            }
        }
    }
}

#[test]
fn interrupted_dungeon_exit_spotlight_publishes_the_rom_prefix_before_waiting() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_indoor_flag(0);
    state.set_main_screen_layers(0x16);
    state.set_sub_screen_layers(0x01);
    state.follower_link_state_mut().set_y(94);
    for row in 0..240 {
        state.set_spotlight_hdma_table_dynamic_entry(row, 0x00ff);
    }

    assert!(state.begin_dungeon_exit_spotlight_entry(
        Some(DungeonExitSpotlightCpuPlan {
            interrupted_pc: 0x00_f38d,
            interrupted_return_address: 0,
            iterations_before_nmi: 19,
            link_position_integrated_before_first_nmi: false,
            returned_to_main_wait_before_first_nmi: false,
            main_loop_sprite_preparation_completed_before_second_nmi: false,
            active_window_words: [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
            following_window_words: [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
            next_entry_earliest: Some(DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_EARLIEST),
            next_entry_latest: Some(DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_LATEST),
            successor_entry_earliest: None,
            successor_entry_latest: None,
        }),
        None,
        SpotlightIteration::closing(SpotlightIterationPhase::CloseEntryBeforeTablePublication,),
    ));

    assert_eq!(state.game_state.display.bg12_window_selection, 0x33);
    assert_eq!(state.game_state.display.bg34_window_selection, 0x03);
    assert_eq!(state.game_state.display.object_color_window_selection, 0x33);
    assert_eq!(state.game_state.display.main_screen_window_layers, 0x16);
    assert_eq!(state.game_state.display.sub_screen_window_layers, 0x01);
    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        0x7e
    );
    assert_eq!(state.game_state.display.spotlight_hdma.window_state(), 0);
    assert!(state.next_display_spotlight_scanout.is_some());
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonExitSpotlightEntry { .. })
    ));
    // The C loop has authored 19 paired rows at the interrupt boundary. For
    // center 106, the upper cursor wraps through rows 0..6 while the lower
    // cursor covers rows 224..206.
    assert_ne!(state.spotlight_hdma_table_dynamic_entry(0), 0x00ff);
    assert_ne!(state.spotlight_hdma_table_dynamic_entry(206), 0x00ff);
    assert_eq!(state.spotlight_hdma_table_dynamic_entry(7), 0x00ff);
    assert_eq!(state.spotlight_hdma_table_dynamic_entry(205), 0x00ff);
}

#[test]
fn rom_spotlight_following_field_uses_the_next_unowned_publication_slot() {
    let active = [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES];
    let mut following = active;
    following[221] = 0xe818;

    assert_eq!(
        spotlight_following_field_publication(&active, &following, false),
        SpotlightFollowingFieldPublication::WithCompletionCapture,
    );
    assert_eq!(
        spotlight_following_field_publication(&active, &following, true),
        SpotlightFollowingFieldPublication::AfterCompletionCapture,
    );
    assert_eq!(
        spotlight_following_field_publication(&active, &active, false),
        SpotlightFollowingFieldPublication::AfterCompletionCapture,
    );
}

#[test]
fn spotlight_projection_generation_is_a_scanout_local_table_mix() {
    let len = ZeldaState::HDMA_DYNAMIC_TABLE_LEN;
    let before_projection = [vec![0x11; len], vec![0x33; len]];
    let after_projection = [vec![0x22; len], vec![0x44; len]];
    let mut ram = vec![0; WRAM_SIZE];

    DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout {
        before_projection,
        after_projection,
        live_tail_start: SPOTLIGHT_MIXED_SCANOUT_LIVE_TAIL_START,
    }
    .compose_into(&mut ram);

    let split = SPOTLIGHT_MIXED_SCANOUT_LIVE_TAIL_START * 2;
    for (table_base, [before, after]) in [HDMA_TABLE_DYNAMIC, RESERVED_HDMA_TABLE]
        .into_iter()
        .zip([[0x11, 0x22], [0x33, 0x44]])
    {
        assert!(ram[table_base..table_base + split]
            .iter()
            .all(|&byte| byte == before));
        assert!(ram[table_base + split..table_base + len]
            .iter()
            .all(|&byte| byte == after));
    }
}

#[test]
fn staged_spotlight_scanout_publishes_one_coherent_hardware_generation() {
    let mut state = ZeldaState::new();
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_bg12_window_selection(0x33);
    state.set_bg34_window_selection(0x03);
    state.set_object_color_window_selection(0x33);
    state.set_main_screen_window_layers(0x16);
    state.set_sub_screen_window_layers(0x00);
    state.set_hdma_enable_mask(0xc0);
    state.dma.channel[6].b_adr = 0x26;
    state.dma.channel[7].b_adr = 0x26;
    state.set_spotlight_hdma_table_dynamic_entry(0, 0xff00);
    write_le_u16(&mut state.ram, RESERVED_HDMA_TABLE, 0xfe01);
    state.stage_spotlight_scanout_for_next_display();

    // The ordinary snapshot is still the pre-NMI hardware generation. The
    // staged iris domain must replace all of its coupled controls together.
    state.set_bg12_window_selection(0);
    state.set_bg34_window_selection(0);
    state.set_object_color_window_selection(0);
    state.set_main_screen_window_layers(0);
    state.set_sub_screen_window_layers(0);
    state.set_hdma_enable_mask(0);
    state.dma.channel[6].b_adr = 0x20;
    state.dma.channel[7].b_adr = 0x21;
    state.set_spotlight_hdma_table_dynamic_entry(0, 0x00ff);
    write_le_u16(&mut state.ram, RESERVED_HDMA_TABLE, 0x01fe);
    state.capture_display_snapshot();

    let captured = state.with_display_snapshot(|display| {
        (
            display.ppu.windowsel,
            display.ppu.screen_windowed,
            display.ram[crate::game_state::constants::HDMAEN_COPY],
            [display.dma.channel[6].b_adr, display.dma.channel[7].b_adr],
            read_le_u16(&display.ram, HDMA_TABLE_DYNAMIC),
            read_le_u16(&display.ram, RESERVED_HDMA_TABLE),
        )
    });

    assert_eq!(
        captured,
        (
            0x0033_0333,
            [0x16, 0x00],
            0xc0,
            [0x26, 0x26],
            0xff00,
            0xfe01,
        )
    );
}

#[test]
fn completed_spotlight_table_projection_overlays_the_staged_scanout_generation() {
    let mut state = ZeldaState::new();
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_bg12_window_selection(0x33);
    state.set_bg34_window_selection(0x03);
    state.set_object_color_window_selection(0x33);
    state.set_main_screen_window_layers(0x16);
    state.set_sub_screen_window_layers(0x00);
    state.set_hdma_enable_mask(0xc0);
    state.set_spotlight_hdma_table_dynamic_entry(0, 0xff00);
    state.stage_spotlight_scanout_for_next_display();
    state.capture_display_snapshot();

    state.set_spotlight_hdma_table_dynamic_entry(0, 0xfe01);
    state
        .display_snapshot
        .as_mut()
        .expect("captured display")
        .hdma_table_generation = DisplayHdmaTableGeneration::SpotlightPublishedAheadOfSnapshot {
        active_table: {
            let mut table = vec![0; ZeldaState::HDMA_DYNAMIC_TABLE_LEN];
            table[..2].copy_from_slice(&0xfe01_u16.to_le_bytes());
            table
        },
    };

    let captured = state.with_display_snapshot(|display| {
        (
            display.ppu.windowsel,
            display.ppu.screen_windowed,
            display.ram[crate::game_state::constants::HDMAEN_COPY],
            read_le_u16(&display.ram, HDMA_TABLE_DYNAMIC),
        )
    });

    assert_eq!(captured, (0x0033_0333, [0x16, 0x00], 0xc0, 0xfe01));
}

#[test]
fn spotlight_return_keeps_obj_dma_on_the_pre_return_boundary() {
    let mut state = ZeldaState::new();
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.next_display_obj_scanout_generation = Some(ObjScanoutGenerations::coherent(
        GraphicsDmaGeneration::HostBoundaryBeforeMain,
    ));
    state.ppu.vram[0x4000] = 0x1111;
    state.ppu.oam[0] = 0x2222;
    state.capture_display_snapshot();

    state.ppu.vram[0x4000] = 0xaaaa;
    state.ppu.oam[0] = 0xbbbb;

    let captured =
        state.with_display_snapshot(|display| (display.ppu.vram[0x4000], display.ppu.oam[0]));

    assert_eq!(captured, (0x1111, 0x2222));
}

#[test]
fn spotlight_hdma_can_publish_ahead_of_retained_obj_domains() {
    let mut state = ZeldaState::new();
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_spotlight_hdma_table_dynamic_entry(0, 0xea0e);
    state.ppu.oam[0] = 0x2222;
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::PublishCaptured);

    // Mirror the ordinary staged boundary immediately before the next circle
    // iteration. The whole-display generation remains the old one.
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::AdvanceStaged);

    state.set_spotlight_hdma_table_dynamic_entry(0, 0xe612);
    state.ppu.oam[0] = 0xbbbb;
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishSpotlightIteration {
            iteration: SpotlightIteration::closing(
                SpotlightIterationPhase::WholeTableAfterTablePublication,
            ),
        },
        SPOTLIGHT_ITERATION_SUFFIX_NMI_SLICES,
    );
    state.capture_display_snapshot_with_override(Some(DisplaySnapshotPublication::AdvanceStaged));

    let display = state.display_snapshot.as_ref().unwrap();
    let mut composed_ram = display.ram.clone();
    display
        .hdma_table_generation
        .compose_into(&mut composed_ram);

    assert_eq!(read_le_u16(&composed_ram, HDMA_TABLE_DYNAMIC), 0xe612);
    assert_eq!(display.ppu.oam[0], 0x2222);
}

#[test]
fn completed_short_spotlight_build_projects_its_authored_table_tail() {
    let mut state = ZeldaState::new();
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_spotlight_window_radius(0x70);
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::PublishCaptured);
    state.set_spotlight_hdma_table_dynamic_entry(221, 0xae4a);
    write_le_u16(&mut state.ram, RESERVED_HDMA_TABLE + 221 * 2, 0xae4a);

    state.project_following_spotlight_tail_to_active_scanout(
        SpotlightIterationPhase::WholeTableAfterTablePublication,
        false,
    );

    let display = state.display_snapshot.as_ref().unwrap();
    let mut composed_ram = display.ram.clone();
    display
        .hdma_table_generation
        .compose_into(&mut composed_ram);
    assert_eq!(
        read_le_u16(&composed_ram, HDMA_TABLE_DYNAMIC + 221 * 2),
        0xae4a
    );
    assert_eq!(
        read_le_u16(&composed_ram, RESERVED_HDMA_TABLE + 221 * 2),
        0xae4a
    );
}

#[test]
fn game_over_fade_oam_scanout_uses_the_published_shadow_generation() {
    let mut entry = crate::game_state::FrameState::default();
    entry.main_module = 0x12;
    entry.submodule = 4;
    entry.frame_counter = 126;
    for exit_submodule in [4, 5] {
        let mut exit = entry;
        exit.submodule = exit_submodule;
        exit.frame_counter = if exit_submodule == 4 { 127 } else { 201 };

        assert_eq!(
            oam_scanout_across_main(entry, exit, OamScanoutSource::ComposeLiveAfterNmi),
            OamScanoutSource::ComposePublishedShadowDma,
        );
        assert_eq!(
            link_obj_scanout_across_main(entry, exit, GraphicsDmaGeneration::LiveAfterMain),
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
        );
    }
}

#[test]
fn moving_game_over_letters_publish_the_entry_oam_generation() {
    let mut entry = crate::game_state::FrameState::default();
    entry.main_module = 0x12;
    entry.submodule = 7;
    entry.frame_counter = 43;
    for exit_submodule in [7, 8] {
        let mut exit = entry;
        exit.submodule = exit_submodule;
        exit.frame_counter = 44;

        assert_eq!(
            oam_scanout_across_main(entry, exit, OamScanoutSource::ComposeLiveAfterNmi),
            OamScanoutSource::ComposePublishedShadowDma,
        );
        assert_eq!(
            link_obj_scanout_across_main(entry, exit, GraphicsDmaGeneration::LiveAfterMain),
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
        );
    }
}

#[test]
fn game_over_menu_retains_the_captured_resident_oam() {
    let frame = crate::game_state::FrameState {
        main_module: 0x12,
        submodule: 9,
        frame_counter: 134,
        ..crate::game_state::FrameState::default()
    };

    assert!(game_over_menu_retains_resident_oam(
        frame,
        frame,
        OamScanoutSource::RetainCapturedBeforeNmi,
    ));
    assert!(!game_over_menu_retains_resident_oam(
        frame,
        frame,
        OamScanoutSource::ComposePublishedShadowDma,
    ));
}

#[test]
fn game_over_menu_oam_dma_samples_the_host_boundary_shadow() {
    let plan = rom_graphics_dma_plan(0x12, 9);
    assert_eq!(
        plan.oam_operands,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(plan.oam_scanout, OamScanoutSource::ComposeLiveAfterNmi);
}

#[test]
fn dungeon_landing_wipe_table_projection_follows_spotlight_row_workload() {
    assert!(rom_dungeon_landing_wipe_is_active(7, 15));
    assert!(!rom_dungeon_landing_wipe_is_active(7, 14));
    assert!(!rom_dungeon_landing_wipe_is_active(14, 15));
    assert_eq!(spotlight_vertical_center(0x215a, 0x2110), 86);
    assert_eq!(spotlight_table_row_pairs(86), 139);

    assert_eq!(spotlight_table_row_pairs(42), 183);
    assert_eq!(spotlight_table_row_pairs(182), 183);
    assert!(!spotlight_table_has_long_nmi_workload(42));
    assert!(!spotlight_table_has_long_nmi_workload(182));

    assert_eq!(spotlight_table_row_pairs(41), 184);
    assert_eq!(spotlight_table_row_pairs(183), 184);
    assert!(spotlight_table_has_long_nmi_workload(41));
    assert!(spotlight_table_has_long_nmi_workload(183));
    assert!(spotlight_opening_projects_live_tail_before_hdma(0x3f, 183));
    assert!(!spotlight_opening_projects_live_tail_before_hdma(0x46, 183));
    assert!(!spotlight_opening_projects_live_tail_before_hdma(0x3f, 182));
}

#[test]
fn spotlight_close_entry_publication_follows_circle_workload() {
    let short_entry = SpotlightIterationPhase::for_close_iteration(0, 0x7e, 42);
    assert_eq!(
        short_entry,
        SpotlightIterationPhase::CloseEntryAfterTablePublication
    );
    assert_eq!(
        short_entry.close_completion_publication(),
        DisplaySnapshotPublication::PublishCaptured
    );

    let long_entry = SpotlightIterationPhase::for_close_iteration(0, 0x7e, 41);
    assert_eq!(
        long_entry,
        SpotlightIterationPhase::CloseEntryBeforeTablePublication
    );
    assert_eq!(
        long_entry.close_completion_publication(),
        DisplaySnapshotPublication::AdvanceStaged
    );
    assert_eq!(
        SpotlightIterationPhase::for_game_over_close_iteration(0x77),
        SpotlightIterationPhase::MixedTailAfterReturn
    );
    assert_eq!(
        SpotlightIterationPhase::for_game_over_close_iteration(0x70),
        SpotlightIterationPhase::MixedTailAfterReturn
    );
    assert_eq!(
        SpotlightIterationPhase::for_game_over_close_iteration(0x69),
        SpotlightIterationPhase::WholeTableAfterTablePublication
    );
    assert!(game_over_spotlight_build_uses_live_oam(Some(
        GameWorkContinuation::FinishGameOverSpotlightBuild {
            table_build: SpotlightTableBuildContinuation::default(),
            entry: false,
            iteration: SpotlightIteration::game_over_closing(
                SpotlightIterationPhase::MixedTailAfterReturn,
                false,
            ),
        },
    )));
    assert!(!game_over_spotlight_build_uses_live_oam(Some(
        GameWorkContinuation::FinishSpotlightIteration {
            iteration: SpotlightIteration::game_over_closing(
                SpotlightIterationPhase::MixedTailAfterReturn,
                false,
            ),
        },
    )));
    let mut build_entry = crate::game_state::FrameState::default();
    build_entry.main_module = 0x12;
    build_entry.submodule = 3;
    build_entry.frame_counter = 105;
    let mut after_leading_nmi = build_entry;
    after_leading_nmi.frame_counter = 106;
    assert!(game_over_spotlight_build_entry_uses_live_oam(
        build_entry,
        after_leading_nmi,
    ));
    assert!(!game_over_spotlight_build_entry_uses_live_oam(
        after_leading_nmi,
        after_leading_nmi,
    ));
    assert!(game_over_spotlight_return_boundary_uses_live_oam(
        after_leading_nmi,
        0x70,
        None,
    ));
    assert!(!game_over_spotlight_return_boundary_uses_live_oam(
        after_leading_nmi,
        0x77,
        None,
    ));
    assert!(SpotlightIteration::game_over_closing(
        SpotlightIterationPhase::MixedTailAfterReturn,
        false,
    )
    .game_over_build_needs_deferred_caller_return());
    assert!(!SpotlightIteration::game_over_closing(
        SpotlightIterationPhase::WholeTableAfterTablePublication,
        false,
    )
    .game_over_build_needs_deferred_caller_return());
    let mut iris_goal = after_leading_nmi;
    iris_goal.submodule = 4;
    assert!(game_over_iris_goal_scanout_is_closed(
        after_leading_nmi,
        iris_goal,
    ));
    assert!(!game_over_iris_goal_scanout_is_closed(iris_goal, iris_goal,));
}

#[test]
fn retained_spotlight_goal_scanout_rejects_the_following_main_force_blank() {
    let mut state = ZeldaState::new();
    state.set_screen_brightness(0x0f);
    state.ppu.brightness = 0x0f;
    state.ppu.forced_blank = false;
    state.capture_display_snapshot();

    // zelda3/src/load_gfx.c:IrisSpotlight_ConfigureTable writes
    // INIDISP_copy=$80 when the closing radius reaches zero. On the standard
    // route Snes9x enters that goal call at V=21 of internal frame 11,478,
    // after it has already presented internal frame 11,477. The caller-return
    // capture therefore retains the visible field; the new live latch belongs
    // to the following scanout.
    state.set_screen_brightness(0x80);
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::RetainPublished);
    state.interrupt_nmi_for_active_scanout(0, None, false);

    assert!(state.ppu.forced_blank);
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|display| !display.accepts_nmi_dma_receipts
            && display.effective_presented_dma.is_none()));

    let presented = state.with_display_snapshot(|display| {
        (
            display.ppu.forced_blank,
            display.ppu.brightness,
            display.ram[crate::game_state::constants::INIDISP_COPY],
        )
    });

    assert_eq!(presented, (false, 0x0f, 0x0f));
    assert!(state.ppu.forced_blank);
    assert_eq!(state.ppu.brightness, 0);
}

#[test]
fn spotlight_goal_nmi_cannot_attach_to_the_field_that_precedes_its_cpu_work() {
    let mut state = ZeldaState::new();
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();
    state.set_screen_brightness(0x0f);
    state.ppu.brightness = 0x0f;
    state.ppu.forced_blank = false;
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::AdvanceStaged);

    // zelda3/src/load_gfx.c:IrisSpotlight_ConfigureTable reaches the closing
    // goal at V=21 in the pinned Snes9x trace. Its INIDISP_copy=$80 assignment
    // and first NMI crossing therefore occur after this active field began.
    state.schedule_dungeon_exit_spotlight_goal_caller(SpotlightIteration::closing(
        SpotlightIterationPhase::WholeTableAfterTablePublication,
    ));
    assert!(state
        .game_execution_scheduler
        .active_field_precedes_current_scheduled_work());
    state.set_screen_brightness(0x80);
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::AdvanceStaged);
    state.interrupt_nmi(0, None, false);

    let published = state.display_snapshot.as_ref().unwrap();
    assert!(!published.ppu.forced_blank);
    assert!(published
        .effective_presented_dma
        .as_ref()
        .and_then(|receipt| receipt.completed_ppu_registers)
        .is_none());
    assert!(!state.with_display_snapshot(|display| display.ppu.forced_blank));
    assert!(state.ppu.forced_blank);
}
