//! Swimming, movement animation, velocity integration, slopes, and direction limits.

use super::*;
use crate::game_state::{CollisionAxis, CollisionDirection, CollisionOrder, MovementProbeKind, PlayerAxis};
use crate::tile_definition::NativeTile;

impl ZeldaState {
    pub(in crate::zelda_rtl) fn link_handle_swim_accels(&mut self) {
        let mut mask = 0x0c;
        for offset in [0, 2] {
            if self.game_state.player.follower_link.joypad1h_last() & mask != 0 {
                let acceleration = self
                    .game_state
                    .player
                    .swim_acceleration
                    .acceleration(offset);
                let max_speed = self.game_state.player.swim_acceleration.max_speed(offset);
                if acceleration != 0 && max_speed >= 384 {
                    let target = LINK_HANDLE_SWIM_ACCELS_SWIM_ACCELERATION_TARGETS
                        .iter()
                        .copied()
                        .find(|value| *value >= acceleration)
                        .unwrap_or(384);
                    self.swim_acceleration_mut().set_max_speed(offset, target);
                } else if max_speed != 0 {
                    let target = max_speed.wrapping_add(160).min(384);
                    self.swim_acceleration_mut().set_max_speed(offset, target);
                } else {
                    self.swim_acceleration_mut().set_acceleration(offset, 1);
                    self.swim_acceleration_mut().set_max_speed(offset, 240);
                }
            }
            mask >>= 2;
        }
    }

    pub(super) fn link_flag_max_accels(&mut self) {
        if self.game_state.player.follower_link.flag_moving() == 0 {
            return;
        }

        for offset in [2, 0] {
            let acceleration = self
                .game_state
                .player
                .swim_acceleration
                .acceleration(offset);
            if acceleration != 0 {
                self.swim_acceleration_mut()
                    .set_max_speed(offset, acceleration);
                self.swim_acceleration_mut().set_mode(offset, 1);
            }
        }
    }

    pub(super) fn link_set_ice_max_accel(&mut self) {
        if self.game_state.player.follower_link.flag_moving() == 0 {
            return;
        }
        self.swim_acceleration_mut().set_max_speed(0, 0x0180);
        self.swim_acceleration_mut().set_max_speed(2, 0x0180);
    }

    pub(in crate::zelda_rtl) fn link_set_momentum(&mut self) {
        let joy = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
        let mut mask = 0x0c;
        let mut bit = 0x08;
        for offset in [0, 2] {
            if joy & mask != 0 {
                let flag_moving = self.game_state.player.follower_link.flag_moving();
                let stroke_timer = if flag_moving != 0 {
                    LINK_SET_MOMENTUM_SWIM_STROKE_TIMERS_BY_MOVING_FLAG[(flag_moving - 1) as usize]
                } else {
                    32
                };
                self.follower_link_state_mut()
                    .set_swim_stroke_frame_counter(offset, stroke_timer as u16);

                if (self.game_state.player.follower_link.swim_direction_flags()
                    | self.game_state.player.follower_link.direction())
                    & mask
                    == mask
                {
                    self.swim_acceleration_mut().set_mode(offset, 2);
                } else {
                    let direction = if joy & bit != 0 { 0 } else { 1 };
                    self.swim_acceleration_mut()
                        .set_acceleration_direction(offset, direction);
                    self.swim_acceleration_mut().set_mode(offset, 0);
                }

                if self.game_state.player.swim_acceleration.max_speed(offset) == 0 {
                    self.swim_acceleration_mut().set_max_speed(offset, 240);
                }
            }
            mask >>= 2;
            bit >>= 2;
        }
    }

    pub(in crate::zelda_rtl) fn player_handler_04_swimming(&mut self) {
        if self.game_state.player.follower_link.auxiliary_state() != 0 {
            self.follower_link_state_mut()
                .interrupt_swimming_for_auxiliary_state();
            self.reset_all_acceleration();
            self.link_state_recoil();
            return;
        }

        self.follower_link_state_mut().clear_swimming_action_state();
        if !self.game_state.inventory.items.has_flippers() {
            return;
        }

        let has_swim_velocity = self.game_state.player.swim_acceleration.acceleration(0)
            | self.game_state.player.swim_acceleration.acceleration(2)
            != 0;
        if !has_swim_velocity {
            let swim = &self.game_state.player.swim_acceleration;
            if swim.mode_low(0) != 2 && swim.mode_low(1) != 2 {
                self.reset_all_acceleration();
            }
            self.follower_link_state_mut().advance_idle_swim_animation();
        } else {
            self.follower_link_state_mut()
                .advance_active_swim_animation(
                    &PLAYER_HANDLER_04_SWIMMING_ACTIVE_SWIM_ANIMATION_DELAYS,
                );
        }

        if self.game_state.player.follower_link.hard_swim_stroke() == 0 {
            let hard_stroke = ((self.game_state.player.follower_link.filtered_joypad_l() & 0x80)
                | self.game_state.player.follower_link.filtered_joypad_h())
                & 0xc0;
            if !has_swim_velocity || hard_stroke == 0 {
                self.link_handle_swim_movements();
                return;
            }
            self.follower_link_state_mut()
                .start_hard_swim_stroke(hard_stroke);
            self.ancilla_sfx2_near(37);
            self.link_handle_swim_accels();
        }

        self.follower_link_state_mut().tick_hard_swim_stroke();
        self.link_handle_swim_movements();
    }

    fn advance_link_animation_step(&mut self, delay: u8, wrap_at: u8, wrap_to: u8) {
        if self
            .follower_link_state_mut()
            .advance_frame_change_counter(delay)
        {
            self.follower_link_state_mut()
                .advance_animation_step(wrap_at, wrap_to);
        }
    }

    fn advance_link_animation_step_at_least(&mut self, delay: u8, wrap_at: u8, wrap_to: u8) {
        if self
            .follower_link_state_mut()
            .advance_frame_change_counter(delay)
        {
            self.follower_link_state_mut()
                .advance_animation_step_at_least(wrap_at, wrap_to);
        }
    }

    pub(in crate::zelda_rtl) fn link_handle_swim_movements(&mut self) {
        let mut direction = (self
            .game_state
            .player
            .follower_link
            .force_move_any_direction() as u8)
            & 0x0f;
        if direction == 0 {
            direction = self.game_state.player.follower_link.joypad1h_last() & 0x0f;
        }

        if direction == 0 {
            self.follower_link_state_mut()
                .clear_swim_movement_velocity();
            self.link_flag_max_accels();
            if self.game_state.player.follower_link.flag_moving() != 0 {
                if self.game_state.player.follower_link.is_running() {
                    direction = self.game_state.player.follower_link.swim_direction_flags();
                } else {
                    if self.game_state.player.swim_acceleration.acceleration(0)
                        | self.game_state.player.swim_acceleration.acceleration(2)
                        == 0
                    {
                        self.follower_link_state_mut().clear_defense_flags();
                        self.link_reset_swimming_state();
                    }
                    self.finish_swim_movement_tail();
                    return;
                }
            } else {
                self.follower_link_state_mut()
                    .reset_idle_swim_animation_if_out_of_water();
                self.finish_swim_movement_tail();
                return;
            }
        }

        if direction != self.game_state.player.follower_link.swim_direction_flags() {
            self.follower_link_state_mut()
                .set_swim_direction_flags(direction);
            self.follower_link_state_mut()
                .reset_swim_subpixel_and_defense_state();
        }
        self.link_set_ice_max_accel();
        self.link_set_momentum();
        self.link_set_the_max_accel();
        self.finish_swim_movement_tail();
    }

    fn finish_swim_movement_tail(&mut self) {
        self.link_handle_diagonal_collision();
        self.link_handle_velocity();
        self.link_handle_cardinal_collision();
        self.link_handle_moving_animation_full_long_entry();
        self.follower_link_state_mut().clear_pit_correction();
        self.handle_indoor_camera_and_doors();
    }

    pub(super) fn link_set_the_max_accel(&mut self) {
        if self.game_state.player.follower_link.flag_moving() != 0
            || self.game_state.player.follower_link.hard_swim_stroke() != 0
        {
            return;
        }

        let mut mask = 0x0c;
        for offset in [0, 2] {
            let mode = self.game_state.player.swim_acceleration.mode(offset);
            if self.game_state.player.follower_link.joypad1h_last() & mask != 0 && mode != 2 {
                let speed_active = self
                    .game_state
                    .player
                    .swim_acceleration
                    .speed_active_flag(offset);
                let acceleration = self
                    .game_state
                    .player
                    .swim_acceleration
                    .acceleration(offset);
                let max_speed = self.game_state.player.swim_acceleration.max_speed(offset);
                if speed_active != 0 || (acceleration >= 240 && acceleration >= max_speed) {
                    self.swim_acceleration_mut().set_mode(offset, 0);
                    if acceleration >= 240 {
                        self.swim_acceleration_mut()
                            .set_speed_active_flag(offset, 1);
                        self.swim_acceleration_mut().set_mode(offset, 1);
                    } else {
                        self.swim_acceleration_mut().set_max_speed(offset, 240);
                        self.swim_acceleration_mut()
                            .set_speed_active_flag(offset, 0);
                    }
                }
            } else {
                self.swim_acceleration_mut().set_max_speed(offset, 240);
                self.swim_acceleration_mut()
                    .set_speed_active_flag(offset, 0);
            }
            mask >>= 2;
        }
    }

