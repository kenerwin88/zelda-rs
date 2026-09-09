use super::*;
use crate::dialogue_ir::{
    DialogueIrKind, TEXT_CMD_2, TEXT_CMD_COLOR, TEXT_CMD_END_MESSAGE, TEXT_CMD_NAME,
    TEXT_CMD_NUMBER, TEXT_CMD_WAIT, TEXT_COMMAND_START_US,
};
use crate::game_state::constants::{
    ANCILLA_ALLOC_ROTATE, ANCILLA_TYPE, ANCILLA_X_LO, ANCILLA_X_VELOCITY, ANIMATED_TILE_DATA_SRC,
    BG2_X_SCROLL, BG_TILE_ANIMATION_COUNTDOWN, DIALOGUE_MESSAGE_INDEX, DMA_SOURCE_ADDR_0,
    DMA_SOURCE_ADDR_1, DMA_SOURCE_ADDR_10, DMA_SOURCE_ADDR_11, DMA_SOURCE_ADDR_12,
    DMA_SOURCE_ADDR_13, DMA_SOURCE_ADDR_14, DMA_SOURCE_ADDR_15, DMA_SOURCE_ADDR_16,
    DMA_SOURCE_ADDR_17, DMA_SOURCE_ADDR_18, DMA_SOURCE_ADDR_19, DMA_SOURCE_ADDR_2,
    DMA_SOURCE_ADDR_20, DMA_SOURCE_ADDR_21, DMA_SOURCE_ADDR_3, DMA_SOURCE_ADDR_4,
    DMA_SOURCE_ADDR_5, DMA_SOURCE_ADDR_6, DMA_SOURCE_ADDR_7, DMA_SOURCE_ADDR_8, DMA_SOURCE_ADDR_9,
    DUNG_BG1, DUNG_BG2, HDMAEN_COPY, INIDISP_COPY, MOSAIC_TARGET_LEVEL, PALETTE_FILTER_COUNTDOWN,
    SUBSUBMODULE, TM_COPY, TS_COPY,
};
use crate::game_state::constants::{FLAG_IS_ANCILLA_TO_PICK_UP, SPRITE_LIMIT_INSTANCE};
use crate::game_state::constants::{MAP16_LOAD_DST_OFF, MAP16_LOAD_SRC_OFF, MAP16_LOAD_Y_UNIT};
use crate::game_state::CachedSpriteCacheField;
use crate::game_state::FrameState;
use crate::{JoypadPublication, NmiUpdateGate};

fn dungeon_reset_progress_receipt(
    progress: DungeonResetSpritesCpuProgress,
    boundary: OriginalTimingBoundary,
) -> OriginalTimingSemanticReceipt {
    OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(DungeonResetSpritesProgressReceipt {
        progress,
        boundary,
    })
}

fn cached_sprite_progress_receipt(
    progress: crate::CachedSpriteExecutionProgress,
    boundary: OriginalTimingBoundary,
) -> OriginalTimingSemanticReceipt {
    OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(
        crate::CachedSpriteExecutionProgressReceipt { progress, boundary },
    )
}

fn test_sync_all(state: &mut ZeldaState) {
    state.ram[0x42] = state.ram[0x42].wrapping_add(1);
}

fn test_run_frame_callback(state: &mut ZeldaState, input: u16, run_what: i32) {
    state.ram[0x43] = state.ram[0x43].wrapping_add(1);
    state.ram[0x44] = input as u8;
    state.ram[0x45] = run_what as u8;
}

fn test_run_frame_callback_calls_internal(state: &mut ZeldaState, input: u16, _: i32) {
    state.ram[0x43] = state.ram[0x43].wrapping_add(1);
    state.run_frame_internal(input, 0);
}

fn test_run_frame_callback_observes_snes9x_semantic_receipts(
    state: &mut ZeldaState,
    input: u16,
    _: i32,
) {
    assert!(state.original_timing_host_dispatch_active);
    assert!(matches!(
        state.original_timing_owner,
        OriginalTimingOwnerState::Live
    ));
    let receipts = state
        .original_timing_semantic_receipts
        .as_ref()
        .expect("oracle semantic receipts remain visible during this host dispatch");
    assert_eq!(receipts.input_state, input);
    assert_eq!(
        receipts.semantic(),
        &[
            dungeon_reset_progress_receipt(
                sprite::DungeonResetSpritesCpuProgress::Load(
                    sprite::DungeonLoadSpritesCpuProgress {
                        normal_load_ordinal: 1,
                        slot: 1,
                        checkpoint: sprite::DungeonSpriteLoadCheckpoint::YHigh,
                    },
                ),
                OriginalTimingBoundary::NmiAccepted,
            ),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ]
    );
    assert!(state
        .take_original_timing_dungeon_reset_sprites_progress()
        .is_some());
    state.ram[0x43] = state.ram[0x43].wrapping_add(1);
}

fn exact_timing_test_rom(opcode: u8) -> Vec<u8> {
    let mut rom = vec![opcode; 0x8000];
    rom[0x7ffc] = 0x00;
    rom[0x7ffd] = 0x80;
    rom
}

fn link_test_byte(state: &ZeldaState, addr: usize) -> u8 {
    state.ram[addr]
}

fn set_link_test_byte(state: &mut ZeldaState, addr: usize, value: u8) {
    state.ram[addr] = value;
}

fn link_test_word(state: &ZeldaState, addr: usize) -> u16 {
    read_le_u16(&state.ram, addr)
}

fn set_link_test_word(state: &mut ZeldaState, addr: usize, value: u16) {
    write_le_u16(&mut state.ram, addr, value);
}

fn put_test_asset(data: &mut Vec<u8>, ranges: &mut [(usize, usize)], index: usize, bytes: Vec<u8>) {
    let start = data.len();
    data.extend(bytes);
    ranges[index] = (start, data.len());
}

fn pack_test_memblk_arrays(items: &[Vec<u8>]) -> Vec<u8> {
    assert!(!items.is_empty());
    let payload_before_last = items[..items.len() - 1].iter().map(Vec::len).sum::<usize>();
    let wide_offsets = payload_before_last >= 65536;
    let mut data = Vec::new();
    let mut offset = 0usize;
    for item in &items[..items.len() - 1] {
        offset += item.len();
        if wide_offsets {
            data.extend_from_slice(&(offset as u32).to_le_bytes());
        } else {
            data.extend_from_slice(&(offset as u16).to_le_bytes());
        }
    }
    for item in items {
        data.extend_from_slice(item);
    }
    let marker = if wide_offsets {
        8192 + items.len() - 1
    } else {
        items.len() - 1
    };
    data.extend_from_slice(&(marker as u16).to_le_bytes());
    data
}

fn dialogue_source_sidecar_asset(messages: &[Vec<u8>]) -> Vec<u8> {
    let state = ZeldaState::new();
    let table = messages
        .iter()
        .map(|message| state.dialogue_ir_for_decoded_bytes(message))
        .collect::<Vec<_>>();
    let mut asset = DIALOGUE_SOURCE_SIDECAR_MAGIC.to_vec();
    asset.extend(bincode::serialize(&table).unwrap());
    asset
}

fn asset_pack_with_named_assets(
    data: Vec<u8>,
    ranges: Vec<(usize, usize)>,
    named_assets: &[(usize, &str)],
) -> AssetPack {
    let mut names = vec![String::new(); ranges.len()];
    for &(index, name) in named_assets {
        names[index] = name.to_string();
    }
    AssetPack::from_named_data_ranges(data, ranges, names)
}

