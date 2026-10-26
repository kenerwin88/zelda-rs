//! Tile responses, indoor and outdoor movement probes, bonking, and cardinal and diagonal collision.

use super::*;
use crate::game_state::constants::FORCE_MOVE_ANY_DIRECTION;
use crate::game_state::{CollisionAxis, CollisionDirection, CollisionOrder, MovementProbeKind};

impl ZeldaState {
    pub(super) fn handle_nudging(&mut self, arg_r0: i8) {
        let last_direction_moved_towards = self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards();
        let p = if last_direction_moved_towards & 2 == 0 {
            if last_direction_moved_towards & 1 != 0 {
                4
            } else {
                0
            }
        } else if last_direction_moved_towards & 1 != 0 {
            12
        } else {
            8
        };
        let o = (((if self.game_state.player.tile_detection.collision_bits() & 4 != 0 {
            0
        } else {
            2
        }) + p)
            >> 1) as usize;

        self.tile_detect_position_mut().clear_pit_tile();
        self.tile_detect_reset_state();

        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let link_y = self.game_state.player.follower_link.y();
        let link_x = self.game_state.player.follower_link.x();
        let y0 = link_y.wrapping_add(HANDLE_NUDGING_NUDGE_PROBE_Y0_OFFSETS[o] as u16) & mask;
        let x0 = (link_x.wrapping_add(HANDLE_NUDGING_NUDGE_PROBE_X0_OFFSETS[o] as u16) & mask) >> 3;
        let y1 = link_y.wrapping_add(HANDLE_NUDGING_NUDGE_PROBE_Y1_OFFSETS[o] as u16) & mask;
        let x1 = (link_x.wrapping_add(HANDLE_NUDGING_NUDGE_PROBE_X1_OFFSETS[o] as u16) & mask) >> 3;

        self.tile_detection_execute(x0, y0, 1);
        self.tile_detection_execute(x1, y1, 2);

        let blocked = (self.game_state.player.tile_detection.collision_bits()
            | self.game_state.player.tile_detection.horizontal_ledge() as u16)
            & 3
            != 0
            || (self.game_state.player.tile_detection.vertical_ledge()
                | self.game_state.player.tile_detection.diagonal_ledge_tiles())
                & 0x33
                != 0;
        if blocked {
            if self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards()
                & 2
                != 0
            {
                let y = self.game_state.player.follower_link.y();
                self.follower_link_state_mut()
                    .set_y(y.wrapping_sub(arg_r0 as i16 as u16));
            } else {
                let x = self.game_state.player.follower_link.x();
                self.follower_link_state_mut()
                    .set_x(x.wrapping_sub(arg_r0 as i16 as u16));
            }
        }
    }

    pub(super) fn handle_pushing_bonking_snaps_y(&mut self) {
        let r14 = self.game_state.player.tile_detection.collision_bits();
        if r14 & 7 == 0 {
            if self.game_state.player.follower_link.is_on_lower_level() {
                return;
            }
            self.follower_link_state_mut().and_defense_flags(!9);
            self.handle_pushing_bonking_snaps_return();
            return;
        }

        let mut used_swim_axis_reprobe = false;
        if self.game_state.player.follower_link.handler_state() == 4 {
            if self.game_state.dungeon.moving_floor.floor_y_velocity_low() == 0 {
                self.reset_all_acceleration();
            }
            if self
                .game_state
                .player
                .follower_link
                .num_orthogonal_directions()
                != 0
            {
                self.link_add_in_velocity_y_falling();
                used_swim_axis_reprobe = true;
            }
        }

        if r14 & 2 != 0 || (r14 & 5) == 5 {
            let bak = r14;
            self.link_bonk_and_smash();
            self.repel_dash();
            self.tile_detect_position_mut().set_collision_bits(bak);
        }

        self.finish_pushing_bonking_snaps_y_after_initial_bonk(r14, used_swim_axis_reprobe);
    }

    fn finish_pushing_bonking_snaps_y_after_initial_bonk(
        &mut self,
        r14: u16,
        used_swim_axis_reprobe: bool,
    ) {
        self.follower_link_state_mut().set_pit_correction_active();

        if !used_swim_axis_reprobe {
            if r14 & 2 == 2 {
                self.link_add_in_velocity_y_falling();
            } else {
                if self
                    .game_state
                    .player
                    .follower_link
                    .num_orthogonal_directions()
                    == 1
                {
                    self.handle_pushing_bonking_snaps_return();
                    return;
                }
                self.link_add_in_velocity_y_falling();
                if self
                    .game_state
                    .player
                    .follower_link
                    .num_orthogonal_directions()
                    == 2
                {
                    self.handle_pushing_bonking_snaps_return();
                    return;
                }
            }
        }

        if (r14 & 5) == 5 {
            self.link_bonk_and_smash();
            self.repel_dash();
        } else if r14 & 2 == 0 {
            let y_vel = self.game_state.player.follower_link.y_velocity();
            let tt = if r14 & 4 != 0 {
                if (y_vel as i8).is_negative() {
                    y_vel
                } else {
                    0u8.wrapping_sub(y_vel)
                }
            } else if (y_vel as i8).is_negative() {
                0u8.wrapping_sub(y_vel)
            } else {
                y_vel
            };
            let delta = if (tt as i8).is_negative() {
                -1i16
            } else {
                1i16
            };
            if self.game_state.player.follower_link.x() & 7 != 0 {
                let x = self.game_state.player.follower_link.x();
                self.follower_link_state_mut()
                    .set_x(x.wrapping_add(delta as u16));
                self.handle_nudging(delta as i8);
                return;
            }
            self.link_bonk_and_smash();
            self.repel_dash();
        }

        self.finish_pushing_bonking_snaps_y_after_final_bonk();
    }

    fn finish_pushing_bonking_snaps_y_after_final_bonk(&mut self) {
        if self.handle_pushing_bonking_dragstate_tail() {
            return;
        }
        self.handle_pushing_bonking_snaps_return();
    }

    pub(super) fn handle_pushing_bonking_snaps_x(&mut self) {
        let r14 = self.game_state.player.tile_detection.collision_bits();
        if r14 & 7 == 0 {
            if self.game_state.player.follower_link.is_on_lower_level() {
                return;
            }
            self.follower_link_state_mut().and_defense_flags(!9);
            self.handle_pushing_bonking_snaps_return();
            return;
        }

        if self.game_state.player.follower_link.handler_state() == 4
            && self.game_state.dungeon.moving_floor.floor_x_velocity_low() == 0
        {
            self.reset_all_acceleration();
        }

        if r14 & 2 != 0 {
            let bak = r14;
            self.link_bonk_and_smash();
            self.repel_dash();
            self.tile_detect_position_mut().set_collision_bits(bak);
        }

        self.follower_link_state_mut().set_pit_correction_active();

        if r14 & 7 == 7 {
            self.snap_on_x();
        } else {
            if self
                .game_state
                .player
                .follower_link
                .num_orthogonal_directions()
                == 2
            {
                self.handle_pushing_bonking_snaps_return();
                return;
            }
            self.snap_on_x();
            if self
                .game_state
                .player
                .follower_link
                .num_orthogonal_directions()
                == 1
            {
                self.handle_pushing_bonking_snaps_return();
                return;
            }
        }

        if (r14 & 5) == 5 {
            self.link_bonk_and_smash();
            self.repel_dash();
        } else if r14 & 2 == 0 {
            let x_vel = self.game_state.player.follower_link.x_velocity();
            let tt = if r14 & 4 != 0 {
                if (x_vel as i8).is_negative() {
                    x_vel
                } else {
                    0u8.wrapping_sub(x_vel)
                }
            } else if (x_vel as i8).is_negative() {
                0u8.wrapping_sub(x_vel)
            } else {
                x_vel
            };
            let delta = if (tt as i8).is_negative() {
                -1i16
            } else {
                1i16
            };
            if self.game_state.player.follower_link.y() & 7 != 0 {
                let y = self.game_state.player.follower_link.y();
                self.follower_link_state_mut()
                    .set_y(y.wrapping_add(delta as u16));
                self.handle_nudging(delta as i8);
                return;
            }
            self.link_bonk_and_smash();
            self.repel_dash();
        }

        if self.handle_pushing_bonking_dragstate_tail() {
            return;
        }
        self.handle_pushing_bonking_snaps_return();
    }

    fn handle_pushing_bonking_dragstate_tail(&mut self) -> bool {
        if self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards()
            .wrapping_mul(2)
            != self.game_state.player.follower_link.facing()
        {
            return false;
        }

        let drag_bits = (self.game_state.player.follower_link.tile_coll_flag() & 1) << 1;
        self.follower_link_state_mut().or_defense_flags(drag_bits);
        let push_fatigue_timer = self
            .follower_link_state_mut()
            .decrement_push_fatigue_timer();
        if self.game_state.player.follower_link.button_b_frames() == 0
            && !(push_fatigue_timer as i8).is_negative()
        {
            return true;
        }

        let tile_coll = self.game_state.player.follower_link.tile_coll_flag();
        let drag_bits = if self.game_state.player.tile_detection.misc_tiles() & 0x20 != 0 {
            tile_coll << 3
        } else {
            tile_coll
        };
        self.follower_link_state_mut().or_defense_flags(drag_bits);
        false
    }

    fn handle_pushing_bonking_snaps_return(&mut self) {
        self.follower_link_state_mut().reset_push_fatigue_timer();
        self.follower_link_state_mut().and_defense_flags(!2);
    }

    pub(in crate::zelda_rtl) fn push_block_attempt_to_push_the_block(&self, what: u8, x: u16, y: u16) -> bool {
        let idx = what as usize * 4
            + self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards() as usize;
        let mask = self.game_state.player.tile_detection.location_calc_mask();

        let x0 = (x
            .wrapping_add(PUSH_BLOCK_ATTEMPT_TO_PUSH_THE_BLOCK_X_OFFSETS_0[idx] as i16 as u16)
            & mask)
            >> 3;
        let y0 = y
            .wrapping_add(PUSH_BLOCK_ATTEMPT_TO_PUSH_THE_BLOCK_Y_OFFSETS_0[idx] as i16 as u16)
            & mask;
        if !self.push_block_target_tile(x0, y0).accepts_push_block() {
            return true;
        }

        let x1 = (x
            .wrapping_add(PUSH_BLOCK_ATTEMPT_TO_PUSH_THE_BLOCK_X_OFFSETS_1[idx] as i16 as u16)
            & mask)
            >> 3;
        let y1 = y
            .wrapping_add(PUSH_BLOCK_ATTEMPT_TO_PUSH_THE_BLOCK_Y_OFFSETS_1[idx] as i16 as u16)
            & mask;
        !self.push_block_target_tile(x1, y1).accepts_push_block()
    }

