//! ZeldaState runtime tests — dungeon.
//! Split mechanically from the hub file; bodies unchanged.

use super::*;
use crate::tile_definition::NativeTile;

#[test]
fn parity_probe_direct_entrance_loads_room_from_entrance_assets() {
    let mut state = ZeldaState::new();
    state.assets = Some(probe_entrance_asset_pack(0x2a, 0x0122));

    let room = state.parity_probe_direct_entrance(0x2a);

    assert_eq!(room, 0x0122);
    assert_eq!(read_le_u16(&state.ram, 0x048e), 0x0122);
    assert_eq!(state.ram[0x001b], 1);
}

#[test]
fn parity_probe_dungeon_room_marks_room_as_indoor_surface() {
    let mut state = ZeldaState::new();

    let room = state.parity_probe_dungeon_room(0x002d);

    assert_eq!(room, 0x002d);
    assert_eq!(read_le_u16(&state.ram, 0x048e), 0x002d);
    assert_eq!(read_le_u16(&state.ram, 0x00a0), 0x002d);
    assert_eq!(state.ram[0x001b], 1);
}

#[test]
fn triforce_poly_step0_falls_through_once_like_c() {
    let mut state = ZeldaState::new();
    state.attract_scene_mut().set_intro_step_index(0);
    state.poly_runtime_mut().set_config1(10);
    state.set_subsubmodule(8);
    state.poly_runtime_mut().set_angle_a(7);
    state.poly_runtime_mut().set_angle_b(11);

    state.triforce_room_handle_poly();

    assert_eq!(state.game_state.poly.runtime.config1(), 8);
    assert_eq!(state.game_state.ending.attract_scene.intro_step_index(), 0);
    assert_eq!(state.game_state.frame.subsubmodule, 8);
    assert_eq!(state.game_state.poly.runtime.angle_a(), 8);
    assert_eq!(state.game_state.poly.runtime.angle_b(), 13);
    assert_eq!(
        state.game_state.ending.attract_scene.intro_did_run_step(),
        1
    );
    assert_eq!(state.ram[0x1e02], 0);
    assert_eq!(
        state.game_state.ending.attract_scene.intro_frame_counter(),
        1
    );
}

#[test]
fn fat_stair_background_conversion_retains_its_caller_step() {
    // ROM host402680 stops at $00:DF92 inside PrepTransAuxGfx; the shared
    // Module07/$06 step3 caller has not reached its subsubmodule increment.
    for submodule in [6, 7] {
        let mut state = ZeldaState::new();
        state.set_rom_startup_timing(true);
        state.set_main_module(7);
        state.set_submodule(submodule);
        state.set_subsubmodule(3);
        state.DungeonTransition_TriggerBGC34UpdateAndAdvance();
        assert_eq!(state.game_state.frame.subsubmodule, 3);
        assert_eq!(
            state.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::FallingBgCharacters34
            })
        );
    }
}

#[test]
fn death_restart_keeps_room_module_and_counters_until_sprite_reset_returns() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(0x12);
    state.set_submodule(9);
    state.set_subsubmodule(0);
    state.set_dungeon_room(0x4a);
    state.set_indoor_flag(1);
    state.save_progress_mut().set_progress_indicator(3);
    state.save_progress_mut().set_dark_world_state(1);
    state.save_progress_mut().set_palace_index_x2(0xff);
    state
        .save_progress_mut()
        .set_total_death_save_counter(0xffff);
    state
        .player_resources_mut()
        .increment_health_capacity_by(0x18);
    let pending_deaths = state
        .game_state
        .inventory
        .save_progress
        .pending_death_save_counter();
    let sram_before = state.sram.clone();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        422610,
        0,
        vec![OriginalTimingSemanticReceipt::SpriteResetAllProgress(
            crate::SpriteResetAllProgressReceipt {
                progress: crate::SpriteResetAllProgress::SpriteDisableAllCompleted,
                boundary: crate::OriginalTimingBoundary::NmiAccepted,
            },
        )],
    ));

    state.Death_Func15(true);
    assert_eq!(
        (
            state.game_state.frame.main_module,
            state.game_state.frame.submodule
        ),
        (0x12, 9)
    );
    assert_eq!(state.game_state.world.location.dungeon_room(), 0x4a);
    assert_eq!(
        state
            .game_state
            .inventory
            .save_progress
            .pending_death_save_counter(),
        pending_deaths
    );
    assert_eq!(state.game_state.system_signals.game_over_check_flag(), 0);
    assert_eq!(state.sram, sram_before);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishGameOverDeathAfterSpriteReset {
            count_as_death: true
        })
    );
    assert!(state
        .game_execution_scheduler
        .work_suspends_translated_call_stack());
    state.latch_nmi_update();
    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        422611,
        0,
        vec![
            OriginalTimingSemanticReceipt::SpriteResetAllProgress(SpriteResetAllProgressReceipt {
                progress: SpriteResetAllProgress::SpriteDisableAllCompleted,
                boundary: OriginalTimingBoundary::NmiAccepted,
            }),
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    ));
    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);
    assert!(state.game_execution_scheduler.is_idle());
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .unwrap()
        .semantic
        .iter()
        .all(|r| !matches!(r, OriginalTimingSemanticReceipt::SpriteResetAllProgress(_))));
    assert_eq!(
        (
            state.game_state.frame.main_module,
            state.game_state.frame.submodule
        ),
        (5, 0)
    );
    assert_eq!(state.game_state.world.location.dungeon_room(), 0x20);
    assert_eq!(
        state
            .game_state
            .inventory
            .save_progress
            .pending_death_save_counter(),
        pending_deaths + 1
    );
    assert_eq!(state.game_state.system_signals.game_over_check_flag(), 1);
}

#[test]
fn fresh_dungeon_iteration_forwards_link_oam_timeline_to_the_caller_owner() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(4);
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

    // ZeldaRunGameLoop has entered Module07 and completed its state-4 body,
    // but the source host returned inside LinkOam_Main. The existing dungeon
    // caller continuation therefore owns LinkOam, both HUD calls,
    // NMI_PrepareSprites, and the final nmi_boolean clear on the next host.
    assert_eq!(state.game_state.frame.subsubmodule, 5);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn),
    );
    assert!(state.active_dungeon_sprite_main_return.is_some());
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(state.original_timing_nmi_publication_pending);
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn fresh_stair_link_oam_progress_requires_its_source_caller() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(18);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::Open, NmiUpdateGate::LatchHeld];
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        359918,
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
            OriginalTimingSemanticReceipt::LinkOamStairProgress(
                crate::LinkOamStairProgress::BodySelection,
            ),
        ],
    ));
    assert!(state
        .original_timing_interrupted_idle_main_loop_plan()
        .is_some());
    state.set_submodule(2);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
        || state.original_timing_interrupted_idle_main_loop_plan()
    ))
    .is_err());
}

#[test]
fn exhausted_room_load_estimate_waits_for_a_source_caller_return() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(1);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
    state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::Open, NmiUpdateGate::LatchHeld];
    state.capture_display_snapshot();
    state.dungeon_room_load_cpu_schedule = Some(DungeonRoomLoadCpuSchedule::default());
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::AuxiliarySpriteGraphics,
        },
        1,
    );

    // The source host publishes the handler which interrupted auxiliary
    // graphics and accepts another NMI before returning from the suspended
    // call. The native one-slice estimate expires here, but without a
    // MainLoopCommonSuffixCompleted receipt it has no authority to retire the
    // room-load caller.
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        13_232,
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

    assert!(state.original_timing_nmi_publication_pending);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::AuxiliarySpriteGraphics,
        }),
    ));
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn outdoor_y_collision_starts_falling_into_pit() {
    let mut state = ZeldaState::new();
    state.set_indoor_flag(0);
    state.tile_detect_position_mut().or_pit_tile(5);

    state.start_movement_collision_checks_y_handle_outdoors();

    assert_eq!(link_test_byte(&state, LINK_SPRITE_OAM_STATE_TIMER), 9);
    assert_eq!(state.game_state.player.follower_link.near_pit_state(), 1);
    assert_eq!(state.game_state.player.follower_link.handler_state(), 1);
}

#[test]
fn outdoor_x_deepwater_without_flippers_hops_from_safe_return() {
    let mut state = ZeldaState::new();
    state.set_indoor_flag(0);
    set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 3);
    set_link_test_byte(&mut state, LINK_LAST_DIRECTION_MOVED_TOWARDS, 3);
    state.tile_detect_position_mut().set_deepwater(4);
    set_link_test_byte(&mut state, LINK_Y_COORD_SAFE_RETURN_LO, 0x34);
    set_link_test_byte(&mut state, LINK_Y_COORD_SAFE_RETURN_HI, 0x12);
    set_link_test_byte(&mut state, LINK_X_COORD_SAFE_RETURN_LO, 0x78);
    set_link_test_byte(&mut state, LINK_X_COORD_SAFE_RETURN_HI, 0x56);

    state.start_movement_collision_checks_x_handle_outdoors();

    assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x1234);
    assert_eq!(link_test_word(&state, LINK_X_COORD), 0x5678);
    assert_eq!(link_test_byte(&state, LINK_IS_IN_DEEP_WATER), 1);
    assert_eq!(
        state.game_state.player.follower_link.swim_direction_flags(),
        3
    );
    assert_eq!(
        state.game_state.player.follower_link.actual_x_velocity(),
        16
    );
    assert_eq!(state.game_state.player.follower_link.actual_y_velocity(), 0);
    assert_eq!(
        state.game_state.player.follower_link.actual_z_velocity(),
        24
    );
    assert_eq!(
        state.game_state.player.follower_link.incapacitated_timer(),
        16
    );
    assert_eq!(state.game_state.player.follower_link.auxiliary_state(), 1);
    assert_eq!(state.game_state.player.follower_link.handler_state(), 6);
}

#[test]
fn outdoor_y_spike_damage_rebounds_and_unequips_cape() {
    let mut state = ZeldaState::new();
    state.tile_detect_position_mut().set_spike_cactus_tiles(1);
    set_link_test_byte(&mut state, LINK_LAST_DIRECTION_MOVED_TOWARDS, 0);
    set_link_test_byte(&mut state, LINK_DISABLE_SPRITE_DAMAGE, 1);
    set_link_test_byte(&mut state, LINK_ELECTROCUTE_ON_TOUCH, 1);
    set_link_test_word(&mut state, LINK_Y_COORD, 0x40);

    state.start_movement_collision_checks_y_handle_outdoors();

    assert_eq!(link_test_byte(&state, LINK_GIVE_DAMAGE), 8);
    assert_eq!(link_test_byte(&state, LINK_BUNNY_TRANSFORM_TIMER), 32);
    assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 0);
    assert_eq!(link_test_byte(&state, LINK_ELECTROCUTE_ON_TOUCH), 0);
    assert_eq!(
        state.game_state.player.follower_link.actual_y_velocity(),
        24
    );
    assert_eq!(state.game_state.player.follower_link.actual_x_velocity(), 0);
    assert_eq!(
        state.game_state.player.follower_link.actual_z_velocity(),
        36
    );
    assert_eq!(
        state.game_state.player.follower_link.incapacitated_timer(),
        24
    );
    assert_eq!(state.game_state.player.follower_link.auxiliary_state(), 1);
}

#[test]
fn outdoor_x_spike_damage_applies_tile_rebound() {
    let mut state = ZeldaState::new();
    state.tile_detect_position_mut().set_spike_cactus_tiles(1);
    set_link_test_byte(&mut state, LINK_LAST_DIRECTION_MOVED_TOWARDS, 2);
    set_link_test_word(&mut state, LINK_X_COORD, 0x40);

    state.start_movement_collision_checks_x_handle_outdoors();

    assert_eq!(link_test_byte(&state, LINK_GIVE_DAMAGE), 8);
    assert_eq!(
        state.game_state.player.follower_link.actual_x_velocity(),
        24
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
}

#[test]
fn outdoor_x_misc_bugfix_runs_slope_check_while_dashing_vertically() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_running_state(1);
    state.follower_link_state_mut().set_facing(0);
    set_link_test_byte(&mut state, LINK_X_VEL, 1);
    state.tile_detect_position_mut().set_slope_collision_bits(5);
    set_link_test_word(&mut state, LINK_X_COORD, 0x44);
    state.enhanced_features_mut().set_bits(0x1000);

    state.start_movement_collision_checks_x_handle_outdoors();

    assert_eq!(link_test_word(&state, LINK_X_COORD), 0x40);
    assert_eq!(link_test_byte(&state, LINK_MOVING_AGAINST_DIAG_TILE), 0x25);
}

#[test]
fn room_61_rescue_follower_trigger_rejects_a_different_follower() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.set_indoor_flag(1);
    state.set_dungeon_room(0x0061);
    state.follower_state_mut().set_indicator(2);
    state.follower_link_state_mut().set_x(0x039d);
    state.follower_link_state_mut().set_y(0x0cf8);

    state.follower_handle_trigger();

    assert_eq!(state.game_state.frame.main_module, 7);
    assert_eq!(state.game_state.frame.submodule, 0);
    assert_eq!(state.game_state.sprites.follower_runtime.event_flags(), 0);
    assert!(state.next_display_obj_memory_generation.is_none());
    assert!(state.next_display_obj_scanout_generation.is_none());
}

#[test]
fn cache_camera_properties_if_outdoors_snapshots_scroll_state() {
    let mut state = ZeldaState::new();
    state.set_bg2_x(0x1111);
    state.set_bg2_y(0x2222);
    set_link_test_word(&mut state, LINK_Y_COORD, 0x3333);
    set_link_test_word(&mut state, LINK_X_COORD, 0x4444);
    state.room_bounds_mut().set_y_bound(0, 0x5555);
    state.room_bounds_mut().set_x_bound(2, 0x6666);
    state.set_up_down_scroll_target(0x7777);
    state.set_left_right_scroll_target_end(0x8888);
    state.set_camera_y_coord_scroll_low(0x9999);
    state.set_quadrant_fullsize_y(2);
    set_link_test_byte(&mut state, LINK_QUADRANT_Y, 2);
    state.follower_link_state_mut().set_facing(8);
    state.follower_link_state_mut().set_lower_level_state(1);
    state.ram[IS_STANDING_IN_DOORWAY] = 2;
    state.dungeon_stair_movement_mut().set_current_floor(0xff);

    state.cache_camera_properties_if_outdoors();

    assert_eq!(
        state
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_h_copy2_cached(),
        0x1111
    );
    assert_eq!(
        state
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2_cached(),
        0x2222
    );
    assert_eq!(link_test_word(&state, LINK_Y_COORD_CACHED), 0x3333);
    assert_eq!(link_test_word(&state, LINK_X_COORD_CACHED), 0x4444);
    assert_eq!(
        state.game_state.world.transient.cached_room_bounds_y_start,
        0x5555
    );
    assert_eq!(
        state.game_state.world.transient.cached_room_bounds_x_end,
        0x6666
    );
    assert_eq!(
        read_le_u16(&state.ram, UP_DOWN_SCROLL_TARGET_CACHED),
        0x7777
    );
    assert_eq!(
        read_le_u16(&state.ram, LEFT_RIGHT_SCROLL_TARGET_END_CACHED),
        0x8888
    );
    assert_eq!(
        read_le_u16(&state.ram, CAMERA_Y_COORD_SCROLL_LOW_CACHED),
        0x9999
    );
    assert_eq!(state.ram[QUADRANT_FULLSIZE_Y_CACHED], 2);
    assert_eq!(link_test_byte(&state, LINK_QUADRANT_Y_CACHED), 2);
    assert_eq!(link_test_byte(&state, LINK_DIRECTION_FACING_CACHED), 8);
    assert_eq!(link_test_byte(&state, LINK_IS_ON_LOWER_LEVEL_CACHED), 1);
    assert_eq!(state.ram[IS_STANDING_IN_DOORWAY_CACHED], 2);
    assert_eq!(state.ram[DUNG_CUR_FLOOR_CACHED], 0xff);
}

#[test]
fn dungeon_layer_change_updates_floor_room_and_visited_flags() {
    let mut state = ZeldaState::new();
    state.set_dungeon_room(0x0104);
    state.ram[ABOUT_TO_JUMP_OFF_LEDGE] = 1;
    state.set_quadrant_fullsize_y(1);
    state.set_quadrant_fullsize_x(1);
    set_link_test_byte(&mut state, LINK_QUADRANT_Y, 1);
    set_link_test_byte(&mut state, LINK_QUADRANT_X, 1);

    state.dungeon_handle_layer_change();

    assert_eq!(state.game_state.world.location.dungeon_room(), 0x0114);
    assert_eq!(link_test_byte(&state, LINK_IS_ON_LOWER_LEVEL_MIRROR), 1);
    assert_eq!(state.game_state.player.follower_link.lower_level_state(), 1);
    assert_eq!(state.ram[ABOUT_TO_JUMP_OFF_LEDGE], 0);
    assert_ne!(read_le_u16(&state.ram, DUNG_QUADRANTS_VISITED), 0);

    state
        .dungeon_stair_movement_mut()
        .set_kind_of_in_room_staircase_word(2);
    state.follower_link_state_mut().set_lower_level_state(0);
    state.dungeon_handle_layer_change();
    assert_eq!(state.game_state.player.follower_link.lower_level_state(), 0);
}

#[test]
fn push_block_target_flag_reads_dungeon_attr_table() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_lower_level_state(1);
    state
        .dungeon_bg2_attributes_mut()
        .set_bg2_attr(0x1000 + 0x145, 0x72);

    assert_eq!(
        state.push_block_target_tile(5, 0x28),
        NativeTile::from_cartridge(0x72)
    );
}

#[test]
fn push_block_attempt_checks_both_target_tiles() {
    let mut state = ZeldaState::new();
    state
        .tile_detect_position_mut()
        .set_location_calc_mask(0x01ff);
    set_link_test_byte(&mut state, LINK_LAST_DIRECTION_MOVED_TOWARDS, 0);
    state
        .dungeon_bg2_attributes_mut()
        .set_bg2_attr(0x18 * 8 + 4, 0);
    state
        .dungeon_bg2_attributes_mut()
        .set_bg2_attr(0x18 * 8 + 5, 11);

    assert!(state.push_block_attempt_to_push_the_block(0, 0x20, 0x20));

    state
        .dungeon_bg2_attributes_mut()
        .set_bg2_attr(0x18 * 8 + 5, 9);
    assert!(!state.push_block_attempt_to_push_the_block(0, 0x20, 0x20));
}

#[test]
fn native_dungeon_map_drawing_consumes_its_measured_interruptions() {
    for room in [0x41, 0x72] {
        for nmi_slices in [0, 1, 2] {
            let mut state = ZeldaState::new();
            state.restore_live_rom_timing_after_checkpoint();
            state.set_dungeon_room_index(room);
            state.pending_dungeon_map_room_drawing_nmi_slices = Some(nmi_slices);

            state.Module0E_03_01_03_DrawRooms();

            assert_eq!(state.pending_dungeon_map_room_drawing_nmi_slices, None);
            if nmi_slices == 0 {
                let mut expected = ZeldaState::new();
                expected.set_dungeon_room_index(room);
                expected.complete_dungeon_map_room_drawing();
                assert!(state.game_execution_scheduler.is_idle());
                assert_eq!(state.ram.as_slice(), expected.ram.as_slice());
            } else {
                for _ in 1..nmi_slices {
                    assert_ne!(
                        state.game_execution_scheduler.advance_work_one_nmi_slice(),
                        Some(GameWorkStep::Complete(GameWorkContinuation::FinishDungeonMapRoomDrawing)),
                    );
                }
                assert_eq!(
                    state.game_execution_scheduler.advance_work_one_nmi_slice(),
                    Some(GameWorkStep::Complete(GameWorkContinuation::FinishDungeonMapRoomDrawing)),
                );
            }
        }
    }
}

#[test]
fn dungeon_map_room_drawing_without_a_source_return_remains_suspended_for_every_room() {
    for room in [0x41, 0x72] {
        let mut state = ZeldaState::new();
        state.restore_live_rom_timing_after_checkpoint();
        state.set_dungeon_room_index(room);

        state.Module0E_03_01_03_DrawRooms();

        assert_eq!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Complete(
                GameWorkContinuation::FinishDungeonMapRoomDrawing,
            )),
            "room ${room:02x} must not select its own timing law",
        );
    }
}