fn test_asset_pack_bytes(named_assets: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut key_signature = Vec::new();
    for &(name, _) in named_assets {
        key_signature.extend_from_slice(name.as_bytes());
        key_signature.push(0);
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(ASSET_SIGNATURE_PREFIX);
    bytes.extend_from_slice(&[0; 64]);
    bytes.extend_from_slice(&(named_assets.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(key_signature.len() as u32).to_le_bytes());
    for (_, payload) in named_assets {
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    }
    bytes.extend_from_slice(&key_signature);
    for (_, payload) in named_assets {
        while bytes.len() & 3 != 0 {
            bytes.push(0);
        }
        bytes.extend_from_slice(payload);
    }
    bytes
}

fn probe_entrance_asset_pack(entrance_index: usize, room: u16) -> AssetPack {
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 56];
    let byte_len = entrance_index + 1;
    let word_len = (entrance_index + 1) * 2;

    let mut rooms = vec![0; word_len];
    write_le_u16(&mut rooms, entrance_index * 2, room);
    put_test_asset(&mut data, &mut ranges, 11, rooms);

    put_test_asset(&mut data, &mut ranges, 12, vec![0; byte_len * 8]);
    for asset in [13, 14, 15, 16, 17, 18, 26] {
        put_test_asset(&mut data, &mut ranges, asset, vec![0; word_len]);
    }
    for asset in [19, 20, 21, 22, 23, 24, 25, 27] {
        put_test_asset(&mut data, &mut ranges, asset, vec![0; byte_len]);
    }
    put_test_asset(&mut data, &mut ranges, 53, Vec::new());
    put_test_asset(&mut data, &mut ranges, 54, vec![0; 116]);
    put_test_asset(&mut data, &mut ranges, 55, Vec::new());

    AssetPack::from_data_ranges(data, ranges)
}

fn probe_overworld_asset_pack(screen: usize) -> AssetPack {
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 109];
    put_test_asset(&mut data, &mut ranges, 107, vec![1; screen + 1]);
    put_test_asset(&mut data, &mut ranges, 108, vec![0; screen + 1]);
    AssetPack::from_data_ranges(data, ranges)
}

fn run4782_spotlight_progress_receipt() -> OriginalTimingSemanticReceipt {
    OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
        crate::SpotlightTableBuildProgressReceipt {
            progress: crate::SpotlightTableBuildProgress {
                completed_iterations: 216,
                checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                    pending_circle_input: 23,
                },
            },
            boundary: OriginalTimingBoundary::NmiAccepted,
        },
    )
}

fn run4782_spotlight_semantic() -> Vec<OriginalTimingSemanticReceipt> {
    vec![
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
            high: 4,
            low: 0,
            high_filtered: 0,
            low_filtered: 0,
        }),
        OriginalTimingSemanticReceipt::MainLoopProgress(crate::MainLoopProgress::IterationStarted),
        OriginalTimingSemanticReceipt::SpriteMainReturned,
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        run4782_spotlight_progress_receipt(),
    ]
}

fn run4784_terminal_spotlight_semantic() -> Vec<OriginalTimingSemanticReceipt> {
    vec![
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ),
        OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
    ]
}

fn run4784_terminal_spotlight_state() -> ZeldaState {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_subsubmodule(0);
    state.set_frame_counter(176);
    state.set_indoor_flag(0);
    state.set_animated_tile_data_source_address(1);
    state.latch_nmi_update();
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    state.schedule_spotlight_iteration_return(SpotlightIteration::closing(
        SpotlightIterationPhase::CloseEntryBeforeTablePublication,
    ));
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            4_784,
            0,
            run4784_terminal_spotlight_semantic(),
        ))
        .unwrap();
    state
}

fn terminal_recurring_spotlight_state(iteration: SpotlightIteration) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_subsubmodule(0);
    state.set_frame_counter(177);
    state.set_indoor_flag(0);
    state.follower_link_state_mut().set_y(8692);
    state.set_bg2_v_copy2(8466);
    state.set_spotlight_window_radius(112);
    state.set_spotlight_window_state(0);
    state.set_animated_tile_data_source_address(1);
    state.latch_nmi_update();
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    state.schedule_dungeon_exit_spotlight_link_oam(iteration);
    let work = GameWorkContinuation::FinishDungeonExitSpotlightLinkOam { iteration };
    assert_eq!(state.game_execution_scheduler.current_work(), Some(work));
    assert_eq!(
        state
            .game_execution_scheduler
            .advance_work_one_nmi_slice_with_authoritative_completion(false),
        Some(GameWorkStep::Waiting),
    );
    assert_eq!(
        state
            .game_execution_scheduler
            .scheduled_work_slices_remaining(),
        Some(0),
    );
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            4_787,
            0,
            run4784_terminal_spotlight_semantic(),
        ))
        .unwrap();
    state
}

fn run4787_terminal_recurring_spotlight_state() -> ZeldaState {
    terminal_recurring_spotlight_state(SpotlightIteration::closing(
        SpotlightIterationPhase::WholeTable,
    ))
}

fn run4786_spotlight_build_link_oam_state() -> ZeldaState {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_subsubmodule(0);
    state.set_frame_counter(177);
    state.set_indoor_flag(0);
    state.follower_link_state_mut().set_x(680);
    state.follower_link_state_mut().set_y(8692);
    state.set_bg2_h_copy2(554);
    state.set_bg2_v_copy2(8466);
    state.set_spotlight_window_radius(119);
    state.set_spotlight_window_state(0);
    state.set_animated_tile_data_source_address(1);
    write_le_u16(&mut state.ram, LINK_DMA_COUNTDOWN, 9);
    state.set_bg_tile_animation_countdown(7);
    state.latch_nmi_update();
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    let iteration = SpotlightIteration::closing(SpotlightIterationPhase::WholeTable);
    assert!(state.begin_dungeon_exit_spotlight_build(
        None,
        Some(crate::SpotlightTableBuildProgress {
            completed_iterations: 105,
            checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeIterationInitialization,
        }),
        iteration,
    ));
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonExitSpotlightBuild { .. })
    ));
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            4_786,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    crate::MainLoopInterruption::LinkOam,
                ),
            ],
        ))
        .unwrap();
    state
}

fn run4788_spotlight_progress_semantic() -> Vec<OriginalTimingSemanticReceipt> {
    vec![
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
            high: 4,
            low: 0,
            high_filtered: 0,
            low_filtered: 0,
        }),
        OriginalTimingSemanticReceipt::MainLoopProgress(crate::MainLoopProgress::IterationStarted),
        OriginalTimingSemanticReceipt::SpriteMainReturned,
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
            crate::SpotlightTableBuildProgressReceipt {
                progress: crate::SpotlightTableBuildProgress {
                    completed_iterations: 224,
                    checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                        pending_circle_input: 15,
                    },
                },
                boundary: OriginalTimingBoundary::NmiAccepted,
            },
        ),
    ]
}

fn run4789_terminal_build_semantic() -> Vec<OriginalTimingSemanticReceipt> {
    vec![
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ),
        OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
    ]
}