    pub(super) fn link_find_valid_landing_tile_north(&mut self) {
        let y_coord_bak = self.game_state.player.follower_link.y();
        self.follower_link_state_mut()
            .set_hop_origin_coord(y_coord_bak);
        loop {
            let y = self.game_state.player.follower_link.y().wrapping_sub(16);
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
            if terrain & 7 == 7 {
                break;
            }
        }

        if self.game_state.player.tile_detection.deepwater() & 7 != 0 {
            self.follower_link_state_mut().set_auxiliary_state(1);
            self.follower_link_state_mut().clear_electrocute_on_touch();
            self.follower_link_state_mut().enter_deep_water_state();
            self.follower_link_state_mut()
                .set_swim_flags_from_last_direction();
            self.link_reset_swimming_state();
            self.follower_link_state_mut().clear_grabbing_wall();
            self.follower_link_state_mut().set_speed_setting(0);
        }

        let y = self.game_state.player.follower_link.y().wrapping_sub(16);
        self.follower_link_state_mut().set_y(y);
        let diff = self
            .follower_link_state_mut()
            .set_hop_origin_delta_from_y(y);
        self.follower_link_state_mut().set_y(y_coord_bak);
        let o = ((diff as u8) >> 3) as usize;
        let dy = LINK_FIND_VALID_LANDING_TILE_NORTH_Y_DELTAS[o];
        let actual_y_velocity = if self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards()
            != 0
        {
            dy
        } else {
            0u8.wrapping_sub(dy)
        };
        self.follower_link_state_mut()
            .set_actual_velocity_xy(0, actual_y_velocity);
        self.follower_link_state_mut()
            .set_actual_z_velocity_and_copy(LINK_FIND_VALID_LANDING_TILE_NORTH_Z_DELTAS[o]);
        self.follower_link_state_mut().set_z(0);
        self.follower_link_state_mut()
            .set_incapacitated_timer(LINK_FIND_VALID_LANDING_TILE_NORTH_TIMERS[o]);
        self.follower_link_state_mut().set_auxiliary_state(2);
        self.follower_link_state_mut().clear_electrocute_on_touch();
        self.follower_link_state_mut().set_handler_state(6);
    }

    pub(super) fn link_find_valid_landing_tile_diagonal_north(&mut self) {
        let y_safe = self.game_state.player.follower_link.safe_return_y_low();
        let x_bak = self.game_state.player.follower_link.x();
        let dir = self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards();

        let actual_x_velocity = if self
            .game_state
            .player
            .follower_link
            .last_direction_moved_towards()
            != 2
        {
            1
        } else {
            0xff
        };
        self.follower_link_state_mut()
            .set_actual_x_velocity(actual_x_velocity);
        self.follower_link_state_mut()
            .set_last_direction_moved_towards(0);
        self.link_hop_find_landing_spot_diagonally_down();

        self.follower_link_state_mut().set_x(x_bak);
        self.follower_link_state_mut().set_safe_return_y_low(y_safe);

        let diff = self
            .game_state
            .player
            .follower_link
            .hop_origin_coord()
            .wrapping_sub(self.game_state.player.follower_link.y());
        let o = (diff >> 3) as usize;
        self.follower_link_state_mut().restore_y_from_hop_origin();

        let actual_y_velocity =
            0u8.wrapping_sub(LINK_FIND_VALID_LANDING_TILE_DIAGONAL_NORTH_Y_DELTAS[o]);
        let actual_x_velocity = if dir != 2 {
            LINK_FIND_VALID_LANDING_TILE_DIAGONAL_NORTH_X_DELTAS[o]
        } else {
            0u8.wrapping_sub(LINK_FIND_VALID_LANDING_TILE_DIAGONAL_NORTH_X_DELTAS[o])
        };
        self.follower_link_state_mut()
            .set_actual_velocity_xy(actual_x_velocity, actual_y_velocity);
        self.follower_link_state_mut()
            .set_actual_z_velocity_and_copy(
                LINK_FIND_VALID_LANDING_TILE_DIAGONAL_NORTH_Z_DELTAS[o],
            );
        self.follower_link_state_mut().set_z(0);
        self.follower_link_state_mut().clear_z_mirror_word_low();
        self.follower_link_state_mut().set_auxiliary_state(2);
        self.follower_link_state_mut().clear_electrocute_on_touch();
        self.follower_link_state_mut().set_handler_state(13);
    }

    pub(in crate::zelda_rtl) fn tile_detect_main_handler(&mut self, item: u8) {
        if let Some(handoff)=self.early_main_tile_probe.take() {
            assert_eq!(item,handoff.item);
            use crate::cycle_models::tile_classifier::TileClassifierTail;
            match handoff.classifier {
                TileClassifierTail::Complete=>{},
                TileClassifierTail::Effect {tile,offset}=>self.tile_detect_execute_definition(crate::tile_definition::NativeTile::from_cartridge(tile),offset,1,true),
                TileClassifierTail::OutdoorAttribute=>self.tile_detection_execute(handoff.x,handoff.y,1),
            }
            if item!=5 {self.finish_main_tile_response();}
            return;
        }
        self.tile_detect_position_mut().clear_pit_tile();
        self.tile_detect_reset_state();

        let probe_base = if item == 8 {
            let spin = self
                .game_state
                .player
                .follower_link
                .state_for_spin_attack()
                .wrapping_sub(2);
            if spin >= 8 {
                return;
            }
            TILE_DETECT_MAIN_HANDLER_SPIN_OFFSETS[spin as usize] as u16 + 0x40
        } else {
            item as u16 * 8 + self.game_state.player.follower_link.facing() as u16
        };
        let offset = probe_base >> 1;

        let mask = self.game_state.player.tile_detection.location_calc_mask();
        let link_x = self.game_state.player.follower_link.x();
        let link_y = self.game_state.player.follower_link.y();
        let x = (link_x
            .wrapping_add(TILE_DETECT_MAIN_HANDLER_X_OFFSETS[offset as usize] as i16 as u16)
            & mask)
            >> 3;
        let y = link_y
            .wrapping_add(TILE_DETECT_MAIN_HANDLER_Y_OFFSETS[offset as usize] as i16 as u16)
            & mask;

        if matches!(item, 1 | 2 | 3 | 6 | 7 | 8) {
            self.tile_behavior_handle_item_and_execute(x, y);
            return;
        }

        self.tile_detection_execute(x, y, 1);
        if item == 5 {
            return;
        }

        self.finish_main_tile_response();
    }

    fn finish_main_tile_response(&mut self) {
        if self.game_state.player.tile_detection.thick_grass() & 0x10 != 0 {
            let tx = self.game_state.player.follower_link.x() & 0x0f;
            let ty = self.game_state.player.follower_link.y().wrapping_add(8) & 0x0f;
            if !(4..11).contains(&ty)
                && !(4..12).contains(&tx)
                && self.game_state.player.follower_link.blink_countdown() == 0
                && !self.game_state.player.follower_link.has_auxiliary_state()
            {
                if self.game_state.world.location.is_indoors() {
                    self.dungeon_flag_room_data_quadrants();
                    self.ancilla_sfx2_near(0x33);
                    self.follower_link_state_mut().set_speed_setting(0);
                    self.set_submodule(21);
                    let prev_room = self.game_state.world.location.dungeon_room_index();
                    self.dungeon_room_tracking_mut()
                        .set_room_index_prev(prev_room);
                    let room = self.game_state.dungeon.header.travel_destination(0);
                    self.set_dungeon_room_index(room);
                    self.handle_layer_of_destination();
                } else if !self.game_state.player.follower_link.whirlpool_triggered() {
                    self.do_sword_interaction_with_tiles_mirror();
                }
            }
        } else {
            self.follower_link_state_mut().clear_whirlpool_trigger();
            if self.game_state.player.tile_detection.thick_grass() & 1 != 0 {
                self.follower_link_state_mut()
                    .set_water_ripple_or_grass_state(2);
                if !self.link_permission_for_slosh_sounds()
                    && !self.game_state.player.follower_link.has_auxiliary_state()
                {
                    self.ancilla_sfx2_near(26);
                }
                return;
            }

            if self.game_state.player.tile_detection.shallow_water() & 1 != 0 {
                self.follower_link_state_mut()
                    .set_water_ripple_or_grass_state(1);
                if self.game_state.world.location.is_outdoors()
                    && self.game_state.player.follower_link.is_in_deep_water()
                    && !self.game_state.player.follower_link.is_bunny_mirror()
                {
                    if self.game_state.inventory.items.has_flippers() {
                        self.follower_link_state_mut().clear_deep_water_state();
                        self.follower_link_state_mut()
                            .set_last_direction_from_swim_flags();
                        self.follower_link_state_mut().clear_handler_state();
                    }
                } else if !self.link_permission_for_slosh_sounds() {
                    if self.game_state.world.location.overworld_screen_index() == 0x70 {
                        self.ancilla_sfx2_near(27);
                    } else if !self.game_state.player.follower_link.has_auxiliary_state() {
                        self.ancilla_sfx2_near(28);
                    }
                }
                return;
            }

            if self.game_state.world.location.is_outdoors()
                && !self.game_state.player.follower_link.is_in_deep_water()
                && self.game_state.player.tile_detection.deepwater() & 1 != 0
            {
                self.follower_link_state_mut()
                    .set_water_ripple_or_grass_state(1);
                if !self.link_permission_for_slosh_sounds() {
                    if self.game_state.world.location.overworld_screen_index() == 0x70 {
                        self.ancilla_sfx2_near(27);
                    } else if !self.game_state.player.follower_link.has_auxiliary_state() {
                        self.ancilla_sfx2_near(28);
                    }
                }
                return;
            }
        }

        self.follower_link_state_mut()
            .clear_water_ripple_or_grass_state();
        if self
            .game_state
            .player
            .tile_detection
            .spike_floor_and_triggers()
            & 1
            != 0
        {
            self.follower_link_state_mut()
                .set_item_pickup_in_progress(1);
            return;
        }
        self.follower_link_state_mut()
            .set_item_pickup_in_progress(0);

        if self
            .game_state
            .player
            .tile_detection
            .spike_floor_and_triggers()
            & 0x10
            != 0
        {
            self.follower_link_state_mut().clear_given_damage();
            if !self.game_state.player.follower_link.is_cape_active()
                && !self.search_for_byrna_spark()
                && self.game_state.player.follower_link.blink_countdown() == 0
            {
                self.follower_link_state_mut()
                    .clear_transform_poof_need_and_temp_bunny_timer();
                if self.game_state.inventory.items.has_moon_pearl() {
                    self.follower_link_state_mut()
                        .clear_bunny_transform_after_moon_pearl();
                }
                self.follower_link_state_mut().set_given_damage(8);
                self.link_cancel_dash();
                return;
            }
        }

        if self.game_state.player.tile_detection.icy_floor() & 0x11 != 0 {
            if self.game_state.player.follower_link.flag_moving() != 0 {
                if self
                    .game_state
                    .player
                    .follower_link
                    .num_orthogonal_directions()
                    != 0
                {
                    self.follower_link_state_mut()
                        .set_last_direction_from_swim_flags();
                }
            } else {
                if self.game_state.player.follower_link.direction() & 0x0c != 0 {
                    self.swim_acceleration_mut().set_acceleration(0, 0x0180);
                }
                if self.game_state.player.follower_link.direction() & 3 != 0 {
                    self.swim_acceleration_mut().set_acceleration(0, 0x0180);
                }
                let flag_moving = if self.game_state.player.tile_detection.icy_floor() & 1 != 0 {
                    1
                } else {
                    2
                };
                self.follower_link_state_mut().set_flag_moving(flag_moving);
                self.follower_link_state_mut()
                    .set_swim_flags_from_last_direction();
                self.link_reset_swimming_state();
            }
        } else {
            if self.game_state.player.follower_link.handler_state() != 4 {
                if self.game_state.player.follower_link.flag_moving() != 0 {
                    self.follower_link_state_mut()
                        .set_last_direction_from_swim_flags();
                }
                self.link_reset_swimming_state();
            }
            self.follower_link_state_mut().set_flag_moving(0);
        }

        if self.game_state.player.tile_detection.spike_cactus_tiles() & 0x10 != 0
            && self.game_state.player.follower_link.blink_countdown() == 0
        {
            self.follower_link_state_mut().set_blink_countdown(58);
        }
    }