#[test]
fn dungeon_map_floor_selection_respects_every_source_lower_bound() {
    // Every scrollable dungeon's lower bound from the source floor table.
    for (palace, bottom) in [
        (0, 0u8),
        (2, 0xfe),
        (6, 0),
        (8, 1),
        (10, 0xff),
        (14, 0xff),
        (18, 0xfa),
        (20, 1),
        (22, 0xff),
        (24, 0xfe),
        (26, 0),
    ] {
        let mut state = ZeldaState::new();
        state.save_progress_mut().set_palace_index_x2(palace);
        state.set_dungeon_map_current_floor(0xa500 | u16::from(bottom));
        state.follower_link_state_mut().set_joypad1h_last(4);
        state.set_bg2_y(0x240);
        state.DungeonMap_HandleMovementInput();
        assert_eq!(
            state.game_state.dungeon_map_display.dungmap_cur_floor(),
            u16::from(bottom)
        );
        assert_eq!(
            state
                .game_state
                .dungeon_map_display
                .dungmap_floor_scroll_step(),
            0
        );
        assert_eq!(
            state.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
            0x240
        );
    }
}

#[test]
fn dungeon_map_floor_selection_uses_word_increments_across_byte_wrap() {
    for (floor, input, expected) in [(0xff, 8, 0x100), (1, 4, 0)] {
        let mut state = ZeldaState::new();
        state.save_progress_mut().set_palace_index_x2(2);
        state.set_dungeon_map_current_floor(floor);
        state.follower_link_state_mut().set_joypad1h_last(input);
        state.DungeonMap_HandleFloorSelect();
        assert_eq!(
            state.game_state.dungeon_map_display.dungmap_cur_floor(),
            expected
        );
        assert_eq!(
            state
                .game_state
                .dungeon_map_display
                .dungmap_floor_scroll_step(),
            1
        );
    }
}

#[test]
fn dungeon_map_room_drawing_consumes_same_host_iteration_return_without_room_selector() {
    for room in [0x41, 0x72] {
        let mut expected = ZeldaState::new();
        expected.set_dungeon_room_index(room);
        expected.complete_dungeon_map_room_drawing();

        let mut state = ZeldaState::new();
        state.restore_live_rom_timing_after_checkpoint();
        state.set_dungeon_room_index(room);
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![OriginalTimingSemanticReceipt::MainLoopIterationReturnedToWait],
        ));

        state.Module0E_03_01_03_DrawRooms();

        assert!(state.game_execution_scheduler.is_idle());
        assert_eq!(state.ram.as_slice(), expected.ram.as_slice());
        assert!(state
            .original_timing_semantic_receipts
            .as_ref()
            .is_some_and(|receipts| receipts.semantic().is_empty()));
    }
}

#[test]
fn dungeon_map_room_drawing_carries_the_following_source_nmi_acceptance() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(14);
    state.set_submodule(3);
    state.set_subsubmodule(0);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state
        .game_execution_scheduler
        .schedule_work(GameWorkContinuation::FinishDungeonMapRoomDrawing, 1);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        1,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                crate::MainLoopInterruption::SpritePreparation,
            ),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
        ],
    ));

    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert!(state.game_execution_scheduler.is_idle());
    assert!(state.original_timing_nmi_publication_pending);
    assert!(!state.original_timing_scheduled_nmi_accepted_at_host_return);
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn throne_room_rom_work_resumes_only_after_every_intervening_nmi_slice() {
    let mut work = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishAttractThroneRoom,
        ATTRACT_THRONE_ROOM_NMI_SLICES,
    );

    for _ in 0..ATTRACT_THRONE_ROOM_NMI_SLICES - 1 {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(GameWorkContinuation::FinishAttractThroneRoom)
    );
    assert!(work.is_complete());
}

#[test]
fn dungeon_falling_entrance_work_resumes_at_measured_cpu_boundaries() {
    let stages = [
        DungeonFallingEntranceWork::RoomAndTilesets,
        DungeonFallingEntranceWork::SpriteGraphics,
    ];

    for stage in stages {
        let continuation = GameWorkContinuation::FinishDungeonFallingEntrance { work: stage };
        let mut work = ScheduledGameWork::schedule(continuation, stage.nmi_slices());
        for _ in 0..stage.nmi_slices() - 1 {
            assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
        }
        assert_eq!(
            work.advance_one_nmi_slice(),
            GameWorkStep::Complete(continuation)
        );
    }
}

#[test]
fn live_falling_entrance_control_state_follows_source_publications_not_slice_counts() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(0x11);
    state.set_submodule(0);
    state.set_subsubmodule(2);
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonFallingEntrance {
            work: DungeonFallingEntranceWork::RoomAndTilesets,
        },
        DUNGEON_FALLING_ENTRANCE_ROOM_LOAD_NMI_SLICES,
    );

    // Elapsed NMI counts are not source statements. Advancing past both old
    // hard-coded thresholds must leave the control bytes untouched.
    for _ in 0..24 {
        assert_eq!(
            state
                .game_execution_scheduler
                .advance_work_one_nmi_slice_with_authoritative_completion(false),
            Some(GameWorkStep::Waiting),
        );
        assert_eq!(
            (
                state.game_state.frame.submodule,
                state.game_state.frame.subsubmodule
            ),
            (0, 2),
        );
    }

    let stages = [
        (
            crate::DungeonFallingEntranceProgress::RoomParserClearedSubsubmodule,
            (0, 0),
        ),
        (
            crate::DungeonFallingEntranceProgress::RoomLoadAdvancedSubsubmodule,
            (0, 3),
        ),
        (
            crate::DungeonFallingEntranceProgress::SongBankTailEntered,
            (7, 3),
        ),
    ];
    for (progress, expected) in stages {
        state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![OriginalTimingSemanticReceipt::DungeonFallingEntranceProgress(progress)],
        ));
        let observed = state
            .take_original_timing_dungeon_falling_entrance_progress()
            .expect("source stage was not consumed");
        state.apply_original_timing_dungeon_falling_entrance_progress(observed);
        assert_eq!(
            (
                state.game_state.frame.submodule,
                state.game_state.frame.subsubmodule
            ),
            expected,
        );
        assert!(state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic()
            .is_empty());
    }
}

#[test]
fn rescued_maiden_tilemap_clear_resumes_from_the_exact_source_store() {
    let mut state = ZeldaState::new();
    state.set_subsubmodule(0);

    // Source NMI $02:985e/X=$03b4 resumes at the sixth store for word 474:
    // all four BG2 quadrant stores and BG1 quadrant zero have committed.
    state.apply_rescued_maiden_tilemap_clear_stores(0, 3797);
    for quadrant in 0..4 {
        assert_eq!(
            read_le_u16(&state.ram, DUNG_BG2 + (quadrant * 0x400 + 474) * 2),
            0x01ec,
        );
    }
    assert_eq!(read_le_u16(&state.ram, DUNG_BG1 + 474 * 2), 0x01ec);
    for quadrant in 1..4 {
        assert_eq!(
            read_le_u16(&state.ram, DUNG_BG1 + (quadrant * 0x400 + 474) * 2),
            0,
        );
    }
    for base in [DUNG_BG2, DUNG_BG1] {
        for quadrant in 0..4 {
            assert_eq!(
                read_le_u16(&state.ram, base + (quadrant * 0x400 + 473) * 2),
                0x01ec,
            );
            assert_eq!(
                read_le_u16(&state.ram, base + (quadrant * 0x400 + 475) * 2),
                0,
            );
        }
    }

    state.complete_rescued_maiden_tilemap_clear(3797);
    assert_eq!(state.game_state.frame.subsubmodule, 1);
    for base in [DUNG_BG2, DUNG_BG1] {
        assert_eq!(read_le_u16(&state.ram, base), 0x01ec);
        assert_eq!(read_le_u16(&state.ram, base + (0x1000 - 1) * 2), 0x01ec,);
    }
}

#[test]
fn rescued_maiden_source_checkpoint_parks_the_real_module_caller() {
    for boundary in [
        OriginalTimingBoundary::NmiAccepted,
        OriginalTimingBoundary::HostReturn,
    ] {
        let mut state = ZeldaState::new();
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        state.set_main_module(7);
        state.set_submodule(0x18);
        state.set_subsubmodule(0);
        state.set_darkening_or_lightening_screen(0xff);
        state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
            0,
            0,
            vec![
                OriginalTimingSemanticReceipt::RescuedMaidenTilemapClearProgress(
                    crate::RescuedMaidenTilemapClearProgressReceipt {
                        completed_stores: 3797,
                        boundary,
                    },
                ),
            ],
        ));
        state.game_execution_scheduler.begin_host_frame();
        state.game_execution_scheduler.begin_main_loop_iteration();

        state.Module07_18_RescuedMaiden();

        assert_eq!(state.game_state.frame.subsubmodule, 0);
        assert_eq!(
            state.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishRescuedMaidenTilemapClear {
                completed_stores: 3797,
            }),
        );
        assert!(state
            .game_execution_scheduler
            .work_suspends_translated_call_stack());
        assert!(state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic()
            .is_empty());
    }
}

#[test]
fn rescued_maiden_initialization_preserves_exact_decompression_prefix() {
    use crate::game_state::constants::SECONDARY_DECOMP_BUFFER_LOAD_GFX;

    let asset_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("zelda3_assets.dat");
    let assets = std::fs::read(&asset_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", asset_path.display()));

    let mut state = ZeldaState::new();
    state.assets = Some(AssetPack::parse(&assets).unwrap());
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(7);
    state.set_submodule(0x18);
    state.set_subsubmodule(10);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![
            OriginalTimingSemanticReceipt::RescuedMaidenInitializationProgress(
                crate::RescuedMaidenInitializationProgressReceipt {
                    stage: crate::RescuedMaidenInitializationStage::FirstFollowerSheet {
                        completed_bytes: 1027,
                    },
                    boundary: OriginalTimingBoundary::HostReturn,
                },
            ),
        ],
    ));
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();

    state.Module07_18_RescuedMaiden();

    let sheet = state.decompressed_sprite_graphics_data(0x66).unwrap();
    assert_eq!(
        &state.ram[SECONDARY_DECOMP_BUFFER_LOAD_GFX..SECONDARY_DECOMP_BUFFER_LOAD_GFX + 1027],
        &sheet[..1027],
    );
    assert_eq!(
        state.ram[SECONDARY_DECOMP_BUFFER_LOAD_GFX + 1027],
        0,
        "the source-unwritten sheet tail must remain untouched",
    );
    assert_eq!(state.game_state.frame.submodule, 0x18);
    assert_eq!(state.game_state.frame.subsubmodule, 10);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishRescuedMaidenInitialization {
            stage: crate::RescuedMaidenInitializationStage::FirstFollowerSheet {
                completed_bytes: 1027,
            },
        }),
    );
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .unwrap()
        .semantic()
        .is_empty());
}

#[test]
fn dungeon_supertile_transition_resumes_at_rom_call_boundaries() {
    let mut state = ZeldaState::new();
    assert!(state.paired_resume_cpu_boundary_is_quiescent());
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::RoomLoad,
        },
        DUNGEON_SUPERTILE_ROOM_LOAD_NMI_SLICES,
    );
    assert!(!state.paired_resume_cpu_boundary_is_quiescent());

    let stages = [
        (
            DungeonSupertileTransitionWork::RoomLoad,
            DUNGEON_SUPERTILE_ROOM_LOAD_NMI_SLICES,
        ),
        (
            DungeonSupertileTransitionWork::AuxiliarySpriteGraphics,
            DUNGEON_SUPERTILE_AUX_SPRITE_GFX_NMI_SLICES,
        ),
        (
            DungeonSupertileTransitionWork::SpriteConversion,
            DUNGEON_SUPERTILE_SPRITE_CONVERSION_NMI_SLICES,
        ),
        (
            DungeonSupertileTransitionWork::RoomLoadCallerResume,
            DUNGEON_SUPERTILE_CALLER_RESUME_NMI_SLICES,
        ),
        (
            DungeonSupertileTransitionWork::SpriteConversionCallerResume,
            DUNGEON_SUPERTILE_CALLER_RESUME_NMI_SLICES,
        ),
        (
            DungeonSupertileTransitionWork::StraightInterroomRoomInitialization,
            DUNGEON_STRAIGHT_INTERROOM_ROOM_INITIALIZATION_NMI_SLICES,
        ),
        (
            DungeonSupertileTransitionWork::StraightInterroomBgCharacters34,
            DUNGEON_STRAIGHT_INTERROOM_BG_CHARACTERS_34_NMI_SLICES,
        ),
        (
            DungeonSupertileTransitionWork::StraightInterroomSpriteGraphics,
            DUNGEON_STRAIGHT_INTERROOM_SPRITE_GRAPHICS_NMI_SLICES,
        ),
        (
            DungeonSupertileTransitionWork::SpiralRoomCallerResume,
            DUNGEON_SUPERTILE_CALLER_RESUME_NMI_SLICES,
        ),
        (
            DungeonSupertileTransitionWork::SpiralBgCharacters34,
            DUNGEON_SPIRAL_BG_CHARACTERS_34_NMI_SLICES,
        ),
    ];

    for (stage, nmi_slices) in stages {
        assert_eq!(stage.nmi_slices(), nmi_slices);
        let continuation = GameWorkContinuation::FinishDungeonSupertileTransition { work: stage };
        let mut work = ScheduledGameWork::schedule(continuation, stage.nmi_slices());
        assert!(work.suspends_translated_call_stack());
        for _ in 0..stage.nmi_slices() - 1 {
            assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
        }
        assert_eq!(
            work.advance_one_nmi_slice(),
            GameWorkStep::Complete(continuation)
        );
        assert!(work.is_complete());
    }
    assert!(!DungeonSupertileTransitionWork::RoomLoadCallerResume
        .next_module_resumes_after_pre_main_nmi());
    assert!(DungeonSupertileTransitionWork::SpriteConversionCallerResume
        .next_module_resumes_after_pre_main_nmi());
    assert!(!DungeonSupertileTransitionWork::SpiralRoomCallerResume
        .next_module_resumes_after_pre_main_nmi());
}

#[test]
fn spiral_initializer_schedule_does_not_depend_on_room_or_staircase_identity() {
    for (room, staircase) in [(0x42, 0x34), (0x42, 0x30), (0x41, 0x34)] {
        let mut state = spiral_cpu_test_state(3, injected_dungeon_cpu_schedule(2, 0));
        state.set_dungeon_room_index(room);
        state
            .dungeon_stair_movement_mut()
            .set_staircase_index(staircase);
        assert!(state.begin_dungeon_supertile_transition_work(
            DungeonSupertileTransitionWork::SpiralRoomInitialization,
        ));
        assert_eq!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Waiting),
        );
    }
}

#[test]
fn straight_interroom_room_initialization_crosses_nineteen_nmi_slices() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(0x12);
    state.set_subsubmodule(2);
    state.set_dungeon_room_index(0x51);

    assert!(state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::StraightInterroomRoomInitialization,
    ));
    assert!(matches!(
        state.active_display_obj_generation,
        DisplayObjGeneration::RetainCapturedOam { .. }
    ));
    for _ in 0..DUNGEON_STRAIGHT_INTERROOM_ROOM_INITIALIZATION_NMI_SLICES - 1 {
        assert_eq!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Waiting),
        );
    }
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::StraightInterroomRoomInitialization,
            },
        )),
    );
}

#[test]
fn straight_interroom_bg_character_conversion_crosses_four_nmi_slices() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(0x12);
    state.set_subsubmodule(3);
    state.set_dungeon_room_index(0x51);

    assert!(state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::StraightInterroomBgCharacters34,
    ));
    for _ in 0..DUNGEON_STRAIGHT_INTERROOM_BG_CHARACTERS_34_NMI_SLICES - 1 {
        assert_eq!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Waiting),
        );
    }
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::StraightInterroomBgCharacters34,
            },
        )),
    );
}

#[test]
fn straight_interroom_sprite_graphics_cross_four_nmi_slices() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(0x12);
    state.set_subsubmodule(9);
    state.set_dungeon_room_index(0x51);

    assert!(state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::StraightInterroomSpriteGraphics,
    ));
    for _ in 0..DUNGEON_STRAIGHT_INTERROOM_SPRITE_GRAPHICS_NMI_SLICES - 1 {
        assert_eq!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Waiting),
        );
    }
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::StraightInterroomSpriteGraphics,
            },
        )),
    );
}

#[test]
fn straight_bg_conversion_return_leaves_uploads_for_the_next_open_nmi() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_animated_tile_data_source_address(0xa680);
    state.set_indoor_flag(1);
    state.set_main_module(7);
    state.set_submodule(0x12);
    state.set_subsubmodule(3);
    state.set_frame_counter(0xa5);
    state.set_pending_nmi_subroutine(9);
    state.set_core_update_disable_flag(9);
    state.latch_nmi_update();
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::StraightInterroomBgCharacters34,
        }, 1,
    );
    state.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(state.game_state.frame.subsubmodule, 4);
    assert_eq!(state.game_state.frame.frame_counter, 0xa5);
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert_eq!(state.ram[crate::game_state::constants::NMI_SUBROUTINE_INDEX], 9);
    assert_eq!(state.ram[crate::game_state::constants::NMI_DISABLE_CORE_UPDATES], 9);
    state.game_execution_scheduler.begin_host_frame();
    assert!(state.game_execution_scheduler.main_return_requires_leading_nmi());
}

#[test]
fn native_straight_reset_preserves_the_measured_prefix_until_resume() {
    for room in [0x51, 0x22] {
        for progress in [
            DungeonResetSpritesCpuProgress::Disable(
                DungeonSpriteDisableCpuProgress::AncillaPickupFlagCleared,
            ),
            DungeonResetSpritesCpuProgress::Disable(
                DungeonSpriteDisableCpuProgress::SpriteLimitInstanceCleared,
            ),
            DungeonResetSpritesCpuProgress::GarnishTypesThrough { slot: 10 },
            DungeonResetSpritesCpuProgress::SpritesDisabled,
            DungeonResetSpritesCpuProgress::CollisionXSizeSet,
            DungeonResetSpritesCpuProgress::RoomHistorySearchStarted,
        ] {
            let mut state = ZeldaState::new();
            state.restore_live_rom_timing_after_checkpoint();
            state.set_main_module(7);
            state.set_submodule(0x12);
            state.set_subsubmodule(5);
            state.set_dungeon_room_index(room);
            state.set_indoor_flag(1);
            state.sprite_slot_view_mut(0).set_state(9);
            state.sprite_slot_view_mut(0).set_sprite_type(0x6e);
            state.sprite_system_mut().set_limit_instance(6);
            state.sprite_battle_mut().set_item_drop_counter(5);
            state.garnish_state_mut().set_sprcoll_x_size(0x1234);
            state.garnish_state_mut().set_sprcoll_y_size(0x5678);
            for k in 0..30 {
                state.garnish_slot_view_mut(k).set_garnish_type(7);
            }
            let mut atomic = state.clone();
            atomic.complete_straight_interroom_sprite_reset();
            state.dungeon_submodule_cpu_schedule = Some(DungeonSubmoduleCpuSchedule {
                reset_progress: Some(progress),
                ..Default::default()
            });

            assert!(state.suspend_straight_interroom_sprite_reset_before_room_load());
            assert!(state.dungeon_submodule_cpu_schedule.is_none());
            assert_eq!(state.sprite_slot_view(0).state(), 0);
            assert_eq!(state.sprite_slot_view(0).sprite_type(), 0x6e);
            if let DungeonResetSpritesCpuProgress::Disable(disable) = progress {
                assert_eq!(state.ram[0x0b9b], 5, "later reset counters are still pending");
                assert_eq!(state.ram[0x0b6a], if disable == DungeonSpriteDisableCpuProgress::AncillaPickupFlagCleared { 6 } else { 0 });
                assert!((0..30).all(|k| state.garnish_slot_view(k).garnish_type() == 7));
            }
            if let DungeonResetSpritesCpuProgress::GarnishTypesThrough { slot } = progress {
                for k in 0..30 {
                    assert_eq!(state.garnish_slot_view(k).garnish_type(), if k < usize::from(slot) { 7 } else { 0 });
                }
            }
            assert_eq!(
                state.game_execution_scheduler.advance_work_one_nmi_slice(),
                Some(GameWorkStep::Complete(GameWorkContinuation::FinishStraightInterroomSpriteReset { progress })),
            );
            state.dungeon_resume_reset_sprites_after_cpu_progress(progress);
            assert_eq!(state.ram, atomic.ram);
            assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
        }
    }
}