fn run4790_spotlight_progress_semantic() -> Vec<OriginalTimingSemanticReceipt> {
    vec![
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
            high: 0,
            low: 0,
            high_filtered: 0,
            low_filtered: 0,
        }),
        OriginalTimingSemanticReceipt::MainLoopProgress(crate::MainLoopProgress::IterationStarted),
        OriginalTimingSemanticReceipt::SpriteMainReturned,
        OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
            crate::SpotlightTableBuildProgressReceipt {
                progress: crate::SpotlightTableBuildProgress {
                    completed_iterations: 229,
                    checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                        pending_circle_input: 10,
                    },
                },
                boundary: OriginalTimingBoundary::HostReturn,
            },
        ),
    ]
}

fn run4791_terminal_build_semantic() -> Vec<OriginalTimingSemanticReceipt> {
    vec![
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ),
        OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
    ]
}

/// Execute run4788's exact fresh-iteration checkpoint and install run4789's
/// carried-Held terminal return without consuming it.
fn run4789_terminal_spotlight_build_state() -> ZeldaState {
    let mut state = run4787_terminal_recurring_spotlight_state();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 112);

    state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::Open, NmiUpdateGate::LatchHeld];
    let receipts = OriginalTimingHostReceipts::new(4_788, 0, run4788_spotlight_progress_semantic());
    let wire_receipts: OriginalTimingHostReceipts = bincode::deserialize(
        &bincode::serialize(&receipts).expect("serialize run4788 source receipts"),
    )
    .expect("deserialize run4788 source receipts");
    assert_eq!(
        wire_receipts.semantic(),
        run4788_spotlight_progress_semantic(),
        "run4788 wire order must retain Held acceptance before its spotlight claim",
    );
    state
        .install_original_timing_host_receipts(wire_receipts)
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);

    let Some(GameWorkContinuation::FinishDungeonExitSpotlightBuild {
        table_build,
        projection_completed,
        iteration,
    }) = state.game_execution_scheduler.current_work()
    else {
        panic!("run4788 lost its suspended recurring spotlight Build")
    };
    assert_eq!(
        table_build,
        SpotlightTableBuildContinuation {
            vertical_center: 238,
            upper_cursor: 224,
            lower_cursor: 252,
            completed: false,
            pending_circle_input: Some(15),
            pending_lower_value: None,
            pending_loop_completion_test: false,
            pending_lower_cursor_decrement: false,
            projection_tail_cleared: false,
            projection_words_copied: 0,
            source_progress: Some(crate::SpotlightTableBuildProgress {
                completed_iterations: 224,
                checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                    pending_circle_input: 15,
                },
            }),
        },
    );
    assert!(!projection_completed);
    assert_eq!(
        iteration,
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTable),
    );
    assert_eq!(
        state
            .game_execution_scheduler
            .scheduled_work_slices_remaining(),
        Some(1),
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
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert!(!state.main_loop_sprite_preparation_completed);
    assert!(state.original_timing_semantic_receipts.is_none());

    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            4_789,
            0,
            run4789_terminal_build_semantic(),
        ))
        .unwrap();
    state
}

/// Execute run4789's terminal Build, then run4790's exact HostReturn table
/// checkpoint and install run4791's same-host Held/trailing-Open return.
fn run4791_terminal_spotlight_build_state() -> ZeldaState {
    let mut state = run4789_terminal_spotlight_build_state();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 105);
    assert!(!state.original_timing_nmi_publication_pending);

    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::Open];
    let receipts = OriginalTimingHostReceipts::new(4_790, 0, run4790_spotlight_progress_semantic());
    let wire_receipts: OriginalTimingHostReceipts = bincode::deserialize(
        &bincode::serialize(&receipts).expect("serialize run4790 source receipts"),
    )
    .expect("deserialize run4790 source receipts");
    assert_eq!(
        wire_receipts.semantic(),
        run4790_spotlight_progress_semantic(),
        "run4790 wire order must retain the HostReturn spotlight checkpoint",
    );
    state
        .install_original_timing_host_receipts(wire_receipts)
        .unwrap();
    state.run_frame_internal(0, crate::RUN_MAIN);

    let Some(GameWorkContinuation::FinishDungeonExitSpotlightBuild {
        table_build,
        projection_completed,
        iteration,
    }) = state.game_execution_scheduler.current_work()
    else {
        panic!("run4790 lost its suspended recurring spotlight Build")
    };
    assert_eq!(
        table_build,
        SpotlightTableBuildContinuation {
            vertical_center: 238,
            upper_cursor: 229,
            lower_cursor: 247,
            completed: false,
            pending_circle_input: Some(10),
            pending_lower_value: None,
            pending_loop_completion_test: false,
            pending_lower_cursor_decrement: false,
            projection_tail_cleared: false,
            projection_words_copied: 0,
            source_progress: Some(crate::SpotlightTableBuildProgress {
                completed_iterations: 229,
                checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                    pending_circle_input: 10,
                },
            }),
        },
    );
    assert!(!projection_completed);
    assert_eq!(
        iteration,
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTable),
    );
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(state.deferred_display_snapshot.is_none());
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert!(!state.main_loop_sprite_preparation_completed);

    state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::Open];
    let receipts = OriginalTimingHostReceipts::new(4_791, 0, run4791_terminal_build_semantic());
    let wire_receipts: OriginalTimingHostReceipts = bincode::deserialize(
        &bincode::serialize(&receipts).expect("serialize run4791 source receipts"),
    )
    .expect("deserialize run4791 source receipts");
    assert_eq!(
        wire_receipts.semantic(),
        run4791_terminal_build_semantic(),
        "run4791 wire order must keep the caller-return token after its trailing Open acceptance",
    );
    state
        .install_original_timing_host_receipts(wire_receipts)
        .unwrap();
    state
}

/// One spotlight-close fresh-iteration host from the run4792..run4818 window:
/// an optional leading Open acceptance, the shared handler/joypad/iteration
/// prefix, then the host-specific tail (a table checkpoint, a LinkOam
/// interruption, or nothing when the module body finished inside the host).
fn run48xx_spotlight_close_fresh_semantic(
    leading_open_acceptance: bool,
    tail: &[OriginalTimingSemanticReceipt],
) -> Vec<OriginalTimingSemanticReceipt> {
    let mut semantic = Vec::new();
    if leading_open_acceptance {
        semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open,
        ));
    }
    semantic.extend([
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
            high: 4,
            low: 0,
            high_filtered: 0,
            low_filtered: 0,
        }),
        OriginalTimingSemanticReceipt::MainLoopProgress(crate::MainLoopProgress::IterationStarted),
        OriginalTimingSemanticReceipt::SpriteMainReturned,
    ]);
    semantic.extend_from_slice(tail);
    semantic
}

fn run48xx_spotlight_projection_claim(
    copied_words: u16,
    boundary: OriginalTimingBoundary,
) -> OriginalTimingSemanticReceipt {
    OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
        crate::SpotlightTableBuildProgressReceipt {
            progress: crate::SpotlightTableBuildProgress {
                completed_iterations: 239,
                checkpoint: crate::SpotlightTableBuildCheckpoint::ProjectionCopy { copied_words },
            },
            boundary,
        },
    )
}

