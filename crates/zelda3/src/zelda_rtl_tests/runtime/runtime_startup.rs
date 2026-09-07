//! ZeldaState runtime tests — startup.
//! Split mechanically from the hub file; bodies unchanged.

use super::*;

#[test]
fn attract_map_mode7_brightness_override_ends_with_fade_in() {
    assert!(rom_attract_world_map_mode7_brightness_is_early_published(
        20, 0, 1, 4
    ));
    assert!(!rom_attract_world_map_mode7_brightness_is_early_published(
        20, 0, 1, 5
    ));
}

#[test]
fn attract_map_exit_drains_the_final_projection_before_the_tilemap_clear_returns() {
    let pending = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishAttractWorldMapExit,
        ATTRACT_WORLD_MAP_EXIT_NMI_SLICES,
    );
    assert_eq!(
        pending.in_flight_display_snapshot_publication_override(),
        Some(DisplaySnapshotPublication::AdvanceStaged)
    );
}

#[test]
fn attract_map_projection_generation_follows_the_cpu_hdma_race() {
    let first_current = (0..ATTRACT_MAP_PROJECTION_WORDS)
        .find(|&line| attract_map_projection_current_word_is_visible(line));
    assert_eq!(first_current, Some(45));

    let mut ram = vec![0x55; 0x20000];
    let before_projection = vec![0xaa; ZeldaState::HDMA_DYNAMIC_TABLE_LEN];
    DisplayHdmaTableGeneration::AttractMapProjectionDuringScanout { before_projection }
        .compose_into(&mut ram);

    assert_eq!(
        &ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + 45 * 2],
        vec![0xaa; 45 * 2]
    );
    assert_eq!(
        &ram[HDMA_TABLE_DYNAMIC + 45 * 2..HDMA_TABLE_DYNAMIC + ATTRACT_MAP_PROJECTION_WORDS * 2],
        vec![0x55; (ATTRACT_MAP_PROJECTION_WORDS - 45) * 2]
    );
}

#[test]
fn reset_can_preserve_sram() {
    let mut state = ZeldaState::new();
    state.ram[1] = 1;
    state.sram[1] = 2;
    state.vram_mut()[1] = 3;

    state.reset(true);

    assert_eq!(state.ram[1], 0);
    assert_eq!(state.sram[1], 2);
    assert_eq!(state.vram()[1], 0);
}

#[test]
fn sram_path_prefers_explicit_save_dir() {
    assert_eq!(
        ZeldaState::sram_path_from_env(
            Some("/tmp/z3-save".into()),
            Some("/tmp/xdg".into()),
            Some("/tmp/home".into()),
        ),
        PathBuf::from("/tmp/z3-save/sram.dat")
    );
}

#[test]
fn intro_background_settings_write_ppu_tilemap_regs() {
    let mut state = ZeldaState::new();
    state.ppu.bg_layer[0].tilemap_adr = 0;
    state.ppu.bg_layer[1].tilemap_adr = 0;
    state.ppu.bg_layer[2].tilemap_adr = 0;

    state.intro_initialize_background_settings();

    assert_eq!(state.game_state.display.bg_mode, 9);
    assert_eq!(state.game_state.display.mosaic_copy, 0);
    assert_eq!(state.ppu.bg_layer[0].tilemap_adr, 0x1000);
    assert!(state.ppu.bg_layer[0].tilemap_wider);
    assert!(state.ppu.bg_layer[0].tilemap_higher);
    assert_eq!(state.ppu.bg_layer[1].tilemap_adr, 0);
    assert!(state.ppu.bg_layer[1].tilemap_wider);
    assert!(state.ppu.bg_layer[1].tilemap_higher);
    assert_eq!(state.ppu.bg_layer[2].tilemap_adr, 0x6000);
    assert!(state.ppu.bg_layer[2].tilemap_wider);
    assert!(state.ppu.bg_layer[2].tilemap_higher);
}

#[test]
fn migrated_select_file_frame_state_uses_semantic_accessors() {
    for (path, source) in [
        ("ancilla.rs", include_str!("../../ancilla.rs")),
        ("attract.rs", include_str!("../../attract.rs")),
        ("audio.rs", include_str!("../../audio.rs")),
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
        ("select_file.rs", include_str!("../../select_file.rs")),
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
            "sprite_main_ganon.rs",
            include_str!("../../sprite_main_ganon.rs"),
        ),
        (
            "sprite_main_guard.rs",
            include_str!("../../sprite_main_guard.rs"),
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
    ] {
        for needle in [
            concat!("self.", "ram[MAIN_MODULE_INDEX]"),
            concat!("self.", "ram[SUBMODULE_INDEX]"),
            concat!("self.", "ram[SUBSUBMODULE_INDEX]"),
        ] {
            assert!(
                !source.contains(needle),
                "{path} should use native frame state for {needle}"
            );
        }
    }
}

#[test]
fn save_quit_intro_memory_return_does_not_repeat_initialization() {
    let asset_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../zelda3_assets.dat");
    let mut atomic = ZeldaState::new();
    atomic.assets = Some(AssetPack::parse(&std::fs::read(asset_path).unwrap()).unwrap());
    atomic.set_main_module(0x17);
    atomic.set_submodule(1);
    atomic.save_quit_reset_hold = true;
    let mut staged = atomic.clone();
    atomic.death_func15_save_quit_reset_state_before_dungeon_info_clear();
    staged.death_func31_through_intro_memory();
    assert_eq!(
        (
            staged.game_state.frame.main_module,
            staged.game_state.frame.submodule
        ),
        (0x17, 2)
    );
    let disable = staged.ram[0x13];
    staged.death_func15_save_quit_reset_state_before_dungeon_info_clear();
    assert_eq!(staged.ram[0x13], disable);
    assert_eq!(staged.game_state, atomic.game_state);
    assert_eq!(staged.ram, atomic.ram);
}

#[test]
fn pinned_snes9x_reset_progress_is_unique_and_unclaimed_facts_fail_host_close() {
    let progress = sprite::DungeonResetSpritesCpuProgress::Cache {
        slot: 15,
        field: CachedSpriteCacheField::StateClear,
    };
    let mut duplicate = ZeldaState::new();
    duplicate.set_rom_startup_timing(true);
    assert_eq!(
        duplicate.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![
                dungeon_reset_progress_receipt(progress, OriginalTimingBoundary::NmiAccepted,),
                dungeon_reset_progress_receipt(progress, OriginalTimingBoundary::NmiAccepted,),
            ],
        )),
        Err(OriginalTimingReceiptInstallError::DuplicateDungeonResetProgress),
    );
    assert_eq!(
        duplicate.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    crate::MainLoopInterruption::SpritePreparation,
                ),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    crate::MainLoopInterruption::SpritePreparation,
                ),
            ],
        )),
        Err(OriginalTimingReceiptInstallError::DuplicateMainLoopInterruption),
    );
    assert_eq!(
        duplicate.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![
                cached_sprite_progress_receipt(
                    crate::CachedSpriteExecutionProgress::Restoring {
                        slot: 7,
                        live_fields: 4,
                    },
                    OriginalTimingBoundary::NmiAccepted,
                ),
                cached_sprite_progress_receipt(
                    crate::CachedSpriteExecutionProgress::Restoring {
                        slot: 7,
                        live_fields: 4,
                    },
                    OriginalTimingBoundary::NmiAccepted,
                ),
            ],
        )),
        Err(OriginalTimingReceiptInstallError::DuplicateCachedSpriteExecutionProgress),
    );
    assert_eq!(
        duplicate.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![
                OriginalTimingSemanticReceipt::PreOverworldStageCompleted(
                    crate::PreOverworldStageCompletion::OverlaysReturned,
                ),
                OriginalTimingSemanticReceipt::PreOverworldStageCompleted(
                    crate::PreOverworldStageCompletion::OverlaysReturned,
                ),
            ],
        )),
        Err(OriginalTimingReceiptInstallError::DuplicatePreOverworldStageCompletion),
    );
    assert_eq!(
        duplicate.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::IterationStarted,
                ),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
            ],
        )),
        Err(OriginalTimingReceiptInstallError::DuplicateMainLoopProgress),
    );
    assert_eq!(
        duplicate.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![
                OriginalTimingSemanticReceipt::MainLoopIterationReturnedToWait,
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            ],
        )),
        Err(OriginalTimingReceiptInstallError::DuplicateMainLoopIterationReturn),
    );
    let spotlight_progress = crate::SpotlightTableBuildProgressReceipt {
        progress: crate::SpotlightTableBuildProgress {
            completed_iterations: 209,
            checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                pending_circle_input: 30,
            },
        },
        boundary: OriginalTimingBoundary::NmiAccepted,
    };
    assert_eq!(
        duplicate.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(spotlight_progress),
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(spotlight_progress),
            ],
        )),
        Err(OriginalTimingReceiptInstallError::DuplicateSpotlightTableBuildProgress),
    );
    assert_eq!(
        duplicate.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![
                OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
                OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
            ],
        )),
        Err(OriginalTimingReceiptInstallError::DuplicateDungeonExitSpotlightCallerReturn),
    );
    assert_eq!(
        duplicate.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![
                OriginalTimingSemanticReceipt::OverworldSpotlightGoalCallerReturned,
                OriginalTimingSemanticReceipt::OverworldSpotlightGoalCallerReturned,
            ],
        )),
        Err(OriginalTimingReceiptInstallError::DuplicateOverworldSpotlightGoalCallerReturn,),
    );
    assert_eq!(
        duplicate.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                crate::SpotlightTableBuildProgressReceipt {
                    progress: crate::SpotlightTableBuildProgress {
                        completed_iterations: 209,
                        checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                            pending_circle_input: 0,
                        },
                    },
                    boundary: OriginalTimingBoundary::HostReturn,
                },
            )],
        )),
        Err(OriginalTimingReceiptInstallError::InvalidSpotlightTableBuildProgress),
    );
    for checkpoint in [
        crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
            pending_circle_input: 1,
        },
        crate::SpotlightTableBuildCheckpoint::BeforeLowerTableWrite {
            lower_cursor: 0,
            circle_value: 0,
        },
    ] {
        assert_eq!(
            duplicate.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
                0,
                0,
                vec![OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                    crate::SpotlightTableBuildProgressReceipt {
                        progress: crate::SpotlightTableBuildProgress {
                            completed_iterations: 240,
                            checkpoint,
                        },
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                )],
            )),
            Err(OriginalTimingReceiptInstallError::InvalidSpotlightTableBuildProgress),
            "a build checkpoint cannot follow all 240 C loop iterations",
        );
    }
    assert_eq!(
        duplicate.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                crate::SpotlightTableBuildProgressReceipt {
                    progress: crate::SpotlightTableBuildProgress {
                        completed_iterations: 241,
                        checkpoint: crate::SpotlightTableBuildCheckpoint::ProjectionCopy {
                            copied_words: 0,
                        },
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            )],
        )),
        Err(OriginalTimingReceiptInstallError::InvalidSpotlightTableBuildProgress),
        "a projection checkpoint cannot exceed the 240-iteration C loop",
    );
    let sprite_reset = crate::SpriteResetAllProgressReceipt {
        progress: crate::SpriteResetAllProgress::SpriteDisableAllCompleted,
        boundary: OriginalTimingBoundary::HostReturn,
    };
    assert_eq!(
        duplicate.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![
                OriginalTimingSemanticReceipt::SpriteResetAllProgress(sprite_reset),
                OriginalTimingSemanticReceipt::SpriteResetAllProgress(sprite_reset),
            ],
        )),
        Err(OriginalTimingReceiptInstallError::DuplicateSpriteResetAllProgress),
    );
    let mut nmi_boundary = ZeldaState::new();
    nmi_boundary.set_rom_startup_timing(true);
    assert_eq!(
        nmi_boundary.install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![OriginalTimingSemanticReceipt::SpriteResetAllProgress(
                crate::SpriteResetAllProgressReceipt {
                    boundary: OriginalTimingBoundary::NmiAccepted,
                    ..sprite_reset
                },
            )],
        )),
        Ok(()),
    );

    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.zelda_setup_emu_callbacks(None, Some(test_run_frame_callback), None);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![dungeon_reset_progress_receipt(
                progress,
                OriginalTimingBoundary::NmiAccepted,
            )],
        ))
        .unwrap();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.zelda_run_frame(0);
    }));

    assert!(result.is_err());
    assert_eq!(state.original_timing_owner(), OriginalTimingOwner::Live);
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(0));
    assert_eq!(
        state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic(),
        &[dungeon_reset_progress_receipt(
            progress,
            OriginalTimingBoundary::NmiAccepted,
        )],
    );
    assert!(state.original_timing_host_dispatch_active);
}

#[test]
fn terminal_intro_initialization_return_carries_its_trailing_open_nmi_across_presentation() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(0);
    state.set_submodule(1);
    state.set_subsubmodule(2);
    state.set_frame_counter(2);
    state.ending_scratch_mut().set_primary_word(0x17fe);
    state.ending_scratch_mut().set_secondary_word(0x13fe);
    state.intro_initialization_work_frames_pending = 3;
    state.intro_initialization_reset_obj_control_pending = true;
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state.ppu.vram[0x1234] = 0x1357;

    // Cold run 84 accepts and completes the held leading NMI, returns the
    // suspended initialization caller through ZeldaRunGameLoop's common
    // suffix, then accepts the following Open NMI at main wait.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            84,
            0x0008,
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
    state.run_frame_internal(0x0008, crate::RUN_MAIN);

    assert_eq!(state.intro_initialization_work_frames_pending, 0);
    assert!(!state.intro_initialization_reset_obj_control_pending);
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

    let first = state.with_display_snapshot(|rendered| rendered.ppu.vram[0x1234]);
    let second = state.with_display_snapshot(|rendered| rendered.ppu.vram[0x1234]);
    assert_eq!((first, second), (0x1357, 0x1357));
    state.advance_display_publication_history();
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));

    // Run 85 completes exactly that carried Open handler, publishes joypad,
    // and begins the next main-loop iteration once.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            85,
            0x0008,
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
            ],
        ))
        .unwrap();
    state.run_frame_internal(0x0008, crate::RUN_MAIN);

    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(85));
}