    pub(in crate::zelda_rtl) fn start_movement_collision_checks(&mut self, axis: CollisionAxis) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(match axis {
            CollisionAxis::Vertical => 0x07_ba0a,
            CollisionAxis::Horizontal => 0x07_c4d4,
        }));
        let player = &self.game_state.player.follower_link;
        let (velocity, position_low, in_aligned_doorway) = match axis {
            CollisionAxis::Vertical => (
                player.y_velocity(),
                player.y_low(),
                player.doorway_state() == 1,
            ),
            CollisionAxis::Horizontal => (
                player.x_velocity(),
                player.x_low(),
                player.doorway_state() == 2,
            ),
        };
        if velocity == 0 {
            // $07:BA0A..BA0E / C4D4..C4D8: LDA, untaken BNE, RTS.
            if self.lag_free { crate::cycle_ledger::charge(82); }
            return;
        }
        let negative = if in_aligned_doorway {
            position_low < 0x80
        } else {
            (velocity as i8).is_negative()
        };
        let direction = CollisionDirection::along(axis, negative);
        if self.lag_free {
            // $07:BA0A..BA2D / C4D4..C4F7, M8/X8. Taken entry
            // BNE; doorway comparison; selected LDY/LDA/sign branch;
            // TYA/LSR/STA and the JSR opcode. The probe owns its body.
            let selection = if in_aligned_doorway {
                72 + if negative { 6 } else { 22 + 16 }
            } else {
                6 + 56 + if negative { 6 } else { 16 }
            };
            crate::cycle_ledger::charge(46 + 56 + selection + 98);
        }
        self.follower_link_state_mut()
            .set_last_direction_moved_towards(direction.legacy_value());
        self.detect_player_movement(axis, direction, MovementProbeKind::Cardinal);
        if self.lag_free {
            // $07:BA2E..BA34 / C4F8..C4FE: LDA $1B, BNE and
            // the outdoor BRL when selected. Response tails remain unpriced.
            crate::cycle_ledger::charge(if self.game_state.world.location.is_indoors() { 46 } else { 70 });
        }
        self.finish_movement_collision_response(axis);
    }

    fn finish_movement_collision_response(&mut self, axis: CollisionAxis) {
        match (axis, self.game_state.world.location.is_indoors()) {
            (CollisionAxis::Vertical, true) => {
                self.start_movement_collision_checks_y_handle_indoors()
            }
            (CollisionAxis::Vertical, false) => {
                self.start_movement_collision_checks_y_handle_outdoors()
            }
            (CollisionAxis::Horizontal, true) => {
                self.start_movement_collision_checks_x_handle_indoors()
            }
            (CollisionAxis::Horizontal, false) => {
                self.start_movement_collision_checks_x_handle_outdoors()
            }
        }
    }

    fn finish_native_vertical_surface(
        &mut self,
        tail: crate::cycle_models::vertical_surface_response::VerticalSurfaceTail,
    ) {
        use crate::cycle_models::vertical_surface_response::VerticalSurfaceTail;
        match tail {
            VerticalSurfaceTail::Returned => {
                unreachable!("completed surface response resumes its native parent")
            }
            VerticalSurfaceTail::SurfaceEffect => self.finish_indoor_y_collision_tail(),
            VerticalSurfaceTail::StairDash => {
                self.link_cancel_dash();
                if self
                    .game_state
                    .player
                    .follower_link
                    .last_direction_moved_towards()
                    & 2
                    == 0
                {
                    self.follower_link_state_mut().arm_stair_speed_modifier();
                } else if !self.finish_indoor_collision_shared_tail(true) {
                    self.handle_pushing_bonking_snaps_y();
                }
            }
            VerticalSurfaceTail::Damage => {
                self.link_cancel_dash();
                self.link_force_unequip_cape_quietly();
                self.link_apply_tile_rebound();
            }
            VerticalSurfaceTail::PushBlock => {
                let flags = self.game_state.player.tile_detection.block_flags();
                self.handle_indoor_pushblock_timeout_after_cache(true, flags);
                self.handle_pushing_bonking_snaps_y();
            }
            VerticalSurfaceTail::ResetSwim => self.handle_pushing_bonking_snaps_y(),
            VerticalSurfaceTail::Bonk {
                return_pc,
                saved_collision_low,
                smash_done,
            } => {
                if !smash_done {
                    self.link_bonk_and_smash();
                }
                self.repel_dash();
                if return_pc == 0xbde5 {
                    let collision = (self.game_state.player.tile_detection.collision_bits()
                        & 0xff00)
                        | u16::from(saved_collision_low.unwrap());
                    self.tile_detect_position_mut()
                        .set_collision_bits(collision);
                    self.finish_pushing_bonking_snaps_y_after_initial_bonk(collision, false);
                } else {
                    assert!(matches!(return_pc, 0xbe3b | 0xbe60));
                    assert!(saved_collision_low.is_none());
                    self.finish_pushing_bonking_snaps_y_after_final_bonk();
                }
            }
            VerticalSurfaceTail::Nudge => {
                let delta = self.ram[0] as i8;
                let x = self
                    .game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_add(delta as i16 as u16);
                self.follower_link_state_mut().set_x(x);
                self.handle_nudging(delta);
            }
        }
    }

    fn finish_native_vertical_response(
        &mut self,
        tail: crate::cycle_models::vertical_collision_response::VerticalResponseTail,
    ) {
        use crate::cycle_models::vertical_collision_response::VerticalResponseTail;
        match tail {
            VerticalResponseTail::Returned => {
                unreachable!("completed vertical response must resume its CPU parent")
            }
            VerticalResponseTail::Outdoor => {
                self.start_movement_collision_checks_y_handle_outdoors()
            }
            VerticalResponseTail::Doorway { perpendicular } => {
                self.link_add_in_velocity_y();
                if perpendicular {
                    self.change_axis_of_perpendicular_door_movement_y();
                } else {
                    if self.game_state.player.follower_link.tile_coll_flag() & 2 == 0 {
                        self.follower_link_state_mut().clear_direction_lock_bits(2);
                        self.set_room_transitioning_flags(0);
                        let force = read_le_u16(&self.ram, FORCE_MOVE_ANY_DIRECTION) & 0xff00;
                        self.follower_link_state_mut()
                            .set_force_move_any_direction(force);
                    }
                    self.finish_indoor_y_collision_after_setup();
                }
            }
            VerticalResponseTail::DoorSnap { velocity_done } => {
                if !velocity_done {
                    self.link_add_in_velocity_y_falling();
                }
                self.calculate_snap_scratch_y();
                self.follower_link_state_mut().clear_doorway_state();
                let bits = self.game_state.player.tile_detection.collision_bits();
                if bits & 0x20 != 0
                    && bits & 1 == 0
                    && self.game_state.player.follower_link.x() & 7 == 1
                {
                    let x = self.game_state.player.follower_link.x() & !7;
                    self.follower_link_state_mut().set_x(x);
                }
                if self.game_state.player.follower_link.tile_coll_flag() & 2 == 0 {
                    self.follower_link_state_mut().clear_direction_lock_bits(2);
                }
            }
            VerticalResponseTail::Slope => {
                self.flag_moving_into_slopes_y();
                if self
                    .game_state
                    .player
                    .follower_link
                    .moving_against_diag_tile()
                    & 15
                    == 0
                {
                    self.finish_indoor_y_collision_after_slope();
                }
            }
            VerticalResponseTail::Chest { collision } => {
                let mut chest_position = 0;
                let _ = self.OpenChestForItem(
                    self.game_state.player.tile_detection.tile_type() as u8,
                    &mut chest_position,
                );
                self.tile_detect_position_mut().clear_tile_type();
                self.tile_detect_position_mut()
                    .set_collision_bits(collision);
                self.finish_indoor_collision_common(true);
            }
            VerticalResponseTail::Rupee { tile_deleted } => {
                if !tile_deleted {
                    let y = read_le_u16(&self.ram, 0);
                    let x = read_le_u16(&self.ram, 2);
                    self.dungeon_delete_rupee_tile(x, y);
                }
                self.ancilla_sfx3_near(10);
                self.finish_indoor_collision_after_rupees(true);
            }
            VerticalResponseTail::SurfaceTail => {
                if let Some(tail) = self.early_vertical_surface_tail.take() {
                    self.finish_native_vertical_surface(tail);
                } else {
                    self.finish_indoor_y_collision_tail();
                }
            }
        }
    }

    fn finish_vertical_cardinal_probe(
        &mut self,
        handoff: crate::cycle_models::vertical_cardinal_probe::VerticalCardinalHandoff,
    ) {
        use crate::cycle_models::tile_classifier::TileClassifierTail;
        assert!(handoff.direction <= 1);
        for index in 0..3 {
            let bits = 1u16 << index;
            match handoff.samples[index] {
                Some(TileClassifierTail::Complete) => {}
                Some(TileClassifierTail::Effect { tile, offset }) => self
                    .tile_detect_execute_definition(
                        crate::tile_definition::NativeTile::from_cartridge(tile),
                        offset,
                        bits,
                        true,
                    ),
                Some(TileClassifierTail::OutdoorAttribute) | None => {
                    self.tile_detection_execute(handoff.x[index], handoff.y, bits)
                }
            }
        }
        if let Some(tail) = self.early_vertical_response_tail.take() {
            self.finish_native_vertical_response(tail);
        } else {
            self.finish_movement_collision_response(CollisionAxis::Vertical);
        }
    }

    pub(super) fn start_movement_collision_checks_y_handle_indoors(&mut self) {
        let mut r14 = self.game_state.player.tile_detection.collision_bits();
        if self
            .game_state
            .player
            .follower_link
            .is_lifting_or_carrying()
            || self.game_state.player.follower_link.incapacitated_timer() != 0
        {
            r14 |= r14 >> 4;
            self.tile_detect_position_mut().set_collision_bits(r14);
        } else {
            if self.game_state.player.follower_link.doorway_state() == 2 {
                if self
                    .game_state
                    .player
                    .follower_link
                    .num_orthogonal_directions()
                    == 0
                {
                    if self.game_state.dungeon.room_load.header_collision() != 3
                        || !self.game_state.player.follower_link.is_on_lower_level()
                    {
                        self.link_add_in_velocity_y();
                        self.change_axis_of_perpendicular_door_movement_y();
                        return;
                    }
                    // $07:BA5E `BRL $BAF4`: past the doorway and lock clears.
                    self.finish_indoor_y_collision_after_setup();
                    return;
                } else if self.game_state.player.tile_detection.door_direction_flags() != 0 {
                    // $07:BA6E `BRA $BAE3`: the lock clears, keeping the doorway.
                    self.link_add_in_velocity_y();
                    self.finish_indoor_y_collision_keeping_doorway();
                    return;
                }
            }

            if r14 & 0x70 != 0 {
                if (r14 >> 8) & 7 != 0 {
                    let force_move = if self
                        .game_state
                        .player
                        .follower_link
                        .y_velocity_signed()
                        .is_negative()
                    {
                        8
                    } else {
                        4
                    };
                    self.follower_link_state_mut()
                        .set_force_move_any_direction(force_move);
                }

                self.follower_link_state_mut().set_doorway_state(1);
                self.follower_link_state_mut().clear_conveyor_belt_state();
                if r14 & 0x70 != 0x70 {
                    if r14 & 5 != 0 {
                        self.follower_link_state_mut()
                            .clear_moving_against_diag_tile();
                        self.link_add_in_velocity_y_falling();
                        self.calculate_snap_scratch_y();
                        self.follower_link_state_mut().clear_doorway_state();
                        if r14 & 0x20 != 0
                            && r14 & 1 == 0
                            && self.game_state.player.follower_link.x() & 7 == 1
                        {
                            let x = self.game_state.player.follower_link.x() & !7;
                            self.follower_link_state_mut().set_x(x);
                        }
                        if self.game_state.player.follower_link.tile_coll_flag() & 2 == 0 {
                            self.follower_link_state_mut().clear_direction_lock_bits(2);
                        }
                        return;
                    }
                    if r14 & 0x20 != 0 {
                        if self.game_state.player.follower_link.tile_coll_flag() & 2 == 0 {
                            self.follower_link_state_mut().clear_direction_lock_bits(2);
                        }
                        return;
                    }
                } else {
                    if self.game_state.player.follower_link.tile_coll_flag() & 2 == 0 {
                        self.follower_link_state_mut().clear_direction_lock_bits(2);
                    }
                    return;
                }
            }
        }

        if self.game_state.player.follower_link.tile_coll_flag() & 2 == 0 {
            self.follower_link_state_mut().clear_doorway_state();
        }
        self.finish_indoor_y_collision();
    }

    pub(super) fn start_movement_collision_checks_x_handle_indoors(&mut self) {
        let mut r14 = self.game_state.player.tile_detection.collision_bits();
        if self
            .game_state
            .player
            .follower_link
            .is_lifting_or_carrying()
            || self.game_state.player.follower_link.incapacitated_timer() != 0
        {
            r14 |= r14 >> 4;
            self.tile_detect_position_mut().set_collision_bits(r14);
        } else {
            if self
                .game_state
                .player
                .follower_link
                .num_orthogonal_directions()
                == 0
            {
                self.follower_link_state_mut().clear_speed_modifier();
            }

            if self.game_state.player.follower_link.doorway_state() == 1
                && self
                    .game_state
                    .player
                    .follower_link
                    .num_orthogonal_directions()
                    == 0
            {
                if self.game_state.dungeon.room_load.header_collision() != 3
                    || !self.game_state.player.follower_link.is_on_lower_level()
                {
                    self.snap_on_x();
                    let spd = self.change_axis_of_perpendicular_door_movement_x();
                    self.handle_nudging_in_a_door(spd);
                    return;
                }
                // $07:C52E `BRL $C59E`: past the doorway and lock clears.
                self.finish_indoor_x_collision_after_setup();
                return;
            }

            if r14 & 0x70 != 0 {
                if (r14 >> 8) & 7 != 0 {
                    let force_move = if self
                        .game_state
                        .player
                        .follower_link
                        .x_velocity_signed()
                        .is_negative()
                    {
                        2
                    } else {
                        1
                    };
                    self.follower_link_state_mut()
                        .set_force_move_any_direction(force_move);
                }

                self.follower_link_state_mut().set_doorway_state(2);
                self.follower_link_state_mut().clear_conveyor_belt_state();
                if r14 & 0x70 != 0x70 {
                    if r14 & 7 != 0 {
                        self.follower_link_state_mut()
                            .clear_moving_against_diag_tile();
                        self.follower_link_state_mut().clear_doorway_state();
                        self.snap_on_x();
                        self.calculate_snap_scratch_x();
                        return;
                    }
                    self.follower_link_state_mut().clear_direction_lock_bits(2);
                    return;
                }

                self.follower_link_state_mut().clear_direction_lock_bits(2);
                return;
            }
        }

        if self.game_state.player.follower_link.tile_coll_flag() & 2 == 0 {
            self.follower_link_state_mut().clear_direction_lock_bits(2);
            self.follower_link_state_mut().clear_doorway_state();
            self.set_room_transitioning_flags(0);
            self.follower_link_state_mut()
                .set_force_move_any_direction(0);
        }

        self.finish_indoor_x_collision_after_setup();
    }

    /// `$07:C59E`: the slope and tile checks after the doorway clears.
    fn finish_indoor_x_collision_after_setup(&mut self) {
        if self.game_state.player.tile_detection.collision_bits() & 2 == 0
            && self.game_state.player.tile_detection.slope_collision_bits() & 5 != 0
        {
            self.follower_link_state_mut().clear_conveyor_belt_state();
            self.flag_moving_into_slopes_x();
            if self
                .game_state
                .player
                .follower_link
                .moving_against_diag_tile()
                & 0x0f
                != 0
            {
                return;
            }
        }

        self.follower_link_state_mut()
            .clear_moving_against_diag_tile();
        self.finish_indoor_collision_common(false);
    }

    pub(in crate::zelda_rtl) fn start_movement_collision_checks_y_handle_outdoors(&mut self) {
        self.follower_link_state_mut().resolve_dash_speed_setting();

        if self.game_state.player.tile_detection.pit_tile() & 5 != 0
            && self.game_state.player.tile_detection.collision_bits() & 2 == 0
        {
            self.start_falling_into_hole();
            return;
        }

        let liftable_primary = if self.game_state.player.tile_detection.read_something() & 2 != 0 {
            self.game_state.player.tile_detection.liftable_tile_index() >> 1
        } else {
            0
        };
        self.tile_detect_position_mut()
            .set_liftable_action_index_primary(liftable_primary);

        if self.game_state.player.tile_detection.deepwater() & 2 != 0
            && !self.game_state.player.follower_link.is_in_deep_water()
            && !self.game_state.player.follower_link.has_auxiliary_state()
        {
            self.link_reset_sword_and_item_usage();
            self.link_cancel_dash();
            self.follower_link_state_mut().enter_deep_water_state();
            self.follower_link_state_mut()
                .set_swim_flags_from_last_direction();
            self.follower_link_state_mut().clear_grabbing_wall();
            self.follower_link_state_mut().set_speed_setting(0);
            self.link_reset_swimming_state();
            if self
                .game_state
                .player
                .follower_link
                .water_ripple_or_grass_state()
                == 1
                && {
                    self.link_force_unequip_cape_quietly();
                    self.game_state.inventory.items.has_flippers()
                }
            {
                if !self.game_state.player.follower_link.is_bunny_mirror() {
                    self.follower_link_state_mut().set_handler_state(4);
                }
            } else {
                self.link_ledge_hop_sfx();
                self.restore_link_safe_return_position();
                self.follower_link_state_mut()
                    .set_sprite_damage_disable_timer(1);
                self.link_hop_in_or_out_of_water_y();
            }
        }

        if self.game_state.player.follower_link.is_in_deep_water() {
            if self.game_state.player.tile_detection.vertical_ledge() & 7 != 0 {
                let r14 = (self.game_state.player.tile_detection.vertical_ledge() & 7) as u16;
                self.tile_detect_position_mut().set_collision_bits(r14);
                self.handle_pushing_bonking_snaps_y();
                return;
            }
            if (self.game_state.player.tile_detection.stair_tile() & 7) == 7
                || self.game_state.player.tile_detection.normal_tiles() & 7 == 7
            {
                self.link_cancel_dash();
                self.follower_link_state_mut().clear_deep_water_state();
                if self.game_state.player.follower_link.auxiliary_state() == 0 {
                    self.follower_link_state_mut()
                        .set_last_direction_from_swim_flags();
                    self.follower_link_state_mut()
                        .set_sprite_damage_disable_timer(1);
                    self.ancilla_add_splash(0x15, 0);
                    self.link_hop_in_or_out_of_water_y();
                    return;
                }
            }
        }

        if self.game_state.player.tile_detection.horizontal_ledge() & 2 != 0
            || self.game_state.player.tile_detection.diagonal_ledge_tiles() & 0x22 != 0
        {
            self.tile_detect_position_mut().set_collision_bits(7);
            self.handle_pushing_bonking_snaps_y();
            return;
        }

        if self.game_state.player.tile_detection.vertical_ledge() & 0x70 != 0
            && self.run_ledge_hop_timer()
        {
            self.link_cancel_dash();
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.set_allow_scroll_z(1);
            self.follower_link_state_mut().set_handler_state(11);
            self.follower_link_state_mut().set_incapacitated_timer(0);
            self.follower_link_state_mut().set_z_mirror(0xffff);
            self.follower_link_state_mut().clear_defense_flags();
            self.follower_link_state_mut().set_speed_setting(0);
            let zvel = if self.game_state.player.follower_link.is_in_deep_water() {
                14
            } else {
                20
            };
            self.follower_link_state_mut()
                .set_actual_z_velocity_mirror_and_copy(zvel);
            let auxiliary_state = if self.game_state.player.follower_link.is_in_deep_water() {
                4
            } else {
                2
            };
            self.follower_link_state_mut()
                .set_auxiliary_state(auxiliary_state);
            return;
        }

        if self.game_state.player.tile_detection.vertical_ledge() & 7 != 0
            && self.run_ledge_hop_timer()
        {
            self.link_ledge_hop_sfx();
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.link_cancel_dash();
            self.follower_link_state_mut().clear_defense_flags();
            self.follower_link_state_mut().set_speed_setting(0);
            self.link_find_valid_landing_tile_north();
            return;
        }

        if !self.game_state.player.follower_link.is_in_deep_water() {
            if self
                .game_state
                .player
                .tile_detection
                .ledges_down_leftright()
                & 7
                != 0
                && self.game_state.player.tile_detection.vertical_ledge() & 0x77 == 0
            {
                let xand = if self.game_state.player.tile_detection.interacting_tile() == 0x2f {
                    4
                } else {
                    1
                };
                if self
                    .game_state
                    .player
                    .tile_detection
                    .ledges_down_leftright()
                    & xand
                    != 0
                    && self.run_ledge_hop_timer()
                {
                    self.link_cancel_dash();
                    let actual_x_velocity = if self
                        .game_state
                        .player
                        .tile_detection
                        .ledges_down_leftright()
                        & 4
                        != 0
                    {
                        16
                    } else {
                        0u8.wrapping_sub(16)
                    };
                    self.follower_link_state_mut()
                        .set_actual_x_velocity(actual_x_velocity);
                    self.setup_horizontal_ledge_hop(14);
                    return;
                }
            }

            if self.game_state.player.tile_detection.horizontal_ledge() & 0x70 != 0
                && self.game_state.player.tile_detection.vertical_ledge() & 0x77 == 0
                && self.run_ledge_hop_timer()
            {
                self.link_cancel_dash();
                self.link_ledge_hop_sfx();
                let last_direction_moved_towards =
                    if self.game_state.player.tile_detection.horizontal_ledge() & 0x40 != 0 {
                        3
                    } else {
                        2
                    };
                self.follower_link_state_mut()
                    .set_last_direction_moved_towards(last_direction_moved_towards);
                self.follower_link_state_mut()
                    .set_sprite_damage_disable_timer(1);
                self.follower_link_state_mut().clear_defense_flags();
                self.follower_link_state_mut().set_speed_setting(0);
                self.link_find_valid_landing_tile_diagonal_north();
                return;
            }
        }

        if (self.game_state.player.tile_detection.stair_tile() & 7) == 7 {
            if self.game_state.player.follower_link.incapacitated_timer() != 0 {
                let r14 = (self.game_state.player.tile_detection.stair_tile() & 7) as u16;
                self.tile_detect_position_mut().set_collision_bits(r14);
                self.handle_pushing_bonking_snaps_y();
                return;
            } else if self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards()
                & 2
                == 0
            {
                self.follower_link_state_mut().arm_stair_speed_modifier();
                return;
            }
        }

        self.follower_link_state_mut().resolve_dash_speed_setting();
        if self.game_state.player.follower_link.speed_modifier() == 1 {
            self.follower_link_state_mut()
                .promote_pending_speed_modifier();
        }

        if self.game_state.player.tile_detection.collision_bits() & 7 == 0
            && self.game_state.player.tile_detection.slope_collision_bits() & 5 != 0
        {
            self.flag_moving_into_slopes_y();
            if self
                .game_state
                .player
                .follower_link
                .moving_against_diag_tile()
                & 0x0f
                != 0
            {
                return;
            }
        }

        self.follower_link_state_mut()
            .clear_moving_against_diag_tile();
        if self
            .game_state
            .player
            .tile_detection
            .key_lock_gravestones_low()
            & 2
            != 0
            && self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards()
                == 0
        {
            let timeout = self
                .game_state
                .player
                .follower_link
                .gravestone_push_timeout()
                .wrapping_sub(1);
            self.follower_link_state_mut()
                .set_gravestone_push_timeout(timeout);
            if self.game_state.player.follower_link.is_running() || (timeout as i8).is_negative() {
                let bak = self.game_state.player.tile_detection.collision_bits();
                self.ancilla_add_grave_stone(0x24, 4);
                self.tile_detect_position_mut().set_collision_bits(bak);
                self.follower_link_state_mut()
                    .set_gravestone_push_timeout(52);
            }
        } else {
            self.follower_link_state_mut()
                .set_gravestone_push_timeout(52);
        }

        if self.game_state.player.tile_detection.spike_cactus_tiles() & 7 != 0 {
            if (self.game_state.player.follower_link.incapacitated_timer()
                | self.game_state.player.follower_link.blink_countdown()
                | self.game_state.player.follower_link.is_cape_active() as u8)
                == 0
            {
                let should_damage = if self
                    .game_state
                    .player
                    .follower_link
                    .last_direction_moved_towards()
                    == 0
                {
                    self.game_state.player.follower_link.y() & 4 == 0
                } else {
                    self.game_state.player.follower_link.y() & 4 != 0
                };
                if should_damage {
                    self.follower_link_state_mut().set_given_damage(8);
                    self.link_cancel_dash();
                    self.link_force_unequip_cape_quietly();
                    self.link_apply_tile_rebound();
                    return;
                }
            } else {
                let r14 = (self.game_state.player.tile_detection.spike_cactus_tiles() & 7) as u16;
                self.tile_detect_position_mut().set_collision_bits(r14);
            }
        }
        self.handle_pushing_bonking_snaps_y();
    }

    pub(in crate::zelda_rtl) fn start_movement_collision_checks_x_handle_outdoors(&mut self) {
        if self
            .game_state
            .player
            .follower_link
            .num_orthogonal_directions()
            == 0
        {
            self.follower_link_state_mut().clear_speed_modifier();
            if self.game_state.player.follower_link.speed_setting() == 2 {
                self.follower_link_state_mut().set_speed_setting(0);
            }
        }

        if self.game_state.player.tile_detection.pit_tile() & 5 != 0
            && self.game_state.player.tile_detection.collision_bits() & 2 == 0
        {
            self.start_falling_into_hole();
            return;
        }

        let liftable_secondary = if self.game_state.player.tile_detection.read_something() & 2 != 0
        {
            self.game_state.player.tile_detection.liftable_tile_index() >> 1
        } else {
            0
        };
        self.tile_detect_position_mut()
            .set_liftable_action_index_secondary(liftable_secondary);

        if self.game_state.player.tile_detection.deepwater() & 4 != 0
            && !self.game_state.player.follower_link.is_in_deep_water()
            && !self.game_state.player.follower_link.has_auxiliary_state()
        {
            self.link_cancel_dash();
            self.link_reset_sword_and_item_usage();
            self.follower_link_state_mut().enter_deep_water_state();
            self.follower_link_state_mut()
                .set_swim_flags_from_last_direction();
            self.link_reset_swimming_state();
            self.follower_link_state_mut().clear_grabbing_wall();
            self.follower_link_state_mut().set_speed_setting(0);
            if self
                .game_state
                .player
                .follower_link
                .water_ripple_or_grass_state()
                == 1
                && {
                    self.link_force_unequip_cape_quietly();
                    self.game_state.inventory.items.has_flippers()
                }
            {
                if !self.game_state.player.follower_link.is_bunny_mirror() {
                    self.follower_link_state_mut().set_handler_state(4);
                }
            } else {
                self.restore_link_safe_return_position();
                self.follower_link_state_mut()
                    .set_sprite_damage_disable_timer(1);
                self.link_hop_in_or_out_of_water_x();
                self.link_ledge_hop_sfx();
            }
        }

        if if self.game_state.player.follower_link.is_in_deep_water() {
            self.game_state.player.tile_detection.horizontal_ledge() & 7 == 7
        } else {
            self.game_state.player.tile_detection.vertical_ledge() & 0x42 != 0
        } {
            self.tile_detect_position_mut().set_collision_bits(7);
            self.handle_pushing_bonking_snaps_x();
            return;
        }

        if self.game_state.player.tile_detection.normal_tiles() & 7 == 7
            && self.game_state.player.follower_link.is_in_deep_water()
        {
            self.link_cancel_dash();
            if self.game_state.player.follower_link.auxiliary_state() == 0 {
                self.follower_link_state_mut()
                    .set_last_direction_from_swim_flags();
                self.follower_link_state_mut().clear_deep_water_state();
                self.follower_link_state_mut()
                    .set_sprite_damage_disable_timer(1);
                self.ancilla_add_splash(0x15, 0);
                self.link_hop_in_or_out_of_water_x();
                return;
            }
        }

        if self.game_state.player.tile_detection.horizontal_ledge() & 7 != 0
            && self.run_ledge_hop_timer()
        {
            self.link_ledge_hop_sfx();
            let actual_x_velocity = if self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards()
                & 1
                != 0
            {
                0x10
            } else {
                0u8.wrapping_sub(0x10)
            };
            self.follower_link_state_mut()
                .set_actual_x_velocity(actual_x_velocity);
            self.link_cancel_dash();
            self.setup_horizontal_ledge_hop(12);
            if self.game_state.world.location.is_outdoors() {
                self.follower_link_state_mut().set_lower_level_state(2);
            }
            let x_bak = self.game_state.player.follower_link.x();
            let rv = self.link_hopping_horizontally_find_tile_x(
                (self
                    .game_state
                    .player
                    .follower_link
                    .last_direction_moved_towards()
                    & !2)
                    * 2,
            );
            self.follower_link_state_mut()
                .set_last_direction_moved_towards(1);
            if rv != 0xff {
                self.link_hopping_horizontally_find_tile_y();
            } else {
                self.link_hop_find_tile_to_land_on_south();
            }
            self.follower_link_state_mut().set_x(x_bak);
            return;
        }

        if self.game_state.player.tile_detection.diagonal_ledge_tiles() & 0x77 != 0
            && self.run_ledge_hop_timer()
        {
            self.link_ledge_hop_sfx();
            let handler_state = if self.game_state.system_signals.sound_effect_1() & 7 == 0 {
                16
            } else {
                15
            };
            self.follower_link_state_mut()
                .set_handler_state(handler_state);
            let actual_x_velocity = if self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards()
                & 1
                != 0
            {
                0x10
            } else {
                0u8.wrapping_sub(0x10)
            };
            self.follower_link_state_mut()
                .set_actual_x_velocity(actual_x_velocity);
            self.link_cancel_dash();
            self.follower_link_state_mut().set_auxiliary_state(2);
            self.follower_link_state_mut()
                .set_actual_z_velocity_mirror_and_copy(20);
            self.set_link_z_coord_mirror_low_ff();
            self.follower_link_state_mut().set_incapacitated_timer(0);
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.set_allow_scroll_z(1);
            self.follower_link_state_mut().clear_defense_flags();
            self.follower_link_state_mut().set_speed_setting(0);
            return;
        }

        if self.game_state.player.tile_detection.horizontal_ledge() & 0x70 != 0
            && self.game_state.player.tile_detection.horizontal_ledge() & 7 == 0
            && self.game_state.player.tile_detection.diagonal_ledge_tiles() & 0x77 == 0
            && self.game_state.player.follower_link.handler_state() != 13
            && self.run_ledge_hop_timer()
        {
            self.link_ledge_hop_sfx();
            self.link_cancel_dash();
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.follower_link_state_mut().clear_defense_flags();
            self.follower_link_state_mut().set_speed_setting(0);
            self.link_find_valid_landing_tile_diagonal_north();
            return;
        }

        if self
            .game_state
            .player
            .tile_detection
            .ledges_down_leftright()
            & 7
            != 0
            && self.game_state.player.tile_detection.horizontal_ledge() & 7 == 0
            && self.game_state.player.tile_detection.diagonal_ledge_tiles() & 0x77 == 0
            && self.run_ledge_hop_timer()
        {
            let actual_x_velocity = if self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards()
                & 1
                != 0
            {
                0x10
            } else {
                0u8.wrapping_sub(0x10)
            };
            self.follower_link_state_mut()
                .set_actual_x_velocity(actual_x_velocity);
            self.link_cancel_dash();
            self.setup_horizontal_ledge_hop(14);
            return;
        }

        if self.game_state.player.tile_detection.collision_bits() & 2 == 0
            && self.game_state.player.tile_detection.slope_collision_bits() & 5 != 0
        {
            let skip_check = self.game_state.player.follower_link.is_running()
                && self.game_state.player.follower_link.facing() & 4 == 0;
            if !skip_check
                || self
                    .game_state
                    .enhanced_features
                    .has(FINISH_INDOOR_COLLISION_COMMON_FEATURES0_MISC_BUG_FIXES)
            {
                self.flag_moving_into_slopes_x();
                if self
                    .game_state
                    .player
                    .follower_link
                    .moving_against_diag_tile()
                    & 0x0f
                    != 0
                {
                    return;
                }
            }
        }

        self.follower_link_state_mut()
            .clear_moving_against_diag_tile();
        if self.game_state.player.tile_detection.spike_cactus_tiles() & 7 != 0 {
            if (self.game_state.player.follower_link.incapacitated_timer()
                | self.game_state.player.follower_link.blink_countdown()
                | self.game_state.player.follower_link.is_cape_active() as u8)
                == 0
            {
                let should_damage = if self
                    .game_state
                    .player
                    .follower_link
                    .last_direction_moved_towards()
                    == 2
                {
                    self.game_state.player.follower_link.x() & 4 == 0
                } else {
                    self.game_state.player.follower_link.x() & 4 != 0
                };
                if should_damage {
                    self.follower_link_state_mut().set_given_damage(8);
                    self.link_cancel_dash();
                    self.link_apply_tile_rebound();
                    return;
                }
            } else {
                let r14 = (self.game_state.player.tile_detection.spike_cactus_tiles() & 7) as u16;
                self.tile_detect_position_mut().set_collision_bits(r14);
            }
        }
        self.handle_pushing_bonking_snaps_x();
    }

    fn start_falling_into_hole(&mut self) {
        if self.game_state.player.follower_link.handler_state() != 5
            && self.game_state.player.follower_link.handler_state() != 2
        {
            self.follower_link_state_mut().set_sprite_oam_state_timer(9);
            self.follower_link_state_mut().begin_pit_check();
            self.follower_link_state_mut().set_handler_state(1);
        }
    }

    fn setup_horizontal_ledge_hop(&mut self, player_state: u8) {
        self.follower_link_state_mut()
            .set_sprite_damage_disable_timer(1);
        self.follower_link_state_mut().clear_defense_flags();
        self.follower_link_state_mut().set_speed_setting(0);
        self.set_allow_scroll_z(1);
        self.follower_link_state_mut().set_auxiliary_state(2);
        self.follower_link_state_mut()
            .set_actual_z_velocity_mirror_and_copy(20);
        self.set_link_z_coord_mirror_low_ff();
        self.follower_link_state_mut().set_incapacitated_timer(0);
        self.follower_link_state_mut()
            .set_handler_state(player_state);
    }

    fn finish_indoor_y_collision(&mut self) {
        if self.game_state.player.follower_link.tile_coll_flag() & 2 == 0 {
            self.follower_link_state_mut().clear_doorway_state();
        }
        self.finish_indoor_y_collision_keeping_doorway();
    }

    /// `$07:BAE3`: the movement-lock clears without the doorway clear.
    fn finish_indoor_y_collision_keeping_doorway(&mut self) {
        if self.game_state.player.follower_link.tile_coll_flag() & 2 == 0 {
            self.follower_link_state_mut().clear_direction_lock_bits(2);
            self.set_room_transitioning_flags(0);
            self.follower_link_state_mut()
                .set_force_move_any_direction(0);
        }

        self.finish_indoor_y_collision_after_setup();
    }

    fn finish_indoor_y_collision_after_setup(&mut self) {
        if self.game_state.player.tile_detection.collision_bits() & 7 == 0
            && self.game_state.player.tile_detection.slope_collision_bits() & 5 != 0
        {
            self.follower_link_state_mut().clear_conveyor_belt_state();
            self.flag_moving_into_slopes_y();
            if self
                .game_state
                .player
                .follower_link
                .moving_against_diag_tile()
                & 0x0f
                != 0
            {
                return;
            }
        }

        self.finish_indoor_y_collision_after_slope();
    }

    fn finish_indoor_y_collision_after_slope(&mut self) {
        self.follower_link_state_mut()
            .clear_moving_against_diag_tile();
        if self.game_state.player.tile_detection.key_lock_gravestones() & 0x20 != 0 {
            let bak = self.game_state.player.tile_detection.collision_bits();
            let mut chest_position = 0;
            let _ = self.OpenChestForItem(
                self.game_state.player.tile_detection.tile_type() as u8,
                &mut chest_position,
            );
            self.tile_detect_position_mut().clear_tile_type();
            self.tile_detect_position_mut().set_collision_bits(bak);
        }

        self.finish_indoor_collision_common(true);
    }

    fn finish_indoor_collision_common(&mut self, y_axis: bool) {
        let r14 = self.game_state.player.tile_detection.collision_bits();
        if !self.game_state.player.follower_link.is_on_lower_level() {
            if self.game_state.player.tile_detection.water_staircase() & 7 != 0 {
                self.set_player_layer_collision(
                    crate::game_state::constants::player::LAYER_COLLISION_BG1,
                    true,
                );
            } else if self.game_state.player.tile_detection.spike_cactus_tiles() & 7 == 0
                && r14 & 2 == 0
            {
                self.set_player_layer_collision(
                    crate::game_state::constants::player::LAYER_COLLISION_BG1,
                    false,
                );
            }
        } else if self.game_state.player.tile_detection.moving_floor_tiles() & 7 != 0 {
            self.set_player_layer_collision(
                crate::game_state::constants::player::LAYER_COLLISION_BG2,
                true,
            );
        } else {
            self.set_player_layer_collision(
                crate::game_state::constants::player::LAYER_COLLISION_BG2,
                false,
            );
        }

        if self.game_state.player.tile_detection.misc_tiles() & 0x2200 != 0 {
            let dy = if self.game_state.player.tile_detection.misc_tiles() & 0x2000 != 0 {
                8
            } else {
                0
            };
            let dir = self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards() as usize;
            let rupees = self
                .game_state
                .inventory
                .player_resources
                .rupees_goal()
                .wrapping_add(5);
            self.player_resources_mut().set_rupees_goal(rupees);
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(FINISH_INDOOR_COLLISION_COMMON_RUPEE_Y_OFFSETS[dir] as u16)
                .wrapping_sub(dy);
            let x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(FINISH_INDOOR_COLLISION_COMMON_RUPEE_X_OFFSETS[dir] as u16);
            self.dungeon_delete_rupee_tile(x, y);
            self.ancilla_sfx3_near(10);
        }

        self.finish_indoor_collision_after_rupees(y_axis);
    }

    fn finish_indoor_collision_after_rupees(&mut self, y_axis: bool) {
        let r14 = self.game_state.player.tile_detection.collision_bits();
        let moving_floor_flags = self
            .game_state
            .dungeon
            .environment
            .moving_floor_check_flags();
        if moving_floor_flags & 0x22 != 0 {
            self.follower_link_state_mut()
                .set_conveyor_belt_state(if moving_floor_flags & 0x20 != 0 { 2 } else { 1 });
        } else if moving_floor_flags & 0x2200 != 0 {
            self.follower_link_state_mut().set_conveyor_belt_state(
                if moving_floor_flags & 0x2000 != 0 {
                    4
                } else {
                    3
                },
            );
        } else if self.game_state.player.tile_detection.spike_cactus_tiles() & 7 == 0
            && r14 & 2 == 0
        {
            self.follower_link_state_mut().clear_conveyor_belt_state();
        }

        if y_axis {
            self.finish_indoor_y_collision_tail();
        } else {
            self.finish_indoor_x_collision_tail();
        }
    }

    fn finish_indoor_y_collision_tail(&mut self) {
        if (self.game_state.player.tile_detection.vertical_ledge() & 7) == 7
            && self.run_ledge_hop_timer()
        {
            self.link_cancel_dash();
            self.follower_link_state_mut()
                .increment_about_to_jump_off_ledge();
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.follower_link_state_mut().set_auxiliary_state(2);
            self.ancilla_sfx2_near(0x20);
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.link_hop_in_or_out_of_water_y();
        } else if (self.game_state.player.tile_detection.deepwater() & 7) == 7
            && !self.game_state.player.follower_link.is_in_deep_water()
        {
            self.link_cancel_dash();
            if self.game_state.display.sub_screen_layers == 0 {
                self.dungeon_handle_layer_change();
            } else {
                self.follower_link_state_mut().enter_deep_water_state();
                self.follower_link_state_mut()
                    .set_swim_flags_from_last_direction();
                self.follower_link_state_mut()
                    .clear_state_item_and_grab_flags();
                self.follower_link_state_mut().set_speed_setting(0);
                self.link_reset_swimming_state();
                self.ancilla_sfx2_near(0x20);
            }
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.link_hop_in_or_out_of_water_y();
        } else if self.game_state.player.tile_detection.normal_tiles() & 2 != 0
            && self.game_state.player.follower_link.is_in_deep_water()
        {
            if self.game_state.player.follower_link.has_auxiliary_state() {
                self.tile_detect_position_mut().set_collision_bits(7);
            } else {
                self.link_cancel_dash();
                self.follower_link_state_mut()
                    .set_last_direction_from_swim_flags();
                self.follower_link_state_mut().clear_deep_water_state();
                if self.ancilla_add_splash(0x15, 0) {
                    self.follower_link_state_mut().enter_deep_water_state();
                    self.tile_detect_position_mut().set_collision_bits(7);
                } else {
                    self.follower_link_state_mut()
                        .set_sprite_damage_disable_timer(1);
                    self.link_hop_in_or_out_of_water_y();
                }
            }
        }

        if (self.game_state.player.tile_detection.stair_tile() & 7) == 7 {
            if self.game_state.player.follower_link.incapacitated_timer() != 0 {
                let stair_bits = (self.game_state.player.tile_detection.stair_tile() & 7) as u16;
                self.tile_detect_position_mut()
                    .set_collision_bits(stair_bits);
                self.handle_pushing_bonking_snaps_y();
                return;
            }
            let stairs = self.game_state.player.tile_detection.inroom_staircase();
            if stairs & 0x77 != 0 {
                let submodule = if stairs & 0x70 != 0 { 16 } else { 8 };
                self.set_submodule(submodule);
                self.set_main_module(7);
                self.link_cancel_dash();
            } else {
                if self
                    .game_state
                    .enhanced_features
                    .has(LINK_STATE_DASHING_FEATURES0_TURN_WHILE_DASHING)
                {
                    self.link_cancel_dash();
                }
            }
            if self
                .game_state
                .player
                .follower_link
                .last_direction_moved_towards()
                & 2
                == 0
            {
                self.follower_link_state_mut().arm_stair_speed_modifier();
                return;
            }
        }

        if self.finish_indoor_collision_shared_tail(true) {
            return;
        }
        self.handle_pushing_bonking_snaps_y();
    }

    fn finish_indoor_x_collision_tail(&mut self) {
        if self.game_state.player.tile_detection.horizontal_ledge() & 7 == 7
            && self.run_ledge_hop_timer()
        {
            self.link_cancel_dash();
            self.follower_link_state_mut()
                .increment_about_to_jump_off_ledge();
            self.follower_link_state_mut().set_auxiliary_state(2);
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.link_hop_in_or_out_of_water_x();
            self.link_ledge_hop_sfx();
            return;
        }

        if self.finish_indoor_collision_shared_tail(false) {
            return;
        }
        if self
            .game_state
            .player
            .follower_link
            .num_orthogonal_directions()
            == 0
        {
            self.follower_link_state_mut().clear_speed_modifier();
            if self.game_state.player.follower_link.speed_setting() == 2 {
                self.follower_link_state_mut().set_speed_setting(0);
            }
        }
        self.handle_pushing_bonking_snaps_x();
    }

    fn finish_indoor_collision_shared_tail(&mut self, y_axis: bool) -> bool {
        if y_axis {
            self.follower_link_state_mut().resolve_dash_speed_setting();
            self.follower_link_state_mut()
                .promote_pending_speed_modifier();
        }

        let r14 = self.game_state.player.tile_detection.collision_bits();
        if self.game_state.player.tile_detection.pit_tile() & 5 != 0 && r14 & 2 == 0 {
            self.start_falling_into_hole();
            return true;
        }

        if y_axis {
            self.follower_link_state_mut().clear_pit_data_index();
        } else {
            self.follower_link_state_mut().clear_near_pit_state();
        }
        if self.game_state.player.tile_detection.spike_cactus_tiles() & 7 != 0 {
            if (self.game_state.player.follower_link.incapacitated_timer()
                | self.game_state.player.follower_link.blink_countdown()
                | self.game_state.player.follower_link.is_cape_active() as u8)
                == 0
            {
                let coord = if self
                    .game_state
                    .player
                    .follower_link
                    .last_direction_moved_towards()
                    & 2
                    == 0
                {
                    self.game_state.player.follower_link.y()
                } else {
                    self.game_state.player.follower_link.x()
                };
                let low_phase = coord & 4 == 0;
                let damage = if self
                    .game_state
                    .player
                    .follower_link
                    .last_direction_moved_towards()
                    & 1
                    == 0
                {
                    low_phase
                } else {
                    !low_phase
                };
                if damage {
                    self.follower_link_state_mut().set_given_damage(8);
                    self.link_cancel_dash();
                    self.link_force_unequip_cape_quietly();
                    self.link_apply_tile_rebound();
                    return true;
                }
            } else {
                let spike_bits =
                    (self.game_state.player.tile_detection.spike_cactus_tiles() & 7) as u16;
                self.tile_detect_position_mut()
                    .set_collision_bits(spike_bits);
            }
        }

        self.finish_indoor_collision_shared_tail_after_damage(y_axis);
        false
    }

    fn finish_indoor_collision_shared_tail_after_damage(&mut self, y_axis: bool) {
        if self.game_state.dungeon.room_load.header_collision() == 0
            || self.game_state.dungeon.room_load.header_collision() == 4
            || !self.game_state.player.follower_link.is_on_lower_level()
        {
            self.handle_indoor_pushblock_timeout(y_axis);
        }
    }

    fn handle_indoor_pushblock_timeout(&mut self, y_axis: bool) {
        let block_flags = self.game_state.player.tile_detection.block_flags();
        if block_flags != 0
            && self
                .game_state
                .player
                .follower_link
                .num_orthogonal_directions()
                == 0
        {
            self.tile_detect_position_mut()
                .set_staircase_cache(block_flags as u8);
            self.handle_indoor_pushblock_timeout_after_cache(y_axis, block_flags);
            return;
        }
        self.follower_link_state_mut()
            .set_gravestone_push_timeout(21);
    }

    fn handle_indoor_pushblock_timeout_after_cache(&mut self, y_axis: bool, block_flags: u16) {
        self.follower_link_state_mut()
            .decrement_gravestone_push_timeout();
        if !(self
            .game_state
            .player
            .follower_link
            .gravestone_push_timeout() as i8)
            .is_negative()
        {
            return;
        }
        let mut bits = block_flags;
        for i in (0..=15).rev() {
            if bits & 0x8000 != 0 {
                let idx = self.find_free_moving_block_slot(i);
                if idx != 0xff {
                    let slot = idx as usize;
                    self.tile_detect_position_mut()
                        .set_collision_bits(idx as u16);
                    if !self.initialize_push_block(idx, i * 2) {
                        self.sprite_dungeon_draw_single_push_block(slot * 2);
                        self.tile_detect_position_mut().set_collision_bits(4);
                        let facing = self
                            .game_state
                            .player
                            .follower_link
                            .last_direction_moved_towards()
                            * 2;
                        self.pushed_block_mut().set_facing_player(slot, facing);
                        self.pushed_block_mut().set_push_direction(facing);
                        let target = if y_axis {
                            let y_lo = self.game_state.player.pushed_block.y_low(slot);
                            y_lo.wrapping_sub(u8::from(
                                self.game_state
                                    .player
                                    .follower_link
                                    .last_direction_moved_towards()
                                    == 1,
                            ))
                        } else {
                            let x_lo = self.game_state.player.pushed_block.x_low(slot);
                            x_lo.wrapping_sub(u8::from(
                                self.game_state
                                    .player
                                    .follower_link
                                    .last_direction_moved_towards()
                                    != 2,
                            ))
                        } & 0x0f;
                        self.pushed_block_mut().set_target_low(slot, target);
                    }
                }
            }
            bits <<= 1;
        }
        self.follower_link_state_mut()
            .set_gravestone_push_timeout(21);
    }

    pub(super) fn finish_native_diagonal_collision(&mut self, tail: crate::cycle_models::diagonal_collision_prefix::DiagonalCollisionTail) {
        match tail {
            crate::cycle_models::diagonal_collision_prefix::DiagonalCollisionTail::Complete => {}
            crate::cycle_models::diagonal_collision_prefix::DiagonalCollisionTail::HorizontalProbe { primary_layer, direction } => {
                assert!(matches!(direction, 2 | 3));
                self.player_limit_directions_after_vertical(Some(CollisionDirection::from_legacy(direction)));
                if primary_layer {
                    self.create_velocity_from_moving_background();
                    self.follower_link_state_mut().mask_direction(0x0f);
                    self.player_limit_directions_inner();
                }
            }
            crate::cycle_models::diagonal_collision_prefix::DiagonalCollisionTail::VerticalProbe { primary_layer, direction } => {
                assert!(direction <= 1);
                self.player_limit_directions_after_reset(Some(CollisionDirection::from_legacy(direction)));
                if primary_layer {
                    self.create_velocity_from_moving_background();
                    self.follower_link_state_mut().mask_direction(0x0f);
                    self.player_limit_directions_inner();
                }
            }
        }
    }

    pub(in crate::zelda_rtl) fn link_handle_diagonal_collision(&mut self) {
        if self.check_if_room_needs_double_layer_check() {
            self.player_limit_directions_inner();
            self.create_velocity_from_moving_background();
        }
        self.follower_link_state_mut().mask_direction(0x0f);
        self.player_limit_directions_inner();
    }

    pub(super) fn link_handle_diagonal_kickback(&mut self) {
        if self.game_state.player.follower_link.x_velocity() == 0
            || self.game_state.player.follower_link.y_velocity() == 0
        {
            self.follower_link_state_mut()
                .set_moving_against_diag_deadlocked(0);
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
            return;
        }

        self.follower_link_state_mut()
            .cache_copied_position_from_current();

        self.link_handle_diagonal_kickback_after_cache();
    }

    fn link_handle_diagonal_kickback_after_cache(&mut self) {
        self.detect_player_movement(
            CollisionAxis::Horizontal,
            CollisionDirection::along(
                CollisionAxis::Horizontal,
                self.game_state
                    .player
                    .follower_link
                    .x_velocity_signed()
                    .is_negative(),
            ),
            MovementProbeKind::Cardinal,
        );
        if self.game_state.player.tile_detection.slope_collision_bits() & 5 == 0 {
            self.follower_link_state_mut()
                .set_moving_against_diag_deadlocked(0);
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
            return;
        }
        self.flag_moving_into_slopes_x();
        if self
            .game_state
            .player
            .follower_link
            .moving_against_diag_tile()
            & 0x0f
            == 0
        {
            self.follower_link_state_mut()
                .set_moving_against_diag_deadlocked(0);
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
            return;
        }

        let xd = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_sub(self.game_state.player.follower_link.copied_x()) as u8;
        let copied_x = self.game_state.player.follower_link.copied_x();
        self.follower_link_state_mut().set_x(copied_x);
        self.follower_link_state_mut().set_x_velocity(xd);

        self.detect_player_movement(
            CollisionAxis::Vertical,
            CollisionDirection::along(
                CollisionAxis::Vertical,
                self.game_state
                    .player
                    .follower_link
                    .y_velocity_signed()
                    .is_negative(),
            ),
            MovementProbeKind::Cardinal,
        );
        if self.game_state.player.tile_detection.slope_collision_bits() & 5 == 0 {
            self.follower_link_state_mut()
                .set_moving_against_diag_deadlocked(0);
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
            return;
        }
        self.flag_moving_into_slopes_y();
        if self
            .game_state
            .player
            .follower_link
            .moving_against_diag_tile()
            & 0x0f
            == 0
        {
            self.follower_link_state_mut()
                .set_moving_against_diag_deadlocked(0);
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
            return;
        }

        let diag_tile = self
            .game_state
            .player
            .follower_link
            .moving_against_diag_tile();
        self.follower_link_state_mut()
            .set_moving_against_diag_deadlocked(diag_tile);
        let yd = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_sub(self.game_state.player.follower_link.copied_y()) as u8;
        self.follower_link_state_mut().set_y_velocity(yd);

        let x_vel = self.game_state.player.follower_link.x_velocity_signed();
        let x_idx = x_vel.unsigned_abs() as usize;
        let x_delta = if x_vel < 0 {
            LINK_HANDLE_DIAGONAL_KICKBACK_X_OFFSETS_1[x_idx]
        } else {
            LINK_HANDLE_DIAGONAL_KICKBACK_X_OFFSETS_0[x_idx]
        };
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(x_delta as i16 as u16);
        self.follower_link_state_mut().set_x(x);

        let y_vel = self.game_state.player.follower_link.y_velocity_signed();
        let y_idx = y_vel.unsigned_abs() as usize;
        let y_delta = if y_vel < 0 {
            LINK_HANDLE_DIAGONAL_KICKBACK_Y_OFFSETS_1[y_idx]
        } else {
            LINK_HANDLE_DIAGONAL_KICKBACK_Y_OFFSETS_0[y_idx]
        };
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(y_delta as i16 as u16);
        self.follower_link_state_mut().set_y(y);

        self.follower_link_state_mut()
            .clear_moving_against_diag_tile();
    }

    fn check_primary_collision_layer(&mut self) {
        let order = CollisionOrder::VerticalFirst;
        if self.game_state.dungeon.room_load.header_collision() >= 2
            && self.game_state.dungeon.room_load.header_collision() != 3
        {
            self.follower_link_state_mut().set_tile_coll_flag(2);
            self.player_tile_detect_nearby();
            self.check_primary_collision_layer_after_nearby();
            return;
        }
        self.run_slope_collision_checks(order);
    }

    fn check_primary_collision_layer_after_nearby(&mut self) {
        let mut order = CollisionOrder::VerticalFirst;
        let collision_bits = self.game_state.player.tile_detection.collision_bits() as u8;
        self.tile_detect_position_mut()
            .set_tile_collision_bits_primary(collision_bits);
        if self
            .game_state
            .player
            .tile_detection
            .tile_collision_bits_primary()
            != 0
        {
            let floor_x_velocity =
                self.game_state.dungeon.moving_floor.floor_x_velocity_low() as u16;
            let floor_y_velocity =
                self.game_state.dungeon.moving_floor.floor_y_velocity_low() as u16;
            self.follower_link_state_mut()
                .add_movement_velocity_delta(floor_x_velocity, floor_y_velocity);
            order = CollisionOrder::for_moving_floor(
                self.game_state.player.tile_detection.collision_bits() as u8,
                self.game_state.player.follower_link.x_velocity(),
                self.game_state.player.follower_link.y_velocity(),
                self.game_state.dungeon.moving_floor.floor_y_velocity_low() as i8,
            );
        }
        self.run_slope_collision_checks(order);
    }

    pub(in crate::zelda_rtl) fn finish_native_cardinal_collision(
        &mut self,
        tail: crate::cycle_models::cardinal_collision_prefix::CardinalCollisionTail,
    ) {
        use crate::cycle_models::cardinal_collision_prefix::CardinalCollisionTail;
        match tail {
            CardinalCollisionTail::KickbackHorizontal { direction } => {
                assert!(matches!(direction, 2 | 3));
                self.link_handle_diagonal_kickback_after_cache();
                if self
                    .game_state
                    .player
                    .follower_link
                    .moving_against_diag_deadlocked()
                    == 0
                    && self.check_if_room_needs_double_layer_check()
                {
                    self.check_primary_collision_layer();
                    self.create_velocity_from_moving_background();
                }
                self.finish_cardinal_collision_after_primary();
            }
            CardinalCollisionTail::PrimaryFootprint => {
                self.player_tile_detect_nearby();
                self.check_primary_collision_layer_after_nearby();
                self.create_velocity_from_moving_background();
                self.finish_cardinal_collision_after_primary();
            }
            CardinalCollisionTail::FinalFootprint { collision_two } => {
                self.player_tile_detect_nearby();
                self.finish_cardinal_collision_after_nearby(collision_two);
            }
            CardinalCollisionTail::SlopeChecks {
                primary_layer,
                horizontal_first,
                vertical_completed,
            } => {
                if vertical_completed {
                    assert!(self.early_vertical_cardinal_probe.is_none());
                    self.start_movement_collision_checks(CollisionAxis::Horizontal);
                } else if let Some(handoff) = self.early_vertical_cardinal_probe.take() {
                    self.finish_vertical_cardinal_probe(handoff);
                    if !horizontal_first
                        && !CollisionAxis::Horizontal.blocked_by_slope(
                            self.game_state
                                .player
                                .follower_link
                                .moving_against_diag_tile(),
                        )
                    {
                        self.start_movement_collision_checks(CollisionAxis::Horizontal);
                    }
                } else {
                    self.run_slope_collision_checks(if horizontal_first {
                        CollisionOrder::HorizontalFirst
                    } else {
                        CollisionOrder::VerticalFirst
                    });
                }
                if primary_layer {
                    self.create_velocity_from_moving_background();
                    self.finish_cardinal_collision_after_primary();
                } else {
                    self.finish_cardinal_collision_after_checks();
                }
            }
            CardinalCollisionTail::MainHandler => self.finish_cardinal_collision_after_checks(),
        }
    }

    pub(in crate::zelda_rtl) fn link_handle_cardinal_collision(&mut self) {
        self.tile_detect_position_mut().clear_diag_state();
        self.tile_detect_position_mut().clear_diagonal_tile();

        let can_double_layer = if self
            .game_state
            .player
            .follower_link
            .moving_against_diag_tile()
            & 0x30
            != 0
        {
            true
        } else {
            self.link_handle_diagonal_kickback();
            self.game_state
                .player
                .follower_link
                .moving_against_diag_deadlocked()
                == 0
        };

        if can_double_layer && self.check_if_room_needs_double_layer_check() {
            self.check_primary_collision_layer();
            self.create_velocity_from_moving_background();
        }

        self.finish_cardinal_collision_after_primary();
    }

    fn finish_cardinal_collision_after_primary(&mut self) {
        let collision = self.game_state.dungeon.room_load.header_collision();
        let moved = (self.game_state.player.follower_link.x_velocity()
            | self.game_state.player.follower_link.y_velocity())
            != 0;
        if collision == 2 {
            self.player_tile_detect_nearby();
            self.finish_cardinal_collision_after_nearby(true);
            return;
        } else if collision == 3 {
            self.follower_link_state_mut().set_tile_coll_flag(1);
            self.run_slope_collision_checks(CollisionOrder::HorizontalFirst);
        } else if collision == 4 || moved {
            self.follower_link_state_mut().set_tile_coll_flag(1);
            self.run_slope_collision_checks(CollisionOrder::VerticalFirst);
        } else if !self
            .game_state
            .player
            .follower_link
            .is_edge_transition_blocked_by_handler_state()
            && self.game_state.player.follower_link.handler_state() != 19
        {
            self.player_tile_detect_nearby();
            self.finish_cardinal_collision_after_nearby(false);
            return;
        }

        self.finish_cardinal_collision_after_checks();
    }

    fn finish_cardinal_collision_after_nearby(&mut self, collision_two: bool) {
        if collision_two {
            if (self.game_state.player.tile_detection.collision_bits() as u8
                | self
                    .game_state
                    .player
                    .tile_detection
                    .tile_collision_bits_primary())
                == 0x0f
            {
                if self.game_state.player.follower_link.blink_countdown() == 0 {
                    self.follower_link_state_mut().set_blink_countdown(58);
                }
                if self.game_state.player.follower_link.direction() == 0 {
                    if self.game_state.dungeon.moving_floor.floor_y_velocity_low() != 0 {
                        let y_velocity = self.game_state.player.follower_link.y_velocity();
                        self.follower_link_state_mut()
                            .set_y_velocity((0u8).wrapping_sub(y_velocity));
                    }
                    if self.game_state.dungeon.moving_floor.floor_x_velocity_low() != 0 {
                        let x_velocity = self.game_state.player.follower_link.x_velocity();
                        self.follower_link_state_mut()
                            .set_x_velocity((0u8).wrapping_sub(x_velocity));
                    }
                }
            }
            self.follower_link_state_mut().set_tile_coll_flag(1);
            self.run_slope_collision_checks(CollisionOrder::VerticalFirst);
        } else {
            if self.game_state.player.tile_detection.pit_tile() & 0x0f != 0 {
                self.follower_link_state_mut().set_handler_state(1);
                if !self.game_state.player.follower_link.is_running() {
                    self.follower_link_state_mut().set_speed_setting(4);
                }
            }
        }
        self.finish_cardinal_collision_after_checks();
    }

    fn finish_cardinal_collision_after_checks(&mut self) {
        if let Some(tail)=self.early_main_tile_response.take() {
            let probe=self.early_main_tile_probe.take().expect("native main response requires a completed probe");
            assert_eq!(probe.item,0);
            assert_eq!(probe.classifier,crate::cycle_models::tile_classifier::TileClassifierTail::Complete);
            use crate::cycle_models::main_tile_response::MainTileResponseTail;
            match tail {
                MainTileResponseTail::Returned=>return,
                MainTileResponseTail::WhirlpoolIndoor|MainTileResponseTail::WhirlpoolOutdoor=>self.finish_main_tile_response(),
                MainTileResponseTail::Sound(sound)=>{self.ancilla_sfx2_near(sound);},
                MainTileResponseTail::Damage=>self.link_cancel_dash(),
            }
        } else {self.tile_detect_main_handler(0);}
        self.finish_cardinal_collision_after_main();
    }

    fn finish_cardinal_collision_after_main(&mut self) {
        if self
            .game_state
            .player
            .follower_link
            .num_orthogonal_directions()
            != 0
        {
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
        }

        self.follower_link_state_mut()
            .refresh_direction_from_safe_return_delta();

        if self.game_state.world.location.is_outdoors()
            || self.game_state.dungeon.room_load.header_collision() != 4
            || self.game_state.player.follower_link.handler_state() != 4
        {
            return;
        }

        if self.game_state.dungeon.moving_floor.floor_y_velocity_low() != 0
            && self
                .game_state
                .player
                .follower_link
                .y_velocity()
                .wrapping_sub(self.game_state.dungeon.moving_floor.floor_y_velocity_low())
                == 0
        {
            if (self.game_state.dungeon.moving_floor.floor_y_velocity_low() as i8).is_negative() {
                self.follower_link_state_mut().clear_direction_flags(8);
            } else {
                self.follower_link_state_mut().clear_direction_flags(4);
            }
        }
        if self.game_state.dungeon.moving_floor.floor_x_velocity_low() != 0
            && self
                .game_state
                .player
                .follower_link
                .x_velocity()
                .wrapping_sub(self.game_state.dungeon.moving_floor.floor_x_velocity_low())
                == 0
        {
            if (self.game_state.dungeon.moving_floor.floor_x_velocity_low() as i8).is_negative() {
                self.follower_link_state_mut().clear_direction_flags(2);
            } else {
                self.follower_link_state_mut().clear_direction_flags(1);
            }
        }
    }
}