/// One module-qualified spotlight-close terminal return from the
/// run4793..run4817 window: a carried or same-host Held handler, the shared
/// continued-return/suffix pair, an optional trailing Open acceptance, and
/// the wire-last caller-return token.
fn run48xx_spotlight_terminal_semantic(
    same_host_held: bool,
    trailing_open: bool,
) -> Vec<OriginalTimingSemanticReceipt> {
    let mut semantic = Vec::new();
    if same_host_held {
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
    if trailing_open {
        semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open,
        ));
    }
    semantic.push(OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait);
    semantic
}

/// run4799: the leading vblank re-checkpoints the saved ProjectionCopy at its
/// acceptance boundary, then the resumed copy completes and returns.
fn run4799_recheckpointed_terminal_semantic() -> Vec<OriginalTimingSemanticReceipt> {
    vec![
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        run48xx_spotlight_projection_claim(117, OriginalTimingBoundary::NmiAccepted),
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ),
        OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
    ]
}

/// run4819: the final suffix-only continuation returns after its module body
/// already advanced the frame out of Module0F, so the adapter proves the
/// return without the module-qualified caller-return token.
fn run4819_tokenless_final_semantic() -> Vec<OriginalTimingSemanticReceipt> {
    vec![
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ),
        OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
    ]
}

fn install_run48xx_host(
    state: &mut ZeldaState,
    host_call: u64,
    semantic: Vec<OriginalTimingSemanticReceipt>,
) {
    let receipts = OriginalTimingHostReceipts::new(host_call, 0, semantic.clone());
    let wire_receipts: OriginalTimingHostReceipts = bincode::deserialize(
        &bincode::serialize(&receipts).expect("serialize run48xx source receipts"),
    )
    .expect("deserialize run48xx source receipts");
    assert_eq!(
        wire_receipts.semantic(),
        semantic,
        "run{host_call} wire order must survive serialization unchanged",
    );
    state
        .install_original_timing_host_receipts(wire_receipts)
        .unwrap();
}

fn assert_run48xx_scheduled_flagged_projection_build(
    state: &ZeldaState,
    copied_words: u16,
    label: &str,
) {
    let Some(GameWorkContinuation::FinishDungeonExitSpotlightBuild {
        table_build,
        projection_completed,
        iteration,
    }) = state.game_execution_scheduler.current_work()
    else {
        panic!("{label} lost its suspended ProjectionCopy spotlight Build")
    };
    assert_eq!(
        table_build,
        SpotlightTableBuildContinuation {
            vertical_center: 238,
            upper_cursor: 238,
            lower_cursor: 238,
            completed: true,
            pending_circle_input: None,
            pending_lower_value: None,
            pending_loop_completion_test: false,
            pending_lower_cursor_decrement: false,
            projection_tail_cleared: true,
            projection_words_copied: copied_words,
            source_progress: Some(crate::SpotlightTableBuildProgress {
                completed_iterations: 239,
                checkpoint: crate::SpotlightTableBuildCheckpoint::ProjectionCopy { copied_words },
            }),
        },
        "{label}",
    );
    assert!(!projection_completed, "{label}");
    assert_eq!(
        iteration,
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTable)
            .with_main_loop_sprite_preparation_before_second_nmi(),
        "{label}: an interrupted ProjectionCopy predicts its same-host suffix on the saved call",
    );
}

/// The live trace holds the follower and BG2 scroll fixed through the whole
/// close window (the exit walk is already over), so every fresh host enters
/// its module body from the same retained pair. Re-pin them where the
/// synthesized lineage would otherwise let the module body drift the iris
/// vertical center away from the trace.
fn pin_live_spotlight_follower(state: &mut ZeldaState) {
    state.follower_link_state_mut().set_y(8692);
    state.set_bg2_v_copy2(8466);
}

/// A synthesized fresh-host entry into the run-67132 calibration lineage's
/// ProjectionCopy window, mirroring `terminal_recurring_spotlight_state`'s
/// idiom: the frame, follower, scroll, and radius values retained from the
/// live trace (the shared follower/scroll pair keeps the iris vertical
/// center at 238 for every remaining close iteration), with the previous
/// terminal's completed common suffix and released latch.
fn spotlight_close_projection_window_state(radius: u16, frame_counter: u8) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.set_subsubmodule(0);
    state.set_frame_counter(frame_counter);
    state.set_indoor_flag(0);
    state.follower_link_state_mut().set_y(8692);
    state.set_bg2_v_copy2(8466);
    state.set_spotlight_window_radius(radius);
    state.set_spotlight_window_state(0);
    state.set_animated_tile_data_source_address(1);
    state.main_loop_sprite_preparation_completed = true;
    state
}

/// Enter the ProjectionCopy window at run4794 and prove its
/// mid-ProjectionCopy interrupt schedules the sprite-preparation estimate,
/// then install run4795's carried-Held terminal return without consuming it.
fn run4795_flagged_terminal_spotlight_build_state() -> ZeldaState {
    let mut state = spotlight_close_projection_window_state(91, 180);

    install_run48xx_host(
        &mut state,
        4_794,
        run48xx_spotlight_close_fresh_semantic(
            true,
            &[
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                run48xx_spotlight_projection_claim(2, OriginalTimingBoundary::NmiAccepted),
            ],
        ),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_run48xx_scheduled_flagged_projection_build(&state, 2, "run4794");
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
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
    );
    assert!(!state.main_loop_sprite_preparation_completed);

    install_run48xx_host(
        &mut state,
        4_795,
        run48xx_spotlight_terminal_semantic(false, false),
    );
    state
}

/// Execute run4795's flagged terminal Build, then run4796's second
/// mid-ProjectionCopy interrupt, and install run4797's flagged terminal with
/// its trailing Open acceptance.
fn run4797_flagged_trailing_open_spotlight_build_state() -> ZeldaState {
    let mut state = run4795_flagged_terminal_spotlight_build_state();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 84);

    pin_live_spotlight_follower(&mut state);
    install_run48xx_host(
        &mut state,
        4_796,
        run48xx_spotlight_close_fresh_semantic(
            true,
            &[
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                run48xx_spotlight_projection_claim(60, OriginalTimingBoundary::NmiAccepted),
            ],
        ),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_run48xx_scheduled_flagged_projection_build(&state, 60, "run4796");

    install_run48xx_host(
        &mut state,
        4_797,
        run48xx_spotlight_terminal_semantic(false, true),
    );
    state
}

/// Execute run4797, then run4798's HostReturn ProjectionCopy checkpoint (the
/// host budget expires inside the memcpy with no vblank), and install
/// run4799's re-checkpointed terminal return.
fn run4799_recheckpointed_terminal_spotlight_build_state() -> ZeldaState {
    let mut state = run4797_flagged_trailing_open_spotlight_build_state();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 77);
    assert!(state.original_timing_nmi_publication_pending);

    pin_live_spotlight_follower(&mut state);
    install_run48xx_host(
        &mut state,
        4_798,
        run48xx_spotlight_close_fresh_semantic(
            false,
            &[run48xx_spotlight_projection_claim(
                117,
                OriginalTimingBoundary::HostReturn,
            )],
        ),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_run48xx_scheduled_flagged_projection_build(&state, 117, "run4798");
    assert!(
        !state.original_timing_nmi_publication_pending,
        "run4798 ends inside the ProjectionCopy without accepting a vblank",
    );

    install_run48xx_host(
        &mut state,
        4_799,
        run4799_recheckpointed_terminal_semantic(),
    );
    state
}