#[test]
fn straight_interroom_sprite_reset_uses_the_live_semantic_progress_token() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![dungeon_reset_progress_receipt(
            DungeonResetSpritesCpuProgress::SpritesDisabled,
            OriginalTimingBoundary::HostReturn,
        )],
    ));
    state.set_main_module(7);
    state.set_submodule(0x12);
    state.set_subsubmodule(5);
    state.set_dungeon_room_index(0x51);
    state.set_indoor_flag(1);
    state.sprite_slot_view_mut(0).set_state(9);
    state.sprite_slot_view_mut(0).set_sprite_type(0x6e);
    let mut atomic = state.clone();
    atomic.complete_straight_interroom_sprite_reset();

    assert!(state.suspend_straight_interroom_sprite_reset_before_room_load());
    assert_eq!(state.sprite_slot_view(0).state(), 0);
    assert_eq!(state.sprite_slot_view(0).sprite_type(), 0x6e);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishStraightInterroomSpriteReset {
            progress: DungeonResetSpritesCpuProgress::SpritesDisabled,
        }),
    );
    let Some(GameWorkStep::Complete(GameWorkContinuation::FinishStraightInterroomSpriteReset {
        progress,
    })) = state.game_execution_scheduler.advance_work_one_nmi_slice()
    else {
        panic!("straight-interroom reset continuation lost its semantic token");
    };
    state.dungeon_resume_reset_sprites_after_cpu_progress(progress);
    assert_eq!(state.ram, atomic.ram);
    assert_eq!(state.game_state.sprites, atomic.game_state.sprites);

    let mut without_receipt = ZeldaState::new();
    without_receipt.restore_live_rom_timing_after_checkpoint();
    without_receipt.original_timing_owner = OriginalTimingOwnerState::Live;
    assert!(!without_receipt.suspend_straight_interroom_sprite_reset_before_room_load());
}

#[test]
fn straight_interroom_reset_refines_its_semantic_prefix_at_the_accepting_nmi() {
    let pickup = DungeonResetSpritesCpuProgress::Disable(
        DungeonSpriteDisableCpuProgress::AncillaPickupFlagCleared,
    );
    let limit = DungeonResetSpritesCpuProgress::Disable(
        DungeonSpriteDisableCpuProgress::SpriteLimitInstanceCleared,
    );
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![dungeon_reset_progress_receipt(
            pickup,
            OriginalTimingBoundary::HostReturn,
        )],
    ));
    state.set_main_module(7);
    state.set_submodule(0x12);
    state.set_subsubmodule(5);
    state.set_dungeon_room_index(0x51);
    state.set_indoor_flag(1);
    state.sprite_slot_view_mut(0).set_state(9);
    state.sprite_slot_view_mut(0).set_sprite_type(0x6e);
    state.ram[ANCILLA_TYPE..ANCILLA_TYPE + 10].fill(3);
    state.ram[FLAG_IS_ANCILLA_TO_PICK_UP] = 7;
    state.ram[SPRITE_LIMIT_INSTANCE] = 9;
    state.sync_native_game_state_from_ram();
    let mut atomic = state.clone();
    atomic.complete_straight_interroom_sprite_reset();

    assert!(state.suspend_straight_interroom_sprite_reset_before_room_load());
    assert!(state.ram[ANCILLA_TYPE..ANCILLA_TYPE + 10]
        .iter()
        .all(|&value| value == 0));
    assert_eq!(state.ram[FLAG_IS_ANCILLA_TO_PICK_UP], 0);
    assert_eq!(state.ram[SPRITE_LIMIT_INSTANCE], 9);

    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        1,
        0,
        vec![dungeon_reset_progress_receipt(
            limit,
            OriginalTimingBoundary::NmiAccepted,
        )],
    ));
    let refined = state.refine_dungeon_reset_progress_before_resume(pickup);
    assert_eq!(refined, limit);
    state.dungeon_resume_reset_sprites_after_cpu_progress(refined);

    assert_eq!(state.ram, atomic.ram);
    assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
}

#[test]
fn pre_dungeon_garnish_clear_publishes_only_completed_descending_stores() {
    let mut state = ZeldaState::new();
    for k in 0..30 {
        state.garnish_slot_view_mut(k).set_garnish_type(7);
    }
    let mut atomic = state.clone();
    atomic.sprite_disable_all();
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishPreDungeonEntranceLoad {
            sprite_reset: PreDungeonSpriteResetContinuation::SpriteDisableAllThrough(0),
        },
        4,
    );
    state.apply_pre_dungeon_garnish_disable_prefix(30);
    assert!(state.ram[0x1f800..0x1f81e].iter().all(|&v| v == 7));
    state.apply_pre_dungeon_garnish_disable_prefix(15);
    assert!(state.ram[0x1f800..0x1f80f].iter().all(|&v| v == 7));
    assert!(state.ram[0x1f80f..0x1f81e].iter().all(|&v| v == 0));
    state.apply_pre_dungeon_sprite_reset_progress(SpriteResetAllProgressReceipt {
        progress: SpriteResetAllProgress::SpriteDisableAllCompleted,
        boundary: OriginalTimingBoundary::HostReturn,
    });
    assert_eq!(state.ram, atomic.ram);
    assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishPreDungeonEntranceLoad {
            sprite_reset: PreDungeonSpriteResetContinuation::SpriteDisableAllCompleted,
        })
    );
}

#[test]
fn trinexx_final_phase_draw_checkpoints_resume_to_the_atomic_draw() {
    for ai_state in [0u8, 2] {
        for segment in 0..=6u8 {
            for stage in (0..8u8).chain(16..30u8) {
                if segment == 6 && stage != 0 {
                    continue;
                }
                if stage >= 16 && (segment != 4 || ai_state == 0) {
                    continue;
                }
                let mut state = ZeldaState::new();
                state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
                state.overlord_slot_view_mut(0).increment_x_high();
                state.sprite_slot_view_mut(0).set_sprite_type(0xcb);
                state.sprite_slot_view_mut(0).set_state(9);
                state.sprite_slot_view_mut(0).set_ai_state(ai_state);
                state.sprite_slot_view_mut(0).set_anim_clock(6);
                state.sprite_slot_view_mut(0).set_a(5);
                state.sprite_slot_view_mut(0).set_x_velocity(31);
                state.sprite_slot_view_mut(0).set_x_low(0x23);
                state.sprite_slot_view_mut(0).set_x_high(8);
                state.sprite_slot_view_mut(0).set_y_low(0xb4);
                state.sprite_slot_view_mut(0).set_y_high(0x15);
                state.sprite_slot_view_mut(0).set_subtype2(0x40);
                for j in 0..0x80 {
                    state
                        .moldorm_history_mut(j)
                        .set_position(0x800 + j as u16 * 3, 0x1500 + j as u16 * 5);
                }
                let mut atomic = state.clone();
                atomic.sprite_trinexx_final_phase(0);
                let continuation =
                    state.begin_trinexx_final_phase_draw_checkpoint(0, segment, stage);
                state.resume_trinexx_final_phase_draw(0, segment, stage, continuation);
                assert_eq!(
                    state.ram, atomic.ram,
                    "ai {ai_state} segment {segment} stage {stage}"
                );
                assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
                assert_eq!(state.game_state.oam, atomic.game_state.oam);
            }
        }
    }
}

#[test]
fn agahnim_motion_blur_spawn_checkpoints_resume_to_the_atomic_body() {
    use crate::SpriteDynamicSpawnProgress as P;
    for progress in [
        P::TypePublished,
        P::StatePublished,
        P::ResetProperties {
            completed_stores: 3,
        },
        P::LoadProperties {
            completed_stores: 2,
        },
        P::IdentityPublished,
        P::FloorPublished,
        P::DirectionPublished,
        P::DieActionCleared,
        P::SubtypeCleared,
    ] {
        let mut state = ZeldaState::new();
        state.game_state.world.location.set_indoor_flag(1);
        state.game_state.frame.frame_counter = 248;
        state.sprite_slot_view_mut(1).set_sprite_type(0x7a);
        state.sprite_slot_view_mut(1).set_state(9);
        state.sprite_slot_view_mut(1).set_ai_state(7);
        state.sprite_slot_view_mut(1).set_anim_clock(1);
        state.sprite_slot_view_mut(1).set_graphics(2);
        state.sprite_slot_view_mut(1).set_x_low(0xb6);
        state.sprite_slot_view_mut(1).set_x_high(0x1a);
        state.sprite_slot_view_mut(1).set_y_low(0xaa);
        state.sprite_slot_view_mut(1).set_y_high(0x01);
        // Every other slot but 3 is busy so both runs spawn into slot 3.
        for slot in (0..16).filter(|slot| *slot != 1 && *slot != 3) {
            state.sprite_slot_view_mut(slot).set_state(9);
        }
        let mut atomic = state.clone();
        let j = atomic.sprite_agahnim_apply_motion_blur(1);
        assert_eq!(j, 3);
        atomic.sprite_slot_view_mut(3).set_oam_flags(4);
        state.sprite_main_cpu_boundary = Some(SpriteMainCpuBoundary::AgahnimMotionBlurSpawn {
            slot: 1,
            spawned_slot: 3,
            progress,
            bound: false,
        });
        assert_eq!(state.sprite_agahnim_apply_motion_blur(1), -1);
        assert!(matches!(
            state.sprite_main_cpu_boundary.take(),
            Some(SpriteMainCpuBoundary::AgahnimMotionBlurSpawn { bound: true, .. })
        ));
        state.resume_agahnim_motion_blur_spawn(1, 3, progress);
        assert_eq!(state.ram, atomic.ram, "{progress:?}");
        assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
    }
}

#[test]
fn trinexx_final_phase_tile_collision_checkpoints_resume_to_the_atomic_phase() {
    for (probes_completed, x_velocity, y_velocity) in [
        (false, 31u8, 0u8),
        (true, 31, 0),
        (true, 0, 0xe1),
        (true, 0, 31),
    ] {
        let mut state = ZeldaState::new();
        state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
        state.overlord_slot_view_mut(0).increment_x_high();
        state.sprite_slot_view_mut(0).set_sprite_type(0xcb);
        state.sprite_slot_view_mut(0).set_state(9);
        state.sprite_slot_view_mut(0).set_a(5);
        state.sprite_slot_view_mut(0).set_x_velocity(x_velocity);
        state.sprite_slot_view_mut(0).set_y_velocity(y_velocity);
        state.sprite_slot_view_mut(0).set_x_low(0x23);
        state.sprite_slot_view_mut(0).set_x_high(8);
        state.sprite_slot_view_mut(0).set_y_low(0xb4);
        state.sprite_slot_view_mut(0).set_y_high(0x15);
        state.sprite_slot_view_mut(0).set_subtype2(0x40);
        let mut atomic = state.clone();
        atomic.sprite_trinexx_final_phase(0);
        state.begin_trinexx_final_phase_tile_collision_checkpoint(0, probes_completed);
        assert_eq!(state.sprite_slot_view(0).a(), 4);
        state.resume_trinexx_final_phase_tile_collision(0, probes_completed);
        assert_eq!(
            state.ram, atomic.ram,
            "probes {probes_completed} {x_velocity} {y_velocity}"
        );
        assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
    }
}

#[test]
fn trinexx_death_explosion_spawn_checkpoints_resume_to_the_atomic_explosion() {
    use crate::SpriteDynamicSpawnProgress as P;
    for progress in [
        P::TypePublished,
        P::StatePublished,
        P::ResetProperties {
            completed_stores: 4,
        },
        P::LoadProperties {
            completed_stores: 0,
        },
        P::LoadProperties {
            completed_stores: 3,
        },
        P::LoadProperties {
            completed_stores: 10,
        },
        P::IdentityPublished,
        P::FloorPublished,
        P::DirectionPublished,
        P::DieActionCleared,
        P::SubtypeCleared,
    ] {
        let mut state = ZeldaState::new();
        state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
        state.sprite_slot_view_mut(0).set_sprite_type(0xcb);
        state.sprite_slot_view_mut(0).set_state(9);
        state.sprite_slot_view_mut(0).set_ai_state(0xff);
        state.sprite_slot_view_mut(0).set_delay_main(0x40);
        state.sprite_slot_view_mut(0).set_x_low(0x23);
        state.sprite_slot_view_mut(0).set_x_high(8);
        state.sprite_slot_view_mut(0).set_y_low(0xb4);
        state.sprite_slot_view_mut(0).set_y_high(0x15);
        for slot in 14..16 {
            state.sprite_slot_view_mut(slot).set_state(4);
        }
        let mut atomic = state.clone();
        atomic.sprite_cb_trinexx_rock_head(0);
        assert_eq!(
            atomic.sprite_slot_view(13).state(),
            4,
            "explosion spawned in slot 13"
        );
        state.begin_trinexx_death_explosion_spawn_checkpoint(0, 13, progress);
        assert_eq!(state.sprite_slot_view(13).sprite_type(), 0);
        state.resume_trinexx_death_explosion_spawn(0, 13, progress);
        assert_eq!(state.ram, atomic.ram, "{progress:?}");
        assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
    }
}

#[test]
fn trinexx_breath_tile_collision_checkpoint_resumes_to_the_atomic_breath() {
    for sprite_type in [0xccu8, 0xcd] {
        let mut state = ZeldaState::new();
        state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
        state.sprite_slot_view_mut(12).set_sprite_type(sprite_type);
        state.sprite_slot_view_mut(12).set_state(9);
        state.sprite_slot_view_mut(12).set_e(1);
        state.sprite_slot_view_mut(12).set_subtype2(0x32);
        state.sprite_slot_view_mut(12).set_x_velocity(0x10);
        state.sprite_slot_view_mut(12).set_x_low(0x45);
        state.sprite_slot_view_mut(12).set_x_high(8);
        state.sprite_slot_view_mut(12).set_y_low(0xa9);
        state.sprite_slot_view_mut(12).set_y_high(0x15);
        let mut atomic = state.clone();
        if sprite_type == 0xcc {
            atomic.sprite_cc(12);
        } else {
            atomic.sprite_cd(12);
        }
        assert!(state.trinexx_breath_until_tile_collision(12));
        assert_eq!(state.sprite_slot_view(12).subtype2(), 0x33);
        assert_eq!(state.sprite_slot_view(12).state(), 9);
        state.trinexx_breath_after_tile_collision(12);
        assert_eq!(state.ram, atomic.ram, "breath type {sprite_type:#x}");
        assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
        assert_eq!(state.game_state.oam, atomic.game_state.oam);
    }
}

#[test]
fn trinexx_head_draw_checkpoint_keeps_coordinates_and_remaining_oam_stores() {
    for segment in 1..9 {
        let mut state = ZeldaState::new();
        state.set_submodule(1);
        state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
        state.sprite_slot_view_mut(0).set_a(100);
        state.sprite_slot_view_mut(0).set_c(100);
        state.sprite_slot_view_mut(2).set_sprite_type(0xcd);
        state.sprite_slot_view_mut(2).set_state(9);
        state.sprite_slot_view_mut(2).set_subtype2(9);
        for j in 18..27 {
            state.cached_sprite_slot_mut(j).set_type_byte(40 + j as u8);
            state.cached_sprite_slot_mut(j).set_y_high(30);
        }
        let mut atomic = state.clone();
        atomic.sprite_sidenexx(2);
        let draw = state.begin_sidenexx_head_draw_checkpoint(2, segment);
        assert_eq!(state.sprite_get_x(2), atomic.sprite_get_x(2));
        assert_eq!(state.sprite_get_y(2), atomic.sprite_get_y(2));
        assert_eq!(draw.next_segment, segment);
        assert_eq!(draw.oam, OAM_BUF + (usize::from(segment) + 4) * 4);
        state.resume_sidenexx_head_draw(2, draw);
        assert_eq!(state.ram, atomic.ram, "segment {segment}");
        assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
        assert_eq!(state.game_state.oam, atomic.game_state.oam);
    }
}

#[test]
fn trinexx_first_part_keeps_unwritten_oam_and_position_bytes_pending() {
    for completed in 0..=30 {
        let mut state = ZeldaState::new();
        state.set_submodule(1);
        state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
        state.sprite_slot_view_mut(0).set_a(100);
        state.sprite_slot_view_mut(0).set_c(100);
        state.sprite_slot_view_mut(2).set_sprite_type(0xcd);
        state.sprite_slot_view_mut(2).set_state(9);
        state.sprite_slot_view_mut(2).set_subtype2(9);
        for j in 18..27 {
            state.cached_sprite_slot_mut(j).set_type_byte(40 + j as u8);
            state.cached_sprite_slot_mut(j).set_y_high(30);
        }
        let mut atomic = state.clone();
        atomic.sprite_sidenexx(2);
        let draw = state.begin_sidenexx_front_part_checkpoint(2, completed);
        if completed == 25 {
            assert_eq!(
                state.ram[0xfb6], 0,
                "scratch advances only after the OAM entries"
            );
        }
        if completed == 26 {
            assert_eq!(state.ram[0xfb6], 20);
        }
        if completed <= 26 {
            assert_eq!(
                state.sprite_slot_view(2).x_low(),
                state.sprite_slot_view(2).a()
            );
            assert_eq!(
                state.sprite_slot_view(2).y_low(),
                state.sprite_slot_view(2).c()
            );
        }
        if completed == 19 {
            assert_eq!(state.game_state.oam.extended_byte(3), 0);
            assert_eq!(state.ram[OAM_BUF + 15], atomic.ram[OAM_BUF + 15]);
        }
        state.resume_sidenexx_head_draw(2, draw);
        assert_eq!(state.ram, atomic.ram, "first-part store {completed}");
        assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
        assert_eq!(state.game_state.oam, atomic.game_state.oam);
    }
}

#[test]
fn every_dungeon_reset_caller_prefix_resumes_to_the_atomic_c_endpoint() {
    for progress in [
        DungeonResetSpritesCpuProgress::LoadStarted,
        DungeonResetSpritesCpuProgress::LoadBeforeOrigin,
        DungeonResetSpritesCpuProgress::SpritesDisabled,
        DungeonResetSpritesCpuProgress::CollisionXSizeSet,
        DungeonResetSpritesCpuProgress::RoomHistorySearchStarted,
    ] {
        let mut state = ZeldaState::new();
        state.set_dungeon_room_index(0x51);
        state.set_indoor_flag(1);
        for slot in 0..16 {
            state.sprite_slot_view_mut(slot).set_state(9);
            state
                .sprite_slot_view_mut(slot)
                .set_sprite_type(0x60 + slot as u8);
        }
        state.garnish_state_mut().set_sprcoll_x_size(0x1234);
        state.garnish_state_mut().set_sprcoll_y_size(0x5678);
        let mut atomic = state.clone();
        atomic.dungeon_reset_sprites();

        state.dungeon_reset_sprites_through_cpu_progress(progress);
        if matches!(
            progress,
            DungeonResetSpritesCpuProgress::LoadStarted
                | DungeonResetSpritesCpuProgress::LoadBeforeOrigin
        ) {
            let prefix_ram = state.ram.clone();
            assert!(state.dungeon_advance_reset_sprites_cpu_progress(progress, progress));
            assert_eq!(
                state.ram, prefix_ram,
                "an unchanged source checkpoint must not replay writes"
            );
        }
        match progress {
            DungeonResetSpritesCpuProgress::SpritesDisabled => {
                assert_eq!(
                    state.game_state.sprites.garnish_runtime.sprcoll_x_size(),
                    0x1234,
                );
                assert_eq!(
                    state.game_state.sprites.garnish_runtime.sprcoll_y_size(),
                    0x5678,
                );
            }
            DungeonResetSpritesCpuProgress::CollisionXSizeSet => {
                assert_eq!(
                    state.game_state.sprites.garnish_runtime.sprcoll_x_size(),
                    0xffff,
                );
                assert_eq!(
                    state.game_state.sprites.garnish_runtime.sprcoll_y_size(),
                    0x5678,
                );
            }
            DungeonResetSpritesCpuProgress::RoomHistorySearchStarted
            | DungeonResetSpritesCpuProgress::LoadBeforeOrigin
            | DungeonResetSpritesCpuProgress::LoadStarted => {
                assert_eq!(
                    state.game_state.sprites.garnish_runtime.sprcoll_x_size(),
                    0xffff,
                );
                assert_eq!(
                    state.game_state.sprites.garnish_runtime.sprcoll_y_size(),
                    0xffff,
                );
            }
            _ => unreachable!(),
        }
        state.dungeon_resume_reset_sprites_after_cpu_progress(progress);

        assert_eq!(state.ram, atomic.ram, "RAM mismatch after {progress:?}");
        assert_eq!(
            state.game_state.sprites, atomic.game_state.sprites,
            "native sprite mismatch after {progress:?}",
        );
    }
}