    pub(in crate::zelda_rtl) fn link_handle_toss(&mut self) -> bool {
        if self.game_state.player.follower_link.y_button_action_flags() & 0x80 == 0
            || self.game_state.player.follower_link.filtered_joypad_l() & 0x80 == 0
            || self.game_state.player.follower_link.is_lift_throw_primed()
        {
            return false;
        }

        self.follower_link_state_mut()
            .clear_lift_throw_scratch_state();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_y_button_action_flags(0);
        self.follower_link_state_mut().clear_direction_lock_bits(1);
        true
    }

    pub(in crate::zelda_rtl) fn link_cancel_dash(&mut self) {
        if !self.game_state.player.follower_link.is_running() {
            return;
        }
        for i in (0..=4).rev() {
            if self.ancilla_slot_view(i).ancilla_type() == 0x1e {
                self.ancilla_slot_view_mut(i).set_ancilla_type(0);
            }
        }
        self.follower_link_state_mut().cancel_dash_state();
        self.swim_acceleration_mut().clear_mode_low_axis();
    }

    pub(in crate::zelda_rtl) fn repel_dash(&mut self) {
        if self.game_state.player.follower_link.is_running()
            && self.game_state.player.follower_link.dash_counter() != 64
        {
            self.link_reset_swimming_state();
            self.ancilla_add_dash_tremor(29, 1);
            self.prepare_apply_rumble_to_sprites();
            if self.game_state.system_signals.sound_effect_2() & 0x3f != 27
                && self.game_state.system_signals.sound_effect_2() & 0x3f != 50
            {
                self.ancilla_sfx3_near(3);
            }
            self.link_apply_tile_rebound();
        }
    }

    pub(in crate::zelda_rtl) fn sprite_repel_dash(&mut self) {
        self.follower_link_state_mut()
            .set_last_direction_moved_towards_from_facing();
        self.repel_dash();
    }

    pub(super) fn link_apply_tile_rebound(&mut self) {
        let dir = self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards() as usize;
        self.follower_link_state_mut().set_actual_velocity_xy(
            LINK_APPLY_TILE_REBOUND_DASH_REBOUND_X_VELOCITY_BY_DIRECTION[dir],
            LINK_APPLY_TILE_REBOUND_DASH_REBOUND_Y_VELOCITY_BY_DIRECTION[dir],
        );
        self.follower_link_state_mut().set_incapacitated_timer(24);
        self.follower_link_state_mut()
            .set_actual_z_velocity_and_copy(36);
        if self.game_state.player.follower_link.flag_moving() != 0 {
            self.follower_link_state_mut().set_direction_and_swim_flags(
                LINK_APPLY_TILE_REBOUND_DIRECTION_BITS_BY_FACING[dir],
            );
            self.swim_acceleration_mut().set_acceleration_direction(
                0,
                LINK_APPLY_TILE_REBOUND_DASH_REBOUND_SWIM_Y_DIRECTIONS[dir] as u16,
            );
            self.swim_acceleration_mut().set_acceleration_direction(
                2,
                LINK_APPLY_TILE_REBOUND_DASH_REBOUND_SWIM_X_DIRECTIONS[dir] as u16,
            );
            let i = (self.game_state.player.follower_link.flag_moving() - 1) as usize * 4 + dir;
            self.swim_acceleration_mut().set_acceleration(
                0,
                LINK_APPLY_TILE_REBOUND_DASH_REBOUND_SWIM_Y_ACCELERATIONS[i],
            );
            self.swim_acceleration_mut().set_acceleration(
                2,
                LINK_APPLY_TILE_REBOUND_DASH_REBOUND_SWIM_X_ACCELERATIONS[i],
            );
        }
        self.follower_link_state_mut().set_auxiliary_state(1);
        self.follower_link_state_mut().set_dash_noise_request();
        self.tile_detect_position_mut()
            .clear_interaction_scratch_x_low();
        self.follower_link_state_mut().clear_electrocute_on_touch();
        self.follower_link_state_mut().set_speed_setting(0);
        self.follower_link_state_mut().clear_direction_lock();
        self.follower_link_state_mut()
            .clear_moving_against_diag_tile();
        if self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards()
            & 2
            != 0
        {
            self.follower_link_state_mut().set_y_velocity(0);
        } else {
            self.follower_link_state_mut().set_x_velocity(0);
        }
    }

    pub(in crate::zelda_rtl) fn link_handle_moving_animation_full_long_entry(&mut self) {
        crate::cycle_ledger::charge_routine(0x07_e6a6,
            crate::cycle_models::player_animation::full_entry_master_cycles(&self.ram));
        if self.game_state.player.follower_link.handler_state() == 4 {
            self.link_handle_moving_animation_swimming();
            return;
        }

        let mut r0 = self.game_state.player.follower_link.last_direction();
        if r0 == 0 {
            return;
        }
        if self.game_state.player.follower_link.flag_moving() != 0 {
            r0 = self.game_state.player.follower_link.swim_direction_flags();
        }
        if self.game_state.player.follower_link.direction_lock() == 0 {
            let mut y;
            if self
                .game_state
                .player
                .follower_link
                .num_orthogonal_directions()
                == 0
            {
                y = if r0 & 0x0c != 0 { 0 } else { 4 };
            } else if self.game_state.player.follower_link.doorway_state() != 0 {
                y = self
                    .game_state
                    .player
                    .follower_link
                    .doorway_state()
                    .wrapping_mul(2)
                    & !3;
            } else if r0
                & LINK_HANDLE_MOVING_ANIMATION_FULL_LONG_ENTRY_FACING_DIRECTION_BITS
                    [self.game_state.player.follower_link.facing_index()]
                != 0
            {
                self.link_handle_moving_animation_start_with_dash();
                return;
            } else {
                y = if r0 & 0x0c != 0 { 0 } else { 4 };
            }
            if y != 4 {
                y = y.wrapping_add(if r0 & 4 != 0 { 2 } else { 0 });
            } else {
                y = y.wrapping_add(if r0 & 1 != 0 { 2 } else { 0 });
            }
            self.follower_link_state_mut().set_facing(y);
        }
        self.link_handle_moving_animation_start_with_dash();
    }

    pub(in crate::zelda_rtl) fn link_handle_moving_animation_start_with_dash(&mut self) {
        if self.game_state.player.follower_link.is_running() {
            self.link_handle_moving_animation_dash();
            return;
        }
        let mut x = self.game_state.player.follower_link.facing() >> 1;
        if self.game_state.player.follower_link.speed_setting() == 6 {
            x = x.wrapping_add(4);
        } else if self.game_state.player.follower_link.flag_moving() != 0 {
            if self.game_state.player.follower_link.joypad1h_last() & 0x0f == 0 {
                self.follower_link_state_mut().clear_animation_step();
                return;
            }
            x = x.wrapping_add(4);
        }

        if self.game_state.player.follower_link.handler_state() == 23
            || (self.game_state.enhanced_features.has(4096)
                && self.game_state.player.follower_link.handler_state() == 28)
        {
            if self.game_state.player.follower_link.animation_step() < 4
                && self.game_state.player.follower_link.on_somaria_platform() != 2
            {
                self.advance_link_animation_step(
                    LINK_HANDLE_MOVING_ANIMATION_START_WITH_DASH_POSE_ANIMATION_DELAYS[x as usize],
                    4,
                    0,
                );
            } else {
                self.follower_link_state_mut().clear_animation_step();
            }
            return;
        }

        if self.game_state.frame.submodule == 18 || self.game_state.frame.submodule == 19 {
            x = 12;
        } else if self.game_state.frame.submodule != 14
            && !self
                .game_state
                .player
                .follower_link
                .is_lifting_or_carrying()
        {
            if self.game_state.player.follower_link.defense_flags() & 0x8d != 0 {
                x = 12;
            } else if self
                .game_state
                .player
                .follower_link
                .water_ripple_or_grass_state()
                == 0
                && self.game_state.player.follower_link.button_b_frames() == 0
            {
                let mut idx = self.game_state.player.follower_link.animation_step();
                if self.game_state.player.follower_link.speed_setting() == 6 {
                    idx = idx.wrapping_add(8);
                }
                if self.game_state.player.follower_link.flag_moving() != 0 {
                    idx = idx.wrapping_add(8);
                }
                if self.game_state.player.follower_link.on_somaria_platform() != 2 {
                    self.advance_link_animation_step(
                        LINK_HANDLE_MOVING_ANIMATION_START_WITH_DASH_WALK_ANIMATION_DELAYS
                            [idx as usize],
                        9,
                        1,
                    );
                }
                return;
            }
        }

        if self.game_state.player.follower_link.animation_step() < 6
            && self.game_state.player.follower_link.on_somaria_platform() != 2
        {
            self.advance_link_animation_step(
                LINK_HANDLE_MOVING_ANIMATION_START_WITH_DASH_POSE_ANIMATION_DELAYS[x as usize],
                6,
                0,
            );
        } else {
            self.follower_link_state_mut().clear_animation_step();
        }
    }