/// Execute run4799..run4804 (the final table upload and the first LinkOam
/// interruptions of the tail phase), and install run4805's carried-Held
/// suffix-only terminal return.
fn run4805_carried_suffix_only_spotlight_state() -> ZeldaState {
    let mut state = run4799_recheckpointed_terminal_spotlight_build_state();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 70);

    pin_live_spotlight_follower(&mut state);
    install_run48xx_host(
        &mut state,
        4_800,
        run48xx_spotlight_close_fresh_semantic(
            false,
            &[
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                run48xx_spotlight_projection_claim(174, OriginalTimingBoundary::NmiAccepted),
            ],
        ),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_run48xx_scheduled_flagged_projection_build(&state, 174, "run4800");

    install_run48xx_host(
        &mut state,
        4_801,
        run48xx_spotlight_terminal_semantic(false, false),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 63);

    pin_live_spotlight_follower(&mut state);
    install_run48xx_host(
        &mut state,
        4_802,
        run48xx_spotlight_close_fresh_semantic(true, &[]),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpotlightIteration { .. })
    ));

    install_run48xx_host(
        &mut state,
        4_803,
        run48xx_spotlight_terminal_semantic(true, false),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());

    pin_live_spotlight_follower(&mut state);
    install_run48xx_host(
        &mut state,
        4_804,
        run48xx_spotlight_close_fresh_semantic(
            true,
            &[
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    crate::MainLoopInterruption::LinkOam,
                ),
            ],
        ),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    let Some(GameWorkContinuation::FinishSpotlightIteration { iteration }) =
        state.game_execution_scheduler.current_work()
    else {
        panic!("run4804 lost its suspended LinkOam spotlight suffix")
    };
    assert_eq!(
        iteration,
        SpotlightIteration::closing(SpotlightIterationPhase::MixedTailAfterReturn),
    );
    assert!(
        state.original_timing_nmi_publication_pending,
        "run4804's LinkOam interruption carries its Held handler into run4805",
    );
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));

    install_run48xx_host(
        &mut state,
        4_805,
        run48xx_spotlight_terminal_semantic(false, false),
    );
    state
}

/// Execute run4805, then run4806's budget-expiry LinkOam interruption, and
/// install run4807's same-host suffix-only terminal with its trailing Open.
fn run4807_trailing_open_suffix_only_spotlight_state() -> ZeldaState {
    let mut state = run4805_carried_suffix_only_spotlight_state();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 49);

    pin_live_spotlight_follower(&mut state);
    install_run48xx_host(
        &mut state,
        4_806,
        run48xx_spotlight_close_fresh_semantic(
            true,
            &[OriginalTimingSemanticReceipt::MainLoopInterrupted(
                crate::MainLoopInterruption::LinkOam,
            )],
        ),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpotlightIteration { .. })
    ));
    assert!(
        !state.original_timing_nmi_publication_pending,
        "run4806 expires inside LinkOam without accepting a vblank",
    );

    install_run48xx_host(
        &mut state,
        4_807,
        run48xx_spotlight_terminal_semantic(true, true),
    );
    state
}

/// Execute run4807..run4818 (the remaining LinkOam tail iterations, whose
/// final module body advances the frame out of Module0F), and install
/// run4819's tokenless final suffix-only return.
fn run4819_tokenless_final_spotlight_state() -> ZeldaState {
    let mut state = run4807_trailing_open_suffix_only_spotlight_state();
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 42);

    for (host_call, fresh, terminal) in [
        (4_808u64, (false, true), (false, false)),
        (4_810, (true, true), (false, false)),
        (4_812, (true, false), (true, true)),
        (4_814, (false, true), (false, true)),
        (4_816, (false, false), (true, true)),
    ] {
        let (leading_open, latch_before_interruption) = fresh;
        let mut tail = Vec::new();
        if latch_before_interruption {
            tail.push(OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::LatchHeld,
            ));
        }
        tail.push(OriginalTimingSemanticReceipt::MainLoopInterrupted(
            crate::MainLoopInterruption::LinkOam,
        ));
        pin_live_spotlight_follower(&mut state);
        install_run48xx_host(
            &mut state,
            host_call,
            run48xx_spotlight_close_fresh_semantic(leading_open, &tail),
        );
        state.run_frame_internal(0, crate::RUN_MAIN);
        assert!(
            matches!(
                state.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishSpotlightIteration { .. })
            ),
            "run{host_call} lost its suspended LinkOam spotlight suffix",
        );

        let (same_host_held, trailing_open) = terminal;
        install_run48xx_host(
            &mut state,
            host_call + 1,
            run48xx_spotlight_terminal_semantic(same_host_held, trailing_open),
        );
        state.run_frame_internal(0, crate::RUN_MAIN);
        assert!(
            state.game_execution_scheduler.is_idle(),
            "run{} left successor scheduled work",
            host_call + 1,
        );
    }
    assert_eq!(state.game_state.display.spotlight_hdma.window_radius(), 7);
    assert_eq!(state.game_state.frame.main_module, 0x0f);
    assert_eq!(state.game_state.frame.submodule, 1);

    pin_live_spotlight_follower(&mut state);
    install_run48xx_host(
        &mut state,
        4_818,
        run48xx_spotlight_close_fresh_semantic(
            false,
            &[
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    crate::MainLoopInterruption::LinkOam,
                ),
            ],
        ),
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpotlightIteration { .. })
    ));
    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        0,
        "run4818's module body writes the final zero radius",
    );
    assert_ne!(
        (
            state.game_state.frame.main_module,
            state.game_state.frame.submodule,
        ),
        (0x0f, 1),
        "run4818's module body advances the frame out of Module0F before its caller returns",
    );
    assert!(state.original_timing_nmi_publication_pending);

    install_run48xx_host(&mut state, 4_819, run4819_tokenless_final_semantic());
    state
}

fn live_dungeon_spotlight_caller_before_terminal_return(remaining_slices: u8) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(7);
    // Keep this non-ROM lifecycle fixture outside the Module-7 CPU timing
    // states. The continuation and typed return receipt own the generic
    // handler/caller/suffix/carry ordering tested below; pinned-ROM coverage
    // separately owns instruction timing for submodule $0f itself.
    state.set_submodule(14);
    state.set_subsubmodule(1);
    state.set_frame_counter(125);
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state.dungeon_landing_entry_started_after_leading_nmi = true;
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
        remaining_slices,
    );
    state
}

fn dungeon_spotlight_terminal_return_receipts(host_call: u64) -> OriginalTimingHostReceipts {
    OriginalTimingHostReceipts::new(
        host_call,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    )
}

fn dungeon_spotlight_terminal_return_without_trailing_nmi_receipts(
    host_call: u64,
) -> OriginalTimingHostReceipts {
    OriginalTimingHostReceipts::new(
        host_call,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    )
}

fn dungeon_spotlight_carried_terminal_return_receipts(
    host_call: u64,
) -> OriginalTimingHostReceipts {
    OriginalTimingHostReceipts::new(
        host_call,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    )
}

