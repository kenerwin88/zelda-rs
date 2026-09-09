//! ZeldaState runtime tests — misc.
//! Split mechanically from the hub file; bodies unchanged.

use super::*;

#[test]
fn owns_oracle_compared_memory_regions() {
    let state = ZeldaState::new();
    assert_eq!(state.ram.len(), WRAM_SIZE);
    assert_eq!(state.sram.len(), SRAM_SIZE);
    assert_eq!(state.vram().len(), VRAM_WORDS);
}

#[test]
fn screen_layer_helpers_keep_world_transient_layer_copy_coherent() {
    let mut state = ZeldaState::new();
    state.ram[TM_COPY] = 0x15;
    state.ram[TS_COPY] = 0x00;
    state.sync_native_game_state_from_ram();

    state.set_main_screen_layers(0x16);
    state.set_sub_screen_layers(0x01);
    state.set_quadrant_fullsize_x(2);

    assert_eq!(state.game_state.display.main_screen_layers, 0x16);
    assert_eq!(state.game_state.display.sub_screen_layers, 0x01);
    assert_eq!(state.game_state.world.transient.tilemap_layer_copy, 0x0116);
    assert_eq!(state.ram[TM_COPY], 0x16);
    assert_eq!(state.ram[TS_COPY], 0x01);
    state.assert_native_display_state_matches_ram();
}

#[test]
fn sram_path_uses_xdg_data_home_for_deck_safe_default() {
    assert_eq!(
        ZeldaState::sram_path_from_env(None, Some("/tmp/xdg".into()), Some("/tmp/home".into())),
        PathBuf::from("/tmp/xdg/zelda3-rs/saves/sram.dat")
    );
}

#[test]
fn sram_path_falls_back_to_home_data_dir() {
    assert_eq!(
        ZeldaState::sram_path_from_env(None, None, Some("/tmp/home".into())),
        PathBuf::from("/tmp/home/.local/share/zelda3-rs/saves/sram.dat")
    );
}

#[test]
fn player_layer_collision_helpers_preserve_unrelated_flags() {
    let mut state = ZeldaState::new();
    state.set_player_layer_collision_flags(0xf0);

    state.set_player_layer_collision(
        crate::game_state::constants::player::LAYER_COLLISION_BG1,
        true,
    );
    assert_eq!(
        state.ram[crate::game_state::constants::PLAYER_LAYER_COLLISION_FLAGS],
        0xf1
    );
    assert!(!state
        .has_player_layer_collision(crate::game_state::constants::player::LAYER_COLLISION_BOTH));

    state.set_player_layer_collision(
        crate::game_state::constants::player::LAYER_COLLISION_BG2,
        true,
    );
    assert_eq!(
        state.ram[crate::game_state::constants::PLAYER_LAYER_COLLISION_FLAGS],
        0xf3
    );
    assert!(state
        .has_player_layer_collision(crate::game_state::constants::player::LAYER_COLLISION_BOTH));

    state.set_player_layer_collision(
        crate::game_state::constants::player::LAYER_COLLISION_BG1,
        false,
    );
    assert_eq!(
        state.ram[crate::game_state::constants::PLAYER_LAYER_COLLISION_FLAGS],
        0xf2
    );
    assert!(!state
        .has_player_layer_collision(crate::game_state::constants::player::LAYER_COLLISION_BOTH));
}

#[test]
fn credits_scene_fade_advances_scratch_when_fade_not_complete() {
    let mut state = ZeldaState::new();
    state.set_submodule(0);
    state.set_screen_brightness(2);
    state.ending_scratch_mut().set_primary_word(0x0300);

    state.credits_handle_scene_fade();

    assert_eq!(state.game_state.display.screen_brightness, 1);
    assert_eq!(state.game_state.dungeon.scratch_word.primary_word(), 0x0301);
    assert_eq!(state.game_state.frame.submodule, 0);
}

#[test]
fn credits_scene_fade_holds_scratch_when_fade_completes() {
    let mut state = ZeldaState::new();
    state.set_submodule(0);
    state.set_screen_brightness(1);
    state.ending_scratch_mut().set_primary_word(0x0300);

    state.credits_handle_scene_fade();

    assert_eq!(state.game_state.display.screen_brightness, 0);
    assert_eq!(state.game_state.dungeon.scratch_word.primary_word(), 0x0300);
    assert_eq!(state.game_state.frame.submodule, 1);
}

#[test]
fn continued_source_call_stack_does_not_reenter_module_routing() {
    let mut state = ZeldaState::new();
    state.set_main_module(8);
    state.set_submodule(1);
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.original_timing_semantic_receipts = Some(OriginalTimingHostReceipts::new(
        0,
        0,
        vec![OriginalTimingSemanticReceipt::MainLoopProgress(
            crate::MainLoopProgress::CallStackContinued,
        )],
    ));
    state.game_execution_scheduler.begin_host_frame();
    let entry_frame = state.game_state.frame;

    state.zelda_run_game_loop();

    assert_eq!(state.game_state.frame, entry_frame);
    assert!(state.game_execution_scheduler.is_idle());
    assert!(state
        .original_timing_semantic_receipts
        .as_ref()
        .is_some_and(|receipts| receipts.semantic().is_empty()));
}

