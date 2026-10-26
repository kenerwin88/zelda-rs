//! Y-button item dispatch, equipment use, spells, mirror transitions, and item ancillas.

use super::*;

impl ZeldaState {
    pub(in crate::zelda_rtl) fn link_force_unequip_cape_quietly(&mut self) {
        self.follower_link_state_mut().unequip_cape_quietly();
    }

    pub(super) fn link_force_unequip_cape(&mut self) {
        self.ancilla_add_cape_poof(35, 4);
        self.ancilla_sfx2_near(21);
        self.link_force_unequip_cape_quietly();
    }

    pub(in crate::zelda_rtl) fn halt_link_when_using_items(&mut self) {
        if self.game_state.dungeon.room_load.header_collision_2() == 2
            && self.has_player_layer_collision(
                crate::game_state::constants::player::LAYER_COLLISION_BOTH,
            )
        {
            self.follower_link_state_mut()
                .clear_movement_velocity_and_direction();
            self.follower_link_state_mut().clear_movement_subpixels();
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
        }
        if self
            .game_state
            .player
            .follower_link
            .has_somaria_platform_state()
        {
            self.follower_link_state_mut().set_direction(0);
        }
    }

    pub(super) fn link_handle_cape_passive_lift_check(&mut self) {
        if self
            .game_state
            .player
            .follower_link
            .is_lifting_or_carrying()
            || (self
                .game_state
                .enhanced_features
                .has(LINK_HANDLE_CAPE_PASSIVE_LIFT_CHECK_FEATURES0_MISC_BUG_FIXES)
                && self
                    .game_state
                    .player
                    .follower_link
                    .has_grabbing_wall_state())
        {
            self.player_check_handle_cape_stuff();
        }
    }

    pub(super) fn player_check_handle_cape_stuff(&mut self) {
        if !self.game_state.player.follower_link.is_cape_active()
            || self.game_state.player.follower_link.current_item_active() != 19
        {
            return;
        }
        if self.game_state.player.follower_link.current_item_active()
            == self.game_state.player.follower_link.current_item_y()
        {
            self.follower_link_state_mut()
                .decrement_cape_decrement_counter();
            if self
                .game_state
                .player
                .follower_link
                .cape_decrement_counter()
                != 0
            {
                return;
            }
            let cape_timer = PLAYER_CHECK_HANDLE_CAPE_STUFF_CAPE_DEPLETION_TIMERS
                [self.magic_consumption_level() as usize];
            self.follower_link_state_mut()
                .set_cape_decrement_counter(cape_timer);
            if self.game_state.inventory.player_resources.magic.amount() == 0 {
                return;
            }
            if self.player_magic_mut().decrement_magic_power() != 0 {
                return;
            }
        }
        self.link_force_unequip_cape();
    }

    pub(super) fn check_y_button_press(&mut self) -> bool {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 != 0
            || self.game_state.player.follower_link.incapacitated_timer() != 0
            || self.game_state.player.follower_link.filtered_joypad_h() & 0x40 == 0
        {
            return false;
        }
        self.follower_link_state_mut()
            .add_button_mask_b_y_bits(0x40);
        true
    }

    pub(super) fn link_check_magic_cost(&mut self, item: u8) -> bool {
        let idx = item as usize * 3 + self.magic_consumption_level() as usize;
        let cost = LINK_CHECK_MAGIC_COST_LINK_ITEM_MAGIC_COSTS[idx];
        if self.player_magic_mut().spend_magic(cost) {
            return true;
        }
        if item != 3 {
            self.ancilla_sfx2_near(60);
            self.dialogue_message_index_mut().set_value(123);
            self.main_show_text_message();
        }
        false
    }

    pub(in crate::zelda_rtl) fn refund_magic(&mut self, item: u8) {
        let idx = item as usize * 3 + self.magic_consumption_level() as usize;
        let cost = REFUND_MAGIC_LINK_ITEM_MAGIC_COSTS[idx];

        let clamp_full = self
            .game_state
            .enhanced_features
            .has(REFUND_MAGIC_FEATURES0_MISC_BUG_FIXES);
        self.player_magic_mut().refund_magic(cost, clamp_full);
    }

    pub(in crate::zelda_rtl) fn link_item_reset_from_overworld_things(&mut self) {
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_y_button_action_flags(0);
        self.follower_link_state_mut()
            .clear_state_item_and_grab_flags();
        self.follower_link_state_mut().clear_direction_lock_bits(1);
    }

    pub(in crate::zelda_rtl) fn link_item_cape(&mut self) {
        if !self.game_state.player.follower_link.is_cape_active() {
            if (self.follower_link_state_mut().tick_cape_transform_timer() as i8) >= 0 {
                self.follower_link_state_mut().clear_direction_flags(0x0f);
                self.halt_link_when_using_items();
                return;
            }

            self.follower_link_state_mut().clear_cape_transform_timer();
            if self.game_state.player.follower_link.doorway_state() != 0
                || !self.check_y_button_press()
            {
                return;
            }
            self.follower_link_state_mut()
                .clear_button_mask_b_y_bits(0x40);
            if self.game_state.inventory.player_resources.magic.amount() == 0 {
                self.ancilla_sfx2_near(60);
                self.dialogue_message_index_mut().set_value(123);
                self.main_show_text_message();
                return;
            }

            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().set_cape_mode(1);
            let cape_timer =
                LINK_ITEM_CAPE_CAPE_DEPLETION_TIMERS[self.magic_consumption_level() as usize];
            self.follower_link_state_mut()
                .set_cape_decrement_counter(cape_timer);
            self.follower_link_state_mut().set_cape_transform_timer(20);
            self.ancilla_add_cape_poof(35, 4);
            self.ancilla_sfx2_near(20);
            return;
        }

        self.follower_link_state_mut()
            .set_sprite_damage_disable_timer(1);
        self.halt_link_when_using_items();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        self.follower_link_state_mut()
            .decrement_cape_decrement_counter();
        if self
            .game_state
            .player
            .follower_link
            .cape_decrement_counter()
            == 0
        {
            let cape_timer =
                LINK_ITEM_CAPE_CAPE_DEPLETION_TIMERS[self.magic_consumption_level() as usize];
            self.follower_link_state_mut()
                .set_cape_decrement_counter(cape_timer);
            if self.game_state.inventory.player_resources.magic.amount() == 0
                && self
                    .game_state
                    .enhanced_features
                    .has(LINK_ITEM_CAPE_FEATURES0_MISC_BUG_FIXES)
            {
                self.link_force_unequip_cape();
                return;
            }
            if self.player_magic_mut().decrement_magic_power() == 0 {
                self.link_force_unequip_cape();
                return;
            }
        }

        if (self.follower_link_state_mut().tick_cape_transform_timer() as i8) < 0 {
            self.follower_link_state_mut().clear_cape_transform_timer();
            if self.game_state.player.follower_link.filtered_joypad_h() & 0x40 != 0 {
                self.link_force_unequip_cape();
            }
        }
    }