#[test]
fn snes9x_room_load_dispatcher_checkpoint_has_processor_status_30() {
    // Both cold room-$50 and room-$72 Snes9x receipts enter the shared
    // Module07 state-1 dispatcher at $02:8a26 with P=$30. Keep this executable
    // checkpoint literal exact; the raster phase is modeled independently.
    let checkpoint = DUNGEON_ROOM_LOAD_CPU_CHECKPOINT;
    let packed_status = u8::from(checkpoint.carry)
        | (u8::from(checkpoint.zero) << 1)
        | (u8::from(checkpoint.interrupt_disable) << 2)
        | (u8::from(checkpoint.decimal) << 3)
        | (u8::from(checkpoint.index_is_8_bit) << 4)
        | (u8::from(checkpoint.accumulator_is_8_bit) << 5)
        | (u8::from(checkpoint.overflow) << 6)
        | (u8::from(checkpoint.negative) << 7);

    assert_eq!(checkpoint.entry_pc, 0x02_8a26);
    assert_eq!(packed_status, 0x30);
    assert!(!checkpoint.emulation);
    assert!(!checkpoint.waiting);
}

#[test]
fn c_dungeon_cache_trans_sprites_resumes_after_slot15_state_clear() {
    // The ROM step at $09:c17f commits the first loop statement from
    // sprite.c Dungeon_CacheTransSprites: alt_sprite_state[15] = 0. The
    // cache flag was written just before it, but no type/position/dynamic
    // field and no lower slot has been touched yet.
    let mut state = ZeldaState::new();
    state.set_indoor_flag(1);
    for slot in 0..16 {
        let mut live = state.sprite_slot_view_mut(slot);
        live.set_state(9);
        live.set_sprite_type(0x20 + slot as u8);
        live.set_x_low(0x30 + slot as u8);
        live.set_x_high(0x40 + slot as u8);
        live.set_y_low(0x50 + slot as u8);
        live.set_y_high(0x60 + slot as u8);
        live.set_graphics(0x70 + slot as u8);
        live.set_a(0x80 + slot as u8);
    }
    for field in CachedSpriteCacheField::C_SOURCE_ORDER {
        for slot in 0..16 {
            state.ram[field.alt_address() + slot] = 0xa5;
        }
    }
    state.sync_native_game_state_from_ram();
    let mut atomic = state.clone();
    let progress = sprite::DungeonResetSpritesCpuProgress::Cache {
        slot: 15,
        field: CachedSpriteCacheField::StateClear,
    };

    state.dungeon_reset_sprites_through_cpu_progress(progress);

    assert_eq!(
        state.ram[CachedSpriteCacheField::StateClear.alt_address() + 15],
        0,
    );
    assert_eq!(
        state.ram[CachedSpriteCacheField::Type.alt_address() + 15],
        0xa5,
    );
    assert_eq!(
        state.ram[CachedSpriteCacheField::StateClear.alt_address() + 14],
        0xa5,
    );
    assert_eq!(state.sprite_slot_view(15).state(), 9);

    state.dungeon_resume_reset_sprites_after_cpu_progress(progress);
    atomic.dungeon_reset_sprites();
    assert_eq!(state.ram, atomic.ram);
    assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
}

#[test]
fn c_dungeon_cache_trans_sprites_resumes_after_room50_slot3_flags2() {
    // Cold Snes9x enters Module07 state 1 for route run 12416 at V=259,H=22.
    // Its next reset-prefix NMI resumes at $09:c1de after the $09:c1db store
    // has copied slot 3's flags2. This is the assignment immediately before
    // floor in sprite.c Dungeon_CacheTransSprites (lines 3578..3579), not an
    // envelope endpoint selected by the translated scheduler.
    let progress = sprite::DungeonResetSpritesCpuProgress::Cache {
        slot: 3,
        field: CachedSpriteCacheField::Flags2,
    };
    let mut state = ZeldaState::new();
    state.set_indoor_flag(1);
    for slot in 0..16 {
        let mut live = state.sprite_slot_view_mut(slot);
        live.set_state(9);
        live.set_sprite_type(0x20 + slot as u8);
        live.set_x_low(0x30 + slot as u8);
        live.set_x_high(0x40 + slot as u8);
        live.set_y_low(0x50 + slot as u8);
        live.set_y_high(0x60 + slot as u8);
        live.set_graphics(0x70 + slot as u8);
        live.set_flags2(0x10 + slot as u8);
        live.set_ignore_projectile(0x80 + slot as u8);
    }
    for field in CachedSpriteCacheField::C_SOURCE_ORDER {
        for slot in 0..16 {
            state.ram[field.alt_address() + slot] = 0xa5;
        }
    }
    state.sync_native_game_state_from_ram();
    let mut atomic = state.clone();

    state.dungeon_reset_sprites_through_cpu_progress(progress);

    for slot in 4..16 {
        assert_eq!(
            state.ram[CachedSpriteCacheField::IgnoreProjectile.alt_address() + slot],
            0x80 + slot as u8,
        );
    }
    assert_eq!(
        state.ram[CachedSpriteCacheField::Flags2.alt_address() + 3],
        0x13,
    );
    assert_eq!(
        state.ram[CachedSpriteCacheField::Floor.alt_address() + 3],
        0xa5,
        "the field after the ROM checkpoint has not committed",
    );
    assert_eq!(
        state.ram[CachedSpriteCacheField::StateClear.alt_address() + 2],
        0xa5,
        "the next descending cache slot has not started at the ROM boundary",
    );

    state.dungeon_resume_reset_sprites_after_cpu_progress(progress);
    atomic.dungeon_reset_sprites();
    assert_eq!(state.ram, atomic.ram);
    assert_eq!(state.game_state.sprites, atomic.game_state.sprites);
}

#[test]
fn c_dungeon_reset_sprites_resumes_after_the_rom_observed_y_checkpoint() {
    // sprite.c Dungeon_ResetSprites -> Dungeon_LoadSprites ->
    // Dungeon_LoadSingleSprite writes state, floor, Y, X, type, subtype, N,
    // and die_action in that order. The observed boundary follows both Y
    // writes of record 1, after record 0 completed.
    let mut data = Vec::new();
    let mut ranges = vec![(0, 0); 60];
    put_test_asset(
        &mut data,
        &mut ranges,
        58,
        vec![
            0x02, 0x66, 0x21, 0x41, 0xfe, 0x00, 0xe4, 0x79, 0x31, 0x42, 0xff,
        ],
    );
    put_test_asset(&mut data, &mut ranges, 59, vec![0, 0]);

    let mut state = ZeldaState::new();
    state.assets = Some(AssetPack::from_data_ranges(data, ranges));
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(1);
    state.dungeon_room_tracking_mut().set_room_index2_word(0);
    for slot in [0, 1, 2, 9, 10] {
        state.sprite_slot_view_mut(slot).set_state(9);
    }
    state.sprite_set_x(1, 0x7777);
    state.sprite_slot_view_mut(1).set_sprite_type(0xaa);
    state.sprite_slot_view_mut(1).set_subtype(0xbb);
    state.sprite_slot_view_mut(1).set_n(0xcc);
    state.sprite_slot_view_mut(1).set_die_action(0xdd);
    state.set_frame_counter(0xf8);
    state.set_pending_nmi_subroutine(24);
    state.latch_nmi_update();
    state.ram[OAM_BUF..OAM_BUF + 0x220].fill(0xa5);
    state.sync_native_game_state_from_ram();
    state.ppu.oam.fill(0x4567);
    state.ppu.vram[0x3ed0..0x3ef0].fill(0x1234);
    state.ppu.vram[0x4000..0x4400].fill(0x2345);
    state.oam_law_entry_frame_counter = Some(0xf8);
    state.capture_display_snapshot();
    state.resident_oam_dma = Some(state.ppu.oam.clone());
    state.oam_law_pending = Some(vec![0x3456; state.ppu.oam.len()]);
    state.oam_law_visible = Some(vec![0x5678; state.ppu.oam.len()]);
    let shadow_oam = state.ram[OAM_BUF..OAM_BUF + 0x220].to_vec();
    let resident_oam = state.ppu.oam.clone();
    let resident_oam_dma = state.resident_oam_dma.clone();
    let pending_oam_law = state.oam_law_pending.clone();
    let visible_oam_law = state.oam_law_visible.clone();
    let star_tiles = state.ppu.vram[0x3ed0..0x3ef0].to_vec();
    let obj_tiles = state.ppu.vram[0x4000..0x4400].to_vec();
    let mut atomic = state.clone();

    let progress =
        sprite::DungeonResetSpritesCpuProgress::Load(sprite::DungeonLoadSpritesCpuProgress {
            normal_load_ordinal: 1,
            slot: 1,
            checkpoint: sprite::DungeonSpriteLoadCheckpoint::YHigh,
        });
    state.dungeon_reset_sprites_through_cpu_progress(progress);

    assert_eq!(state.sprite_slot_view(0).state(), 8);
    assert_eq!(state.sprite_get_y(0), 0x0060);
    assert_eq!(state.sprite_get_x(0), 0x0010);
    assert_eq!(state.sprite_slot_view(0).sprite_type(), 0x41);
    assert_eq!(state.sprite_slot_view(0).subtype(), 0x19);
    assert_eq!(state.sprite_slot_view(0).n(), 0);
    assert_eq!(state.sprite_slot_view(0).die_action(), 1);
    assert_eq!(state.sprite_slot_view(1).state(), 8);
    assert_eq!(state.sprite_slot_view(1).floor(), 0);
    assert_eq!(state.sprite_get_y(1), 0x0190);
    assert_eq!(state.sprite_get_x(1), 0x7777);
    assert_eq!(state.sprite_slot_view(1).sprite_type(), 0xaa);
    assert_eq!(state.sprite_slot_view(1).subtype(), 0xbb);
    assert_eq!(state.sprite_slot_view(1).n(), 0xcc);
    assert_eq!(state.sprite_slot_view(1).die_action(), 0xdd);
    assert_eq!(state.game_state.scratch_counter.value(), 0x79);
    for slot in [2, 9, 10] {
        assert_eq!(state.sprite_slot_view(slot).state(), 0);
    }

    state.set_subsubmodule(2);
    state.capture_display_snapshot();
    state.interrupt_nmi(0, None, false);
    assert_eq!(state.game_state.frame.frame_counter, 0xf8);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 24);
    assert_eq!(&state.ram[OAM_BUF..OAM_BUF + 0x220], shadow_oam);
    assert_eq!(state.ppu.oam, resident_oam);
    assert_eq!(state.resident_oam_dma, resident_oam_dma);
    assert_eq!(state.oam_law_pending, pending_oam_law);
    assert_eq!(state.oam_law_visible, visible_oam_law);
    assert_eq!(&state.ppu.vram[0x3ed0..0x3ef0], star_tiles);
    assert_eq!(&state.ppu.vram[0x4000..0x4400], obj_tiles);
    let snapshot = state
        .display_snapshot
        .as_ref()
        .expect("held room-load NMI must retain a display snapshot");
    assert_eq!(snapshot.completed_oam_dma_after_capture, None);
    if let Some(receipt) = snapshot.effective_presented_dma.as_ref() {
        assert!(receipt.vram_writes.is_empty());
        assert!(receipt.decoded_bg_vram_writes.is_empty());
        assert!(receipt.completed_oam.is_none());
        assert!(receipt.completed_link_obj_dma.is_none());
    }

    state.dungeon_resume_reset_sprites_after_cpu_progress(progress);
    atomic.dungeon_reset_sprites();
    for slot in 0..2 {
        assert_eq!(
            state.sprite_slot_view(slot).state(),
            atomic.sprite_slot_view(slot).state()
        );
        assert_eq!(state.sprite_get_y(slot), atomic.sprite_get_y(slot));
        assert_eq!(state.sprite_get_x(slot), atomic.sprite_get_x(slot));
        assert_eq!(
            state.sprite_slot_view(slot).sprite_type(),
            atomic.sprite_slot_view(slot).sprite_type(),
        );
        assert_eq!(
            state.sprite_slot_view(slot).subtype(),
            atomic.sprite_slot_view(slot).subtype(),
        );
        assert_eq!(
            state.sprite_slot_view(slot).n(),
            atomic.sprite_slot_view(slot).n()
        );
        assert_eq!(
            state.sprite_slot_view(slot).die_action(),
            atomic.sprite_slot_view(slot).die_action(),
        );
    }
}

#[test]
fn c_room_load_caller_clears_12_before_the_next_leading_nmi() {
    // C ZeldaRunGameLoop returns through NMI_PrepareSprites and then executes
    // `nmi_boolean = 0` before waiting. The next Interrupt_NMI therefore runs
    // NMI_DoUpdates exactly once; hardware cadence belongs to the scheduler,
    // not to a synthetic retained software latch.
    let continuation = GameWorkContinuation::FinishDungeonSupertileTransition {
        work: DungeonSupertileTransitionWork::RoomLoadCallerResume,
    };
    let mut state = ZeldaState::new();
    state
        .game_execution_scheduler
        .schedule_work(continuation, 1);
    state.game_execution_scheduler.begin_host_frame();
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(continuation)),
    );
    assert!(state
        .game_execution_scheduler
        .resumed_call_stack_is_before_nmi());

    state.latch_nmi_update();
    state.finish_dungeon_room_load_caller_at_main_wait();

    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_execution_scheduler.pre_main_nmi_resume(), None);
    assert!(state
        .game_execution_scheduler
        .returned_main_is_waiting_for_nmi());

    state.game_execution_scheduler.begin_host_frame();
    assert!(state
        .game_execution_scheduler
        .main_return_requires_leading_nmi());

    // nmi.c NMI_DoUpdates performs one complete $220-byte OAM copy and
    // consumes pending BG work when that cleared latch reaches the NMI.
    let expected_oam = (0..0x220)
        .map(|index| (index as u8).wrapping_mul(37).wrapping_add(11))
        .collect::<Vec<_>>();
    state.ram[OAM_BUF..OAM_BUF + expected_oam.len()].copy_from_slice(&expected_oam);
    state.ppu.oam.fill(0xdead);
    state.set_pending_nmi_subroutine(1);
    state.set_nmi_load_target_page(0x22);
    state.capture_display_snapshot();
    state.interrupt_nmi_for_active_scanout(0, None, false);

    let resident_oam = state
        .ppu
        .oam
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect::<Vec<_>>();
    assert_eq!(resident_oam, expected_oam);
    assert_eq!(state.game_state.display.pending_nmi_subroutine, 0);
    assert!(state.game_state.display.nmi_update_is_latched());
}

#[test]
fn dungeon_room_load_and_sprite_conversion_route_contracts() {
    assert_eq!(
        PreMainNmiResume::DungeonSupertileQuadrantUploads.nmi_latch_clear_phase(),
        Some(NmiPhase::BeforeNmi),
    );
    assert_eq!(
        PreMainNmiResume::OverworldAuxGraphicsReturn.nmi_latch_clear_phase(),
        None,
    );

    let room_load_return = FrameState {
        main_module: 7,
        submodule: 2,
        subsubmodule: 2,
        ..FrameState::default()
    };
    assert!(rom_dungeon_module_iteration_runs_after_leading_nmi(
        room_load_return,
        0x61,
    ));
    assert!(!rom_dungeon_module_iteration_runs_after_leading_nmi(
        room_load_return,
        0x52,
    ));
    assert!(!rom_dungeon_module_iteration_runs_after_leading_nmi(
        FrameState {
            subsubmodule: 3,
            ..room_load_return
        },
        0x61,
    ));

    assert!(room_61_sprite_conversion_retains_resident_oam(
        DungeonSupertileTransitionWork::SpriteConversion,
        0x61,
    ));
    assert!(room_61_sprite_conversion_retains_resident_oam(
        DungeonSupertileTransitionWork::SpriteConversionCallerResume,
        0x61,
    ));
    assert!(!room_61_sprite_conversion_retains_resident_oam(
        DungeonSupertileTransitionWork::SpriteConversion,
        0x52,
    ));
    assert!(!room_61_sprite_conversion_retains_resident_oam(
        DungeonSupertileTransitionWork::AuxiliarySpriteGraphics,
        0x61,
    ));
}

#[test]
fn room_load_sprite_main_preserves_every_measured_nmi_slice() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(2);
    state.set_dungeon_room_index(0x22);
    for slot in 0..=6 {
        state.sprite_slot_view_mut(slot).set_state(8);
        state.sprite_slot_view_mut(slot).set_sprite_type(0x6d);
    }
    state.sprite_main_cpu_boundary = Some(SpriteMainCpuBoundary::AfterSlot(5));
    state.sprite_main_cpu_nmi_slices = 4;

    state.sprite_main();

    assert_eq!(state.sprite_slot_view(6).state(), 9);
    assert_eq!(state.sprite_slot_view(5).state(), 9);
    for slot in 0..5 {
        assert_eq!(state.sprite_slot_view(slot).state(), 8);
    }
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::AfterSlot(5),
            caller: SpriteMainCpuCaller::DungeonModule07,
        }),
    );
    for _ in 0..3 {
        assert_eq!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Waiting),
        );
    }
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishSpriteMain {
                boundary: SpriteMainCpuBoundary::AfterSlot(5),
                caller: SpriteMainCpuCaller::DungeonModule07,
            },
        )),
    );
}

#[test]
fn forwarded_sprite_main_interruption_proves_spiral_caller_resumed() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    let interruption = crate::MainLoopInterruption::SpriteMainAfterAntfairySubtype2Increment(0);
    let mut receipts = OriginalTimingHostReceipts::new(
        95_850,
        0,
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        )],
    );
    receipts
        .forward_main_loop_interruption(interruption, OriginalTimingBoundary::NmiAccepted)
        .unwrap();
    state.original_timing_semantic_receipts = Some(receipts);

    assert!(state.wire_spiral_caller_resumed_this_host());
}

#[test]
fn dungeon_push_block_checkpoint_preserves_the_common_callers_scroll_locals() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(2);
    state.set_bg2_h_copy2(0x1010);
    state.set_bg2_v_copy2(0x2020);
    state.set_bg1_h_copy2(0x3030);
    state.set_bg1_v_copy2(0x4040);
    state.set_bg1_x_offset(3);
    state.set_bg1_y_offset(5);
    let mut atomic = state.clone();
    let old_slots = state.game_state.sprites.sprite_slots.clone();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        1,
        0,
        vec![OriginalTimingSemanticReceipt::DungeonPushBlocksPending],
    ));
    state.complete_module07_dungeon_after_submodule();
    assert_eq!(state.game_state.sprites.sprite_slots, old_slots);
    let Some(GameWorkContinuation::FinishDungeonPushBlocks { dungeon }) =
        state.game_execution_scheduler.current_work()
    else {
        panic!("push-block source caller was not retained");
    };
    assert_eq!(
        (dungeon.bg2_x, dungeon.bg2_y, dungeon.bg1_x, dungeon.bg1_y),
        (0x1010, 0x2020, 0x3030, 0x4040)
    );
    assert_eq!(
        state.game_state.display.ppu_scroll_copy.bg2_h_copy2(),
        0x1013
    );
    assert!(!state.original_timing_dungeon_push_blocks_pending());
    state
        .game_execution_scheduler
        .advance_work_one_nmi_slice_with_authoritative_completion(true);
    state.original_timing_owner = atomic.original_timing_owner.clone();
    state.original_timing_semantic_receipts = None;
    state.sprite_dungeon_draw_all_push_blocks();
    state.run_module07_sprite_main_caller(dungeon);
    atomic.complete_module07_dungeon_after_submodule();
    assert_eq!(state.game_state, atomic.game_state);
    assert_eq!(state.ram, atomic.ram);
}

