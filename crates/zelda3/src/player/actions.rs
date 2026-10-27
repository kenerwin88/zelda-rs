//! Recoil, pits, hopping and jumping, dashing, sword controls, and spin attacks.

use super::*;
use crate::game_state::{CollisionAxis, CollisionDirection, MovementProbeKind};

impl ZeldaState {
    pub(in crate::zelda_rtl) fn link_state_recoil(&mut self) {
        let old_x = self.game_state.player.follower_link.x();
        let old_y = self.game_state.player.follower_link.y();
        self.store_link_safe_return_position(old_x, old_y);

        self.link_handle_change_in_z_velocity();
        self.follower_link_state_mut().clear_direction_lock();
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();

        if self.game_state.player.follower_link.is_z_low_negative()
            && (self.game_state.player.follower_link.actual_z_velocity() as i8).is_negative()
        {
            self.tile_detect_main_handler(5);
            if self.game_state.player.tile_detection.deepwater() & 1 != 0 {
                self.follower_link_state_mut().set_handler_state(4);
                self.link_set_to_deep_water();
                self.link_reset_sword_and_item_usage();
                self.ancilla_add_splash(21, 0);
                self.link_handle_recoil_and_timer(true);
            } else {
                let recoil_timer = self.follower_link_state_mut().increment_recoil_timer();
                if recoil_timer != 4 {
                    let mut z = self
                        .game_state
                        .player
                        .follower_link
                        .actual_z_velocity_copy();
                    let mut s = recoil_timer;
                    loop {
                        z >>= 1;
                        s = s.wrapping_sub(1);
                        if s != 0 {
                            break;
                        }
                    }
                    self.follower_link_state_mut().set_actual_z_velocity(z);
                } else {
                    self.follower_link_state_mut().set_recoil_timer(3);
                }
                self.link_handle_recoil_and_timer(false);
            }
        } else {
            self.link_handle_recoil_and_timer(false);
        }
        self.follower_link_state_mut().clear_z_high();
    }

    pub(super) fn link_state_sleeping(&mut self) {
        match self.game_state.player.follower_link.sleep_in_bed_state() {
            0 => {
                if self.game_state.frame.frame_counter & 0x1f == 0 {
                    self.ancilla_add_snoring(0x21, 1);
                }
            }
            1 => {
                if self.game_state.frame.submodule == 0
                    && (self.follower_link_state_mut().decrement_dash_countdown() as i8)
                        .is_negative()
                {
                    self.follower_link_state_mut().set_dash_countdown(0);
                    let input = (self.game_state.player.follower_link.filtered_joypad_h() & 0xe0)
                        | (self.game_state.player.follower_link.filtered_joypad_h() << 4)
                        | self.game_state.player.follower_link.filtered_joypad_l();
                    if input & 0xf0 != 0 {
                        self.follower_link_state_mut().increment_opening_pose();
                        self.follower_link_state_mut().set_facing(6);
                        self.follower_link_state_mut()
                            .increment_sleep_in_bed_state();
                        self.follower_link_state_mut().set_dash_countdown(4);
                    }
                }
            }
            2 => {
                if (self.follower_link_state_mut().decrement_dash_countdown() as i8).is_negative() {
                    self.follower_link_state_mut().set_actual_velocity_xy(21, 4);
                    self.follower_link_state_mut()
                        .set_actual_z_velocity_and_copy(24);
                    self.follower_link_state_mut().set_incapacitated_timer(16);
                    self.follower_link_state_mut().set_auxiliary_state(2);
                    self.follower_link_state_mut().set_handler_state(6);
                }
            }
            _ => {}
        }
    }

    pub(in crate::zelda_rtl) fn link_state_zapped(&mut self) {
        self.cache_camera_properties_if_outdoors();
        self.link_zap_handle_mosaic();

        let delay = self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer();
        if !(delay as i8).is_negative() {
            return;
        }

        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(2);
        self.follower_link_state_mut()
            .increment_action_handler_timer();
        if self.game_state.player.follower_link.action_handler_timer() & 1 != 0 {
            self.palette_electro_themed_gear();
        } else {
            self.load_actual_gear_palettes();
        }
        if self.game_state.player.follower_link.action_handler_timer() == 8 {
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut().clear_handler_state();
            self.follower_link_state_mut()
                .clear_sprite_damage_disable_timer();
            self.follower_link_state_mut().clear_electrocute_on_touch();
            self.follower_link_state_mut().clear_auxiliary_state();
            self.player_set_custom_mosaic_level(0);
        }
    }

    pub(in crate::zelda_rtl) fn link_state_exiting_dash(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.game_state.player.follower_link.joypad1h_last() & 0x0f != 0
            || self.game_state.player.follower_link.dash_countdown() >= 16
        {
            self.follower_link_state_mut().set_dash_countdown(0);
            self.follower_link_state_mut().set_speed_setting(0);
            self.follower_link_state_mut().clear_handler_state();
            self.follower_link_state_mut().clear_running();
            self.swim_acceleration_mut().set_mode(0, 0);
            if self.game_state.player.follower_link.button_b_frames() < 9 {
                self.follower_link_state_mut().clear_direction_lock();
            }
        } else {
            self.follower_link_state_mut().increment_dash_countdown();
        }
        self.link_handle_moving_animation_full_long_entry();
    }

    pub(in crate::zelda_rtl) fn reset_all_acceleration(&mut self) {
        if self.lag_free {
            // $07:9873..9895: REP, ten word clears, SEP and RTS.
            crate::cycle_ledger::charge_routine(0x07_9873, 486);
        }
        let mut acceleration = self.swim_acceleration_mut();
        acceleration.clear_axis_motion(0);
        acceleration.clear_axis_motion(2);
        self.follower_link_state_mut().clear_swim_stroke_counters();
    }

    pub(super) fn link_state_pits(&mut self) {
        self.follower_link_state_mut().set_direction(0);
        if self.game_state.player.follower_link.pit_correction_active() && {
            self.follower_link_state_mut()
                .increment_pit_correction_timer();
            self.game_state.player.follower_link.pit_correction_timer() == 0x20
        } {
            self.follower_link_state_mut().set_pit_correction_timer(31);
        } else {
            if !self.game_state.player.follower_link.is_running() {
                if !self
                    .game_state
                    .player
                    .follower_link
                    .is_in_auxiliary_state(1)
                {
                    let direction = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
                    self.follower_link_state_mut().set_direction(direction);
                }
                self.link_state_pits_after_aux_state();
                return;
            }
            if self.game_state.player.follower_link.dash_countdown() != 0
                && (!self
                    .game_state
                    .enhanced_features
                    .has(LINK_STATE_PITS_FEATURES0_MISC_BUG_FIXES)
                    || self.game_state.player.follower_link.joypad1l_last() & 0x80 != 0)
            {
                self.link_state_dashing();
                return;
            }
            if self.game_state.player.follower_link.joypad1h_last() & 0x0f != 0
                && (self.game_state.player.follower_link.joypad1h_last()
                    & 0x0f
                    & self.game_state.player.follower_link.direction())
                    == 0
            {
                self.link_cancel_dash();
                if !self
                    .game_state
                    .player
                    .follower_link
                    .is_in_auxiliary_state(1)
                {
                    let direction = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
                    self.follower_link_state_mut().set_direction(direction);
                }
            }
        }

        self.link_state_pits_after_aux_state();
    }