#[test]
fn byte_array_append_vl_matches_c_encoding() {
    let mut arr = ByteArray::default();

    ZeldaState::byte_array_append_vl(&mut arr, 0);
    ZeldaState::byte_array_append_vl(&mut arr, 254);
    ZeldaState::byte_array_append_vl(&mut arr, 255);
    ZeldaState::byte_array_append_vl(&mut arr, 511);

    assert_eq!(arr.data, vec![0, 254, 255, 0, 255, 255, 1]);

    let mut pos = 0usize;
    assert_eq!(ZeldaState::state_recorder_read_vl(&arr.data, &mut pos), 0);
    assert_eq!(ZeldaState::state_recorder_read_vl(&arr.data, &mut pos), 254);
    assert_eq!(ZeldaState::state_recorder_read_vl(&arr.data, &mut pos), 255);
    assert_eq!(ZeldaState::state_recorder_read_vl(&arr.data, &mut pos), 511);
    assert_eq!(pos, arr.data.len());
}

#[test]
fn multi_patch_and_patch_command_update_ram_and_log() {
    let mut state = ZeldaState::new();
    let mut sr = StateRecorder::default();
    let mut mp = StateRecoderMultiPatch::default();
    ZeldaState::state_recoder_multi_patch_init(&mut mp);

    state.state_recoder_multi_patch_patch(&mut sr, &mut mp, 0x20, 0xaa);
    state.state_recoder_multi_patch_patch(&mut sr, &mut mp, 0x21, 0xbb);
    ZeldaState::state_recoder_multi_patch_commit(&mut sr, &mut mp);

    assert_eq!(&state.ram[0x20..0x22], &[0xaa, 0xbb]);
    assert_eq!(sr.log.data, vec![0xc4, 0x00, 0x20, 0xaa, 0xbb]);

    state.patch_command('w');
    assert_eq!(state.ram[0xf372], 80);
    assert_eq!(state.ram[0xf373], 80);
    assert!(!state.state_recorder.log.data.is_empty());
}

#[test]
fn ledge_hop_timer_restores_previous_position_until_triggered() {
    let mut state = ZeldaState::new();
    set_link_test_word(&mut state, LINK_Y_COORD, 0x120);
    set_link_test_word(&mut state, LINK_X_COORD, 0x240);
    set_link_test_word(&mut state, LINK_Y_COORD_PREV, 0x100);
    set_link_test_word(&mut state, LINK_X_COORD_PREV, 0x200);
    set_link_test_byte(&mut state, LINK_SUBPIXEL_Y, 3);
    set_link_test_byte(&mut state, LINK_SUBPIXEL_X, 4);
    set_link_test_byte(&mut state, LINK_TIMER_JUMP_LEDGE, 2);

    assert!(!state.run_ledge_hop_timer());
    assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x100);
    assert_eq!(link_test_word(&state, LINK_X_COORD), 0x200);
    assert_eq!(link_test_byte(&state, LINK_SUBPIXEL_Y), 0);
    assert_eq!(link_test_byte(&state, LINK_SUBPIXEL_X), 0);
}

#[test]
fn move_position_applies_sand_drag_to_velocity_delta() {
    let mut state = ZeldaState::new();
    set_link_test_word(&mut state, LINK_X_COORD, 0x0100);
    set_link_test_word(&mut state, LINK_Y_COORD, 0x0200);
    state.follower_link_state_mut().set_actual_x_velocity(16);
    state
        .follower_link_state_mut()
        .set_actual_y_velocity(0u8.wrapping_sub(16));
    write_le_u16(&mut state.ram, DRAG_PLAYER_X, 1);
    write_le_u16(&mut state.ram, DRAG_PLAYER_Y, 0xffff);

    state.link_move_position();

    assert_eq!(link_test_word(&state, LINK_X_COORD), 0x0102);
    assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x01fe);
    assert_eq!(link_test_byte(&state, LINK_X_VEL), 2);
    assert_eq!(link_test_byte(&state, LINK_Y_VEL), 0xfe);
    assert_eq!(link_test_byte(&state, LINK_X_COORD_SAFE_RETURN_LO), 0x00);
    assert_eq!(link_test_byte(&state, LINK_X_COORD_SAFE_RETURN_HI), 0x01);
    assert_eq!(link_test_byte(&state, LINK_Y_COORD_SAFE_RETURN_LO), 0x00);
    assert_eq!(link_test_byte(&state, LINK_Y_COORD_SAFE_RETURN_HI), 0x02);
}

#[test]
fn move_position_applies_moving_floor_before_velocity_delta() {
    let mut state = ZeldaState::new();
    set_link_test_word(&mut state, LINK_X_COORD, 0x0100);
    set_link_test_word(&mut state, LINK_Y_COORD, 0x0200);
    state.dungeon_room_load_mut().set_header_collision(1);
    state.set_player_layer_collision_flags(
        crate::game_state::constants::player::LAYER_COLLISION_BOTH,
    );
    state.dungeon_moving_floor_mut().set_floor_x_velocity(2);
    state
        .dungeon_moving_floor_mut()
        .set_floor_y_velocity(0xffff);

    state.link_move_position();

    assert_eq!(link_test_word(&state, LINK_X_COORD), 0x0102);
    assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x01ff);
    assert_eq!(link_test_byte(&state, LINK_DIRECTION), 0x09);
    assert_eq!(link_test_byte(&state, LINK_X_VEL), 2);
    assert_eq!(link_test_byte(&state, LINK_Y_VEL), 0xff);
}