    pub(super) fn link_handle_moving_animation_swimming(&mut self) {
        let r0 = self.game_state.player.follower_link.swim_direction_flags();
        if r0 == 0 || self.game_state.player.follower_link.direction_lock() != 0 {
            return;
        }
        let mut y;
        if self
            .game_state
            .player
            .follower_link
            .num_orthogonal_directions()
            != 0
        {
            if self.game_state.player.follower_link.doorway_state() != 0 {
                y = self
                    .game_state
                    .player
                    .follower_link
                    .doorway_state()
                    .wrapping_mul(2)
                    & !3;
            } else if r0
                & LINK_HANDLE_MOVING_ANIMATION_SWIMMING_FACING_DIRECTION_BITS
                    [self.game_state.player.follower_link.facing_index()]
                != 0
            {
                return;
            } else {
                y = if r0 & 0x0c != 0 { 0 } else { 4 };
            }
        } else {
            y = if r0 & 0x0c != 0 { 0 } else { 4 };
        }
        if y != 4 {
            y = y.wrapping_add(if r0 & 4 != 0 { 2 } else { 0 });
        } else {
            y = y.wrapping_add(if r0 & 1 != 0 { 2 } else { 0 });
        }
        self.follower_link_state_mut().set_facing(y);
    }

    pub(super) fn link_handle_moving_animation_dash(&mut self) {
        let mut t = 6usize;
        while self.game_state.player.follower_link.dash_countdown()
            >= LINK_HANDLE_MOVING_ANIMATION_DASH_DASH_CHARGE_ANIM_THRESHOLDS[t]
            && t != 0
        {
            t -= 1;
        }
        if self.game_state.player.follower_link.button_b_frames() < 9
            && self
                .game_state
                .player
                .follower_link
                .water_ripple_or_grass_state()
                == 0
        {
            self.advance_link_animation_step(
                LINK_HANDLE_MOVING_ANIMATION_DASH_DASH_CHARGE_ANIM_DELAYS[t * 8],
                9,
                1,
            );
        } else {
            self.advance_link_animation_step_at_least(
                LINK_HANDLE_MOVING_ANIMATION_DASH_DASH_CHARGE_MIN_ANIM_STEPS[t],
                6,
                0,
            );
        }
    }

    pub(super) fn link_apply_moving_floor_velocity(&mut self) {
        self.follower_link_state_mut()
            .clear_orthogonal_direction_count();
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(self.game_state.dungeon.moving_floor.floor_y_velocity());
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(self.game_state.dungeon.moving_floor.floor_x_velocity());
        self.follower_link_state_mut().set_y(y);
        self.follower_link_state_mut().set_x(x);
    }

    pub(super) fn link_apply_conveyor(&mut self) {
        if self.game_state.player.follower_link.conveyor_belt_state() == 0 {
            return;
        }
        if !self
            .game_state
            .player
            .follower_link
            .is_grounded_or_z_sentinel()
        {
            return;
        }
        if self.game_state.player.follower_link.grabbing_wall_has(1)
            || self.game_state.player.follower_link.handler_state() == 19
            || self.game_state.player.follower_link.has_auxiliary_state()
        {
            return;
        }
        let j = self
            .game_state
            .player
            .follower_link
            .conveyor_belt_state()
            .wrapping_sub(1) as usize;
        if j >= LINK_APPLY_CONVEYOR_MOVE_POS_DIR_FLAG.len() {
            return;
        }
        if self.game_state.player.follower_link.is_running()
            && self.game_state.player.follower_link.dash_counter() == 32
            && self.game_state.player.follower_link.direction()
                & LINK_APPLY_CONVEYOR_MOVE_POS_DIR_FLAG[j]
                != 0
        {
            return;
        }
        self.follower_link_state_mut()
            .clear_orthogonal_direction_count();
        self.follower_link_state_mut()
            .add_direction_flags(LINK_APPLY_CONVEYOR_MOVE_POS_DIR_FLAG[j]);
        let y = ((self.game_state.player.follower_link.y() as u32) << 8
            | self.game_state.player.bg1_movement_accumulator.y_subpixel() as u32)
            .wrapping_add(((LINK_APPLY_CONVEYOR_MOVING_BELT_Y[j] as i32) << 4) as u32);
        self.bg1_move_calc_mut().set_y_subpixel(y as u8);
        self.follower_link_state_mut().set_y((y >> 8) as u16);
        let x = ((self.game_state.player.follower_link.x() as u32) << 8
            | self.game_state.player.bg1_movement_accumulator.x_subpixel() as u32)
            .wrapping_add(((LINK_APPLY_CONVEYOR_MOVING_BELT_X[j] as i32) << 4) as u32);
        self.bg1_move_calc_mut().set_x_subpixel(x as u8);
        self.follower_link_state_mut().set_x((x >> 8) as u16);
    }

    pub(in crate::zelda_rtl) fn flag67_with_directions(&mut self) {
        self.follower_link_state_mut()
            .derive_direction_from_actual_velocity();
    }

    pub(super) fn link_add_in_velocity_y_falling(&mut self) {
        let adjust = i16::from(self.game_state.player.tile_detection.y_low() & 7)
            - if self
                .game_state
                .player
                .follower_link
                .y_velocity_signed()
                .is_negative()
            {
                8
            } else {
                0
            };
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_sub(adjust as u16);
        self.follower_link_state_mut().set_y(y);
    }

    pub(super) fn link_add_in_velocity_y(&mut self) {
        let y =
            self.game_state.player.follower_link.y().wrapping_sub(
                self.game_state.player.follower_link.y_velocity() as i8 as i16 as u16,
            );
        self.follower_link_state_mut().set_y(y);
    }

    pub(super) fn player_change_z(&mut self, z_delta: u8) {
        if (self.game_state.player.follower_link.actual_z_velocity() as i8).is_negative() {
            if self.game_state.player.follower_link.z_low() == 0 {
                return;
            }
            if (self.game_state.player.follower_link.z_low() as i8).is_negative() {
                self.follower_link_state_mut().set_z(0xffff);
                self.follower_link_state_mut().set_actual_z_velocity(0xff);
                return;
            }
        }
        self.follower_link_state_mut()
            .decrement_actual_z_velocity(z_delta);
    }

    pub(in crate::zelda_rtl) fn link_move_position(&mut self) {
        if let Some(position_return) = self.link_move_position_until_coordinates_integrated() {
            self.complete_link_move_position_after_coordinates(position_return);
        }
    }

    fn link_move_position_until_coordinates_integrated(
        &mut self,
    ) -> Option<LinkMovePositionReturn> {
        let x = self.game_state.player.follower_link.x();
        let y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .store_safe_return_position(x, y);

        if self.game_state.player.follower_link.handler_state() != 10
            && self.game_state.player.follower_link.on_somaria_platform() == 2
        {
            self.link_handle_velocity_and_sand_drag(x, y);
            return None;
        }

        let actual_x_velocity = self.game_state.player.follower_link.actual_x_velocity();
        let actual_y_velocity = self.game_state.player.follower_link.actual_y_velocity();
        let airborne = self.game_state.player.follower_link.auxiliary_state() != 0;
        let actual_z_velocity = self.game_state.player.follower_link.actual_z_velocity();
        // No external observer runs between these stores. Observe legacy writes
        // once and keep the ordinary X/Y/Z publication order within this borrow.
        let mut player = self.follower_link_state_mut();
        player.move_x_by_velocity(actual_x_velocity);
        player.move_y_by_velocity(actual_y_velocity);
        if airborne {
            player.move_z_by_velocity(actual_z_velocity);
        }

        Some(LinkMovePositionReturn { old_x: x, old_y: y })
    }

    pub(in crate::zelda_rtl) fn complete_link_move_position_after_coordinates(
        &mut self,
        position_return: LinkMovePositionReturn,
    ) {
        self.link_handle_moving_floor();
        self.link_apply_conveyor();
        self.link_handle_velocity_and_sand_drag(position_return.old_x, position_return.old_y);
    }

    pub(super) fn link_handle_velocity_and_sand_drag(&mut self, old_x: u16, old_y: u16) {
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(self.game_state.player.follower_link.drag_player_y());
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(self.game_state.player.follower_link.drag_player_x());
        let mut player = self.follower_link_state_mut();
        player.set_y(y);
        player.set_x(x);
        player.set_movement_velocity_from_position_delta(x, y, old_x, old_y);
    }