    pub(in crate::zelda_rtl) fn handle_dungeon_landing_from_pit(&mut self) {
        self.link_oam_main();
        self.follower_link_state_mut()
            .cache_previous_position_from_current_xy_order();
        if self.game_state.frame.submodule == 7 {
            self.follower_link_state_mut().set_visibility_status(0);
        }
        if self.game_state.frame.frame_counter & 3 == 0 {
            self.follower_link_state_mut().advance_pit_data_index();
            if self.game_state.player.follower_link.pit_data_index() == 10 {
                self.follower_link_state_mut().set_pit_data_index(6);
            }
        }
        self.follower_link_state_mut().set_direction(4);
        self.link_handle_velocity();

        let link_y = self.game_state.player.follower_link.y();
        let target_y = self.game_state.player.tile_detection.y();
        if (link_y as i16).is_negative() && !(target_y as i16).is_negative() {
            if (!link_y).wrapping_add(1).wrapping_add(target_y) < 0x8000 {
                return;
            }
        } else if target_y >= link_y {
            return;
        }

        if self
            .game_state
            .enhanced_features
            .has(HANDLE_DUNGEON_LANDING_FROM_PIT_FEATURES0_MISC_BUG_FIXES)
        {
            self.follower_link_state_mut()
                .clear_about_to_jump_off_ledge();
        }
        self.follower_link_state_mut().set_y(target_y);
        self.follower_link_state_mut().clear_animation_step();
        self.follower_link_state_mut().clear_speed_modifier();
        self.follower_link_state_mut().clear_pit_data_index();
        self.follower_link_state_mut().clear_near_pit_state();
        self.follower_link_state_mut().set_speed_setting(0);
        self.set_subsubmodule(0);
        self.set_submodule(0);
        self.follower_link_state_mut()
            .clear_sprite_damage_disable_timer();
        if self.game_state.sprites.follower_runtime.indicator() != 0
            && self.game_state.sprites.follower_runtime.indicator() != 3
        {
            self.follower_state_mut().set_appearance_none_flag(0);
            if self.game_state.sprites.follower_runtime.indicator() == 13 {
                self.follower_state_mut().set_indicator(0);
                self.set_super_bomb_indicator_timer(0);
                self.set_super_bomb_indicator_counter(0);
                self.follower_state_mut().set_dropped(0);
            } else {
                self.follower_initialize();
            }
        }
        self.tile_detect_main_handler(0);
        if self.game_state.player.tile_detection.shallow_water() & 1 != 0 {
            self.ancilla_sfx2_near(0x24);
        }
        // ROM $07:95C7
        self.player_tile_detect_nearby_marked();
        if self.game_state.system_signals.sound_effect_1() & 0x3f != 0x24 {
            self.ancilla_sfx2_near(0x21);
        }

        if self.game_state.dungeon.room_load.header_collision_2() == 2
            && self.game_state.player.tile_detection.water_staircase() & 0x0f != 0
        {
            self.set_player_layer_collision_flags(
                crate::game_state::constants::player::LAYER_COLLISION_BOTH,
            );
        }
        if self.game_state.player.tile_detection.deepwater() & 0x0f == 0x0f {
            self.follower_link_state_mut().enter_deep_water_state();
            self.follower_link_state_mut()
                .set_swim_flags_from_last_direction();
            self.link_reset_swimming_state();
            self.follower_link_state_mut().mark_lower_level();
            self.ancilla_add_splash(0x15, 1);
            self.follower_link_state_mut().set_handler_state(4);
            self.link_force_unequip_cape_quietly();
            self.follower_link_state_mut()
                .clear_state_item_and_grab_flags();
            self.follower_link_state_mut().set_speed_setting(0);
        } else {
            let handler_state = if self.game_state.player.tile_detection.pit_tile() & 0x0f != 0 {
                1
            } else {
                0
            };
            self.follower_link_state_mut()
                .set_handler_state(handler_state);
        }
    }

    pub(super) fn link_state_spin_attack(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.game_state.player.follower_link.has_auxiliary_state() {
            for i in (0..5).rev() {
                if matches!(self.ancilla_slot_view(i).ancilla_type(), 0x2a | 0x2b) {
                    self.ancilla_slot_view_mut(i).set_ancilla_type(0);
                }
            }
            self.follower_link_state_mut().clear_z_high();
            self.follower_link_state_mut().clear_direction_lock_bits(1);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(0);
            self.follower_link_state_mut().set_button_b_frames(0);
            self.follower_link_state_mut().set_button_mask_b_y(0);
            self.follower_link_state_mut().set_y_button_action_flags(0);
            self.follower_link_state_mut().set_state_for_spin_attack(0);
            self.follower_link_state_mut()
                .clear_spin_animation_step_counter();
            self.follower_link_state_mut().set_speed_setting(0);
            if self.game_state.player.follower_link.electrocute_on_touch() != 0 {
                if self.game_state.player.follower_link.is_cape_active() {
                    self.link_force_unequip_cape_quietly();
                }
                self.link_reset_sword_and_item_usage();
                self.follower_link_state_mut()
                    .set_sprite_damage_disable_timer(1);
                self.follower_link_state_mut().clear_action_handler_timer();
                self.follower_link_state_mut()
                    .set_spin_attack_delay_timer(2);
                self.follower_link_state_mut().clear_animation_step();
                self.follower_link_state_mut().clear_direction_flags(0x0f);
                self.ancilla_sfx3_near(43);
                self.follower_link_state_mut().set_handler_state(7);
                self.link_state_zapped();
            } else {
                self.follower_link_state_mut().set_handler_state(2);
                self.link_state_recoil();
            }
            return;
        }

        if self.game_state.player.follower_link.incapacitated_timer() != 0 {
            self.link_handle_recoil_and_timer(false);
        } else {
            self.follower_link_state_mut().set_direction(0);
            self.link_handle_velocity();
            self.link_handle_cardinal_collision();
            self.follower_link_state_mut().set_handler_state(3);
            self.follower_link_state_mut().clear_pit_correction();
            self.handle_indoor_camera_and_doors();
        }

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
            == 2
        {
            self.ancilla_sfx3_near(35);
        }
        if self
            .game_state
            .player
            .follower_link
            .spin_animation_step_counter()
            == 12
        {
            self.follower_link_state_mut().clear_direction_lock_bits(1);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(0);
            self.follower_link_state_mut().set_button_b_frames(0);
            self.follower_link_state_mut().set_state_for_spin_attack(0);
            self.follower_link_state_mut()
                .clear_spin_animation_step_counter();
            if self.game_state.player.follower_link.handler_state() != 30 {
                let button_mask_b_y = if self.game_state.player.follower_link.button_b_frames() != 0
                {
                    self.game_state.player.follower_link.joypad1h_last() & 0x80
                } else {
                    0
                };
                self.follower_link_state_mut()
                    .set_button_mask_b_y(button_mask_b_y);
            }
            self.follower_link_state_mut().clear_handler_state();
        } else {
            let idx = self
                .game_state
                .player
                .follower_link
                .spin_animation_step_counter()
                .wrapping_add(self.game_state.player.follower_link.spin_offsets())
                as usize;
            self.follower_link_state_mut()
                .set_state_for_spin_attack(LINK_SPIN_GRAPHICS_BY_DIR[idx]);
            let delay = LINK_SPIN_DELAYS[self
                .game_state
                .player
                .follower_link
                .spin_animation_step_counter() as usize];
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(delay);
            self.tile_detect_main_handler(8);
        }
    }

