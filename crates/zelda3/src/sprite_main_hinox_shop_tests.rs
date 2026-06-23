use super::*;

fn fresh_state() -> ZeldaState {
    ZeldaState::new()
}

// ---- Hinox tests ----

#[test]
fn hinox_set_direction_writes_velocity_tables() {
    let mut s = fresh_state();
    // Seed RNG-related state so get_random_number is deterministic.
    s.sprite_slot_view_mut(3).set_ai_state(0);
    s.hinox_set_direction(3, 0);
    let sprite = s.sprite_slot_view(3);
    assert_eq!(sprite.direction(), 0);
    assert_eq!(sprite.x_velocity(), 8);
    assert_eq!(sprite.y_velocity(), 0);
    assert_eq!(sprite.ai_state(), 1);

    s.hinox_set_direction(3, 1);
    let sprite = s.sprite_slot_view(3);
    assert_eq!(sprite.direction(), 1);
    assert_eq!(sprite.x_velocity(), (-8i8) as u8);
    assert_eq!(sprite.y_velocity(), 0);
    assert_eq!(sprite.ai_state(), 2);

    s.hinox_set_direction(3, 2);
    let sprite = s.sprite_slot_view(3);
    assert_eq!(sprite.x_velocity(), 0);
    assert_eq!(sprite.y_velocity(), 8);

    s.hinox_set_direction(3, 3);
    let sprite = s.sprite_slot_view(3);
    assert_eq!(sprite.x_velocity(), 0);
    assert_eq!(sprite.y_velocity(), (-8i8) as u8);
}

#[test]
fn hinox_face_link_doubles_velocity_after_set_direction() {
    let mut s = fresh_state();
    // Set up so DirectionToFaceLink returns 0 (right).
    // sprite at (0,0), link at (100,0) -> dx>0 dominant axis -> dir 0.
    s.follower_link_state_mut().set_x(100);
    s.follower_link_state_mut().set_y(0);
    s.hinox_face_link(5);
    // After hinox_set_direction with dir=0, x_vel=8, then shifted left by 1 -> 16.
    let sprite = s.sprite_slot_view(5);
    assert_eq!(sprite.x_velocity(), 16);
    assert_eq!(sprite.y_velocity(), 0);
    assert_eq!(sprite.direction(), 0);
}

#[test]
fn hinox_throw_bomb_is_noop() {
    let mut s = fresh_state();
    let before = s.ram.clone();
    s.hinox_throw_bomb(7);
    assert_eq!(s.ram, before, "Hinox_ThrowBomb's C body is empty");
}

// ---- ShopItem tests ----

#[test]
fn shop_item_handle_cost_succeeds_when_affordable() {
    let mut s = fresh_state();
    s.player_resources_mut().set_rupees_goal(200);
    assert!(s.shop_item_handle_cost(150));
    assert_eq!(s.game_state.inventory.player_resources.rupees_goal(), 50);
}

#[test]
fn shop_item_handle_cost_rejects_when_too_expensive() {
    let mut s = fresh_state();
    s.player_resources_mut().set_rupees_goal(50);
    assert!(!s.shop_item_handle_cost(150));
    assert_eq!(
        s.game_state.inventory.player_resources.rupees_goal(),
        50,
        "rupees unchanged on failed cost"
    );
}

#[test]
fn shop_item_check_for_a_press_requires_a_button() {
    let mut s = fresh_state();
    s.follower_link_state_mut().set_filtered_joypad_l(0);
    assert!(!s.shop_item_check_for_a_press(0));
    // With A pressed but no Link overlap, the canonical damage check still
    // returns false; just confirm the early-exit doesn't fire.
    s.follower_link_state_mut().set_filtered_joypad_l(0x80);
    let _ = s.shop_item_check_for_a_press(0);
}

#[test]
fn shop_item_make_shields_deflect_writes_expected_flags() {
    let mut s = fresh_state();
    {
        let mut sprite = s.sprite_slot_view_mut(4);
        sprite.set_ignore_projectile(0xff);
        sprite.set_flags(0);
        sprite.set_deflection_bits(0);
        sprite.set_flags4(0);
    }
    s.shop_item_make_shields_deflect(4);
    let sprite = s.sprite_slot_view(4);
    assert_eq!(sprite.ignore_projectile(), 0);
    assert_eq!(sprite.flags(), 8);
    assert_eq!(sprite.deflection_bits(), 4);
    // Final flags4 value is 0xa (overwrites the bracketing 0x1c).
    assert_eq!(sprite.flags4(), 0xa);
}

#[test]
fn shop_item_handle_receipt_clears_method_and_skips_msg_for_low_subtype() {
    let mut s = fresh_state();
    s.follower_link_state_mut().set_item_receipt_method(5);
    s.sprite_slot_view_mut(1).set_subtype2(3); // < 7, no message branch
    s.shop_item_handle_receipt(1, 0x2e);
    assert_eq!(s.game_state.player.follower_link.item_receipt_method(), 0);
}