    pub(in crate::zelda_rtl) fn link_item_rod(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if self.game_state.player.follower_link.doorway_state() != 0
                || !self.check_y_button_press()
            {
                return;
            }
            if !self.link_check_magic_cost(0) {
                self.follower_link_state_mut()
                    .clear_button_mask_b_y_bits(0x40);
                return;
            }
            self.follower_link_state_mut()
                .set_item_action_debug_value_2(1);
            if self.game_state.player.follower_link.selected_rod() == 1 {
                self.ancilla_add_fire_rod_shot(2, 1);
            } else {
                self.ancilla_add_ice_rod_shot(11, 1);
            }
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_ROD_ROD_ANIM_DELAYS[0]);
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().set_item_in_hand(1);
        }
        self.halt_link_when_using_items();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }
        self.follower_link_state_mut()
            .increment_action_handler_timer();
        let step = self.game_state.player.follower_link.action_handler_timer() as usize;
        if step < LINK_ITEM_ROD_ROD_ANIM_DELAYS.len() {
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_ROD_ROD_ANIM_DELAYS[step]);
            return;
        }
        self.follower_link_state_mut()
            .set_item_action_debug_value_2(0);
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut().clear_action_handler_timer();
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(0);
        self.follower_link_state_mut().clear_item_in_hand_bits(1);
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
    }

    pub(in crate::zelda_rtl) fn link_item_hammer(&mut self) {
        if self.game_state.player.follower_link.item_in_hand_has(0x10) {
            return;
        }
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if self.game_state.player.follower_link.doorway_state() != 0
                || self.game_state.player.follower_link.filtered_joypad_h() & 0x40 == 0
            {
                return;
            }
            self.follower_link_state_mut()
                .add_button_mask_b_y_bits(0x40);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_HAMMER_HAMMER_ANIM_DELAYS[0]);
            self.follower_link_state_mut().set_direction_lock_bits(1);
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().set_item_in_hand(2);
        }
        self.halt_link_when_using_items();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }
        self.follower_link_state_mut()
            .increment_action_handler_timer();
        let step = self.game_state.player.follower_link.action_handler_timer() as usize;
        self.follower_link_state_mut().set_spin_attack_delay_timer(
            LINK_ITEM_HAMMER_HAMMER_ANIM_DELAYS
                [step.min(LINK_ITEM_HAMMER_HAMMER_ANIM_DELAYS.len() - 1)],
        );
        if self.game_state.player.follower_link.action_handler_timer() == 1 {
            self.tile_detect_main_handler(3);
            self.ancilla_add_hit_stars(22, 0);
            if self.game_state.system_signals.sound_effect_1() == 0 {
                self.ancilla_sfx2_near(16);
                self.spawn_hammer_water_splash();
            }
        } else if self.game_state.player.follower_link.action_handler_timer() == 3 {
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(0);
            self.follower_link_state_mut()
                .clear_button_mask_b_y_bits(0x40);
            self.follower_link_state_mut().clear_direction_lock_bits(1);
            self.follower_link_state_mut().clear_item_in_hand_bits(2);
        }
    }

    pub(in crate::zelda_rtl) fn link_item_bow(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if self.game_state.player.follower_link.doorway_state() != 0
                || !self.check_y_button_press()
            {
                return;
            }
            self.follower_link_state_mut().set_direction_lock_bits(1);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_BOW_BOW_DELAYS[0]);
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().set_item_in_hand(16);
        }
        self.halt_link_when_using_items();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }
        self.follower_link_state_mut()
            .increment_action_handler_timer();
        let step = self.game_state.player.follower_link.action_handler_timer() as usize;
        if step < LINK_ITEM_BOW_BOW_DELAYS.len() {
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_BOW_BOW_DELAYS[step]);
            return;
        }

        let k = self.ancilla_add_arrow(
            9,
            self.game_state.player.follower_link.facing(),
            2,
            self.game_state.player.follower_link.x(),
            self.game_state.player.follower_link.y(),
        );
        if k >= 0 {
            let k = k as usize;
            if self.game_state.archery_game.arrows_left() != 0 {
                self.archery_game_mut().decrement_arrows_left();
                self.player_resources_mut().increment_arrows_by(2);
            }
            if self.game_state.archery_game.out_of_arrows() == 0
                && self.game_state.inventory.player_resources.arrows() != 0
            {
                if self.player_resources_mut().decrement_arrows() == 0 {
                    self.hud_refresh_icon();
                }
            } else {
                self.ancilla_slot_view_mut(k).set_ancilla_type(0);
                self.ancilla_sfx2_near(60);
            }
        }

        self.follower_link_state_mut().clear_action_handler_timer();
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(0);
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        self.follower_link_state_mut().clear_direction_lock_bits(1);
        self.follower_link_state_mut().clear_item_in_hand_bits(0x10);
        if self.game_state.player.follower_link.button_b_frames() >= 9 {
            self.follower_link_state_mut().set_button_b_frames(9);
        }
    }

    pub(in crate::zelda_rtl) fn link_item_boomerang(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if self.game_state.player.follower_link.doorway_state() != 0
                || !self.check_y_button_press()
                || self.game_state.minigame.flag_boomerang_in_place() != 0
            {
                return;
            }
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().set_item_in_hand(0x80);
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(7);
            let s0 = self.ancilla_add_boomerang(5, 0);
            if self.game_state.player.follower_link.button_b_frames() >= 9 {
                self.link_reset_boomerang_y_stuff();
                return;
            }
            if s0 == 0 {
                let last_direction = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
                self.follower_link_state_mut()
                    .set_last_direction(last_direction);
            } else {
                self.follower_link_state_mut().set_direction_lock_bits(1);
            }
        } else {
            self.follower_link_state_mut().set_direction_lock_bits(1);
        }

        if self.game_state.player.follower_link.has_item_in_hand() {
            self.halt_link_when_using_items();
            self.follower_link_state_mut().clear_direction_flags(0x0f);
            if (self
                .follower_link_state_mut()
                .decrement_spin_attack_delay_timer() as i8)
                >= 0
            {
                return;
            }
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(5);
            self.follower_link_state_mut()
                .increment_action_handler_timer();
            if self.game_state.player.follower_link.action_handler_timer() != 2 {
                return;
            }
        }
        self.link_reset_boomerang_y_stuff();
    }

    pub(super) fn link_reset_boomerang_y_stuff(&mut self) {
        self.follower_link_state_mut().clear_item_in_hand();
        self.follower_link_state_mut().clear_action_handler_timer();
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(0);
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        if self.game_state.player.follower_link.button_mask_b_y() & 0x80 == 0 {
            self.follower_link_state_mut().clear_direction_lock_bits(1);
        }
    }

    pub(in crate::zelda_rtl) fn link_item_bombs(&mut self) {
        if self.game_state.player.follower_link.doorway_state() != 0
            || self.game_state.sprites.follower_runtime.indicator() == 13
            || !self.check_y_button_press()
        {
            return;
        }
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        let limit = if self
            .game_state
            .enhanced_features
            .has(LINK_ITEM_BOMBS_FEATURES0_MORE_ACTIVE_BOMBS)
        {
            3
        } else {
            1
        };
        self.ancilla_add_bomb(7, limit);
        self.follower_link_state_mut().clear_item_in_hand();
    }

    pub(in crate::zelda_rtl) fn link_item_book(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 != 0
            || self.game_state.player.follower_link.doorway_state() != 0
            || !self.check_y_button_press()
        {
            return;
        }
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        if self
            .game_state
            .player
            .follower_link
            .item_pickup_in_progress()
        {
            self.link_perform_desert_prayer();
        } else {
            self.ancilla_sfx2_near(60);
        }
    }

    pub(super) fn link_item_bottle(&mut self) {
        if !self.check_y_button_press() {
            return;
        }
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        let btidx = self
            .game_state
            .inventory
            .player_resources
            .equipped_bottle_index()
            .wrapping_sub(1) as usize;
        if btidx >= 4 {
            return;
        }
        let bottle = self.game_state.inventory.items.bottle(btidx);
        if bottle == 0 {
            return;
        }
        if bottle < 3 {
            self.ancilla_sfx2_near(60);
        } else if bottle == 3 {
            if self.game_state.inventory.player_resources.health_capacity()
                == self.game_state.inventory.player_resources.current_health()
            {
                self.ancilla_sfx2_near(60);
                return;
            }
            let value = 2;
            self.inventory_items_mut().set_bottle(btidx, value);
            self.follower_link_state_mut().clear_item_in_hand();
            let main_module = self.game_state.frame.main_module;
            self.set_submodule(4);
            self.set_saved_module_for_menu(main_module);
            self.set_main_module(14);
            self.set_heart_refill_countdown(7);
            self.hud_rebuild();
        } else if bottle == 4 {
            if self.game_state.inventory.player_resources.magic.amount() == 128 {
                self.ancilla_sfx2_near(60);
                return;
            }
            let value = 2;
            self.inventory_items_mut().set_bottle(btidx, value);
            self.follower_link_state_mut().clear_item_in_hand();
            let main_module = self.game_state.frame.main_module;
            self.set_submodule(8);
            self.set_saved_module_for_menu(main_module);
            self.set_main_module(14);
            self.set_heart_refill_countdown(7);
            self.hud_rebuild();
        } else if bottle == 5 {
            if self.game_state.inventory.player_resources.health_capacity()
                == self.game_state.inventory.player_resources.current_health()
                && self.game_state.inventory.player_resources.magic.amount() == 128
            {
                self.ancilla_sfx2_near(60);
                return;
            }
            let value = 2;
            self.inventory_items_mut().set_bottle(btidx, value);
            self.follower_link_state_mut().clear_item_in_hand();
            let main_module = self.game_state.frame.main_module;
            self.set_submodule(9);
            self.set_saved_module_for_menu(main_module);
            self.set_main_module(14);
            self.set_heart_refill_countdown(7);
            self.hud_rebuild();
        } else if bottle == 6 {
            self.follower_link_state_mut().clear_item_in_hand();
            if self.release_fairy() < 0 {
                self.ancilla_sfx2_near(60);
                return;
            }
            let value = 2;
            self.inventory_items_mut().set_bottle(btidx, value);
            self.hud_rebuild();
        } else if bottle == 7 || bottle == 8 {
            if self.release_bee_from_bottle(btidx) == 0 {
                self.ancilla_sfx2_near(60);
                return;
            }
            let value = 2;
            self.inventory_items_mut().set_bottle(btidx, value);
            self.hud_rebuild();
        }
    }

    pub(super) fn link_perform_desert_prayer(&mut self) {
        let main_module = self.game_state.frame.main_module;
        self.set_submodule(5);
        self.set_saved_module_for_menu(main_module);
        self.set_main_module(14);
        self.set_modal_pause_flag(1);
        self.follower_link_state_mut().set_y_button_action_timer(22);
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_state_bits(2);
        self.follower_link_state_mut().set_direction_lock_bits(1);
        self.follower_link_state_mut().clear_animation_step();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        self.set_ambient_sound_effect(17);
        self.set_music_control(242);
    }

    pub(in crate::zelda_rtl) fn link_item_lamp(&mut self) {
        if self.game_state.player.follower_link.doorway_state() != 0 || !self.check_y_button_press()
        {
            return;
        }
        if self.game_state.inventory.items.torch() != 0 && self.link_check_magic_cost(6) {
            self.ancilla_add_magic_powder(0x1a, 0);
            self.dungeon_light_torch();
            self.ancilla_add_lamp_flame(0x2f, 2);
        }
        self.finish_lamp_item();
    }

    pub(super) fn link_item_lamp_after_native_magic_entry(&mut self) {
        if self.link_check_magic_cost(6) {
            self.ancilla_add_magic_powder(0x1a, 0);
            self.dungeon_light_torch();
            self.ancilla_add_lamp_flame(0x2f, 2);
        }
        self.finish_lamp_item();
    }

    fn finish_lamp_item(&mut self) {
        self.follower_link_state_mut().clear_item_in_hand();
        self.follower_link_state_mut().set_button_mask_b_y(0);
        self.follower_link_state_mut().set_button_b_frames(0);
        self.follower_link_state_mut().clear_direction_lock();
    }

    pub(in crate::zelda_rtl) fn link_item_powder(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if self.game_state.player.follower_link.doorway_state() != 0
                || !self.check_y_button_press()
            {
                return;
            }
            if self.game_state.inventory.items.mushroom() != 2 {
                self.ancilla_sfx2_near(60);
                self.finish_powder_item();
                return;
            }
            if !self.link_check_magic_cost(2) {
                self.finish_powder_item();
                return;
            }
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_POWDER_MUSHROOM_TIMER[0]);
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().clear_direction_flags(0x0f);
            self.follower_link_state_mut().set_item_in_hand(0x40);
        }

        self.follower_link_state_mut()
            .clear_movement_velocity_and_direction();
        self.follower_link_state_mut().clear_movement_subpixels();
        self.follower_link_state_mut()
            .clear_moving_against_diag_tile();
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.follower_link_state_mut()
            .increment_action_handler_timer();
        let step = self.game_state.player.follower_link.action_handler_timer() as usize;
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(LINK_ITEM_POWDER_MUSHROOM_TIMER[step]);
        if self.game_state.player.follower_link.action_handler_timer() == 4 {
            self.ancilla_add_magic_powder(26, 0);
        }
        if self.game_state.player.follower_link.action_handler_timer() == 9 {
            if self.game_state.frame.submodule == 0 {
                self.tile_detect_main_handler(1);
            }
            self.finish_powder_item();
        }
    }

    pub(in crate::zelda_rtl) fn link_item_shovel_and_flute(&mut self) {
        if self.game_state.inventory.items.flute() == 1 {
            self.link_item_shovel();
        } else if self.game_state.inventory.items.flute() != 0 {
            self.link_item_flute();
        }
    }

    pub(in crate::zelda_rtl) fn link_item_shovel(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if self.game_state.player.follower_link.doorway_state() != 0
                || !self.check_y_button_press()
            {
                return;
            }
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_SHOVEL_SHOVEL_ANIM_DELAY[0]);
            self.follower_link_state_mut().clear_item_action_step_var();
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().set_position_mode(1);
            self.follower_link_state_mut().set_direction_lock_bits(1);
            self.follower_link_state_mut().clear_animation_step();
        }
        self.halt_link_when_using_items();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }
        let step = self
            .follower_link_state_mut()
            .increment_item_action_step_var() as usize;
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(LINK_ITEM_SHOVEL_SHOVEL_ANIM_DELAY[step]);
        self.follower_link_state_mut()
            .set_action_handler_timer(LINK_ITEM_SHOVEL_SHOVEL_ANIM_DELAY2[step]);

        if self.game_state.player.follower_link.action_handler_timer() == 1 {
            self.tile_detect_main_handler(2);
            if self.game_state.world.transient.overworld_hole_tilemap_pos() != 0 {
                self.ancilla_sfx3_near(27);
                self.ancilla_add_dug_up_flute(54, 0);
            }
            if (self.game_state.player.tile_detection.thick_grass()
                | self
                    .game_state
                    .player
                    .tile_detection
                    .destruction_aftermath())
                & 1
                == 0
            {
                self.ancilla_add_hit_stars(22, 0);
                self.ancilla_sfx2_near(5);
            } else {
                self.ancilla_add_shovel_dirt(23, 0);
                if self.game_state.minigame.is_archer_or_shovel_game() != 0 {
                    self.digging_game_guy_attempt_prize_spawn();
                }
                self.ancilla_sfx2_near(18);
            }
        }

        if self.game_state.player.follower_link.item_action_step_var() == 3 {
            self.follower_link_state_mut().clear_item_action_step_var();
            self.follower_link_state_mut().clear_action_handler_timer();
            let button_mask_b_y = self.game_state.player.follower_link.button_mask_b_y() & 0x80;
            self.follower_link_state_mut()
                .set_button_mask_b_y(button_mask_b_y);
            self.follower_link_state_mut().clear_position_mode();
            self.follower_link_state_mut().clear_direction_lock_bits(1);
        }
    }

    pub(in crate::zelda_rtl) fn link_item_flute(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 != 0 {
            self.follower_link_state_mut().decrement_flute_countdown();
            if self.game_state.player.follower_link.flute_countdown() != 0 {
                return;
            }
            self.follower_link_state_mut()
                .clear_button_mask_b_y_bits(0x40);
        }
        if !self.check_y_button_press() {
            return;
        }
        self.follower_link_state_mut().set_flute_countdown(128);
        self.ancilla_sfx2_near(19);
        if self.game_state.world.location.is_indoors()
            || u16::from(self.game_state.world.location.overworld_screen_index()) & 0x40 != 0
            || self.game_state.frame.main_module == 11
        {
            return;
        }
        if (0..5).any(|i| self.ancilla_slot_view(i).ancilla_type() == 0x27) {
            return;
        }
        if self.game_state.inventory.items.flute() == 2 {
            let screen = u16::from(self.game_state.world.location.overworld_screen_index());
            let y = self.game_state.player.follower_link.y();
            let x = self.game_state.player.follower_link.x();
            if screen == 0x18 && (0x760..0x7e0).contains(&y) && (0x1cf..0x230).contains(&x) {
                self.set_submodule(45);
                self.ancilla_add_exploding_weather_vane(55, 0);
            }
        } else {
            self.ancilla_add_duck_take_off(39, 4);
            self.follower_link_state_mut()
                .clear_pull_for_rupees_sprite_need();
        }
    }

    pub(super) fn link_handle_y_item(&mut self) {
        if self.game_state.player.follower_link.button_b_frames() != 0
            && self.game_state.player.follower_link.button_b_frames() < 9
        {
            return;
        }

        let mut item = self.game_state.player.follower_link.current_item_y();
        if self.game_state.player.follower_link.is_bunny_mirror() && item != 11 && item != 20 {
            return;
        }

        if self.game_state.minigame.is_archer_or_shovel_game() != 0
            && !self.game_state.player.follower_link.is_bunny_mirror()
        {
            if self.game_state.minigame.is_archer_or_shovel_game() == 2 {
                self.link_item_bow();
            } else {
                self.link_item_shovel();
            }
            return;
        }

        let old_down = self.game_state.player.follower_link.joypad1h_last();
        let old_pressed = self.game_state.player.follower_link.filtered_joypad_h();
        let old_bottle = self
            .game_state
            .inventory
            .player_resources
            .equipped_bottle_index();
        if !self.game_state.player.follower_link.has_item_in_hand()
            && !self.game_state.player.follower_link.has_position_mode()
            && old_down & 0x40 == 0
        {
            let btn_index = self.get_current_item_button_index();
            if btn_index != 0 {
                let hud_item = self
                    .game_state
                    .inventory
                    .save_progress
                    .hud_current_item_slot(btn_index);
                if hud_item != 0 {
                    if hud_item >= 21 {
                        self.player_resources_mut()
                            .set_equipped_bottle_index(hud_item - 20);
                    }
                    item = self.hud_lookup_inventory_item(hud_item);
                    self.follower_link_state_mut()
                        .set_joypad1h_last(old_down | 0x40);
                    if self.game_state.player.follower_link.filtered_joypad_l()
                        & LINK_ITEM_Y_BUTTON_BUTTON_INDEX_KEYS[btn_index]
                        != 0
                    {
                        self.follower_link_state_mut()
                            .set_filtered_joypad_h(old_pressed | 0x40);
                    }
                }
            }
        }

        if item != self.game_state.player.follower_link.current_item_active() {
            if self.game_state.player.follower_link.current_item_active() == 8
                && self.game_state.inventory.items.flute() & 2 != 0
            {
                self.follower_link_state_mut()
                    .clear_button_mask_b_y_bits(0x40);
            }
            if self.game_state.player.follower_link.current_item_active() == 19
                && self.game_state.player.follower_link.is_cape_active()
            {
                self.link_force_unequip_cape();
            }
        }

        self.link_select_and_dispatch_y_item(item);
        self.follower_link_state_mut().set_joypad1h_last(old_down);
        self.follower_link_state_mut()
            .set_filtered_joypad_h(old_pressed);
        self.player_resources_mut()
            .set_equipped_bottle_index(old_bottle);
    }

    pub(super) fn link_select_and_dispatch_y_item(&mut self, item: u8) {
        if !self.game_state.player.follower_link.has_item_in_hand()
            && !self.game_state.player.follower_link.has_position_mode()
        {
            self.follower_link_state_mut().set_current_item_active(item);
        }
        if matches!(
            self.game_state.player.follower_link.current_item_active(),
            5 | 6
        ) {
            let rod = self.game_state.player.follower_link.current_item_active() - 4;
            self.follower_link_state_mut().set_selected_rod(rod);
        }

        self.link_dispatch_y_item(self.game_state.player.follower_link.current_item_active());
    }

    pub(super) fn link_dispatch_y_item(&mut self, item: u8) {
        match item {
            0 => {}
            1 => self.link_item_bombs(),
            2 => self.link_item_boomerang(),
            3 => self.link_item_bow(),
            4 => self.link_item_hammer(),
            5 | 6 => self.link_item_rod(),
            7 => self.link_item_net(),
            8 => self.link_item_shovel_and_flute(),
            9 => self.link_item_lamp(),
            10 => self.link_item_powder(),
            11 => self.link_item_bottle(),
            12 => self.link_item_book(),
            13 => self.link_item_cane_of_byrna(),
            14 => self.link_item_hookshot(),
            15 => self.link_item_bombos(),
            16 => self.link_item_ether(),
            17 => self.link_item_quake(),
            18 => self.link_item_cane_of_somaria(),
            19 => self.link_item_cape(),
            20 => self.link_item_mirror(),
            21 => self.link_item_shovel(),
            // C Link_HandleYItem asserts outside item slots 0..=21.
            _ => panic!("Link_HandleYItem current_item_active {}", item),
        }
    }

    pub(in crate::zelda_rtl) fn link_item_ether(&mut self) {
        self.start_medallion_item(
            8,
            LINK_ITEM_ETHER_ETHER_ANIM_DELAYS[0],
            LINK_ITEM_ETHER_ETHER_ANIM_STATES[0],
            None,
        );
    }

    pub(in crate::zelda_rtl) fn link_item_bombos(&mut self) {
        self.start_medallion_item(
            9,
            LINK_ITEM_BOMBOS_BOMBOS_ANIM_DELAYS[0],
            LINK_ITEM_BOMBOS_BOMBOS_ANIM_STATES[0],
            None,
        );
    }

    pub(in crate::zelda_rtl) fn link_item_quake(&mut self) {
        self.start_medallion_item(
            10,
            LINK_ITEM_QUAKE_QUAKE_ANIM_DELAYS[0],
            LINK_ITEM_QUAKE_QUAKE_ANIM_STATES[0],
            Some(()),
        );
    }

    pub(in crate::zelda_rtl) fn link_state_using_ether(&mut self) {
        self.increment_modal_pause_flag();
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.follower_link_state_mut()
            .increment_spin_animation_step_counter();
        if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 4
        {
            self.ancilla_sfx3_near(35);
        } else if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 9
        {
            self.ancilla_sfx2_near(44);
        } else if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 12
        {
            self.follower_link_state_mut()
                .set_spin_animation_step_counter(10);
        }

        let step = self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter() as usize;
        let delays = if self
            .game_state
            .enhanced_features
            .has(LINK_STATE_USING_ETHER_FEATURES0_DIM_FLASHES)
        {
            LINK_STATE_USING_ETHER_ETHER_ANIM_DELAYS_NO_FLASH
        } else {
            LINK_STATE_USING_ETHER_ETHER_ANIM_DELAYS
        };
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(delays[step]);
        self.follower_link_state_mut()
            .set_state_for_spin_attack(LINK_STATE_USING_ETHER_ETHER_ANIM_STATES[step]);
        if self
            .game_state
            .player
            .follower_link
            .spin_attack_sound_latch()
            == 0
            && self
                .game_state
                .player
                .follower_link
                .spin_animation_step_counter()
                == 10
        {
            self.follower_link_state_mut()
                .set_spin_attack_sound_latch(1);
            self.ancilla_add_ether_spell(24, 0);
            self.follower_link_state_mut().clear_auxiliary_state();
            self.follower_link_state_mut().set_incapacitated_timer(0);
        }
    }

    pub(super) fn link_state_using_bombos(&mut self) {
        self.increment_modal_pause_flag();
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.follower_link_state_mut()
            .increment_spin_animation_step_counter();
        if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 4
        {
            self.ancilla_sfx3_near(35);
        } else if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 10
        {
            self.ancilla_sfx2_near(44);
        } else if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 20
        {
            self.follower_link_state_mut()
                .set_spin_animation_step_counter(19);
        }
        let step = self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter() as usize;
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(LINK_STATE_USING_BOMBOS_BOMBOS_ANIM_DELAYS[step]);
        self.follower_link_state_mut()
            .set_state_for_spin_attack(LINK_STATE_USING_BOMBOS_BOMBOS_ANIM_STATES[step]);
        if self
            .game_state
            .player
            .follower_link
            .spin_attack_sound_latch()
            == 0
            && self
                .game_state
                .player
                .follower_link
                .spin_animation_step_counter()
                == 19
        {
            self.follower_link_state_mut()
                .set_spin_attack_sound_latch(1);
            self.ancilla_add_bombos_spell(25, 0);
            self.follower_link_state_mut().clear_auxiliary_state();
            self.follower_link_state_mut().set_incapacitated_timer(0);
        }
    }

    pub(super) fn link_state_using_quake(&mut self) {
        self.increment_modal_pause_flag();
        self.follower_link_state_mut().clear_actual_velocity_xy();

        if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 10
        {
            self.follower_link_state_mut()
                .restore_actual_z_velocity_from_mirror();
            self.follower_link_state_mut().restore_z_low_from_mirror();
            self.follower_link_state_mut().set_auxiliary_state(2);
            self.player_change_z(2);
            self.link_move_position();
            self.follower_link_state_mut()
                .cache_actual_z_velocity_to_mirror();
            self.follower_link_state_mut().cache_z_low_to_mirror();
            if !self.game_state.player.follower_link.is_z_low_negative() {
                let spin_state =
                    if (self.game_state.player.follower_link.actual_z_velocity() as i8) < 0 {
                        21
                    } else {
                        20
                    };
                self.follower_link_state_mut()
                    .set_state_for_spin_attack(spin_state);
                return;
            }
        } else {
            if (self
                .follower_link_state_mut()
                .decrement_spin_attack_delay_timer() as i8)
                >= 0
            {
                return;
            }
        }

        self.follower_link_state_mut()
            .increment_spin_animation_step_counter();
        if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 4
        {
            self.ancilla_sfx3_near(35);
        } else if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 10
        {
            self.ancilla_sfx2_near(44);
        } else if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 11
        {
            self.ancilla_sfx2_near(12);
        } else if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 12
        {
            self.follower_link_state_mut()
                .set_spin_animation_step_counter(11);
        }
        let step = self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter() as usize;
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(LINK_STATE_USING_QUAKE_QUAKE_ANIM_DELAYS[step]);
        self.follower_link_state_mut()
            .set_state_for_spin_attack(LINK_STATE_USING_QUAKE_QUAKE_ANIM_STATES[step]);
        if self
            .game_state
            .player
            .follower_link
            .spin_attack_sound_latch()
            == 0
            && self
                .game_state
                .player
                .follower_link
                .spin_animation_step_counter()
                == 11
        {
            self.follower_link_state_mut()
                .set_spin_attack_sound_latch(1);
            self.ancilla_add_quake_spell(28, 0);
            self.follower_link_state_mut().clear_auxiliary_state();
            self.follower_link_state_mut().set_incapacitated_timer(0);
        }
    }

    pub(in crate::zelda_rtl) fn link_item_mirror(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if !self.check_y_button_press() {
                return;
            }
            if self.game_state.sprites.follower_runtime.indicator() == 10 {
                self.dialogue_message_index_mut().set_value(289);
                self.main_show_text_message();
                return;
            }
        }
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);

        if self.game_state.player.follower_link.doorway_state() != 0
            || (self
                .game_state
                .player
                .follower_link
                .cheat_walk_through_walls()
                == 0
                && !self
                    .game_state
                    .enhanced_features
                    .has(LINK_ITEM_MIRROR_FEATURES0_MIRROR_TO_DARKWORLD)
                && self.game_state.world.location.is_outdoors()
                && u16::from(self.game_state.world.location.overworld_screen_index()) & 0x40 == 0)
        {
            self.ancilla_sfx2_near(60);
            return;
        }

        self.do_sword_interaction_with_tiles_mirror();
    }

    pub(super) fn do_sword_interaction_with_tiles_mirror(&mut self) {
        if self.game_state.world.location.is_indoors() {
            if self.game_state.player.follower_link.is_menu_blocked() {
                return;
            }
            self.mirror_save_room_data();
            if self.game_state.system_signals.sound_effect_1() != 60 {
                self.dungeon_object_tracking_mut()
                    .clear_changeable_object_index(0);
                self.dungeon_object_tracking_mut()
                    .clear_changeable_object_index(1);
            }
            return;
        }
        if self.game_state.frame.main_module == 11 {
            return;
        }
        let screen = u16::from(self.game_state.world.location.overworld_screen_index());
        self.world_palette_theme_mut()
            .set_last_light_vs_dark_world((screen & 0x40) as u8);
        if self
            .game_state
            .world
            .palette_theme
            .last_light_vs_dark_world()
            != 0
        {
            let y = self.game_state.player.follower_link.y();
            let x = self.game_state.player.follower_link.x();
            self.set_bird_travel_destination(15, x, y);
        }
        self.set_submodule(35);
        self.follower_link_state_mut()
            .clear_pull_for_rupees_sprite_need();
        self.follower_link_state_mut().set_whirlpool_trigger();
        self.set_subsubmodule(0);
        self.follower_link_state_mut().clear_actual_velocity_xy();
        self.follower_link_state_mut().set_handler_state(20);
    }

    pub(in crate::zelda_rtl) fn link_state_crossing_worlds(&mut self) {
        self.link_reset_properties_b();
        self.tile_check_for_mirror_bonk();
        let world_changed =
            (u16::from(self.game_state.world.location.overworld_screen_index()) as u8 & 0x40)
                != self
                    .game_state
                    .world
                    .palette_theme
                    .last_light_vs_dark_world();
        let bonk_bits = self.game_state.player.tile_detection.bonk_bits_low();
        if world_changed && bonk_bits & 0x0c != 0 && Self::bit_sum4(bonk_bits) >= 2 {
            self.start_mirror_transition(44);
            return;
        }

        if Self::bit_sum4(self.game_state.player.tile_detection.deepwater() as u8) >= 2 {
            if self.game_state.inventory.items.has_flippers() {
                self.link_set_to_deep_water();
                self.follower_link_state_mut().set_handler_state(4);
                self.link_force_unequip_cape_quietly();
                return;
            }
            if world_changed {
                self.start_mirror_transition(44);
                return;
            }
            self.check_ability_to_swim();
        }

        if self.game_state.player.follower_link.is_in_deep_water() {
            self.follower_link_state_mut().clear_deep_water_state();
            self.follower_link_state_mut()
                .set_last_direction_from_swim_flags();
        }
        self.follower_link_state_mut().set_dash_countdown(0);
        self.follower_link_state_mut().clear_running();
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut().set_button_mask_b_y(0);
        self.follower_link_state_mut().set_button_b_frames(0);
        self.follower_link_state_mut().clear_direction_lock();
        self.swim_acceleration_mut().set_mode(0, 0);
        self.follower_link_state_mut().set_actual_y_velocity(0);
        if world_changed {
            self.memorized_tile_mut().set_count(0);
        }
        let handler_state = if self.game_state.inventory.items.has_moon_pearl()
            || u16::from(self.game_state.world.location.overworld_screen_index()) & 0x40 == 0
        {
            0
        } else {
            23
        };
        self.follower_link_state_mut()
            .set_handler_state(handler_state);
    }

    pub(in crate::zelda_rtl) fn handle_followers_after_mirroring(&mut self) {
        self.tile_detect_main_handler(0);
        self.follower_link_state_mut().clear_animation_step();
        match self.game_state.sprites.follower_runtime.indicator() {
            12 | 13 => {
                if self.game_state.sprites.follower_runtime.indicator() == 13 {
                    self.set_super_bomb_indicator_timer(0xfe);
                    self.set_super_bomb_indicator_counter(0);
                }
                if self.game_state.sprites.follower_runtime.dropped() != 0 {
                    self.follower_state_mut().set_dropped(0);
                    self.follower_state_mut().set_indicator(0);
                }
            }
            9 | 10 => self.follower_state_mut().set_indicator(0),
            7 | 8 => {
                self.follower_state_mut().xor_indicator(7 ^ 8);
                self.load_follower_graphics();
                self.ancilla_add_dwarf_poof(0x40, 4);
            }
            _ => {}
        }

        if !self.game_state.inventory.items.has_moon_pearl() {
            self.ancilla_add_bunny_poof(0x23, 4);
            self.link_force_unequip_cape_quietly();
            self.follower_link_state_mut().clear_cape_transform_timer();
        } else if self.game_state.player.follower_link.is_cape_active() {
            self.link_force_unequip_cape();
            self.follower_link_state_mut().clear_cape_transform_timer();
        }
    }

    pub(in crate::zelda_rtl) fn link_item_hookshot(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 != 0
            || self.game_state.player.follower_link.doorway_state() != 0
            || self.game_state.player.follower_link.defense_flags() & 2 != 0
            || !self.check_y_button_press()
        {
            return;
        }

        self.reset_all_acceleration();
        self.follower_link_state_mut().clear_action_handler_timer();
        self.follower_link_state_mut().set_direction_lock_bits(1);
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(7);
        self.follower_link_state_mut().clear_animation_step();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        self.follower_link_state_mut().set_position_mode(4);
        self.follower_link_state_mut().set_handler_state(19);
        self.follower_link_state_mut()
            .set_sprite_damage_disable_timer(1);
        self.ancilla_add_hookshot(0x1f, 3);
    }

    pub(in crate::zelda_rtl) fn link_state_hookshotting(&mut self) {
        self.follower_link_state_mut().clear_given_damage();
        self.follower_link_state_mut().clear_auxiliary_state();
        self.follower_link_state_mut().set_incapacitated_timer(0);
        let hookshot = (0..=4)
            .rev()
            .find(|&i| self.ancilla_slot_view(i).ancilla_type() == 0x1f);
        let Some(_k) = hookshot else {
            if (self
                .follower_link_state_mut()
                .decrement_spin_attack_delay_timer() as i8)
                >= 0
            {
                return;
            }
            self.finish_hookshot_state_after_missing_ancilla();
            return;
        };

        if self
            .game_state
            .player
            .follower_link
            .spin_attack_delay_timer()
            != 0
            && (self
                .follower_link_state_mut()
                .decrement_spin_attack_delay_timer() as i8)
                < 0
        {
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(0);
        }

        if !self
            .game_state
            .player
            .follower_link
            .has_hookshot_interlock()
        {
            self.follower_link_state_mut()
                .store_safe_return_low_from_current();
            self.follower_link_state_mut().set_y_velocity(0);
            self.follower_link_state_mut().set_x_velocity(0);
            self.link_handle_cardinal_collision();
            return;
        }

        self.follower_link_state_mut()
            .clear_somaria_platform_state();

        let hei = self.game_state.messaging.runtime.effect_index() as usize;
        let item_to_link = self.ancilla_slot_view(hei).item_to_link().wrapping_sub(1);
        self.ancilla_slot_view_mut(hei)
            .set_item_to_link(item_to_link);
        if (item_to_link as i8) < 0 {
            self.ancilla_slot_view_mut(hei).set_item_to_link(0);
        } else {
            let hookshot = self.ancilla_slot_view(hei);
            let dir = hookshot.direction() as usize;
            let x = hookshot.x();
            let y = hookshot.y();
            let target_y = y
                .wrapping_add(LINK_STATE_HOOKSHOTTING_HOOKSHOT_TARGET_Y_OFFSETS[dir] as i16 as u16);
            let target_x = x
                .wrapping_add(LINK_STATE_HOOKSHOTTING_HOOKSHOT_TARGET_X_OFFSETS[dir] as i16 as u16);
            let yd = target_y.wrapping_sub(self.game_state.player.follower_link.y()) as i16;
            let mut actual_y_velocity = 0;
            if yd.wrapping_abs() >= 2 {
                actual_y_velocity = LINK_STATE_HOOKSHOTTING_HOOKSHOT_PULL_Y_VELOCITIES[dir];
            }
            let xd = target_x.wrapping_sub(self.game_state.player.follower_link.x()) as i16;
            let mut actual_x_velocity = 0;
            if xd.wrapping_abs() >= 2 {
                actual_x_velocity = LINK_STATE_HOOKSHOTTING_HOOKSHOT_PULL_X_VELOCITIES[dir];
            }
            self.follower_link_state_mut()
                .set_actual_velocity_xy(actual_x_velocity, actual_y_velocity);
            if (actual_x_velocity | actual_y_velocity) != 0 {
                self.continue_hookshot_drag();
                return;
            }
        }

        self.ancilla_slot_view_mut(hei).set_ancilla_type(0);
        self.follower_state_mut()
            .set_hookshot_release_tail_index_from_tail_write_index();
        self.finish_hookshot_state_without_button_clamp();

        if self.ancilla_slot_view(hei).work_byte_1() != 0 {
            self.follower_link_state_mut()
                .toggle_lower_level_mirror_state();
            self.dungeon_stair_movement_mut().decrement_current_floor();
            if self
                .game_state
                .dungeon
                .stair_movement
                .kind_of_in_room_staircase()
                == 0
            {
                let dungeon_room_index = self.game_state.world.location.dungeon_room_index();
                self.dungeon_room_tracking_mut()
                    .set_room_index2(dungeon_room_index);
                self.increment_dungeon_room_index_by(0x10);
            }
            if self
                .game_state
                .dungeon
                .stair_movement
                .kind_of_in_room_staircase()
                != 2
            {
                self.follower_link_state_mut().toggle_lower_level_state();
            }
            self.dungeon_flag_room_data_quadrants();
        }

        // ROM $07:B876, $07:B88F, $07:B7F5, $07:ACCF
        self.player_tile_detect_nearby_marked();
        if self.game_state.player.tile_detection.deepwater() & 0x0f != 0
            && !self.game_state.player.follower_link.is_in_deep_water()
        {
            self.link_set_to_deep_water();
            self.ancilla_add_splash(21, 0);
            self.follower_link_state_mut().set_handler_state(4);
            self.link_force_unequip_cape_quietly();
            self.follower_link_state_mut()
                .clear_state_item_and_grab_flags();
            self.follower_link_state_mut().set_speed_setting(0);
            if self.game_state.world.location.is_indoors() {
                self.follower_link_state_mut().mark_lower_level();
            }
            if self.game_state.player.follower_link.button_b_frames() >= 9 {
                self.follower_link_state_mut().set_button_b_frames(9);
            }
        } else if self.game_state.player.tile_detection.pit_tile() & 0x0f != 0 {
            self.follower_link_state_mut().set_sprite_oam_state_timer(9);
            self.follower_link_state_mut().begin_pit_check();
            self.follower_link_state_mut().set_handler_state(1);
            if self.game_state.player.follower_link.button_b_frames() >= 9 {
                self.follower_link_state_mut().set_button_b_frames(9);
            }
        } else {
            let y = self.game_state.player.follower_link.y();
            let x = self.game_state.player.follower_link.x();
            self.follower_link_state_mut()
                .store_safe_return_position(x, y);
            self.link_handle_cardinal_collision();
            self.handle_indoor_camera_and_doors();
        }
    }

    fn continue_hookshot_drag(&mut self) {
        self.link_move_position();
        self.tile_detect_main_handler(5);
        if self.game_state.world.location.is_indoors() {
            let x = (self.game_state.player.tile_detection.vertical_ledge() >> 4)
                | self.game_state.player.tile_detection.vertical_ledge()
                | self.game_state.player.tile_detection.horizontal_ledge();
            if x & 1 != 0 {
                self.follower_link_state_mut()
                    .decrement_hookshot_bg_check_off_timer();
                if (self
                    .game_state
                    .player
                    .follower_link
                    .hookshot_bg_check_off_timer() as i8)
                    < 0
                {
                    self.follower_link_state_mut()
                        .set_hookshot_bg_check_off_timer(3);
                    self.follower_link_state_mut().xor_hookshot_interlock(2);
                }
            }
        }
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        if !self
            .game_state
            .player
            .follower_link
            .hookshot_interlock_has(2)
        {
            if self.game_state.player.tile_detection.thick_grass_low() & 1 != 0 {
                self.follower_link_state_mut()
                    .set_water_ripple_or_grass_state(2);
                if !self.link_permission_for_slosh_sounds() {
                    self.ancilla_sfx2_near(26);
                }
            } else if (self.game_state.player.tile_detection.shallow_water_low()
                | self.game_state.player.tile_detection.deepwater() as u8)
                & 1
                != 0
            {
                self.follower_link_state_mut()
                    .increment_water_ripple_or_grass_state();
                self.ancilla_sfx2_near(
                    if self.game_state.world.location.overworld_screen_index() == 0x70 {
                        27
                    } else {
                        28
                    },
                );
            }
        }
        self.handle_indoor_camera_and_doors();
    }

    pub(in crate::zelda_rtl) fn link_item_cane_of_somaria(&mut self) {
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if self
                .game_state
                .player
                .follower_link
                .has_somaria_platform_state()
                || self.game_state.player.follower_link.doorway_state() != 0
                || !self.check_y_button_press()
            {
                return;
            }

            let mut did_charge_magic = false;
            if !(0..5).any(|i| self.ancilla_slot_view(i).ancilla_type() == 0x2c) {
                if !self.link_check_magic_cost(4) {
                    if self
                        .game_state
                        .enhanced_features
                        .has(LINK_ITEM_CANE_OF_SOMARIA_FEATURES0_MISC_BUG_FIXES)
                    {
                        self.follower_link_state_mut()
                            .clear_button_mask_b_y_bits(0x40);
                    }
                    return;
                }
                did_charge_magic = true;
            }

            self.follower_link_state_mut()
                .set_item_action_debug_value_2(1);
            if self.ancilla_add_somaria_block(0x2c, 1).is_none()
                && (did_charge_magic
                    || !self
                        .game_state
                        .enhanced_features
                        .has(LINK_ITEM_CANE_OF_SOMARIA_FEATURES0_MISC_BUG_FIXES))
            {
                self.refund_magic(4);
            }
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_CANE_OF_SOMARIA_ROD_ANIM_DELAYS[0]);
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().clear_item_in_hand();
            self.follower_link_state_mut().set_position_mode_bits(8);
        }

        self.halt_link_when_using_items();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.follower_link_state_mut()
            .increment_action_handler_timer();
        let step = self.game_state.player.follower_link.action_handler_timer() as usize;
        if step < LINK_ITEM_CANE_OF_SOMARIA_ROD_ANIM_DELAYS.len() {
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_CANE_OF_SOMARIA_ROD_ANIM_DELAYS[step]);
            return;
        }
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut().clear_action_handler_timer();
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(0);
        self.follower_link_state_mut()
            .set_item_action_debug_value_2(0);
        self.follower_link_state_mut().clear_position_mode_bits(8);
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
    }

    pub(in crate::zelda_rtl) fn link_item_cane_of_byrna(&mut self) {
        if self.search_for_byrna_spark() {
            return;
        }
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if self.game_state.player.follower_link.doorway_state() != 0
                || !self.check_y_button_press()
            {
                return;
            }
            if !self.link_check_magic_cost(8) {
                self.finish_byrna_item();
                return;
            }
            self.ancilla_add_cane_of_byrna_init_spark(0x30, 0);
            self.follower_link_state_mut()
                .clear_spin_attack_step_counter();
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(LINK_ITEM_CANE_OF_BYRNA_BYRNA_DELAYS[0]);
            self.follower_link_state_mut().clear_item_action_step_var();
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().set_position_mode(8);
            self.follower_link_state_mut().set_direction_lock_bits(1);
            self.follower_link_state_mut().clear_animation_step();
        }

        self.halt_link_when_using_items();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.follower_link_state_mut()
            .increment_action_handler_timer();
        let step = self.game_state.player.follower_link.action_handler_timer() as usize;
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(LINK_ITEM_CANE_OF_BYRNA_BYRNA_DELAYS[step]);
        if self.game_state.player.follower_link.action_handler_timer() == 1 {
            self.ancilla_sfx3_near(42);
        } else if self.game_state.player.follower_link.action_handler_timer() == 3 {
            self.finish_byrna_item();
        }
    }

    pub(in crate::zelda_rtl) fn link_item_net(&mut self) {
        let base = self.game_state.player.follower_link.facing_index() * 10;
        if self.game_state.player.follower_link.button_mask_b_y() & 0x40 == 0 {
            if self.game_state.player.follower_link.doorway_state() != 0
                || !self.check_y_button_press()
            {
                return;
            }
            self.follower_link_state_mut()
                .set_action_handler_timer(LINK_ITEM_NET_BUG_NET_TIMERS[base]);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(3);
            self.follower_link_state_mut().clear_item_action_step_var();
            self.follower_link_state_mut().set_position_mode(16);
            self.follower_link_state_mut().set_direction_lock_bits(1);
            self.follower_link_state_mut().clear_animation_step();
            self.ancilla_sfx2_near(50);
        }

        self.halt_link_when_using_items();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            >= 0
        {
            return;
        }

        self.follower_link_state_mut()
            .increment_item_action_step_var();
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(3);
        if self.game_state.player.follower_link.item_action_step_var() == 10 {
            self.follower_link_state_mut().clear_item_action_step_var();
            self.follower_link_state_mut().clear_action_handler_timer();
            let button_mask_b_y = self.game_state.player.follower_link.button_mask_b_y() & 0x80;
            self.follower_link_state_mut()
                .set_button_mask_b_y(button_mask_b_y);
            self.follower_link_state_mut().clear_position_mode();
            self.follower_link_state_mut().clear_direction_lock_bits(1);
            self.follower_link_state_mut().disable_oam_offsets();
            return;
        }

        let index = base + self.game_state.player.follower_link.item_action_step_var() as usize;
        self.follower_link_state_mut()
            .set_action_handler_timer(LINK_ITEM_NET_BUG_NET_TIMERS[index]);
    }

    pub(super) fn ancilla_add_dug_up_flute(&mut self, ty: u8, limit: u8) {
        let Some(k) = self.ancilla_add_simple(ty, limit) else {
            return;
        };
        let x_velocity = if self.game_state.player.follower_link.facing() == 4 {
            (-8i8) as u8
        } else {
            8
        };
        {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_z(0);
            ancilla.set_z_velocity(24);
            ancilla.set_x_velocity(x_velocity);
            ancilla.set_step(0);
        }
        self.decode_animated_sprite_tile_variable(12);
        self.ancilla_set_xy(k, 0x0490, 0x0a8a);
    }

    pub(super) fn ancilla_add_cane_of_byrna_init_spark(&mut self, ty: u8, limit: u8) {
        for k in (0..5).rev() {
            if self.ancilla_slot_view(k).ancilla_type() == 0x31 {
                self.ancilla_slot_view_mut(k).set_ancilla_type(0);
            }
        }
        if let Some(k) = self.ancilla_add_simple(ty, limit) {
            let mut spark = self.ancilla_slot_view_mut(k);
            spark.set_item_to_link(0);
            spark.set_aux_timer(9);
            spark.set_work_byte_3(2);
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
        }
    }

    pub(super) fn ancilla_add_shovel_dirt(&mut self, ty: u8, limit: u8) {
        if let Some(k) = self.ancilla_add_simple(ty, limit) {
            let mut dirt = self.ancilla_slot_view_mut(k);
            dirt.set_item_to_link(0);
            dirt.set_timer(20);
            self.ancilla_set_xy(
                k,
                self.game_state.player.follower_link.x(),
                self.game_state.player.follower_link.y(),
            );
        }
    }

    fn start_medallion_item(
        &mut self,
        player_state: u8,
        delay: u8,
        spin_state: u8,
        quake: Option<()>,
    ) {
        if !self.check_y_button_press() {
            return;
        }
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);

        let sword_ok = self.game_state.inventory.items.sword_type().wrapping_add(1) & !1 != 0;
        let blocked = self.game_state.player.follower_link.doorway_state() != 0
            || self.game_state.player.follower_link.is_menu_blocked()
            || self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x8000 != 0
            || !sword_ok
            || (self.game_state.sprites.follower_runtime.dropped() != 0
                && self.game_state.sprites.follower_runtime.indicator() == 13);
        if blocked {
            self.ancilla_sfx2_near(60);
            return;
        }

        if (0..3).any(|i| self.ancilla_slot_view(i).ancilla_type() != 0) {
            return;
        }
        if !self.link_check_magic_cost(1) {
            return;
        }

        self.follower_link_state_mut()
            .set_handler_state(player_state);
        self.follower_link_state_mut().set_direction_lock_bits(1);
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(delay);
        self.follower_link_state_mut()
            .set_state_for_spin_attack(spin_state);
        self.follower_link_state_mut()
            .clear_spin_animation_step_counter();
        self.follower_link_state_mut()
            .set_spin_attack_sound_latch(0);
        if quake.is_some() {
            self.follower_link_state_mut()
                .set_actual_z_velocity_mirror_and_copy(40);
            self.follower_link_state_mut().clear_z_mirror_low();
        }
        self.ancilla_sfx3_near(35);
    }

    fn start_mirror_transition(&mut self, submodule: u8) {
        self.set_submodule(submodule);
        self.follower_link_state_mut()
            .clear_pull_for_rupees_sprite_need();
        self.follower_link_state_mut().set_whirlpool_trigger();
        self.set_subsubmodule(0);
        self.follower_link_state_mut().clear_actual_velocity_xy();
        self.follower_link_state_mut().set_handler_state(20);
    }

    pub(super) fn ancilla_add_hookshot(&mut self, a: u8, y: u8) {
        self.ancilla_add_hookshot_inner(a, y);
    }

    fn ancilla_add_hookshot_inner(&mut self, a: u8, y: u8) -> Option<usize> {
        let k = self.ancilla_add_simple(a, y)?;
        self.follower_link_state_mut().clear_hookshot_interlock();
        self.messaging_state_mut().set_effect_index(k as u8);
        let dir = self.game_state.player.follower_link.facing() >> 1;
        {
            let mut hookshot = self.ancilla_slot_view_mut(k);
            hookshot.set_aux_timer(3);
            hookshot.set_step(0);
            hookshot.set_l(0);
            hookshot.set_k(0);
            hookshot.set_g(0xff);
            hookshot.set_work_byte_1(0);
            hookshot.set_item_to_link(0);
            hookshot.set_timer(0);
            hookshot.set_direction(dir);
            hookshot.set_x_velocity(ANCILLA_ADD_HOOKSHOT_INNER_HOOKSHOT_X_VEL[dir as usize]);
            hookshot.set_y_velocity(ANCILLA_ADD_HOOKSHOT_INNER_HOOKSHOT_Y_VEL[dir as usize]);
        }
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(ANCILLA_ADD_HOOKSHOT_INNER_HOOKSHOT_X_DELTA[dir as usize] as u16);
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(ANCILLA_ADD_HOOKSHOT_INNER_HOOKSHOT_Y_DELTA[dir as usize] as u16);
        self.ancilla_set_xy(k, x, y);
        Some(k)
    }

    fn finish_hookshot_state_after_missing_ancilla(&mut self) {
        self.follower_link_state_mut().clear_handler_state();
        self.follower_link_state_mut().clear_action_handler_timer();
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        self.follower_link_state_mut().clear_direction_lock_bits(1);
        self.follower_link_state_mut().clear_position_mode_bits(4);
        self.follower_link_state_mut()
            .clear_sprite_damage_disable_timer();
        if self.game_state.player.follower_link.button_b_frames() >= 9 {
            self.follower_link_state_mut().set_button_b_frames(9);
        }
    }

    fn finish_hookshot_state_without_button_clamp(&mut self) {
        self.follower_link_state_mut().clear_handler_state();
        self.follower_link_state_mut().clear_action_handler_timer();
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(0);
        self.follower_link_state_mut().clear_hookshot_interlock();
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        self.follower_link_state_mut().clear_direction_lock_bits(1);
        self.follower_link_state_mut().clear_position_mode_bits(4);
        self.follower_link_state_mut()
            .clear_sprite_damage_disable_timer();
    }

    fn finish_byrna_item(&mut self) {
        self.follower_link_state_mut().clear_item_action_step_var();
        self.follower_link_state_mut().clear_action_handler_timer();
        let button_mask_b_y = self.game_state.player.follower_link.button_mask_b_y() & 0x80;
        self.follower_link_state_mut()
            .set_button_mask_b_y(button_mask_b_y);
        self.follower_link_state_mut().clear_position_mode();
        self.follower_link_state_mut().clear_direction_lock_bits(1);
    }

    fn finish_powder_item(&mut self) {
        self.follower_link_state_mut().clear_item_in_hand();
        self.follower_link_state_mut().clear_action_handler_timer();
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
    }

    pub(super) fn link_reset_sword_and_item_usage(&mut self) {
        self.follower_link_state_mut().cancel_sword_and_item_usage();
    }
}