#[test]
fn nonterminal_intro_initialization_carries_runs_159_and_160_into_terminal_run_161() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(0);
    state.set_submodule(1);
    state.set_subsubmodule(10);
    state.set_frame_counter(10);
    state.intro_initialization_work_frames_pending = 3;
    state.intro_initialization_reset_obj_control_pending = true;
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state.ppu.vram[0x1234] = 0x1590;

    // Source run 159 accepts and completes one held NMI while the long
    // item-graphics caller is still active, then returns video and accepts
    // the next held NMI. CallStackContinued is the host's terminal progress
    // fact; it is not proof that the common ZeldaRunGameLoop suffix returned.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            159,
            0x0008,
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
    state.run_frame_internal(0x0008, crate::RUN_MAIN);

    assert_eq!(state.intro_initialization_work_frames_pending, 3);
    assert!(!state.intro_initialization_reset_obj_control_pending);
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
    assert_eq!(
        state.with_display_snapshot(|rendered| rendered.ppu.vram[0x1234]),
        0x1590,
    );
    state.advance_display_publication_history();
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));

    // Run 160 first completes that carried handler, resumes the same CPU
    // caller, returns its next video generation, and accepts another held NMI.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            160,
            0x0008,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
            ],
        ))
        .unwrap();
    state.run_frame_internal(0x0008, crate::RUN_MAIN);

    assert_eq!(state.intro_initialization_work_frames_pending, 3);
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
    assert_eq!(
        state.with_display_snapshot(|rendered| rendered.ppu.vram[0x1234]),
        0x1590,
    );
    state.advance_display_publication_history();
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));

    // Run 161 completes the carried handler, then the resumed CPU caller
    // reaches NMI_PrepareSprites and the $12 clear before returning at $8036.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            161,
            0x0008,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            ],
        ))
        .unwrap();
    state.run_frame_internal(0x0008, crate::RUN_MAIN);

    assert_eq!(state.intro_initialization_work_frames_pending, 0);
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(161));
}

#[test]
fn nonterminal_intro_acceptance_captures_reset_obj_addresses_before_retiring_flag() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(0);
    state.set_submodule(1);
    state.set_subsubmodule(9);
    state.intro_initialization_work_frames_pending = 3;
    state.intro_initialization_reset_obj_control_pending = true;
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state.ppu.obj_tile_adr1 = 0x4000;
    state.ppu.obj_tile_adr2 = 0x6000;

    // The item-graphics caller returns this host at a newly accepted Held NMI;
    // its handler does not run yet. Reset OBSEL therefore belongs to the
    // immutable acceptance-host scanout even though the CPU transient retires
    // before the next host begins.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            158,
            0x0008,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
            ],
        ))
        .unwrap();
    state.run_frame_internal(0x0008, crate::RUN_MAIN);

    assert_eq!(state.intro_initialization_work_frames_pending, 3);
    assert!(!state.intro_initialization_reset_obj_control_pending);
    assert_eq!(
        (state.ppu.obj_tile_adr1, state.ppu.obj_tile_adr2),
        (0x4000, 0x6000)
    );
    assert_eq!(
        state.display_snapshot.as_ref().map(|snapshot| (
            snapshot.ppu.obj_tile_adr1,
            snapshot.ppu.obj_tile_adr2,
            snapshot.accepts_nmi_dma_receipts,
        )),
        Some((0, 0x1000, true)),
    );
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
}

#[test]
fn nonterminal_intro_host_without_nmi_preserves_reset_obj_transient() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(0);
    state.set_submodule(1);
    state.set_subsubmodule(9);
    state.intro_initialization_work_frames_pending = 3;
    state.intro_initialization_reset_obj_control_pending = true;
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();

    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            158,
            0x0008,
            vec![OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            )],
        ))
        .unwrap();
    state.run_frame_internal(0x0008, crate::RUN_MAIN);

    assert_eq!(state.intro_initialization_work_frames_pending, 3);
    assert!(state.intro_initialization_reset_obj_control_pending);
    assert!(!state.original_timing_nmi_publication_pending);
    assert!(state.display_snapshot.is_none());
}

#[test]
fn nonterminal_intro_initialization_requires_complete_typed_authority_before_mutation() {
    for missing in ["progress", "invalid-nmi-phases"] {
        let mut state = ZeldaState::new();
        state.rom_startup_timing = true;
        state.initialized = true;
        state.rom_reset_frame_delay = 0;
        state.set_animated_tile_data_source_address(1);
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        state.set_main_module(0);
        state.set_submodule(1);
        state.set_subsubmodule(10);
        state.intro_initialization_work_frames_pending = 3;
        state.intro_initialization_reset_obj_control_pending = true;
        state.pending_main_loop_common_suffix =
            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
        state.latch_nmi_update();
        state.ram[0x12] = 1;
        state.capture_display_snapshot();
        state.original_timing_nmi_publication_pending = true;
        state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
        let semantic = match missing {
            "progress" => vec![OriginalTimingSemanticReceipt::NmiHandlerCompleted],
            "invalid-nmi-phases" => vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
            ],
            _ => unreachable!(),
        };
        state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
            160,
            0x0008,
            semantic.clone(),
        ));
        let snapshot_before = state
            .display_snapshot
            .as_ref()
            .map(|snapshot| (snapshot.accepts_nmi_dma_receipts, snapshot.ppu.vram[0x1234]));
        let latch_before = state.game_state.display.nmi_update_is_latched();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal_after_original_timing(0x0008, crate::RUN_MAIN);
        }));

        assert!(result.is_err(), "case={missing}");
        assert_eq!(state.intro_initialization_work_frames_pending, 3);
        assert!(state.intro_initialization_reset_obj_control_pending);
        assert_eq!(
            state.game_state.display.nmi_update_is_latched(),
            latch_before,
        );
        assert_eq!(
            state
                .display_snapshot
                .as_ref()
                .map(|snapshot| (snapshot.accepts_nmi_dma_receipts, snapshot.ppu.vram[0x1234],)),
            snapshot_before,
        );
        assert_eq!(
            state
                .original_timing_semantic_receipts
                .as_ref()
                .unwrap()
                .semantic,
            semantic,
            "case={missing}",
        );
    }
}

#[test]
fn terminal_intro_initialization_return_rejects_incompatible_suffixes_before_mutation() {
    for incompatible in ["extended-oam", "interrupted"] {
        let mut state = ZeldaState::new();
        state.rom_startup_timing = true;
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        state.set_main_module(0);
        state.set_submodule(1);
        state.set_subsubmodule(2);
        state.intro_initialization_work_frames_pending = 3;
        state.intro_initialization_reset_obj_control_pending = true;
        state.pending_main_loop_common_suffix = Some(if incompatible == "extended-oam" {
            MainLoopCommonSuffixContinuation::ResumeSpritePreparationExtendedOamPackingAndClearNmiLatch {
                next_group_start: 4,
            }
        } else {
            MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch
        });
        state.latch_nmi_update();
        state.ram[0x12] = 1;
        let latch_before = state.game_state.display.nmi_update_is_latched();
        let mut semantic = vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ];
        if incompatible == "interrupted" {
            semantic.insert(
                3,
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    crate::MainLoopInterruption::LinkOam,
                ),
            );
        }
        state.original_timing_semantic_receipts =
            Some(OriginalTimingHostReceipts::new(84, 0x0008, semantic));
        let semantic_before = state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic
            .clone();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal_after_original_timing(0x0008, crate::RUN_MAIN);
        }));

        assert!(result.is_err(), "case={incompatible}");
        assert_eq!(state.intro_initialization_work_frames_pending, 3);
        assert!(state.intro_initialization_reset_obj_control_pending);
        assert_eq!(
            state.game_state.display.nmi_update_is_latched(),
            latch_before,
        );
        assert!(state.display_snapshot.is_none());
        assert_eq!(
            state
                .original_timing_semantic_receipts
                .as_ref()
                .unwrap()
                .semantic,
            semantic_before,
        );
    }
}

#[test]
fn terminal_intro_initialization_return_rejects_active_scheduler_before_mutation() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(0);
    state.set_submodule(1);
    state.set_subsubmodule(10);
    state.intro_initialization_work_frames_pending = 3;
    state.intro_initialization_reset_obj_control_pending = true;
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state.ram[0x12] = 1;
    state
        .game_execution_scheduler
        .schedule_work(GameWorkContinuation::FinishAttractThroneRoom, 1);
    let semantic = vec![
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ),
        OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
    ];
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        161,
        0x0008,
        semantic.clone(),
    ));
    let work_before = state.game_execution_scheduler.current_work();
    let latch_before = state.game_state.display.nmi_update_is_latched();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.run_frame_internal_after_original_timing(0x0008, crate::RUN_MAIN);
    }));

    assert!(result.is_err());
    assert_eq!(state.intro_initialization_work_frames_pending, 3);
    assert!(state.intro_initialization_reset_obj_control_pending);
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch,)
    );
    assert_eq!(state.game_execution_scheduler.current_work(), work_before);
    assert_eq!(
        state.game_state.display.nmi_update_is_latched(),
        latch_before,
    );
    assert!(state.display_snapshot.is_none());
    assert_eq!(
        state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic,
        semantic,
    );
}

#[test]
fn paired_resume_rejects_nonterminal_intro_initialization_counter() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_last_oracle_host_call = Some(160);
    state.intro_initialization_work_frames_pending = 3;

    assert!(!state.paired_resume_cpu_boundary_is_quiescent());
    assert_eq!(
        state.capture_original_timing_resume_checkpoint(),
        Err(OriginalTimingResumeCheckpointError::ActiveTranslatedContinuation),
    );
}

#[test]
fn paired_resume_rejects_suspended_intro_memory_darken_caller() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.intro_memory_darken_frame_delay = 1;

    assert!(!state.paired_resume_cpu_boundary_is_quiescent());
    assert_eq!(
        state.capture_original_timing_resume_checkpoint(),
        Err(OriginalTimingResumeCheckpointError::ActiveTranslatedContinuation),
    );
}

#[test]
fn live_save_menu_initialization_holds_then_executes_the_c_endpoint_once() {
    let mut held = ZeldaState::new();
    held.restore_live_rom_timing_after_checkpoint();
    held.set_main_module(14);
    held.set_submodule(11);
    held.set_subsubmodule(0);
    held.original_timing_owner = OriginalTimingOwnerState::Live;
    held.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![
            OriginalTimingSemanticReceipt::SaveMenuInitializationProgress(
                crate::SaveMenuInitializationProgress::InProgress,
            ),
        ],
    ));
    let ram_before = held.ram.to_vec();

    held.Module0E_0B_SaveMenu();

    assert_eq!(held.ram.as_slice(), ram_before.as_slice());
    assert!(held
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));

    let mut expected = ZeldaState::new();
    expected.set_main_module(14);
    expected.set_submodule(11);
    expected.set_subsubmodule(0);
    expected.Module0E_0B_SaveMenu();

    held.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        1,
        0,
        vec![
            OriginalTimingSemanticReceipt::SaveMenuInitializationProgress(
                crate::SaveMenuInitializationProgress::Completed,
            ),
        ],
    ));
    held.Module0E_0B_SaveMenu();

    assert_eq!(held.ram.as_slice(), expected.ram.as_slice());
    assert_eq!(held.game_state.frame, expected.game_state.frame);
    assert_eq!(
        held.game_state.messaging.runtime,
        expected.game_state.messaging.runtime,
    );
    assert_eq!(
        held.game_state.messaging.render_buffer,
        expected.game_state.messaging.render_buffer,
    );
}

#[test]
fn save_and_load_func_append_and_copy_bytes() {
    let mut arr = ByteArray::default();
    let mut src = [1, 2, 3, 4];
    ZeldaState::save_func(&mut arr, &mut src);
    assert_eq!(arr.data, src);

    let mut st = LoadFuncState::new(&arr.data);
    let mut dst = [0; 4];
    ZeldaState::load_func(&mut st, &mut dst);
    assert_eq!(dst, src);
    assert_eq!(st.remaining(), 0);
}

#[test]
fn snes_state_save_load_roundtrips_runtime_regions() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    assert_eq!(
        state.original_timing_owner(),
        OriginalTimingOwner::PendingColdStart
    );
    state.ram[0x100] = 0x12;
    write_le_u16(&mut state.ram, MAP16_LOAD_SRC_OFF, 0x1390);
    write_le_u16(&mut state.ram, MAP16_LOAD_DST_OFF, 0x001f);
    write_le_u16(&mut state.ram, MAP16_LOAD_Y_UNIT, 0x000e);
    state.sync_overworld_map16_state_from_ram();
    state.set_spotlight_hdma_table_dynamic_entry(0, 0x00ab);
    state.sram[0x22] = 0x34;
    state.ppu.cgram[7] = 0x2468;
    state.ppu.bg_layer[1].tilemap_higher = true;
    state.ppu.bg_layer[1].tilemap_adr = 0x1357;
    state.dma.channel[6].a_adr = 0x4567;

    let mut arr = ByteArray::default();
    let mut save = SaveLoadFunc::Save(&mut arr);
    state.save_snes_state(&mut save);
    assert_eq!(state.ram[0x1b00], 0xab);

    state.ram[0x100] = 0;
    write_le_u16(&mut state.ram, MAP16_LOAD_SRC_OFF, 0);
    write_le_u16(&mut state.ram, MAP16_LOAD_DST_OFF, 0);
    write_le_u16(&mut state.ram, MAP16_LOAD_Y_UNIT, 0);
    state.sync_overworld_map16_state_from_ram();
    state.set_spotlight_hdma_table_dynamic_entry(0, 0);
    state.sram[0x22] = 0;
    state.ppu.cgram[7] = 0;
    state.ppu.bg_layer[1].tilemap_higher = false;
    state.ppu.bg_layer[1].tilemap_adr = 0;
    state.dma.channel[6].a_adr = 0;

    let mut st = LoadFuncState::new(&arr.data);
    let mut load = SaveLoadFunc::Load(&mut st);
    state.load_snes_state(&mut load);

    assert_eq!(state.ram[0x100], 0x12);
    assert_eq!(
        state.game_state.world.overworld.map16.active_load,
        OverworldMap16LoadState {
            src_off: 0x1390,
            dst_off: 0x001f,
            y_unit: 0x000e
        }
    );
    assert_eq!(state.ram[HDMA_TABLE_DYNAMIC], 0xab);
    assert_eq!(state.sram[0x22], 0x34);
    assert_eq!(state.ppu.cgram[7], 0x2468);
    assert!(state.ppu.bg_layer[1].tilemap_higher);
    assert_eq!(state.ppu.bg_layer[1].tilemap_adr, 0x1357);
    assert_eq!(state.dma.channel[6].a_adr, 0x4567);
    assert_eq!(
        state.original_timing_owner(),
        OriginalTimingOwner::Unavailable(OriginalTimingUnavailableReason::CheckpointRestore)
    );
}

