//! Player initialization and action transitions; RAM publication lives in compatibility.

use super::FollowerLinkState;

impl FollowerLinkState {
    pub(super) fn initialize_link_action_state(&mut self) {
        self.movement.facing = 2;
        self.movement.last_direction = 0;
        self.clear_item_action_sequence();
        self.actions.transforming = 0;
        self.actions.y_button_action_flags = 0;
        self.input.button_mask_b_y &= !0x40;
        self.clear_state_item_and_grab_flags();
    }

    pub(super) fn reset_action_state(&mut self) {
        self.movement.tile_action_index = 0;
        self.presentation.spin_animation_step_counter = 0;
        self.actions.spin_attack_state = 0;
        self.movement.tile_collision_flag = 0;
        self.presentation.force_hold_sword_up = 0;
        self.actions.sword_delay_timer = 0;
        self.clear_item_action_sequence();
        self.actions.y_button_action_flags = 0;
        self.input.button_mask_b_y = 0;
        self.input.clear_button_b_frames();
        self.clear_state_item_and_grab_flags();
        self.movement.direction_lock = 0;
        self.actions.auxiliary_state = 0;
        self.actions.incapacitated_timer = 0;
        self.actions.action_handler_timer = 0;
        self.actions.sprite_damage_disabled = 0;
        self.presentation.item_hold_pose = 0;
        self.actions.ancilla_pickup_flag = 0;
        self.actions.sprite_pickup_flag = 0;
        self.actions.clear_pull_for_rupees_sprite_need();
        self.movement.clear_near_moveable_statue();
        self.actions.spin_attack_step_counter = 0;
        // Reset C publishes these flags too. Keep native state coherent here
        // instead of relying on the cancellation entry to reload them from RAM.
        self.actions.electrocute_on_touch = 0;
        self.actions.cape_mode = 0;
        self.actions.hookshot_interlock = 0;
        self.cancel_sword_and_item_usage();
    }

    pub(super) fn finish_initialization(&mut self) {
        self.movement.direction_lock &= !1;
        self.movement.z &= 0x00ff;
        self.actions.auxiliary_state = 0;
        self.actions.incapacitated_timer = 0;
        self.presentation.blink_countdown = 0;
        self.actions.electrocute_on_touch = 0;
        self.presentation.item_hold_pose = 0;
        self.actions.cape_mode = 0;
        self.actions.sprite_damage_disabled = 0;
        self.actions.action_handler_timer = 0;
        self.movement.direction &= !0x0f;
        self.movement.somaria_platform_state = 0;
        self.actions.spin_attack_step_counter = 0;
        self.unequip_cape_quietly();
        self.cancel_sword_and_item_usage();
    }

    pub(super) fn reset_movement_and_transformation_state(&mut self) {
        self.movement.last_direction = 0;
        self.movement.direction = 0;
        self.movement.movement_flag = 0;
        self.presentation.blink_countdown = 0;
        self.actions.transforming = 0;
        self.actions.bunny_state = 0;
        self.actions.bunny_mirror = 0;
        // C clears only the LOW byte here (ram[LINK_TIMER_TEMPBUNNY] = 0, a u8 store),
        // leaving the high byte intact — so a 0x0100 temp-bunny timer stays 0x0100 and the
        // curse continues. Zeroing the full u16 ended the bunny transformation a frame early.
        self.actions.temp_bunny_timer &= 0xff00;
        self.actions.transform_poof_needed = 0;
        self.actions.clear_pull_for_rupees_sprite_need();
        self.actions.hookshot_grave_latch = 0;
        self.actions.given_damage = 0;
        self.presentation.spin_offsets = 0;
        self.presentation.dash_noise_requested = 0;
        self.actions.item_receipt_method = 0;
        self.movement.bit9_of_xcoord = 0;
        self.movement.whirlpool_trigger = 0;
    }

    pub(super) fn reset_platform_and_pit_state(&mut self) {
        self.movement.somaria_platform_state = 0;
        self.actions.spin_attack_step_counter = 0;
        self.actions.defense_flags = 0;
        self.actions.sprite_pickup_flag_cached = 0;
        self.movement.clear_pit_correction();
        self.movement.pit_data_index = 0;
        self.movement.near_pit_state = 0;
    }

    pub(super) fn cancel_sword_and_item_usage(&mut self) {
        self.movement.set_speed_setting(0);
        self.actions.and_defense_flags(!9);
        self.actions.set_spin_attack_delay_timer(0);
        self.input.set_button_b_frames(0);
        self.input.clear_button_mask_b_y_bits(0x81);
        self.movement.clear_direction_lock_bits(1);
    }

    pub(super) fn finish_recoil_landing(&mut self) {
        self.movement.set_z(0);
        self.actions.clear_auxiliary_state();
        self.movement.set_speed_setting(0);
        self.movement.clear_direction_lock();
        self.actions.clear_item_in_hand();
        self.movement.clear_position_mode();
        self.actions.clear_action_handler_timer();
        self.actions.clear_sprite_damage_disable_timer();
        self.actions.clear_electrocute_on_touch();
        self.movement.clear_actual_velocity_xy();
    }

    pub(super) fn unequip_cape_quietly(&mut self) {
        self.actions.set_cape_transform_timer(32);
        self.actions.clear_sprite_damage_disable_timer();
        self.actions.set_cape_mode(0);
        self.actions.clear_electrocute_on_touch();
    }

    pub(super) fn clear_swim_stroke_counters(&mut self) {
        self.movement.set_swim_stroke_frame_counter(0, 0);
        self.movement.set_swim_stroke_frame_counter(2, 0);
    }

    fn clear_item_action_sequence(&mut self) {
        self.actions.item_in_hand = 0;
        self.movement.position_mode = 0;
        self.actions.item_debug_value_1 = 0;
        self.clear_action_scratch_state();
        self.actions.y_button_action_step = 0;
    }
}
