use super::*;

fn fresh_state() -> ZeldaState {
    let mut state = ZeldaState::new();
    state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    state
        .oam_state_mut()
        .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
    state
}

#[test]
fn somaria_platform_handle_drag_x_writes_drag_when_dirs_oppose() {
    let mut s = fresh_state();
    // sprite_D[k] = 0, sprite_head_dir[k] = 2 → XOR&2 == 2 (truthy).
    s.sprite_slot_view_mut(3).set_direction(0);
    s.sprite_slot_view_mut(3).set_head_direction(2);
    // sprite_x_lo[k] = 0 — x will become 4, t = 4.
    s.sprite_slot_view_mut(3).set_x_low(0);
    write_le_u16(&mut s.ram, DRAG_PLAYER_X, 0);
    s.somaria_platform_handle_drag_x(3);
    assert_eq!(read_le_u16(&s.ram, DRAG_PLAYER_X), 4);
    assert_eq!(s.sprite_slot_view(3).x_low(), 4);
}

#[test]
fn somaria_platform_handle_drag_returns_when_aligned() {
    let mut s = fresh_state();
    // Bit 2 of XOR is zero → no drag.
    s.sprite_slot_view_mut(1).set_direction(0);
    s.sprite_slot_view_mut(1).set_head_direction(1);
    s.sprite_slot_view_mut(1).set_x_low(0);
    write_le_u16(&mut s.ram, DRAG_PLAYER_X, 0);
    s.somaria_platform_handle_drag_x(1);
    assert_eq!(read_le_u16(&s.ram, DRAG_PLAYER_X), 0);
    assert_eq!(s.sprite_slot_view(1).x_low(), 0);
}

#[test]
fn somaria_platform_handle_junctions_b2_xors_d_with_3() {
    let mut s = fresh_state();
    s.sprite_slot_view_mut(5).set_e(0xb2);
    s.sprite_slot_view_mut(5).set_direction(1);
    s.somaria_platform_handle_junctions(5);
    assert_eq!(s.sprite_slot_view(5).direction(), 1 ^ 3);
}

#[test]
fn somaria_platform_handle_junctions_b6_clears_ai_state_when_correct_key_pressed() {
    let mut s = fresh_state();
    s.sprite_slot_view_mut(0).set_e(0xb6);
    s.sprite_slot_view_mut(0).set_direction(0);
    s.follower_link_state_mut().clear_auxiliary_state();
    // kSomariaPlatform_TransitDir[0] = 4; press that bit.
    s.follower_link_state_mut().set_joypad1h_last(4);
    s.somaria_platform_handle_junctions(0);
    assert_eq!(s.sprite_slot_view(0).ai_state(), 0);
    assert_eq!(s.sprite_slot_view(0).direction(), 0 ^ 1);
    assert_eq!(s.game_state.player.follower_link.on_somaria_platform(), 1);
}

#[test]
fn master_sword_main_clears_state_when_event_bit_set() {
    let mut s = fresh_state();
    s.set_main_module(9); // not 26
    s.set_overworld_screen(0x02);
    s.set_overworld_event_info(0x02, 0x40);
    s.sprite_slot_view_mut(4).set_state(9);
    s.master_sword_main(4);
    assert_eq!(s.sprite_slot_view(4).state(), 0);
}

#[test]
fn master_sword_spawn_pendant_prop_sets_velocities_from_ain() {
    let mut s = fresh_state();
    s.follower_link_state_mut().set_x(100);
    s.follower_link_state_mut().set_y(50);
    // Canonical Sprite_SpawnDynamically walks j_in=15 down; the highest
    // free slot in 0..=15 wins.
    for j in 0..=15 {
        s.sprite_slot_view_mut(j).set_state(0);
    }
    s.master_sword_spawn_pendant_prop(0, 9);
    // ain=9 → (9>>1)&3 = 4 & 3 = 0 → xv = -4, yv = -2.
    // The spawn lands in slot 15 (highest free under canonical helper).
    assert_eq!(s.sprite_slot_view(15).graphics(), 4);
    assert_eq!(s.sprite_slot_view(15).subtype2(), 3);
    assert_eq!(s.sprite_slot_view(15).flags2(), 64);
    assert_eq!(s.sprite_slot_view(15).delay_main(), 228);
    assert_eq!(s.sprite_slot_view(15).oam_flags(), 9);
    assert_eq!(s.sprite_slot_view(15).x_velocity() as i8, -4);
    assert_eq!(s.sprite_slot_view(15).y_velocity() as i8, -2);
}

