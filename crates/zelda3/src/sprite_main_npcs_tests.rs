use super::*;

fn fresh_state() -> ZeldaState {
    let mut state = ZeldaState::new();
    state.oam_state_mut().set_current_pointer(OAM_BUF as u16);
    state
        .oam_state_mut()
        .set_current_extended_pointer(BYTEWISE_EXTENDED_OAM as u16);
    state
}

fn make_link_idle(state: &mut ZeldaState) {
    state.clear_modal_pause_flag();
    state.set_submodule(0);
    state.follower_link_state_mut().clear_auxiliary_state();
    state.follower_link_state_mut().clear_item_hold_pose();
    state.follower_link_state_mut().clear_state_bits();
    for slot in 0..5 {
        state.ancilla_slot_view_mut(slot).clear();
    }
    state.follower_link_state_mut().set_x(0x1000);
    state.follower_link_state_mut().set_y(0x1000);
}

#[test]
fn bee_handle_z_sets_z_and_palette_when_head_dir() {
    // Bee_HandleZ: sprite_z[k] = 16; if (head_dir) palette bits from
    // frame_counter are written into sprite_oam_flags.
    let mut state = fresh_state();
    let k = 3;
    state.set_frame_counter(0x20); // (0x20 >> 4) & 3 == 2 → palette = (2+1)<<1 = 6
    {
        let mut sprite = state.sprite_slot_view_mut(k);
        sprite.set_head_direction(1);
        sprite.set_oam_flags(0xff);
    }
    state.bee_handle_z(k);
    let sprite = state.sprite_slot_view(k);
    assert_eq!(sprite.z(), 16);
    assert_eq!(sprite.oam_flags(), (0xff & 0xf1) | 6);
}

#[test]
fn bee_handle_z_skips_palette_when_head_dir_zero() {
    let mut state = fresh_state();
    let k = 5;
    state.set_frame_counter(0xff);
    {
        let mut sprite = state.sprite_slot_view_mut(k);
        sprite.set_head_direction(0);
        sprite.set_oam_flags(0xaa);
    }
    state.bee_handle_z(k);
    let sprite = state.sprite_slot_view(k);
    assert_eq!(sprite.z(), 16);
    assert_eq!(sprite.oam_flags(), 0xaa);
}

#[test]
fn initialize_spawned_bee_sets_active_state_and_delay() {
    let mut state = fresh_state();
    let k = 6; // k & 3 == 2 -> 255

    state.initialize_spawned_bee(k);

    let sprite = state.sprite_slot_view(k);
    assert_eq!(sprite.ai_state(), 1);
    assert_eq!(sprite.delay_main(), 255);
    assert_eq!(sprite.a(), 255);
    assert_eq!(sprite.delay_aux4(), 96);
    assert!(BEE_SPAWN_INITIAL_VELOCITIES.contains(&(sprite.x_velocity() as i8)));
    assert!(BEE_SPAWN_INITIAL_VELOCITIES.contains(&(sprite.y_velocity() as i8)));
}

#[test]
fn sprite_79_bee_routes_dormant_hive_state() {
    let mut state = fresh_state();
    let k = 0;
    {
        let mut sprite = state.sprite_slot_view_mut(k);
        sprite.set_ai_state(0);
        sprite.set_state(9);
    }
    state.sprite_79_bee(k);

    assert_eq!(state.sprite_slot_view(k).state(), 0);
    assert_eq!(state.sprite_slot_view(15).sprite_type(), 0x79);
    assert_eq!(state.sprite_slot_view(15).ai_state(), 1);
}

#[test]
fn bee_main_updates_motion_and_retarget_timer() {
    let mut state = fresh_state();
    let k = 4;
    {
        let mut sprite = state.sprite_slot_view_mut(k);
        sprite.set_state(9);
        sprite.set_deflection_bits(0x80);
        sprite.set_a(64);
        sprite.set_delay_aux4(1); // skip damage path
        sprite.set_delay_main(0);
    }
    state.set_frame_counter(4);
    state.follower_link_state_mut().set_x(0x120);
    state.follower_link_state_mut().set_y(0x220);

    state.bee_main(k);

    let sprite = state.sprite_slot_view(k);
    assert_eq!(sprite.z(), 16);
    assert_eq!(sprite.graphics(), 0);
    assert_eq!(sprite.delay_main(), 68);
    assert_eq!(sprite.oam_flags() & 0x40, 0x40);
}

#[test]
fn player_bee_state_one_caps_after_enough_hits() {
    let mut state = fresh_state();
    let k = 3;
    {
        let mut sprite = state.sprite_slot_view_mut(k);
        sprite.set_ai_state(1);
        sprite.set_state(9);
        sprite.set_deflection_bits(0x80);
        sprite.set_head_direction(1);
        sprite.set_b(0x14);
    }

    state.sprite_b2_player_bee(k);

    let sprite = state.sprite_slot_view(k);
    assert_eq!(sprite.ignore_projectile(), 1);
    assert_eq!(sprite.deflection_bits(), 64);
}