#[test]
fn swim_stroke_updates_subpixels_and_actual_velocity() {
    let mut state = ZeldaState::new();
    set_link_test_word(&mut state, LINK_X_COORD, 0x0100);
    set_link_test_word(&mut state, LINK_Y_COORD, 0x0200);
    write_le_u16(&mut state.ram, SWIM_STROKE_FRAME_COUNTER, 1);
    write_le_u16(&mut state.ram, SWIM_STROKE_FRAME_COUNTER + 2, 1);
    state.swim_acceleration_mut().set_mode(0, 0);
    state.swim_acceleration_mut().set_mode(2, 0);
    state.swim_acceleration_mut().set_acceleration(0, 4);
    state.swim_acceleration_mut().set_acceleration(2, 4);
    state.swim_acceleration_mut().set_max_speed(0, 32);
    state.swim_acceleration_mut().set_max_speed(2, 32);
    state
        .swim_acceleration_mut()
        .set_acceleration_direction(0, 1);
    state
        .swim_acceleration_mut()
        .set_acceleration_direction(2, 1);

    state.handle_swim_stroke_and_subpixels();

    assert_eq!(link_test_byte(&state, LINK_DIRECTION), 0x05);
    assert_eq!(
        state.game_state.player.swim_acceleration.acceleration(0),
        12
    );
    assert_eq!(
        state.game_state.player.swim_acceleration.acceleration(2),
        12
    );
    assert_eq!(link_test_byte(&state, LINK_SUBPIXEL_X), 12);
    assert_eq!(link_test_byte(&state, LINK_SUBPIXEL_Y), 12);
    assert_eq!(state.game_state.player.follower_link.actual_x_velocity(), 0);
    assert_eq!(state.game_state.player.follower_link.actual_y_velocity(), 0);
}

#[test]
fn moving_animation_uses_some_direction_bits_when_flag_moving() {
    let mut state = ZeldaState::new();
    set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 1);
    set_link_test_byte(&mut state, LINK_FLAG_MOVING, 1);
    state.follower_link_state_mut().set_swim_direction_flags(8);
    state.follower_link_state_mut().set_joypad1h_last(8);

    state.link_handle_moving_animation_full_long_entry();

    assert_eq!(state.game_state.player.follower_link.facing(), 0);
    assert_eq!(state.game_state.player.follower_link.animation_step(), 1);
}

#[test]
fn moving_animation_dash_advances_dash_cycle() {
    let mut state = ZeldaState::new();
    set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 1);
    state.follower_link_state_mut().set_running_state(1);
    set_link_test_byte(&mut state, LINK_COUNTDOWN_FOR_DASH, 32);
    set_link_test_byte(&mut state, LINK_FRAME_CHANGE_COUNTER, 1);

    state.link_handle_moving_animation_full_long_entry();

    assert_eq!(link_test_byte(&state, LINK_FRAME_CHANGE_COUNTER), 0);
    assert_eq!(state.game_state.player.follower_link.animation_step(), 1);
}

#[test]
fn recoil_z_velocity_shift_matches_c_do_while_condition() {
    fn run_recoil_step(initial_recoil_timer: u8) -> (u8, u8) {
        let mut state = ZeldaState::new();
        state.follower_link_state_mut().set_handler_state(2);
        state.follower_link_state_mut().set_auxiliary_state(1);
        state.follower_link_state_mut().set_incapacitated_timer(8);
        set_link_test_byte(&mut state, LINK_RECOILMODE_TIMER, initial_recoil_timer);
        state.follower_link_state_mut().set_actual_z_velocity(0xf8);
        set_link_test_byte(&mut state, LINK_ACTUAL_VEL_Z_COPY, 0x24);
        set_link_test_word(&mut state, LINK_Z_COORD, 0xffff);

        state.link_state_recoil();

        (
            link_test_byte(&state, LINK_RECOILMODE_TIMER),
            state.game_state.player.follower_link.actual_z_velocity(),
        )
    }

    assert_eq!(run_recoil_step(0), (1, 0x09));
    assert_eq!(run_recoil_step(1), (2, 0x12));
    assert_eq!(run_recoil_step(2), (3, 0x12));
}