#[test]
fn master_sword_prop_state_2_doubles_velocity_when_delay_zero() {
    let mut s = fresh_state();
    s.sprite_slot_view_mut(6).set_ai_state(2);
    s.sprite_slot_view_mut(6).set_delay_main(0);
    s.sprite_slot_view_mut(6).set_x_velocity(3);
    s.sprite_slot_view_mut(6).set_y_velocity(5);
    s.sprite_slot_view_mut(6).set_e(7);
    s.sprite_master_sword_prop(6);
    assert_eq!(s.sprite_slot_view(6).x_velocity(), 6);
    assert_eq!(s.sprite_slot_view(6).y_velocity(), 10);
    assert_eq!(s.sprite_slot_view(6).delay_main(), 6);
    assert_eq!(s.sprite_slot_view(6).e(), 8);
}

#[test]
fn flute_boy_check_if_player_close_true_when_within_48() {
    let mut s = fresh_state();
    // Place sprite at (100, 100) → yy = 84. link at (130, 130).
    s.sprite_slot_view_mut(2).set_x_low(100);
    s.sprite_slot_view_mut(2).set_y_low(100);
    s.sprite_slot_view_mut(2).set_x_high(0);
    s.sprite_slot_view_mut(2).set_y_high(0);
    s.follower_link_state_mut().set_x(130);
    s.follower_link_state_mut().set_y(130);
    assert!(s.flute_boy_check_if_player_close(2));
}

#[test]
fn flute_boy_check_if_player_close_false_when_distant() {
    let mut s = fresh_state();
    s.sprite_slot_view_mut(2).set_x_low(0);
    s.sprite_slot_view_mut(2).set_y_low(0);
    s.follower_link_state_mut().set_x(200);
    s.follower_link_state_mut().set_y(200);
    assert!(!s.flute_boy_check_if_player_close(2));
}

#[test]
fn flute_kid_spawn_quaver_initializes_z_vel_and_delay() {
    let mut s = fresh_state();
    // Canonical Sprite_SpawnDynamically uses j_in=15 and reads coords
    // from sprite_x_lo[k]/sprite_x_hi[k] (Sprite_GetX), so seed slot 0.
    s.sprite_slot_view_mut(0).set_x_low(0x00);
    s.sprite_slot_view_mut(0).set_x_high(0x01); // = 0x100
    s.sprite_slot_view_mut(0).set_y_low(0x80);
    s.sprite_slot_view_mut(0).set_y_high(0x00); // = 0x080
    for j in 0..=15 {
        s.sprite_slot_view_mut(j).set_state(0);
    }
    s.flute_kid_spawn_quaver(0);
    // Canonical helper picks slot 15 (highest free in 0..=15).
    assert_eq!(s.sprite_slot_view(15).head_direction(), 1);
    assert_eq!(s.sprite_slot_view(15).z_velocity(), 8);
    assert_eq!(s.sprite_slot_view(15).delay_main(), 96);
    assert_eq!(s.sprite_slot_view(15).ignore_projectile(), 96);
    assert_eq!(s.sprite_slot_view(15).sprite_type(), 0x2e);
}

#[test]
fn sprite_flute_kid_quaver_zeros_state_when_delay_zero() {
    let mut s = fresh_state();
    s.sprite_slot_view_mut(4).set_state(9);
    s.sprite_slot_view_mut(4).set_delay_main(0);
    s.set_frame_counter(0); // even → adjust x_vel
    s.sprite_slot_view_mut(4).set_x_velocity(10);
    s.ram[CUR_OBJECT_INDEX] = 0;
    s.sprite_flute_kid_quaver(4);
    // Sprite_ReturnIfInactive is false (state != 0 and other guards), so
    // the body executes. Delay==0 → state cleared.
    assert_eq!(s.sprite_slot_view(4).state(), 0);
}

#[test]
fn sprite_flute_kid_stumpy_starts_music_when_flute_equipped_and_y_pressed() {
    let mut s = fresh_state();
    let k = 3;

    s.sprite_slot_view_mut(k).set_state(9);
    s.sprite_slot_view_mut(k).set_ai_state(3);
    s.save_progress_mut().set_hud_current_item(HUD_ITEM_FLUTE);
    s.follower_link_state_mut().set_joypad1h_last(0x40);

    s.sprite_flute_kid_stumpy(k);

    assert_eq!(s.sprite_slot_view(k).ai_state(), 4);
    assert_eq!(s.game_state.system_signals.music_control(), 0xf2);
    assert_eq!(s.game_state.system_signals.sound_effect_1(), 0);
    assert_eq!(s.game_state.system_signals.ambient_sound_effect(), 23);
    assert_eq!(s.game_state.player.follower_link.immobilized_flag(), 1);
}