#[test]
fn continued_peg_cursor_is_validated_before_timeline_consumption() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(7);
    state.set_submodule(7);
    let progress = crate::DungeonPegAttributeFlipProgressReceipt {
        index: 51,
        completed_banks: 2,
        boundary: OriginalTimingBoundary::NmiAccepted,
    };
    state.dungeon_peg_attribute_flip_pending = Some(DungeonPegAttributeFlipContinuation {
        caller: DungeonPegAttributeFlipCaller::FallingTransition,
        progress,
    });
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn,
        1,
    );
    let semantic = vec![
        OriginalTimingSemanticReceipt::DungeonPegAttributeFlipProgress(progress),
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        OriginalTimingSemanticReceipt::SpriteMainReturned,
        OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
        OriginalTimingSemanticReceipt::MainLoopInterrupted(
            crate::MainLoopInterruption::SpritePreparation,
        ),
        OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        ),
    ];
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        1206044,
        0,
        semantic.clone(),
    ));
    assert_eq!(
        state.preflight_continued_peg_attribute_flip(),
        Some(progress)
    );
    assert_eq!(
        state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic(),
        semantic
    );
    state
        .original_timing_semantic_receipts
        .as_mut()
        .unwrap()
        .semantic
        .remove(1);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
        || state.preflight_continued_peg_attribute_flip()
    ))
    .is_err());
}

#[test]
fn dungeon_caller_owns_sprite_return_while_common_suffix_remains_held() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        1206044,
        0,
        vec![OriginalTimingSemanticReceipt::SpriteMainReturned],
    ));
    state.complete_dungeon_after_submodule_caller_return();
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .unwrap()
        .semantic()
        .is_empty());
    assert_eq!(
        state.original_timing_sprite_main_return_claims_remaining,
        None
    );
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch)
    );
}

#[test]
fn overworld_link_shadow_checkpoint_restores_the_temporary_stair_y() {
    let mut state = ZeldaState::new();
    state.set_main_module(9);
    state.set_submodule(18);
    state.follower_link_state_mut().set_position(0x80, 0x80);
    state.follower_link_state_mut().set_animation_step(1);
    let mut atomic = state.clone();
    atomic.link_oam_main();
    let continuation = state.link_oam_before_equipment();
    let continuation = state.link_oam_before_shadow(continuation);
    assert_eq!(state.game_state.player.follower_link.y(), 0x7e);
    state.link_oam_after_equipment(continuation);
    assert_eq!(state.game_state.player.follower_link.y(), 0x80);
    assert_eq!(state.game_state, atomic.game_state);
    assert_eq!(state.ram, atomic.ram);
}

#[test]
fn room_72_quadrant_builder_completes_inside_rom_timed_dispatcher_iteration() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(5);
    state.set_dungeon_room_index(0x72);

    state.Dungeon_InterRoomTrans_notDarkRoom();

    assert_eq!(state.game_state.frame.subsubmodule, 6);
    assert!(state.paired_resume_cpu_boundary_is_quiescent());
}

#[test]
fn filtered_state_10_quadrant_work_is_not_room_specific() {
    for room in [0x21, 0x22, 0x41] {
        let mut state = ZeldaState::new();
        state.restore_live_rom_timing_after_checkpoint();
        state.set_main_module(7);
        state.set_submodule(2);
        state.set_subsubmodule(10);
        state.set_dungeon_room_index(room);

        assert!(state.begin_dungeon_supertile_transition_work_with_palette(
            DungeonSupertileTransitionWork::FilteredQuadrantTilemapBuild,
            Some(187_620),
        ));
        assert_eq!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Complete(
                GameWorkContinuation::FinishDungeonSupertileTransition {
                    work: DungeonSupertileTransitionWork::FilteredQuadrantTilemapBuild,
                },
            )),
        );
    }
}

#[test]
fn filtered_state_11_quadrant_work_is_not_room_specific() {
    assert!(matches!(
        dungeon_supertile_state_11_cpu_advance(189_976),
        CpuPhaseSequenceAdvance::ReachedBoundary {
            boundary: CpuRasterBoundary::VblankPublication,
            phase_index: 0,
            ..
        }
    ));

    for room in [0x21, 0x22, 0x41] {
        let mut state = ZeldaState::new();
        state.restore_live_rom_timing_after_checkpoint();
        state.set_main_module(7);
        state.set_submodule(2);
        state.set_subsubmodule(11);
        state.set_dungeon_room_index(room);
        state.dungeon_torch_mut().set_lights_out_request(1);
        state.set_countdown_word(29);
        for index in [0..1, 0x20..0xd8, 0xe0..0xf0].into_iter().flatten() {
            state.set_aux_color_constant(index, 0x7fff);
        }
        let palette_work = palette_filter_bounce_loop_master_cycles(&state);
        assert!(matches!(
            dungeon_supertile_state_11_cpu_advance(palette_work),
            CpuPhaseSequenceAdvance::ReachedBoundary {
                boundary: CpuRasterBoundary::VblankPublication,
                phase_index: 0,
                ..
            }
        ));

        state.Dungeon_InterRoomTrans_State9();

        assert_eq!(state.game_state.frame.subsubmodule, 11);
        assert_eq!(
            state.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::QuadrantUploadCallerReturn,
            })
        );
    }
}

#[test]
fn state_13_caller_return_accepts_post_call_state_14_in_any_room() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(14);
    state.set_dungeon_room_index(0x42);

    assert!(state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::State13CallerReturn,
    ));
}

#[test]
fn spiral_return_main_loop_reentry_advances_frame_and_clears_oam() {
    let mut state = ZeldaState::new();
    state.set_frame_counter(0x12);
    state.oam_state_mut().set_entry_y(OAM_BUF, 0x44);
    state.begin_spiral_stair_return_main_loop_reentry();

    assert_eq!(FrameState::load_from_ram(&state.ram).frame_counter, 0x13);
    assert_eq!(state.ram[OAM_BUF + 1], 0xf0);
}

#[test]
fn room_1_staircase_30_spiral_bg_character_graphics_cross_three_nmi_slices() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_dungeon_room_index(0x01);
    state.dungeon_stair_movement_mut().set_staircase_index(0x30);
    state
        .dungeon_stair_movement_mut()
        .set_staircase_lower_level_status(0);

    assert!(state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::SpiralBgCharacters34,
    ));
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Waiting),
    );
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Waiting),
    );
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishDungeonSupertileTransition {
                work: DungeonSupertileTransitionWork::SpiralBgCharacters34,
            },
        )),
    );
}

#[test]
fn spiral_room_initialization_uses_cpu_derived_submodule_slices() {
    let mut state = spiral_cpu_test_state(3, injected_dungeon_cpu_schedule(2, 0));
    assert!(state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::SpiralRoomInitialization,
    ));
    assert_eq!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Waiting),
    );
    assert!(matches!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(_)),
    ));
}

#[test]
fn spiral_sprite_graphics_uses_cpu_derived_submodule_slices() {
    let mut state = spiral_cpu_test_state(6, injected_dungeon_cpu_schedule(4, 0));
    assert!(state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::SpiralSpriteGraphics,
    ));
    for _ in 0..3 {
        assert_eq!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Waiting),
        );
    }
    assert!(matches!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(_)),
    ));
}

#[test]
fn zero_slice_spiral_background_schedule_does_not_suspend() {
    let mut state = spiral_cpu_test_state(7, injected_dungeon_cpu_schedule(0, 0));
    assert!(!state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::SpiralBackgroundSync,
    ));
    assert!(state.dungeon_submodule_cpu_schedule.is_none());
    assert!(state.game_execution_scheduler.is_idle());
}

#[test]
fn spiral_sprite_schedule_does_not_depend_on_room_identity() {
    for room in [0x70, 0x71, 0x80] {
        let mut state = spiral_cpu_test_state(6, injected_dungeon_cpu_schedule(2, 0));
        state.set_dungeon_room_index(room);
        assert!(state.begin_dungeon_supertile_transition_work(
            DungeonSupertileTransitionWork::SpiralSpriteGraphics,
        ));
        assert_eq!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Waiting),
        );
        assert!(matches!(
            state.game_execution_scheduler.advance_work_one_nmi_slice(),
            Some(GameWorkStep::Complete(_)),
        ));
    }
}

#[test]
fn spiral_cpu_schedule_keeps_caller_nmis_separate_from_submodule_nmis() {
    let schedule = injected_dungeon_cpu_schedule(1, 4);
    let mut state = spiral_cpu_test_state(3, schedule);
    assert!(state.begin_dungeon_supertile_transition_work(
        DungeonSupertileTransitionWork::SpiralRoomInitialization,
    ));
    assert!(matches!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(_)),
    ));
    assert_eq!(state.dungeon_submodule_cpu_schedule, Some(schedule));
}

#[test]
fn spiral_room_initializer_completes_from_consumed_sprite_main_return_authority() {
    let schedule = injected_dungeon_cpu_schedule(4, 0);
    let mut state = spiral_cpu_test_state(3, schedule);
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_nmi_publication_pending = true;
    state.latch_nmi_update();
    state.capture_display_snapshot();
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::SpiralRoomInitialization,
        },
        4,
    );
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        237_365,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::SpriteMainReturned,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
        ],
    ));

    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.subsubmodule, 4);
    assert!(state.game_execution_scheduler.is_idle());
    assert!(state.original_timing_nmi_publication_pending);
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn spiral_background_return_retires_its_already_consumed_suffix_authority() {
    let mut state = spiral_cpu_test_state(4, injected_dungeon_cpu_schedule(3, 0));
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.latch_nmi_update();
    state.original_timing_expected_nmi_update_gates = vec![NmiUpdateGate::LatchHeld];
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishDungeonSupertileTransition {
            work: DungeonSupertileTransitionWork::SpiralBgCharacters34,
        },
        3,
    );
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        13_627,
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
    ));
    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);
    assert_eq!(state.game_state.frame.subsubmodule, 5);
    assert!(state.game_execution_scheduler.is_idle());
    assert!(state.pending_main_loop_common_suffix.is_none());
    assert!(!state.game_state.display.nmi_update_is_latched());
}

#[test]
fn dungeon_quadrant_hold_uses_the_dma_latched_resident_oam_without_copying_it() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.resident_oam_dma = Some(vec![0x1111; state.ppu.oam.len()]);
    state.ppu.oam.fill(0x2222);

    state.stage_dungeon_supertile_quadrant_upload_obj_scanout();

    assert_eq!(state.next_display_obj_memory_generation, None);
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
        })
    );

    state.capture_display_snapshot();
    let display = state.display_snapshot.as_deref().unwrap();
    assert!(display.ppu.oam.iter().all(|&word| word == 0x1111));
    assert_eq!(
        display.obj_generation,
        DisplayObjGeneration::FollowModuleCadence
    );
}

#[test]
fn live_fresh_dungeon_iteration_retires_only_the_pre_main_nmi_timing_shadow() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(4);
    state.set_frame_counter(3);
    state.project_native_game_state_to_ram();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_nmi_publication_pending = true;
    state.original_timing_pending_nmi_update_gate = Some(NmiUpdateGate::Open);
    state.original_timing_expected_nmi_update_gates =
        vec![NmiUpdateGate::Open, NmiUpdateGate::LatchHeld];
    state.capture_display_snapshot();
    state
        .game_execution_scheduler
        .schedule_pre_main_nmi_resume(PreMainNmiResume::DungeonSupertileQuadrantUploads);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        31_364,
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
            OriginalTimingSemanticReceipt::MainLoopInterrupted(
                crate::MainLoopInterruption::LinkOam,
            ),
        ],
    ));

    state.run_frame_internal_after_original_timing(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.frame_counter, 4);
    assert_eq!(state.game_state.frame.subsubmodule, 5);
    assert!(state.original_timing_nmi_publication_pending);
    assert_ne!(
        state.game_execution_scheduler.pre_main_nmi_resume(),
        Some(PreMainNmiResume::DungeonSupertileQuadrantUploads),
        "the source-proven fresh iteration must retire the native timing shadow",
    );
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonPostSpriteMainCallerReturn),
    ));
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn pre_dungeon_work_resumes_at_room_and_song_bank_transfer_boundaries() {
    assert_eq!(PRE_DUNGEON_ENTRANCE_LOAD_NMI_SLICES, 58);
    let stages = [
        (
            GameWorkContinuation::FinishPreDungeonEntranceLoad {
                sprite_reset: PreDungeonSpriteResetContinuation::Pending,
            },
            PRE_DUNGEON_ENTRANCE_LOAD_NMI_SLICES,
        ),
        (
            GameWorkContinuation::FinishPreDungeonSongBankTransfer,
            PRE_DUNGEON_SONG_BANK_TRANSFER_NMI_SLICES,
        ),
    ];

    for (continuation, nmi_slices) in stages {
        let mut work = ScheduledGameWork::schedule(continuation, nmi_slices);
        for _ in 0..nmi_slices - 1 {
            assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
        }
        assert_eq!(
            work.advance_one_nmi_slice(),
            GameWorkStep::Complete(continuation)
        );
        assert!(work.is_complete());
    }
}

#[test]
fn pre_dungeon_return_leaves_the_landing_cpu_trace_to_own_the_first_boundary() {
    let mut state = ZeldaState::new();
    state.game_execution_scheduler.begin_host_frame();

    assert_eq!(state.finish_pre_dungeon_caller_at_main_wait(), None);

    assert_eq!(state.sprite_main_cpu_boundary, None);
    assert_eq!(state.sprite_main_cpu_nmi_slices, 0);
    assert!(state
        .game_execution_scheduler
        .returned_main_is_waiting_for_nmi());
}

#[test]
fn pre_dungeon_return_defers_until_the_native_module6_owner_runs() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(6);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::PreDungeonModuleReturned,
        ],
    ));

    state.defer_original_timing_pre_dungeon_return_before_native_owner();

    assert_eq!(
        state.original_timing_pre_dungeon_return_pending,
        Some(crate::MainLoopProgress::CallStackContinued),
    );
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
    assert_eq!(
        state.take_deferred_original_timing_pre_dungeon_return(),
        Some(crate::MainLoopProgress::CallStackContinued),
    );
    assert_eq!(
        state.take_deferred_original_timing_pre_dungeon_return(),
        None,
        "the source return is a one-shot semantic fact",
    );
}

#[test]
#[should_panic(expected = "Module_PreDungeon returned without its delayed native Module 6 owner")]
fn pre_dungeon_return_before_native_owner_fails_closed_outside_module6() {
    let mut state = ZeldaState::new();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(7);
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::PreDungeonModuleReturned,
        ],
    ));

    state.defer_original_timing_pre_dungeon_return_before_native_owner();
}

#[test]
#[should_panic(expected = "outlived its timing authority")]
fn deferred_pre_dungeon_return_cannot_survive_authority_loss() {
    let mut state = ZeldaState::new();
    state.set_main_module(6);
    state.original_timing_pre_dungeon_return_pending =
        Some(crate::MainLoopProgress::CallStackContinued);

    let _ = state.take_deferred_original_timing_pre_dungeon_return();
}

#[test]
fn pre_dungeon_publishes_entrance_before_scheduled_room_construction() {
    let mut expected = ZeldaState::new();
    expected.assets = Some(probe_entrance_asset_pack(0, 0x0061));
    expected.set_which_entrance(0);
    expected.save_progress_mut().set_progress_indicator(2);
    expected.Dungeon_LoadEntrance();
    let expected_room = expected.game_state.world.location.dungeon_room();
    let expected_x = expected.game_state.player.follower_link.x();
    let expected_y = expected.game_state.player.follower_link.y();

    let mut state = ZeldaState::new();
    state.assets = Some(probe_entrance_asset_pack(0, 0x0061));
    state.set_main_module(6);
    state.set_which_entrance(0);
    state.save_progress_mut().set_progress_indicator(2);
    state.set_dungeon_room(0x0055);
    state.follower_link_state_mut().set_x(0x07f8);
    state.follower_link_state_mut().set_y(0x06f8);
    state.set_rom_startup_timing(true);

    state.module_pre_dungeon();

    assert_eq!(state.game_state.frame.main_module, 6);
    assert_eq!(
        state.game_state.world.location.dungeon_room(),
        expected_room
    );
    assert_eq!(state.game_state.player.follower_link.x(), expected_x);
    assert_eq!(state.game_state.player.follower_link.y(), expected_y);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishPreDungeonEntranceLoad {
            sprite_reset: PreDungeonSpriteResetContinuation::Pending,
        })
    );
}

#[test]
fn throne_room_work_budget_follows_the_retained_sprite_tileset() {
    assert_eq!(attract_throne_room_nmi_slices(19), 42);
    assert_eq!(attract_throne_room_nmi_slices(66), 44);
}

#[test]
fn prison_room_rom_work_resumes_after_its_room_build_nmis() {
    let mut work = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishAttractZeldaPrison,
        ATTRACT_ZELDA_PRISON_NMI_SLICES,
    );

    for _ in 0..ATTRACT_ZELDA_PRISON_NMI_SLICES - 1 {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(GameWorkContinuation::FinishAttractZeldaPrison)
    );
}

#[test]
fn maiden_warp_room_rom_work_resumes_after_its_room_build_nmis() {
    let mut work = ScheduledGameWork::schedule(
        GameWorkContinuation::FinishAttractMaidenWarp,
        ATTRACT_MAIDEN_WARP_NMI_SLICES,
    );

    for _ in 0..ATTRACT_MAIDEN_WARP_NMI_SLICES - 1 {
        assert_eq!(work.advance_one_nmi_slice(), GameWorkStep::Waiting);
    }
    assert_eq!(
        work.advance_one_nmi_slice(),
        GameWorkStep::Complete(GameWorkContinuation::FinishAttractMaidenWarp)
    );
}

#[test]
fn dungeon_landing_goal_authors_final_circle_before_resetting_the_live_table() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_main_module(7);
    state.set_submodule(0x0f);
    state.set_saved_module_for_menu(7);
    state.follower_link_state_mut().set_x(0x80);
    state.follower_link_state_mut().set_y(0x70);
    state.set_spotlight_window_state(2);
    state.set_spotlight_window_radius(0x77);

    assert!(state.iris_spotlight_configure_table());
    assert!(state.dungeon_landing_goal_transition_pending);

    let dynamic = state.ram
        [HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + ZeldaState::HDMA_DYNAMIC_TABLE_LEN]
        .to_vec();
    let reserved = state.ram
        [RESERVED_HDMA_TABLE..RESERVED_HDMA_TABLE + ZeldaState::HDMA_DYNAMIC_TABLE_LEN]
        .to_vec();
    assert_eq!(dynamic, reserved, "the C memcpy preserves the final circle");
    assert!(dynamic.chunks_exact(2).any(|word| word != [0x00, 0xff]));
    state.capture_display_snapshot();

    state.complete_iris_spotlight_goal_transition();

    assert!(
        state.ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + ZeldaState::HDMA_DYNAMIC_TABLE_LEN]
            .chunks_exact(2)
            .all(|word| word == [0x00, 0xff])
    );
    assert_eq!(
        &state.ram[RESERVED_HDMA_TABLE..RESERVED_HDMA_TABLE + ZeldaState::HDMA_DYNAMIC_TABLE_LEN],
        reserved,
        "the reset is a later live-table generation, not a rewrite of the final circle",
    );
    assert_eq!(
        state.with_display_snapshot(|display| display.ram
            [HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + ZeldaState::HDMA_DYNAMIC_TABLE_LEN]
            .to_vec()),
        dynamic,
        "the C reset cannot rewrite the field that already captured the final circle",
    );

    state.complete_dungeon_landing_goal_active_scanout(Vec::new());
    let reset_stream =
        state.with_display_snapshot(|display| spotlight_hdma_tables_from_ram(&display.ram));
    assert!(reset_stream
        .iter()
        .all(|table| table.chunks_exact(2).all(|word| word == [0x00, 0xff])));

    state.capture_display_snapshot();
    assert!(state
        .with_display_snapshot(|display| display.ram
            [HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + ZeldaState::HDMA_DYNAMIC_TABLE_LEN]
            .to_vec())
        .chunks_exact(2)
        .all(|word| word == [0x00, 0xff]));
}