#[test]
fn link_initialize_applies_misc_bugfix_cleanup() {
    let mut state = ZeldaState::new();
    state.enhanced_features_mut().set_bits(0x1000);
    state.follower_link_state_mut().set_button_mask_b_y(0xff);
    state.ram[ABOUT_TO_JUMP_OFF_LEDGE] = 1;
    set_link_test_byte(&mut state, LINK_IS_NEAR_MOVEABLE_STATUE, 1);
    set_link_test_byte(&mut state, LINK_ON_CONVEYOR_BELT, 1);
    set_link_test_byte(&mut state, LINK_FLAG_MOVING, 1);
    state.set_bg1_y_offset(0x1234);
    state.set_bg1_x_offset(0x5678);
    state.save_progress_mut().set_dark_world_state(1);

    state.link_initialize();

    assert_eq!(state.game_state.player.follower_link.facing(), 2);
    assert_eq!(
        state.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );
    assert_eq!(state.ram[ABOUT_TO_JUMP_OFF_LEDGE], 0);
    assert_eq!(link_test_byte(&state, LINK_IS_NEAR_MOVEABLE_STATUE), 0);
    assert_eq!(link_test_byte(&state, LINK_ON_CONVEYOR_BELT), 0);
    assert_eq!(link_test_byte(&state, LINK_FLAG_MOVING), 0);
    assert_eq!(read_le_u16(&state.ram, BG1_Y_OFFSET), 0);
    assert_eq!(read_le_u16(&state.ram, BG1_X_OFFSET), 0);
    assert_eq!(state.game_state.player.follower_link.handler_state(), 23);
    assert_eq!(link_test_byte(&state, LINK_IS_BUNNY), 1);
    assert_eq!(link_test_byte(&state, LINK_IS_BUNNY_MIRROR), 1);
}

#[test]
fn swim_accels_start_ramp_and_snap_to_table() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_joypad1h_last(0x0d);
    state.swim_acceleration_mut().set_acceleration(0, 0);
    state.swim_acceleration_mut().set_max_speed(0, 0);
    state.swim_acceleration_mut().set_acceleration(2, 260);
    state.swim_acceleration_mut().set_max_speed(2, 384);

    state.link_handle_swim_accels();

    assert_eq!(state.game_state.player.swim_acceleration.acceleration(0), 1);
    assert_eq!(state.game_state.player.swim_acceleration.max_speed(0), 240);
    assert_eq!(state.game_state.player.swim_acceleration.max_speed(2), 288);

    state.link_handle_swim_accels();
    assert_eq!(state.game_state.player.swim_acceleration.max_speed(0), 384);
}

#[test]
fn swim_momentum_sets_direction_and_starting_accel() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_joypad1h_last(0x09);
    set_link_test_byte(&mut state, LINK_FLAG_MOVING, 2);
    state
        .follower_link_state_mut()
        .set_swim_direction_flags(0x04);
    set_link_test_byte(&mut state, LINK_DIRECTION, 0x08);
    state.swim_acceleration_mut().set_max_speed(2, 0x1234);

    state.link_set_momentum();

    assert_eq!(read_le_u16(&state.ram, SWIM_STROKE_FRAME_COUNTER), 8);
    assert_eq!(state.game_state.player.swim_acceleration.mode(0), 2);
    assert_eq!(state.game_state.player.swim_acceleration.max_speed(0), 240);
    assert_eq!(read_le_u16(&state.ram, SWIM_STROKE_FRAME_COUNTER + 2), 8);
    assert_eq!(state.game_state.player.swim_acceleration.mode(2), 0);
    assert_eq!(
        state
            .game_state
            .player
            .swim_acceleration
            .acceleration_direction(2),
        1
    );
    assert_eq!(
        state.game_state.player.swim_acceleration.max_speed(2),
        0x1234
    );
}

#[test]
fn swimming_handler_starts_hard_stroke_and_advances_swim_animation() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_handler_state(4);
    set_link_test_byte(&mut state, LINK_ITEM_FLIPPERS, 1);
    set_link_test_byte(&mut state, LINK_FRAME_CHANGE_COUNTER, 7);
    state.follower_link_state_mut().set_filtered_joypad_l(0x80);
    state.follower_link_state_mut().set_joypad1h_last(8);
    state.swim_acceleration_mut().set_acceleration(0, 1);

    state.player_handler_04_swimming();

    assert_eq!(link_test_byte(&state, LINK_FRAME_CHANGE_COUNTER), 0);
    assert_eq!(state.game_state.player.follower_link.animation_step(), 1);
    assert_eq!(state.ram[SWIM_STROKE_ANIM_STEP], 0);
    assert_eq!(link_test_byte(&state, LINK_SWIM_HARD_STROKE), 0x80);
    assert_eq!(link_test_byte(&state, LINK_MAYBE_SWIM_FASTER), 1);
    assert_eq!(state.ram[SWIMMING_COUNTDOWN], 6);
    assert_eq!(state.game_state.system_signals.sound_effect_1() & 0x3f, 37);
    assert_eq!(
        state.game_state.player.follower_link.swim_direction_flags(),
        8
    );
}

