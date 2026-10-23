//! Bomb behaviour, blast walls, explosions, and their drawing.

use super::*;
use crate::types::{abs16, sign8, Point16U, ProjectSpeedRet, SpriteHitBox};

impl ZeldaState {
    fn ancilla_add_blast_wall_fireball(&mut self, _a: u8, _y: u8, r4: usize) {
        for k in (5..=10).rev() {
            if self.ancilla_slot_view(k).ancilla_type() == 0 {
                let floor = self.game_state.player.follower_link.lower_level_state();
                {
                    let mut ancilla = self.ancilla_slot_view_mut(k);
                    ancilla.set_ancilla_type(0x32);
                    ancilla.set_floor(floor);
                }
                self.blast_wall_fireball_mut(k).set_timer(16);
                let j = (self.game_state.frame.frame_counter & 15) as usize;
                let velocity = ANCILLA_ADD_BLAST_WALL_FIREBALL_BLAST_WALL_FIREBALL_VELOCITY[j];
                {
                    let mut ancilla = self.ancilla_slot_view_mut(k);
                    ancilla.set_y_velocity(velocity.y as u8);
                    ancilla.set_x_velocity(velocity.x as u8);
                }
                self.ancilla_set_xy(
                    k,
                    self.game_state
                        .effects
                        .entrance_effects
                        .blast_wall_fragment_slot(r4)
                        .x()
                        .wrapping_add(16),
                    self.game_state
                        .effects
                        .entrance_effects
                        .blast_wall_fragment_slot(r4)
                        .y()
                        .wrapping_add(8),
                );
                return;
            }
        }
    }

    fn bomb_project_speed_towards_player(
        &mut self,
        _k: usize,
        x: u16,
        y: u16,
        vel: u8,
    ) -> ProjectSpeedRet {
        let old_x = self.sprite_get_x(0);
        let old_y = self.sprite_get_y(0);
        let old_z = self.sprite_slot_view(0).z();
        self.sprite_set_x(0, x);
        self.sprite_set_y(0, y);
        self.sprite_slot_view_mut(0).set_z(0);
        let pt = self.sprite_project_speed_towards_link(0, vel);
        self.sprite_slot_view_mut(0).set_z(old_z);
        self.sprite_set_x(0, old_x);
        self.sprite_set_y(0, old_y);
        pt
    }

    fn bomb_check_sprite_damage(&mut self, k: usize) {
        for j in (0..16).rev() {
            if (((j as u8 ^ self.game_state.frame.frame_counter) & 3)
                | self.sprite_slot_view(j).hit_timer()
                | self.sprite_slot_view(j).ignore_projectile())
                != 0
            {
                continue;
            }
            if self.sprite_slot_view(j).floor() != self.ancilla_slot_view(k).floor()
                || self.sprite_slot_view(j).state() < 9
            {
                continue;
            }
            let ax = self.ancilla_get_x(k).wrapping_sub(24);
            let ay = self
                .ancilla_get_y(k)
                .wrapping_sub(24)
                .wrapping_sub(self.ancilla_slot_view(k).z() as u16);
            let mut hb = SpriteHitBox {
                r0_xlo: ax as u8,
                r8_xhi: (ax >> 8) as u8,
                r1_ylo: ay as u8,
                r9_yhi: (ay >> 8) as u8,
                r2: 48,
                r3: 48,
                r4_spr_xlo: 0,
                r10_spr_xhi: 0,
                r5_spr_ylo: 0,
                r11_spr_yhi: 0,
                r6_spr_xsize: 0,
                r7_spr_ysize: 0,
            };
            self.sprite_setup_hit_box(j, &mut hb);
            if !self.check_if_hit_boxes_overlap(&hb) {
                continue;
            }
            if self.sprite_slot_view(j).sprite_type() == 0x92 && self.sprite_slot_view(j).c() >= 3 {
                continue;
            }
            self.ancilla_check_damage_to_sprite(j, self.ancilla_slot_view(k).ancilla_type());
            let pt = self.ancilla_project_reflexive_speed_onto_sprite(
                j,
                self.ancilla_get_x(k),
                self.ancilla_get_y(k),
                64,
            );
            let value = 0u8.wrapping_sub(pt.x);
            self.sprite_slot_view_mut(j).set_x_recoil(value);
            let value = 0u8.wrapping_sub(pt.y);
            self.sprite_slot_view_mut(j).set_y_recoil(value);
        }
    }