    pub(super) fn link_hop_hopping_south_ow(&mut self) {
        self.follower_link_state_mut()
            .set_last_direction_moved_towards(1);
        self.follower_link_state_mut().clear_direction_lock();
        self.follower_link_state_mut().clear_actual_velocity_xy();
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        if self.game_state.player.follower_link.incapacitated_timer() == 0
            && self
                .game_state
                .player
                .follower_link
                .actual_z_velocity_mirror()
                == 0
        {
            self.ancilla_sfx2_near(32);
            self.link_hop_find_tile_to_land_on_south();
            if self.game_state.world.location.is_outdoors() {
                self.follower_link_state_mut().set_lower_level_state(2);
            }
        }

        self.follower_link_state_mut().restore_z_from_mirror();
        self.follower_link_state_mut()
            .restore_actual_z_velocity_from_mirror();
        self.follower_link_state_mut()
            .decrement_actual_z_velocity(2);
        self.link_move_position();

        if (self.game_state.player.follower_link.actual_z_velocity() as i8).is_negative() {
            if self.game_state.player.follower_link.actual_z_velocity() < 0xa0 {
                self.follower_link_state_mut().set_actual_z_velocity(0xa0);
            }
            if self
                .game_state
                .player
                .follower_link
                .is_landing_at_or_above_ground()
            {
                self.follower_link_state_mut().set_z(0);
                self.link_splash_upon_landing();
                if self.game_state.player.follower_link.is_near_pit() {
                    self.follower_link_state_mut().set_handler_state(1);
                }
                if self.game_state.player.follower_link.handler_state() != 4
                    && self.game_state.player.follower_link.handler_state() != 1
                    && !self.game_state.player.follower_link.is_in_deep_water()
                {
                    self.ancilla_sfx2_near(33);
                }
                self.follower_link_state_mut()
                    .clear_sprite_damage_disable_timer();
                self.set_allow_scroll_z(0);
                self.follower_link_state_mut().clear_auxiliary_state();
                self.follower_link_state_mut().set_actual_z_velocity(0xff);
                self.follower_link_state_mut().set_z(0xffff);
                self.follower_link_state_mut().set_incapacitated_timer(0);
                if self.game_state.world.location.is_outdoors() {
                    self.follower_link_state_mut().clear_lower_level();
                }
            } else {
                let y_velocity = self.game_state.player.follower_link.z_mirror_delta_low();
                self.follower_link_state_mut().set_y_velocity(y_velocity);
            }
        } else {
            let y_velocity = self.game_state.player.follower_link.z_mirror_delta_low();
            self.follower_link_state_mut().set_y_velocity(y_velocity);
        }
        self.follower_link_state_mut()
            .cache_actual_z_velocity_to_mirror();
        self.follower_link_state_mut().cache_z_to_mirror();
    }

    pub(super) fn link_hop_find_tile_to_land_on_south(&mut self) {
        let original_y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .set_hop_origin_coord(original_y);
        let y_velocity = self
            .game_state
            .player
            .follower_link
            .y_low_delta_from_safe_return();
        self.follower_link_state_mut().set_y_velocity(y_velocity);
        loop {
            let dir = self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards_index();
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(HOP_SOUTH_Y[dir] as i16 as u16);
            self.follower_link_state_mut().set_y(y);
            self.detect_player_movement(
                CollisionAxis::Vertical,
                CollisionDirection::from_legacy(
                    self.game_state
                        .player
                        .follower_link
                        .last_direction_moved_towards(),
                ),
                MovementProbeKind::Cardinal,
            );
            let terrain = self.game_state.player.tile_detection.normal_tiles()
                | self.game_state.player.tile_detection.pit_tile_word()
                | self
                    .game_state
                    .player
                    .tile_detection
                    .destruction_aftermath()
                | self.game_state.player.tile_detection.thick_grass()
                | self.game_state.player.tile_detection.deepwater();
            if terrain & 7 == 7 {
                break;
            }
        }
        if self.game_state.player.tile_detection.deepwater() & 7 != 0 {
            self.follower_link_state_mut().enter_deep_water_state();
            if !self
                .game_state
                .player
                .follower_link
                .is_in_auxiliary_state(4)
            {
                self.follower_link_state_mut().set_auxiliary_state(2);
            }
            self.follower_link_state_mut()
                .set_swim_flags_from_last_direction();
            self.link_reset_swimming_state();
            self.follower_link_state_mut().clear_grabbing_wall();
            self.follower_link_state_mut().set_speed_setting(0);
        }
        if self.game_state.player.tile_detection.pit_tile_word() & 7 != 0 {
            self.follower_link_state_mut().mark_pit_landing_oam_state();
            self.follower_link_state_mut().begin_pit_check();
        }
        let dir = self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards_index();
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(HOP_SOUTH_Y2[dir] as i16 as u16);
        self.follower_link_state_mut().set_y(y);
        self.follower_link_state_mut().store_safe_return_y(y);
        self.follower_link_state_mut().set_incapacitated_timer(1);
        let mut z = self.game_state.player.follower_link.z_low();
        if z >= 0xf0 {
            z = 0;
        }
        let z = y.wrapping_sub(original_y).wrapping_add(z as u16);
        self.follower_link_state_mut().set_z_and_mirror(z);
    }

    pub(super) fn link_state_hopping_horizontally_ow(&mut self) {
        let direction = if self
            .game_state
            .player
            .follower_link
            .actual_x_velocity_signed()
            .is_negative()
        {
            6
        } else {
            5
        };
        self.follower_link_state_mut().set_direction(direction);
        self.follower_link_state_mut().clear_direction_lock();
        self.follower_link_state_mut().set_actual_y_velocity(0);
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        self.link_state_handling_jump();
    }

    pub(super) fn link_hopping_horizontally_find_tile_y(&mut self) {
        let original_y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .set_hop_origin_coord(original_y);
        let y_velocity = self
            .game_state
            .player
            .follower_link
            .y_low_delta_from_safe_return();
        self.follower_link_state_mut().set_y_velocity(y_velocity);

        let dir = self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards_index();
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(HOP_SOUTH_Y[dir] as i16 as u16);
        self.follower_link_state_mut().set_y(y);
        self.detect_player_movement(
            CollisionAxis::Vertical,
            CollisionDirection::from_legacy(
                self.game_state
                    .player
                    .follower_link
                    .last_direction_moved_towards(),
            ),
            MovementProbeKind::Cardinal,
        );

        let terrain = self.game_state.player.tile_detection.normal_tiles()
            | self
                .game_state
                .player
                .tile_detection
                .destruction_aftermath()
            | self.game_state.player.tile_detection.thick_grass()
            | self.game_state.player.tile_detection.deepwater();

        if terrain & 7 != 7 {
            self.follower_link_state_mut().set_y(original_y);
            self.follower_link_state_mut().set_incapacitated_timer(1);

            let org_velx = self.game_state.player.follower_link.actual_x_velocity();
            let mut velx = org_velx as i8;
            if velx < 0 {
                velx = velx.wrapping_neg();
            }
            let idx = ((velx as u8) >> 4) as usize;
            self.follower_link_state_mut()
                .set_actual_z_velocity_mirror_and_copy(HOP_HORIZ_VEL_Z[idx]);
            let mut xt = HOP_HORIZ_VEL_X[idx];
            if (org_velx as i8) < 0 {
                xt = 0u8.wrapping_sub(xt);
            }
            self.follower_link_state_mut().set_actual_x_velocity(xt);
        } else {
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(HOP_SOUTH_Y2[dir] as i16 as u16);
            self.follower_link_state_mut().set_y(y);
            self.follower_link_state_mut().store_safe_return_y(y);
            self.follower_link_state_mut().set_incapacitated_timer(1);
            let mut z = self.game_state.player.follower_link.z_low();
            if z == 255 {
                z = 0;
            }
            let z = y.wrapping_sub(original_y).wrapping_add(z as u16);
            self.follower_link_state_mut().set_z_and_mirror(z);
        }

        if self.game_state.player.tile_detection.deepwater() & 7 != 0 {
            self.follower_link_state_mut().set_auxiliary_state(2);
            self.link_set_to_deep_water();
        }
    }