#[test]
fn interrupted_dungeon_exit_build_retains_the_c_suffix() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_indoor_flag(0);
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.follower_link_state_mut().set_y(93);
    state.set_spotlight_window_radius(0x70);
    state.set_spotlight_window_state(0);
    for row in 0..240 {
        state.set_spotlight_hdma_table_dynamic_entry(row, 0x00ff);
    }
    let iteration =
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTableAfterTablePublication)
            .with_main_loop_sprite_preparation_before_second_nmi();
    assert!(state.begin_dungeon_exit_spotlight_build(
        Some(DungeonExitSpotlightCpuPlan {
            interrupted_pc: 0x00_f3be,
            interrupted_return_address: 0,
            iterations_before_nmi: usize::MAX,
            link_position_integrated_before_first_nmi: false,
            returned_to_main_wait_before_first_nmi: false,
            main_loop_sprite_preparation_completed_before_second_nmi: true,
            active_window_words: [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
            following_window_words: [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
            next_entry_earliest: Some(DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_EARLIEST),
            next_entry_latest: Some(DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_LATEST),
        }),
        None,
        iteration,
    ));

    // The C builder and copy have completed, but the radius write and Module0F
    // Link/OAM suffix remain on the interrupted call stack.
    assert_ne!(state.spotlight_hdma_table_dynamic_entry(0), 0x00ff);
    assert_eq!(
        read_le_u16(&state.ram, RESERVED_HDMA_TABLE),
        state.spotlight_hdma_table_dynamic_entry(0),
    );
    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        0x70
    );
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonExitSpotlightBuild {
            iteration: pending,
            ..
        }) if pending == iteration
    ));

    // Model the long-close path which already returned through the C main-loop
    // sprite preparation before this scheduled publication continuation.
    state.nmi_prepare_sprites_for_main_loop();
    write_le_u16(&mut state.ram, LINK_DMA_COUNTDOWN, 9);
    state.set_bg_tile_animation_countdown(7);
    state.complete_dungeon_exit_spotlight_build(
        SpotlightTableBuildContinuation::default(),
        true,
        iteration,
        false,
        false,
    );
    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        0x69
    );
    assert_eq!(read_le_u16(&state.ram, LINK_DMA_COUNTDOWN), 9);
    assert_eq!(state.game_state.display.bg_tile_animation_countdown, 7);
}

#[test]
fn interrupted_dungeon_exit_table_build_defers_the_radius_write_until_return() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_indoor_flag(0);
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.follower_link_state_mut().set_y(93);
    state.set_spotlight_window_radius(0x77);
    state.set_spotlight_window_state(0);
    let iteration = SpotlightIteration::closing(SpotlightIterationPhase::WholeTable)
        .with_main_loop_sprite_preparation_before_second_nmi();

    assert!(state.begin_dungeon_exit_spotlight_build(
        Some(DungeonExitSpotlightCpuPlan {
            interrupted_pc: 0x00_f38d,
            interrupted_return_address: 0,
            iterations_before_nmi: 19,
            link_position_integrated_before_first_nmi: false,
            returned_to_main_wait_before_first_nmi: false,
            main_loop_sprite_preparation_completed_before_second_nmi: true,
            active_window_words: [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
            following_window_words: [0x00ff; SPOTLIGHT_VISIBLE_SCANLINES],
            next_entry_earliest: Some(DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_EARLIEST),
            next_entry_latest: Some(DUNGEON_EXIT_SPOTLIGHT_CPU_ENTRY_LATEST),
        }),
        None,
        iteration,
    ));

    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        0x77
    );
    let Some(GameWorkContinuation::FinishDungeonExitSpotlightBuild {
        table_build,
        projection_completed,
        iteration: pending,
    }) = state.game_execution_scheduler.current_work()
    else {
        panic!("interrupted table build did not retain its C continuation");
    };
    assert!(!projection_completed);
    assert_eq!(pending, iteration);

    state.game_execution_scheduler.finish_work();
    state.complete_dungeon_exit_spotlight_build(
        table_build,
        projection_completed,
        iteration,
        false,
        false,
    );
    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        0x70
    );
}

#[test]
fn interrupted_dungeon_exit_build_runs_the_unfinished_c_main_loop_sprite_prep() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_indoor_flag(0);
    state.set_main_module(0x0f);
    state.set_submodule(1);
    state.follower_link_state_mut().set_y(93);
    state.set_spotlight_window_radius(0x70);
    state.set_spotlight_window_state(0);
    write_le_u16(&mut state.ram, LINK_DMA_COUNTDOWN, 9);
    state.set_bg_tile_animation_countdown(7);
    state.complete_dungeon_exit_spotlight_build(
        SpotlightTableBuildContinuation::default(),
        true,
        SpotlightIteration::closing(SpotlightIterationPhase::WholeTableAfterTablePublication)
            .with_main_loop_sprite_preparation_before_second_nmi(),
        false,
        false,
    );

    assert_eq!(
        state.game_state.display.spotlight_hdma.window_radius(),
        0x69
    );
    assert_eq!(read_le_u16(&state.ram, LINK_DMA_COUNTDOWN), 8);
    assert_eq!(state.game_state.display.bg_tile_animation_countdown, 6);
}

#[test]
fn dungeon_map_oam_shadow_reaches_the_following_scanout() {
    let map = rom_graphics_dma_plan(14, 3);

    assert_eq!(map.oam_operands, GraphicsDmaGeneration::LiveAfterMain);
    assert_eq!(map.oam_scanout, OamScanoutSource::RetainCapturedBeforeNmi);
    assert_eq!(map.link_obj_scanout, GraphicsDmaGeneration::LiveAfterMain);
}

#[test]
fn room_01_spiral_state_8_marks_the_pending_hud_dma_live() {
    let frame = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0x0e,
        subsubmodule: 8,
        ..Default::default()
    };
    assert!(rom_dungeon_spiral_state_8_publishes_live_hud_tilemap(
        frame, 0x01, true,
    ));
    assert!(!rom_dungeon_spiral_state_8_publishes_live_hud_tilemap(
        frame, 0x02, true,
    ));
    assert!(!rom_dungeon_spiral_state_8_publishes_live_hud_tilemap(
        frame, 0x01, false,
    ));
}

#[test]
fn resumed_spiral_state_7_keeps_its_sound_queued_until_the_next_nmi() {
    // Source host 23932: Open NMI, handler completion, fresh main, suffix.
    // The floor-change blip ($24) is authored after that audio sample.
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.rom_reset_frame_delay = 0;
    state.initialized = true;
    state.set_animated_tile_data_source_address(0xa680);
    state.set_indoor_flag(1);
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_subsubmodule(7);
    state.dungeon_stair_movement_mut().set_staircase_index(4);
    state.game_state.write_to_ram(&mut state.ram);
    state.ram[0xa0] = 1;
    state.sync_native_game_state_from_ram();
    // The source completes this short state in one host. Isolate command
    // publication from the development-only ROM CPU schedule measurement.
    state.dungeon_submodule_cpu_schedule = Some(DungeonSubmoduleCpuSchedule::default());
    state.game_execution_scheduler.schedule_pre_main_nmi_resume(
        PreMainNmiResume::DungeonSupertileQuadrantUploads,
    );
    assert!(state.resume_after_pre_main_nmi(0, None));
    assert_eq!(state.game_state.frame.subsubmodule, 8);
    assert_eq!(state.game_state.system_signals.sound_effect_2(), 0x24);
    assert_eq!(state.zelda_audio_route_state().queue.write[3], 0);
    state.interrupt_nmi_audio_parts();
    assert_eq!(state.game_state.system_signals.sound_effect_2(), 0);
    assert_eq!(state.zelda_audio_route_state().queue.write[3], 0x24);
}

#[test]
fn dungeon_entrance_publishes_the_animated_bg_written_by_its_leading_nmi() {
    let mut state = ZeldaState::new();
    let destination = 0x3b00;
    state.set_main_module(0x11);
    state.set_submodule(7);
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
fn throne_room_story_resumes_one_nmi_slice_sooner_than_the_first_story() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.attract_scene_mut().set_sequence(2);
    state.attract_scene_mut().set_state(4);
    state.set_screen_brightness(15);

    state.attract_fade_in_sequence();

    assert_eq!(state.game_state.ending.attract_scene.state(), 5);
    assert_eq!(state.attract_first_story_render_delay, 5);
}

#[test]
fn pre_dungeon_audio_boundary_completes_then_carries_held_nmis_across_presentation() {
    let mut state = live_selected_game_load_state_before_pre_dungeon_audio();
    state.set_ambient_sound_effect(3);

    // Source host 2235 accepts a held NMI inside Decompression_GetNextByte,
    // completes that handler, resumes into Module_PreDungeon and the entry
    // room load, then accepts a second held NMI at $02:dc76.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2235,
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
        SELECTED_GAME_LOAD_AFTER_PRE_DUNGEON_AUDIO_NMI_SLICES,
    );
    assert_eq!(
        state.game_state.system_signals.last_ambient_sound_effect(),
        3,
        "the first handler must consume the pre-closure ambient command",
    );
    assert_eq!(
        state.game_state.system_signals.ambient_sound_effect(),
        5,
        "Module_PreDungeon must publish ambient 5 after the completed handler",
    );
    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_after_pre_dungeon_audio_sprite_reset(),
        Some(PreDungeonSpriteResetContinuation::Pending),
    );
    assert!(state.game_state.display.nmi_update_is_latched());
    assert_eq!(
        state.pending_main_loop_common_suffix,
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch),
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

    // The runner presents and advances history between the acceptance and
    // completion hosts. That must not close the carried scanout owner.
    state.with_display_snapshot(|_| ());
    state.advance_display_publication_history();
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));

    // Host 2236 completes exactly that handler before continuing the same
    // selected-load caller and accepting the following held NMI.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2236,
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
        state.game_state.system_signals.last_ambient_sound_effect(),
        5,
        "the next host must consume the ambient command exactly once",
    );
    assert_eq!(state.game_state.system_signals.ambient_sound_effect(), 0);
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
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(2236));
}

#[test]
fn selected_game_load_freezes_a_mosaic_selected_dungeon_before_the_prefix_clears_it() {
    let mut state = live_selected_game_load_state_before_pre_dungeon_audio();
    state.game_execution_scheduler.reset();
    state.save_progress_mut().set_dark_world_state(0);
    state.save_progress_mut().set_progress_indicator(2);
    state.save_progress_mut().set_which_starting_point(0);
    state.clear_game_over_check_flag();
    state.clear_restart_check_flag();
    state.clear_mosaic_level();
    assert_eq!(
        state.selected_game_load_destination(),
        SelectedGameLoadDestination::Message,
        "without mosaic no other mutable selector should choose the dungeon route",
    );
    state.set_mosaic_level(0x10);
    assert_eq!(
        state.selected_game_load_destination(),
        SelectedGameLoadDestination::Dungeon,
    );

    state.begin_selected_game_load();
    state.rom_load_partial_nmi_this_frame = false;
    for _ in 0..SELECTED_GAME_LOAD_BEFORE_PRE_DUNGEON_AUDIO_NMI_SLICES - 1 {
        assert_eq!(
            state.game_execution_scheduler.advance_startup_sequence(),
            Some(StartupSequenceStep::SelectedGameLoadWaiting),
        );
    }
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2235,
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

    assert_eq!(state.game_state.display.mosaic_level, 0);
    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_destination(),
        Some(SelectedGameLoadDestination::Dungeon),
        "the scheduler must retain the destination selected before the HUD prefix cleared mosaic",
    );
    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_after_pre_dungeon_audio_sprite_reset(),
        Some(PreDungeonSpriteResetContinuation::Pending),
    );

    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2236,
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

    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_destination(),
        Some(SelectedGameLoadDestination::Dungeon),
    );
    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_after_pre_dungeon_audio_sprite_reset(),
        Some(PreDungeonSpriteResetContinuation::SpriteDisableAllCompleted),
    );

    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2237,
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

    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.frame.main_module, 7);
}

#[test]
fn pre_dungeon_audio_boundary_rejects_malformed_nmi_order_before_mutation() {
    for semantic in [
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
        ],
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
        ],
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::MainLoopProgress(
                crate::MainLoopProgress::CallStackContinued,
            ),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ],
    ] {
        let mut state = live_selected_game_load_state_before_pre_dungeon_audio();
        state.set_ambient_sound_effect(3);
        state.set_sound_effect_1(0x44);
        // Install the malformed runtime fixture directly: the public
        // installer independently rejects some of these lifecycles, while
        // this regression proves the scheduler boundary itself is also
        // failure-atomic if restored state is corrupt.
        state.original_timing_owner = OriginalTimingOwnerState::Live;
        state.original_timing_semantic_receipts =
            Some(OriginalTimingHostReceipts::new(2235, 0, semantic));
        state.original_timing_expected_nmi_update_gates =
            vec![NmiUpdateGate::LatchHeld, NmiUpdateGate::LatchHeld];
        let receipts_before = state
            .original_timing_semantic_receipts
            .as_ref()
            .unwrap()
            .semantic
            .clone();
        let expected_gates_before = state.original_timing_expected_nmi_update_gates.clone();
        let frame_before = state.game_state.frame;
        let room_before = state.game_state.world.location.dungeon_room();
        let scheduler_before = state.game_execution_scheduler;
        let remaining_before = state
            .game_execution_scheduler
            .selected_game_load_remaining_nmi_slices();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            state.run_frame_internal(0, crate::RUN_MAIN);
        }));

        assert!(result.is_err());
        assert_eq!(
            state
                .game_execution_scheduler
                .selected_game_load_remaining_nmi_slices(),
            remaining_before,
        );
        assert_eq!(state.game_state.frame, frame_before);
        assert_eq!(state.game_state.world.location.dungeon_room(), room_before);
        assert_eq!(state.game_execution_scheduler, scheduler_before);
        assert_eq!(state.game_state.system_signals.ambient_sound_effect(), 3);
        assert_eq!(
            state.game_state.system_signals.last_ambient_sound_effect(),
            0
        );
        assert_eq!(state.game_state.system_signals.sound_effect_1(), 0x44);
        assert!(state
            .game_execution_scheduler
            .selected_game_load_after_pre_dungeon_audio_sprite_reset()
            .is_none());
        assert!(state.display_snapshot.is_none());
        assert!(!state.original_timing_nmi_publication_pending);
        assert_eq!(state.original_timing_pending_nmi_update_gate, None);
        assert_eq!(
            state.original_timing_expected_nmi_update_gates,
            expected_gates_before,
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
fn terminal_dark_world_overworld_load_returns_without_pre_dungeon_audio() {
    let mut state = live_selected_game_load_state_before_pre_dungeon_audio();
    state.game_execution_scheduler.reset();
    // The dark-world overworld reload after a dark-world death returns directly
    // from Module05 to Module08. It never enters
    // Module_PreDungeon and therefore owns neither its audio prefix nor its
    // nested Sprite_ResetAll call.
    state
        .game_execution_scheduler
        .schedule_selected_game_load(SelectedGameLoadDestination::DarkWorldOverworld);
    state.set_vertical_irq_trigger(0x7b);
    state.set_ambient_sound_effect(3);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2235,
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

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_after_pre_dungeon_audio_sprite_reset(),
        None,
    );
    assert!(state.game_execution_scheduler.is_idle());
    assert_eq!(state.game_state.frame.main_module, 8);
    assert_eq!(state.game_state.system_signals.ambient_sound_effect(), 0);
    assert_eq!(
        state.game_state.system_signals.last_ambient_sound_effect(),
        3
    );
}

#[test]
fn pre_dungeon_audio_boundary_requires_held_native_latch_before_mutation() {
    let mut state = live_selected_game_load_state_before_pre_dungeon_audio();
    state.set_ambient_sound_effect(3);
    state.set_sound_effect_1(0x44);
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2235,
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
    // Corrupt only the native latch after installing otherwise coherent
    // source authority. Preflight must reject it before audio or CPU work.
    state.clear_nmi_update_latch();
    let receipts_before = state
        .original_timing_semantic_receipts
        .as_ref()
        .unwrap()
        .semantic
        .clone();
    let expected_gates_before = state.original_timing_expected_nmi_update_gates.clone();
    let frame_before = state.game_state.frame;
    let room_before = state.game_state.world.location.dungeon_room();
    let remaining_before = state
        .game_execution_scheduler
        .selected_game_load_remaining_nmi_slices();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.run_frame_internal(0, crate::RUN_MAIN);
    }));

    assert!(result.is_err());
    assert_eq!(
        state
            .game_execution_scheduler
            .selected_game_load_remaining_nmi_slices(),
        remaining_before,
    );
    assert_eq!(state.game_state.frame, frame_before);
    assert_eq!(state.game_state.world.location.dungeon_room(), room_before);
    assert_eq!(state.game_state.system_signals.ambient_sound_effect(), 3);
    assert_eq!(
        state.game_state.system_signals.last_ambient_sound_effect(),
        0
    );
    assert_eq!(state.game_state.system_signals.sound_effect_1(), 0x44);
    assert!(state
        .game_execution_scheduler
        .selected_game_load_after_pre_dungeon_audio_sprite_reset()
        .is_none());
    assert!(state.display_snapshot.is_none());
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert_eq!(
        state.original_timing_expected_nmi_update_gates,
        expected_gates_before,
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
#[ignore = "requires the local pinned Zelda ROM used by the retained source trace"]
fn selected_game_load_host2293_consumes_the_leading_open_nmi_before_dungeon_main() {
    let mut state = live_selected_game_load_state_before_terminal_hosts();
    state.set_frame_counter(124);
    let rom_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("saves/zelda3.sfc");
    state.set_rom(&std::fs::read(&rom_path).expect("read the local pinned Zelda ROM"));

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
    assert!(state
        .game_execution_scheduler
        .returned_main_is_waiting_for_nmi());
    assert!(state.dungeon_landing_cpu_advance_pending.is_none());

    // The pinned successor enters at $00:8034, accepts the leading Open NMI
    // at $00:8036, completes the handler, and only then starts the first
    // dungeon iteration. It returns mid-call, so this host has no common
    // suffix completion fact.
    state
        .install_original_timing_host_receipts(OriginalTimingHostReceipts::new(
            2293,
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
        ))
        .unwrap();

    state.run_frame_internal(0, crate::RUN_MAIN);

    assert_eq!(state.game_state.frame.main_module, 7);
    assert_eq!(state.game_state.frame.submodule, 15);
    assert_eq!(state.game_state.frame.subsubmodule, 1);
    assert_eq!(state.game_state.frame.frame_counter, 125);
    assert!(state.game_state.display.nmi_update_is_latched());
    assert!(!state.original_timing_nmi_publication_pending);
    assert_eq!(state.original_timing_pending_nmi_update_gate, None);
    assert!(state.dungeon_landing_cpu_advance_pending.is_none());
    assert_eq!(state.last_consumed_original_timing_host_call(), Some(2293));
}

#[test]
fn startup_dispatcher_runs_pre_dungeon_audio_at_the_measured_boundary() {
    let asset_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("zelda3_assets.dat");
    let assets = std::fs::read(&asset_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", asset_path.display()));
    let mut selected_game = ZeldaState::new();
    selected_game.assets = Some(AssetPack::parse(&assets).unwrap());
    selected_game.set_rom_startup_timing(true);
    selected_game.rom_reset_frame_delay = 0;
    selected_game.initialized = true;
    selected_game.set_animated_tile_data_source_address(1);
    selected_game
        .game_execution_scheduler
        .schedule_selected_game_load(SelectedGameLoadDestination::Dungeon);

    for _ in 0..SELECTED_GAME_LOAD_BEFORE_PRE_DUNGEON_AUDIO_NMI_SLICES - 1 {
        selected_game.run_frame_internal(0, crate::RUN_MAIN);
    }
    assert_ne!(
        selected_game
            .game_state
            .system_signals
            .ambient_sound_effect(),
        5
    );
    selected_game.run_frame_internal(0, crate::RUN_MAIN);
    assert_eq!(
        selected_game
            .game_state
            .system_signals
            .ambient_sound_effect(),
        5
    );
    assert_eq!(
        selected_game
            .game_execution_scheduler
            .selected_game_load_remaining_nmi_slices(),
        SELECTED_GAME_LOAD_AFTER_PRE_DUNGEON_AUDIO_NMI_SLICES
    );
    assert!(selected_game.display_snapshot.is_some());
}

#[test]
fn dungeon_faded_filter_second_pass_resumes_without_a_new_main_iteration() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(2);
    state.set_countdown_word(0);
    state.dungeon_torch_mut().set_lights_out_request(1);
    state.ram[MAIN_PALETTE_BUFFER..MAIN_PALETTE_BUFFER + 4]
        .copy_from_slice(&[0x11, 0x11, 0x22, 0x22]);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInSubmodule,
        resumed_phase: Some(ModuleCpuPhase::CompleteBeforeNmi),
        submodule_nmi_slices: 1,
        subsubmodule: 2,
        palette_countdown: 1,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });

    state.Module07_02_FadedFilter();

    assert_eq!(state.game_state.display.palette_filter.countdown(), 1);
    assert_eq!(
        state
            .game_execution_scheduler
            .pre_main_caller_continuation(),
        Some(
            PreMainCallerContinuation::DungeonFadedFilterSecondPalettePass {
                resumed_phase: ModuleCpuPhase::CompleteBeforeNmi,
            }
        )
    );
    assert!(state
        .game_execution_scheduler
        .work_suspends_translated_call_stack());
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(ObjScanoutGenerations {
            oam: OamScanoutSource::RetainCapturedBeforeNmi,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        })
    );
    assert_eq!(
        state
            .next_display_cgram_override
            .as_deref()
            .map(|cgram| &cgram[..2]),
        Some(&[0x1111, 0x2222][..])
    );

    state.finish_pre_main_caller_continuation(
        PreMainCallerContinuation::DungeonFadedFilterSecondPalettePass {
            resumed_phase: ModuleCpuPhase::CompleteBeforeNmi,
        },
    );
    state.complete_dungeon_faded_filter_second_palette_pass();

    assert_eq!(state.game_state.display.palette_filter.countdown(), 2);
    assert!(state.game_execution_scheduler.is_idle());
}