fn carry_dungeon_spotlight_nmi_from_acceptance_host(
    state: &mut ZeldaState,
    host_call: u64,
    gate: NmiUpdateGate,
) {
    match gate {
        NmiUpdateGate::Open => state.clear_nmi_update_latch(),
        NmiUpdateGate::LatchHeld => state.latch_nmi_update(),
    }
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            host_call,
            0,
            vec![OriginalTimingSemanticReceipt::NmiAccepted(gate)],
        ))
        .unwrap();
    let owns_dispatch = state.begin_original_timing_host_dispatch(0);
    assert!(owns_dispatch);
    let phases = state.take_original_timing_nmi_phases();
    assert_eq!(phases, [OriginalTimingNmiPhase::Accepted(gate)]);
    let classification = classify_original_timing_nmi_phases_with_ownership(false, &phases);
    assert_eq!(
        classification.handler_completion,
        OriginalTimingNmiHandlerCompletionOwner::None,
    );
    assert!(classification.publication_pending_at_exit);
    state.capture_and_carry_original_timing_nmi_publication_at_host_return();
    state.finish_original_timing_host_dispatch(owns_dispatch);
    assert!(state.original_timing_semantic_receipts.is_none());
}

fn live_dialogue_terminal_return_state(endpoint: u16) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(14);
    state.set_submodule(2);
    state.set_bg2_h_copy2(0x1010);
    state.set_bg2_v_copy2(0x2020);
    state.set_bg1_h_copy2(0x3030);
    state.set_bg1_v_copy2(0x4040);
    state.set_bg1_x_offset(3);
    state.set_bg1_y_offset(5);
    state
        .messaging_state_mut()
        .set_dialogue_msg_read_pos(endpoint);
    // The initializer's source caller has already selected the active
    // RenderText state by the time its final character-buffer suffix resumes.
    state.messaging_state_mut().set_module(1);
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state.ppu.vram[0x5a20] = 0x1357;
    state.capture_display_snapshot();
    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .accepts_nmi_dma_receipts = true;
    state.ppu.vram[0x5a20] = 0x2468;
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    state.set_pending_nmi_subroutine(0x77);
    state.set_core_update_disable_flag(4);
    state.set_ambient_sound_effect(3);
    state.follower_link_state_mut().set_joypad1h_last(0xa5);
    state
        .game_execution_scheduler
        .schedule_after_current_trailing_nmi(
            GameWorkContinuation::FinishDialogueInitializationCallerReturn,
        );
    state
}

fn dialogue_terminal_return_receipts(host: u64) -> OriginalTimingHostReceipts {
    dialogue_terminal_return_receipts_for_nmi_cell(host, true, false)
}

fn dialogue_terminal_return_receipts_for_nmi_cell(
    host: u64,
    publication_pending_at_entry: bool,
    trailing_open_acceptance: bool,
) -> OriginalTimingHostReceipts {
    // `$0e:c58b` is outside the VWF endpoint range. The terminal host therefore
    // publishes one Held completion, the continued call stack, and the exact
    // ZeldaRunGameLoop common suffix. Raster position decides whether the Held
    // acceptance belongs to the preceding host or this host, and whether an
    // Open acceptance follows the suffix.
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
    OriginalTimingHostReceipts::new(host, 0, semantic)
}

fn ordinary_dialogue_iteration_receipts(host: u64) -> OriginalTimingHostReceipts {
    OriginalTimingHostReceipts::new(
        host,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                high: 0x40,
                low: 0x02,
                high_filtered: 0x40,
                low_filtered: 0x02,
            }),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::IterationStarted,
            ),
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    )
}

fn live_idle_dialogue_main_loop_state() -> ZeldaState {
    let mut state = live_dialogue_terminal_return_state(0x37);
    state.game_execution_scheduler = GameExecutionScheduler::default();
    state.pending_main_loop_common_suffix = None;
    state.main_loop_sprite_preparation_completed = false;
    state.original_timing_nmi_publication_pending = false;
    state.original_timing_pending_nmi_update_gate = None;
    state.original_timing_expected_nmi_update_gates.clear();
    state.original_timing_semantic_receipts = None;
    state.original_timing_last_oracle_host_call = None;
    state.set_pending_nmi_subroutine(0);
    state.set_core_update_disable_flag(0);
    state.clear_nmi_update_latch();
    state
}

fn live_idle_terminal_dialogue_endpoint_state(endpoint: u16) -> ZeldaState {
    let mut state = live_idle_dialogue_main_loop_state();
    state.messaging_state_mut().set_text_render_state(3);
    state
        .messaging_state_mut()
        .set_dialogue_msg_read_pos(endpoint);
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state.dialogue_fast_forward_hold_pending = false;
    state
}

fn idle_terminal_dialogue_endpoint_receipts(
    host: u64,
    endpoint: u16,
) -> OriginalTimingHostReceipts {
    OriginalTimingHostReceipts::new(
        host,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                crate::DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                    message_read_position: endpoint,
                },
            ),
        ],
    )
}

fn idle_terminal_dialogue_endpoint_with_trailing_open_receipts(
    host: u64,
    endpoint: u16,
) -> OriginalTimingHostReceipts {
    OriginalTimingHostReceipts::new(
        host,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                crate::DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                    message_read_position: endpoint,
                },
            ),
        ],
    )
}

fn idle_terminal_suspended_vwf_receipts(host: u64) -> OriginalTimingHostReceipts {
    OriginalTimingHostReceipts::new(
        host,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    )
}

fn live_idle_terminal_suspended_vwf_state() -> ZeldaState {
    let mut state = live_idle_terminal_dialogue_endpoint_state(25);
    state.messaging_text_mut().load_decoded_dialogue(&[0; 27]);
    state.messaging_state_mut().set_vwf_line_speed_cur(3);
    state.dialogue_fast_forward_hold_active = true;
    state.capture_display_snapshot();
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    state
        .install_original_timing_host_receipts(idle_terminal_suspended_vwf_receipts(3_823))
        .unwrap();
    state
}

fn intro_memory_darken_suspended_state(
    pending_at_entry: bool,
    trailing_acceptance: bool,
    host_call: u64,
) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(0);
    state.set_submodule(1);
    state.set_subsubmodule(26);
    state.intro_memory_darken_frame_delay = 1;
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    if pending_at_entry {
        state.ppu.vram[0] = host_call as u16;
        state.capture_display_snapshot();
        state.original_timing_nmi_publication_pending = true;
        state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    }
    let mut semantic = Vec::new();
    if !pending_at_entry {
        semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::LatchHeld,
        ));
    }
    semantic.push(OriginalTimingSemanticReceipt::NmiHandlerCompleted);
    if trailing_acceptance {
        semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::LatchHeld,
        ));
    }
    semantic.push(OriginalTimingSemanticReceipt::MainLoopProgress(
        crate::MainLoopProgress::CallStackContinued,
    ));
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            host_call, 0, semantic,
        ))
        .unwrap();
    state
}

fn injected_dungeon_cpu_schedule(
    submodule_nmis: u8,
    caller_nmis: u8,
) -> DungeonSubmoduleCpuSchedule {
    DungeonSubmoduleCpuSchedule {
        submodule_nmis,
        caller_nmis,
        caller_sprite_main_nmis: 0,
        caller_suffix_nmis: caller_nmis,
        caller_first_nmi_phase: None,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
        reenters_main_loop_before_nmi: false,
    }
}

fn spiral_cpu_test_state(subsubmodule: u8, schedule: DungeonSubmoduleCpuSchedule) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_subsubmodule(subsubmodule);
    state.dungeon_submodule_cpu_schedule = Some(schedule);
    state
}