    pub(super) fn link_hopping_horizontally_find_tile_x(&mut self, o: u8) -> u8 {
        assert!(o == 0 || o == 2);
        let original_x = self.game_state.player.follower_link.x();
        self.follower_link_state_mut()
            .set_hop_origin_coord(original_x);
        let table_idx = (o >> 1) as usize;
        let mut i: i16 = 7;
        loop {
            let x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(HOP_HORIZ_X_STEP[table_idx] as i16 as u16);
            self.follower_link_state_mut().set_x(x);
            self.detect_player_movement(
                CollisionAxis::Horizontal,
                CollisionDirection::from_legacy(
                    self.game_state
                        .player
                        .follower_link
                        .last_direction_moved_towards(),
                ),
                MovementProbeKind::Cardinal,
            );

            let terrain = self.game_state.player.tile_detection.normal_tiles()
                | self
                    .game_state
                    .player
                    .tile_detection
                    .destruction_aftermath()
                | self.game_state.player.tile_detection.thick_grass()
                | self.game_state.player.tile_detection.deepwater()
                | self.game_state.player.tile_detection.pit_tile_word();

            if terrain & 7 == 7 {
                if self.game_state.player.tile_detection.deepwater() & 7 == 7 {
                    self.follower_link_state_mut().enter_deep_water_state();
                    self.follower_link_state_mut().set_auxiliary_state(2);
                    self.follower_link_state_mut()
                        .set_swim_flags_from_last_direction();
                    self.follower_link_state_mut().clear_swimming_countdown();
                    self.follower_link_state_mut().set_speed_setting(0);
                    self.follower_link_state_mut().clear_grabbing_wall();
                    self.reset_all_acceleration();
                }
                break;
            }
            i -= 1;
            if i < 0 {
                let x = original_x.wrapping_add(HOP_HORIZ_X_FALLBACK[table_idx] as i16 as u16);
                self.follower_link_state_mut().set_x(x);
                break;
            }
        }

        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(HOP_HORIZ_X_FINAL[table_idx] as i16 as u16);
        self.follower_link_state_mut().set_x(x);
        let distance = original_x.wrapping_sub(x) as i16;
        let distance = if distance < 0 { -distance } else { distance };
        let idx = (distance as u16 >> 3) as usize;
        let mut velx = HOP_HORIZ_X_VEL[idx];
        if o != 2 {
            velx = 0u8.wrapping_sub(velx);
        }
        self.follower_link_state_mut().set_actual_x_velocity(velx);
        self.follower_link_state_mut()
            .set_actual_z_velocity_mirror_and_copy(HOP_HORIZ_Z_VEL[idx]);
        i as u8
    }

    pub(super) fn link_state_hopping_diagonally_up_ow(&mut self) {
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        self.player_change_z(2);
        self.link_move_position();
        if self.game_state.player.follower_link.is_z_low_negative() {
            self.link_splash_upon_landing();
            if self.game_state.player.follower_link.handler_state() != 4
                && !self.game_state.player.follower_link.is_in_deep_water()
            {
                self.ancilla_sfx2_near(33);
            }
            self.follower_link_state_mut()
                .clear_sprite_damage_disable_timer();
            self.follower_link_state_mut().clear_auxiliary_state();
            self.follower_link_state_mut().set_actual_z_velocity(0xff);
            self.follower_link_state_mut().set_z(0xffff);
            self.follower_link_state_mut().set_incapacitated_timer(0);
            self.follower_link_state_mut().clear_direction_lock();
        }
    }

    pub(super) fn link_state_hopping_diagonally_down_ow(&mut self) {
        let dir = if self
            .game_state
            .player
            .follower_link
            .actual_x_velocity_signed()
            .is_negative()
        {
            2
        } else {
            3
        };
        self.follower_link_state_mut()
            .set_last_direction_moved_towards(dir);
        self.follower_link_state_mut().clear_direction_lock();
        self.follower_link_state_mut().set_actual_y_velocity(0);
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        if self.game_state.player.follower_link.incapacitated_timer() == 0
            && self
                .game_state
                .player
                .follower_link
                .actual_z_velocity_mirror()
                == 0
        {
            self.follower_link_state_mut()
                .set_last_direction_moved_towards(1);
            let old_x = self.game_state.player.follower_link.x();
            self.ancilla_sfx2_near(32);
            self.link_hop_find_landing_spot_diagonally_down();
            self.follower_link_state_mut().set_x(old_x);

            let distance = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_sub(self.game_state.player.follower_link.hop_origin_coord());
            let idx = ((distance >> 3) as usize).min(23);
            let mut velx = LEDGE_DOWN_X_VEL[idx];
            if dir == 2 {
                velx = 0u8.wrapping_sub(velx);
            }
            self.follower_link_state_mut().set_actual_x_velocity(velx);
            if self.game_state.world.location.is_outdoors() {
                self.follower_link_state_mut().set_lower_level_state(2);
            }
        }
        self.link_state_handling_jump();
    }

    pub(super) fn link_hop_find_landing_spot_diagonally_down(&mut self) {
        let original_y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .set_hop_origin_coord(original_y);
        let y_velocity = self
            .game_state
            .player
            .follower_link
            .y_low_delta_from_safe_return();
        self.follower_link_state_mut().set_y_velocity(y_velocity);

        let scratch = loop {
            let o = if self
                .game_state
                .player
                .follower_link
                .actual_x_velocity_signed()
                .is_negative()
            {
                0
            } else {
                1
            };
            let x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(LEDGE_DIAG_DX[o] as i16 as u16);
            self.follower_link_state_mut().set_x(x);
            let dir = self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards_index();
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(LEDGE_DIAG_DY[dir] as i16 as u16);
            self.follower_link_state_mut().set_y(y);
            self.detect_player_movement(
                CollisionAxis::Vertical,
                CollisionDirection::from_legacy(
                    self.game_state
                        .player
                        .follower_link
                        .last_direction_moved_towards(),
                ),
                MovementProbeKind::Cardinal,
            );
            let scratch = LEDGE_DIAG_BITS[o];
            let terrain = self.game_state.player.tile_detection.normal_tiles()
                | self
                    .game_state
                    .player
                    .tile_detection
                    .destruction_aftermath_low() as u16
                | self.game_state.player.tile_detection.thick_grass_low() as u16
                | self.game_state.player.tile_detection.deepwater();
            if terrain & scratch as u16 == scratch as u16 {
                break scratch;
            }
        };

        if self.game_state.player.tile_detection.deepwater() & scratch as u16 != 0 {
            self.follower_link_state_mut().enter_deep_water_state();
            self.follower_link_state_mut().set_auxiliary_state(2);
            self.follower_link_state_mut()
                .set_swim_flags_from_last_direction();
            self.link_reset_swimming_state();
            self.follower_link_state_mut().set_speed_setting(0);
            self.follower_link_state_mut().clear_grabbing_wall();
        }

        let dir = self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards_index();
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(LEDGE_DIAG_DY2[dir] as i16 as u16);
        self.follower_link_state_mut().set_y(y);
        self.follower_link_state_mut().store_safe_return_y(y);
        self.follower_link_state_mut().set_incapacitated_timer(1);
        let z = y
            .wrapping_sub(original_y)
            .wrapping_add(self.game_state.player.follower_link.z_low() as u16);
        self.follower_link_state_mut().set_z_and_mirror(z);
    }