#[test]
fn read_from_file_and_state_recorder_save_load_match_c_layout() {
    let mut cursor = std::io::Cursor::new(vec![1, 2, 3, 4]);
    let mut bytes = [0; 4];
    ZeldaState::read_from_file(&mut cursor, &mut bytes);
    assert_eq!(bytes, [1, 2, 3, 4]);

    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    assert_eq!(
        state.original_timing_owner(),
        OriginalTimingOwner::PendingColdStart
    );
    state.ram[0x123] = 0x45;
    state.sram[0x234] = 0x67;
    let mut sr = StateRecorder {
        last_inputs: 0x00ff,
        frames_since_last: 7,
        total_frames: 9,
        log: ByteArray {
            data: vec![0x01, 0x23],
        },
        ..StateRecorder::default()
    };
    let mut out = Vec::new();
    state.state_recorder_save(&mut sr, &mut out);

    state.ram[0x123] = 0;
    state.sram[0x234] = 0;
    let mut loaded = StateRecorder::default();
    state.state_recorder_load(&mut loaded, &mut std::io::Cursor::new(out), false);

    assert_eq!(loaded.last_inputs, 0x00ff);
    assert_eq!(loaded.frames_since_last, 7);
    assert_eq!(loaded.total_frames, 9);
    assert_eq!(loaded.log.data, vec![0x01, 0x23]);
    assert_eq!(state.ram[0x123], 0x45);
    assert_eq!(state.sram[0x234], 0x67);
    assert_eq!(
        state.original_timing_owner(),
        OriginalTimingOwner::Unavailable(OriginalTimingUnavailableReason::CheckpointRestore)
    );
}

#[test]
fn language_and_save_slot_shells_match_defaults() {
    let mut state = ZeldaState::new();
    state.dialogue_blk_index = 7;
    state.dialogue_font_blk_index = 8;
    state.dialogue_flags = 9;

    state.zelda_set_language(None);

    assert_eq!(state.dialogue_blk_index, 0);
    assert_eq!(state.dialogue_font_blk_index, 0);
    assert_eq!(state.dialogue_flags, 0);
    assert_eq!(
        ZeldaState::save_slot_path(SaveLoadCommand::Load, 3).unwrap(),
        PathBuf::from("saves/save3.sav")
    );
    assert!(ZeldaState::save_slot_path(SaveLoadCommand::Save, 256).is_none());
    assert_eq!(
        ZeldaState::save_slot_path(SaveLoadCommand::Replay, 256).unwrap(),
        Path::new("saves/ref").join("Chapter 1 - Zelda's Rescue.sav")
    );
}

#[test]
fn damaging_pit_reset_restores_ground_or_permabunny_state() {
    let mut state = ZeldaState::new();
    set_link_test_byte(&mut state, LINK_IS_BUNNY, 1);
    set_link_test_byte(&mut state, LINK_ITEM_MOON_PEARL, 0);
    state.follower_link_state_mut().set_swim_direction_flags(8);
    set_link_test_byte(&mut state, LINK_IS_IN_DEEP_WATER, 1);
    set_link_test_byte(&mut state, LINK_DISABLE_SPRITE_DAMAGE, 1);
    state.follower_link_state_mut().set_pit_data_index(1);
    state.ram[SWIMMING_COUNTDOWN] = 7;
    state
        .swim_acceleration_mut()
        .set_speed_active_flag(0, 0x1234);

    state.link_reset_state_after_damaging_pit();

    assert_eq!(state.game_state.player.follower_link.handler_state(), 23);
    assert_eq!(link_test_byte(&state, LINK_DIRECTION_LAST), 8);
    assert_eq!(link_test_byte(&state, LINK_IS_IN_DEEP_WATER), 0);
    assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 0);
    assert_eq!(state.game_state.player.follower_link.pit_data_index(), 0);
    assert_eq!(state.ram[SWIMMING_COUNTDOWN], 0);
    assert_eq!(
        state
            .game_state
            .player
            .swim_acceleration
            .speed_active_flag(0),
        0
    );

    set_link_test_byte(&mut state, LINK_ITEM_MOON_PEARL, 1);
    state.follower_link_state_mut().set_handler_state(6);
    state.link_reset_state_after_damaging_pit();
    assert_eq!(state.game_state.player.follower_link.handler_state(), 0);
}

#[test]
fn set_to_deep_water_resets_swim_state_and_latches_direction() {
    let mut state = ZeldaState::new();
    set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 8);
    state.follower_link_state_mut().set_grabbing_wall(1);
    state.follower_link_state_mut().set_speed_setting(2);
    state.ram[SWIMMING_COUNTDOWN] = 7;
    state.swim_acceleration_mut().set_acceleration(0, 0x1234);

    state.link_set_to_deep_water();

    assert_eq!(link_test_byte(&state, LINK_IS_IN_DEEP_WATER), 1);
    assert_eq!(
        state.game_state.player.follower_link.swim_direction_flags(),
        8
    );
    assert_eq!(state.game_state.player.follower_link.grabbing_wall(), 0);
    assert_eq!(state.game_state.player.follower_link.speed_setting(), 0);
    assert_eq!(state.ram[SWIMMING_COUNTDOWN], 0);
    assert_eq!(state.game_state.player.swim_acceleration.acceleration(0), 0);
}

#[test]
fn reset_all_acceleration_clears_swim_accel_pairs() {
    let mut state = ZeldaState::new();
    for offset in [0, 2] {
        state
            .swim_acceleration_mut()
            .set_speed_active_flag(offset, 0xffff);
        state.swim_acceleration_mut().set_mode(offset, 0xffff);
        state
            .swim_acceleration_mut()
            .set_acceleration(offset, 0xffff);
        state.swim_acceleration_mut().set_max_speed(offset, 0xffff);
    }
    for offset in [SWIM_STROKE_FRAME_COUNTER, SWIM_STROKE_FRAME_COUNTER + 2] {
        write_le_u16(&mut state.ram, offset, 0xffff);
    }
    state
        .swim_acceleration_mut()
        .set_acceleration_direction(0, 0xffff);

    state.reset_all_acceleration();

    for offset in [0, 2] {
        assert_eq!(
            state
                .game_state
                .player
                .swim_acceleration
                .speed_active_flag(offset),
            0
        );
        assert_eq!(state.game_state.player.swim_acceleration.mode(offset), 0);
        assert_eq!(
            state
                .game_state
                .player
                .swim_acceleration
                .acceleration(offset),
            0
        );
        assert_eq!(
            state.game_state.player.swim_acceleration.max_speed(offset),
            0
        );
    }
    for offset in [SWIM_STROKE_FRAME_COUNTER, SWIM_STROKE_FRAME_COUNTER + 2] {
        assert_eq!(read_le_u16(&state.ram, offset), 0);
    }
    assert_eq!(
        state
            .game_state
            .player
            .swim_acceleration
            .acceleration_direction(0),
        0xffff
    );
}

#[test]
fn swim_movement_without_input_resets_idle_flag_moving_state() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_handler_state(4);
    set_link_test_byte(&mut state, LINK_FLAG_MOVING, 1);
    state.ram[PLAYER_DEFENSE_FLAGS] = 0xff;
    state.ram[PIT_CORRECTION_ACTIVE_FLAG] = 1;
    state
        .swim_acceleration_mut()
        .set_speed_active_flag(0, 0x1111);

    state.link_handle_swim_movements();

    assert_eq!(link_test_byte(&state, LINK_Y_VEL), 0);
    assert_eq!(link_test_byte(&state, LINK_X_VEL), 0);
    assert_eq!(state.ram[PLAYER_DEFENSE_FLAGS] & 0x0f, 0);
    assert_eq!(
        state
            .game_state
            .player
            .swim_acceleration
            .speed_active_flag(0),
        0
    );
    assert_eq!(state.ram[PIT_CORRECTION_ACTIVE_FLAG], 0);
}

#[test]
fn exiting_dash_resets_or_counts_down_like_c_state() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_handler_state(18);
    set_link_test_byte(&mut state, LINK_COUNTDOWN_FOR_DASH, 3);

    state.link_state_exiting_dash();

    assert_eq!(link_test_byte(&state, LINK_COUNTDOWN_FOR_DASH), 4);
    assert_eq!(state.game_state.player.follower_link.handler_state(), 18);

    state.follower_link_state_mut().set_joypad1h_last(1);
    state.follower_link_state_mut().set_running_state(1);
    state.follower_link_state_mut().set_speed_setting(16);
    set_link_test_byte(&mut state, LINK_CANT_CHANGE_DIRECTION, 1);
    state.follower_link_state_mut().set_button_b_frames(8);
    state.swim_acceleration_mut().set_mode(0, 0x1234);

    state.link_state_exiting_dash();

    assert_eq!(link_test_byte(&state, LINK_COUNTDOWN_FOR_DASH), 0);
    assert_eq!(state.game_state.player.follower_link.speed_setting(), 0);
    assert_eq!(state.game_state.player.follower_link.handler_state(), 0);
    assert_eq!(state.game_state.player.follower_link.running_state(), 0);
    assert_eq!(state.game_state.player.swim_acceleration.mode(0), 0);
    assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION), 0);
}

#[test]
fn tile_main_handler_spike_trigger_applies_damage_and_bunny_reset() {
    let mut state = ZeldaState::new();
    state.set_indoor_flag(1);
    set_link_test_byte(&mut state, LINK_IS_BUNNY, 1);
    set_link_test_byte(&mut state, LINK_IS_BUNNY_MIRROR, 1);
    set_link_test_byte(&mut state, LINK_ITEM_MOON_PEARL, 1);
    set_link_test_word(&mut state, LINK_TIMER_TEMPBUNNY, 0x1234);
    set_link_test_byte(&mut state, LINK_NEED_FOR_POOF_FOR_TRANSFORM, 1);
    state
        .tile_detect_position_mut()
        .set_location_calc_mask(0x01ff);
    state
        .dungeon_bg2_attributes_mut()
        .set_bg2_attr(16 * 8 + 1, 0x0d);

    state.tile_detect_main_handler(0);

    assert_eq!(link_test_byte(&state, LINK_GIVE_DAMAGE), 8);
    assert_eq!(link_test_byte(&state, LINK_IS_BUNNY), 0);
    assert_eq!(link_test_byte(&state, LINK_IS_BUNNY_MIRROR), 0);
    assert_eq!(link_test_byte(&state, LINK_NEED_FOR_POOF_FOR_TRANSFORM), 0);
    assert_eq!(link_test_word(&state, LINK_TIMER_TEMPBUNNY), 0);
}

#[test]
fn first_frame_runs_startup_writes() {
    let mut state = ZeldaState::new();
    state.sram[0x03e5] = 0xaa;
    state.sram[0x03e6] = 0x55;
    state.sram[0x08e5] = 0x12;
    state.ram[MAIN_PALETTE_BUFFER] = 0xff;

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(read_le_u16(&state.ram, ANIMATED_TILE_DATA_SRC), 0xa680);
    assert_eq!(
        state.game_state.display.animated_tile_data_source_address,
        0xa680
    );
    // Exact Snes9x DMA trace at the first NMI upload: channels 3 and 8 read
    // from $7E:0000. These pointers remain zero until ROM code assigns them.
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_9), 0);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_14), 0);
    assert_eq!(&state.ram[0..3], &[0x00, 0x80, 0x00]);
    assert_eq!(state.game_state.display.screen_brightness, 15);
    assert_eq!(state.ram[FLAG_UPDATE_CGRAM_IN_NMI], 0);
    assert_eq!(read_le_u16(&state.sram, 0x03e5), 0x55aa);
    assert_eq!(read_le_u16(&state.sram, 0x08e5), 0);
    assert_eq!(state.selected_save_slot_x2(), 0);
    assert_eq!(read_le_u16(&state.ram, MAIN_PALETTE_BUFFER), 0);
}

#[test]
fn rom_startup_preroll_matches_live_console_timing() {
    let mut state = ZeldaState::new();

    state.set_rom_startup_timing(true);

    assert_eq!(state.rom_reset_frame_delay, 81);
    assert_eq!(
        state.original_timing_owner(),
        OriginalTimingOwner::PendingColdStart
    );
}

#[test]
fn public_frame_wrapper_consumes_unseedable_cold_start_on_reset_delay_frame() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    assert_eq!(state.rom_reset_frame_delay, 81);
    assert_eq!(
        state.original_timing_owner(),
        OriginalTimingOwner::PendingColdStart
    );

    state.zelda_run_frame_with_replay_input_override(0, None);

    assert_eq!(state.frame_ctr_dbg, 1);
    assert_eq!(state.rom_reset_frame_delay, 80);
    assert_eq!(
        state.original_timing_owner(),
        OriginalTimingOwner::Unavailable(OriginalTimingUnavailableReason::MissingRom)
    );
    assert!(!state.original_timing_cold_start_eligible);
}

#[test]
fn direct_run_frame_internal_consumes_unseedable_cold_start_on_reset_delay_frame() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    assert_eq!(state.rom_reset_frame_delay, 81);
    assert_eq!(
        state.original_timing_owner(),
        OriginalTimingOwner::PendingColdStart
    );

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.rom_reset_frame_delay, 80);
    assert_eq!(
        state.original_timing_owner(),
        OriginalTimingOwner::Unavailable(OriginalTimingUnavailableReason::MissingRom)
    );
    assert!(!state.original_timing_cold_start_eligible);
}

#[test]
fn reset_is_the_only_progressed_state_path_back_to_pending_cold_start() {
    let mut disabled = ZeldaState::new();
    disabled.set_rom_startup_timing(true);
    disabled.frame_ctr_dbg = 1;
    disabled.set_rom_startup_timing(false);

    disabled.reset(true);
    assert_eq!(
        disabled.original_timing_owner(),
        OriginalTimingOwner::Disabled
    );
    disabled.set_rom_startup_timing(true);
    assert_eq!(
        disabled.original_timing_owner(),
        OriginalTimingOwner::PendingColdStart
    );

    let mut restored = ZeldaState::new();
    restored.restore_live_rom_timing_after_checkpoint();
    assert!(matches!(
        restored.original_timing_owner(),
        OriginalTimingOwner::Unavailable(_)
    ));
    restored.reset(true);
    assert_eq!(
        restored.original_timing_owner(),
        OriginalTimingOwner::PendingColdStart
    );
}

#[test]
fn rom_startup_holds_only_the_interrupted_intro_initialization_frame() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;

    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.game_state.frame.subsubmodule, 1);

    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.game_state.frame.subsubmodule, 1);

    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.game_state.frame.subsubmodule, 2);
}