#[test]
fn halt_link_when_using_items_stops_floor_and_platform_motion() {
    let mut state = ZeldaState::new();
    state.dungeon_room_load_mut().set_header_collision_2(2);
    state.set_player_layer_collision_flags(
        crate::game_state::constants::player::LAYER_COLLISION_BOTH,
    );
    set_link_test_byte(&mut state, LINK_Y_VEL, 0x80);
    set_link_test_byte(&mut state, LINK_X_VEL, 0x40);
    set_link_test_byte(&mut state, LINK_DIRECTION, 0x0f);
    set_link_test_byte(&mut state, LINK_SUBPIXEL_Y, 0x55);
    set_link_test_byte(&mut state, LINK_SUBPIXEL_X, 0xaa);
    set_link_test_byte(&mut state, LINK_MOVING_AGAINST_DIAG_TILE, 1);

    state.halt_link_when_using_items();

    assert_eq!(link_test_byte(&state, LINK_Y_VEL), 0);
    assert_eq!(link_test_byte(&state, LINK_X_VEL), 0);
    assert_eq!(link_test_byte(&state, LINK_DIRECTION), 0);
    assert_eq!(link_test_byte(&state, LINK_SUBPIXEL_Y), 0);
    assert_eq!(link_test_byte(&state, LINK_SUBPIXEL_X), 0);
    assert_eq!(link_test_byte(&state, LINK_MOVING_AGAINST_DIAG_TILE), 0);

    state.ram[DUNG_HDR_COLLISION_2] = 0;
    state.set_player_layer_collision_flags(0);
    state
        .follower_link_state_mut()
        .set_somaria_platform_state(1);
    set_link_test_byte(&mut state, LINK_Y_VEL, 7);
    set_link_test_byte(&mut state, LINK_DIRECTION, 0x0f);
    state.halt_link_when_using_items();
    assert_eq!(link_test_byte(&state, LINK_DIRECTION), 0);
    assert_eq!(link_test_byte(&state, LINK_Y_VEL), 7);
}

#[test]
fn rod_hammer_and_bow_item_handlers_advance_c_timers() {
    let mut rod = ZeldaState::new();
    rod.follower_link_state_mut().set_filtered_joypad_h(0x40);
    rod.player_magic_mut().set_magic_power(20);
    rod.ram[EQ_SELECTED_ROD] = 1;
    rod.link_item_rod();
    assert_eq!(link_test_byte(&rod, LINK_MAGIC_POWER), 4);
    assert!(rod.game_state.player.follower_link.item_in_hand_has(1));
    assert_eq!(link_test_byte(&rod, LINK_DEBUG_VALUE_2), 1);
    assert_eq!(link_test_byte(&rod, LINK_DELAY_TIMER_SPIN_ATTACK), 2);
    assert_eq!(rod.ancilla_slot_view(4).ancilla_type(), 2);

    let mut hammer = ZeldaState::new();
    hammer.follower_link_state_mut().set_filtered_joypad_h(0x40);
    hammer.link_item_hammer();
    assert_eq!(hammer.game_state.player.follower_link.item_in_hand(), 2);
    assert_eq!(link_test_byte(&hammer, LINK_CANT_CHANGE_DIRECTION) & 1, 1);
    assert_eq!(link_test_byte(&hammer, LINK_DELAY_TIMER_SPIN_ATTACK), 2);

    let mut bow = ZeldaState::new();
    bow.follower_link_state_mut().set_button_mask_b_y(0x40);
    bow.follower_link_state_mut().set_item_in_hand(0x10);
    set_link_test_byte(&mut bow, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
    bow.ram[PLAYER_HANDLER_TIMER] = 2;
    set_link_test_byte(&mut bow, LINK_CANT_CHANGE_DIRECTION, 1);
    bow.player_resources_mut().set_arrows(2);
    bow.follower_link_state_mut().set_button_b_frames(12);
    bow.link_item_bow();
    assert_eq!(link_test_byte(&bow, LINK_NUM_ARROWS), 1);
    assert!(!bow.game_state.player.follower_link.item_in_hand_has(0x10));
    assert_eq!(
        bow.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );
    assert_eq!(bow.game_state.player.follower_link.button_b_frames(), 9);
    assert_eq!(bow.ancilla_slot_view(4).ancilla_type(), 9);
}

#[test]
fn cane_of_somaria_start_consumes_magic_and_enters_item_pose() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_filtered_joypad_h(0x40);
    set_link_test_byte(&mut state, LINK_MAGIC_POWER, 32);
    set_link_test_byte(&mut state, LINK_MAGIC_CONSUMPTION, 0);

    state.link_item_cane_of_somaria();

    assert_eq!(link_test_byte(&state, LINK_MAGIC_POWER), 24);
    assert_eq!(
        state.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0x40
    );
    assert!(state.game_state.player.follower_link.position_mode_has(8));
    assert_eq!(link_test_byte(&state, LINK_DEBUG_VALUE_2), 1);
    assert_eq!(link_test_byte(&state, LINK_DELAY_TIMER_SPIN_ATTACK), 2);
    assert_eq!(state.ancilla_slot_view(4).ancilla_type(), 0x2c);
}

#[test]
fn bug_net_start_and_finish_match_c_timer_table() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_filtered_joypad_h(0x40);
    state.follower_link_state_mut().set_facing(4);

    state.link_item_net();

    assert_eq!(state.ram[PLAYER_HANDLER_TIMER], 9);
    assert_eq!(link_test_byte(&state, LINK_DELAY_TIMER_SPIN_ATTACK), 2);
    assert_eq!(state.game_state.player.follower_link.position_mode(), 16);
    assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 1);
    assert_eq!(state.game_state.system_signals.sound_effect_1() & 0x3f, 50);

    state.follower_link_state_mut().set_button_mask_b_y(0x40);
    set_link_test_byte(&mut state, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
    state.follower_link_state_mut().set_item_action_step_var(9);
    state.link_item_net();

    assert_eq!(
        state.game_state.player.follower_link.item_action_step_var(),
        0
    );
    assert_eq!(state.ram[PLAYER_HANDLER_TIMER], 0);
    assert_eq!(
        state.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );
    assert_eq!(state.game_state.player.follower_link.position_mode(), 0);
    assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 0);
    assert_eq!(state.game_state.player.follower_link.oam_x_offset(), 0x80);
    assert_eq!(state.game_state.player.follower_link.oam_y_offset(), 0x80);
}