    pub(super) fn link_state_handling_jump(&mut self) {
        self.follower_link_state_mut()
            .restore_actual_z_velocity_from_mirror();
        self.follower_link_state_mut().restore_z_low_from_mirror();
        self.follower_link_state_mut()
            .decrement_actual_z_velocity(2);
        self.link_move_position();
        if (self.game_state.player.follower_link.actual_z_velocity() as i8).is_negative() {
            if self.game_state.player.follower_link.actual_z_velocity() < 0xa0 {
                self.follower_link_state_mut().set_actual_z_velocity(0xa0);
            }
            if self
                .game_state
                .player
                .follower_link
                .is_low_z_landing_at_or_above_ground()
            {
                self.follower_link_state_mut().set_z(0);
                let mut falling_into_pit = false;
                if matches!(
                    self.game_state.player.follower_link.handler_state(),
                    12 | 14
                ) {
                    self.tile_detect_main_handler(0);
                    if self.game_state.player.tile_detection.deepwater() as u8 & 1 != 0 {
                        self.follower_link_state_mut().set_handler_state(4);
                        self.link_set_to_deep_water();
                        self.link_reset_sword_and_item_usage();
                        self.ancilla_add_splash(21, 0);
                    } else if self.game_state.player.tile_detection.pit_tile() & 1 != 0 {
                        self.follower_link_state_mut().mark_pit_landing_oam_state();
                        self.follower_link_state_mut().begin_pit_check();
                        self.follower_link_state_mut().set_handler_state(1);
                        falling_into_pit = true;
                    }
                }
                if !falling_into_pit {
                    self.link_splash_upon_landing();
                    if self.game_state.player.follower_link.handler_state() != 4
                        && !self.game_state.player.follower_link.is_in_deep_water()
                    {
                        self.ancilla_sfx2_near(33);
                    }
                }
                if self.game_state.player.follower_link.handler_state() != 4
                    || !self.game_state.player.follower_link.is_bunny_mirror()
                {
                    self.follower_link_state_mut()
                        .clear_sprite_damage_disable_timer();
                }
                self.set_allow_scroll_z(0);
                self.follower_link_state_mut().clear_auxiliary_state();
                self.follower_link_state_mut().set_actual_z_velocity(0xff);
                self.follower_link_state_mut().set_z(0xffff);
                self.follower_link_state_mut().set_incapacitated_timer(0);
                if self.game_state.world.location.is_outdoors() {
                    self.follower_link_state_mut().clear_lower_level();
                }
            } else {
                let y_velocity = self.game_state.player.follower_link.z_mirror_delta_low();
                self.follower_link_state_mut().set_y_velocity(y_velocity);
            }
        } else {
            let y_velocity = self.game_state.player.follower_link.z_mirror_delta_low();
            self.follower_link_state_mut().set_y_velocity(y_velocity);
        }
        self.follower_link_state_mut()
            .cache_actual_z_velocity_to_mirror();
        self.follower_link_state_mut().cache_z_low_to_mirror();
    }