#[test]
fn resumed_dungeon_fade_runs_the_next_iteration_after_a_leading_nmi() {
    for (room, entry_countdown) in [(0x21, 0), (0x21, 2), (0x21, 16), (0x21, 24), (0x22, 0)] {
        let mut state = ZeldaState::new();
        state.rom_startup_timing = true;
        state.set_dungeon_room_index(room);
        state.set_main_module(7);
        state.set_submodule(2);
        state.set_subsubmodule(2);
        state.set_countdown_word(entry_countdown);
        state.set_mosaic_target_level(31);
        state.dungeon_torch_mut().set_lights_out_request(1);
        state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
            phase: ModuleCpuPhase::InterruptedInSubmodule,
            resumed_phase: Some(ModuleCpuPhase::CompleteBeforeNmi),
            submodule_nmi_slices: 1,
            subsubmodule: 2,
            palette_countdown: entry_countdown as u8 + 1,
            sprite_main_boundary: None,
            cached_sprite_interruption: None,
        });

        state.Module07_02_FadedFilter();

        assert_eq!(
            state
                .game_execution_scheduler
                .pre_main_caller_continuation(),
            Some(
                PreMainCallerContinuation::DungeonFadedFilterSecondPalettePass {
                    resumed_phase: ModuleCpuPhase::CompleteBeforeNmi,
                }
            )
        );
        assert!(state.resume_pre_main_caller_continuation(0, None));

        assert_eq!(state.game_state.frame.subsubmodule, 2);
        assert_eq!(
            u16::from(state.game_state.display.palette_filter.countdown()),
            entry_countdown + 2
        );
        assert_eq!(
            state.game_execution_scheduler.pre_main_nmi_resume(),
            Some(PreMainNmiResume::DungeonSupertileNextIterationAfterLeadingNmi),
            "room={room:#04x} entry_countdown={entry_countdown}",
        );
    }
}

#[test]
fn nmi_inside_sprite_preparation_resumes_without_replaying_dungeon_main() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_dungeon_room_index(0x21);
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(2);
    state.set_countdown_word(16);
    state.set_mosaic_target_level(31);
    state.dungeon_torch_mut().set_lights_out_request(1);
    state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
        phase: ModuleCpuPhase::InterruptedInSubmodule,
        resumed_phase: Some(ModuleCpuPhase::InterruptedInNmiPrepareSprites),
        submodule_nmi_slices: 1,
        subsubmodule: 2,
        palette_countdown: 17,
        sprite_main_boundary: None,
        cached_sprite_interruption: None,
    });
    let entry_frame_counter = state.game_state.frame.frame_counter;

    state.Module07_02_FadedFilter();
    assert!(state.resume_pre_main_caller_continuation(0, None));

    assert_eq!(state.game_state.frame.frame_counter, entry_frame_counter);
    assert_eq!(state.game_state.display.palette_filter.countdown(), 18);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishNmiPrepareSpritesCallerReturn {
            caller: NmiPrepareSpritesCpuCaller::DungeonModule07,
        })
    );
    assert!(state
        .game_execution_scheduler
        .work_suspends_translated_call_stack());
}

#[test]
fn completed_dungeon_fade_waits_for_leading_nmi_before_state_three() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_main_module(7);
    state.set_submodule(2);
    state.set_subsubmodule(2);
    state.set_countdown_word(30);
    state.set_mosaic_target_level(31);
    state.dungeon_torch_mut().set_lights_out_request(1);

    state.Module07_02_FadedFilter();

    assert_eq!(state.game_state.frame.subsubmodule, 3);
    assert_eq!(state.game_state.display.palette_filter.countdown(), 0);
    assert_eq!(
        state.game_execution_scheduler.pre_main_nmi_resume(),
        Some(PreMainNmiResume::DungeonModuleCallerCompletedBeforeNextNmi)
    );
    assert!(!state
        .game_execution_scheduler
        .work_suspends_translated_call_stack());
    assert_eq!(
        PreMainNmiResume::DungeonModuleCallerCompletedBeforeNextNmi.nmi_latch_clear_phase(),
        Some(NmiPhase::AfterNmi)
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

#[test]
fn peg_attribute_flip_checkpoint_publishes_exact_bank_prefix_then_resumes() {
    let mut base = ZeldaState::new();
    base.set_main_module(7);
    base.set_submodule(0x16);
    base.set_subsubmodule(0x0f);
    for index in [0x0592, 0x0594, 0x0596] {
        let mut attributes = base.dungeon_bg2_attributes_mut();
        attributes.set_bg2_attr(index, 0x66);
        attributes.set_bg2_attr(0x0800 + index, 0x66);
        attributes.set_bg1_attr_word(index, 0x6666);
        attributes.set_bg1_attr_word(0x0800 + index, 0x6666);
    }

    let mut atomic = base.clone();
    atomic.Module07_16_UpdatePegs();

    let progress = crate::DungeonPegAttributeFlipProgressReceipt {
        index: 0x0594,
        completed_banks: 2,
        boundary: OriginalTimingBoundary::HostReturn,
    };
    let mut resumed = base;
    resumed.original_timing_owner = OriginalTimingOwnerState::Live;
    resumed.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        709_307,
        0,
        vec![OriginalTimingSemanticReceipt::DungeonPegAttributeFlipProgress(progress)],
    ));
    resumed.Module07_16_UpdatePegs();

    assert_eq!(resumed.game_state.frame.submodule, 0x16);
    assert_eq!(resumed.game_state.frame.subsubmodule, 0x10);
    assert_eq!(
        resumed.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn),
    );
    let attributes = &resumed.game_state.dungeon.bg2_attributes;
    for (index, expected) in [
        (0x0596, [0x67; 4]),
        (0x0594, [0x67, 0x67, 0x66, 0x66]),
        (0x0592, [0x66; 4]),
    ] {
        assert_eq!(
            [
                attributes.bg2_attr(index),
                attributes.bg2_attr(0x0800 + index),
                attributes.bg1_attr(index),
                attributes.bg1_attr(0x0800 + index),
            ],
            expected,
            "wrong source-visible bank prefix at index ${index:04x}",
        );
    }

    let pending = resumed
        .dungeon_peg_attribute_flip_pending
        .take()
        .expect("the exact source cursor must remain bound to the caller continuation");
    assert_eq!(pending.caller, DungeonPegAttributeFlipCaller::UpdatePegs);
    let next = crate::DungeonPegAttributeFlipProgressReceipt {
        index: 0x0592,
        completed_banks: 1,
        boundary: OriginalTimingBoundary::NmiAccepted,
    };
    resumed.advance_dungeon_peg_attribute_flip_between_progress(pending.progress, next);
    let attributes = &resumed.game_state.dungeon.bg2_attributes;
    assert_eq!(
        [
            attributes.bg2_attr(0x0594),
            attributes.bg2_attr(0x0d94),
            attributes.bg1_attr(0x0594),
            attributes.bg1_attr(0x0d94),
        ],
        [0x67; 4],
    );
    assert_eq!(
        [
            attributes.bg2_attr(0x0592),
            attributes.bg2_attr(0x0d92),
            attributes.bg1_attr(0x0592),
            attributes.bg1_attr(0x0d92),
        ],
        [0x67, 0x66, 0x66, 0x66],
    );
    resumed.complete_dungeon_peg_attribute_flip_after_progress(next);

    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(
        resumed.game_state.dungeon.bg2_attributes,
        atomic.game_state.dungeon.bg2_attributes,
    );
    assert_eq!(resumed.game_state.frame.submodule, 0);
    assert_eq!(resumed.game_state.frame.subsubmodule, 0);
}

#[test]
fn selectable_peg_attribute_flip_resumes_state_8_only_after_source_return() {
    let mut base = ZeldaState::new();
    base.set_main_module(7);
    base.set_submodule(2);
    base.set_subsubmodule(8);
    base.set_overworld_map_state(4);
    base.dungeon_environment_mut()
        .toggle_orange_blue_barrier_state();
    for index in [0x02ce, 0x02cf, 0x02d0] {
        let mut attributes = base.dungeon_bg2_attributes_mut();
        attributes.set_bg2_attr(index, 0x66);
        attributes.set_bg2_attr(0x0800 + index, 0x66);
        attributes.set_bg1_attr_word(index, 0x6666);
        attributes.set_bg1_attr_word(0x0800 + index, 0x6666);
    }

    let mut atomic = base.clone();
    atomic.Module07_02_SupertileTransition();
    atomic.complete_module07_dungeon_after_submodule();

    let progress = crate::DungeonPegAttributeFlipProgressReceipt {
        index: 0x02cf,
        completed_banks: 2,
        boundary: OriginalTimingBoundary::NmiAccepted,
    };
    let mut resumed = base;
    resumed.original_timing_owner = OriginalTimingOwnerState::Live;
    resumed.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        711_897,
        0,
        vec![OriginalTimingSemanticReceipt::DungeonPegAttributeFlipProgress(progress)],
    ));
    let link_before = resumed.game_state.player.follower_link.x();
    let bg2_before = resumed.game_state.display.ppu_scroll_copy.bg2_h_copy2();
    resumed.Module07_02_SupertileTransition();

    assert_eq!(resumed.overworld_map_state(), 5);
    assert_eq!(resumed.game_state.player.follower_link.x(), link_before);
    assert_eq!(
        resumed.game_state.display.ppu_scroll_copy.bg2_h_copy2(),
        bg2_before,
    );
    assert_eq!(
        resumed.dungeon_peg_attribute_flip_pending,
        Some(DungeonPegAttributeFlipContinuation {
            caller: DungeonPegAttributeFlipCaller::SupertileTransition {
                state_12_cpu_advance: None,
                quadrant_cpu_advance: None,
            },
            progress,
        }),
    );
    assert_eq!(
        resumed
            .game_execution_scheduler
            .take_after_current_trailing_nmi(),
        Some(GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn),
    );

    resumed.complete_dungeon_after_submodule_cpu_caller_return();
    assert_eq!(resumed.ram, atomic.ram);
    assert_eq!(
        resumed.game_state.dungeon.bg2_attributes,
        atomic.game_state.dungeon.bg2_attributes,
    );
    assert!(resumed.dungeon_peg_attribute_flip_pending.is_none());
}

#[test]
fn room_41_landing_tail_uses_leading_nmi_order_and_post_interrupt_latch_release() {
    let state_15 = crate::game_state::FrameState {
        main_module: 7,
        submodule: 2,
        subsubmodule: 15,
        ..crate::game_state::FrameState::default()
    };
    let shutter_tail = crate::game_state::FrameState {
        submodule: 5,
        subsubmodule: 0,
        ..state_15
    };

    assert!(rom_dungeon_module_iteration_runs_after_leading_nmi(
        state_15, 0x41
    ));
    assert!(rom_dungeon_module_iteration_runs_after_leading_nmi(
        shutter_tail,
        0x41
    ));
    assert!(!rom_dungeon_module_iteration_runs_after_leading_nmi(
        state_15, 0x42
    ));
    let player_control = crate::game_state::FrameState {
        submodule: 0,
        subsubmodule: 0,
        ..state_15
    };
    assert!(post_landing_player_control_runs_after_leading_nmi(
        Some(0x41),
        player_control,
        0x41,
    ));
    assert!(!post_landing_player_control_runs_after_leading_nmi(
        Some(0x41),
        player_control,
        0x42,
    ));
    assert!(
        faded_filter_palette_completion_clears_nmi_latch_after_interrupt(Some(27248), 27248, true)
    );
    assert!(
        !faded_filter_palette_completion_clears_nmi_latch_after_interrupt(
            Some(27249),
            27249,
            false
        )
    );
}

#[test]
fn only_interrupted_dungeon_fade_selects_the_host_boundary_oam_shadow() {
    let dungeon_fade = Some(
        PreMainCallerContinuation::DungeonFadedFilterSecondPalettePass {
            resumed_phase: ModuleCpuPhase::CompleteBeforeNmi,
        },
    );
    let spiral_fade = Some(PreMainCallerContinuation::SpiralStairsSecondPaletteFilter);

    assert!(pre_main_caller_uses_host_boundary_shadow_oam(
        dungeon_fade,
        1
    ));
    assert!(!pre_main_caller_uses_host_boundary_shadow_oam(
        dungeon_fade,
        3
    ));
    assert!(!pre_main_caller_uses_host_boundary_shadow_oam(
        spiral_fade,
        1
    ));
    assert!(!pre_main_caller_uses_host_boundary_shadow_oam(None, 1));
    assert_eq!(
        dungeon_faded_filter_first_pass_oam_scanout(),
        OamScanoutSource::RetainCapturedBeforeNmi
    );

    for subsubmodule in [2, 14] {
        assert!(interrupted_dungeon_faded_filter_uses_host_link_obj_cache(
            FrameState {
                main_module: 7,
                submodule: 2,
                subsubmodule,
                ..FrameState::default()
            },
            dungeon_fade,
        ));
    }
    assert!(!interrupted_dungeon_faded_filter_uses_host_link_obj_cache(
        FrameState {
            main_module: 7,
            submodule: 2,
            subsubmodule: 13,
            ..FrameState::default()
        },
        dungeon_fade,
    ));
}

#[test]
fn spiral_stair_double_palette_pass_resumes_without_a_new_main_iteration() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_subsubmodule(2);
    state.set_countdown_word(0);
    state
        .dungeon_stair_movement_mut()
        .set_staircase_move_counter(8);

    state.Module07_0E_02_ApplyFilterIf();

    assert_eq!(state.game_state.display.palette_filter.countdown(), 1);
    assert_eq!(
        state
            .game_execution_scheduler
            .pre_main_caller_continuation(),
        Some(PreMainCallerContinuation::SpiralStairsSecondPaletteFilter)
    );
    assert!(state
        .game_execution_scheduler
        .work_suspends_translated_call_stack());
    assert_eq!(
        state
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter(),
        8
    );

    state.finish_pre_main_caller_continuation(
        PreMainCallerContinuation::SpiralStairsSecondPaletteFilter,
    );
    state.complete_spiral_stairs_second_palette_filter();

    assert_eq!(state.game_state.display.palette_filter.countdown(), 2);
    assert_eq!(
        state
            .game_state
            .dungeon
            .stair_movement
            .staircase_move_counter(),
        7
    );
}

#[test]
fn spiral_stair_grayscale_pass_resumes_on_its_second_palette_walk() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_subsubmodule(15);
    state.set_countdown_word(0);

    state.Dungeon_DoubleApplyAndIncrementGrayscale();

    assert_eq!(state.game_state.display.palette_filter.countdown(), 1);
    assert_eq!(
        state
            .game_execution_scheduler
            .pre_main_caller_continuation(),
        Some(PreMainCallerContinuation::SpiralStairsSecondGrayscalePaletteFilter)
    );
    assert!(state
        .game_execution_scheduler
        .work_suspends_translated_call_stack());

    state.finish_pre_main_caller_continuation(
        PreMainCallerContinuation::SpiralStairsSecondGrayscalePaletteFilter,
    );
    state.complete_spiral_stairs_second_grayscale_palette_filter();

    assert_eq!(state.game_state.display.palette_filter.countdown(), 2);
}

#[test]
fn spiral_stair_second_palette_return_defers_only_animated_bg_operands() {
    let frame = crate::game_state::FrameState {
        main_module: 7,
        submodule: 0x0e,
        ..Default::default()
    };

    assert!(
        !rom_spiral_stairs_second_palette_return_uses_host_animated_bg_operands(
            frame, true, 1, 0xaa80,
        )
    );
    assert!(
        rom_spiral_stairs_second_palette_return_uses_host_animated_bg_operands(
            frame, true, 1, 0xae80,
        )
    );
}

#[test]
fn straight_interroom_grayscale_pass_uses_the_shared_second_palette_return() {
    let mut state = ZeldaState::new();
    state.rom_startup_timing = true;
    state.set_main_module(7);
    state.set_submodule(0x12);
    state.set_subsubmodule(14);
    state.set_countdown_word(0);

    state.Dungeon_DoubleApplyAndIncrementGrayscale();

    assert_eq!(state.game_state.display.palette_filter.countdown(), 1);
    assert_eq!(
        state
            .game_execution_scheduler
            .pre_main_caller_continuation(),
        Some(PreMainCallerContinuation::SpiralStairsSecondGrayscalePaletteFilter)
    );

    state.finish_pre_main_caller_continuation(
        PreMainCallerContinuation::SpiralStairsSecondGrayscalePaletteFilter,
    );
    state.complete_spiral_stairs_second_grayscale_palette_filter();

    assert_eq!(state.game_state.display.palette_filter.countdown(), 2);
}

#[test]
fn spiral_stair_grayscale_return_releases_core_dma_after_retaining_its_scanout() {
    let mut state = ZeldaState::new();
    state.set_core_update_disable_flag(1);

    let animated_bg_operands = state.stage_spiral_stairs_second_grayscale_nmi();

    assert!(!state.game_state.display.core_updates_are_disabled());
    assert_eq!(
        animated_bg_operands,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(
        state.next_display_animated_bg_scanout_generation,
        Some(AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi)
    );
}

#[test]
fn grayscale_caller_finishes_held_nmi_before_authoring_the_next_palette() {
    // Original host 23945 completes a held handler, runs the second palette
    // walk and suffix, and only accepts the following Open NMI at return.
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_animated_tile_data_source_address(0xa680);
    state.set_indoor_flag(1);
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_subsubmodule(15);
    state.set_frame_counter(0xb2);
    state.set_countdown_word(1);
    state.ppu.cgram.fill(0x1234);
    state.latch_nmi_update();
    state.schedule_pre_main_caller_continuation(
        PreMainCallerContinuation::SpiralStairsSecondGrayscalePaletteFilter,
    );
    assert!(state.resume_pre_main_caller_continuation(0, None));
    assert_eq!(state.game_state.frame.frame_counter, 0xb2);
    assert_eq!(state.game_state.display.palette_filter.countdown(), 2);
    assert_eq!(state.ppu.cgram[0x20], 0x1234);
    assert!(state.game_state.system_signals.should_update_cgram());
    assert!(!state.game_state.display.nmi_update_is_latched());
    state.game_execution_scheduler.begin_host_frame();
    assert!(state.game_execution_scheduler.main_return_requires_leading_nmi());
    state.interrupt_nmi(0, None, false);
    assert_eq!(state.ppu.cgram[0x20], read_le_u16(&state.ram, MAIN_PALETTE_BUFFER + 0x40));
    assert!(!state.game_state.system_signals.should_update_cgram());
}

#[test]
fn spiral_stair_grayscale_return_publishes_an_advanced_animated_bg_batch() {
    let mut state = ZeldaState::new();
    state.set_core_update_disable_flag(1);
    state.set_bg_tile_animation_countdown(1);

    let animated_bg_operands = state.stage_spiral_stairs_second_grayscale_nmi();

    assert!(!state.game_state.display.core_updates_are_disabled());
    assert_eq!(animated_bg_operands, GraphicsDmaGeneration::LiveAfterMain);
    assert_eq!(
        state.next_display_animated_bg_scanout_generation,
        Some(AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi)
    );
}

#[test]
fn dungeon_landing_wipe_uses_typed_snapshot_generation_without_menu_retention() {
    // Pre-dungeon staging now advances through its typed snapshot generation;
    // it does not use the menu-stripe memory-retention rule.
    assert!(!rom_display_memory_publication_is_deferred(7, 15, 0, false));
    assert!(!rom_display_memory_publication_is_deferred(7, 14, 0, false));
    assert_eq!(
        rom_display_snapshot_publication(7, 15),
        DisplaySnapshotPublication::AdvanceStaged
    );
    assert_eq!(
        rom_display_snapshot_publication(16, 1),
        DisplaySnapshotPublication::AdvanceStaged
    );
    assert_eq!(
        rom_display_snapshot_publication(16, 0),
        DisplaySnapshotPublication::PublishCaptured
    );

    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(15);
    state.ppu.vram[0] = 0x1111;
    state.capture_display_snapshot();
    state.ppu.vram[0] = 0x2222;
    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[0]),
        0x2222
    );

    state.capture_display_snapshot();
    state.ppu.vram[0] = 0x3333;
    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[0]),
        0x3333
    );

    state.capture_display_snapshot();
    state.ppu.vram[0] = 0x4444;
    assert_eq!(
        state.with_display_snapshot(|display| display.ppu.vram[0]),
        0x4444
    );
}