#[test]
fn first_rom_frame_publishes_intro_audio_on_the_following_nmi() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;

    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.zelda_debug_apu_write_ports(), [0, 0, 0, 0]);

    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.zelda_debug_apu_write_ports(), [0, 0, 0, 10]);
}

#[test]
fn legacy_intro_memory_initialization_fallback_keeps_configured_delay() {
    assert_eq!(configured_intro_memory_initialization_frames(), 41);
    assert_eq!(
        intro_memory_initialization_delay_for_owner(true, 0),
        Some(1)
    );
    assert_eq!(
        intro_memory_initialization_delay_for_owner(true, 41),
        Some(1)
    );
    assert_eq!(
        intro_memory_initialization_delay_for_owner(false, 41),
        Some(41),
    );
    assert_eq!(intro_memory_initialization_delay_for_owner(false, 0), None);
}

#[test]
fn live_intro_brightness_zero_arms_typed_memory_caller_without_finishing_it() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(0);
    state.set_submodule(1);
    state.set_subsubmodule(11);
    state.set_screen_brightness(1);

    state.intro_init_continue();

    assert_eq!(state.intro_memory_darken_frame_delay, 1);
    assert_eq!(state.game_state.frame.submodule, 1);
    assert_eq!(state.intro_poly_thread_initialization_phase, 0);
}

#[test]
fn intro_memory_darken_nonterminal_plan_owns_all_four_source_nmi_shapes() {
    for (case, pending_at_entry, trailing_acceptance) in [
        ("carry-complete", true, false),
        ("carry-complete-and-accept", true, true),
        ("accept-and-complete", false, false),
        ("accept-complete-accept", false, true),
    ] {
        let mut state =
            intro_memory_darken_suspended_state(pending_at_entry, trailing_acceptance, 181);
        let accepted_snapshot_epoch = state
            .display_snapshot
            .as_ref()
            .map(|snapshot| snapshot.publication_epoch);
        if pending_at_entry && trailing_acceptance {
            assert_eq!(
                state.with_display_snapshot(|display| display.ppu.vram[0]),
                181,
                "run180's acceptance-host scanout must present before run181 resumes",
            );
            state.advance_display_publication_history();
            assert!(state
                .display_snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
        }

        state.run_frame_internal(0, 0);

        assert_eq!(
            state.intro_memory_darken_frame_delay, 1,
            "{case}: a nonterminal source slice cannot decrement the translated sentinel",
        );
        assert_eq!(
            state.pending_main_loop_common_suffix,
            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
            "{case}: the suspended common suffix retired too early",
        );
        assert!(state.game_state.display.nmi_update_is_latched(), "{case}");
        assert_eq!(
            state.original_timing_nmi_publication_pending, trailing_acceptance,
            "{case}",
        );
        assert_eq!(
            state.original_timing_pending_nmi_update_gate,
            trailing_acceptance.then_some(NmiUpdateGate::LatchHeld),
            "{case}",
        );
        assert!(state.original_timing_semantic_receipts.is_none(), "{case}");
        if pending_at_entry && !trailing_acceptance {
            assert_eq!(
                state
                    .display_snapshot
                    .as_ref()
                    .map(|snapshot| snapshot.publication_epoch),
                accepted_snapshot_epoch,
                "{case}: completing a carried handler must not recapture its acceptance scanout",
            );
        }
        if trailing_acceptance {
            assert!(state
                .display_snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
        }
    }
}

#[test]
fn intro_memory_darken_nonterminal_plan_rejects_malformed_authority_before_mutation() {
    for malformed in [
        "extra-joypad",
        "wrong-gate",
        "missing-completion",
        "closed-carry",
    ] {
        let mut state = intro_memory_darken_suspended_state(true, false, 181);
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        match malformed {
            "extra-joypad" => state
                .original_timing_semantic_receipts
                .as_mut()
                .unwrap()
                .semantic
                .insert(
                    1,
                    OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                        high: 1,
                        low: 2,
                        high_filtered: 3,
                        low_filtered: 4,
                    }),
                ),
            "wrong-gate" => {
                state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open)
            }
            "missing-completion" => state
                .original_timing_semantic_receipts
                .as_mut()
                .unwrap()
                .semantic
                .retain(|receipt| {
                    !matches!(receipt, OriginalTimingSemanticReceipt::NmiHandlerCompleted)
                }),
            "closed-carry" => {
                state
                    .display_snapshot
                    .as_mut()
                    .unwrap()
                    .accepts_nmi_dma_receipts = false;
            }
            _ => unreachable!(),
        }
        let sentinel = state.intro_memory_darken_frame_delay;
        let suffix = state.pending_main_loop_common_suffix;
        let semantic = state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic
            .clone();
        let pending = state.original_timing_nmi_publication_pending;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.original_timing_intro_memory_darken_plan();
        }));
        assert!(result.is_err(), "{malformed}");
        assert_eq!(
            state.intro_memory_darken_frame_delay, sentinel,
            "{malformed}"
        );
        assert_eq!(state.pending_main_loop_common_suffix, suffix, "{malformed}");
        assert_eq!(
            state
                .original_timing_semantic_receipts
                .as_ref()
                .unwrap()
                .semantic,
            semantic,
            "{malformed}",
        );
        assert_eq!(state.original_timing_nmi_publication_pending, pending);
    }
}

#[test]
fn intro_memory_darken_runtime_preflight_rejects_extra_fact_before_host_setup() {
    let mut state = intro_memory_darken_suspended_state(true, false, 181);
    state
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .insert(
            1,
            OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                high: 1,
                low: 2,
                high_filtered: 3,
                low_filtered: 4,
            }),
        );
    state.ppu.vram[0] = 0x5a5a;
    let frame = state.game_state.frame;
    let scheduler = state.game_execution_scheduler;
    let sentinel = state.intro_memory_darken_frame_delay;
    let suffix = state.pending_main_loop_common_suffix;
    let snapshot_epoch = state
        .display_snapshot
        .as_ref()
        .map(|snapshot| snapshot.publication_epoch);
    let semantic = state
        .original_timing_semantic_receipts
        .as_ref()
        .unwrap()
        .semantic
        .clone();
    assert!(!state.audio_nmi_processed_before_main);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.run_frame_internal(0, 0);
    }));

    assert!(result.is_err());
    assert_eq!(state.game_state.frame, frame);
    assert_eq!(state.game_execution_scheduler, scheduler);
    assert_eq!(state.intro_memory_darken_frame_delay, sentinel);
    assert_eq!(state.pending_main_loop_common_suffix, suffix);
    assert_eq!(state.ppu.vram[0], 0x5a5a);
    assert_eq!(
        state
            .display_snapshot
            .as_ref()
            .map(|snapshot| snapshot.publication_epoch),
        snapshot_epoch,
    );
    assert_eq!(
        state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic,
        semantic,
    );
    assert!(!state.audio_nmi_processed_before_main);
}

#[test]
fn terminal_intro_memory_darken_return_seeds_two_then_suffix_decrements_once() {
    let mut state = intro_memory_darken_suspended_state(true, false, 221);
    let sprite_packs = (0..9)
        .map(|pack| vec![pack as u8 + 1; 0x600])
        .collect::<Vec<_>>();
    let mut asset_data = Vec::new();
    let mut asset_ranges = vec![(0, 0); 65];
    put_test_asset(
        &mut asset_data,
        &mut asset_ranges,
        64,
        pack_test_memblk_arrays(&sprite_packs),
    );
    state.assets = Some(AssetPack::from_data_ranges(asset_data, asset_ranges));
    state
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic = vec![
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ),
        OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
    ];
    state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::Open];
    state.original_timing_expected_nmi_ppu_register_operands = vec![None, None];
    state.run_frame_internal(0, 0);

    assert_eq!(state.game_state.display.bg_tile_animation_countdown, 1);
    assert_eq!(state.intro_memory_darken_frame_delay, 0);
    assert_eq!(state.game_state.frame.submodule, 2);
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_nmi_publication_pending);
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));

    // The following Open handler owns publication, not another sprite-prep
    // countdown tick. Completing it must leave the source value at one.
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        222,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                high: 0,
                low: 0,
                high_filtered: 0,
                low_filtered: 0,
            }),
        ],
    ));
    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
    let phases = state.take_original_timing_nmi_phases();
    let classification = classify_original_timing_nmi_phases_with_ownership(true, &phases);
    state
        .complete_original_timing_nmi_handler_for_active_scanout(
            classification.handler_completion,
            0,
            None,
        )
        .assert_no_unclaimed_dialogue_text_dma();
    assert_eq!(state.game_state.display.bg_tile_animation_countdown, 1);
}

#[test]
fn rom_intro_poly_thread_initializer_resumes_across_host_frames() {
    assert_eq!(rom_intro_poly_init_decision(3), (true, false, 2));
    assert_eq!(rom_intro_poly_init_decision(2), (false, false, 1));
    assert_eq!(rom_intro_poly_init_decision(1), (false, true, 0));
}

#[test]
fn intro_poly_suspended_caller_follows_exact_hosts_222_through_225() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.ram[crate::game_state::constants::MAIN_MODULE] = 0;
    state.ram[crate::game_state::constants::SUBMODULE] = 2;
    state.ram[crate::game_state::constants::SUBSUBMODULE] = 26;
    state.sync_native_game_state_from_ram();
    let sprite_packs = (0..9)
        .map(|pack| vec![pack as u8 + 1; 0x600])
        .collect::<Vec<_>>();
    let mut asset_data = Vec::new();
    let mut asset_ranges = vec![(0, 0); 65];
    put_test_asset(
        &mut asset_data,
        &mut asset_ranges,
        64,
        pack_test_memblk_arrays(&sprite_packs),
    );
    state.assets = Some(AssetPack::from_data_ranges(asset_data, asset_ranges));
    state.set_animated_tile_data_source_address(1);
    state.initialized = true;
    state.intro_poly_thread_initialization_phase = 3;
    state.main_loop_sprite_preparation_completed = true;
    state.capture_display_snapshot();
    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .accepts_nmi_dma_receipts = true;
    let host_221_snapshot_epoch = state.display_snapshot.as_ref().unwrap().publication_epoch;
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
    let obj_vram_before = state.ppu.vram[0x4400..0x5000].to_vec();
    let entry_frame_counter = state.game_state.frame.frame_counter;

    // Host 222 completes the Open NMI accepted at host 221, begins the one
    // ZeldaRunGameLoop iteration which owns the long poly initializer, then
    // returns with a Held NMI accepted inside that suspended caller.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            222,
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
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            ],
        ))
        .unwrap();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.intro_poly_thread_initialization_phase, 1);
    assert_eq!(
        state.game_state.frame.frame_counter,
        entry_frame_counter.wrapping_add(1),
    );
    assert!(!state.main_loop_sprite_preparation_completed);
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
    let link = &state.game_state.player.follower_link;
    assert_eq!(link.joypad1h_last(), 0x12);
    assert_eq!(link.joypad1l_last(), 0x34);
    assert_eq!(link.filtered_joypad_h(), 0x56);
    assert_eq!(link.filtered_joypad_l(), 0x78);
    assert_ne!(
        state.display_snapshot.as_ref().unwrap().publication_epoch,
        host_221_snapshot_epoch,
        "the trailing Held acceptance owns a distinct host-222 scanout",
    );
    state.with_display_snapshot(|_| ());
    state.advance_display_publication_history();
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));

    // Host 223 is one of an unbounded number of possible nonterminal source
    // slices. It completes the carried Held handler without decrementing the
    // translated call-stack sentinel or replaying the main-loop prefix.
    let host_222_snapshot_epoch = state.display_snapshot.as_ref().unwrap().publication_epoch;
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            223,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
            ],
        ))
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.intro_poly_thread_initialization_phase, 1);
    assert_eq!(
        state.game_state.frame.frame_counter,
        entry_frame_counter.wrapping_add(1),
    );
    assert_eq!(
        state.display_snapshot.as_ref().unwrap().publication_epoch,
        host_222_snapshot_epoch,
        "a carried Held completion must not recapture its acceptance scanout",
    );
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);

    // Host 224 accepts and completes another Held NMI, returns the suspended
    // caller, executes the one common suffix, and carries the following Open
    // acceptance to the next host.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            224,
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

    assert_eq!(state.game_state.frame.main_module, 0);
    assert_eq!(state.game_state.frame.submodule, 3);
    assert_eq!(
        state.game_state.frame.frame_counter,
        entry_frame_counter.wrapping_add(1),
    );
    assert_eq!(state.intro_poly_thread_initialization_phase, 0);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.main_loop_sprite_preparation_completed);
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert_ne!(state.ppu.vram[0x4400..0x5000], obj_vram_before);
    assert_eq!(
        state.display_snapshot.as_ref().unwrap().ppu.vram[0x4400..0x5000],
        state.ppu.vram[0x4400..0x5000],
        "the returned field must contain the OBJ generation authored after the held NMI",
    );
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::Open),
    );
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(224));
    assert!(state.original_timing_semantic_receipts.is_none());

    state.set_sound_effect_1(0x67);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            225,
            0,
            vec![
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
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            ],
        ))
        .unwrap();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.zelda_debug_apu_write_ports()[2], 0x67);
    assert_eq!(
        state.game_state.frame.frame_counter,
        entry_frame_counter.wrapping_add(2),
    );
    let link = &state.game_state.player.follower_link;
    assert_eq!(link.joypad1h_last(), 0xa5);
    assert_eq!(link.joypad1l_last(), 0x5a);
    assert_eq!(link.filtered_joypad_h(), 0x81);
    assert_eq!(link.filtered_joypad_l(), 0x42);
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(225));
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn intro_bg_fade_common_suffix_precedes_poly_nmi_and_next_host_completes_it_once() {
    let mut state = ZeldaState::new();
    state.initialized = true;
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.intro_memory_darken_frame_delay = 0;
    state.intro_poly_upload_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(0);
    state.set_submodule(7);
    state.set_subsubmodule(0);
    state.set_frame_counter(147);
    state.set_countdown(31);

    // Source run 889 completes the leading open handler, begins the fresh
    // ZeldaRunGameLoop iteration, runs its unconditional common suffix, and
    // only then accepts the following open NMI from the cooperative poly
    // thread. The handler for that last acceptance belongs to run 890.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            889,
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
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            ],
        ))
        .unwrap();

    state.run_frame_internal(0, crate::RUN_MAIN | crate::RUN_POLY);

    assert_eq!(state.game_state.frame.frame_counter, 148);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.main_loop_sprite_preparation_completed);
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::Open),
    );
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(889));
    assert!(state.original_timing_semantic_receipts.is_none());

    state.set_sound_effect_1(0x67);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            890,
            0,
            vec![
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
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            ],
        ))
        .unwrap();

    state.run_frame_internal(0, crate::RUN_MAIN | crate::RUN_POLY);

    // Interrupt_NMI_AudioParts_Locked consumes this command once. Replaying
    // the carried handler would overwrite the published port with zero.
    assert_eq!(state.zelda_debug_apu_write_ports()[2], 0x67);
    assert_eq!(state.game_state.system_signals.sound_effect_1(), 0);
    assert_eq!(state.game_state.frame.frame_counter, 149);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    let link = &state.game_state.player.follower_link;
    assert_eq!(link.joypad1h_last(), 0xa5);
    assert_eq!(link.joypad1l_last(), 0x5a);
    assert_eq!(link.filtered_joypad_h(), 0x81);
    assert_eq!(link.filtered_joypad_l(), 0x42);
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(890));
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn intro_poly_phase_arms_and_retires_one_common_suffix_owner() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.original_timing_owner =
        OriginalTimingOwnerState::Unavailable(OriginalTimingUnavailableReason::ProgressedState);
    state.rom_reset_frame_delay = 0;
    state.ram[crate::game_state::constants::MAIN_MODULE] = 0;
    state.ram[crate::game_state::constants::SUBMODULE] = 2;
    state.ram[crate::game_state::constants::SUBSUBMODULE] = 26;
    state.sync_native_game_state_from_ram();
    state.set_animated_tile_data_source_address(1);
    state.intro_poly_thread_initialization_phase = 3;
    state.main_loop_sprite_preparation_completed = true;

    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.intro_poly_thread_initialization_phase, 2);
    assert!(!state.main_loop_sprite_preparation_completed);
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );

    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.intro_poly_thread_initialization_phase, 1);
    assert!(state.pending_main_loop_common_suffix.is_some());

    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.intro_poly_thread_initialization_phase, 0);
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert_eq!(state.game_state.frame.submodule, 3);
}