    pub(in crate::zelda_rtl) fn link_state_dashing(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.link_handle_bunny_transformation() {
            if self.game_state.player.follower_link.handler_state() == 23 {
                self.player_handler_17_bunny();
            }
            return;
        }
        if !self.game_state.player.follower_link.is_running() {
            self.follower_link_state_mut()
                .clear_sprite_damage_disable_timer();
            self.follower_link_state_mut().set_dash_countdown(0);
            self.follower_link_state_mut().set_speed_setting(0);
            self.follower_link_state_mut().clear_handler_state();
            self.follower_link_state_mut().clear_direction_lock();
            return;
        }
        if self.game_state.player.follower_link.button_mask_b_y() & 0x80 != 0
            && self.game_state.player.follower_link.button_b_frames() >= 9
        {
            self.follower_link_state_mut().set_button_b_frames(9);
        }
        self.follower_link_state_mut().set_pit_correction_timer(0);

        if self.game_state.player.follower_link.has_auxiliary_state() {
            self.follower_link_state_mut()
                .clear_sprite_damage_disable_timer();
            self.follower_link_state_mut().set_dash_countdown(0);
            self.follower_link_state_mut().set_speed_setting(0);
            self.follower_link_state_mut().clear_direction_lock();
            self.follower_link_state_mut().clear_running();
            self.follower_link_state_mut().clear_defense_flags();
            if self.game_state.player.follower_link.electrocute_on_touch() != 0 {
                if self.game_state.player.follower_link.is_cape_active() {
                    self.link_force_unequip_cape_quietly();
                }
                self.link_reset_sword_and_item_usage();
                self.follower_link_state_mut()
                    .set_sprite_damage_disable_timer(1);
                self.follower_link_state_mut().clear_action_handler_timer();
                self.follower_link_state_mut()
                    .set_spin_attack_delay_timer(2);
                self.follower_link_state_mut().clear_animation_step();
                self.follower_link_state_mut().clear_direction_flags(0x0f);
                self.ancilla_sfx3_near(43);
                self.follower_link_state_mut().set_handler_state(7);
                self.link_state_zapped();
            } else {
                self.follower_link_state_mut().set_handler_state(2);
                self.link_state_recoil();
            }
            return;
        }

        let mut a = self.game_state.player.follower_link.dash_countdown();
        if a == 0 {
            a = self.game_state.player.follower_link.index_of_dashing_sfx();
            self.follower_link_state_mut()
                .decrement_index_of_dashing_sfx();
        }
        if LINK_STATE_DASHING_DASH_SFX_TRIGGER_MASKS
            [(self.game_state.player.follower_link.dash_countdown() >> 4) as usize]
            & a
            == 0
        {
            self.ancilla_sfx2_near(35);
        }
        if (self.follower_link_state_mut().decrement_dash_countdown() as i8).is_negative() {
            self.follower_link_state_mut().set_dash_countdown(0);
            let follower = self.game_state.sprites.follower_runtime.indicator() as usize;
            if self.game_state.sprites.follower_runtime.indicator()
                == DASH_FOLLOWER_SLOWDOWN_INDICATORS[follower]
            {
                self.follower_state_mut()
                    .set_indicator(DASH_FOLLOWER_RELEASE_INDICATORS[follower]);
            }
        } else {
            self.follower_link_state_mut().clear_index_of_dashing_sfx();
            if self.game_state.player.follower_link.joypad1l_last() & 0x80 == 0 {
                self.follower_link_state_mut().clear_animation_step();
                self.follower_link_state_mut().set_dash_countdown(0);
                self.follower_link_state_mut().set_speed_setting(0);
                self.follower_link_state_mut().clear_handler_state();
                self.follower_link_state_mut().clear_running();
                if self.game_state.player.follower_link.button_mask_b_y() & 0x80 == 0 {
                    self.follower_link_state_mut().clear_direction_lock();
                }
                return;
            }
            self.ancilla_add_dash_dust_charging(30, 0);
            self.follower_link_state_mut().clear_movement_velocity();
            self.follower_link_state_mut().prime_dash_counter();
            self.follower_link_state_mut().set_speed_setting(16);
            let mut dir = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
            if self.game_state.player.follower_link.button_mask_b_y() & 0x80 != 0
                || self.game_state.player.follower_link.doorway_state() != 0
                || dir == 0
            {
                dir = LINK_STATE_DASHING_DASH_DIRECTION_BITS_BY_FACING
                    [self.game_state.player.follower_link.facing_index()];
            }
            self.follower_link_state_mut()
                .set_direction_and_last_direction(dir);
            self.follower_link_state_mut()
                .set_swim_flags_from_last_direction();
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
            self.link_handle_moving_animation_full_long_entry();
            let org_x = self.game_state.player.follower_link.x();
            let org_y = self.game_state.player.follower_link.y();
            self.store_link_safe_return_position(org_x, org_y);
            self.link_handle_moving_floor();
            self.link_apply_conveyor();
            if self
                .game_state
                .player
                .follower_link
                .has_somaria_platform_state()
            {
                self.link_handle_velocity_and_sand_drag(org_x, org_y);
            }
            let x = self.game_state.player.follower_link.x();
            let y = self.game_state.player.follower_link.y();
            self.follower_link_state_mut()
                .set_movement_velocity_from_position_delta(x, y, org_x, org_y);
            self.link_handle_cardinal_collision();
            self.handle_indoor_camera_and_doors();
            return;
        }

        self.follower_link_state_mut()
            .clear_animation_step_if_at_least(6);
        self.follower_link_state_mut()
            .decrement_dash_counter_clamped_to_minimum(32);
        self.ancilla_add_dash_dust(30, 0);
        self.follower_link_state_mut()
            .clear_spin_attack_step_counter();
        if self.game_state.inventory.items.sword_type().wrapping_add(1) & 0xfe != 0 {
            self.tile_detect_main_handler(7);
        }
        if self.game_state.inventory.save_progress.progress_indicator() != 0 {
            self.follower_link_state_mut()
                .add_button_mask_b_y_bits(0x80);
            self.follower_link_state_mut().set_button_b_frames(9);
        }
        self.follower_link_state_mut().set_incapacitated_timer(0);

        let mut want_stop_dash = false;

        if self
            .game_state
            .enhanced_features
            .has(LINK_STATE_DASHING_FEATURES0_TURN_WHILE_DASHING)
        {
            if self.game_state.player.follower_link.joypad1l_last() & 0x80 == 0 {
                self.follower_link_state_mut().set_dash_countdown(0x11);
                want_stop_dash = true;
            } else {
                let t = LINK_STATE_DASHING_DASH_CONTROLS_TO_DIRECTION
                    [(self.game_state.player.follower_link.joypad1h_last() & 0x0f) as usize];
                if t != 0 && t != self.game_state.player.follower_link.last_direction() {
                    self.follower_link_state_mut()
                        .set_direction_and_last_direction(t);
                    self.follower_link_state_mut()
                        .set_swim_flags_from_last_direction();
                    self.link_handle_moving_animation_full_long_entry();
                }
            }
        } else {
            let dir = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
            want_stop_dash = dir != 0
                && dir
                    != LINK_STATE_DASHING_DASH_DIRECTION_BITS_BY_FACING
                        [self.game_state.player.follower_link.facing_index()];
        }
        if want_stop_dash {
            self.follower_link_state_mut().set_handler_state(18);
            self.follower_link_state_mut()
                .clear_button_mask_b_y_bits(0x80);
            self.follower_link_state_mut().set_button_b_frames(0);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(0);
            self.link_state_exiting_dash();
            return;
        }

        if self.game_state.player.follower_link.speed_setting() == 0
            && self
                .game_state
                .enhanced_features
                .has(LINK_STATE_DASHING_FEATURES0_TURN_WHILE_DASHING)
        {
            self.follower_link_state_mut().set_speed_setting(16);
        }
        let mut dir = (self
            .game_state
            .player
            .follower_link
            .force_move_any_direction() as u8)
            & 0x0f;
        if dir == 0 {
            dir = LINK_STATE_DASHING_DASH_DIRECTION_BITS_BY_FACING
                [self.game_state.player.follower_link.facing_index()];
        }
        self.follower_link_state_mut()
            .set_direction_and_last_direction(dir);
        self.link_handle_diagonal_collision();
        self.link_handle_velocity();
        self.link_handle_cardinal_collision();
        self.link_handle_moving_animation_full_long_entry();
        self.follower_link_state_mut().clear_pit_correction();
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn link_handle_sword_cooldown(&mut self) {
        // $07:9AC2..9AE5; sword-swing/control children own their bodies.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x079ac2));
        let blocks = crate::cycle_ledger::blocks(self.lag_free);
        blocks.block(0x079ac2);
        if (self.follower_link_state_mut().decrement_sword_delay_timer() as i8) >= 0 {
            blocks.taken(0x079ac5);
            blocks.block(0x079ae5);
            return;
        }
        blocks.block(0x079ac7);
        self.follower_link_state_mut().clear_sword_delay_timer();
        if self
            .game_state
            .player
            .follower_link
            .has_item_or_position_mode()
        {
            blocks.taken(0x079ad0);
            blocks.block(0x079ae5);
            return;
        }
        blocks.block(0x079ad2);
        if self.game_state.player.follower_link.button_b_frames() < 9 {
            blocks.block(0x079ad8);
            if !self.game_state.player.follower_link.is_running() {
                blocks.block(0x079add);
                self.link_check_for_sword_swing();
                blocks.block(0x079ae0);
            } else {
                blocks.taken(0x079adb);
            }
        } else {
            blocks.taken(0x079ad6);
            blocks.block(0x079ae2);
            self.handle_sword_controls();
        }
        blocks.block(0x079ae5);
    }

    pub(super) fn handle_sword_sfx_and_beam(&mut self) {
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        self.follower_link_state_mut().clear_button_b_frames();
        self.follower_link_state_mut()
            .clear_spin_attack_step_counter();

        let health = self
            .game_state
            .inventory
            .player_resources
            .health_capacity()
            .wrapping_sub(4);
        let sword = self.game_state.inventory.items.sword_type();
        if health < self.game_state.inventory.player_resources.current_health()
            && sword.wrapping_add(1) & 0xfe != 0
            && sword >= 2
            && !(0..5)
                .rev()
                .any(|i| self.ancilla_slot_view(i).ancilla_type() == 0x31)
        {
            self.add_sword_beam(0);
        }
        let sword = sword.wrapping_sub(1);
        if sword != 0xfe && sword != 0xff {
            self.set_sound_effect_1_with_link_pan(FIRE_BEAM_SOUNDS[sword as usize]);
        }
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(1);
    }

    pub(super) fn link_check_for_sword_swing(&mut self) {
        if self.game_state.player.follower_link.y_button_action_flags() & 0x10 != 0 {
            return;
        }
        if self.game_state.player.follower_link.button_mask_b_y() & 0x80 == 0 {
            if self.game_state.player.follower_link.filtered_joypad_h() & 0x80 == 0 {
                return;
            }
            if self.game_state.player.follower_link.doorway_state() != 0 {
                self.tile_detect_sword_swing_deep_in_door(
                    self.game_state.player.follower_link.doorway_state(),
                );
                if self.game_state.player.tile_detection.collision_bits_low() & 0x30 == 0x30 {
                    return;
                }
            }
            self.follower_link_state_mut()
                .add_button_mask_b_y_bits(0x80);
            self.handle_sword_sfx_and_beam();
            self.follower_link_state_mut().set_direction_lock_bits(1);
            self.follower_link_state_mut().clear_animation_step();
        }

        if self.game_state.player.follower_link.joypad1h_last() & 0x80 == 0 {
            self.follower_link_state_mut().add_button_mask_b_y_bits(1);
        }
        self.halt_link_when_using_items();
        self.follower_link_state_mut().clear_direction_flags(0x0f);
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            .is_negative()
        {
            let frames = self.follower_link_state_mut().increment_button_b_frames();
            if frames >= 9 {
                self.handle_sword_controls();
                return;
            }
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(spin_attack_delay_for_frames(frames));
            let sword = self.game_state.inventory.items.sword_type();
            if frames == 5 {
                if sword != 0 && sword != 1 && sword != 0xff {
                    self.ancilla_add_sword_swing_sparkle(0x26, 4);
                }
                if sword != 0 && sword != 0xff {
                    self.tile_detect_main_handler(if sword == 1 { 1 } else { 6 });
                }
            } else if frames >= 4
                && self.game_state.player.follower_link.button_mask_b_y() & 1 != 0
                && self.game_state.player.follower_link.joypad1h_last() & 0x80 != 0
            {
                self.follower_link_state_mut().clear_button_mask_b_y_bits(1);
                self.handle_sword_sfx_and_beam();
                return;
            }
        }
        self.calculate_sword_hit_box();
    }

    pub(super) fn handle_sword_controls(&mut self) {
        if self.game_state.player.follower_link.joypad1h_last() & 0x80 != 0 {
            self.player_sword_spin_attack_jerks_hold_down();
        } else if self
            .game_state
            .player
            .follower_link
            .spin_attack_step_counter()
            < 48
        {
            self.link_reset_sword_and_item_usage();
        } else {
            self.link_reset_sword_and_item_usage();
            self.follower_link_state_mut()
                .clear_spin_attack_step_counter();
            self.link_activate_spin_attack();
        }
    }

    pub(super) fn player_sword_spin_attack_jerks_hold_down(&mut self) {
        if self.game_state.player.follower_link.defense_flags() & 0x80 != 0
            || self.game_state.player.follower_link.defense_flags() & 9 == 0
        {
            if self.game_state.sprite_battle.damaging_enemies_timer() == 0 {
                self.follower_link_state_mut().set_button_b_frames(9);
                self.follower_link_state_mut().set_direction_lock_bits(1);
                self.follower_link_state_mut()
                    .set_spin_attack_delay_timer(0);
                if self.game_state.player.follower_link.speed_setting() != 4
                    && self.game_state.player.follower_link.speed_setting() != 16
                {
                    self.follower_link_state_mut().set_speed_setting(12);
                    if self.game_state.inventory.items.sword_type().wrapping_add(1) & !1 == 0 {
                        return;
                    }
                    if (0..5)
                        .rev()
                        .any(|i| matches!(self.ancilla_slot_view(i).ancilla_type(), 0x30 | 0x31))
                    {
                        return;
                    }
                    if self
                        .game_state
                        .player
                        .follower_link
                        .spin_attack_step_counter()
                        >= 6
                        && self.game_state.frame.frame_counter & 3 == 0
                    {
                        self.ancilla_spawn_sword_charge_sparkle();
                    }
                    if self
                        .game_state
                        .player
                        .follower_link
                        .spin_attack_step_counter()
                        < 64
                        && self
                            .follower_link_state_mut()
                            .increment_spin_attack_step_counter()
                            == 48
                    {
                        self.ancilla_sfx2_near(55);
                        self.ancilla_add_charged_spin_attack_sparkle();
                    }
                } else {
                    self.calculate_sword_hit_box();
                }
                return;
            } else if self.game_state.sprite_battle.damaging_enemies_timer() == 1 {
                self.link_reset_sword_and_item_usage();
                return;
            }
        }
        if self.game_state.player.follower_link.button_b_frames() == 9 {
            self.follower_link_state_mut().set_button_b_frames(10);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(SPIN_ATTACK_DELAYS[10]);
        }
        if (self
            .follower_link_state_mut()
            .decrement_spin_attack_delay_timer() as i8)
            .is_negative()
        {
            let mut frames = self
                .game_state
                .player
                .follower_link
                .button_b_frames()
                .wrapping_add(1);
            if frames == 13 {
                if self.game_state.inventory.items.sword_type().wrapping_add(1) & !1 != 0
                    && self.game_state.player.follower_link.defense_flags() & 9 != 0
                {
                    self.ancilla_add_wall_tap_spark(27, 1);
                    self.ancilla_sfx2_near(
                        if self.game_state.player.follower_link.defense_flags() & 8 != 0 {
                            6
                        } else {
                            5
                        },
                    );
                    self.tile_detect_main_handler(1);
                }
                frames = 10;
            }
            self.follower_link_state_mut().set_button_b_frames(frames);
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(spin_attack_delay_for_frames(frames));
        }
        self.calculate_sword_hit_box();
    }

    pub(super) fn link_activate_spin_attack(&mut self) {
        self.ancilla_add_spin_attack_init_spark(42, 0, 0);
        self.link_animate_victory_spin();
    }

    pub(in crate::zelda_rtl) fn link_animate_victory_spin(&mut self) {
        self.follower_link_state_mut().set_handler_state(3);
        let spin_offsets = (self.game_state.player.follower_link.facing() >> 1) * 12;
        self.follower_link_state_mut()
            .set_spin_offsets(spin_offsets);
        self.follower_link_state_mut()
            .set_spin_attack_delay_timer(3);
        let spin_state =
            LINK_SPIN_GRAPHICS_BY_DIR[self.game_state.player.follower_link.spin_offsets() as usize];
        self.follower_link_state_mut()
            .set_state_for_spin_attack(spin_state);
        self.follower_link_state_mut()
            .clear_spin_animation_step_counter();
        self.follower_link_state_mut().set_button_b_frames(144);
        self.follower_link_state_mut().set_direction_lock_bits(1);
        self.follower_link_state_mut().set_button_mask_b_y(0x80);
        self.link_state_spin_attack();
    }

    pub(super) fn link_state_tree_pull(&mut self) {
        self.cache_camera_properties_if_outdoors();
        if self.game_state.player.follower_link.has_auxiliary_state() {
            self.handle_link_from1_d();
            return;
        }

        if self
            .game_state
            .player
            .follower_link
            .has_grabbing_wall_state()
        {
            if self.game_state.player.follower_link.button_mask_b_y() == 0 {
                if self.game_state.player.follower_link.joypad1l_last() & 0x80 == 0 {
                    self.follower_link_state_mut().clear_grabbing_wall();
                    self.follower_link_state_mut().clear_item_action_step_var();
                    self.follower_link_state_mut().set_y_button_action_timer(2);
                    self.follower_link_state_mut().set_y_button_action_step(0);
                    self.follower_link_state_mut().clear_direction_lock();
                    self.follower_link_state_mut().clear_handler_state();
                    self.link_state_default();
                    return;
                }
                if self.game_state.player.follower_link.joypad1h_last() & 4 == 0 {
                    self.link_state_tree_pull_tail();
                    return;
                }
                self.follower_link_state_mut().set_button_mask_b_y(4);
                self.ancilla_sfx2_near(0x22);
            }

            self.follower_link_state_mut()
                .decrement_y_button_action_timer();
            if (self.game_state.player.follower_link.y_button_action_timer() as i8) >= 0 {
                self.link_state_tree_pull_tail();
                return;
            }
            let j = self
                .follower_link_state_mut()
                .increment_item_action_step_var() as usize;
            self.follower_link_state_mut()
                .set_y_button_action_step(*GRAB_WALL_ANIM_STEPS.get(j).unwrap_or(&0));
            self.follower_link_state_mut()
                .set_y_button_action_timer(*GRAB_WALL_ANIM_TIMER.get(j).unwrap_or(&0));
            if j != 7 {
                self.link_state_tree_pull_tail();
                return;
            }

            self.follower_link_state_mut().clear_grabbing_wall();
            self.follower_link_state_mut().clear_item_action_step_var();
            self.follower_link_state_mut().set_y_button_action_timer(2);
            self.follower_link_state_mut().set_y_button_action_step(0);
            self.follower_link_state_mut().set_state_bits(1);
            self.follower_link_state_mut().clear_picking_throw_state();
        }

        if self.game_state.player.follower_link.defense_flags() & 9 != 0 {
            self.link_state_tree_pull_reset_to_normal();
            return;
        }
        if self.game_state.player.follower_link.item_action_step_var() == 9 {
            if self.game_state.player.follower_link.filtered_joypad_h() & 0x0f == 0 {
                self.link_handle_cardinal_collision();
                self.handle_indoor_camera_and_doors();
                return;
            }
            self.follower_link_state_mut().clear_handler_state();
            self.link_state_default();
            return;
        }
        self.ancilla_add_dash_dust_charging(0x1e, 0);
        self.follower_link_state_mut()
            .decrement_y_button_action_timer();
        if (self.game_state.player.follower_link.y_button_action_timer() as i8) < 0 {
            let j = self
                .follower_link_state_mut()
                .increment_item_action_step_var() as usize;
            self.follower_link_state_mut()
                .set_y_button_action_step(GRAB_WALL_ANIM_STEPS2[j]);
            self.follower_link_state_mut().set_y_button_action_timer(2);
            self.follower_link_state_mut().set_actual_y_velocity(48);
            if j == 9 {
                self.link_state_tree_pull_reset_to_normal();
                return;
            }
        }
        self.flag67_with_directions();
        if self.game_state.player.follower_link.direction() & 3 == 0 {
            self.follower_link_state_mut().clear_actual_x_velocity();
        }
        if self.game_state.player.follower_link.direction() & 0x0c == 0 {
            self.follower_link_state_mut().clear_actual_y_velocity();
        }
        self.link_state_tree_pull_tail();
    }

    pub(super) fn link_handle_recoil_and_timer(&mut self, jump_into_middle: bool) {
        if !jump_into_middle {
            self.follower_link_state_mut().clear_page_movement_deltas();
            self.follower_link_state_mut()
                .clear_orthogonal_direction_count();
            self.link_handle_recoiling();
            if self
                .follower_link_state_mut()
                .decrement_incapacitated_timer()
                == 0
            {
                self.follower_link_state_mut()
                    .reset_elapsed_incapacitated_timer();
                if self
                    .game_state
                    .player
                    .follower_link
                    .is_recoil_landing_z_window()
                    && (self.game_state.player.follower_link.actual_z_velocity() as i8) < 0
                {
                    if self.game_state.player.follower_link.has_auxiliary_state() {
                        self.follower_link_state_mut()
                            .clear_sprite_damage_disable_timer();
                        let old_state = self.game_state.player.follower_link.handler_state();
                        self.tile_detect_position_mut()
                            .set_interaction_scratch_y(old_state as u16);
                        if self.game_state.player.follower_link.handler_state() != 6 {
                            self.follower_link_state_mut().clear_button_b_frames();
                            self.follower_link_state_mut().set_button_mask_b_y(0);
                            self.follower_link_state_mut()
                                .set_spin_attack_delay_timer(0);
                            self.follower_link_state_mut()
                                .clear_spin_attack_step_counter();
                        }
                        self.link_splash_upon_landing();
                        if !self.game_state.player.follower_link.is_bunny_mirror()
                            || !self.game_state.player.follower_link.is_in_deep_water()
                        {
                            if self.game_state.player.follower_link.dash_noise_requested() {
                                self.follower_link_state_mut().clear_dash_noise_request();
                                self.ancilla_sfx2_near(33);
                            } else if old_state != 2
                                && self.game_state.player.follower_link.handler_state() != 4
                            {
                                self.ancilla_sfx2_near(33);
                            }
                            if self.game_state.player.follower_link.handler_state() == 4 {
                                self.link_force_unequip_cape_quietly();
                                if self.game_state.world.location.is_indoors()
                                    && old_state != 2
                                    && self.game_state.inventory.items.has_flippers()
                                {
                                    self.follower_link_state_mut().mark_lower_level();
                                }
                                self.ancilla_add_splash(21, 0);
                            }
                            self.tile_detect_main_handler(0);
                            if self.game_state.player.tile_detection.thick_grass_low() & 1 != 0 {
                                self.ancilla_sfx2_near(26);
                            }
                            if self.game_state.player.tile_detection.shallow_water_low() & 1 != 0
                                && self.game_state.system_signals.sound_effect_1() != 36
                            {
                                self.ancilla_sfx2_near(28);
                            }
                            if self.game_state.player.tile_detection.deepwater() & 1 != 0 {
                                self.follower_link_state_mut().set_handler_state(4);
                                self.link_set_to_deep_water();
                                self.link_reset_sword_and_item_usage();
                                self.ancilla_add_splash(21, 0);
                            }
                        }
                        self.finish_recoil_landing();
                    }
                    self.follower_link_state_mut().clear_animation_step();
                    self.follower_link_state_mut().set_incapacitated_timer(0);
                }
            }
        } else {
            self.finish_recoil_landing();
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().set_incapacitated_timer(0);
        }

        if self.game_state.player.follower_link.handler_state() != 5
            && self.game_state.player.follower_link.incapacitated_timer() >= 33
        {
            if (self
                .follower_link_state_mut()
                .decrement_incapacitated_camera_timer() as i8)
                >= 0
            {
                self.handle_indoor_camera_and_doors();
                self.follower_link_state_mut().clear_z_high();
                return;
            }
            self.follower_link_state_mut()
                .reset_incapacitated_camera_timer_from_incapacitated();
        }

        self.flag67_with_directions();
        if self.game_state.player.follower_link.handler_state() != 6 {
            self.link_handle_diagonal_collision();
            if self.game_state.player.follower_link.direction() & 3 == 0 {
                self.follower_link_state_mut().clear_actual_x_velocity();
            }
            if self.game_state.player.follower_link.direction() & 0x0c == 0 {
                self.follower_link_state_mut().clear_actual_y_velocity();
            }
        }
        self.link_move_position();

        if self.game_state.player.follower_link.handler_state() != 6 {
            self.link_handle_cardinal_collision();
            self.follower_link_state_mut().clear_pit_correction();
        }
        self.handle_indoor_camera_and_doors();
        if self
            .game_state
            .player
            .follower_link
            .should_probe_recoil_landing_tile()
        {
            // ROM $07:885C
            self.player_tile_detect_nearby_marked();
            if self.game_state.player.tile_detection.pit_tile() & 0x0f == 0x0f {
                self.follower_link_state_mut().set_handler_state(1);
                self.follower_link_state_mut().set_speed_setting(4);
            }
        }
        self.follower_link_state_mut().clear_z_high();
    }

    pub(super) fn link_handle_recoiling(&mut self) {
        self.follower_link_state_mut().set_direction(0);
        if self.game_state.player.follower_link.actual_y_velocity() != 0 {
            let direction = if self
                .game_state
                .player
                .follower_link
                .actual_y_velocity_signed()
                .is_negative()
            {
                8
            } else {
                4
            };
            self.follower_link_state_mut()
                .add_direction_flags(direction);
            self.follower_link_state_mut()
                .set_last_direction_from_current_direction();
            self.player_handle_incapacitated_inner2();
        }
        if self.game_state.player.follower_link.actual_x_velocity() != 0 {
            let direction = if self
                .game_state
                .player
                .follower_link
                .actual_x_velocity_signed()
                .is_negative()
            {
                2
            } else {
                1
            };
            self.follower_link_state_mut()
                .add_direction_flags(direction);
            self.follower_link_state_mut()
                .set_last_direction_from_current_direction();
        }
        self.player_handle_incapacitated_inner2();
    }

    pub(super) fn player_handle_incapacitated_inner2(&mut self) {
        if self
            .game_state
            .player
            .follower_link
            .is_moving_against_diag_tile_on_both_axes()
            && self.game_state.player.follower_link.handler_state() == 2
        {
            self.follower_link_state_mut().invert_actual_velocity_xy();
        }
        if self.game_state.player.follower_link.doorway_state() == 1 {
            self.follower_link_state_mut().mask_last_direction(0x0c);
            self.follower_link_state_mut().mask_direction(0x0c);
            self.follower_link_state_mut().clear_actual_x_velocity();
        } else if self.game_state.player.follower_link.doorway_state() == 2 {
            self.follower_link_state_mut().mask_last_direction(3);
            self.follower_link_state_mut().mask_direction(3);
            self.follower_link_state_mut().clear_actual_y_velocity();
        }
    }

    pub(in crate::zelda_rtl) fn handle_layer_of_destination(&mut self) {
        let hole_teleporter_plane = self.game_state.dungeon.header.hole_teleporter_plane(0);
        self.follower_link_state_mut().set_lower_level_states(
            u8::from(hole_teleporter_plane >= 2),
            u8::from(hole_teleporter_plane >= 1),
        );
    }

    pub(in crate::zelda_rtl) fn dungeon_pit_do_damage(&mut self) {
        self.set_submodule(20);
        if self.player_resources_mut().decrement_current_health_by(8) >= 0xa8 {
            self.player_resources_mut().set_current_health(0);
        }
    }

    pub(in crate::zelda_rtl) fn reset_some_things_after_death(&mut self, speed_setting: u8) {
        self.follower_link_state_mut().clear_deep_water_state();
        self.follower_link_state_mut()
            .set_speed_setting(speed_setting);
        self.follower_link_state_mut().clear_conveyor_belt_state();
        self.set_player_layer_collision_flags(0);
        self.follower_link_state_mut().clear_immobilized();
        self.follower_state_mut().clear_palette_swap_flag();
        self.follower_link_state_mut()
            .clear_faint_animation_active();
        self.follower_link_state_mut().clear_given_damage();
        self.follower_link_state_mut().clear_actual_velocity_xy();
        self.follower_link_state_mut().set_actual_z_velocity(0);
        // C writes `link_z_coord = 0` as a single byte (LINK_Z_COORD low only); the
        // high byte 0x25 is left stale. Use set_z_low, not the 16-bit set_z, so we
        // don't clobber 0x25 (mode-reused; e.g. carries Link-Z high into the intro).
        self.follower_link_state_mut().set_z_low(0);
        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        self.tile_detect_position_mut()
            .set_tile_collision_bits_primary(0);
        self.follower_link_state_mut().clear_blink_countdown();
        self.follower_link_state_mut().clear_handler_state();
        self.follower_link_state_mut().set_visibility_status(0);
        self.ancilla_terminate_select_interactives(0);
        self.link_reset_properties_a();
    }
}