#[test]
fn bug_net_right_facing_finish_does_not_read_past_timer_table() {
    let mut state = ZeldaState::new();
    state.follower_link_state_mut().set_button_mask_b_y(0x40);
    state.follower_link_state_mut().set_facing(6);
    set_link_test_byte(&mut state, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
    state.follower_link_state_mut().set_item_action_step_var(9);
    state.ram[PLAYER_HANDLER_TIMER] = 8;
    state.follower_link_state_mut().set_position_mode(16);
    set_link_test_byte(&mut state, LINK_CANT_CHANGE_DIRECTION, 1);

    state.link_item_net();

    assert_eq!(
        state.game_state.player.follower_link.item_action_step_var(),
        0
    );
    assert_eq!(state.ram[PLAYER_HANDLER_TIMER], 0);
    assert_eq!(
        state.game_state.player.follower_link.button_mask_b_y() & 0x40,
        0
    );
    assert_eq!(state.game_state.player.follower_link.position_mode(), 0);
    assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 0);
    assert_eq!(state.game_state.player.follower_link.oam_x_offset(), 0x80);
    assert_eq!(state.game_state.player.follower_link.oam_y_offset(), 0x80);
}

#[test]
fn tile_main_handler_shallow_water_sets_ripple_and_slosh_sound() {
    let mut state = ZeldaState::new();
    state.set_indoor_flag(1);
    set_link_test_byte(&mut state, LINK_DIRECTION, 1);
    state
        .tile_detect_position_mut()
        .set_location_calc_mask(0x01ff);
    state
        .dungeon_bg2_attributes_mut()
        .set_bg2_attr(16 * 8 + 1, 0x09);

    state.tile_detect_main_handler(0);

    assert_eq!(
        state
            .game_state
            .player
            .follower_link
            .water_ripple_or_grass_state(),
        1
    );
    assert_eq!(state.game_state.system_signals.sound_effect_1() & 0x3f, 28);
    assert_eq!(state.ram[RAW_SFX_PAN_VALUE], 28);
}

#[test]
fn progressed_disable_then_reenable_refuses_a_cold_timing_restart() {
    let mut state = ZeldaState::new();
    state.set_rom_startup_timing(true);
    state.frame_ctr_dbg = 1;

    state.set_rom_startup_timing(false);
    assert_eq!(state.original_timing_owner(), OriginalTimingOwner::Disabled);
    let audio_after_disable = state.zelda_audio_snapshot_bytes();

    state.set_rom_startup_timing(true);

    assert!(state.rom_startup_timing());
    assert_eq!(
        state.original_timing_owner(),
        OriginalTimingOwner::Unavailable(OriginalTimingUnavailableReason::ProgressedState)
    );
    assert_eq!(state.zelda_audio_snapshot_bytes(), audio_after_disable);
    assert_eq!(state.rom_reset_frame_delay, 0);
}

#[test]
fn game_execution_scheduler_transitions_between_typed_continuations() {
    let mut scheduler = GameExecutionScheduler::default();
    scheduler.schedule_work(GameWorkContinuation::FinishAttractThroneRoom, 2);
    assert_eq!(
        scheduler.current_work(),
        Some(GameWorkContinuation::FinishAttractThroneRoom)
    );
    assert_eq!(
        scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Waiting)
    );
    assert_eq!(
        scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishAttractThroneRoom
        ))
    );

    scheduler.schedule_pre_main_nmi_resume(PreMainNmiResume::OverworldAuxGraphicsReturn);
    assert!(!scheduler.is_idle());
    assert_eq!(scheduler.current_work(), None);
    assert_eq!(
        scheduler.take_pre_main_nmi_resume(),
        Some(PreMainNmiResume::OverworldAuxGraphicsReturn)
    );

    scheduler.schedule_pre_main_caller_continuation(PreMainCallerContinuation::DialogueVwfReturn);
    assert!(scheduler.pre_main_caller_continuation_is(PreMainCallerContinuation::DialogueVwfReturn));
    scheduler.finish_pre_main_caller_continuation(PreMainCallerContinuation::DialogueVwfReturn);
    assert_eq!(scheduler, GameExecutionScheduler::default());
}