#[test]
fn live_intro_poly_entry_rejects_wrong_policy_or_vector_before_mutation() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(0);
    state.set_submodule(2);
    state.set_subsubmodule(26);
    state.intro_poly_thread_initialization_phase = 3;
    state.main_loop_sprite_preparation_completed = true;
    state.capture_display_snapshot();
    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .accepts_nmi_dma_receipts = true;
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
    state.set_sound_effect_1(0x6a);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            222,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                    high: 1,
                    low: 2,
                    high_filtered: 3,
                    low_filtered: 4,
                }),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::IterationStarted,
                ),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            ],
        ))
        .unwrap();

    for malformed in ["wrong-policy", "missing-joypad"] {
        let mut rejected = state.clone();
        if malformed == "missing-joypad" {
            rejected
                .original_timing_semantic_receipts
                .as_mut()
                .unwrap()
                .semantic
                .retain(|receipt| {
                    !matches!(receipt, OriginalTimingSemanticReceipt::JoypadPublication(_))
                });
        }
        let frame = rejected.game_state.frame;
        let phase = rejected.intro_poly_thread_initialization_phase;
        let suffix = rejected.pending_main_loop_common_suffix;
        let sprite_preparation = rejected.main_loop_sprite_preparation_completed;
        let pending = rejected.original_timing_nmi_publication_pending;
        let pending_gate = rejected.original_timing_pending_nmi_update_gate;
        let native_latch = rejected.game_state.display.nmi_update_is_latched();
        let oam = rejected.ppu.oam.clone();
        let snapshot = rejected.display_snapshot.as_ref().map(|snapshot| {
            (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.oam.clone(),
                snapshot.ppu.vram.clone(),
            )
        });
        let semantic = rejected.original_timing_semantic_receipts.clone();
        let gates = rejected.original_timing_expected_nmi_update_gates.clone();
        let sound_effect = rejected.game_state.system_signals.sound_effect_1();
        let audio_ports = rejected.zelda_debug_apu_write_ports();
        assert!(!rejected.audio_nmi_processed_before_main);

        let run_what = if malformed == "wrong-policy" {
            crate::RUN_POLY
        } else {
            crate::RUN_MAIN
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            rejected.run_frame_internal(0, run_what);
        }));

        assert!(result.is_err(), "{malformed}");
        assert_eq!(rejected.game_state.frame, frame, "{malformed}");
        assert_eq!(
            rejected.intro_poly_thread_initialization_phase, phase,
            "{malformed}",
        );
        assert_eq!(
            rejected.pending_main_loop_common_suffix, suffix,
            "{malformed}"
        );
        assert_eq!(
            rejected.main_loop_sprite_preparation_completed, sprite_preparation,
            "{malformed}",
        );
        assert_eq!(rejected.original_timing_nmi_publication_pending, pending);
        assert_eq!(
            rejected.original_timing_pending_nmi_update_gate, pending_gate,
            "{malformed}",
        );
        assert_eq!(
            rejected.game_state.display.nmi_update_is_latched(),
            native_latch,
            "{malformed}",
        );
        assert_eq!(rejected.ppu.oam, oam, "{malformed}");
        assert_eq!(
            rejected.display_snapshot.as_ref().map(|snapshot| (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.oam.clone(),
                snapshot.ppu.vram.clone(),
            )),
            snapshot,
            "{malformed}",
        );
        assert_eq!(rejected.original_timing_semantic_receipts, semantic);
        assert_eq!(rejected.original_timing_expected_nmi_update_gates, gates);
        assert_eq!(
            rejected.game_state.system_signals.sound_effect_1(),
            sound_effect,
            "{malformed}",
        );
        assert_eq!(rejected.zelda_debug_apu_write_ports(), audio_ports);
        assert!(!rejected.audio_nmi_processed_before_main);
    }
}

#[test]
fn live_terminal_intro_poly_rejects_legacy_return_fact_before_mutation() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(0);
    state.set_submodule(2);
    state.set_subsubmodule(26);
    state.intro_poly_thread_initialization_phase = 1;
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            224,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopIterationReturnedToWait,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            ],
        ))
        .unwrap();
    let semantic = state.original_timing_semantic_receipts.clone();
    let gates = state.original_timing_expected_nmi_update_gates.clone();
    let frame = state.game_state.frame;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.run_frame_internal(0, crate::RUN_MAIN);
    }));

    assert!(result.is_err());
    assert_eq!(state.intro_poly_thread_initialization_phase, 1);
    assert_eq!(state.game_state.frame, frame);
    assert!(state.pending_main_loop_common_suffix.is_some());
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_semantic_receipts, semantic);
    assert_eq!(state.original_timing_expected_nmi_update_gates, gates);
}

#[test]
fn attract_low_work_area_clear_refreshes_native_state_before_reuse() {
    let mut state = ZeldaState::new();
    state.attract_scene_mut().set_state(1);

    state.clear_attract_low_work_area();

    assert_eq!(state.game_state.ending.attract_scene.state(), 0);
}

#[test]
fn attract_graphics_initializer_resumes_at_semantic_work_boundaries() {
    assert_eq!(rom_attract_init_graphics_decision(4), (false, 3));
    assert_eq!(rom_attract_init_graphics_decision(3), (false, 2));
    assert_eq!(rom_attract_init_graphics_decision(2), (true, 1));
    assert_eq!(rom_attract_init_graphics_decision(1), (false, 0));
}

#[test]
fn intro_poly_initialization_resumes_for_cold_start_and_attract_restart() {
    assert!(rom_intro_poly_initialization_is_active(0, 2));
    assert!(rom_intro_poly_initialization_is_active(0, 10));
    assert!(!rom_intro_poly_initialization_is_active(0, 11));
    assert!(!rom_intro_poly_initialization_is_active(20, 10));
}

#[test]
fn attract_first_story_render_wait_is_armed_by_fade_completion() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.attract_scene_mut().set_state(4);
    state.set_screen_brightness(15);

    state.attract_fade_in_sequence();

    assert_eq!(state.game_state.ending.attract_scene.state(), 5);
    assert_eq!(state.attract_first_story_render_delay, 7);
}

#[test]
fn file_select_graphics_resumes_the_module_after_every_intervening_nmi() {
    let mut scheduler = GameExecutionScheduler::default();
    scheduler.schedule_file_select_graphics();

    for _ in 0..FILE_SELECT_GRAPHICS_NMI_SLICES - 1 {
        assert_eq!(
            scheduler.advance_startup_sequence(),
            Some(StartupSequenceStep::FileSelectWaiting)
        );
    }
    assert_eq!(
        scheduler.advance_startup_sequence(),
        Some(StartupSequenceStep::CompleteFileSelectGraphics)
    );
    assert!(!scheduler.is_idle());
    assert_eq!(
        scheduler.advance_startup_sequence(),
        Some(StartupSequenceStep::ResumeFileSelectModule)
    );
    assert!(scheduler.is_idle());
}

#[test]
fn file_select_graphics_source_authority_ignores_the_legacy_slice_count() {
    let mut almost_complete = GameExecutionScheduler::default();
    almost_complete.schedule_file_select_graphics();
    for _ in 0..FILE_SELECT_GRAPHICS_NMI_SLICES - 1 {
        assert_eq!(
            almost_complete.advance_startup_sequence(),
            Some(StartupSequenceStep::FileSelectWaiting),
        );
    }
    let before_nonterminal = almost_complete;
    assert_eq!(
        almost_complete.advance_file_select_graphics_with_authoritative_completion(false),
        Some(StartupSequenceStep::FileSelectWaiting),
    );
    assert_eq!(almost_complete, before_nonterminal);
    let mut legacy_probe = almost_complete;
    assert_eq!(
        legacy_probe.advance_startup_sequence(),
        Some(StartupSequenceStep::CompleteFileSelectGraphics),
        "the numeric estimate deliberately disagrees with Continued authority",
    );

    let mut just_started = GameExecutionScheduler::default();
    just_started.schedule_file_select_graphics();
    assert_eq!(
        just_started.advance_file_select_graphics_with_authoritative_completion(true),
        Some(StartupSequenceStep::CompleteFileSelectGraphics),
    );
    assert_eq!(
        just_started.advance_startup_sequence(),
        Some(StartupSequenceStep::ResumeFileSelectModule),
    );
    assert!(just_started.is_idle());
}

#[test]
fn live_file_select_waiting_uses_all_four_source_continuation_shapes() {
    for (case, pending_at_entry, trailing_acceptance) in [
        ("carry-complete", true, false),
        ("carry-complete-accept", true, true),
        ("accept-complete", false, false),
        ("accept-complete-accept", false, true),
    ] {
        // Put the legacy estimate one slice from completion. Continued is the
        // authoritative proof that this source caller remains Loading.
        let mut state = live_file_select_waiting_state(
            pending_at_entry,
            trailing_acceptance,
            FILE_SELECT_GRAPHICS_NMI_SLICES - 1,
            967,
        );
        let entry_snapshot_epoch = state
            .display_snapshot
            .as_ref()
            .map(|snapshot| snapshot.publication_epoch);
        let frame_counter = state.game_state.frame.frame_counter;

        state.run_frame_internal(0, crate::RUN_MAIN);

        assert_eq!(
            state.game_state.frame.frame_counter, frame_counter,
            "{case}"
        );
        assert_eq!(
            state.pending_main_loop_common_suffix,
            Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
            "{case}",
        );
        assert!(state.game_state.display.nmi_update_is_latched(), "{case}");
        assert_eq!(
            state.original_timing_nmi_publication_pending, trailing_acceptance,
            "{case}",
        );
        assert_eq!(
            state.original_timing_pending_nmi_update_gate,
            trailing_acceptance.then_some(NmiUpdateGate::LatchHeld),
            "{case}",
        );
        assert!(state.original_timing_semantic_receipts.is_none(), "{case}");
        if pending_at_entry && !trailing_acceptance {
            assert_eq!(
                state
                    .display_snapshot
                    .as_ref()
                    .map(|snapshot| snapshot.publication_epoch),
                entry_snapshot_epoch,
                "{case}: a carried handler must refine rather than recapture its acceptance host",
            );
        }
        if trailing_acceptance {
            assert!(
                state
                    .display_snapshot
                    .as_ref()
                    .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts),
                "{case}",
            );
        }
        let mut legacy_probe = state.game_execution_scheduler;
        assert_eq!(
            legacy_probe.advance_startup_sequence(),
            Some(StartupSequenceStep::CompleteFileSelectGraphics),
            "{case}: Live Continued authority must not decrement the one remaining legacy slice",
        );
    }
}