fn live_terminal_ground_item_receipt_state_with_gfx(gfx: u8) -> ZeldaState {
    const ANIMATED_SOURCE: usize = 0xa680;
    const ANIMATED_DESTINATION: usize = 0x3b00;

    let item = match gfx {
        0x06 => 0x00,
        0x14 => 0x12,
        0x21 => 0x33,
        0x22 => 0x32,
        _ => panic!("terminal ground-item test needs a source-coherent item for gfx {gfx:02x}"),
    };
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(0);
    state.set_subsubmodule(0);
    state.set_frame_counter(0xfa);
    state.set_animated_tile_data_source_address(ANIMATED_SOURCE as u16);
    state.set_animated_tile_vram_destination_address(ANIMATED_DESTINATION as u16);
    state.ram[ANIMATED_SOURCE..ANIMATED_SOURCE + 0x400].fill(0x5a);
    state.ppu.vram[ANIMATED_DESTINATION] = 0x1111;
    state.capture_display_snapshot();
    state.ppu.vram[ANIMATED_DESTINATION] = 0;
    state.latch_nmi_update();
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishItemReceiptGraphics {
            continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                gfx,
                ground_apress_tail: Some(ItemReceiptReturn {
                    ancilla_slot: 4,
                    item,
                    chest_position: 0x0182,
                }),
            },
        },
        1,
    );
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            4_586,
            0,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::SpriteMainReturned,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    crate::MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            ],
        ))
        .unwrap();
    state
}

fn live_terminal_ground_item_receipt_state() -> ZeldaState {
    live_terminal_ground_item_receipt_state_with_gfx(0x14)
}

fn live_file_select_graphics_state_before_slice(completed_waiting_slices: u8) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(1);
    state.set_submodule(1);
    state.set_subsubmodule(249);
    state
        .game_execution_scheduler
        .schedule_file_select_graphics();
    for _ in 0..completed_waiting_slices {
        assert_eq!(
            state.game_execution_scheduler.advance_startup_sequence(),
            Some(StartupSequenceStep::FileSelectWaiting),
        );
    }
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state
}

fn install_zero_file_select_test_assets(state: &mut ZeldaState) {
    // Sheets below 103 are stored raw by the Rust asset pack. The remaining
    // file-select sheets use two LC-LZ2 fill commands for exactly 0x600 zero
    // bytes, followed by the source terminator.
    let compressed_zero_sheet = vec![0xe7, 0xff, 0, 0xe5, 0xff, 0, 0xff];
    let sprite_packs = (0..=0x6b)
        .map(|gfx| {
            if gfx < 103 {
                vec![0; 0x600]
            } else {
                compressed_zero_sheet.clone()
            }
        })
        .collect::<Vec<_>>();
    let mut asset_data = Vec::new();
    let mut asset_ranges = vec![(0, 0); 65];
    put_test_asset(&mut asset_data, &mut asset_ranges, 56, vec![0; 0x800]);
    put_test_asset(
        &mut asset_data,
        &mut asset_ranges,
        64,
        pack_test_memblk_arrays(&sprite_packs),
    );
    state.assets = Some(AssetPack::from_data_ranges(asset_data, asset_ranges));
}

fn live_file_select_waiting_state(
    pending_at_entry: bool,
    trailing_acceptance: bool,
    completed_waiting_slices: u8,
    host_call: u64,
) -> ZeldaState {
    let mut state = live_file_select_graphics_state_before_slice(completed_waiting_slices);
    if pending_at_entry {
        state.ppu.vram[0] = host_call as u16;
        state.capture_display_snapshot();
        state.original_timing_nmi_publication_pending = true;
        state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    }
    let mut semantic = Vec::new();
    if !pending_at_entry {
        semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::LatchHeld,
        ));
    }
    semantic.push(OriginalTimingSemanticReceipt::NmiHandlerCompleted);
    if trailing_acceptance {
        semantic.push(OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::LatchHeld,
        ));
    }
    semantic.push(OriginalTimingSemanticReceipt::MainLoopProgress(
        crate::MainLoopProgress::CallStackContinued,
    ));
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            host_call, 0, semantic,
        ))
        .unwrap();
    state
}

fn live_selected_game_load_state_before_terminal_hosts() -> ZeldaState {
    let asset_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("zelda3_assets.dat");
    let assets = std::fs::read(&asset_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", asset_path.display()));

    let mut state = ZeldaState::new();
    state.assets = Some(AssetPack::parse(&assets).unwrap());
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(5);
    state.set_submodule(0);
    state.set_subsubmodule(0);
    state
        .game_execution_scheduler
        .schedule_selected_game_load(SelectedGameLoadDestination::Dungeon);
    for _ in 0..SELECTED_GAME_LOAD_NMI_SLICES - 2 {
        assert!(state
            .game_execution_scheduler
            .advance_startup_sequence()
            .is_some());
    }
    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_remaining_nmi_slices(),
        2,
    );
    state.enable_force_blank();
    state.begin_selected_game_load_pre_dungeon_audio(SelectedGameLoadDestination::Dungeon);
    state.complete_pending_selected_game_load_entry_room_load();
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state.capture_display_snapshot();
    state
        .display_snapshot
        .as_mut()
        .unwrap()
        .accepts_nmi_dma_receipts = true;
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    state
}

fn live_selected_game_load_state_before_pre_dungeon_audio() -> ZeldaState {
    let asset_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("zelda3_assets.dat");
    let assets = std::fs::read(&asset_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", asset_path.display()));

    let mut state = ZeldaState::new();
    state.assets = Some(AssetPack::parse(&assets).unwrap());
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(5);
    state.set_submodule(0);
    state.set_subsubmodule(0);
    state
        .game_execution_scheduler
        .schedule_selected_game_load(SelectedGameLoadDestination::Dungeon);
    for _ in 0..SELECTED_GAME_LOAD_BEFORE_PRE_DUNGEON_AUDIO_NMI_SLICES - 1 {
        assert_eq!(
            state.game_execution_scheduler.advance_startup_sequence(),
            Some(StartupSequenceStep::SelectedGameLoadWaiting),
        );
    }
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.latch_nmi_update();
    state
}

fn mark_selected_game_load_sprite_disable_all_completed(state: &mut ZeldaState) {
    assert_eq!(
        state
            .game_execution_scheduler
            .advance_selected_game_load_from_source(
                Some(crate::SpriteResetAllProgress::SpriteDisableAllCompleted),
                false,
            ),
        Some(StartupSequenceStep::SelectedGameLoadWaiting),
    );
}

fn early_dialogue_completion_state(pending_subroutine: u8) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(0xa680);
    state.set_main_module(14);
    state.set_submodule(2);
    state.messaging_state_mut().set_module(1);
    state.messaging_state_mut().set_text_render_state(5);
    state.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::BeforeNextVblank,
    );
    for index in 0..crate::PresentedDialogueText::WORD_COUNT {
        state.set_messaging_render_buffer_word(index, 0);
    }
    state.clear_nmi_update_latch();
    state.set_pending_nmi_subroutine(pending_subroutine);
    state.set_core_update_disable_flag(2);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state
        .install_original_timing_host_receipts(
            OriginalTimingHostReceipts::new(
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
                    OriginalTimingSemanticReceipt::MainLoopProgress(
                        crate::MainLoopProgress::IterationStarted,
                    ),
                    OriginalTimingSemanticReceipt::SpriteMainReturned,
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
    state
}

fn publish_staged_dialogue_text_dma(state: &mut ZeldaState) {
    let staged = state
        .dialogue_scroll_completion_staged
        .clone()
        .expect("test dialogue publication requires a staged text generation");
    if !state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts)
    {
        state.capture_display_snapshot();
    }
    for (offset, &word) in staged.vram.iter().enumerate() {
        write_le_u16(&mut state.ram, 0x10000 + offset * 2, word);
    }
    state.bg3_vwf_glyph_runs = staged.glyph_runs.clone();
    state.bg3_vwf_glyph_run_dialogue_offsets = staged.glyph_run_dialogue_offsets.clone();
    state.bg3_vwf_glyph_run_dialogue_message_id = staged.dialogue_message_id;
    state
        .messaging_state_mut()
        .set_dialogue_msg_read_pos(staged.dialogue_msg_read_pos);
    state.clear_nmi_update_latch();
    state.set_pending_nmi_subroutine(2);
    state.set_core_update_disable_flag(2);
    assert!(
        state
            .interrupt_nmi_for_active_scanout(0, None, false)
            .is_none(),
        "a staged BG3 text DMA must consume its linear evidence at NMI completion",
    );
    assert_eq!(
        state.dialogue_scroll_phase(),
        DialogueScrollPhase::CompletedScroll,
    );
}

