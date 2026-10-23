//! Ancilla slot allocation and effect spawning.

use super::*;
use crate::types::{sign8};

impl ZeldaState {
    pub(in crate::zelda_rtl) fn ancilla_add_blanket(&mut self, a: u8) {
        let k = 0;
        let floor = self.game_state.player.follower_link.lower_level_state();
        let mirror_floor = self
            .game_state
            .player
            .follower_link
            .lower_level_mirror_state();
        {
            let mut blanket = self.ancilla_slot_view_mut(k);
            blanket.set_ancilla_type(a);
            blanket.set_num_sprites(ANCILLA_DRAW_SPRITE_COUNTS[a as usize]);
            blanket.set_floor(floor);
            blanket.set_floor2(mirror_floor);
            blanket.set_object_priority(0);
        }
        self.ancilla_set_xy(k, 0x0938, 0x2162);
    }

    pub(in crate::zelda_rtl) fn ancilla_add_cape_poof(&mut self, ty: u8, limit: u8) {
        if let Some(k) = self.ancilla_add_simple(ty, limit) {
            self.follower_link_state_mut().set_transforming();
            self.follower_link_state_mut().set_direction_lock_bits(1);
            self.follower_link_state_mut().set_direction(0);
            self.follower_link_state_mut().set_last_direction(0);
            {
                let mut cape_poof = self.ancilla_slot_view_mut(k);
                cape_poof.set_step(1);
                cape_poof.set_item_to_link(0);
                cape_poof.set_aux_timer(7);
            }
            let x = self.game_state.player.follower_link.x();
            let y = self.game_state.player.follower_link.y().wrapping_add(4);
            self.ancilla_set_xy(k, x, y);
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_hit_stars(&mut self, a: u8, y: u8) {
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            {
                let mut ancilla = self.ancilla_slot_view_mut(k);
                ancilla.set_aux_timer(2);
                ancilla.set_work_byte_3(1);
                ancilla.set_item_to_link(0);
                ancilla.set_y_velocity(0);
                ancilla.set_x_velocity(0);
            }

            let mut j = a;
            if self.game_state.player.follower_link.has_item_in_hand() {
                j = (self.game_state.player.follower_link.facing() >> 1).wrapping_add(2);
            } else if self.game_state.player.follower_link.has_position_mode() {
                j = if self.game_state.player.follower_link.facing() != 4 {
                    1
                } else {
                    0
                };
            }

            self.ancilla_slot_view_mut(k).set_step(j);
            let j = j as usize;
            let link_x = self.game_state.player.follower_link.x();
            let link_y = self.game_state.player.follower_link.y();
            let t = link_x.wrapping_add(ANCILLA_ADD_HIT_STARS_SHOVEL_HIT_STARS_X2[j] as i16 as u16);
            let value = t as u8;
            self.ancilla_slot_view_mut(k).set_a(value);
            let value = (t >> 8) as u8;
            self.ancilla_slot_view_mut(k).set_b(value);
            let offset = ANCILLA_ADD_HIT_STARS_SHOVEL_HIT_STARS_OFFSET[j];
            self.ancilla_set_xy(
                k,
                link_x.wrapping_add(offset.x as i16 as u16),
                link_y.wrapping_add(offset.y as i16 as u16),
            );
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_fire_rod_shot(&mut self, type_: u8, _y: u8) {
        let y = 1;
        let Some(mut j) = self.ancilla_alloc_init(type_, y) else {
            if type_ != 1 {
                self.refund_magic(0);
            }
            return;
        };

        if type_ != 1 {
            self.ancilla_sfx2_near(0x0e);
        }

        let mut i = self.game_state.player.follower_link.facing_index();
        {
            let mut ancilla = self.ancilla_slot_view_mut(j);
            ancilla.set_ancilla_type(type_);
            ancilla.set_num_sprites(ANCILLA_DRAW_SPRITE_COUNTS[type_ as usize]);
            ancilla.set_object_priority(0);
            ancilla.set_u(0);
            ancilla.set_step(0);
            ancilla.set_timer(3);
            ancilla.set_item_to_link(0);
            ancilla.set_direction(i as u8);
        }

        if self.ancilla_check_initial_tile_a(j) < 0 {
            self.ancilla_set_xy(
                j,
                self.game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_add(ANCILLA_ADD_FIRE_ROD_SHOT_FIRE_ROD_X[i] as i16 as u16),
                self.game_state
                    .player
                    .follower_link
                    .y()
                    .wrapping_add(ANCILLA_ADD_FIRE_ROD_SHOT_FIRE_ROD_Y[i] as i16 as u16),
            );
            if type_ != 1 {
                let mut ancilla = self.ancilla_slot_view_mut(j);
                ancilla.set_x_velocity(ANCILLA_ADD_FIRE_ROD_SHOT_FIRE_ROD_XVEL[i] as u8);
                ancilla.set_y_velocity(ANCILLA_ADD_FIRE_ROD_SHOT_FIRE_ROD_YVEL[i] as u8);
            } else {
                i += self.game_state.inventory.items.sword_type().wrapping_sub(2) as usize * 4;
                let mut ancilla = self.ancilla_slot_view_mut(j);
                ancilla.set_x_velocity(FIRE_ROD_SPARK_X_VELOCITIES[i] as u8);
                ancilla.set_y_velocity(FIRE_ROD_SPARK_Y_VELOCITIES[i] as u8);
            }
            let floor = self.game_state.player.follower_link.lower_level_state();
            let mirror_floor = self
                .game_state
                .player
                .follower_link
                .lower_level_mirror_state();
            let mut ancilla = self.ancilla_slot_view_mut(j);
            ancilla.set_floor(floor);
            ancilla.set_floor2(mirror_floor);
        } else if type_ == 1 {
            let mut ancilla = self.ancilla_slot_view_mut(j);
            ancilla.set_ancilla_type(4);
            ancilla.set_timer(7);
            ancilla.set_num_sprites(16);
        } else {
            let mut ancilla = self.ancilla_slot_view_mut(j);
            ancilla.set_step(1);
            ancilla.set_timer(31);
            ancilla.set_num_sprites(8);
            j = self.game_state.player.follower_link.facing_index();
            self.ancilla_sfx2_pan(j, 0x2a);
        }
    }

    pub(in crate::zelda_rtl) fn add_sword_beam(&mut self, y: u8) {
        let Some(k) = self.ancilla_add_simple(0x0c, y) else {
            return;
        };
        let mut j = self.game_state.player.follower_link.facing() as usize * 2;
        self.effect_angle_scratch_mut()
            .set_angles4(&ADD_SWORD_BEAM_SWORD_BEAM_TRAILING_ANGLES, j);
        self.effect_angle_scratch_mut()
            .set_trailing_angle(ADD_SWORD_BEAM_SWORD_BEAM_TRAILING_ANGLES[j + 3]);
        self.effect_angle_scratch_mut().set_radial_radius(14);
        j = self.game_state.player.follower_link.facing_index();
        {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_l(0);
            ancilla.set_g(0);
            ancilla.set_work_byte_1(0);
            ancilla.set_aux_timer(2);
            ancilla.set_work_byte_3(8);
            ancilla.set_step(0);
            ancilla.set_item_to_link(0x4c);
            ancilla.set_direction(j as u8);
            ancilla.set_y_velocity(ADD_SWORD_BEAM_SWORD_BEAM_YVEL[j] as u8);
            ancilla.set_x_velocity(ADD_SWORD_BEAM_SWORD_BEAM_XVEL[j] as u8);
            ancilla.set_s_player(ADD_SWORD_BEAM_SWORD_BEAM_S[j] as u8);
        }

        let swordbeam_temp_y = self.game_state.player.follower_link.y().wrapping_add(12);
        let swordbeam_temp_x = self.game_state.player.follower_link.x().wrapping_add(8);
        self.ether_orbit_mut()
            .set_swordbeam_temp(swordbeam_temp_x, swordbeam_temp_y);

        if self.ancilla_check_initial_tile_a(k) >= 0 {
            self.ancilla_set_xy(
                k,
                swordbeam_temp_x.wrapping_add(ADD_SWORD_BEAM_SWORD_BEAM_X[j] as i16 as u16),
                swordbeam_temp_y.wrapping_add(ADD_SWORD_BEAM_SWORD_BEAM_Y[j] as i16 as u16),
            );
            self.set_sound_effect_2_with_ancilla_pan(k, 1);
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_ancilla_type(4);
            ancilla.set_timer(7);
            ancilla.set_num_sprites(16);
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_spawn_sword_charge_sparkle(&mut self) {
        let Some(k) = self.ancilla_alloc_high() else {
            return;
        };
        {
            let mut sparkle = self.ancilla_slot_view_mut(k);
            sparkle.set_ancilla_type(0x3c);
            sparkle.set_item_to_link(0);
            sparkle.set_timer(4);
        }
        let floor = self.game_state.player.follower_link.lower_level_state();
        self.ancilla_slot_view_mut(k).set_floor(floor);
        let j = self.game_state.player.follower_link.facing_index();
        let mut x = 0i8;
        let mut y = 0i8;
        let m0 = ANCILLA_SPAWN_SWORD_CHARGE_SPARKLE_SWORD_CHARGE_SPARKLE_A[j];
        if m0 == 0 {
            y = (self
                .game_state
                .player
                .follower_link
                .spin_attack_step_counter()
                >> 2) as i8;
            if j == 0 {
                y = -y;
            }
        }
        let m1 = ANCILLA_SPAWN_SWORD_CHARGE_SPARKLE_SWORD_CHARGE_SPARKLE_B[j];
        if m1 == 0 {
            x = (self
                .game_state
                .player
                .follower_link
                .spin_attack_step_counter()
                >> 2) as i8;
            if j == 2 {
                x = -x;
            }
        }
        let r = self.get_random_number();
        let dst_x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(x as i16 as u16)
            .wrapping_add(ANCILLA_SPAWN_SWORD_CHARGE_SPARKLE_SWORD_CHARGE_SPARKLE_X[j] as u16)
            .wrapping_add(((r & m1) >> 4) as u16);
        let dst_y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(y as i16 as u16)
            .wrapping_add(ANCILLA_SPAWN_SWORD_CHARGE_SPARKLE_SWORD_CHARGE_SPARKLE_Y[j] as u16)
            .wrapping_add((r & m0) as u16);
        self.ancilla_set_xy(k, dst_x, dst_y);
    }

    pub(in crate::zelda_rtl) fn ancilla_add_sword_charge_sparkle_from_ancilla(&mut self, source: usize) {
        let Some(k) = self.ancilla_alloc_high() else {
            return;
        };
        {
            let mut sparkle = self.ancilla_slot_view_mut(k);
            sparkle.set_ancilla_type(60);
            sparkle.set_item_to_link(0);
            sparkle.set_timer(4);
        }
        let floor = self.game_state.player.follower_link.lower_level_state();
        self.ancilla_slot_view_mut(k).set_floor(floor);

        let rand = self.get_random_number();
        let mut z = self.ancilla_slot_view(source).z();
        if z >= 0xf8 {
            z = 0;
        }
        self.ancilla_set_xy(
            k,
            self.ancilla_get_x(source)
                .wrapping_add(2)
                .wrapping_add(u16::from(rand >> 5)),
            self.ancilla_get_y(source)
                .wrapping_sub(2)
                .wrapping_sub(u16::from(z))
                .wrapping_add(u16::from(rand & 0x0f)),
        );
    }

    fn add_dashing_dust_ex(&mut self, a: u8, y: u8, flag: u8) {
        if let Some(k) = self.ancilla_add_simple(a, y) {
            let j = self.game_state.player.follower_link.facing_index();
            {
                let mut dust = self.ancilla_slot_view_mut(k);
                dust.set_step(flag);
                dust.set_item_to_link(0);
                dust.set_timer(3);
                dust.set_direction(j as u8);
            }
            if flag == 0 {
                self.ancilla_set_xy(
                    k,
                    self.game_state.player.follower_link.x(),
                    self.game_state.player.follower_link.y().wrapping_add(20),
                );
            } else {
                self.ancilla_set_xy(
                    k,
                    self.game_state
                        .player
                        .follower_link
                        .x()
                        .wrapping_add(ADD_DASHING_DUST_EX_ADD_DASHING_DUST_X[j] as i16 as u16),
                    self.game_state
                        .player
                        .follower_link
                        .y()
                        .wrapping_add(ADD_DASHING_DUST_EX_ADD_DASHING_DUST_Y[j] as i16 as u16),
                );
            }
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_dash_dust(&mut self, a: u8, y: u8) {
        self.add_dashing_dust_ex(a, y, 1);
    }

    pub(in crate::zelda_rtl) fn ancilla_add_dash_dust_charging(&mut self, a: u8, y: u8) {
        self.add_dashing_dust_ex(a, y, 0);
    }

    pub(in crate::zelda_rtl) fn ancilla_add_arrow(
        &mut self,
        a: u8,
        ax: u8,
        ay: u8,
        xcoord: u16,
        ycoord: u16,
    ) -> i32 {
        self.tile_detect_position_mut()
            .set_interaction_scratch_y(ycoord);
        self.tile_detect_position_mut()
            .set_interaction_scratch_x(xcoord);
        self.tile_detect_position_mut().set_interacting_tile_low(ax);

        if self.ancilla_add_check_for_presence(a) {
            return -1;
        }

        let k = self.ancilla_add_arrow_find_slot(a, ay);

        if k >= 0 {
            let k = k as usize;
            self.set_sound_effect_1_with_link_pan(7);
            let j = (ax >> 1) as usize;
            {
                let mut ancilla = self.ancilla_slot_view_mut(k);
                ancilla.set_h(0);
                ancilla.set_item_to_link(8);
                ancilla.set_direction(j as u8 | 4);
                ancilla.set_y_velocity(ANCILLA_ADD_ARROW_SHOOT_BOW_YVEL[j] as u8);
                ancilla.set_x_velocity(ANCILLA_ADD_ARROW_SHOOT_BOW_XVEL[j] as u8);
            }
            self.ancilla_set_xy(
                k,
                xcoord.wrapping_add(ANCILLA_ADD_ARROW_SHOOT_BOW_X[j] as i16 as u16),
                ycoord
                    .wrapping_add(8)
                    .wrapping_add(ANCILLA_ADD_ARROW_SHOOT_BOW_Y[j] as i16 as u16),
            );
        }
        k
    }

    fn ancilla_add_arrow_find_slot(&mut self, type_: u8, ay: u8) -> i32 {
        let mut n = 0;
        for k in (0..=4).rev() {
            if self.ancilla_slot_view(k).ancilla_type() == 10 {
                n += 1;
            }
        }

        let mut k = -1;
        if n != ay.wrapping_add(1) {
            for i in (0..=4).rev() {
                if self.ancilla_slot_view(i).ancilla_type() == 0 {
                    k = i as i32;
                    break;
                }
            }
        } else {
            loop {
                let rotate = self.ancilla_alloc_rotate().wrapping_sub(1);
                self.set_ancilla_alloc_rotate(rotate);
                if sign8(rotate) {
                    self.set_ancilla_alloc_rotate(4);
                }
                k = i32::from(self.ancilla_alloc_rotate());
                // `LDA $0C4A,X` with the rotation index: on the hardware the
                // index can exceed the five slots (arr25[2] aliases $03C4), so
                // read the raw byte like the ROM does.
                if self.ram[crate::game_state::constants::ANCILLA_TYPE + k as usize] == 10 {
                    break;
                }
            }
        }

        if k >= 0 {
            let k = k as usize;
            let floor = self.game_state.player.follower_link.lower_level_state();
            let mirror_floor = self
                .game_state
                .player
                .follower_link
                .lower_level_mirror_state();
            {
                let mut ancilla = self.ancilla_slot_view_mut(k);
                ancilla.set_ancilla_type(type_);
                ancilla.set_floor(floor);
                ancilla.set_floor2(mirror_floor);
                ancilla.set_y_velocity(0);
                ancilla.set_x_velocity(0);
                ancilla.set_object_priority(0);
                ancilla.set_u(0);
                ancilla.set_num_sprites(ANCILLA_DRAW_SPRITE_COUNTS[type_ as usize]);
            }
        }
        k
    }

    fn add_bird_common(&mut self, k: usize) {
        {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_y_velocity(0);
            ancilla.set_x_velocity(56);
            ancilla.set_item_to_link(0);
        }
        {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_aux_timer(1);
            ancilla.set_work_byte_3(3);
        }
        {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_k(0);
            ancilla.set_g(0);
        }

        let xt: u16 = if self.game_state.enhanced_features.has(1) {
            0x40
        } else {
            0
        };
        self.ancilla_set_xy(
            k,
            self.game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_sub(16)
                .wrapping_sub(xt),
            self.game_state.player.follower_link.y().wrapping_sub(8),
        );
    }

    pub(super) fn ancilla_add_boomerang_wall_clink(&mut self, k: usize) {
        let temp_x = self.ancilla_get_x(k);
        let temp_y = self.ancilla_get_y(k);
        self.minigame_state_mut().set_boomerang_temp_x(temp_x);
        self.minigame_state_mut().set_boomerang_temp_y(temp_y);
        // BOOMERANG_TEMP_X/Y (0x399-0x39c) alias ANCILLA_G[5..9]; the writes above bypassed
        // the ancilla model, so resync it or a later ancilla slot's sync re-stamps the stale
        // frame-start value over the temp coords (f370034).
        self.game_state
            .sprites
            .reload_ancilla_slots_from_ram(&self.ram);
        if let Some(k) = self.ancilla_add_ancilla(6, 1) {
            {
                let mut wall_clink = self.ancilla_slot_view_mut(k);
                wall_clink.set_item_to_link(0);
                wall_clink.set_work_byte_3(1);
            }
            let j = (ANCILLA_ADD_BOOMERANG_WALL_CLINK_BOOMERANG_WALL_HIT_OFFSET_INDEX
                [self.game_state.messaging.runtime.effect_index() as usize]
                >> 1) as usize;
            self.ancilla_set_xy(
                k,
                self.game_state.minigame.boomerang_temp_x().wrapping_add(
                    ANCILLA_ADD_BOOMERANG_WALL_CLINK_BOOMERANG_WALL_HIT_X[j] as i16 as u16,
                ),
                self.game_state.minigame.boomerang_temp_y().wrapping_add(
                    ANCILLA_ADD_BOOMERANG_WALL_CLINK_BOOMERANG_WALL_HIT_Y[j] as i16 as u16,
                ),
            );
        }
    }

    pub(in crate::zelda_rtl) fn call_for_duck_indoors(&mut self) {
        self.ancilla_sfx2_near(0x13);
        self.ancilla_add_duck_take_off(0x27, 4);
    }

    pub(in crate::zelda_rtl) fn ancilla_add_duck_take_off(&mut self, a: u8, y: u8) {
        if self.ancilla_add_check_for_presence(a) {
            return;
        }
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            self.ancilla_slot_view_mut(k).set_timer(0x78);
            let value = 0;
            self.ancilla_slot_view_mut(k).set_l(value);
            {
                let mut duck = self.ancilla_slot_view_mut(k);
                duck.set_z_velocity(0);
                duck.set_z(0);
                duck.set_step(0);
            }
            self.add_bird_common(k);
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_bunny_poof(&mut self, a: u8, y: u8) {
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            self.follower_link_state_mut().set_visibility_status(0x0c);
            let effect = if !self.game_state.player.follower_link.is_bunny_mirror() {
                0x14
            } else {
                0x15
            };
            self.set_sound_effect_1_with_link_pan(effect);
            {
                let mut poof = self.ancilla_slot_view_mut(k);
                poof.set_step(0);
                poof.set_item_to_link(0);
                poof.set_aux_timer(7);
            }
            self.ancilla_set_xy(
                k,
                self.game_state.player.follower_link.x(),
                self.game_state.player.follower_link.y().wrapping_add(4),
            );
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_dwarf_poof(&mut self, ain: u8, yin: u8) {
        let Some(k) = self.ancilla_add_ancilla(ain, yin) else {
            return;
        };
        let effect = if self.game_state.sprites.follower_runtime.indicator() == 8 {
            0x14
        } else {
            0x15
        };
        self.set_sound_effect_1_with_link_pan(effect);

        {
            let mut poof = self.ancilla_slot_view_mut(k);
            poof.set_item_to_link(0);
            poof.set_step(0);
            poof.set_aux_timer(7);
        }
        self.follower_state_mut().set_appearance_none_flag(1);
        let j = self.game_state.sprites.follower_runtime.data_index() as usize;
        let x = self.tagalong_slot(j).x();
        let y = self.tagalong_slot(j).y();
        self.ancilla_set_xy(k, x, y.wrapping_add(4));
    }

    pub(in crate::zelda_rtl) fn ancilla_add_bush_poof(&mut self, x: u16, y: u16) {
        if !self.game_state.player.follower_link.item_in_hand_has(0x40) {
            return;
        }
        if let Some(k) = self.ancilla_add_ancilla(0x3f, 4) {
            let mut poof = self.ancilla_slot_view_mut(k);
            poof.set_item_to_link(0);
            poof.set_timer(7);
            self.set_sound_effect_1_with_link_pan(21);
            self.ancilla_set_xy(k, x, y.wrapping_sub(2));
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_victory_spin(&mut self) {
        if self.game_state.inventory.items.sword_type().wrapping_add(1) & 0xfe != 0 {
            if let Some(k) = self.ancilla_add_ancilla(0x3b, 0) {
                let mut spin = self.ancilla_slot_view_mut(k);
                spin.set_item_to_link(0);
                spin.set_work_byte_3(1);
                spin.set_aux_timer(34);
            }
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_wall_tap_spark(&mut self, a: u8, y: u8) {
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            {
                let mut spark = self.ancilla_slot_view_mut(k);
                spark.set_item_to_link(5);
                spark.set_aux_timer(1);
            }
            let i = self.game_state.player.follower_link.facing_index();
            self.ancilla_set_xy(
                k,
                self.game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_add(ANCILLA_ADD_WALL_TAP_SPARK_WALL_TAP_SPARK_X[i] as i16 as u16),
                self.game_state
                    .player
                    .follower_link
                    .y()
                    .wrapping_add(ANCILLA_ADD_WALL_TAP_SPARK_WALL_TAP_SPARK_Y[i] as i16 as u16),
            );
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_dash_tremor(&mut self, a: u8, y: u8) {
        if self.ancilla_add_check_for_presence(a) {
            return;
        }
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            self.ancilla_slot_view_mut(k).set_item_to_link(16);
            let value = 0;
            self.ancilla_slot_view_mut(k).set_l(value);
            let mut j = self.game_state.player.follower_link.facing_index();
            j = ANCILLA_ADD_DASH_TREMOR_ADD_DASH_TREMOR_DIR[j] as usize;
            self.ancilla_slot_view_mut(k).set_direction(j as u8);
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
                as u8;
            let x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2())
                as u8;
            let coord = if j != 0 { y } else { x };
            self.ancilla_set_y(
                k,
                if coord < ANCILLA_ADD_DASH_TREMOR_DASH_TREMOR_COORD_LIMITS[j >> 1] {
                    3
                } else {
                    (-3i8) as u16
                },
            );
        }
    }

    pub(super) fn ancilla_add_hookshot_wall_clink(&mut self, kin: usize, a: u8, y: u8) {
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            {
                let mut wall_clink = self.ancilla_slot_view_mut(k);
                wall_clink.set_item_to_link(0);
                wall_clink.set_work_byte_3(1);
            }
            let j = self.ancilla_slot_view(kin).direction() as usize;
            self.ancilla_set_xy(
                k,
                self.ancilla_get_x(kin).wrapping_add(
                    ANCILLA_ADD_HOOKSHOT_WALL_CLINK_HOOKSHOT_WALL_HIT_X[j] as i16 as u16,
                ),
                self.ancilla_get_y(kin).wrapping_add(
                    ANCILLA_ADD_HOOKSHOT_WALL_CLINK_HOOKSHOT_WALL_HIT_Y[j] as i16 as u16,
                ),
            );
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_gt_cutscene(&mut self) {
        if let Some(k) = self.ancilla_add_gt_cutscene_before_graphics() {
            self.decode_animated_sprite_tile_variable(0x28);
            self.ancilla_add_gt_cutscene_after_graphics(k);
        }
    }

    /// `AncillaAdd_GTCutscene` up to its `DecodeAnimatedSpriteTile_variable`
    /// call: the guards, sparkle termination, the cutscene ancilla, the
    /// waterfall-sprite kills and the sparkle phase fill. Returns the added
    /// ancilla slot when the decode is reached.
    pub(in crate::zelda_rtl) fn ancilla_add_gt_cutscene_before_graphics(&mut self) -> Option<usize> {
        if self
            .game_state
            .player
            .follower_link
            .is_lifting_or_carrying()
            || self.game_state.player.follower_link.has_auxiliary_state()
            || self.game_state.inventory.player_resources.crystal_flags() & 0x7f != 0x7f
            || self.game_state.world.overworld.event_info.event_info(0x43) & 0x20 != 0
        {
            return None;
        }

        self.ancilla_terminate_sparkle_objects();

        if self.ancilla_add_check_for_presence(0x43) {
            return None;
        }

        let k = self.ancilla_add_ancilla(0x43, 4)?;

        for i in (0..=15).rev() {
            if self.sprite_slot_view(i).sprite_type() == 0x37 {
                let value = 0;
                self.sprite_slot_view_mut(i).set_state(value);
            }
        }

        for i in (0..=0x17).rev() {
            self.tower_seal_sparkle_mut(i).set_phase(0xff);
        }
        Some(k)
    }

    /// `AncillaAdd_GTCutscene` after its `DecodeAnimatedSpriteTile_variable`
    /// call, for the cutscene ancilla `k` it added.
    pub(in crate::zelda_rtl) fn ancilla_add_gt_cutscene_after_graphics(&mut self, k: usize) {
        self.set_sp6r_indoors(4);
        self.select_overworld_aux_palette_offset();
        self.palette_load_sprite_environment_dungeon();
        self.increment_cgram_update_flag();
        self.follower_link_state_mut().immobilize();
        let value = 0;
        self.ancilla_slot_view_mut(k).set_y_subpixel(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_x_subpixel(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_step(value);
        self.tower_seal_scratch_mut().set_wait_countdown(240);
        self.tower_seal_scratch_mut().set_ring_radius(0);

        self.tower_seal_orbit_mut(0).set_angle(0);
        self.tower_seal_orbit_mut(1).set_angle(10);
        self.tower_seal_orbit_mut(2).set_angle(22);
        self.tower_seal_orbit_mut(3).set_angle(32);
        self.tower_seal_orbit_mut(4).set_angle(42);
        self.tower_seal_orbit_mut(5).set_angle(54);

        self.ancilla_set_xy(
            k,
            self.game_state.player.follower_link.x(),
            self.game_state.player.follower_link.y().wrapping_sub(16),
        );
    }

    pub(in crate::zelda_rtl) fn add_bird_travel_something(&mut self, a: u8, y: u8) {
        if self.ancilla_add_check_for_presence(a) {
            return;
        }
        if let Some(k) = self.ancilla_add_simple(a, y) {
            self.follower_link_state_mut().clear_handler_state();
            self.follower_link_state_mut().set_speed_setting(0);
            self.follower_link_state_mut()
                .clear_button_mask_b_y_bits(0x81);
            self.follower_link_state_mut().clear_button_b_frames();
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(0);
            self.follower_link_state_mut().clear_direction_lock_bits(1);
            let value = 1;
            self.ancilla_slot_view_mut(k).set_l(value);

            let enhanced_bird_travel = self.game_state.enhanced_features.has(1);
            let mut bird = self.ancilla_slot_view_mut(k);
            if enhanced_bird_travel {
                bird.set_z_velocity(58);
                bird.set_z((-105i8) as u8);
            } else {
                bird.set_z_velocity(40);
                bird.set_z((-51i8) as u8);
            }
            bird.set_step(2);
            self.add_bird_common(k);
        }
    }

    fn ancilla_add_check_for_presence(&self, a: u8) -> bool {
        (0..=5)
            .rev()
            .any(|k| self.ancilla_slot_view(k).ancilla_type() == a)
    }

    pub(in crate::zelda_rtl) fn ancilla_add_snoring(&mut self, a: u8, y: u8) {
        let Some(k) = self.ancilla_add_simple(a, y) else {
            return;
        };
        {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_item_to_link(0);
            ancilla.set_y_velocity((-8i8) as u8);
            ancilla.set_x_velocity(8);
            ancilla.set_aux_timer(7);
            ancilla.set_step(255);
        }
        self.ancilla_set_xy(
            k,
            self.game_state.player.follower_link.x().wrapping_add(16),
            self.game_state.player.follower_link.y().wrapping_add(4),
        );
    }

    pub(in crate::zelda_rtl) fn ancilla_add_boomerang(&mut self, a: u8, y: u8) -> u8 {
        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            return 0;
        };
        let value = 0;
        self.ancilla_slot_view_mut(k).set_k(value);
        {
            let mut boomerang = self.ancilla_slot_view_mut(k);
            boomerang.set_aux_timer(0);
            boomerang.set_item_to_link(0);
            boomerang.set_z(0);
        }
        let value = self.ancilla_slot_view(k).num_sprites();
        self.ancilla_slot_view_mut(k).set_l(value);
        self.minigame_state_mut().set_flag_boomerang_in_place(1);
        let mut j = self.game_state.inventory.items.boomerang().wrapping_sub(1) as usize;
        let value = j as u8;
        self.ancilla_slot_view_mut(k).set_g(value);
        {
            let mut boomerang = self.ancilla_slot_view_mut(k);
            boomerang.set_step(ANCILLA_ADD_BOOMERANG_BOOMERANG_INITIAL_STEP_BY_TYPE[j]);
            boomerang.set_work_byte_3(ANCILLA_ADD_BOOMERANG_BOOMERANG_FRAME_RESET_BY_TYPE[j]);
        }

        let s = self.ancilla_slot_view(k).g() as usize * 2
            + if self.game_state.player.follower_link.joypad1h_last() & 0x0c != 0
                && self.game_state.player.follower_link.joypad1h_last() & 3 != 0
            {
                1
            } else {
                0
            };
        let r0 = ANCILLA_ADD_BOOMERANG_BOOMERANG_SPEED_BY_TYPE[s];
        let value = r0;
        self.ancilla_slot_view_mut(k).set_h(value);

        let r1 = if self.game_state.player.follower_link.joypad1h_last() & 0x0f != 0 {
            self.game_state.player.follower_link.joypad1h_last() & 0x0f
        } else {
            ANCILLA_ADD_BOOMERANG_BOOMERANG_DIRECTION_BITS
                [self.game_state.player.follower_link.facing_index()]
        };
        self.messaging_state_mut().clear_effect_index();

        if r1 & 0x0c != 0 {
            let y_velocity = if r1 & 8 != 0 { (-(r0 as i8)) as u8 } else { r0 };
            self.ancilla_slot_view_mut(k).set_y_velocity(y_velocity);
            let i = if sign8(y_velocity) { 0 } else { 1 };
            self.ancilla_slot_view_mut(k).set_direction(i);
            self.messaging_state_mut()
                .set_effect_index(ANCILLA_ADD_BOOMERANG_BOOMERANG_DIRECTION_BITS[i as usize]);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_s_player(value);

        if r1 & 3 != 0 {
            if r1 & 2 == 0 {
                let value = 1;
                self.ancilla_slot_view_mut(k).set_s_player(value);
            }
            let x_velocity = if r1 & 2 != 0 { (-(r0 as i8)) as u8 } else { r0 };
            self.ancilla_slot_view_mut(k).set_x_velocity(x_velocity);
            let i = if sign8(x_velocity) { 2 } else { 3 };
            self.ancilla_slot_view_mut(k).set_direction(i);
            self.messaging_state_mut()
                .or_effect_index(ANCILLA_ADD_BOOMERANG_BOOMERANG_DIRECTION_BITS[i as usize]);
        }

        j = ANCILLA_ADD_BOOMERANG_BOOMERANG_INPUT_MASKS
            .iter()
            .position(|&v| v == r1)
            .unwrap_or(0);
        let value = ANCILLA_ADD_BOOMERANG_BOOMERANG_INITIAL_SPIN_BY_INPUT[j];
        self.ancilla_slot_view_mut(k).set_work_byte_1(value);
        let value = (j << 1) as u8;
        self.ancilla_slot_view_mut(k).set_work_byte_23(value);
        if self.game_state.player.follower_link.button_b_frames() >= 9 {
            self.ancilla_slot_view_mut(k).add_aux_timer(1);
        } else if s != 0 || self.game_state.player.follower_link.joypad1h_last() & 0x0f == 0 {
            j = self.game_state.player.follower_link.facing_index();
        }

        let s = self.ancilla_check_initial_tile_a(k);
        if s < 0 {
            if self.ancilla_slot_view(k).aux_timer() != 0 {
                self.ancilla_set_xy(
                    k,
                    self.game_state.player.follower_link.x().wrapping_add(
                        ANCILLA_ADD_BOOMERANG_BOOMERANG_CHARGED_X_OFFSET[j] as i16 as u16,
                    ),
                    self.game_state
                        .player
                        .follower_link
                        .y()
                        .wrapping_add(8)
                        .wrapping_add(
                            ANCILLA_ADD_BOOMERANG_BOOMERANG_CHARGED_Y_OFFSET[j] as i16 as u16,
                        ),
                );
            } else {
                self.ancilla_set_xy(
                    k,
                    self.game_state.player.follower_link.x().wrapping_add(
                        ANCILLA_ADD_BOOMERANG_BOOMERANG_INITIAL_X_OFFSET[j] as i16 as u16,
                    ),
                    self.game_state
                        .player
                        .follower_link
                        .y()
                        .wrapping_add(8)
                        .wrapping_add(
                            ANCILLA_ADD_BOOMERANG_BOOMERANG_INITIAL_Y_OFFSET[j] as i16 as u16,
                        ),
                );
            }
        } else {
            self.ancilla_slot_view_mut(k).clear();
            self.minigame_state_mut().clear_flag_boomerang_in_place();
            let effect = if self.ancilla_slot_view(k).tile_attribute() != 0xf0 {
                5
            } else {
                6
            };
            self.set_sound_effect_1_with_ancilla_pan(k, effect);
            self.ancilla_add_boomerang_wall_clink(k);
        }
        s as u8
    }

    pub(super) fn ancilla_add_cutscene_duck(&mut self, a: u8, y: u8) {
        if self.ancilla_add_check_for_presence(a) {
            return;
        }
        if let Some(k) = self.ancilla_add_ancilla(a, y) {
            {
                let mut duck = self.ancilla_slot_view_mut(k);
                duck.set_direction(2);
                duck.set_work_byte_3(3);
                duck.set_step(0);
                duck.set_aux_timer(32);
                duck.set_item_to_link(116);
                duck.set_z_velocity(0);
                duck.set_z(0);
            }
            let value = 0;
            self.ancilla_slot_view_mut(k).set_l(value);
            let value = 0;
            self.ancilla_slot_view_mut(k).set_s_player(value);
            self.ancilla_set_xy(k, 0x0200, 0x0788);
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_exploding_weather_vane(&mut self, a: u8, y: u8) {
        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            return;
        };

        {
            let mut weather_vane = self.ancilla_slot_view_mut(k);
            weather_vane.set_aux_timer(10);
            weather_vane.set_step(0);
            weather_vane.set_work_byte_3(0);
        }
        let value = 128;
        self.ancilla_slot_view_mut(k).set_g(value);
        self.set_sound_effect_1(0);
        self.set_music_control(0xf2);
        self.set_ambient_sound_effect(0x17);

        self.set_weather_vane_music_latch(0);
        self.set_weather_vane_countdown(0x0280);

        for i in (0..=11).rev() {
            self.weather_vane_debris_mut(i).initialize(
                u16::from(ANCILLA_ADD_EXPLODING_WEATHER_VANE_WEATHERVANE_DEBRIS_START_X[i])
                    | 0x0200,
                u16::from(ANCILLA_ADD_EXPLODING_WEATHER_VANE_WEATHERVANE_DEBRIS_START_Y[i])
                    | 0x0700,
                ANCILLA_ADD_EXPLODING_WEATHER_VANE_WEATHERVANE_DEBRIS_X_VELOCITY[i] as u8,
                0,
                ANCILLA_ADD_EXPLODING_WEATHER_VANE_WEATHERVANE_DEBRIS_Y_VELOCITY[i] as u8,
                ANCILLA_ADD_EXPLODING_WEATHER_VANE_WEATHERVANE_DEBRIS_CHAR[i],
                (i & 1) as u8,
            );
        }
    }

    pub(super) fn ancilla_add_ice_rod_sparkle(&mut self, k: usize) {
        if self.game_state.frame.submodule != 0 {
            return;
        }
        self.ancilla_slot_view_mut(k).subtract_work_byte_4(1);
        if !sign8(self.ancilla_slot_view(k).work_byte_4()) {
            return;
        }

        let value = 5;

        self.ancilla_slot_view_mut(k).set_work_byte_4(value);
        if let Some(j) = self.ancilla_alloc_high() {
            let value = 0x13;
            self.ancilla_slot_view_mut(j).set_ancilla_type(value);
            let value = 15;
            self.ancilla_slot_view_mut(j).set_timer(value);

            let i = self.ancilla_slot_view(k).direction() as usize;
            {
                let mut sparkle = self.ancilla_slot_view_mut(j);
                sparkle.set_x_velocity(ANCILLA_ADD_ICE_ROD_SPARKLE_ICE_SHOT_SPARKLE_XVEL[i] as u8);
                sparkle.set_y_velocity(ANCILLA_ADD_ICE_ROD_SPARKLE_ICE_SHOT_SPARKLE_YVEL[i] as u8);
            }

            let value = self.ancilla_slot_view(k).x_low();

            self.ancilla_slot_view_mut(j).set_x_low(value);
            let value = self.ancilla_slot_view(k).y_low();
            self.ancilla_slot_view_mut(j).set_y_low(value);
            let value = self.ancilla_slot_view(k).floor();
            self.ancilla_slot_view_mut(j).set_floor(value);
            let value = 0;
            self.ancilla_slot_view_mut(j).set_num_sprites(value);
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_simple(&mut self, ty: u8, limit: u8) -> Option<usize> {
        self.ancilla_add_ancilla(ty, limit)
    }

    pub(super) fn ancilla_add_ancilla(&mut self, a: u8, y: u8) -> Option<usize> {
        let k = self.ancilla_alloc_init(a, y)?;
        let value = a;
        self.ancilla_slot_view_mut(k).set_ancilla_type(value);
        let value = self.game_state.player.follower_link.lower_level_state();
        self.ancilla_slot_view_mut(k).set_floor(value);
        let value = self
            .game_state
            .player
            .follower_link
            .lower_level_mirror_state();
        self.ancilla_slot_view_mut(k).set_floor2(value);
        {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_y_velocity(0);
            ancilla.set_x_velocity(0);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_object_priority(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_u(value);
        let value = ANCILLA_DRAW_SPRITE_COUNTS[a as usize];
        self.ancilla_slot_view_mut(k).set_num_sprites(value);
        Some(k)
    }

    fn ancilla_alloc_high(&self) -> Option<usize> {
        (0..=9)
            .rev()
            .find(|&k| self.ancilla_slot_view(k).ancilla_type() == 0)
    }

    pub(in crate::zelda_rtl) fn ancilla_alloc_init(&mut self, ty: u8, limit: u8) -> Option<usize> {
        if self.game_state.system_signals.bugs_fixed() >= BUGFIX_POLY_RENDERER {
            self.tile_detect_position_mut()
                .set_collision_bits(u16::from(limit.wrapping_add(1)));
        }

        let n = (0..5)
            .filter(|&i| self.ancilla_slot_view(i).ancilla_type() == ty)
            .count();
        if limit as usize + 1 == n {
            return None;
        }

        let start = if ty == 7 || ty == 8 {
            limit as usize
        } else {
            4
        };
        for j in (0..=start).rev() {
            if self.ancilla_slot_view(j).ancilla_type() == 0 {
                return Some(j);
            }
        }

        // ancilla_alloc_rotate ($03C4) is also ancilla_arr25[2] / arr26[4] on
        // the hardware, so the rotation index may be any byte (the fairy
        // revival stores 9 there). Mirror the ROM: `LDX $03C4 : DEX` and read
        // `$0C4A,X` raw for any X.
        let mut k = i32::from(self.ancilla_alloc_rotate());
        loop {
            k -= 1;
            if k < 0 {
                k = i32::from(limit);
            }
            let old_type = self.ram[crate::game_state::constants::ANCILLA_TYPE + k as usize];
            if old_type == 0x3c || old_type == 0x13 || old_type == 0x0a {
                self.set_ancilla_alloc_rotate(k as u8);
                if k as usize >= crate::game_state::ANCILLA_SLOT_COUNT {
                    // The ROM would now build the ancilla in a slot past the
                    // arrays (garbage RAM). No modeled slot exists for it;
                    // treat it as allocation failure and record the event.
                    eprintln!(
                        "ancilla allocation rotation selected unmodeled slot {k} (host {})",
                        self.frame_ctr_dbg
                    );
                    return None;
                }
                return Some(k as usize);
            }
            if k == 0 {
                break;
            }
        }
        self.set_ancilla_alloc_rotate(0);
        None
    }

    pub(super) fn ancilla_add_add_ancilla_bank08(&mut self, ty: u8, y: u8) -> Option<usize> {
        self.ancilla_add_simple(ty, y)
    }

    pub(in crate::zelda_rtl) fn ancilla_add_charged_spin_attack_sparkle(&mut self) {
        for k in (0..10).rev() {
            if self.ancilla_slot_view(k).ancilla_type() == 0
                || self.ancilla_slot_view(k).ancilla_type() == 0x3c
            {
                self.ancilla_slot_view_mut(k).set_ancilla_type(13);
                let value = self.game_state.player.follower_link.lower_level_state();
                self.ancilla_slot_view_mut(k).set_floor(value);
                self.ancilla_slot_view_mut(k).set_timer(6);
                break;
            }
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_sword_swing_sparkle(&mut self, a: u8, y: u8) {
        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            return;
        };
        let direction = self.game_state.player.follower_link.facing() >> 1;
        {
            let mut sparkle = self.ancilla_slot_view_mut(k);
            sparkle.set_item_to_link(0);
            sparkle.set_aux_timer(1);
            sparkle.set_direction(direction);
        }
        self.ancilla_set_xy(
            k,
            self.game_state.player.follower_link.x(),
            self.game_state.player.follower_link.y(),
        );
    }

    pub(in crate::zelda_rtl) fn ancilla_add_spin_attack_init_spark(&mut self, a: u8, x: u8, y: u8) {
        let k = self.ancilla_add_ancilla(a, y);
        for i in (0..=4).rev() {
            if self.ancilla_slot_view(i).ancilla_type() == 0x31 {
                let value = 0;
                self.ancilla_slot_view_mut(i).set_ancilla_type(value);
            }
        }
        let j = self.game_state.player.follower_link.facing_index();
        let spark_x = self.game_state.player.follower_link.x().wrapping_add(
            ANCILLA_ADD_SPIN_ATTACK_INIT_SPARK_SPIN_ATTACK_START_SPARKLE_X[j] as i16 as u16,
        );
        let spark_y = self.game_state.player.follower_link.y().wrapping_add(
            ANCILLA_ADD_SPIN_ATTACK_INIT_SPARK_SPIN_ATTACK_START_SPARKLE_Y[j] as i16 as u16,
        );
        let Some(k) = k else {
            // C writes through k = -1 on allocation failure; preserve those
            // aliasing writes explicitly instead of silently returning.
            self.ancilla_spawn_scratch_mut()
                .write_failed_spin_sparkle(x, spark_x, spark_y);
            return;
        };
        {
            let mut sparkle = self.ancilla_slot_view_mut(k);
            sparkle.set_item_to_link(0);
            sparkle.set_step(x);
            sparkle.set_timer(4);
            sparkle.set_aux_timer(3);
        }
        self.ancilla_set_xy(k, spark_x, spark_y);
    }

    pub(super) fn ancilla_add_sword_charge_sparkle(&mut self, k: usize) {
        let mut j = 9usize;
        while self.ancilla_slot_view(j).ancilla_type() != 0 {
            if j == 0 {
                return;
            }
            j -= 1;
        }
        self.ancilla_slot_view_mut(j).set_ancilla_type(60);
        let value = self.game_state.player.follower_link.lower_level_state();
        self.ancilla_slot_view_mut(j).set_floor(value);
        {
            let mut sparkle = self.ancilla_slot_view_mut(j);
            sparkle.set_item_to_link(0);
            sparkle.set_timer(4);
        }

        let rand = self.get_random_number();

        let mut z = self.ancilla_slot_view(k).z();
        if z >= 0xf8 {
            z = 0;
        }
        let dst_x = self
            .ancilla_get_x(k)
            .wrapping_add(2)
            .wrapping_add((rand >> 5) as u16);
        let dst_y = self
            .ancilla_get_y(k)
            .wrapping_sub(2)
            .wrapping_sub(z as u16)
            .wrapping_add((rand & 0xf) as u16);
        self.ancilla_set_xy(j, dst_x, dst_y);
    }

    pub(super) fn ancilla_add_silver_arrow_sparkle(&mut self, kin: usize) {
        if let Some(k) = self.ancilla_alloc_high() {
            {
                let mut sparkle = self.ancilla_slot_view_mut(k);
                sparkle.set_ancilla_type(0x3c);
                sparkle.set_item_to_link(0);
                sparkle.set_timer(4);
            }
            let value = self.game_state.player.follower_link.lower_level_state();
            self.ancilla_slot_view_mut(k).set_floor(value);
            let m = self.get_random_number();
            let j = (self.ancilla_slot_view(kin).direction() & 3) as usize;
            self.ancilla_set_xy(
                k,
                self.ancilla_get_x(kin)
                    .wrapping_add(
                        ANCILLA_ADD_SILVER_ARROW_SPARKLE_SILVER_ARROW_SPARKLE_X[j] as i16 as u16,
                    )
                    .wrapping_add(((m >> 4) & 7) as u16),
                self.ancilla_get_y(kin)
                    .wrapping_add(
                        ANCILLA_ADD_SILVER_ARROW_SPARKLE_SILVER_ARROW_SPARKLE_Y[j] as i16 as u16,
                    )
                    .wrapping_add((m & 7) as u16),
            );
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_ice_rod_shot(&mut self, a: u8, y: u8) {
        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            self.refund_magic(0);
            return;
        };
        self.set_sound_effect_1_with_link_pan(15);
        let value = 1;
        self.ancilla_slot_view_mut(k).set_l(value);
        let j = self.game_state.player.follower_link.facing_index();
        {
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_step(0);
            ancilla.set_work_byte_25(0);
            ancilla.set_item_to_link(255);
            ancilla.set_aux_timer(3);
            ancilla.set_work_byte_3(6);
            ancilla.set_direction(j as u8);
            ancilla.set_y_velocity(ANCILLA_ADD_ICE_ROD_SHOT_ICE_ROD_YVEL[j] as u8);
            ancilla.set_x_velocity(ANCILLA_ADD_ICE_ROD_SHOT_ICE_ROD_XVEL[j] as u8);
        }

        if self.ancilla_check_initial_tile_a(k) < 0 {
            let x = self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_add(ANCILLA_ADD_ICE_ROD_SHOT_ICE_ROD_X[j] as i16 as u16);
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_add(ANCILLA_ADD_ICE_ROD_SHOT_ICE_ROD_Y[j] as i16 as u16);

            if (x.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2())
                | y.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()))
                & 0xff00
                != 0
            {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                return;
            }
            self.ancilla_set_xy(k, x, y);
        } else {
            self.ancilla_slot_view_mut(k).set_ancilla_type(0x11);
            let value = ANCILLA_DRAW_SPRITE_COUNTS[0x11];
            self.ancilla_slot_view_mut(k).set_num_sprites(value);
            let mut ancilla = self.ancilla_slot_view_mut(k);
            ancilla.set_item_to_link(0);
            ancilla.set_aux_timer(4);
        }
    }

    #[track_caller]
    pub(in crate::zelda_rtl) fn ancilla_add_splash(&mut self, a: u8, y: u8) -> bool {
        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            return true;
        };
        self.set_sound_effect_1_with_link_pan(0x24);
        {
            let mut splash = self.ancilla_slot_view_mut(k);
            splash.set_item_to_link(0);
            splash.set_aux_timer(2);
        }
        if self.game_state.world.location.is_indoors()
            && !self.game_state.player.follower_link.is_in_deep_water()
        {
            self.follower_link_state_mut().set_lower_level_state(0);
        }
        self.ancilla_set_xy(
            k,
            self.game_state.player.follower_link.x().wrapping_sub(11),
            self.game_state.player.follower_link.y().wrapping_add(8),
        );
        false
    }

    pub(in crate::zelda_rtl) fn ancilla_add_grave_stone(&mut self, ain: u8, yin: u8) {
        let Some(k) = self.ancilla_add_ancilla(ain, yin) else {
            return;
        };
        let link_y = self.game_state.player.follower_link.y();
        let t = if link_y & 0x0f < 7 {
            link_y
        } else {
            link_y.wrapping_add(16)
        } & !0x0f;

        let mut i = 7usize;
        while ANCILLA_ADD_GRAVE_STONE_MOVE_GRAVESTONE_Y[i] != t {
            if i == 0 {
                self.ancilla_slot_view_mut(k).set_ancilla_type(0);
                return;
            }
            i -= 1;
        }

        let mut j = ANCILLA_ADD_GRAVE_STONE_MOVE_GRAVESTONE_IDX[i] as usize;
        let end = ANCILLA_ADD_GRAVE_STONE_MOVE_GRAVESTONE_IDX[i + 1] as usize;
        loop {
            let x = ANCILLA_ADD_GRAVE_STONE_MOVE_GRAVESTONE_X[j];
            let link_x = self.game_state.player.follower_link.x();
            if x < link_x && x.wrapping_add(15) >= link_x {
                if (j == 13) != self.game_state.player.follower_link.is_running() {
                    break;
                }

                let pos = ANCILLA_ADD_GRAVE_STONE_MOVE_GRAVESTONE_POS[j];
                self.dungeon_object_tracking_mut()
                    .set_big_rock_starting_address(pos);
                self.dungeon_doors_mut()
                    .set_door_open_counter(ANCILLA_ADD_GRAVE_STONE_MOVE_GRAVESTONE_CTR[j] as u16);
                if self.game_state.dungeon.doors.door_open_counter_low() == 0x58 {
                    self.set_sound_effect_2_with_link_pan(0x1b);
                } else if self.game_state.dungeon.doors.door_open_counter_low() == 0x38 {
                    let screen = self.game_state.world.location.overworld_screen_index() as usize;
                    self.set_overworld_event_bits(screen, 0x20);
                    self.set_sound_effect_2_with_link_pan(0x1b);
                }

                let debris = pos.wrapping_sub(0x80);
                self.door_debris_mut()
                    .set_y_low_and_x_low_from_word(k, debris);

                self.overworld_do_map_update32x32_b();

                if self.game_state.system_signals.sound_effect_2() & 0x3f != 0x1b {
                    self.set_sound_effect_1_with_link_pan(0x22);
                }

                let yy = ANCILLA_ADD_GRAVE_STONE_MOVE_GRAVESTONE_Y1[j];
                let xx = ANCILLA_ADD_GRAVE_STONE_MOVE_GRAVESTONE_X1[j];
                self.follower_link_state_mut().set_defense_flags(4);
                self.follower_link_state_mut().set_hookshot_grave_latch();
                let ancilla_a = yy.wrapping_sub(18);
                let value = ancilla_a as u8;
                self.ancilla_slot_view_mut(k).set_a(value);
                let value = (ancilla_a >> 8) as u8;
                self.ancilla_slot_view_mut(k).set_b(value);
                self.ancilla_set_xy(k, xx, yy.wrapping_sub(2));
                return;
            }
            j += 1;
            if j == end {
                break;
            }
        }
        self.ancilla_slot_view_mut(k).set_ancilla_type(0);
    }

    pub(in crate::zelda_rtl) fn ancilla_add_waterfall_splash(&mut self) {
        if self.ancilla_add_check_for_presence(0x41) {
            return;
        }
        if let Some(k) = self.ancilla_add_ancilla(0x41, 4) {
            let mut splash = self.ancilla_slot_view_mut(k);
            splash.set_timer(2);
            splash.set_item_to_link(0);
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_door_debris(&mut self) -> i32 {
        let Some(k) = self.ancilla_add_ancilla(8, 1) else {
            return -1;
        };
        {
            let mut debris = self.ancilla_slot_view_mut(k);
            debris.set_work_byte_25(0);
            debris.set_work_byte_26(7);
        }
        k as i32
    }

    pub(super) fn ancilla_add_occasional_sparkle(&mut self, k: usize) {
        if self.game_state.frame.frame_counter & 7 == 0 {
            self.ancilla_add_sword_charge_sparkle(k);
        }
    }

}

impl ZeldaState {
    /// `ancilla_alloc_rotate` ($03C4). On the hardware this byte is also
    /// `ancilla_arr26[4]` and `ancilla_arr25[2]`; the ancilla slot bank owns
    /// it, so read the RAM byte the bank projects.
    pub(crate) fn ancilla_alloc_rotate(&self) -> u8 {
        self.ram[crate::game_state::constants::ANCILLA_ALLOC_ROTATE]
    }

    pub(crate) fn set_ancilla_alloc_rotate(&mut self, value: u8) {
        self.game_state
            .sprites
            .ancilla_slots
            .set_shared_byte(crate::game_state::constants::ANCILLA_ALLOC_ROTATE, value);
        self.ram[crate::game_state::constants::ANCILLA_ALLOC_ROTATE] = value;
    }

}