#[test]
fn dungeon_falling_entry_retains_the_pre_transition_obj_generation() {
    let mut state = ZeldaState::new();
    state.set_main_module(9);
    state.set_submodule(0);
    state.ppu.oam[204] = 0x6c8a;
    state.ppu.vram[0x4000] = 0x1234;
    state.capture_display_snapshot();

    // The ROM module switch reaches WRAM before the native frame projection is
    // synchronized by NMI. Exercise that real publication-boundary ownership.
    state.ram[crate::game_state::constants::MAIN_MODULE] = 0x11;
    state.ram[crate::game_state::constants::SUBMODULE] = 0;
    assert_eq!(state.game_state.frame.main_module, 9);
    state.ppu.oam[204] = 0xf08a;
    state.ppu.vram[0x4000] = 0x5678;
    state.capture_display_snapshot();

    assert!(rom_dungeon_falling_entry_retains_published_obj_generation(
        9, 0, 0x11, 0,
    ));
    assert!(!rom_dungeon_falling_entry_retains_published_obj_generation(
        9, 1, 0x11, 0,
    ));
    assert_eq!(
        state.with_display_snapshot(|display| (display.ppu.oam[204], display.ppu.vram[0x4000])),
        (0x6c8a, 0x1234),
    );
    assert_eq!(state.ppu.oam[204], 0xf08a);
    assert_eq!(state.ppu.vram[0x4000], 0x5678);
}

#[test]
fn native_straight_quadrant_returns_wait_for_the_next_open_nmi() {
    for step in 5..=8 {
        let mut state = ZeldaState::new();
        state.restore_live_rom_timing_after_checkpoint();
        state.set_animated_tile_data_source_address(0xa680);
        state.set_indoor_flag(1);
        state.set_main_module(7);
        state.set_submodule(0x12);
        state.set_subsubmodule(step);
        state.set_frame_counter(0xa7);
        state.latch_nmi_update();
        state.set_core_update_disable_flag(1);
        state.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishSpiralStaircasePaletteFilter {
                tail: if step & 1 != 0 {
                    SpiralStaircasePaletteTail::BuildQuadrantForVram
                } else {
                    SpiralStaircasePaletteTail::PrepareNextQuadrant
                },
                caller: InterruptedPaletteFilterCaller::StraightInterroomStairs,
            }, 1,
        );
        state.run_frame_internal(0, crate::RUN_MAIN);
        assert_eq!(state.game_state.frame.subsubmodule, step + 1);
        assert_eq!(state.game_state.frame.frame_counter, 0xa7);
        assert!(!state.game_state.display.nmi_update_is_latched());
        state.game_execution_scheduler.begin_host_frame();
        assert!(state.game_execution_scheduler.main_return_requires_leading_nmi());
    }
}

#[test]
fn suspended_spiral_palette_filter_holds_core_nmi_updates() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_main_module(7);
    state.set_submodule(0x0e);

    assert!(state
        .suspend_spiral_staircase_palette_filter(SpiralStaircasePaletteTail::PrepareNextQuadrant,));
    assert!(state.game_state.display.core_updates_are_disabled());
}

#[test]
fn completed_spiral_palette_caller_carries_the_following_source_nmi_at_early_return() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.set_main_module(7);
    state.set_submodule(0x0e);
    state.set_subsubmodule(11);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_nmi_publication_pending = true;
    state.capture_display_snapshot();
    state.set_core_update_disable_flag(1);
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishSpiralStaircasePaletteFilter {
            tail: SpiralStaircasePaletteTail::BuildQuadrantForVram,
            caller: InterruptedPaletteFilterCaller::SpiralStairs,
        },
        2,
    );
    assert!(matches!(
        state.game_execution_scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Waiting),
    ));

    // The handler which interrupted the palette/quadrant call publishes
    // first. The resumed C stack then completes the quadrant, returns through
    // Module 7 and accepts the following NMI at the host boundary.
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        13_638,
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

    assert!(state.original_timing_nmi_publication_pending);
    assert!(!matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpiralStaircasePaletteFilter { .. }),
    ));
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn retained_display_memory_still_publishes_live_dungeon_obj_chr() {
    let mut state = ZeldaState::new();
    state.set_main_module(7);
    state.set_submodule(0);
    state.ppu.vram[0] = 0x1111;
    state.ppu.vram[0x4000] = 0x2222;
    state.ppu.vram[0x4050] = 0x3333;
    state.ppu.vram[0x4100] = 0x4444;
    state.ppu.vram[0x4150] = 0x5555;
    state.capture_display_snapshot();
    state.display_snapshot.as_mut().unwrap().vram_generation =
        DisplayVramGeneration::RetainCapturedBeforeNmi;

    state.ppu.vram[0] = 0xaaaa;
    state.ppu.vram[0x4000] = 0xbbbb;
    state.ppu.vram[0x4050] = 0xcccc;
    state.ppu.vram[0x4100] = 0xdddd;
    state.ppu.vram[0x4150] = 0xeeee;

    assert_eq!(
        state.with_display_snapshot(|display| {
            (
                display.ppu.vram[0],
                display.ppu.vram[0x4000],
                display.ppu.vram[0x4050],
                display.ppu.vram[0x4100],
                display.ppu.vram[0x4150],
            )
        }),
        (0x1111, 0xbbbb, 0x3333, 0xdddd, 0x5555),
    );
}

#[test]
fn dungeon_exit_nmi_publishes_coherent_oam_link_tiles_and_scroll() {
    let mut state = ZeldaState::new();
    state.set_main_module(0x0f);
    state.set_submodule(0);
    state.next_display_obj_scanout_generation = Some(ObjScanoutGenerations::coherent(
        GraphicsDmaGeneration::HostBoundaryBeforeMain,
    ));
    state.ppu.vram[0x4000] = 0x1111;
    state.ppu.oam[0] = 0x2222;
    state.ppu.bg_layer[1].v_scroll = 0x3333;
    state.capture_display_snapshot();

    state.set_submodule(1);
    state.ppu.vram[0x4000] = 0xaaaa;
    state.ppu.oam[0] = 0xbbbb;
    state.ppu.bg_layer[1].v_scroll = 0xcccc;

    let captured = state.with_display_snapshot(|display| {
        (
            display.ppu.vram[0x4000],
            display.ppu.oam[0],
            display.ppu.bg_layer[1].v_scroll,
        )
    });

    assert_eq!(captured, (0xaaaa, 0xbbbb, 0xcccc));
}

#[test]
fn dungeon_exit_prep_oam_scanout_uses_the_published_shadow_generation() {
    let mut entry = crate::game_state::FrameState::default();
    entry.main_module = 7;
    let mut exit = entry;
    exit.main_module = 0x0f;
    let entry_scanout = rom_graphics_dma_plan(entry.main_module, entry.submodule).oam_scanout;
    let transition_scanout = oam_scanout_across_main(entry, exit, entry_scanout);

    assert_eq!(
        rom_graphics_dma_plan(exit.main_module, exit.submodule).oam_operands,
        GraphicsDmaGeneration::LiveAfterMain,
    );
    assert_eq!(
        transition_scanout,
        OamScanoutSource::ComposePublishedShadowDma,
    );
    assert_eq!(
        transition_scanout.resolve_live_override(false),
        OamScanoutSource::ComposePublishedShadowDma,
    );
    assert_eq!(
        transition_scanout.resolve_live_override(true),
        OamScanoutSource::ComposeLiveAfterNmi,
    );
}

#[test]
fn dungeon_transition_handoffs_use_the_published_oam_shadow() {
    let mut entry = crate::game_state::FrameState::default();
    entry.main_module = 7;
    let entry_scanout = rom_graphics_dma_plan(entry.main_module, entry.submodule).oam_scanout;

    for submodule in [1, 4, 0x11, 0x12, 0x13] {
        let mut exit = entry;
        exit.submodule = submodule;
        assert_eq!(
            oam_scanout_across_main(entry, exit, entry_scanout),
            OamScanoutSource::ComposePublishedShadowDma,
        );
        let expected_link = if submodule == 0x12 {
            // Straight inter-room stairs perform their first Link upload after
            // the leading NMI, before active OBJ evaluation.
            GraphicsDmaGeneration::LiveAfterMain
        } else {
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        };
        assert_eq!(
            link_obj_scanout_across_main(entry, exit, GraphicsDmaGeneration::LiveAfterMain),
            expected_link,
        );
    }

    let mut subtile_entry = entry;
    subtile_entry.submodule = 1;
    let mut subtile_exit = subtile_entry;
    subtile_exit.subsubmodule = 1;
    assert_eq!(
        oam_scanout_across_main(subtile_entry, subtile_exit, entry_scanout),
        OamScanoutSource::ComposePublishedShadowDma,
    );
}

#[test]
fn straight_interroom_stairs_keep_the_host_boundary_display_generation() {
    let plan = rom_graphics_dma_plan(7, 0x12);
    assert_eq!(plan.oam_scanout, OamScanoutSource::RetainCapturedBeforeNmi);
    assert_eq!(
        plan.link_obj_operands,
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
    assert_eq!(plan.link_obj_scanout, GraphicsDmaGeneration::LiveAfterMain);

    for submodule in [0x11, 0x13] {
        let neighbor = rom_graphics_dma_plan(7, submodule);
        assert_eq!(neighbor.oam_scanout, OamScanoutSource::ComposeLiveAfterNmi);
        assert_eq!(
            neighbor.link_obj_operands,
            GraphicsDmaGeneration::LiveAfterMain
        );
    }
}

#[test]
fn spiral_stairs_first_steady_slice_consumes_the_host_boundary_link_operands() {
    // Measured at route frame 28837 (host 28838, entry $0e/$00 -> exit $0e/$01):
    // that slice advances Link's body-pointer source words $0af0/$0af2 by 0x40
    // while Snes9x's OBJ CHR at VRAM $4220/$4320 still holds the pre-advance
    // data, so the NMI must consume the host-boundary operands. The phase begins
    // at subsubmodule $01, not $02.
    let mut exit = crate::game_state::FrameState::default();
    exit.main_module = 7;
    exit.submodule = 0x0e;
    let mut entry = exit;
    let live = rom_graphics_dma_plan(7, 0x0e).link_obj_operands;
    assert_eq!(live, GraphicsDmaGeneration::LiveAfterMain);

    for subsubmodule in 1..=3 {
        entry.subsubmodule = subsubmodule - 1;
        exit.subsubmodule = subsubmodule;
        assert_eq!(
            link_obj_operands_across_main(entry, exit, live),
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
            "spiral-stair subsubmodule {subsubmodule:#x} must use host-boundary operands",
        );
    }

    // Subsubmodule $00 is still authored before its own NMI, so an already
    // resident $0e/$00 slice keeps the live operands; only the module entry
    // from submodule $00 overrides it.
    entry.subsubmodule = 0;
    exit.subsubmodule = 0;
    assert_eq!(
        link_obj_operands_across_main(entry, exit, live),
        GraphicsDmaGeneration::LiveAfterMain
    );
    let mut module_entry = entry;
    module_entry.submodule = 0;
    assert_eq!(
        link_obj_operands_across_main(module_entry, exit, live),
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    );
}

#[test]
fn supertile_scroll_keeps_the_pre_main_link_generation_from_palette_entry_through_scroll() {
    let live = GraphicsDmaGeneration::LiveAfterMain;
    let state_7 = crate::game_state::FrameState {
        main_module: 7,
        submodule: 2,
        subsubmodule: 7,
        ..Default::default()
    };
    let state_8 = crate::game_state::FrameState {
        subsubmodule: 8,
        ..state_7
    };

    for entry in [state_7, state_8] {
        assert!(dungeon_supertile_scroll_nmi_precedes_link_animation(
            entry, state_8,
        ));
        assert_eq!(
            link_obj_scanout_across_main(entry, state_8, live),
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
        );
        assert_eq!(
            link_obj_operands_across_main(entry, state_8, live),
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
        );
    }

    let state_6 = crate::game_state::FrameState {
        subsubmodule: 6,
        ..state_7
    };
    let state_9 = crate::game_state::FrameState {
        subsubmodule: 9,
        ..state_8
    };
    assert!(!dungeon_supertile_scroll_nmi_precedes_link_animation(
        state_6, state_7,
    ));
    assert!(!dungeon_supertile_scroll_nmi_precedes_link_animation(
        state_8, state_9,
    ));
}

#[test]
fn straight_interroom_legacy_fadeout_schedule_is_receipt_scoped() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(7);
    state.set_submodule(0x12);
    state.set_subsubmodule(1);
    state.set_dungeon_room_index(0x51);
    state.dungeon_stair_movement_mut().set_staircase_index(0x30);

    state.suspend_straight_interroom_fadeout_suffix_if_crosses_nmi(1);
    assert_eq!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishStraightInterroomFadeoutSuffix)
    );
    assert!(
        state
            .game_execution_scheduler
            .work_suspends_translated_call_stack(),
        "the held frame must resume Module 7's caller suffix rather than run a fresh iteration",
    );
    assert_eq!(
        straight_interroom_fadeout_return_obj_scanout(),
        ObjScanoutGenerations {
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        },
        "the held return must keep the resident OAM and decoded Link OBJ generation",
    );
    assert_eq!(
        straight_interroom_fadeout_obj_scanout(),
        ObjScanoutGenerations {
            oam: OamScanoutSource::RetainCapturedBeforeNmi,
            link_obj: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        },
        "the filter scanout keeps live raw OBJ bytes tied to the host-boundary source identity",
    );
    assert_eq!(
        straight_interroom_fadeout_following_obj_scanout(),
        ObjScanoutGenerations {
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
        },
        "the following scanout retains OAM while publishing the independently completed Link DMA",
    );
    state.next_display_obj_scanout_generation =
        Some(straight_interroom_fadeout_following_obj_scanout());
    state.stage_straight_interroom_fadeout_obj_source();
    assert_eq!(
        state.next_display_obj_scanout_generation,
        Some(straight_interroom_fadeout_following_obj_scanout()),
        "ordinary fadeout staging must not overwrite a resumed-caller handoff",
    );

    state.game_execution_scheduler.finish_work();
    state.suspend_straight_interroom_fadeout_suffix_if_crosses_nmi(2);
    assert!(!state.game_execution_scheduler.work_is_pending());

    state.set_dungeon_room_index(0x52);
    state.suspend_straight_interroom_fadeout_suffix_if_crosses_nmi(1);
    assert!(!state.game_execution_scheduler.work_is_pending());
}

#[test]
fn native_straight_fadeout_uses_the_measured_caller_phase() {
    for room in [0x51, 0x22] {
        for phase in [ModuleCpuPhase::CompleteBeforeNmi, ModuleCpuPhase::InterruptedInNmiPrepareSprites] {
            let mut state = ZeldaState::new();
            state.restore_live_rom_timing_after_checkpoint();
            state.set_animated_tile_data_source_address(0xa680);
            state.set_indoor_flag(1);
            state.set_main_module(7);
            state.set_submodule(0x12);
            state.set_subsubmodule(1);
            state.set_dungeon_room_index(room);
            state.dungeon_stair_movement_mut().set_staircase_index(0x30);
            state.set_countdown_word(0);
            state.dungeon_landing_cpu_advance_pending = Some(DungeonModuleCpuAdvance {
                phase, resumed_phase: Some(ModuleCpuPhase::CompleteBeforeNmi),
                submodule_nmi_slices: 0, subsubmodule: 1, palette_countdown: 1,
                sprite_main_boundary: None, cached_sprite_interruption: None,
            });
            state.Module07_11_StraightInterroomStairs();
            assert!(state.dungeon_landing_cpu_advance_pending.is_none());
            assert!(state.game_execution_scheduler.is_idle(),
                "a caller interruption cannot park before Sprite_Main");
            assert_eq!(state.dungeon_nmi_prepare_sprites_return_pending,
                phase == ModuleCpuPhase::InterruptedInNmiPrepareSprites);
        }
    }
}

#[test]
fn straight_interroom_return_carries_the_following_source_nmi_acceptance() {
    let mut state = ZeldaState::new();
    state.restore_live_rom_timing_after_checkpoint();
    state.initialized = true;
    state.rom_reset_frame_delay = 0;
    state.set_animated_tile_data_source_address(1);
    state.set_main_module(7);
    state.set_submodule(0x12);
    state.set_subsubmodule(1);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_nmi_publication_pending = true;
    state.ppu.vram[0x5800..0x5c00].fill(0x1357);
    state.capture_display_snapshot();
    state.ppu.vram[0x5800..0x5c00].fill(0x2468);
    state.game_execution_scheduler.schedule_work(
        GameWorkContinuation::FinishStraightInterroomFadeoutSuffix,
        1,
    );
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        1,
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

    assert!(state.game_execution_scheduler.is_idle());
    assert!(!state.game_state.display.nmi_update_is_latched());
    assert_eq!(
        state.display_snapshot.as_ref().unwrap().ppu.vram[0x5800..0x5c00],
        [0x2468; 0x400],
        "the scheduled caller's trailing acceptance must replace the completed carry-in scanout",
    );
    assert!(state
        .display_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.accepts_nmi_dma_receipts));
    assert!(state.original_timing_nmi_publication_pending);
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn parked_dungeon_sprite_main_terminal_return_retires_the_pending_suffix_once() {
    // Route host 717302: a resumed live Module 7 Sprite_Main returns inside a
    // typed terminal main-loop return whose wire is [SpriteMainReturned,
    // CallStackContinued, MainLoopCommonSuffixCompleted, NmiAccepted(Open)].
    // The typed return owns the shared ZeldaRunGameLoop suffix; the parked
    // caller's completion must retire that one owner, not add a second
    // NMI_PrepareSprites (which advanced the Link animated-tile DMA cycle one
    // frame ahead of the oracle, visible at route frame 732911).
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.set_indoor_flag(1);
    state.set_main_module(0x07);
    state.set_submodule(2);
    write_le_u16(&mut state.ram, LINK_DMA_COUNTDOWN, 9);
    state.set_bg_tile_animation_countdown(7);
    state.pending_main_loop_common_suffix =
        Some(MainLoopCommonSuffixContinuation::PrepareSpritesAndClearNmiLatch);

    state.complete_parked_dungeon_sprite_main_suffix_by_wire(false);
    assert!(state.pending_main_loop_common_suffix.is_none());
    state.complete_pending_main_loop_common_suffix_after_module_return();

    assert_eq!(read_le_u16(&state.ram, LINK_DMA_COUNTDOWN), 8);
    assert_eq!(state.game_state.display.bg_tile_animation_countdown, 6);

    // Control: without a pending typed suffix the parked caller still runs
    // the ordinary one-shot suffix itself.
    let mut bare = ZeldaState::new();
    bare.set_rom_startup_timing(true);
    bare.set_indoor_flag(1);
    bare.set_main_module(0x07);
    bare.set_submodule(2);
    write_le_u16(&mut bare.ram, LINK_DMA_COUNTDOWN, 9);
    bare.set_bg_tile_animation_countdown(7);
    bare.complete_parked_dungeon_sprite_main_suffix_by_wire(false);
    assert_eq!(read_le_u16(&bare.ram, LINK_DMA_COUNTDOWN), 8);
    assert_eq!(bare.game_state.display.bg_tile_animation_countdown, 6);
}