#[test]
fn live_file_select_waiting_publishes_only_the_committed_low_wram_clear_prefix() {
    let mut state = live_file_select_waiting_state(false, false, 0, 968);
    state.ram[0x0d00..0x1000].fill(0x5a);
    state.game_state.sprites.sprite_slots = SpriteSlotsState::load_from_ram(&state.ram);
    let progress = FileSelectGraphicsLowWramClearProgress {
        word_offset: 0xfe,
        completed_page_stores: 1,
    };
    let semantic = &mut state
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic;
    semantic.insert(
        semantic.len() - 1,
        OriginalTimingSemanticReceipt::FileSelectGraphicsLowWramClearProgress(progress),
    );

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(&state.ram[0x0dfe..0x0e00], &[0, 0]);
    assert_eq!(&state.ram[0x0efe..0x0f00], &[0x5a, 0x5a]);
    assert_eq!(&state.ram[0x0ffe..0x1000], &[0x5a, 0x5a]);
    assert_eq!(state.ram[0x0dfc], 0x5a);
    assert_eq!(
        state.game_state.sprites.sprite_slots,
        SpriteSlotsState::load_from_ram(&state.ram),
    );
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn file_select_low_wram_clear_precedes_its_trailing_held_acceptance() {
    // Cold pinned-ROM host1019: [Held, Handler, ClearReturned, Held, Continued].
    let mut state = live_file_select_waiting_state(false, true, 0, 1019);
    state.ram[0x0d00..0x1000].fill(0x5a);
    state.game_state.sprites.sprite_slots = SpriteSlotsState::load_from_ram(&state.ram);
    state
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .insert(
            2,
            OriginalTimingSemanticReceipt::FileSelectGraphicsLowWramCleared,
        );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.ram[0x0d00..0x1000].iter().all(|byte| *byte == 0));
    assert!(state.original_timing_nmi_publication_pending);
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn live_file_select_waiting_rejects_bad_gate_snapshot_or_phase_before_mutation() {
    for malformed in ["wrong-gate", "closed-carry", "resume-module"] {
        let mut state = live_file_select_waiting_state(
            malformed != "wrong-gate",
            false,
            FILE_SELECT_GRAPHICS_NMI_SLICES - 1,
            967,
        );
        match malformed {
            "wrong-gate" => {
                state
                    .original_timing_semantic_receipts
                    .as_mut()
                    .unwrap()
                    .semantic[0] = OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open);
                state.original_timing_expected_nmi_update_gates[0] = NmiUpdateGate::Open;
            }
            "closed-carry" => {
                state
                    .display_snapshot
                    .as_mut()
                    .unwrap()
                    .accepts_nmi_dma_receipts = false;
            }
            "resume-module" => {
                assert_eq!(
                    state.game_execution_scheduler.advance_startup_sequence(),
                    Some(StartupSequenceStep::CompleteFileSelectGraphics),
                );
            }
            _ => unreachable!(),
        }
        let scheduler = state.game_execution_scheduler;
        let semantic = state.original_timing_semantic_receipts.clone();
        let gates = state.original_timing_expected_nmi_update_gates.clone();
        let pending = state.original_timing_nmi_publication_pending;
        let pending_gate = state.original_timing_pending_nmi_update_gate;
        let latch = state.game_state.display.nmi_update_is_latched();
        let suffix = state.pending_main_loop_common_suffix;
        let snapshot = state.display_snapshot.as_ref().map(|snapshot| {
            (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.vram.clone(),
            )
        });

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal(0, crate::RUN_MAIN);
        }));

        assert!(result.is_err(), "{malformed}");
        assert_eq!(state.game_execution_scheduler, scheduler, "{malformed}");
        assert_eq!(
            state.original_timing_semantic_receipts, semantic,
            "{malformed}"
        );
        assert_eq!(
            state.original_timing_expected_nmi_update_gates, gates,
            "{malformed}"
        );
        assert_eq!(
            state.original_timing_nmi_publication_pending, pending,
            "{malformed}"
        );
        assert_eq!(
            state.original_timing_pending_nmi_update_gate, pending_gate,
            "{malformed}"
        );
        assert_eq!(
            state.game_state.display.nmi_update_is_latched(),
            latch,
            "{malformed}"
        );
        assert_eq!(state.pending_main_loop_common_suffix, suffix, "{malformed}");
        assert_eq!(
            state.display_snapshot.as_ref().map(|snapshot| (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.vram.clone(),
            )),
            snapshot,
            "{malformed}",
        );
    }
}

#[test]
fn terminal_selected_game_load_completes_host2291_held_nmi_before_host2292_return() {
    let mut state = live_selected_game_load_state_before_terminal_hosts();

    // Source host 2291 completes the previously held handler, remains inside
    // the selected-game loader, then accepts another held NMI at $09:c47b.
    // Its CallStackContinued fact is nonterminal because $00:805f has not run.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2291,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::SpriteResetAllProgress(
                    crate::SpriteResetAllProgressReceipt {
                        progress: crate::SpriteResetAllProgress::SpriteDisableAllCompleted,
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
            ],
        ))
        .unwrap();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.main_module, 5);
    assert!(state.game_state.display.nmi_update_is_latched());
    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_remaining_nmi_slices(),
        2,
    );
    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_after_pre_dungeon_audio_sprite_reset(),
        Some(PreDungeonSpriteResetContinuation::SpriteDisableAllCompleted),
    );
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
    let acceptance_epoch = state.display_snapshot.as_ref().unwrap().publication_epoch;
    let captured_vram = state.display_snapshot.as_ref().unwrap().ppu.vram[0x1234];
    state.ppu.vram[0x1234] = captured_vram ^ 0xffff;
    state.set_vertical_irq_trigger(0x7b);

    // Host 2292 completes that held handler at $00:8225, resumes $09:c47b,
    // and only then returns through NMI_PrepareSprites and `$12 = 0` at
    // $00:805f. No post-return NMI belongs to this source host.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2292,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            ],
        ))
        .unwrap();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.main_module, 7);
    assert_eq!(state.game_state.frame.submodule, 15);
    assert_eq!(state.game_state.frame.subsubmodule, 0);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(state.original_timing_expected_nmi_update_gates.is_empty());
    assert_eq!(
        state.display_snapshot.as_ref().unwrap().publication_epoch,
        acceptance_epoch,
        "the carry-in completion must refine the acceptance snapshot without recapturing it",
    );
    assert_eq!(
        state.display_snapshot.as_ref().unwrap().ppu.vram[0x1234],
        captured_vram,
        "terminal live PPU work must not replace the acceptance-owned scanout base",
    );
    assert_eq!(
        state.game_state.display.vertical_irq_trigger, 0x7b,
        "the terminal resume must not replay the Module05 common prefix",
    );
    assert!(state.game_execution_scheduler.is_idle());
    assert!(state
        .game_execution_scheduler
        .returned_main_is_waiting_for_nmi());
    assert!(state.dungeon_landing_cpu_advance_pending.is_none());
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(2292));
}

#[test]
fn nonterminal_selected_game_load_completes_a_carried_held_handler_without_recapture() {
    let mut state = live_selected_game_load_state_before_terminal_hosts();
    state.ppu.vram[0x1234] = 0x2222;
    state.display_snapshot.as_mut().unwrap().ppu.vram[0x1234] = 0x1111;
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2290,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
            ],
        ))
        .unwrap();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_remaining_nmi_slices(),
        2,
    );
    assert_eq!(
        state.display_snapshot.as_ref().unwrap().ppu.vram[0x1234],
        0x1111,
        "completing the carried handler must not recapture the newer live PPU",
    );
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(2290));
}

#[test]
fn nonterminal_selected_game_load_completes_run2215_carried_open_then_accepts_held() {
    let mut state = live_selected_game_load_state_before_pre_dungeon_audio();
    state.game_execution_scheduler = GameExecutionScheduler::default();
    state
        .game_execution_scheduler
        .schedule_selected_game_load(SelectedGameLoadDestination::Dungeon);
    for _ in 0..SELECTED_GAME_LOAD_BEFORE_PRE_DUNGEON_AUDIO_NMI_SLICES - 2 {
        assert_eq!(
            state.game_execution_scheduler.advance_startup_sequence(),
            Some(StartupSequenceStep::SelectedGameLoadWaiting),
        );
    }
    let remaining_before = state
        .game_execution_scheduler
        .selected_game_load_remaining_nmi_slices();
    state.clear_nmi_update_latch();
    state.capture_display_snapshot();
    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .accepts_nmi_dma_receipts = true;
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2215,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                    high: 0x80,
                    low: 0,
                    high_filtered: 0,
                    low_filtered: 0,
                }),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
            ],
        ))
        .unwrap();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_remaining_nmi_slices(),
        remaining_before - 1,
    );
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(2215));
}

#[test]
fn nonterminal_selected_game_load_rejects_misplaced_open_handler_joypad_before_mutation() {
    let joypad = JoypadPublication {
        high: 0x80,
        low: 0,
        high_filtered: 0,
        low_filtered: 0,
    };
    for semantic in [
        vec![
            OriginalTimingSemanticReceipt::JoypadPublication(joypad),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
        ],
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::JoypadPublication(joypad),
        ],
    ] {
        let mut state = live_selected_game_load_state_before_pre_dungeon_audio();
        state.game_execution_scheduler.reset();
        state
            .game_execution_scheduler
            .schedule_selected_game_load(SelectedGameLoadDestination::Dungeon);
        for _ in 0..SELECTED_GAME_LOAD_BEFORE_PRE_DUNGEON_AUDIO_NMI_SLICES - 2 {
            assert_eq!(
                state.game_execution_scheduler.advance_startup_sequence(),
                Some(StartupSequenceStep::SelectedGameLoadWaiting),
            );
        }
        state.clear_nmi_update_latch();
        state.capture_display_snapshot();
        state
            .display_snapshot
            .as_mut()
            .unwrap()
            .accepts_nmi_dma_receipts = true;
        state.original_timing_nmi_publication_pending = true;
        state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        state.original_timing_expected_nmi_update_gates =
            vec![NmiUpdateGate::Open, NmiUpdateGate::LatchHeld];
        state.original_timing_semantic_receipts =
            Some(OriginalTimingHostReceipts::new(2215, 0, semantic));
        state.set_ambient_sound_effect(3);
        state.set_sound_effect_1(0x44);

        let scheduler_before = state.game_execution_scheduler;
        let receipts_before = state.original_timing_semantic_receipts.clone();
        let gates_before = state.original_timing_expected_nmi_update_gates.clone();
        let frame_before = state.game_state.frame;
        let suffix_before = state.pending_main_loop_common_suffix;
        let snapshot_before = state.display_snapshot.as_ref().map(|snapshot| {
            (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.vram.clone(),
            )
        });

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
        assert_eq!(state.game_state.frame, frame_before);
        assert_eq!(state.pending_main_loop_common_suffix, suffix_before);
        assert_eq!(state.game_state.system_signals.ambient_sound_effect(), 3);
        assert_eq!(
            state.game_state.system_signals.last_ambient_sound_effect(),
            0
        );
        assert_eq!(state.game_state.system_signals.sound_effect_1(), 0x44);
        assert!(state.original_timing_nmi_publication_pending);
        assert_eq!(
            state.original_timing_pending_nmi_update_gate,
            Some(NmiUpdateGate::Open),
        );
        assert_eq!(
            state.display_snapshot.as_ref().map(|snapshot| (
                snapshot.publication_epoch,
                snapshot.accepts_nmi_dma_receipts,
                snapshot.ppu.vram.clone(),
            )),
            snapshot_before,
        );
    }
}

#[test]
fn nonterminal_selected_game_load_accepts_and_completes_a_held_handler_in_sequence() {
    let mut state = live_selected_game_load_state_before_terminal_hosts();
    state.original_timing_nmi_publication_pending = false;
    state.original_timing_pending_nmi_update_gate = None;
    state.display_snapshot = None;
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2217,
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

    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_remaining_nmi_slices(),
        2,
    );
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(2217));
}

#[test]
fn nonterminal_selected_game_load_rejects_unowned_or_open_nmi_slices_before_advancing() {
    let malformed = [
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        )],
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
        ],
    ];

    for semantic in malformed {
        let mut state = live_selected_game_load_state_before_terminal_hosts();
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        state.original_timing_semantic_receipts =
            Some(OriginalTimingHostReceipts::new(2290, 0, semantic));
        let receipts_before = state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic
            .clone();
        let frame_before = state.game_state.frame;
        let suffix_before = state.pending_main_loop_common_suffix;

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal(0, crate::RUN_MAIN);
        }));

        assert!(result.is_err());
        assert_eq!(
            state
                .game_execution_scheduler
                .selected_game_load_remaining_nmi_slices(),
            2,
        );
        assert_eq!(state.game_state.frame, frame_before);
        assert_eq!(state.pending_main_loop_common_suffix, suffix_before);
        assert!(state.game_state.display.nmi_update_is_latched());
        assert!(state.original_timing_nmi_publication_pending);
        assert_eq!(
            state.original_timing_pending_nmi_update_gate,
            Some(NmiUpdateGate::LatchHeld),
        );
        assert_eq!(
            state
                .original_timing_semantic_receipts
                .as_ref()
                .unwrap()
                .semantic,
            receipts_before,
        );
    }

    for corrupt_owner in [0_u8, 1, 2] {
        let mut state = live_selected_game_load_state_before_terminal_hosts();
        match corrupt_owner {
            0 => state.pending_main_loop_common_suffix = None,
            1 => state.clear_nmi_update_latch(),
            2 => {
                state.original_timing_nmi_publication_pending = false;
                state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
            }
            _ => unreachable!(),
        }
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
            2290,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
            ],
        ));
        let frame_before = state.game_state.frame;
        let suffix_before = state.pending_main_loop_common_suffix;
        let latch_before = state.game_state.display.nmi_update_is_latched();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal(0, crate::RUN_MAIN);
        }));

        assert!(result.is_err());
        assert_eq!(
            state
                .game_execution_scheduler
                .selected_game_load_remaining_nmi_slices(),
            2,
        );
        assert_eq!(state.game_state.frame, frame_before);
        assert_eq!(state.pending_main_loop_common_suffix, suffix_before);
        assert_eq!(
            state.game_state.display.nmi_update_is_latched(),
            latch_before,
        );
        assert!(state.original_timing_semantic_receipts.is_some());
    }
}

#[test]
fn terminal_selected_game_load_preflight_is_failure_atomic() {
    for malformed in [
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        )],
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    ] {
        let mut state = live_selected_game_load_state_before_terminal_hosts();
        mark_selected_game_load_sprite_disable_all_completed(&mut state);
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        state.original_timing_semantic_receipts =
            Some(OriginalTimingHostReceipts::new(2292, 0, malformed));
        let receipts_before = state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic
            .clone();
        let module_before = state.game_state.frame;
        let suffix_before = state.pending_main_loop_common_suffix;

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal(0, crate::RUN_MAIN);
        }));

        assert!(result.is_err());
        assert_eq!(
            state
                .game_execution_scheduler
                .selected_game_load_after_pre_dungeon_audio_sprite_reset(),
            Some(PreDungeonSpriteResetContinuation::SpriteDisableAllCompleted),
        );
        assert_eq!(state.game_state.frame, module_before);
        assert!(state.game_state.display.nmi_update_is_latched());
        assert_eq!(state.pending_main_loop_common_suffix, suffix_before);
        assert!(state.original_timing_nmi_publication_pending);
        assert_eq!(
            state.original_timing_pending_nmi_update_gate,
            Some(NmiUpdateGate::LatchHeld),
        );
        assert_eq!(
            state
                .original_timing_semantic_receipts
                .as_ref()
                .unwrap()
                .semantic,
            receipts_before,
        );
    }
}

#[test]
fn terminal_selected_game_load_requires_receptive_held_carry_in_before_advancing() {
    let mut state = live_selected_game_load_state_before_terminal_hosts();
    mark_selected_game_load_sprite_disable_all_completed(&mut state);
    state.display_snapshot = None;
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        2292,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    ));
    let receipts_before = state
        .original_timing_semantic_receipts
        .as_ref()
        .unwrap()
        .semantic
        .clone();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.run_frame_internal(0, crate::RUN_MAIN);
    }));

    assert!(result.is_err());
    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_after_pre_dungeon_audio_sprite_reset(),
        Some(PreDungeonSpriteResetContinuation::SpriteDisableAllCompleted),
    );
    assert_eq!(state.game_state.frame.main_module, 5);
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert!(state.display_snapshot.is_none());
    assert_eq!(
        state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic,
        receipts_before,
    );
}