#[test]
fn somaria_platform_drag_link_subtracts_when_link_north_west_of_sprite() {
    let mut s = fresh_state();
    // cur_sprite=(100,200), link=(20,80). x = 92-20 = 72 > 0 (no high
    // bit) → +1. y = 184-80 = 104 > 0 → +1.
    s.sprite_workspace_mut().set_current_sprite_x(100);
    s.sprite_workspace_mut().set_current_sprite_y(200);
    s.follower_link_state_mut().set_x(20);
    s.follower_link_state_mut().set_y(80);
    write_le_u16(&mut s.ram, DRAG_PLAYER_X, 0);
    write_le_u16(&mut s.ram, DRAG_PLAYER_Y, 0);
    s.somaria_platform_drag_link(0);
    assert_eq!(read_le_u16(&s.ram, DRAG_PLAYER_X), 1);
    assert_eq!(read_le_u16(&s.ram, DRAG_PLAYER_Y), 1);
}

#[test]
fn somaria_platform_handle_junctions_bc_no_input_sets_ai_state_1() {
    let mut s = fresh_state();
    s.sprite_slot_view_mut(2).set_e(0xbc);
    s.sprite_slot_view_mut(2).set_direction(0);
    s.follower_link_state_mut().set_joypad1h_last(0);
    s.somaria_platform_handle_junctions(2);
    // KEYS6[0] = 0xc; t = 0; ai_state stays 1, player_on flag set.
    assert_eq!(s.sprite_slot_view(2).ai_state(), 1);
    assert_eq!(s.game_state.player.follower_link.on_somaria_platform(), 1);
}

#[test]
fn pipe_handle_player_movement_sets_direction_and_runs_player_motion_tail() {
    let mut s = fresh_state();
    s.set_indoor_flag(0);
    s.follower_link_state_mut().set_speed_setting(0);
    s.follower_link_state_mut().set_facing(0);
    s.follower_link_state_mut()
        .set_direction_and_last_direction(0);
    s.follower_link_state_mut().set_x(0x0120);
    s.follower_link_state_mut().set_y(0x0230);

    s.pipe_handle_player_movement(2);

    assert_eq!(s.game_state.player.follower_link.direction(), 2);
    assert_eq!(s.game_state.player.follower_link.last_direction(), 2);
    // Link_HandleMovingAnimation_FullLongEntry maps left/right movement
    // onto facing 4/6 when direction changes are allowed.
    assert_eq!(s.game_state.player.follower_link.facing(), 4);
    assert_eq!(s.game_state.player.follower_link.x(), 0x011e);
    assert_eq!(s.game_state.player.follower_link.y(), 0x0230);
}

#[test]
fn faerie_handle_movement_animates_moves_and_clamps_low_z() {
    let mut s = fresh_state();
    let k = 3;
    s.set_frame_counter(8);
    s.sprite_set_x(k, 0x0100);
    s.sprite_set_y(k, 0x0200);
    s.sprite_slot_view_mut(k).set_x_velocity(16);
    s.sprite_slot_view_mut(k).set_y_velocity(0);
    s.sprite_slot_view_mut(k).set_z(7);
    s.sprite_slot_view_mut(k).set_z_velocity(0);
    s.sprite_slot_view_mut(k).set_oam_flags(0);

    s.faerie_handle_movement(k);

    assert_eq!(s.sprite_slot_view(k).graphics(), 1);
    assert_eq!(s.sprite_slot_view(k).oam_flags() & 0x40, 0x40);
    assert_eq!(s.sprite_get_x(k), 0x0101);
    assert_eq!(s.sprite_get_y(k), 0x0200);
    assert_eq!(s.sprite_slot_view(k).z(), 8);
    assert_eq!(s.sprite_slot_view(k).z_velocity(), 5);
}

#[test]
fn faerie_handle_movement_averages_velocities_on_16_frame_tick() {
    let mut s = fresh_state();
    let k = 4;
    s.set_frame_counter(16);
    s.sprite_set_x(k, 0x0120);
    s.sprite_set_y(k, 0x0230);
    s.sprite_slot_view_mut(k).set_x_velocity(10);
    s.sprite_slot_view_mut(k).set_y_velocity((-10i8) as u8);
    s.sprite_slot_view_mut(k).set_direction(20);
    s.sprite_slot_view_mut(k).set_a((-20i8) as u8);
    s.sprite_slot_view_mut(k).set_z(12);
    s.sprite_slot_view_mut(k).set_z_velocity(0);

    s.faerie_handle_movement(k);

    assert_eq!(s.sprite_slot_view(k).graphics(), 0);
    assert_eq!(s.sprite_slot_view(k).x_velocity(), 15);
    assert_eq!(s.sprite_slot_view(k).y_velocity(), (-15i8) as u8);
    assert_eq!(s.sprite_get_x(k), 0x0120);
    assert_eq!(s.sprite_get_y(k), 0x022f);
    assert_eq!(s.sprite_slot_view(k).z(), 12);
}