#[test]
fn game_execution_scheduler_preserves_non_work_continuations_when_advanced() {
    let mut scheduler = GameExecutionScheduler::default();
    scheduler.schedule_work(GameWorkContinuation::FinishAttractThroneRoom, 1);
    assert_eq!(scheduler.advance_startup_sequence(), None);
    assert_eq!(
        scheduler.advance_work_one_nmi_slice(),
        Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishAttractThroneRoom
        ))
    );

    scheduler.schedule_pre_main_nmi_resume(PreMainNmiResume::OverworldAuxGraphicsReturn);
    assert_eq!(scheduler.advance_work_one_nmi_slice(), None);
    assert_eq!(scheduler.advance_startup_sequence(), None);
    assert_eq!(
        scheduler.take_pre_main_nmi_resume(),
        Some(PreMainNmiResume::OverworldAuxGraphicsReturn)
    );

    scheduler.schedule_pre_main_caller_continuation(PreMainCallerContinuation::DialogueVwfReturn);
    assert_eq!(scheduler.advance_work_one_nmi_slice(), None);
    assert_eq!(scheduler.advance_startup_sequence(), None);
    assert!(scheduler.pre_main_caller_continuation_is(PreMainCallerContinuation::DialogueVwfReturn));
    scheduler.finish_pre_main_caller_continuation(PreMainCallerContinuation::DialogueVwfReturn);

    scheduler.schedule_file_select_graphics();
    assert_eq!(scheduler.advance_work_one_nmi_slice(), None);
    assert_eq!(
        scheduler.advance_startup_sequence(),
        Some(StartupSequenceStep::FileSelectWaiting)
    );
    scheduler.reset();

    scheduler.schedule_selected_game_load(SelectedGameLoadDestination::Dungeon);
    assert_eq!(scheduler.advance_work_one_nmi_slice(), None);
    assert_eq!(
        scheduler.advance_startup_sequence(),
        Some(StartupSequenceStep::SelectedGameLoadWaiting)
    );
}

#[test]
#[should_panic(expected = "cannot schedule")]
fn game_execution_scheduler_rejects_parallel_continuation_kinds() {
    let mut scheduler = GameExecutionScheduler::default();
    scheduler.schedule_work(GameWorkContinuation::FinishAttractThroneRoom, 1);
    scheduler.schedule_pre_main_caller_continuation(PreMainCallerContinuation::DialogueVwfReturn);
}

#[test]
fn hog_spear_body_boundary_keeps_both_increments_without_replaying_movement() {
    let mut state = ZeldaState::new();
    state.set_main_module(9);
    state.set_submodule(0);
    state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    state
        .oam_state_mut()
        .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
    state.follower_link_state_mut().set_position(0x00c0, 0x00c0);
    {
        let mut sprite = state.sprite_slot_view_mut(0);
        sprite.set_state(9);
        sprite.set_sprite_type(0x45);
        sprite.set_x(0x0040);
        sprite.set_y(0x0080);
        sprite.set_subtype2(6);
        sprite.set_graphics(0);
        sprite.set_x_velocity(16);
    }
    let mut atomic = state.clone();
    atomic.sprite_main();
    state.arm_sprite_main_cpu_continuation(
        SpriteMainCpuBoundary::HogSpearBodyGraphicsPending { slot: 0 },
        1,
        SpriteMainCpuCaller::DungeonModule07,
    );
    state.sprite_main();
    assert_eq!(state.sprite_slot_view(0).subtype2(), 8);
    assert_eq!(state.sprite_slot_view(0).graphics(), 0);
    let x = state.sprite_slot_view(0).x();
    let y = state.sprite_slot_view(0).y();
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishSpriteMain {
            boundary: SpriteMainCpuBoundary::HogSpearBodyGraphicsPending { slot: 0 },
            ..
        })
    ));
    state.guard_update_body_graphics(0);
    assert_eq!(state.sprite_slot_view(0).subtype2(), 8);
    assert_eq!(state.sprite_slot_view(0).x(), x);
    assert_eq!(state.sprite_slot_view(0).y(), y);
    assert_eq!(
        state.sprite_slot_view(0).graphics(),
        atomic.sprite_slot_view(0).graphics()
    );
    assert_eq!(
        state.sprite_slot_view(0).x(),
        atomic.sprite_slot_view(0).x()
    );
    assert_eq!(
        state.sprite_slot_view(0).y(),
        atomic.sprite_slot_view(0).y()
    );
}

#[test]
fn happiness_pond_payment_precedes_graphics_and_is_not_replayed() {
    for contribution in [5, 100] {
        let mut state = ZeldaState::new();
        state.set_main_module(7);
        state.set_submodule(0);
        state.set_dungeon_room(21);
        state.follower_link_state_mut().set_position(0x80, 0x80);
        state.player_resources_mut().set_rupees_goal(300);
        {
            let mut sprite = state.sprite_slot_view_mut(5);
            sprite.set_state(9);
            sprite.set_sprite_type(0x72);
            sprite.set_ai_state(3);
            sprite.set_direction(contribution);
            sprite.set_head_direction(0);
            sprite.set_x(0x80);
            sprite.set_y(0x80);
        }
        let mut atomic = state.clone();
        atomic.sprite_72_fairy_pond(5);
        assert!(state.sprite_happiness_pond_before_rupee_graphics(5));
        assert_eq!(state.sprite_slot_view(5).delay_main(), 80);
        assert_eq!(state.sprite_slot_view(5).ai_state(), 3);
        assert_eq!(
            state.game_state.inventory.player_resources.rupees_goal(),
            300 - u16::from(contribution)
        );
        state.complete_happiness_pond_rupee_graphics(5);
        assert_eq!(state.game_state, atomic.game_state);
        assert_eq!(state.ram, atomic.ram);
    }
}