fn return_only_dialogue_state_before_run2508() -> ZeldaState {
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

    state.ppu.vram[0x7c00..0x7ff0].fill(0x1111);
    state.capture_display_snapshot();
    state.begin_dialogue_scroll(
        DialogueTextGeneration::PublishedDisplay,
        DialogueScrollCompletionTiming::AfterReturnBoundary,
    );
    state.finish_dialogue_scroll_remaining_pixels();
    for index in 0..crate::PresentedDialogueText::WORD_COUNT {
        state.set_messaging_render_buffer_word(index, 0x3000 | index as u16);
    }
    state.bg3_vwf_glyph_runs = vec![Bg3VwfGlyphRun {
        glyph_code: 0x41,
        origin_tile_number: 0x180,
        x: 4,
        y: 0,
        width: 3,
    }];
    state.bg3_vwf_glyph_run_dialogue_offsets = vec![0x2d];
    state.bg3_vwf_glyph_run_dialogue_message_id = 32;
    state.messaging_state_mut().set_dialogue_msg_read_pos(0);

    // Run 2508 enters in the Held handler accepted by run 2507. The outgoing
    // acceptance-host image is still receptive because that handler has not
    // yet returned.
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
    state.latch_nmi_update();
    state.set_pending_nmi_subroutine(2);
    state.set_core_update_disable_flag(2);
    state.frame_ctr_dbg = 2508;
    state
}

fn run2508_dialogue_receipts() -> OriginalTimingHostReceipts {
    OriginalTimingHostReceipts::new(
        2508,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
        ],
    )
}

fn run2509_dialogue_receipts() -> OriginalTimingHostReceipts {
    OriginalTimingHostReceipts::new(
        2509,
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
    )
    .with_dialogue_scroll_progress(vec![crate::DialogueScrollProgressReceipt {
        entered: true,
        completed_pixel_passes: 2,
        returned: false,
    }])
}

fn staged_dialogue_carry_state(gate: NmiUpdateGate) -> ZeldaState {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(14);
    state.set_submodule(2);
    state.capture_display_snapshot();
    for index in 0..crate::PresentedDialogueText::WORD_COUNT {
        state.set_messaging_render_buffer_word(index, 0x4444);
    }
    state.bg3_vwf_glyph_runs = vec![Bg3VwfGlyphRun {
        glyph_code: 0x42,
        origin_tile_number: 0x181,
        x: 7,
        y: 0,
        width: 4,
    }];
    state.bg3_vwf_glyph_run_dialogue_offsets = vec![0x31];
    state.bg3_vwf_glyph_run_dialogue_message_id = 32;
    state.messaging_state_mut().set_dialogue_msg_read_pos(0x31);
    let staged = state.dialogue_text_scanout_from_render_buffer();
    state.stage_dialogue_scroll_completion_after_return(staged);
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(gate);
    if gate == NmiUpdateGate::LatchHeld {
        state.latch_nmi_update();
    } else {
        state.clear_nmi_update_latch();
    }
    state.set_pending_nmi_subroutine(2);
    state.set_core_update_disable_flag(2);
    let mut semantic = vec![OriginalTimingSemanticReceipt::NmiHandlerCompleted];
    if gate == NmiUpdateGate::Open {
        semantic.push(OriginalTimingSemanticReceipt::JoypadPublication(
            JoypadPublication {
                high: 0,
                low: 0,
                high_filtered: 0,
                low_filtered: 0,
            },
        ));
    }
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(1, 0, semantic))
        .unwrap();
    assert_eq!(
        state.take_original_timing_nmi_phases(),
        [OriginalTimingNmiPhase::HandlerCompleted],
    );
    state.begin_original_timing_host_dispatch(0);
    state
}

// Topical test files split out of this hub (see the declarations' paths).
#[path = "zelda_rtl_tests/runtime/runtime_audio.rs"]
mod runtime_audio;
#[path = "zelda_rtl_tests/runtime/runtime_dialogue.rs"]
mod runtime_dialogue;
#[path = "zelda_rtl_tests/runtime/runtime_display.rs"]
mod runtime_display;
#[path = "zelda_rtl_tests/runtime/runtime_dungeon.rs"]
mod runtime_dungeon;
#[path = "zelda_rtl_tests/runtime/runtime_dungeon_roles.rs"]
mod runtime_dungeon_roles;
#[path = "zelda_rtl_tests/runtime/runtime_entity_tiles.rs"]
mod runtime_entity_tiles;
#[path = "zelda_rtl_tests/runtime/runtime_entrance_backup.rs"]
mod runtime_entrance_backup;
#[path = "zelda_rtl_tests/runtime/runtime_items.rs"]
mod runtime_items;
#[path = "zelda_rtl_tests/runtime/runtime_misc.rs"]
mod runtime_misc;
#[path = "zelda_rtl_tests/runtime/runtime_overworld.rs"]
mod runtime_overworld;
#[path = "zelda_rtl_tests/runtime/runtime_palette_backup.rs"]
mod runtime_palette_backup;
#[path = "zelda_rtl_tests/runtime/runtime_player_collision.rs"]
mod runtime_player_collision;
#[path = "zelda_rtl_tests/runtime/runtime_player_transitions.rs"]
mod runtime_player_transitions;
#[path = "zelda_rtl_tests/runtime/runtime_spotlight.rs"]
mod runtime_spotlight;
#[path = "zelda_rtl_tests/runtime/runtime_sprites.rs"]
mod runtime_sprites;
#[path = "zelda_rtl_tests/runtime/runtime_startup.rs"]
mod runtime_startup;
#[path = "zelda_rtl_tests/runtime/runtime_tile_behavior.rs"]
mod runtime_tile_behavior;
#[path = "zelda_rtl_tests/runtime/runtime_tile_definitions.rs"]
mod runtime_tile_definitions;
#[path = "zelda_rtl_tests/runtime/runtime_timing.rs"]
mod runtime_timing;
#[path = "zelda_rtl_tests/runtime/runtime_world_transient_owners.rs"]
mod runtime_world_transient_owners;