#[test]
fn bottle_merchant_detect_fish_sets_negative_sprite_index() {
    let mut state = fresh_state();
    let vendor = 2;
    let fish = 9;
    state.sprite_set_x(vendor, 0x40);
    state.sprite_set_y(vendor, 0x50);
    state.sprite_set_x(fish, 0x40);
    state.sprite_set_y(fish, 0x50);
    {
        let mut sprite = state.sprite_slot_view_mut(fish);
        sprite.set_state(9);
        sprite.set_sprite_type(0xd2);
    }

    state.bottle_merchant_detect_fish(vendor);

    assert_eq!(state.sprite_slot_view(vendor).e(), 0x80 | fish as u8);
}

#[test]
fn bottle_merchant_buy_bee_spawns_five_rewards() {
    let mut state = fresh_state();
    let k = 1;
    state.sprite_set_x(k, 0x30);
    state.sprite_set_y(k, 0x60);

    state.bottle_merchant_buy_bee(k);

    for j in 11..=15 {
        let sprite = state.sprite_slot_view(j);
        assert_eq!(sprite.sprite_type(), 0xdb);
        assert_eq!(sprite.stunned(), 0xff);
        assert_eq!(sprite.z_velocity(), 32);
        assert_eq!(sprite.delay_aux4(), 32);
    }
    assert_eq!(state.game_state.scratch_counter.value(), 0xff);
}

#[test]
fn bottle_merchant_buy_fish_spawns_reward_types() {
    let mut state = fresh_state();
    let k = 1;
    state.sprite_set_x(k, 0x30);
    state.sprite_set_y(k, 0x60);

    state.bottle_merchant_buy_fish(k);

    assert_eq!(state.sprite_slot_view(15).sprite_type(), 0xd9);
    assert_eq!(state.sprite_slot_view(14).sprite_type(), 0xe2);
    assert_eq!(state.sprite_slot_view(13).sprite_type(), 0xde);
    assert_eq!(state.sprite_slot_view(12).sprite_type(), 0xe0);
    assert_eq!(state.sprite_slot_view(11).sprite_type(), 0xdb);
    assert_eq!(state.sprite_slot_view(11).delay_aux4(), 32);
    assert_eq!(state.game_state.scratch_counter.value(), 0xff);
}

#[test]
fn sprite_bottle_vendor_base_detects_trade_offer() {
    let mut state = fresh_state();
    let k = 2;
    {
        let mut sprite = state.sprite_slot_view_mut(k);
        sprite.set_state(9);
        sprite.set_deflection_bits(0x80);
        sprite.set_ai_state(0);
        sprite.set_e(3);
    }
    make_link_idle(&mut state);

    state.sprite_bottle_vendor(k);

    assert_eq!(state.sprite_slot_view(k).ai_state(), 3);
}

#[test]
fn sprite_bottle_vendor_selling_accepts_when_affordable() {
    let mut state = fresh_state();
    let k = 2;
    {
        let mut sprite = state.sprite_slot_view_mut(k);
        sprite.set_state(9);
        sprite.set_deflection_bits(0x80);
        sprite.set_ai_state(1);
    }
    state.multiselect_choice_mut().set_value(0);
    state.player_resources_mut().set_rupees_goal(100);
    make_link_idle(&mut state);

    state.sprite_bottle_vendor(k);

    assert_eq!(state.sprite_slot_view(k).ai_state(), 2);
    assert_eq!(
        state.game_state.messaging.dialogue_message_index.value(),
        0xd2
    );
}

#[test]
fn sprite_bottle_vendor_giving_marks_bottle_bought_and_charges_rupees() {
    let mut state = fresh_state();
    let k = 2;
    {
        let mut sprite = state.sprite_slot_view_mut(k);
        sprite.set_state(9);
        sprite.set_deflection_bits(0x80);
        sprite.set_ai_state(2);
    }
    state.player_resources_mut().set_rupees_goal(150);
    make_link_idle(&mut state);

    state.sprite_bottle_vendor(k);

    assert_eq!(state.sprite_slot_view(k).ai_state(), 0);
    assert_eq!(state.ram[SRAM_PROGRESS_INDICATOR_3_NPCS] & 2, 2);
    assert_eq!(
        state.game_state.inventory.player_resources.rupees_goal(),
        50
    );
}

#[test]
fn sprite_bottle_vendor_reward_clears_fish_and_spawns_rewards() {
    let mut state = fresh_state();
    let k = 2;
    let fish = 5;
    {
        let mut sprite = state.sprite_slot_view_mut(k);
        sprite.set_state(9);
        sprite.set_deflection_bits(0x80);
        sprite.set_ai_state(4);
        sprite.set_e(0x80 | fish as u8);
    }
    state.sprite_slot_view_mut(fish).set_state(9);
    make_link_idle(&mut state);

    state.sprite_bottle_vendor(k);

    assert_eq!(state.sprite_slot_view(fish).state(), 0);
    assert_eq!(state.sprite_slot_view(k).e(), 0);
    assert_eq!(state.sprite_slot_view(k).ai_state(), 0);
    assert_eq!(state.sprite_slot_view(15).sprite_type(), 0xd9);
}