    fn bomb_check_sprite_and_player_damage(&mut self, k: usize) {
        let bomb_phase = self.ancilla_slot_view(k).item_to_link();
        if bomb_phase == 0 || bomb_phase >= 9 {
            return;
        }
        self.bomb_check_sprite_damage(k);
        if self
            .game_state
            .player
            .follower_link
            .sprite_damage_disable_timer()
            != 0
        {
            if k + 1 == self.game_state.player.follower_link.ancilla_pickup_flag() as usize
                && self
                    .game_state
                    .player
                    .follower_link
                    .is_lifting_or_carrying()
            {
                self.follower_link_state_mut()
                    .clear_lifting_or_carrying_state();
                self.follower_link_state_mut().clear_direction_lock();
            }
            return;
        }

        if self.game_state.player.follower_link.has_auxiliary_state()
            || self.game_state.player.follower_link.incapacitated_timer() != 0
            || self.ancilla_slot_view(k).floor()
                != self.game_state.player.follower_link.lower_level_state()
        {
            return;
        }

        let link_x = self.game_state.player.follower_link.x();
        let link_y = self.game_state.player.follower_link.y();
        let ax = self.ancilla_get_x(k).wrapping_sub(16);
        let ay = self.ancilla_get_y(k).wrapping_sub(16);
        let hb = SpriteHitBox {
            r0_xlo: link_x as u8,
            r8_xhi: (link_x >> 8) as u8,
            r1_ylo: link_y as u8,
            r9_yhi: (link_y >> 8) as u8,
            r2: 0x10,
            r3: 0x18,
            r4_spr_xlo: ax as u8,
            r10_spr_xhi: (ax >> 8) as u8,
            r5_spr_ylo: ay as u8,
            r11_spr_yhi: (ay >> 8) as u8,
            r6_spr_xsize: 32,
            r7_spr_ysize: 32,
        };

        if !self.check_if_hit_boxes_overlap(&hb) {
            return;
        }

        let x = self.ancilla_get_x(k).wrapping_sub(8);
        let y = self.ancilla_get_y(k).wrapping_sub(12);
        let j = self.bomb_get_displacement_from_link(k) as usize;
        let pt = self.bomb_project_speed_towards_player(
            k,
            x,
            y,
            BOMB_CHECK_SPRITE_AND_PLAYER_DAMAGE_BOMB_DMG_SPEED[j],
        );
        if self.game_state.player.follower_link.blink_countdown() != 0
            || self.game_state.player.follower_link.has_menu_block_flag(2)
        {
            return;
        }
        self.follower_link_state_mut()
            .set_actual_velocity_xy(pt.x, pt.y);
        self.follower_link_state_mut()
            .set_actual_z_velocity_and_copy(BOMB_CHECK_SPRITE_AND_PLAYER_DAMAGE_BOMB_DMG_ZVEL[j]);
        self.follower_link_state_mut()
            .set_incapacitated_timer(BOMB_CHECK_SPRITE_AND_PLAYER_DAMAGE_BOMB_DMG_DELAY[j]);
        self.follower_link_state_mut().set_auxiliary_state(1);
        self.follower_link_state_mut().set_blink_countdown(58);
        if self.game_state.dungeon.savegame_state.savegame_state_bits() & 0x8000 == 0 {
            let armor = self.game_state.inventory.items.armor() as usize;
            self.follower_link_state_mut()
                .set_given_damage(BOMB_CHECK_SPRITE_AND_PLAYER_DAMAGE_BOMB_DMG_TO_LINK[armor]);
        }
    }