#[test]
fn terminal_selected_game_load_rejects_a_nonheld_carry_in_before_advancing() {
    let mut state = live_selected_game_load_state_before_terminal_hosts();
    mark_selected_game_load_sprite_disable_all_completed(&mut state);
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        2292,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    ));
    let receipts_before = state
        .original_timing_semantic_receipts
        .as_ref()
        .unwrap()
        .semantic
        .clone();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.run_frame_internal(0, crate::RUN_MAIN);
    }));

    assert!(result.is_err());
    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_after_pre_dungeon_audio_sprite_reset(),
        Some(PreDungeonSpriteResetContinuation::SpriteDisableAllCompleted),
    );
    assert_eq!(state.game_state.frame.main_module, 5);
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::Open),
    );
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert_eq!(
        state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic,
        receipts_before,
    );
}

#[test]
fn terminal_file_select_graphics_return_captures_the_trailing_open_nmi_for_the_next_host() {
    let mut state =
        live_file_select_graphics_state_before_slice(FILE_SELECT_GRAPHICS_NMI_SLICES - 1);
    install_zero_file_select_test_assets(&mut state);

    state.ppu.vram[0x1000] = 0x1111;
    state.capture_display_snapshot();
    state.ppu.vram[0x1000] = 0x2222;

    // Source host 1022 finishes its leading held handler before the suspended
    // file-select caller. The caller then returns through the exact common
    // suffix and accepts an Open NMI at the main wait.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            1022,
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
        state.with_display_snapshot(|display| display.ppu.vram[0x1000]),
        0x2222,
        "the acceptance host must publish its post-return scanout",
    );
    state.advance_display_publication_history();
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));

    // Host 1023 completes that exact Open handler before beginning the next
    // FileSelect iteration. The handler and its joypad publication execute
    // once against the receptive snapshot retained across presentation.
    state.set_sound_effect_1(0x67);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            1023,
            0,
            vec![
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

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.zelda_debug_apu_write_ports()[2], 0x67);
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    let link = &state.game_state.player.follower_link;
    assert_eq!(link.joypad1h_last(), 0xa5);
    assert_eq!(link.joypad1l_last(), 0x5a);
    assert_eq!(link.filtered_joypad_h(), 0x81);
    assert_eq!(link.filtered_joypad_l(), 0x42);
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(1023));
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn live_nonterminal_file_select_graphics_retains_loading_at_legacy_count_one() {
    let mut state =
        live_file_select_graphics_state_before_slice(FILE_SELECT_GRAPHICS_NMI_SLICES - 1);
    state.ppu.vram[0x1000] = 0x1357;
    state.capture_display_snapshot();
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            1022,
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

    let mut scheduler = state.game_execution_scheduler;
    assert_eq!(
        scheduler.advance_startup_sequence(),
        Some(StartupSequenceStep::CompleteFileSelectGraphics),
        "the Continued receipt must retain the last legacy Loading slice",
    );
    assert_eq!(state.ppu.vram[0x1000], 0x1357);
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(state.pending_main_loop_common_suffix.is_some());
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn live_terminal_file_select_graphics_ignores_the_legacy_remaining_count() {
    let mut state =
        live_file_select_graphics_state_before_slice(FILE_SELECT_GRAPHICS_NMI_SLICES - 2);
    install_zero_file_select_test_assets(&mut state);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            1022,
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
    let mut legacy_probe = state.game_execution_scheduler;
    assert_eq!(
        legacy_probe.advance_startup_sequence(),
        Some(StartupSequenceStep::FileSelectWaiting),
        "the legacy estimate still has two slices remaining",
    );

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
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
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn terminal_file_select_graphics_prevalidates_nmi_phases_before_advancing() {
    let mut state =
        live_file_select_graphics_state_before_slice(FILE_SELECT_GRAPHICS_NMI_SLICES - 1);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        1022,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    ));
    let receipts_before = state
        .original_timing_semantic_receipts
        .as_ref()
        .unwrap()
        .semantic
        .clone();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.run_frame_internal(0, crate::RUN_MAIN);
    }));

    assert!(result.is_err());
    let mut scheduler = state.game_execution_scheduler;
    assert_eq!(
        scheduler.advance_startup_sequence(),
        Some(StartupSequenceStep::CompleteFileSelectGraphics),
    );
    assert!(state.game_state.display.nmi_update_is_latched());
    assert_eq!(
        state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic,
        receipts_before,
    );
}

#[test]
fn terminal_file_select_graphics_requires_receptive_carry_in_before_advancing() {
    let mut state =
        live_file_select_graphics_state_before_slice(FILE_SELECT_GRAPHICS_NMI_SLICES - 1);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    state.display_snapshot = None;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        1022,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    ));
    let receipts_before = state
        .original_timing_semantic_receipts
        .as_ref()
        .unwrap()
        .semantic
        .clone();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.run_frame_internal(0, crate::RUN_MAIN);
    }));

    assert!(result.is_err());
    let mut scheduler = state.game_execution_scheduler;
    assert_eq!(
        scheduler.advance_startup_sequence(),
        Some(StartupSequenceStep::CompleteFileSelectGraphics),
    );
    assert!(state.original_timing_nmi_publication_pending);
    assert_eq!(
        state.original_timing_pending_nmi_update_gate,
        Some(NmiUpdateGate::LatchHeld),
    );
    assert!(state.display_snapshot.is_none());
    assert_eq!(
        state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic,
        receipts_before,
    );
}

#[test]
fn file_select_checkerboard_finishes_through_the_pre_main_dispatcher() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(0xa680);
    state.set_main_module(3);
    state.set_submodule(1);

    state.module_erase_file_1();

    assert!(state
        .pre_main_caller_continuation_is(PreMainCallerContinuation::FileSelectCheckerboardUpload));
    assert_eq!(state.game_state.frame.submodule, 1);

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert!(state
        .game_execution_scheduler
        .pre_main_caller_continuation()
        .is_none());
    assert_eq!(state.game_state.frame.submodule, 2);
    assert_eq!(state.game_state.display.bg_vram_load_mode, 0);
}

#[test]
fn file_select_terminal_caller_runs_held_nmi_before_suffix_and_carries_open_nmi() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(1);
    state.set_submodule(2);
    state.set_subsubmodule(249);
    state.set_frame_counter(220);

    // The preceding source host returned inside SelectFile_Func1 after its
    // checkerboard CPU work had started but before the call returned through
    // ZeldaRunGameLoop's shared suffix.
    state.module_erase_file_1();
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state.set_sound_effect_1(0x56);
    assert!(state
        .pre_main_caller_continuation_is(PreMainCallerContinuation::FileSelectCheckerboardUpload));

    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            1025,
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

    // Interrupt_NMI_AudioParts_Locked ran once while `$12` was still held.
    // It therefore did not consume the checkerboard upload or publish joypad
    // state. Only afterward did the CPU finish SelectFile_Func1 and the shared
    // suffix clear `$12`; the following open NMI remains pending.
    assert_eq!(state.zelda_debug_apu_write_ports()[2], 0x56);
    assert_eq!(state.game_state.system_signals.sound_effect_1(), 0);
    assert_eq!(state.game_state.frame.frame_counter, 220);
    assert_eq!(state.game_state.frame.submodule, 3);
    assert_eq!(state.game_state.frame.subsubmodule, 249);
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
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(1025));
    assert!(state.original_timing_semantic_receipts.is_none());

    state.set_sound_effect_1(0x67);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            1026,
            0,
            vec![
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
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            ],
        ))
        .unwrap();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.zelda_debug_apu_write_ports()[2], 0x67);
    assert_eq!(state.game_state.system_signals.sound_effect_1(), 0);
    assert_eq!(state.game_state.frame.frame_counter, 221);
    assert_eq!(state.game_state.frame.submodule, 4);
    assert_eq!(state.game_state.display.bg_vram_load_mode, 6);
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    let link = &state.game_state.player.follower_link;
    assert_eq!(link.joypad1h_last(), 0xa5);
    assert_eq!(link.joypad1l_last(), 0x5a);
    assert_eq!(link.filtered_joypad_h(), 0x81);
    assert_eq!(link.filtered_joypad_l(), 0x42);
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(1026));
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn file_select_main_publishes_display_memory_at_the_following_nmi() {
    assert!(rom_display_memory_publication_is_deferred(1, 5, 0, true));
    assert!(!rom_display_memory_publication_is_deferred(1, 4, 0, false));
    assert!(!rom_display_memory_publication_is_deferred(2, 5, 0, false));

    let mut state = ZeldaState::new();
    state.set_main_module(1);
    state.set_submodule(5);
    state.ppu.vram[0] = 0x1111;
    state.ppu.oam[0] = 0x2222;
    state.ppu.cgram[0] = 0x3333;
    state.ram[NMI_LOAD_BG_FROM_VRAM] = 1;
    state.capture_display_snapshot();
    state.ppu.vram[0] = 0xaaaa;
    state.ppu.oam[0] = 0xbbbb;
    state.ppu.cgram[0] = 0xcccc;

    let captured = state.with_display_snapshot(|display| {
        (
            display.ppu.vram[0],
            display.ppu.oam[0],
            display.ppu.cgram[0],
        )
    });

    assert_eq!(captured, (0x1111, 0x2222, 0x3333));
    assert_eq!(state.ppu.vram[0], 0xaaaa);
    assert_eq!(state.ppu.oam[0], 0xbbbb);
    assert_eq!(state.ppu.cgram[0], 0xcccc);
}

#[test]
fn selected_game_load_resumes_until_the_cpu_heavy_setup_finishes() {
    let mut scheduler = GameExecutionScheduler::default();
    scheduler.schedule_selected_game_load(SelectedGameLoadDestination::Dungeon);
    assert_eq!(
        scheduler.selected_game_load_remaining_nmi_slices(),
        SELECTED_GAME_LOAD_NMI_SLICES
    );

    for _ in 0..SELECTED_GAME_LOAD_BEFORE_PRE_DUNGEON_AUDIO_NMI_SLICES - 1 {
        assert_eq!(
            scheduler.advance_startup_sequence(),
            Some(StartupSequenceStep::SelectedGameLoadWaiting)
        );
    }
    assert_eq!(
        scheduler.selected_game_load_remaining_nmi_slices(),
        SELECTED_GAME_LOAD_AFTER_PRE_DUNGEON_AUDIO_NMI_SLICES + 1
    );
    assert_eq!(
        scheduler.advance_startup_sequence(),
        Some(StartupSequenceStep::BeginPreDungeonAudio)
    );
    assert_eq!(
        scheduler.selected_game_load_remaining_nmi_slices(),
        SELECTED_GAME_LOAD_AFTER_PRE_DUNGEON_AUDIO_NMI_SLICES
    );

    for _ in 0..SELECTED_GAME_LOAD_AFTER_PRE_DUNGEON_AUDIO_NMI_SLICES - 1 {
        assert_eq!(
            scheduler.advance_startup_sequence(),
            Some(StartupSequenceStep::SelectedGameLoadWaiting)
        );
    }
    assert_eq!(
        scheduler.advance_startup_sequence(),
        Some(StartupSequenceStep::CompleteSelectedGameLoad)
    );
    assert!(scheduler.is_idle());
}

#[test]
fn rom_intro_poly_thread_begins_on_the_measured_frame() {
    assert_eq!(configured_intro_thread_start_delay(), 0);
}

#[test]
fn rom_intro_poly_thread_remains_concurrent_during_title_fade() {
    for submodule in [3, 4, 5, 7, 9, 11] {
        assert!(rom_intro_poly_thread_is_active(0, submodule));
    }
    assert!(!rom_intro_poly_thread_is_active(0, 6));
    assert!(!rom_intro_poly_thread_is_active(1, 5));
    assert!(rom_intro_wait_player_tears_down_poly_thread(0, 8, true));
    assert!(!rom_intro_wait_player_tears_down_poly_thread(0, 8, false));
    assert!(!rom_intro_wait_player_tears_down_poly_thread(0, 7, true));
    assert_eq!(
        [0, 1, 2, 0, 1, 2]
            .into_iter()
            .map(rom_intro_title_fade_runs_main)
            .collect::<Vec<_>>(),
        vec![true, true, false, true, true, false]
    );
    assert_eq!(
        [0, 1, 2, 0, 1, 2]
            .into_iter()
            .map(rom_intro_title_fade_should_yield_suffix)
            .collect::<Vec<_>>(),
        vec![false, true, false, false, true, false]
    );
}

#[test]
fn legacy_poly_stack_marker_cannot_own_a_frame_after_worker_shutdown() {
    assert!(legacy_poly_scheduler_is_active(0, false, true));
    assert!(!legacy_poly_scheduler_is_active(0, false, false));
    assert!(!legacy_poly_scheduler_is_active(
        BUGFIX_POLY_RENDERER,
        false,
        true,
    ));
    assert!(!legacy_poly_scheduler_is_active(0, true, true));
}

#[test]
fn carry_in_held_nmi_completion_preserves_the_already_returned_poly_scanout() {
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

    // The preceding source host returned this scanout before accepting its
    // latch-held NMI. Live PPU memory may already belong to the following
    // field when the next host completes that carried handler.
    state.ppu.vram[0x5800..0x5c00].fill(0x1357);
    let mut accepted_obj_latch = vec![0; state.ppu.vram.len()];
    accepted_obj_latch[0x5a20] = 0x3579;
    accepted_obj_latch[0x5a40] = 0x579b;
    state.set_obj_vram_latch_traced(Some(accepted_obj_latch));
    state.capture_display_snapshot();
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
    state.ppu.vram[0x5800..0x5c00].fill(0x2468);
    let mut following_obj_latch = vec![0; state.ppu.vram.len()];
    following_obj_latch[0x5a20] = 0x468a;
    following_obj_latch[0x5a40] = 0x68ac;
    state.set_obj_vram_latch_traced(Some(following_obj_latch));
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            806,
            0x0008,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            ],
        ))
        .unwrap();

    state.run_frame_internal(0x0008, crate::RUN_MAIN);

    assert_eq!(
        state.display_snapshot.as_ref().unwrap().ppu.vram[0x5800..0x5c00],
        [0x1357; 0x400],
        "completing a carry-in handler must not recapture the next live poly generation",
    );
    assert_eq!(state.ppu.vram[0x5800..0x5c00], [0x2468; 0x400]);
    assert_eq!(
        state
            .display_snapshot
            .as_ref()
            .unwrap()
            .ppu
            .obj_vram_latch
            .as_ref()
            .unwrap()[0x5a20],
        0x3579,
        "the carry-in held handler must retain the acceptance-host OBJ page 2 latch",
    );
    assert_eq!(
        state
            .display_snapshot
            .as_ref()
            .unwrap()
            .ppu
            .obj_vram_latch
            .as_ref()
            .unwrap()[0x5a40],
        0x579b,
    );
    assert_eq!(
        state.ppu.obj_vram_latch.as_ref().unwrap()[0x5a20],
        0x468a,
        "the following live OBJ page 2 latch remains separately owned",
    );
    assert_eq!(state.ppu.obj_vram_latch.as_ref().unwrap()[0x5a40], 0x68ac,);
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(806));
    assert!(state.original_timing_semantic_receipts.is_none());
}