#[test]
fn module09_item_graphics_boundary_executes_the_vendor_prefix_before_suspending() {
    let mut state = ZeldaState::new();
    let slot = 12usize;
    state.rom_startup_timing = true;
    state.original_timing_owner = OriginalTimingOwnerState::Live;
    state.set_main_module(9);
    state.set_submodule(0);
    state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    state
        .oam_state_mut()
        .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
    state.clear_modal_pause_flag();
    state.follower_link_state_mut().clear_auxiliary_state();
    state.follower_link_state_mut().clear_item_hold_pose();
    state.follower_link_state_mut().clear_state_bits();
    state.follower_link_state_mut().set_x(0x1000);
    state.follower_link_state_mut().set_y(0x1000);
    for ancilla in 0..5 {
        state.ancilla_slot_view_mut(ancilla).clear();
    }
    {
        let mut sprite = state.sprite_slot_view_mut(slot);
        sprite.set_state(9);
        sprite.set_sprite_type(0x75);
        sprite.set_deflection_bits(0x80);
        sprite.set_ai_state(2);
    }
    state.player_resources_mut().set_rupees_goal(150);
    state.install_rom_random_replay(vec![crate::RomRandomSample::new(0, 0x85)], 0);
    state.rom_random_replay.begin_frame();

    let interruption =
        crate::MainLoopInterruption::SpriteMainItemReceiptGraphicsStarted(slot as u8);
    let mut receipts = OriginalTimingHostReceipts::new(
        0,
        0,
        vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                ItemReceiptGraphicsProgressReceipt {
                    caller: ItemReceiptGraphicsCaller::SpriteMain { slot: slot as u8 },
                    progress: SourceCallProgress::Suspended,
                },
            ),
        ],
    );
    receipts
        .forward_main_loop_interruption(interruption, OriginalTimingBoundary::NmiAccepted)
        .unwrap();
    state.original_timing_semantic_receipts = Some(receipts);
    state.game_execution_scheduler.begin_host_frame();
    state.game_execution_scheduler.begin_main_loop_iteration();

    state.complete_module09_sprite_and_hud_suffix();

    assert_eq!(state.rom_random_replay.remaining(), Some(0));
    assert_eq!(state.sprite_slot_view(slot).ai_state(), 2);
    assert!(matches!(
        state.game_execution_scheduler.current_work(),
        Some(GameWorkContinuation::FinishItemReceiptGraphics {
            continuation:
                ItemReceiptGraphicsContinuation::ResumeSpriteMainItemReceipt {
                    sprite_slot,
                    suffix: SpriteMainItemReceiptSuffix::BottleVendor,
                    caller: SpriteMainItemReceiptCallerReturn::Module09(_),
                    ..
                },
        }) if sprite_slot == slot as u8
    ));
}

#[test]
fn deleted_file_return_blanks_after_the_first_active_scanline() {
    let mut state = ZeldaState::new();
    state.set_main_module(3);
    state.set_submodule(4);
    state.set_subsubmodule(0);
    state.clear_select_file_cursor();
    state.follower_link_state_mut().set_filtered_joypad_h(0x10);

    // zelda3/src/select_file.c:SelectFile_Func16 clears both SRAM copies and
    // calls ZeldaWriteSram before ReturnToFileSelect. Snes9x reaches that
    // return at V=58 and the following EnableForceBlank at V=1.
    state.select_file_func16();
    assert_eq!(
        state.pending_file_select_force_blank_output_scanline,
        Some(1)
    );

    state.file_select_erase_triforce();

    assert_eq!(state.pending_file_select_force_blank_output_scanline, None);
    assert_eq!(state.active_display_force_blank_event, Some(1));
}

#[test]
fn ppu_write_helpers_route_to_ppu_registers() {
    let mut state = ZeldaState::new();

    state.zelda_ppu_write(0x2100, 0x8f);
    assert!(state.ppu.forced_blank);
    assert_eq!(state.ppu.brightness, 0x0f);

    state.zelda_ppu_write_word(0x2116, 0x1234);
    assert_eq!(state.ppu.vram_pointer, 0x1234);
}

#[test]
fn configure_ppu_side_space_matches_module_cases() {
    let mut state = ZeldaState::new();
    state.set_main_module(20);
    state.configure_ppu_side_space();
    assert_eq!(state.ppu.extra_left_cur, PPU_SIDE_SPACE_LIMIT as u8);
    assert_eq!(state.ppu.extra_right_cur, PPU_SIDE_SPACE_LIMIT as u8);
    assert_eq!(state.ppu.extra_bottom_cur, 16);

    state.set_main_module(7);
    state.set_bg2_x(0x0110);
    state.set_bg2_y(0x0108);
    state.room_bounds_mut().set_x_bound(0, 0x0100);
    state.room_bounds_mut().set_x_bound(2, 0x0140);
    state.room_bounds_mut().set_y_bound(2, 0x0120);
    state.ram[QUADRANT_FULLSIZE_X] = 0;
    state.ram[QUADRANT_FULLSIZE_Y] = 0;
    state.configure_ppu_side_space();
    assert_eq!(state.ppu.extra_left_cur, 0x10);
    assert_eq!(state.ppu.extra_right_cur, 0x30);
    assert_eq!(state.ppu.extra_bottom_cur, 16);
}