    pub(super) fn ancilla07_bomb(&mut self, k: usize) {
        if self.game_state.frame.submodule != 0 {
            if self.game_state.frame.submodule == 8 || self.game_state.frame.submodule == 16 {
                self.ancilla_handle_lift_logic(k);
            } else if k + 1 == self.game_state.player.follower_link.ancilla_pickup_flag() as usize
                && self.ancilla_slot_view(k).k() != 0
            {
                if self.ancilla_slot_view(k).k() != 3 {
                    self.ancilla_latch_link_coordinates(k, 3);
                    self.ancilla_latch_altitude_above_link(k);
                    self.ancilla_slot_view_mut(k).set_k(3);
                }
                self.ancilla_latch_carried_position(k);
            }
            self.bomb_draw(k);
            return;
        }
        self.ancilla_handle_lift_logic(k);

        let mut old_y = self.ancilla_latch_y_coord_to_z(k);
        let s1a = self.ancilla_slot_view(k).direction();
        let s1b = self.ancilla_slot_view(k).object_priority();
        self.ancilla_slot_view_mut(k).set_object_priority(0);
        let mut flag = self.ancilla_check_tile_collision_class2(k);

        if self.game_state.world.location.is_indoors()
            && self.ancilla_slot_view(k).l() != 0
            && self.ancilla_slot_view(k).tile_attribute() == 0x1c
        {
            let value = 1;
            self.ancilla_slot_view_mut(k).set_t_player(value);
        }

        loop {
            if flag
                && (!self
                    .game_state
                    .player
                    .follower_link
                    .is_lifting_or_carrying()
                    || self
                        .game_state
                        .player
                        .follower_link
                        .has_picking_throw_state())
            {
                if s1b == 0 && self.ancilla_slot_view(k).work_byte_4() == 0 {
                    self.ancilla_slot_view_mut(k).set_work_byte_4(1);
                    let qq = if self.ancilla_slot_view(k).direction() == 1 {
                        16
                    } else {
                        4
                    };
                    let y_velocity = self.ancilla_slot_view(k).y_velocity();
                    if y_velocity != 0 {
                        self.ancilla_slot_view_mut(k)
                            .set_y_velocity(if sign8(y_velocity) {
                                qq
                            } else {
                                (-(qq as i8)) as u8
                            });
                    }
                    let x_velocity = self.ancilla_slot_view(k).x_velocity();
                    if x_velocity != 0 {
                        self.ancilla_slot_view_mut(k)
                            .set_x_velocity(if sign8(x_velocity) { 4 } else { (-4i8) as u8 });
                    }
                    if self.ancilla_slot_view(k).direction() == 1
                        && self.ancilla_slot_view(k).z() != 0
                    {
                        self.ancilla_slot_view_mut(k).set_y_velocity((-4i8) as u8);
                        self.ancilla_slot_view_mut(k).set_l(2);
                    }
                }
            } else if !(k + 1
                == self.game_state.player.follower_link.ancilla_pickup_flag() as usize
                && self
                    .game_state
                    .player
                    .follower_link
                    .is_lifting_or_carrying())
                && (self.ancilla_slot_view(k).z() == 0 || self.ancilla_slot_view(k).z() == 0xff)
            {
                self.ancilla_slot_view_mut(k).set_direction(16);
                let bak0 = self.ancilla_slot_view(k).object_priority();
                self.ancilla_check_tile_collision(k);
                self.ancilla_slot_view_mut(k).set_object_priority(bak0);
                let a = self.ancilla_slot_view(k).tile_attribute();
                if a == 0x26 {
                    flag = true;
                    continue;
                } else if a == 0x0c || a == 0x1c {
                    if self.game_state.dungeon.room_load.header_collision() != 3 {
                        if self.ancilla_slot_view(k).floor() == 0
                            && self.ancilla_slot_view(k).z() != 0
                            && self.ancilla_slot_view(k).z() != 0xff
                        {
                            self.ancilla_slot_view_mut(k).set_floor(1);
                        }
                    } else {
                        old_y = self
                            .ancilla_get_y(k)
                            .wrapping_add(self.game_state.dungeon.moving_floor.floor_y_velocity());
                        self.ancilla_set_x(
                            k,
                            self.ancilla_get_x(k).wrapping_add(
                                self.game_state.dungeon.moving_floor.floor_x_velocity(),
                            ),
                        );
                    }
                } else if a == 0x20 || (a & 0xf0) == 0xb0 && a != 0xb6 && a != 0xbc {
                    if !self
                        .game_state
                        .player
                        .follower_link
                        .is_lifting_or_carrying()
                    {
                        if k + 1
                            == self.game_state.player.follower_link.ancilla_pickup_flag() as usize
                        {
                            self.follower_link_state_mut().clear_ancilla_pickup_flag();
                        }
                        if self.ancilla_slot_view(k).timer() == 0 {
                            self.ancilla_slot_view_mut(k).clear();
                            return;
                        }
                    }
                } else if a == 8 {
                    if k + 1 == self.game_state.player.follower_link.ancilla_pickup_flag() as usize
                    {
                        self.follower_link_state_mut().clear_ancilla_pickup_flag();
                    }
                    if self.ancilla_slot_view(k).timer() == 0 {
                        self.ancilla_set_y(k, self.ancilla_get_y(k).wrapping_sub(24));
                        self.ancilla_transmute_to_splash(k);
                        return;
                    }
                } else if matches!(a, 0x68..=0x6b) {
                    self.ancilla_apply_conveyor(k);
                    old_y = self.ancilla_get_y(k);
                } else {
                    let timer = if self.ancilla_slot_view(k).l() != 0 {
                        0
                    } else {
                        2
                    };
                    self.ancilla_slot_view_mut(k).set_timer(timer);
                }
            }
            break;
        }

        self.ancilla_set_y(k, old_y);
        self.ancilla_slot_view_mut(k).set_direction(s1a);
        let object_priority = self.ancilla_slot_view(k).object_priority() | s1b;
        self.ancilla_slot_view_mut(k)
            .set_object_priority(object_priority);
        self.bomb_check_sprite_and_player_damage(k);
        if self.ancilla_slot_view_mut(k).tick_work_byte_3() == 0 {
            let bomb_phase = self.ancilla_slot_view_mut(k).advance_item_to_link();
            if bomb_phase == 1 {
                self.ancilla_sfx2_pan(k, 0x0c);
                if k + 1 == self.game_state.player.follower_link.ancilla_pickup_flag() as usize {
                    self.follower_link_state_mut().clear_ancilla_pickup_flag();
                    if self
                        .game_state
                        .player
                        .follower_link
                        .is_lifting_or_carrying()
                    {
                        self.follower_link_state_mut().clear_state_bits();
                        self.follower_link_state_mut().clear_direction_lock();
                    }
                }
            }

            if bomb_phase == 11 {
                let next_type = if self.ancilla_slot_view(k).step() != 0 {
                    8
                } else {
                    0
                };
                self.ancilla_slot_view_mut(k).set_ancilla_type(next_type);
                return;
            }
            self.ancilla_slot_view_mut(k)
                .set_work_byte_3(BOMB_PHASE_TIMERS[bomb_phase as usize]);
        }

        if self.ancilla_slot_view(k).item_to_link() == 7
            && self.ancilla_slot_view(k).work_byte_3() == 2
        {
            self.door_debris_mut().set_x_word(k, 0);
            self.bomb_check_for_destructibles(
                self.ancilla_get_x(k),
                self.ancilla_get_y(k),
                k as u8,
            );
            if self.door_debris_x_word(k) != 0 {
                self.ancilla_slot_view_mut(k).set_step(1);
            }
        }
        self.bomb_draw(k);
    }