#[test]
fn file_select_teardown_shares_the_handoff_frame_with_outgoing_poly_worker() {
    assert!(rom_file_select_teardown_runs_with_outgoing_poly_worker(
        1, 0, true, true
    ));
    assert!(!rom_file_select_teardown_runs_with_outgoing_poly_worker(
        1, 1, true, true
    ));
    assert!(!rom_file_select_teardown_runs_with_outgoing_poly_worker(
        1, 0, false, true
    ));
    assert!(!rom_file_select_teardown_runs_with_outgoing_poly_worker(
        0, 7, true, true
    ));
}

#[test]
fn rom_intro_background_fade_preserves_cooperative_poly_cadence() {
    let mut carry_frames = 0;
    let mut poly_phase = 0;
    let mut decisions = Vec::new();
    let mut suffix_yields = Vec::new();

    for _ in 0..12 {
        let (run_main, yield_before_suffix, next_carry_frames, next_poly_phase) =
            rom_intro_bg_fade_main_decision(carry_frames, poly_phase);
        decisions.push(run_main);
        suffix_yields.push(yield_before_suffix);
        carry_frames = next_carry_frames;
        poly_phase = next_poly_phase;
    }

    assert_eq!(
        decisions,
        vec![true, true, true, true, true, true, false, true, true, true, true, false]
    );
    assert_eq!(
        suffix_yields,
        vec![false, false, false, false, false, true, false, false, false, false, true, false]
    );
    assert!(rom_intro_bg_fade_should_yield_suffix(true, 2, 5));
    assert!(rom_intro_bg_fade_should_yield_suffix(true, 2, 4));
    assert!(!rom_intro_bg_fade_should_yield_suffix(true, 2, 3));
    assert!(!rom_intro_bg_fade_should_yield_suffix(true, 4, 5));
    assert!(!rom_intro_bg_fade_should_yield_suffix(false, 2, 5));
}

#[test]
fn bare_live_call_stack_progress_cannot_expire_from_poly_only_idle_dispatch() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(0);
    state.set_submodule(6);
    state.set_subsubmodule(42);
    state.set_frame_counter(105);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            848,
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
            ],
        ))
        .unwrap();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.run_frame_internal(0, crate::RUN_POLY);
    }));

    assert!(result.is_err());
    assert_eq!(state.game_state.frame.frame_counter, 105);
    assert_eq!(state.game_state.frame.subsubmodule, 42);
    assert!(!state.game_state.display.nmi_thread_active);
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(848));
    assert_eq!(
        state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic(),
        &[
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
        ],
    );
    assert!(state.original_timing_host_dispatch_active);
}

#[test]
fn title_poly_thread_teardown_defers_the_next_main_loop_tick() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_main_module(0);
    state.set_submodule(6);
    state.set_subsubmodule(42);
    state.activate_nmi_thread();

    state.intro_sword_coming_down();

    assert_eq!(state.game_state.frame.subsubmodule, 41);
    assert!(!state.game_state.display.nmi_thread_active);
    assert!(state.intro_poly_thread_teardown_pending);
}

#[test]
fn rom_intro_waits_for_poly_thread_completion_before_advancing_again() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.intro_memory_darken_frame_delay = 0;
    state.intro_poly_upload_delay = 0;
    state.attract_scene_mut().set_intro_step_index(1);
    state.attract_scene_mut().mark_intro_did_run_step();
    state.poly_runtime_mut().set_config1(165);

    state.intro_animate_triforce();

    assert_eq!(state.game_state.poly.runtime.config1(), 165);
    assert_eq!(
        state.game_state.ending.attract_scene.intro_did_run_step(),
        1
    );
}

#[test]
fn main_loop_does_not_complete_poly_work_that_was_not_scheduled() {
    let mut state = ZeldaState::new();
    state.initialized = true;
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.intro_memory_darken_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(0);
    state.set_submodule(4);
    state.set_frame_counter(0x86);
    state.set_bg_mode(9);
    state.attract_scene_mut().set_intro_step_index(1);
    state.attract_scene_mut().mark_intro_did_run_step();
    state.clear_pending_polyhedral_update();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(
        state.game_state.ending.attract_scene.intro_did_run_step(),
        1
    );
    assert!(!state.game_state.display.has_pending_polyhedral_update());
}

#[test]
fn poly_worker_budget_tracks_geometry_cost_instead_of_route_frames() {
    let mut state = ZeldaState::new();
    state.poly_runtime_mut().set_model(1);
    state.poly_runtime_mut().set_base_x(32);
    state.poly_runtime_mut().set_base_y(32);

    state.poly_runtime_mut().set_config1(175);
    state.poly_runtime_mut().set_angle_a(216);
    state.poly_runtime_mut().set_angle_b(104);
    state.poly_run_frame();
    assert_eq!(
        state.debug_last_poly_work(),
        PolyWorkMetrics {
            divide_calls: 12,
            divide_shifts: 12,
            faces: 5,
            visible_faces: 3,
            edge_segments: 11,
            scanlines: 24,
            span_words: 62,
        }
    );
    assert_eq!(state.debug_last_poly_work().worker_frames(), 1);

    state.poly_runtime_mut().set_config1(255);
    state.poly_runtime_mut().set_angle_a(244);
    state.poly_runtime_mut().set_angle_b(236);
    state.poly_run_frame();
    assert_eq!(
        state.debug_last_poly_work(),
        PolyWorkMetrics {
            divide_calls: 12,
            divide_shifts: 24,
            faces: 5,
            visible_faces: 3,
            edge_segments: 9,
            scanlines: 25,
            span_words: 62,
        }
    );
    assert_eq!(state.debug_last_poly_work().worker_frames(), 2);
}

#[test]
fn poly_worker_cost_model_handles_sparse_wide_faces() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_main_module(0);
    state.set_submodule(4);
    state.attract_scene_mut().set_intro_step_index(1);
    state.attract_scene_mut().mark_intro_did_run_step();
    state.clear_pending_polyhedral_update();
    state.poly_runtime_mut().set_model(1);
    state.poly_runtime_mut().set_base_x(32);
    state.poly_runtime_mut().set_base_y(32);
    state.poly_runtime_mut().set_config1(67);
    state.poly_runtime_mut().set_angle_a(122);
    state.poly_runtime_mut().set_angle_b(118);

    state.zelda_run_poly_loop();

    assert_eq!(state.debug_last_poly_work().worker_frames(), 1);
    assert_eq!(
        state.game_state.ending.attract_scene.intro_did_run_step(),
        0
    );
    assert!(state.game_state.display.has_pending_polyhedral_update());
}

#[test]
fn staged_attract_exit_does_not_rewrite_the_drained_projection() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.ppu.forced_blank = false;

    // Seed the staged pipeline, then author the final visible field.
    state.set_screen_brightness(3);
    state.ppu.brightness = 3;
    state.ppu.vram[0] = 0xaaaa;
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::AdvanceStaged);
    state.set_screen_brightness(2);
    state.ppu.brightness = 2;
    state.ppu.vram[0] = 0x1111;
    state.attract_map_hdma_projection_before = Some(vec![0; ATTRACT_MAP_PROJECTION_WORDS * 2]);
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::AdvanceStaged);

    // The following CPU generation requests force-blank and clears VRAM before
    // its NMI. Advancing the staged pipeline must drain the visible field
    // without merging either future write back into it.
    state.set_screen_brightness(0x80);
    state.ppu.vram[0] = 0;
    state.capture_display_snapshot_with_publication(DisplaySnapshotPublication::AdvanceStaged);
    assert_eq!(state.display_snapshot.as_ref().unwrap().ppu.vram[0], 0x1111);
    assert!(!state.display_snapshot.as_ref().unwrap().ppu.forced_blank);
    assert_eq!(
        state.deferred_display_snapshot.as_ref().unwrap().ram
            [crate::game_state::constants::INIDISP_COPY],
        0x80
    );
    state.ppu.forced_blank = true;
    state.ppu.brightness = 0;

    let presented = state.with_display_snapshot(|display| {
        (
            display.ppu.vram[0],
            display.ppu.forced_blank,
            display.ppu.brightness,
        )
    });

    assert_eq!(presented, (0x1111, false, 2));
    assert_eq!(state.ppu.vram[0], 0);
    assert!(state.ppu.forced_blank);
}

#[test]
fn first_intro_step_matches_top_level_state_writes() {
    let mut state = ZeldaState::new();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.submodule, 1);
    assert_eq!(state.game_state.frame.subsubmodule, 1);
    assert_eq!(state.game_state.display.screen_brightness, 15);
    assert_eq!(state.game_state.display.main_screen_layers, 16);
    assert_eq!(state.game_state.display.bg_mode, 9);
    assert_eq!(
        state
            .game_state
            .display
            .palette_filter
            .color_window_selection(),
        0x20
    );
    assert_eq!(
        state.game_state.display.palette_filter.color_math_control(),
        0x20
    );
    assert_eq!(
        state.game_state.display.palette_filter.fixed_color_red(),
        0x20
    );
    assert_eq!(
        state.game_state.display.palette_filter.fixed_color_green(),
        0x40
    );
    assert_eq!(
        state.game_state.display.palette_filter.fixed_color_blue(),
        0x80
    );
    assert_eq!(state.game_state.display.core_update_disable_flag, 0x80);
    assert_eq!(state.game_state.display.nmi_load_target_page(), 0x46);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0);
    assert_eq!(state.game_state.system_signals.sound_effect_2(), 0);
    assert_eq!(state.game_state.dungeon.scratch_word.primary_word(), 0x1bfe);
    assert_eq!(
        state.game_state.dungeon.scratch_word.secondary_word(),
        0x17fe
    );
    assert_eq!(
        &state.ram[OAM_BUF..OAM_BUF + 16],
        &[
            0x60, 0x68, 0x69, 0x32, 0x70, 0x68, 0x6b, 0x32, 0x80, 0x68, 0x6d, 0x32, 0x88, 0x68,
            0x6e, 0x32
        ]
    );
    assert_eq!(
        &state.ram[BYTEWISE_EXTENDED_OAM..BYTEWISE_EXTENDED_OAM + 4],
        &[2; 4]
    );
    assert_eq!(state.ram[EXTENDED_OAM], 0xaa);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_3), 0x8080);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_0), 0x8280);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_4), 0x8840);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_1), 0x8a40);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_5), 0x9a40);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_2), 0x9a40);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_6), 0x9000);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_11), 0x9180);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_7), 0x9300);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_12), 0x93c0);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_8), 0x9480);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_13), 0x9560);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_10), 0xa480);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_15), 0xa580);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_16), 0xb940);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_18), 0xbb40);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_17), 0xb940);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_19), 0xbb40);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_20), 0xb540);
    assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_21), 0xb740);
    assert_eq!(read_le_u16(&state.ram, BG_TILE_ANIMATION_COUNTDOWN), 0xffff);
    assert_eq!(link_test_word(&state, LINK_DMA_COUNTDOWN), 0xffff);
    assert_eq!(state.ppu.cgram[144], 0x7fff);
}

#[test]
fn intro_fade_in_bg_start_skips_to_file_select_loader() {
    let mut state = ZeldaState::new();
    state.set_main_module(0);
    state.set_submodule(7);
    state.set_subsubmodule(0xf3);
    state.set_countdown(0);
    state.follower_link_state_mut().set_filtered_joypad_h(0x10);
    state.set_indoor_flag(1);
    set_link_test_byte(&mut state, LINK_Y_COORD, 0x12);
    state.ram[LINK_Y_COORD + 0x6f] = 0x34;
    state.save_progress_mut().set_dungeon_info_word(0, 0x56);

    state.module00_intro();

    assert_eq!(state.game_state.display.irq_control_flag, 0xff);
    assert_eq!(state.game_state.display.main_screen_layers, 0x15);
    assert_eq!(state.game_state.display.sub_screen_layers, 0);
    assert_eq!(state.game_state.world.location.indoor_flag(), 0);
    assert_eq!(state.game_state.system_signals.music_control(), 0xf1);
    assert_eq!(state.game_state.frame.main_module, 1);
    assert_eq!(state.game_state.frame.submodule, 0);
    assert_eq!(state.ram[RESTART_CHECK_FLAG], 1);
    assert_eq!(link_test_byte(&state, LINK_Y_COORD), 0);
    assert_eq!(state.ram[LINK_Y_COORD + 0x6f], 0);
    assert_eq!(state.ram[SAVE_DUNG_INFO], 0);
}

#[test]
fn fire_debirando_nested_property_reset_resumes_from_the_exact_source_call() {
    let mut split = ZeldaState::new();
    let slot = 0;
    {
        let mut sprite = split.sprite_slot_view_mut(slot);
        sprite.set_state(8);
        sprite.set_sprite_type(0x64);
        sprite.set_head_direction(0x0e);
        sprite.set_ai_state(2);
    }
    let mut atomic = split.clone();

    split.sprite_module_initialize_properties(slot);
    split.sprite_slot_view_mut(slot).set_sprite_type(0x63);
    split.sprite_prep_reset_properties_prefix(slot, 1);
    {
        let sprite = split.sprite_slot_view(slot);
        assert_eq!(
            sprite.state(),
            9,
            "the outer property load already returned"
        );
        assert_eq!(
            sprite.sprite_type(),
            0x63,
            "the conversion precedes the nested reset"
        );
        assert_eq!(
            sprite.head_direction(),
            0,
            "the outer reset already cleared this field"
        );
        assert_eq!(
            sprite.ai_state(),
            0,
            "the outer reset already cleared this field"
        );
    }

    split.sprite_prep_reset_properties_from(slot, 1);
    split.sprite_prep_load_properties_after_reset(slot);
    split.sprite_prep_fire_debirando_after_property_reload(slot);
    atomic.sprite_module_initialize(slot);

    assert_eq!(
        split.game_state.sprites.sprite_slots, atomic.game_state.sprites.sprite_slots,
        "split execution must equal the atomic Fire Debirando initializer",
    );
}