    pub(super) fn link_handle_moving_floor(&mut self) {
        if self.game_state.dungeon.room_load.header_collision() == 0 {
            return;
        }
        let z = self.game_state.player.follower_link.z_low();
        if z != 0 && z != 0xff {
            return;
        }
        if !self
            .has_player_layer_collision(crate::game_state::constants::player::LAYER_COLLISION_BOTH)
        {
            return;
        }
        if self.game_state.player.follower_link.handler_state() == 19 {
            return;
        }

        let floor_y = self.game_state.dungeon.moving_floor.floor_y_velocity();
        let floor_x = self.game_state.dungeon.moving_floor.floor_x_velocity();
        self.follower_link_state_mut()
            .mark_moving_floor_direction(floor_y, floor_x);

        self.link_apply_moving_floor_velocity();
    }

    pub(super) fn check_if_room_needs_double_layer_check(&mut self) -> bool {
        if self.game_state.dungeon.room_load.header_collision() == 0
            || self.game_state.dungeon.room_load.header_collision() == 4
        {
            return false;
        }

        if self.game_state.dungeon.room_load.header_collision() >= 2 {
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(self.game_state.display.ppu_scroll_copy.bg1_v_copy2())
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
            self.follower_link_state_mut().set_y(y);

            let x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(self.game_state.display.ppu_scroll_copy.bg1_h_copy2())
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
            self.follower_link_state_mut().set_x(x);
            self.follower_link_state_mut()
                .cache_moving_floor_position(x, y);
        }
        self.follower_link_state_mut().mark_lower_level();
        true
    }