#[test]
fn sprite_find_empty_bottle_locates_value_two() {
    // Sprite_Find_EmptyBottle returns first slot whose value is 2.
    let mut state = fresh_state();
    state.inventory_items_mut().set_bottle(0, 1);
    state.inventory_items_mut().set_bottle(1, 1);
    state.inventory_items_mut().set_bottle(2, 2);
    state.inventory_items_mut().set_bottle(3, 2);
    assert_eq!(state.sprite_find_empty_bottle(), 2);

    // None empty → returns -1.
    state.inventory_items_mut().set_bottle(2, 1);
    state.inventory_items_mut().set_bottle(3, 1);
    assert_eq!(state.sprite_find_empty_bottle(), -1);
}

#[test]
fn bee_put_in_bottle_stores_bottle_and_clears_state() {
    // First branch: choice_in_multiselect_box == 0 and an empty bottle
    // exists. Expect: link_bottle_info[j] = 7 + sprite_head_dir[k],
    // sprite_state[k] = 0.
    let mut state = fresh_state();
    // Make Sprite_ReturnIfInactive(k) return false: state=9, no flags.
    let k = 1;
    state.sprite_slot_view_mut(k).set_state(9);
    state.clear_modal_pause_flag();
    state.set_submodule(0);
    {
        let mut sprite = state.sprite_slot_view_mut(k);
        sprite.set_deflection_bits(0x80);
        sprite.set_pause(0);
    }
    state.multiselect_choice_mut().set_value(0);
    state.inventory_items_mut().set_bottle(0, 1);
    state.inventory_items_mut().set_bottle(1, 2); // first empty
    state.sprite_slot_view_mut(k).set_head_direction(0);

    state.bee_put_in_bottle(k);
    assert_eq!(state.game_state.inventory.items.bottle(1), 7);
    assert_eq!(state.sprite_slot_view(k).state(), 0);
}

#[test]
fn bee_put_in_bottle_arms_delay_when_no_bottle() {
    // No empty bottle → calls Sprite_ShowMessageUnconditional(0xca)
    // and falls through to delay+ai_state writes.
    let mut state = fresh_state();
    let k = 2;
    {
        let mut sprite = state.sprite_slot_view_mut(k);
        sprite.set_state(9);
        sprite.set_deflection_bits(0x80);
    }
    state.multiselect_choice_mut().set_value(0);
    for i in 0..4 {
        state.inventory_items_mut().set_bottle(i, 1);
    }

    state.bee_put_in_bottle(k);
    // delay and ai_state are written unconditionally after the bottle
    // check fails.
    let sprite = state.sprite_slot_view(k);
    assert_eq!(sprite.delay_aux4(), 64);
    assert_eq!(sprite.ai_state(), 1);
    // Sprite_ShowMessageUnconditional(0xca) wrote dialogue index and
    // bumped main_module_index to 14.
    assert_eq!(
        state.game_state.messaging.dialogue_message_index.value(),
        0xca
    );
    assert_eq!(state.game_state.frame.main_module, 14);
}

#[test]
fn player_bee_hone_in_on_target_bumps_sprite_b_and_recoil() {
    // For a normal target (type != 0x88, flags&2 == 0, type != 0x75)
    // within range, expect F=15, recoil = vel<<1, sprite_B[k] +=1.
    let mut state = fresh_state();
    let k = 2;
    let j = 5;
    {
        let mut target = state.sprite_slot_view_mut(j);
        target.set_sprite_type(0x10);
        target.set_flags(0); // bit 1 clear
    }
    // x=y=0; cur_x=0,cur_y=0 → (0-0+16)=16<24 ✓; (0-0-8)=0xfff8 huge — fails.
    // Set Sprite_GetX/Y to large so the deltas pass.
    state.sprite_slot_view_mut(j).set_x(0x10);
    state.sprite_slot_view_mut(j).set_y(0x10);
    state.sprite_workspace_mut().set_current_sprite_x(0x10);
    state.sprite_workspace_mut().set_current_sprite_y(0x18);
    // cur_x - x + 16 = 0 + 16 = 16  (<24 ✓)
    // cur_y - y - 8  = 8         (<24 ✓)
    {
        let mut source = state.sprite_slot_view_mut(k);
        source.set_x_velocity(3);
        source.set_y_velocity(2);
        source.set_b(7);
    }

    state.player_bee_hone_in_on_target(j, k);
    let target = state.sprite_slot_view(j);
    assert_eq!(target.f(), 15);
    assert_eq!(target.x_recoil(), 6);
    assert_eq!(target.y_recoil(), 4);
    assert_eq!(state.sprite_slot_view(k).b(), 8);
}