    fn bomb_get_displacement_from_link(&self, k: usize) -> i32 {
        let x = self.ancilla_get_x(k);
        let y = self.ancilla_get_y(k);
        (((abs16(
            self.game_state
                .player
                .follower_link
                .x()
                .wrapping_add(8)
                .wrapping_sub(x),
        ) + abs16(
            self.game_state
                .player
                .follower_link
                .y()
                .wrapping_add(12)
                .wrapping_sub(y),
        )) & 0xfc)
            >> 2) as i32
    }

    pub(in crate::zelda_rtl) fn ancilla_add_blast_wall(&mut self) {
        self.ancilla_slot_view_mut(0).set_ancilla_type(0x33);
        let value = 0x33;
        self.ancilla_slot_view_mut(1).set_ancilla_type(value);
        let value = 0;
        self.ancilla_slot_view_mut(2).set_ancilla_type(value);
        let value = 0;
        self.ancilla_slot_view_mut(3).set_ancilla_type(value);
        let value = 0;
        self.ancilla_slot_view_mut(4).set_ancilla_type(value);
        let value = 0;
        self.ancilla_slot_view_mut(5).set_ancilla_type(value);
        self.ancilla_slot_view_mut(0).set_item_to_link(0);
        self.follower_link_state_mut().clear_ancilla_pickup_flag();
        self.follower_link_state_mut().clear_state_bits();
        self.follower_link_state_mut().clear_direction_lock();
        self.ancilla_slot_view_mut(0).set_k(0);
        let player_floor = self.game_state.player.follower_link.lower_level_state();
        self.ancilla_slot_view_mut(0).set_floor(player_floor);
        let value = player_floor;
        self.ancilla_slot_view_mut(1).set_floor(value);
        let value = self
            .game_state
            .player
            .follower_link
            .lower_level_mirror_state();
        self.ancilla_slot_view_mut(0).set_floor2(value);
        self.blast_wall_scratch_mut().clear_entry_state();
        self.blast_wall_explosion_mut(1).set_timer(0);
        self.blast_wall_explosion_mut(1).set_phase(0);
        self.blast_wall_scratch_mut().clear_secondary_state();
        self.blast_wall_explosion_mut(0).set_phase(1);
        self.set_custom_spell_animation_active();
        self.blast_wall_explosion_mut(0).set_timer(3);

        let mut j = self.blast_wall_direction() as usize;
        let (blast_wall_center_x, blast_wall_center_y) =
            self.blast_wall_scratch_mut().offset_center(
                ANCILLA_ADD_BLAST_WALL_BLAST_WALL_FRAGMENT_Y_OFFSET[j],
                ANCILLA_ADD_BLAST_WALL_BLAST_WALL_FRAGMENT_X_OFFSET[j],
            );
        j = if j < 4 { 4 } else { 0 };
        for k in (0..=3).rev() {
            let offset = ANCILLA_ADD_BLAST_WALL_BLAST_WALL_FRAGMENT_OFFSET[j];
            let y = blast_wall_center_y.wrapping_add(offset.y as i16 as u16);
            let x = blast_wall_center_x.wrapping_add(offset.x as i16 as u16);
            self.blast_wall_fragment_mut(k).set_position(x, y);
            let x = x.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
            if x < 256 {
                self.set_sound_effect_1(BOMBOS_PANNED_SFX_BITS[(x >> 5) as usize] | 0x0c);
            }
            j += 1;
        }
    }