    pub(super) fn create_velocity_from_moving_background(&mut self) {
        if self.game_state.dungeon.room_load.header_collision() != 1 {
            let x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_sub(self.game_state.player.follower_link.moving_floor_x());
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_sub(self.game_state.player.follower_link.moving_floor_y());
            let new_y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg1_v_copy2());
            let new_x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(self.game_state.display.ppu_scroll_copy.bg2_h_copy2())
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg1_h_copy2());
            self.follower_link_state_mut().set_y(new_y);
            self.follower_link_state_mut().set_x(new_x);
            if self.game_state.player.follower_link.direction() != 0 {
                self.follower_link_state_mut()
                    .add_movement_velocity_delta(x, y);
            }
        }
        self.follower_link_state_mut().clear_lower_level();
    }

    pub(super) fn calculate_snap_scratch_y(&mut self) {
        let mut y_vel = self.game_state.player.follower_link.y_velocity() as i8;
        if self.game_state.player.tile_detection.collision_bits() & 4 != 0 {
            if y_vel >= 0 {
                y_vel = y_vel.wrapping_neg();
            }
        } else if y_vel < 0 {
            y_vel = y_vel.wrapping_neg();
        }

        let x = self.game_state.player.follower_link.x();
        let delta = if y_vel < 0 { -1i16 } else { 1i16 };
        self.follower_link_state_mut()
            .set_x(x.wrapping_add(delta as u16));
    }

    pub(super) fn change_axis_of_perpendicular_door_movement_y(&mut self) {
        self.follower_link_state_mut().set_direction_lock_bits(2);
        let r14 = self.game_state.player.tile_detection.collision_bits();
        let t = ((r14 | (r14 >> 4)) & 0x0f) as u8;
        if t & 7 == 0 {
            self.follower_link_state_mut().clear_doorway_state();
            return;
        }

        let mut t = self.game_state.player.follower_link.y_velocity();
        let dir = if self.game_state.player.follower_link.x_low() >= 0x80 {
            if t & 0x80 == 0 {
                t = t.wrapping_neg();
            }
            4
        } else {
            if t & 0x80 != 0 {
                t = t.wrapping_neg();
            }
            6
        };
        let vel = if t & 0x80 != 0 { -1i16 } else { 1i16 };
        if !self.game_state.player.follower_link.direction_lock_has(1) {
            self.follower_link_state_mut().set_facing(dir);
        }
        let x = self.game_state.player.follower_link.x();
        self.follower_link_state_mut()
            .set_x(x.wrapping_add(vel as u16));
    }

    pub(super) fn snap_on_x(&mut self) {
        let x = self.game_state.player.follower_link.x();
        let adjust = (x & 7).wrapping_sub(
            if (self.game_state.player.follower_link.x_velocity() as i8).is_negative() {
                8
            } else {
                0
            },
        );
        self.follower_link_state_mut().set_x(x.wrapping_sub(adjust));
    }

    pub(super) fn calculate_snap_scratch_x(&mut self) {
        let mut x_vel = self.game_state.player.follower_link.x_velocity() as i8;
        if self.game_state.player.tile_detection.collision_bits() & 4 != 0 {
            if x_vel >= 0 {
                x_vel = x_vel.wrapping_neg();
            }
        } else if x_vel < 0 {
            x_vel = x_vel.wrapping_neg();
        }

        let y = self.game_state.player.follower_link.y();
        let delta = if x_vel < 0 { -1i16 } else { 1i16 };
        self.follower_link_state_mut()
            .set_y(y.wrapping_add(delta as u16));
    }

    pub(super) fn change_axis_of_perpendicular_door_movement_x(&mut self) -> i8 {
        self.follower_link_state_mut().set_direction_lock_bits(2);
        let r14 = self.game_state.player.tile_detection.collision_bits();
        let r0 = ((r14 | (r14 >> 4)) & 0x0f) as u8;
        if r0 & 7 == 0 {
            self.follower_link_state_mut().clear_doorway_state();
            return r0 as i8;
        }

        let mut x_vel = self.game_state.player.follower_link.x_velocity_signed();
        let dir = if self.game_state.player.follower_link.y_low() >= 0x80 {
            if x_vel >= 0 {
                x_vel = x_vel.wrapping_neg();
            }
            0
        } else {
            if x_vel < 0 {
                x_vel = x_vel.wrapping_neg();
            }
            2
        };
        if !self.game_state.player.follower_link.direction_lock_has(1) {
            self.follower_link_state_mut().set_facing(dir);
        }
        let y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .set_y(y.wrapping_add(x_vel as i16 as u16));
        x_vel
    }

    pub(super) fn tile_behavior_handle_item_and_execute(&mut self, x: u16, y: u16) {
        let tile = self.handle_item_tile_action_overworld(x, y);
        self.tile_detect_execute_definition(tile, 0, 1, false);
    }

    pub(in crate::zelda_rtl) fn push_block_target_tile(&self, x: u16, y: u16) -> NativeTile {
        let offset = ((y & !7) as usize) * 8
            + (x & 0x3f) as usize
            + if self.game_state.player.follower_link.is_on_lower_level() {
                0x1000
            } else {
                0
            };
        self.game_state.dungeon.bg2_attributes.bg2_tile(offset)
    }

    pub(super) fn link_handle_change_in_z_velocity(&mut self) {
        self.player_change_z(
            if self.game_state.player.follower_link.handler_state() == 19 {
                1
            } else {
                2
            },
        );
    }

    pub(in crate::zelda_rtl) fn run_slope_collision_checks(&mut self, order: CollisionOrder) {
        for axis in order.axes() {
            // The first axis may change these flags: read again for the second.
            if !axis.blocked_by_slope(
                self.game_state
                    .player
                    .follower_link
                    .moving_against_diag_tile(),
            ) {
                self.start_movement_collision_checks(axis);
            }
        }
    }

    pub(super) fn link_hop_in_or_out_of_water_y(&mut self) {
        let ts = if self.game_state.world.location.is_outdoors() {
            2
        } else if self
            .game_state
            .player
            .follower_link
            .about_to_jump_off_ledge()
            != 0
        {
            0
        } else {
            self.game_state.display.sub_screen_layers
        };

        let mut vel = LINK_HOP_IN_OR_OUT_OF_WATER_Y_RECOIL_VEL_Y[ts as usize];
        if self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards()
            == 0
        {
            vel = 0u8.wrapping_sub(vel);
        }

        self.follower_link_state_mut()
            .set_actual_velocity_xy(0, vel);
        self.follower_link_state_mut()
            .set_actual_z_velocity_and_copy(
                LINK_HOP_IN_OR_OUT_OF_WATER_Y_RECOIL_VEL_Z[ts as usize],
            );
        self.follower_link_state_mut().set_z(0);
        self.follower_link_state_mut().set_incapacitated_timer(16);
        self.follower_link_state_mut().enter_water_hop_state();
    }

    pub(super) fn link_hop_in_or_out_of_water_x(&mut self) {
        let ts = if self.game_state.world.location.is_outdoors() {
            2
        } else if self
            .game_state
            .player
            .follower_link
            .about_to_jump_off_ledge()
            != 0
        {
            0
        } else {
            self.game_state.display.sub_screen_layers
        };

        let mut vel = LINK_HOP_IN_OR_OUT_OF_WATER_X_RECOIL_VEL_X[ts as usize];
        if self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards()
            & 1
            == 0
        {
            vel = 0u8.wrapping_sub(vel);
        }
        self.follower_link_state_mut()
            .set_actual_velocity_xy(vel, 0);
        self.follower_link_state_mut()
            .set_actual_z_velocity_and_copy(
                LINK_HOP_IN_OR_OUT_OF_WATER_X_RECOIL_VEL_Z[ts as usize],
            );
        self.follower_link_state_mut().set_incapacitated_timer(16);
        self.follower_link_state_mut().enter_water_hop_state();
    }

    pub(in crate::zelda_rtl) fn run_ledge_hop_timer(&mut self) -> bool {
        let mut rv = false;
        if !self
            .game_state
            .player
            .follower_link
            .is_in_auxiliary_state(1)
        {
            if !self.game_state.player.follower_link.is_running() {
                if self
                    .follower_link_state_mut()
                    .tick_jump_ledge_timer_or_reset()
                {
                    return true;
                }
            } else {
                rv = true;
            }
        }
        self.follower_link_state_mut()
            .restore_position_from_previous();
        self.follower_link_state_mut().clear_movement_subpixels();
        rv
    }

    /// Ledge-hop sound. C plays this via `Ancilla_Sfx2_Near(0x20)`, which routes through
    /// `PlaySfx_SetPanFrom`: it stamps RAW_SFX_PAN_VALUE (0xcf8) = 0x20 AND sets
    /// SOUND_EFFECT_1 = `0x20 | pan`. `ancilla_sfx2_near` mirrors that exactly.
    pub(super) fn link_ledge_hop_sfx(&mut self) {
        self.ancilla_sfx2_near(0x20);
    }

    pub(super) fn flag_moving_into_slopes_y(&mut self) {
        let diag_state = self.game_state.player.tile_detection.diag_state() as usize;
        let x = self.game_state.player.follower_link.x().wrapping_sub(
            if self.game_state.player.tile_detection.slope_collision_bits() & 4 != 0 {
                1
            } else {
                0
            },
        );
        let o = diag_state * 4 + (x & 7) as usize;
        let mut y = (self.game_state.player.tile_detection.y_low() & 7) as i8;

        if self.game_state.player.tile_detection.diagonal_tile() & 5 != 0 {
            let mut ym = (self.game_state.player.tile_detection.y_low() & 7) as i8;
            if self.game_state.enhanced_features.has(4096) {
                if diag_state & 2 != 0 {
                    ym = ym.wrapping_neg();
                } else {
                    ym = FLAG_MOVING_INTO_SLOPES_Y_AVOID_JUDDER[o].wrapping_sub(8 - ym);
                }
            } else {
                if diag_state & 2 == 0 {
                    ym = 8 - ym;
                } else {
                    ym += 8;
                }
                ym = FLAG_MOVING_INTO_SLOPES_Y_AVOID_JUDDER[o].wrapping_sub(ym);
            }

            let y_velocity = self.game_state.player.follower_link.y_velocity_signed();
            if y_velocity == 0 {
                return;
            }
            if y_velocity.is_negative() {
                ym = ym.wrapping_neg();
            }
            y = ym;
        } else {
            y = FLAG_MOVING_INTO_SLOPES_Y_AVOID_JUDDER[o].wrapping_sub(y);
        }

        if self
            .game_state
            .player
            .follower_link
            .y_velocity_signed()
            .is_negative()
        {
            if y <= 0 {
                return;
            }
            let coord = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(y as i16 as u16);
            self.follower_link_state_mut().set_y(coord);
            self.follower_link_state_mut()
                .set_moving_against_diag_tile(8);
        } else {
            if y >= 0 {
                return;
            }
            let coord = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(y as i16 as u16);
            self.follower_link_state_mut().set_y(coord);
            self.follower_link_state_mut()
                .set_moving_against_diag_tile(4);
        }

        let diag_flags = if self.game_state.player.tile_detection.slope_collision_bits() & 4 != 0 {
            0x12
        } else {
            0x11
        };
        self.follower_link_state_mut()
            .add_moving_against_diag_tile_flags(diag_flags);
    }

    pub(super) fn flag_moving_into_slopes_x(&mut self) {
        let diag_state = self.game_state.player.tile_detection.diag_state();
        let mut x = (self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_sub(if diag_state == 6 { 1 } else { 0 })
            & 7) as i8;
        let which_y_offset =
            if self.game_state.player.tile_detection.slope_collision_bits() & 4 != 0 {
                2
            } else {
                0
            };
        let o = diag_state as usize * 4
            + (self
                .game_state
                .player
                .tile_detection
                .y_low_at(which_y_offset)
                & 7) as usize;

        if self.game_state.player.tile_detection.diagonal_tile() & 5 != 0 {
            let mut xm = (self.game_state.player.follower_link.x() & 7) as i8;
            if diag_state != 4 && diag_state != 6 {
                xm = xm.wrapping_neg();
            } else {
                xm = FLAG_MOVING_INTO_SLOPES_X_AVOID_JUDDER[o].wrapping_sub(8 - xm);
            }
            let x_velocity = self.game_state.player.follower_link.x_velocity_signed();
            if x_velocity == 0 {
                return;
            }
            if x_velocity.is_negative() {
                xm = xm.wrapping_neg();
            }
            x = xm;
        } else {
            x = FLAG_MOVING_INTO_SLOPES_X_AVOID_JUDDER[o].wrapping_sub(x);
        }

        if self
            .game_state
            .player
            .follower_link
            .x_velocity_signed()
            .is_negative()
        {
            if x <= 0 {
                return;
            }
            let coord = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(x as i16 as u16);
            self.follower_link_state_mut().set_x(coord);
            self.follower_link_state_mut()
                .set_moving_against_diag_tile(2);
        } else {
            if x >= 0 {
                return;
            }
            let coord = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(x as i16 as u16);
            self.follower_link_state_mut().set_x(coord);
            self.follower_link_state_mut()
                .set_moving_against_diag_tile(1);
        }

        self.follower_link_state_mut()
            .add_moving_against_diag_tile_flags(if diag_state & 2 != 0 { 0x28 } else { 0x24 });
    }

    pub(in crate::zelda_rtl) fn player_something_with_velocity_tired_or_swim(&mut self, xvel: u16, yvel: u16) {
        let old_x = self.game_state.player.follower_link.x();
        let old_y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .store_safe_return_position(old_x, old_y);

        self.follower_link_state_mut()
            .move_x_by_subpixel_delta(xvel);
        let u = (xvel >> 8) as u8;
        let actual_x_velocity = (if (u as i8).is_negative() {
            0u8.wrapping_sub(u)
        } else {
            u
        } << 4)
            | ((xvel as u8) >> 4);

        self.follower_link_state_mut()
            .move_y_by_subpixel_delta(yvel);
        let u = (yvel >> 8) as u8;
        let actual_y_velocity = (if (u as i8).is_negative() {
            0u8.wrapping_sub(u)
        } else {
            u
        } << 4)
            | ((yvel as u8) >> 4);
        self.follower_link_state_mut()
            .set_actual_velocity_xy(actual_x_velocity, actual_y_velocity);

        if self.game_state.dungeon.room_load.header_collision() == 4 {
            self.link_apply_moving_floor_velocity();
        }
        self.follower_link_state_mut().clear_page_movement_deltas();
        self.link_handle_velocity_and_sand_drag(old_x, old_y);
    }

    pub(in crate::zelda_rtl) fn link_check_for_edge_screen_transition(&mut self) -> bool {
        let _scope = crate::cycle_ledger::routine(0x07_f439);
        if !self
            .game_state
            .player
            .follower_link
            .is_edge_transition_blocked_by_handler_state()
            && self.game_state.player.follower_link.incapacitated_timer() == 0
        {
            return false;
        }
        self.follower_link_state_mut().clear_actual_velocity_xy();
        self.follower_link_state_mut().set_recoil_timer(3);
        self.follower_link_state_mut()
            .restore_position_from_previous();
        true
    }

    pub(super) fn player_limit_directions_inner(&mut self) {
        self.follower_link_state_mut().reset_direction_limits();
        self.player_limit_directions_after_reset(None);
    }

    pub(super) fn player_limit_directions_after_reset(&mut self, native_vertical_probe: Option<CollisionDirection>) {
        let direction = self.game_state.player.follower_link.direction();
        if direction & 0x0c != 0 {
            let probe_direction = native_vertical_probe.unwrap_or_else(|| {
                self.follower_link_state_mut().increment_orthogonal_direction_count();
                let probe_direction = CollisionDirection::along(CollisionAxis::Vertical, direction & 8 != 0);
                self.follower_link_state_mut().set_last_direction_moved_towards(probe_direction.legacy_value());
                probe_direction
            });
            if std::mem::take(&mut self.early_vertical_slope_probe) {
                assert!(native_vertical_probe.is_some());
                let y = read_le_u16(&self.ram, 0);
                let low_x = read_le_u16(&self.ram, 2);
                let high_x = read_le_u16(&self.ram, 4);
                use crate::cycle_models::tile_classifier::TileClassifierTail;
                match self.early_tile_classifier_tail.take() {
                    Some(TileClassifierTail::Complete) => {}
                    Some(TileClassifierTail::Effect {tile, offset}) => self.tile_detect_execute_definition(
                        crate::tile_definition::NativeTile::from_cartridge(tile), offset, 1, true),
                    Some(TileClassifierTail::OutdoorAttribute) | None => self.tile_detection_execute(low_x, y, 1),
                }
                match self.early_high_tile_classifier_tail.take() {
                    Some(TileClassifierTail::Complete) => {}
                    Some(TileClassifierTail::Effect {tile, offset}) => self.tile_detect_execute_definition(
                        crate::tile_definition::NativeTile::from_cartridge(tile), offset, 2, true),
                    Some(TileClassifierTail::OutdoorAttribute) | None => self.tile_detection_execute(high_x, y, 2),
                }
            } else {
            self.detect_player_movement(
                CollisionAxis::Vertical,
                probe_direction,
                MovementProbeKind::Slope,
            );
            }

            let r14 = self.game_state.player.tile_detection.collision_bits();
            if r14 & 0x30 != 0
                && self.game_state.player.tile_detection.door_direction_flags() as u8 & 2 == 0
                && ((((r14 & 0x30) >> 4) as u8) & self.game_state.player.follower_link.direction())
                    == 0
                && self.game_state.player.follower_link.direction() & 3 != 0
            {
                let mask = PLAYER_LIMIT_DIRECTIONS_INNER_MASKS[if self
                    .game_state
                    .player
                    .follower_link
                    .direction()
                    & 2
                    != 0
                {
                    2
                } else {
                    3
                }];
                self.follower_link_state_mut().set_direction_mask_a(mask);
            } else {
                let mut set_thingy = false;
                if self.game_state.dungeon.room_load.header_collision() == 0
                    && self.game_state.player.follower_link.has_auxiliary_state()
                    && self.game_state.player.tile_detection.slope_collision_bits() & 3 != 0
                {
                    set_thingy = true;
                }

                if r14 & 3 != 0 {
                    self.follower_link_state_mut()
                        .clear_moving_against_diag_tile();
                    if self.game_state.player.follower_link.flag_moving() != 0
                        && self.game_state.player.tile_detection.spike_cactus_tiles() & 3 == 0
                        && self.game_state.player.follower_link.direction() & 3 != 0
                    {
                        self.swim_acceleration_mut().set_speed_active_flag(0, 0);
                        self.swim_acceleration_mut().set_mode(0, 0);
                        self.swim_acceleration_mut().set_acceleration(0, 0);
                        self.swim_acceleration_mut().set_max_speed(0, 0);
                    }
                    set_thingy = true;
                }

                if set_thingy {
                    self.follower_link_state_mut().set_pit_correction_active();
                    let mask = PLAYER_LIMIT_DIRECTIONS_INNER_MASKS[self
                        .game_state
                        .player
                        .follower_link
                        .last_direction_moved_towards()
                        as usize];
                    self.follower_link_state_mut().set_direction_mask_a(mask);
                }
            }
        }

        self.player_limit_directions_after_vertical(None);
    }

    pub(super) fn player_limit_directions_after_vertical(&mut self, native_horizontal_probe: Option<CollisionDirection>) {
        let direction = self.game_state.player.follower_link.direction();
        if native_horizontal_probe.is_some() || (direction & 0x0c != 0 && direction & 3 != 0) {
            let probe_direction = native_horizontal_probe.unwrap_or_else(|| {
                self.follower_link_state_mut().increment_orthogonal_direction_count();
                let direction = CollisionDirection::along(CollisionAxis::Horizontal, direction & 2 != 0);
                self.follower_link_state_mut().set_last_direction_moved_towards(direction.legacy_value());
                direction
            });
            self.detect_player_movement(
                CollisionAxis::Horizontal,
                probe_direction,
                MovementProbeKind::Slope,
            );

            let r14 = self.game_state.player.tile_detection.collision_bits();
            if r14 & 0x30 != 0
                && self.game_state.player.tile_detection.door_direction_flags() as u8 & 2 != 0
                && ((((r14 & 0x30) >> 2) as u8) & self.game_state.player.follower_link.direction())
                    == 0
                && self.game_state.player.follower_link.direction() & 0x0c != 0
            {
                let mask = PLAYER_LIMIT_DIRECTIONS_INNER_MASKS[if self
                    .game_state
                    .player
                    .follower_link
                    .direction()
                    & 8
                    != 0
                {
                    0
                } else {
                    1
                }];
                self.follower_link_state_mut().set_direction_mask_b(mask);
            } else {
                let mut set_thingy_b = false;
                if self.game_state.dungeon.room_load.header_collision() == 0
                    && self.game_state.player.follower_link.has_auxiliary_state()
                    && self.game_state.player.tile_detection.slope_collision_bits() & 3 != 0
                {
                    set_thingy_b = true;
                }

                if r14 & 3 != 0 {
                    self.follower_link_state_mut()
                        .clear_moving_against_diag_tile();
                    if self.game_state.player.follower_link.flag_moving() != 0
                        && self.game_state.player.tile_detection.spike_cactus_tiles() & 3 == 0
                        && self.game_state.player.follower_link.direction() & 0x0c != 0
                    {
                        self.swim_acceleration_mut().set_speed_active_flag(2, 0);
                        self.swim_acceleration_mut().set_mode(2, 0);
                        self.swim_acceleration_mut().set_acceleration(2, 0);
                        self.swim_acceleration_mut().set_max_speed(2, 0);
                    }
                    set_thingy_b = true;
                }

                if set_thingy_b {
                    self.follower_link_state_mut().set_pit_correction_active();
                    let mask = PLAYER_LIMIT_DIRECTIONS_INNER_MASKS[self
                        .game_state
                        .player
                        .follower_link
                        .last_direction_moved_towards()
                        as usize];
                    self.follower_link_state_mut().set_direction_mask_b(mask);
                }
            }

            self.follower_link_state_mut().apply_direction_masks();
        }

        self.follower_link_state_mut()
            .force_direction_from_diag_tile_if_needed();
        self.follower_link_state_mut()
            .resolve_orthogonal_direction_count_from_facing();
    }

    pub(in crate::zelda_rtl) fn link_handle_velocity(&mut self) {
        if let Some(position_return) = self.link_handle_velocity_until_position_integrated() {
            self.complete_link_move_position_after_coordinates(position_return);
        }
    }

    pub(in crate::zelda_rtl) fn link_handle_velocity_until_position_integrated(
        &mut self,
    ) -> Option<LinkMovePositionReturn> {
        if !self.link_handle_velocity_before_move_position() {
            return None;
        }
        self.link_move_position_until_coordinates_integrated()
    }

    /// `Link_HandleVelocity` through `Link_MovePosition`'s axis loop, stopping
    /// after the selected axis' subpixel store (the ROM's mid-loop interruption
    /// at route host 179586). `None` when the velocity handler took an early
    /// exit before `Link_MovePosition`.
    pub(in crate::zelda_rtl) fn link_handle_velocity_until_position_partial(
        &mut self,
        axis: PlayerAxis,
    ) -> Option<LinkMovePositionPartialReturn> {
        if !self.link_handle_velocity_before_move_position() {
            return None;
        }
        Some(self.link_move_position_partial_after_subpixel(axis))
    }

    /// `Link_HandleVelocity` through the low coordinate-byte store for
    /// `axis`, retaining the computed high byte and later movement suffix.
    pub(in crate::zelda_rtl) fn link_handle_velocity_until_position_after_coordinate_low(
        &mut self,
        axis: PlayerAxis,
    ) -> Option<LinkMovePositionAfterCoordinateLowReturn> {
        if !self.link_handle_velocity_before_move_position() {
            return None;
        }
        Some(self.link_move_position_after_coordinate_low(axis))
    }

    /// `Link_HandleVelocity` through both coordinate stores for `axis` in
    /// `Link_MovePosition`. `None` when the velocity handler returned before
    /// entering the movement loop.
    pub(in crate::zelda_rtl) fn link_handle_velocity_until_position_after_coordinates(
        &mut self,
        axis: PlayerAxis,
    ) -> Option<LinkMovePositionAfterCoordinatesReturn> {
        if !self.link_handle_velocity_before_move_position() {
            return None;
        }
        Some(self.link_move_position_after_coordinates(axis))
    }

    /// Run speed selection and the source-proven actual-velocity prefix,
    /// retaining whichever components have not yet been published.
    pub(in crate::zelda_rtl) fn link_handle_velocity_until_actual_checkpoint(
        &mut self,
        checkpoint: impl Into<LinkActualVelocityCheckpoint>,
    ) -> Option<LinkActualVelocityReturn> {
        let checkpoint = checkpoint.into();
        let completed_clear_stores = match checkpoint {
            LinkActualVelocityCheckpoint::Clearing { completed } => {
                assert!((1..4).contains(&completed));
                completed
            }
            _ => 4,
        };
        let speed_index = self.link_handle_velocity_before_velocity_clear()?;
        for store in 0..completed_clear_stores {
            self.follower_link_state_mut()
                .clear_velocity_selection_store(store);
        }
        if matches!(
            checkpoint,
            LinkActualVelocityCheckpoint::BeforeSelection
                | LinkActualVelocityCheckpoint::Clearing { .. }
        ) {
            return Some(LinkActualVelocityReturn {
                completed_clear_stores,
                pending_speed_index: Some(speed_index),
                pending_actual_x: None,
                pending_actual_y: None,
            });
        }
        let (direction, velocity) = self.link_handle_velocity_after_velocity_cleared(speed_index);
        let mut pending_actual_x = (direction & 0x03 != 0).then_some(if direction & 0x02 != 0 {
            0u8.wrapping_sub(velocity)
        } else {
            velocity
        });
        if let Some(actual_x) = pending_actual_x.filter(|_| {
            matches!(
                checkpoint,
                LinkActualVelocityCheckpoint::BeforeY | LinkActualVelocityCheckpoint::AfterBoth
            )
        }) {
            self.follower_link_state_mut()
                .set_actual_x_velocity(actual_x);
            pending_actual_x = None;
        }
        let mut pending_actual_y = (direction & 0x0c != 0).then_some(if direction & 0x08 != 0 {
            0u8.wrapping_sub(velocity)
        } else {
            velocity
        });
        if checkpoint == LinkActualVelocityCheckpoint::AfterBoth {
            if let Some(actual_y) = pending_actual_y.take() {
                self.follower_link_state_mut()
                    .set_actual_y_velocity(actual_y);
            }
        }
        Some(LinkActualVelocityReturn {
            completed_clear_stores,
            pending_speed_index: None,
            pending_actual_x,
            pending_actual_y,
        })
    }

    /// Resume pending actual-velocity publication in X-then-Y order, airborne defaults,
    /// and `Link_MovePosition` without replaying speed/modifier selection.
    pub(in crate::zelda_rtl) fn link_handle_velocity_from_actual_checkpoint_until_position_integrated(
        &mut self,
        velocity_return: LinkActualVelocityReturn,
    ) -> Option<LinkMovePositionReturn> {
        for store in velocity_return.completed_clear_stores..4 {
            self.follower_link_state_mut()
                .clear_velocity_selection_store(store);
        }
        if let Some(speed_index) = velocity_return.pending_speed_index {
            let (direction, velocity) =
                self.link_handle_velocity_after_velocity_cleared(speed_index);
            self.follower_link_state_mut()
                .set_actual_velocity_from_direction(direction, velocity);
        }
        if let Some(actual_x) = velocity_return.pending_actual_x {
            self.follower_link_state_mut()
                .set_actual_x_velocity(actual_x);
        }
        if let Some(actual_y) = velocity_return.pending_actual_y {
            self.follower_link_state_mut()
                .set_actual_y_velocity(actual_y);
        }
        self.follower_link_state_mut().prime_airborne_z_velocity();
        self.link_move_position_until_coordinates_integrated()
    }

    /// `Link_HandleVelocity` up to its `Link_MovePosition` call. `false` when
    /// the ROM returned earlier (safe-return/sand-drag, swim, or the
    /// all-blocked collision exit).
    fn link_handle_velocity_before_move_position(&mut self) -> bool {
        let Some((direction, velocity)) =
            self.link_handle_velocity_before_actual_velocity_resolution()
        else {
            return false;
        };
        self.follower_link_state_mut()
            .set_actual_velocity_from_direction(direction, velocity);
        self.follower_link_state_mut().prime_airborne_z_velocity();
        true
    }

    /// Run the stateful speed-selection prefix, stopping before either
    /// component of actual velocity is resolved.
    fn link_handle_velocity_before_actual_velocity_resolution(&mut self) -> Option<(u8, u8)> {
        let speed_index = self.link_handle_velocity_until_velocity_cleared()?;
        Some(self.link_handle_velocity_after_velocity_cleared(speed_index))
    }

    fn link_handle_velocity_until_velocity_cleared(&mut self) -> Option<u8> {
        let speed_index = self.link_handle_velocity_before_velocity_clear()?;
        self.follower_link_state_mut()
            .clear_actual_velocity_and_page_movement_deltas();
        Some(speed_index)
    }

    fn link_handle_velocity_before_velocity_clear(&mut self) -> Option<u8> {
        let old_x = self.game_state.player.follower_link.x();
        let old_y = self.game_state.player.follower_link.y();

        if (self.game_state.frame.submodule == 2 && self.game_state.frame.main_module == 14)
            || self
                .game_state
                .player
                .follower_link
                .is_prevented_from_moving()
        {
            self.store_link_safe_return_position(old_x, old_y);
            self.link_handle_velocity_and_sand_drag(old_x, old_y);
            return None;
        }

        if self.game_state.player.follower_link.handler_state() == 4 {
            self.handle_swim_stroke_and_subpixels();
            return None;
        }

        let speed_index = if self.game_state.player.follower_link.flag_moving() != 0 {
            if !self.game_state.player.follower_link.is_running() {
                self.handle_swim_stroke_and_subpixels();
                return None;
            }
            24
        } else {
            if self.game_state.player.follower_link.is_running() {
                self.follower_link_state_mut().clear_speed_modifier();
                assert!(self.game_state.player.follower_link.dash_counter() >= 32);
            }
            if (self
                .game_state
                .player
                .tile_detection
                .tile_collision_bits_primary()
                | self
                    .game_state
                    .player
                    .tile_detection
                    .tile_collision_bits_secondary())
                == 0x0f
            {
                return None;
            }
            if self
                .game_state
                .player
                .follower_link
                .water_ripple_or_grass_state()
                != 0
            {
                match self.game_state.player.follower_link.speed_setting() {
                    16 => 22,
                    12 => 14,
                    _ => 12,
                }
            } else {
                self.game_state.player.follower_link.speed_setting()
            }
        };

        Some(speed_index)
    }

    fn link_handle_velocity_after_velocity_cleared(&mut self, mut speed_index: u8) -> (u8, u8) {
        let direction = self.game_state.player.follower_link.direction();
        if (direction & 0x0c) != 0 && (direction & 0x03) != 0 {
            speed_index = speed_index.wrapping_add(1);
        }

        if self.game_state.player.follower_link.is_near_pit() {
            if self.game_state.player.follower_link.near_pit_state_is(3) {
                self.follower_link_state_mut()
                    .increase_near_pit_speed_modifier();
            }
        } else if self.game_state.player.follower_link.speed_modifier() != 0 {
            speed_index =
                if self.game_state.frame.submodule == 8 || self.game_state.frame.submodule == 16 {
                    10
                } else {
                    2
                };
            let speed_modifier = self.game_state.player.follower_link.speed_modifier();
            if speed_modifier != 1 && speed_modifier < 16 {
                self.follower_link_state_mut().advance_dash_deceleration();
                speed_index = 26;
            } else if speed_modifier != 1 {
                self.follower_link_state_mut().clear_speed_modifier();
                self.follower_link_state_mut().set_speed_setting(0);
            }
        }

        let velocity = self
            .game_state
            .player
            .follower_link
            .speed_modifier()
            .wrapping_add(LINK_HANDLE_VELOCITY_SPEED_MOD[speed_index as usize]);
        (self.game_state.player.follower_link.direction(), velocity)
    }

    /// `Link_MovePosition` up to and including the selected axis' subpixel
    /// store: the ROM's loop runs z (airborne only), then x, then y; earlier
    /// axes complete, the selected axis owes its coordinate delta.
    fn link_move_position_partial_after_subpixel(
        &mut self,
        axis: PlayerAxis,
    ) -> LinkMovePositionPartialReturn {
        let x = self.game_state.player.follower_link.x();
        let y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .store_safe_return_position(x, y);
        assert!(
            !(self.game_state.player.follower_link.handler_state() != 10
                && self.game_state.player.follower_link.on_somaria_platform() == 2),
            "a mid-loop Link_MovePosition interruption cannot take the Somaria platform exit",
        );
        for &candidate in self.link_move_position_axes() {
            let velocity = self.link_move_position_axis_velocity(candidate);
            if candidate == axis {
                let pending_pixel_delta = self
                    .follower_link_state_mut()
                    .move_axis_subpixel_only_by_velocity(candidate, velocity);
                return LinkMovePositionPartialReturn {
                    old_x: x,
                    old_y: y,
                    partial: LinkMovePositionPartial {
                        axis,
                        pending_pixel_delta,
                    },
                };
            }
            self.link_move_position_full_axis(candidate, velocity);
        }
        panic!("Link_MovePosition interruption named axis {axis:?} outside the ROM's axis loop");
    }

    /// `Link_MovePosition` through both coordinate stores for `axis`.
    fn link_move_position_after_coordinates(
        &mut self,
        axis: PlayerAxis,
    ) -> LinkMovePositionAfterCoordinatesReturn {
        let x = self.game_state.player.follower_link.x();
        let y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .store_safe_return_position(x, y);
        assert!(
            !(self.game_state.player.follower_link.handler_state() != 10
                && self.game_state.player.follower_link.on_somaria_platform() == 2),
            "a mid-loop Link_MovePosition interruption cannot take the Somaria platform exit",
        );
        for &candidate in self.link_move_position_axes() {
            let velocity = self.link_move_position_axis_velocity(candidate);
            self.link_move_position_full_axis(candidate, velocity);
            if candidate == axis {
                return LinkMovePositionAfterCoordinatesReturn {
                    old_x: x,
                    old_y: y,
                    axis,
                };
            }
        }
        panic!("Link_MovePosition interruption named axis {axis:?} outside the ROM's axis loop");
    }

    /// `Link_MovePosition` through the current axis' low coordinate-byte
    /// store, preserving its high byte and every later axis.
    fn link_move_position_after_coordinate_low(
        &mut self,
        axis: PlayerAxis,
    ) -> LinkMovePositionAfterCoordinateLowReturn {
        let x = self.game_state.player.follower_link.x();
        let y = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .store_safe_return_position(x, y);
        assert!(
            !(self.game_state.player.follower_link.handler_state() != 10
                && self.game_state.player.follower_link.on_somaria_platform() == 2),
            "a mid-loop Link_MovePosition interruption cannot take the Somaria platform exit",
        );
        for &candidate in self.link_move_position_axes() {
            let velocity = self.link_move_position_axis_velocity(candidate);
            if candidate == axis {
                let mut player = self.follower_link_state_mut();
                let pending_pixel_delta =
                    player.move_axis_subpixel_only_by_velocity(candidate, velocity);
                let pending_coordinate_high =
                    player.apply_axis_pixel_delta_low(candidate, pending_pixel_delta);
                return LinkMovePositionAfterCoordinateLowReturn {
                    old_x: x,
                    old_y: y,
                    axis,
                    pending_coordinate_high,
                };
            }
            self.link_move_position_full_axis(candidate, velocity);
        }
        panic!("Link_MovePosition interruption named axis {axis:?} outside the ROM's axis loop");
    }

    /// Resume `Link_MovePosition` after `link_move_position_partial_after_subpixel`.
    pub(in crate::zelda_rtl) fn complete_link_move_position_from_partial(
        &mut self,
        position_return: LinkMovePositionPartialReturn,
    ) {
        let LinkMovePositionPartial {
            axis,
            pending_pixel_delta,
        } = position_return.partial;
        self.follower_link_state_mut()
            .apply_axis_pixel_delta(axis, pending_pixel_delta);
        let passes = self.link_move_position_axes();
        let resume_at = passes
            .iter()
            .position(|candidate| *candidate == axis)
            .expect("a suspended Link_MovePosition axis must belong to the ROM's axis loop");
        for candidate in passes[resume_at + 1..].iter().copied() {
            let velocity = self.link_move_position_axis_velocity(candidate);
            self.link_move_position_full_axis(candidate, velocity);
        }
        self.complete_link_move_position_after_coordinates(LinkMovePositionReturn {
            old_x: position_return.old_x,
            old_y: position_return.old_y,
        });
    }

    /// Resume after the current axis' low coordinate byte was published.
    pub(in crate::zelda_rtl) fn complete_link_move_position_from_after_coordinate_low(
        &mut self,
        position_return: LinkMovePositionAfterCoordinateLowReturn,
    ) {
        self.follower_link_state_mut().apply_axis_coordinate_high(
            position_return.axis,
            position_return.pending_coordinate_high,
        );
        self.complete_link_move_position_from_after_coordinates(
            LinkMovePositionAfterCoordinatesReturn {
                old_x: position_return.old_x,
                old_y: position_return.old_y,
                axis: position_return.axis,
            },
        );
    }

    /// Resume after `link_move_position_after_coordinates`, beginning with
    /// the following axis and then completing the ordinary movement tail.
    pub(in crate::zelda_rtl) fn complete_link_move_position_from_after_coordinates(
        &mut self,
        position_return: LinkMovePositionAfterCoordinatesReturn,
    ) {
        let passes = self.link_move_position_axes();
        let resume_at = passes
            .iter()
            .position(|candidate| *candidate == position_return.axis)
            .expect("a suspended Link_MovePosition axis must belong to the ROM's axis loop");
        for candidate in passes[resume_at + 1..].iter().copied() {
            let velocity = self.link_move_position_axis_velocity(candidate);
            self.link_move_position_full_axis(candidate, velocity);
        }
        self.complete_link_move_position_after_coordinates(LinkMovePositionReturn {
            old_x: position_return.old_x,
            old_y: position_return.old_y,
        });
    }

    /// The ROM's `Link_MovePosition` axes in their original execution order.
    fn link_move_position_axes(&self) -> &'static [PlayerAxis] {
        if self.game_state.player.follower_link.auxiliary_state() != 0 {
            &[PlayerAxis::Z, PlayerAxis::X, PlayerAxis::Y]
        } else {
            &[PlayerAxis::X, PlayerAxis::Y]
        }
    }

    fn link_move_position_axis_velocity(&self, axis: PlayerAxis) -> u8 {
        match axis {
            PlayerAxis::Z => self.game_state.player.follower_link.actual_z_velocity(),
            PlayerAxis::X => self.game_state.player.follower_link.actual_x_velocity(),
            PlayerAxis::Y => self.game_state.player.follower_link.actual_y_velocity(),
        }
    }

    fn link_move_position_full_axis(&mut self, axis: PlayerAxis, velocity: u8) {
        self.follower_link_state_mut()
            .move_axis_by_velocity(axis, velocity);
    }

    pub(in crate::zelda_rtl) fn handle_swim_stroke_and_subpixels(&mut self) {
        self.follower_link_state_mut().clear_actual_velocity_xy();

        let mut stroke = [0u16; 2];

        for i in (0..=1).rev() {
            let offset = i * 2;
            let stroke_timer = self
                .game_state
                .player
                .follower_link
                .swim_stroke_frame_counter(offset)
                .wrapping_sub(1);
            self.follower_link_state_mut()
                .set_swim_stroke_frame_counter(offset, stroke_timer);
            if (stroke_timer as i16) < 0 {
                self.follower_link_state_mut()
                    .set_swim_stroke_frame_counter(offset, 0);
                self.swim_acceleration_mut().set_mode(offset, 1);
            }

            let mut table_index = self.game_state.player.swim_acceleration.mode(offset);
            if self.game_state.player.follower_link.flag_moving() != 0 {
                table_index = table_index.wrapping_add(
                    u16::from(self.game_state.player.follower_link.flag_moving()) * 4,
                );
            }

            let delta = HANDLE_SWIM_STROKE_AND_SUBPIXELS_SWIM_ACCELERATION_DELTAS
                [table_index as usize] as i16 as u16;
            let mut sum = self
                .game_state
                .player
                .swim_acceleration
                .acceleration(offset)
                .wrapping_add(delta);
            if (sum as i16) <= 0 {
                self.follower_link_state_mut().mask_direction(
                    HANDLE_SWIM_STROKE_AND_SUBPIXELS_SWIM_AXIS_DIRECTION_CLEAR_MASKS[i],
                );
                self.follower_link_state_mut()
                    .set_last_direction_from_current_direction();
                if self.game_state.player.swim_acceleration.mode(offset) == 2 {
                    self.swim_acceleration_mut().set_mode(offset, 0);
                    self.swim_acceleration_mut().set_max_speed(offset, 240);
                    self.swim_acceleration_mut().set_acceleration(offset, 2);
                } else {
                    self.swim_acceleration_mut().set_mode(offset, 0);
                    self.swim_acceleration_mut().set_max_speed(offset, 0);
                    self.swim_acceleration_mut().set_acceleration(offset, 0);
                }
            } else {
                let dir_index = self
                    .game_state
                    .player
                    .swim_acceleration
                    .acceleration_direction(offset) as usize
                    + i * 2;
                self.follower_link_state_mut().add_direction_flags(
                    HANDLE_SWIM_STROKE_AND_SUBPIXELS_SWIM_DIRECTION_BITS_BY_AXIS[dir_index],
                );
                let max_sum = self.game_state.player.swim_acceleration.max_speed(offset);
                if sum >= max_sum {
                    sum = max_sum;
                }
                self.swim_acceleration_mut().set_acceleration(offset, sum);
            }

            stroke[i] = self
                .game_state
                .player
                .swim_acceleration
                .acceleration(offset);
            if self.game_state.player.follower_link.has_swim_axis_drag() {
                stroke[i] = stroke[i].wrapping_sub(stroke[i] >> 2);
            }
            if self
                .game_state
                .player
                .swim_acceleration
                .acceleration_direction(offset)
                == 0
            {
                stroke[i] = 0u16.wrapping_sub(stroke[i]);
            }
        }

        self.player_something_with_velocity_tired_or_swim(stroke[1], stroke[0]);
    }
}