    pub(in crate::zelda_rtl) fn ancilla_add_bomb(&mut self, a: u8, y: u8) {
        let Some(k) = self.ancilla_add_ancilla(a, y) else {
            return;
        };
        if self.game_state.inventory.player_resources.bombs() == 0 {
            self.ancilla_slot_view_mut(k).clear();
            return;
        }

        if self.player_resources_mut().decrement_bombs() == 0 {
            self.hud_refresh_icon();
        }

        let value = 0;

        self.ancilla_slot_view_mut(k).set_r(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_l(value);
        let direction = self.game_state.player.follower_link.facing() >> 1;
        {
            let mut bomb = self.ancilla_slot_view_mut(k);
            bomb.set_step(0);
            bomb.set_item_to_link(0);
            bomb.set_work_byte_3(BOMB_PHASE_TIMERS[0]);
            bomb.set_work_byte_25(0);
            bomb.set_work_byte_26(7);
            bomb.set_z(0);
            bomb.set_timer(8);
            bomb.set_direction(direction);
        }
        let value = 0;
        self.ancilla_slot_view_mut(k).set_t_player(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_work_byte_23(value);
        let value = 0;
        self.ancilla_slot_view_mut(k).set_work_byte_22(value);
        let j = direction as usize;
        if self.ancilla_check_initial_tile_collision_class2(k) {
            self.ancilla_set_xy(
                k,
                self.game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_add(ANCILLA_ADD_BOMB_BOMB_PLACE_X0[j] as i16 as u16),
                self.game_state
                    .player
                    .follower_link
                    .y()
                    .wrapping_add(ANCILLA_ADD_BOMB_BOMB_PLACE_Y0[j] as i16 as u16),
            );
        } else {
            self.ancilla_set_xy(
                k,
                self.game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_add(ANCILLA_ADD_BOMB_BOMB_PLACE_X1[j] as i16 as u16),
                self.game_state
                    .player
                    .follower_link
                    .y()
                    .wrapping_add(ANCILLA_ADD_BOMB_BOMB_PLACE_Y1[j] as i16 as u16),
            );
        }
        self.set_sound_effect_1_with_link_pan(0x0b);
    }

    pub(super) fn ancilla3_a_big_bomb_explosion(&mut self, k: usize) {
        if self.game_state.frame.submodule == 0
            && self.ancilla_slot_view_mut(k).tick_work_byte_3() == 0
        {
            let bomb_phase = self.ancilla_slot_view_mut(k).advance_item_to_link();
            if bomb_phase == 2 {
                self.ancilla_sfx2_pan(k, 0x0c);
            }
            if bomb_phase == 11 {
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
            self.ancilla_slot_view_mut(k)
                .set_work_byte_3(BOMB_PHASE_TIMERS[bomb_phase as usize]);
        }

        self.oam_state_mut().set_priority_word(0x3000);
        let bomb_phase = self.ancilla_slot_view(k).item_to_link() as usize;
        let numframes = BOMB_DRAW_FRAME_COUNTS[bomb_phase] as usize;
        let j = BOMB_DRAW_FRAME_STARTS[bomb_phase] as usize * 6;
        self.ancilla_slot_view_mut(k).set_step((j * 2) as u8);

        let mut yy = 0usize;
        for i in (0..=8).rev() {
            let x = self
                .ancilla_get_x(k)
                .wrapping_add(ANCILLA3_A_BIG_BOMB_EXPLOSION_SUPER_BOMB_EXPLODE_X[i] as i16 as u16)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
            let y = self
                .ancilla_get_y(k)
                .wrapping_add(ANCILLA3_A_BIG_BOMB_EXPLOSION_SUPER_BOMB_EXPLODE_Y[i] as i16 as u16)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
            if x < 256 && y < 256 {
                self.ancilla_allocate_oam_from_region_a_or_d_or_f(j * 2, 0x18);
                let base_oam = self.game_state.oam.current_pointer_usize();
                let oam = base_oam + yy;
                let next_oam = self.ancilla_draw_explosion(oam, j, 0, numframes, 0x32, x, y);
                yy += next_oam - oam;
            }
        }

        if self.ancilla_slot_view(k).item_to_link() == 3
            && self.ancilla_slot_view(k).work_byte_3() == 1
        {
            let old = if self
                .game_state
                .enhanced_features
                .has(ANCILLA3_A_BIG_BOMB_EXPLOSION_FEATURES0_MISC_BUG_FIXES)
            {
                self.game_state.sprites.follower_runtime.indicator()
            } else {
                0
            };
            self.follower_state_mut().set_indicator(13);
            self.bomb_check_for_destructibles(self.ancilla_get_x(k), self.ancilla_get_y(k), 0);
            self.follower_state_mut().set_indicator(old);
        }
    }

    fn bomb_check_underside_sprite_status(&mut self, k: usize, pt: &mut Point16U) -> Option<u8> {
        if self.ancilla_slot_view(k).item_to_link() != 0 {
            return None;
        }

        let mut r10 = 0;
        if self.ancilla_slot_view(k).tile_attribute() == 9 {
            self.ancilla_slot_view_mut(k).subtract_work_byte_22(1);
            if sign8(self.ancilla_slot_view(k).work_byte_22()) {
                let value = 3;
                self.ancilla_slot_view_mut(k).set_work_byte_22(value);
                self.ancilla_slot_view_mut(k).add_work_byte_23(1);
                if self.ancilla_slot_view(k).work_byte_23() == 3 {
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_work_byte_23(value);
                }
            }
            r10 = self.ancilla_slot_view(k).work_byte_23().wrapping_add(4);
            if self.game_state.system_signals.sound_effect_1() & 0x3f == 0x0b
                || self.game_state.system_signals.sound_effect_1() & 0x3f == 0x21
            {
                self.set_sound_effect_1_with_ancilla_pan(k, 0x28);
            }
        } else if self.ancilla_slot_view(k).tile_attribute() == 0x40 {
            r10 = 3;
        }

        if self.ancilla_slot_view(k).z() >= 2 && self.ancilla_slot_view(k).z() < 252 {
            r10 = 2;
        }
        if k + 1 == self.game_state.player.follower_link.ancilla_pickup_flag() as usize
            && self
                .game_state
                .player
                .follower_link
                .is_lifting_or_carrying()
        {
            return None;
        }
        let z = self.ancilla_slot_view(k).z() as i8;
        pt.y = pt.y.wrapping_add(z as i16 as u16).wrapping_add(2);
        pt.x = pt.x.wrapping_sub(8);
        Some(r10)
    }

    fn ancilla_draw_explosion(
        &mut self,
        mut oam: usize,
        mut frame: usize,
        mut idx: usize,
        idx_end: usize,
        r11: u8,
        x: u16,
        y: u16,
    ) -> usize {
        let base_frame = frame;
        loop {
            let tile = ANCILLA_DRAW_EXPLOSION_BOMB_DRAW_EXPLOSION_TILE[frame];
            if tile.char != 0xff {
                let i = idx + base_frame;
                let offset = ANCILLA_DRAW_EXPLOSION_BOMB_DRAW_EXPLOSION_OFFSET[i];
                self.ancilla_set_oam_safe(
                    oam,
                    x.wrapping_add(offset.x as i16 as u16),
                    y.wrapping_add(offset.y as i16 as u16),
                    tile.char,
                    tile.flags & !0x3e | self.game_state.oam.priority_high() | r11,
                    ANCILLA_DRAW_EXPLOSION_BOMB_DRAW_EXPLOSION_EXT[frame],
                );
                oam += 4;
            }
            frame += 1;
            idx += 1;
            if idx == idx_end {
                break;
            }
        }
        oam
    }

    fn bomb_draw(&mut self, k: usize) {
        let (x, mut y) = self.ancilla_prep_adjusted_oam_coord(k);
        let z = self.ancilla_slot_view(k).z() as i8;
        if z != 0
            && z != -1
            && self.ancilla_slot_view(k).k() != 3
            && self.ancilla_slot_view(k).object_priority() != 0
        {
            self.oam_state_mut().set_priority_word(0x3000);
        }
        y = y.wrapping_sub(z as i16 as u16);
        let j =
            BOMB_DRAW_FRAME_STARTS[self.ancilla_slot_view(k).item_to_link() as usize] as usize * 6;

        let mut r11 = 2;
        if self.ancilla_slot_view(k).item_to_link() == 0 {
            r11 = if self.ancilla_slot_view(k).work_byte_3() < 0x20 {
                self.ancilla_slot_view(k).work_byte_3() & 0x0e
            } else {
                4
            };
        }

        if self.ancilla_slot_view(k).item_to_link() == 0 {
            if self.ancilla_slot_view(k).l() == 0
                && (self.sprite_slot_view(0).sprite_type() == 0x92
                    || k + 1 == self.game_state.player.follower_link.ancilla_pickup_flag() as usize)
                && (!self
                    .game_state
                    .player
                    .follower_link
                    .is_lifting_or_carrying()
                    || self.ancilla_slot_view(k).k() != 3
                        && self.game_state.player.follower_link.facing() == 0)
            {
                self.ancilla_allocate_oam_from_region_b_or_e(12);
            } else if self.game_state.oam.has_sprite_sorting()
                && self.ancilla_slot_view(k).floor() != 0
                && (self.ancilla_slot_view(k).l() != 0
                    || k + 1 == self.game_state.player.follower_link.ancilla_pickup_flag() as usize
                        && self
                            .game_state
                            .player
                            .follower_link
                            .is_lifting_or_carrying())
            {
                self.oam_state_mut().set_current_pointer(0x0800 + 0x34 * 4);
                self.oam_state_mut()
                    .set_current_extended_pointer(0x0a20 + 0x34);
            }
        }

        let oam_org = self.game_state.oam.current_pointer_usize();
        let numframes =
            BOMB_DRAW_FRAME_COUNTS[self.ancilla_slot_view(k).item_to_link() as usize] as usize;
        let mut oam = oam_org;
        if self.ancilla_slot_view(k).item_to_link() == 0
            && (self.ancilla_slot_view(k).tile_attribute() == 9
                || self.ancilla_slot_view(k).tile_attribute() == 0x40)
        {
            oam += 8;
        }

        self.ancilla_draw_explosion(oam, j, 0, numframes, r11, x, y);
        oam += numframes * 4;

        let mut pt = Point16U { x, y };
        if let Some(r10) = self.bomb_check_underside_sprite_status(k, &mut pt) {
            if oam != oam_org + 4 {
                oam = oam_org;
            }
            self.ancilla_draw_shadow(
                oam,
                r10 as usize,
                pt.x,
                pt.y,
                self.game_state.oam.priority_high(),
            );
        }
    }

    pub(super) fn ancilla32_blast_wall_fireball(&mut self, k: usize) {
        if self.game_state.frame.submodule == 0 {
            self.ancilla_slot_view_mut(k).add_item_to_link(2);
            let value = self
                .ancilla_slot_view(k)
                .y_velocity()
                .wrapping_add(self.ancilla_slot_view(k).item_to_link());
            self.ancilla_slot_view_mut(k).set_y_velocity(value);
            self.ancilla_move_y(k);
            self.ancilla_move_x(k);
            let timer = self.blast_wall_fireball_mut(k).tick_timer();
            if sign8(timer) {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                return;
            }
        }

        if self.game_state.oam.has_sprite_sorting() {
            self.oam_allocate_from_region_d(4);
        } else {
            self.oam_allocate_from_region_a(4);
        }

        let (x, y) = self.ancilla_prep_oam_coord(k);
        let timer = self
            .game_state
            .effects
            .entrance_effects
            .blast_wall_fireball_slot(k)
            .timer();
        let j = if timer & 8 != 0 {
            0
        } else if timer & 4 != 0 {
            1
        } else {
            2
        };
        self.ancilla_set_oam(
            self.game_state.oam.current_pointer_usize(),
            x,
            y,
            ANCILLA32_BLAST_WALL_FIREBALL_BLAST_WALL_FIREBALL_CHAR[j],
            0x22,
            0,
        );
    }

    pub(super) fn ancilla33_blast_wall_explosion(&mut self, k: usize) {
        if self.game_state.frame.submodule == 0 {
            if self
                .game_state
                .effects
                .entrance_effects
                .blast_wall_explosion_slot(k)
                .phase()
                != 0
            {
                let timer = self.blast_wall_explosion_mut(k).tick_timer();
                if timer == 0 {
                    let phase = self.blast_wall_explosion_mut(k).advance_phase();
                    if phase != 0 && phase < 9 {
                        self.ancilla_add_blast_wall_fireball(0x32, 10, k * 4);
                    }
                    if phase == 11 {
                        self.blast_wall_explosion_mut(k).set_phase(0);
                        self.blast_wall_explosion_mut(k).set_timer(0);
                    } else {
                        self.blast_wall_explosion_mut(k).set_timer(3);
                    }
                }
            } else {
                let k = k ^ 1;
                if self
                    .game_state
                    .effects
                    .entrance_effects
                    .blast_wall_explosion_slot(k)
                    .phase()
                    == 6
                    && self
                        .game_state
                        .effects
                        .entrance_effects
                        .blast_wall_explosion_slot(k)
                        .timer()
                        == 2
                    && self.ancilla_slot_view(0).item_to_link().wrapping_add(1) < 7
                {
                    self.ancilla_slot_view_mut(0).advance_item_to_link();
                    self.blast_wall_explosion_mut(k).set_phase(1);
                    self.blast_wall_explosion_mut(k).set_timer(3);
                    for i in (0..=3).rev() {
                        let mut arr = [0i8, 0i8];
                        let j = if self.blast_wall_direction() < 4 {
                            1
                        } else {
                            0
                        };
                        arr[j] = if i & 2 != 0 { -13 } else { 13 };
                        let j = k * 4 + i;
                        let (x, _y) = self
                            .blast_wall_fragment_mut(j)
                            .offset(arr[1] as i16, arr[0] as i16);
                        let x =
                            x.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
                        if x < 256 {
                            self.set_sound_effect_1(
                                BOMBOS_PANNED_SFX_BITS[(x >> 5) as usize] | 0x0c,
                            );
                        }
                    }
                }
            }
        }

        let k = self.ancilla_slot_view(0).k() as usize;
        if self
            .game_state
            .effects
            .entrance_effects
            .blast_wall_explosion_slot(k)
            .phase()
            != 0
        {
            let first_i = if k == 1 { 7 } else { 3 };
            for i in (first_i - 3..=first_i).rev() {
                self.ancilla_draw_blast_wall_blast(
                    k,
                    self.game_state
                        .effects
                        .entrance_effects
                        .blast_wall_fragment_slot(i)
                        .x(),
                    self.game_state
                        .effects
                        .entrance_effects
                        .blast_wall_fragment_slot(i)
                        .y(),
                );
            }
        }
        if self.ancilla_slot_view(0).item_to_link() == 6
            && self
                .game_state
                .effects
                .entrance_effects
                .blast_wall_explosion_slot(0)
                .phase()
                == 0
            && self
                .game_state
                .effects
                .entrance_effects
                .blast_wall_explosion_slot(1)
                .phase()
                == 0
        {
            self.ancilla_slot_view_mut(0).set_ancilla_type(0);
            let value = 0;
            self.ancilla_slot_view_mut(1).set_ancilla_type(value);
            self.clear_custom_spell_animation();
        }
    }

    fn ancilla_draw_blast_wall_blast(&mut self, k: usize, x: u16, y: u16) {
        self.oam_state_mut().set_priority_word(0x3000);
        if self.game_state.oam.has_sprite_sorting() {
            self.oam_allocate_from_region_d(0x18);
        } else {
            self.oam_allocate_from_region_a(0x18);
        }
        let oam = self.game_state.oam.current_pointer_usize();
        let i = self
            .game_state
            .effects
            .entrance_effects
            .blast_wall_explosion_slot(k)
            .phase() as usize;
        self.ancilla_draw_explosion(
            oam,
            BOMB_DRAW_FRAME_STARTS[i] as usize * 6,
            0,
            BOMB_DRAW_FRAME_COUNTS[i] as usize,
            0x32,
            x.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()),
            y.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()),
        );
    }

}
