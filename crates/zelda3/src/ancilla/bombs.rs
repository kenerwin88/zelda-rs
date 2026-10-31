//! Bomb behaviour, blast walls, explosions, and their drawing.

use crate::zelda_rtl::sprite::SpriteHitBoxEntry;
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
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x088287));
        // $08:8287..832A: all sixteen slot tests and actual damage paths.
        // Hitbox, overlap, damage and recoil children retain their own work.
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x088287);
        for j in (0..16).rev() {
            b.block(0x088289);
            'slot: {
            if (((j as u8 ^ self.game_state.frame.frame_counter) & 3)
                | self.sprite_slot_view(j).hit_timer()
                | self.sprite_slot_view(j).ignore_projectile())
                != 0
            {
                b.block(0x088296); break 'slot;
            }
            b.taken(0x088294); b.block(0x088299);
            if self.sprite_slot_view(j).floor() != self.ancilla_slot_view(k).floor() {
                b.taken(0x08829f); b.block(0x088296); break 'slot;
            }
            b.block(0x0882a1);
            if self.sprite_slot_view(j).state() < 9 {
                b.taken(0x0882a6); break 'slot;
            }
            b.block(0x0882a8);
            if k >= 6 { b.crossing(0x0882b7); }
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
            self.sprite_setup_hit_box(j, &mut hb, SpriteHitBoxEntry::Unpriced);
            b.block(0x0882da);
            let overlaps = self.check_if_hit_boxes_overlap(&hb);
            b.block(0x0882df);
            if !overlaps {
                b.taken(0x0882df); break 'slot;
            }
            b.block(0x0882e1);
            if self.sprite_slot_view(j).sprite_type() == 0x92 {
                b.block(0x0882e8);
                if self.sprite_slot_view(j).c() >= 3 { b.taken(0x0882ed); break 'slot; }
            } else { b.taken(0x0882e6); }
            b.block(0x0882ef);
            if k >= 6 { b.crossing(0x0882f9); }
            self.ancilla_check_damage_to_sprite(j, self.ancilla_slot_view(k).ancilla_type());
            b.block(0x08830d);
            let pt = self.ancilla_project_reflexive_speed_onto_sprite(
                j,
                self.ancilla_get_x(k),
                self.ancilla_get_y(k),
                64,
            );
            let value = 0u8.wrapping_sub(pt.x);
            b.block(0x088312);
            self.sprite_slot_view_mut(j).set_x_recoil(value);
            let value = 0u8.wrapping_sub(pt.y);
            self.sprite_slot_view_mut(j).set_y_recoil(value);
            }
            b.block(0x088324);
            if j == 0 { b.taken(0x088325); } else { b.block(0x088327); }
        }
        b.block(0x08832a);
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
        // $08:95A1 JSR $08:8BCF; full collision body, call opcode separate.
        let mut flag = self.ancilla_check_tile_collision_class2(k, AncillaClass2Entry::Rom088BCF);

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
        // $08:F18D..F27B: timer and nine explosion callers. A7AB's
        // existing writer, allocation, sound and destructible children stay separate.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x08f18d));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x08f18d);
        if self.game_state.frame.submodule != 0 { b.taken(0x08f18f); }
        else { b.block(0x08f191); }
        if self.game_state.frame.submodule == 0
            && self.ancilla_slot_view_mut(k).tick_work_byte_3() == 0
        {
            b.block(0x08f199);
            let bomb_phase = self.ancilla_slot_view_mut(k).advance_item_to_link();
            if bomb_phase == 2 {
                b.block(0x08f1a3);
                self.ancilla_sfx2_pan(k, 0x0c);
            } else { b.taken(0x08f1a1); }
            b.block(0x08f1a8);
            if bomb_phase == 11 {
                b.block(0x08f1af);
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
            b.taken(0x08f1ad); b.block(0x08f1b3);
            self.ancilla_slot_view_mut(k)
                .set_work_byte_3(BOMB_PHASE_TIMERS[bomb_phase as usize]);
        } else if self.game_state.frame.submodule == 0 { b.taken(0x08f197); }

        b.block(0x08f1ba);
        self.oam_state_mut().set_priority_word(0x3000);
        let bomb_phase = self.ancilla_slot_view(k).item_to_link() as usize;
        let numframes = BOMB_DRAW_FRAME_COUNTS[bomb_phase] as usize;
        let j = BOMB_DRAW_FRAME_STARTS[bomb_phase] as usize * 6;
        self.ancilla_slot_view_mut(k).set_step((j * 2) as u8);

        let mut yy = 0usize;
        for i in (0..=8).rev() {
            b.block(0x08f1e7);
            if 0xfa + k > 0xff { b.crossing(0x08f1e9); }
            let x = self
                .ancilla_get_x(k)
                .wrapping_add(ANCILLA3_A_BIG_BOMB_EXPLOSION_SUPER_BOMB_EXPLODE_X[i] as i16 as u16)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
            let y = self
                .ancilla_get_y(k)
                .wrapping_add(ANCILLA3_A_BIG_BOMB_EXPLOSION_SUPER_BOMB_EXPLODE_Y[i] as i16 as u16)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
            if y >= 256 { b.taken(0x08f222); }
            else {
                b.block(0x08f224);
                if x >= 256 { b.taken(0x08f226); }
            }
            if x < 256 && y < 256 {
                b.block(0x08f228);
                self.ancilla_allocate_oam_from_region_a_or_d_or_f(j * 2, 0x18);
                b.block(0x08f22f);
                let base_oam = self.game_state.oam.current_pointer_usize();
                let oam = base_oam + yy;
                let next_oam = self.ancilla_draw_explosion(oam, j, 0, numframes, 0x32, x, y);
                yy += next_oam - oam;
            }
            b.block(0x08f248);
            if i != 0 { b.taken(0x08f24b); }
        }

        b.block(0x08f24d);
        if self.ancilla_slot_view(k).item_to_link() != 3 { b.taken(0x08f252); }
        else {
            b.block(0x08f254);
            if self.ancilla_slot_view(k).work_byte_3() != 1 { b.taken(0x08f259); }
        }
        if self.ancilla_slot_view(k).item_to_link() == 3
            && self.ancilla_slot_view(k).work_byte_3() == 1
        {
            b.block(0x08f25b);
            if 0xfa + k > 0xff { b.crossing(0x08f25b); }
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
            b.block(0x08f274);
            self.follower_state_mut().set_indicator(old);
        }
        b.block(0x08f27b);
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
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x08_a7ab));
        let base_frame = frame;
        loop {
            let tile = ANCILLA_DRAW_EXPLOSION_BOMB_DRAW_EXPLOSION_TILE[frame];
            // $08:A7AB..A7B0: tile read, sentinel comparison and BEQ.
            if self.lag_free { crate::cycle_ledger::charge(64); }
            if tile.char != 0xff {
                let i = idx + base_frame;
                if self.lag_free {
                    // $08:A7B2..A800, including JSR but excluding $08:F702.
                    // $9D7C,X/$9D7E,X read the two offset words at X=i*4.
                    let offset = (i * 4) as u8;
                    let crossings = u64::from(u16::from(offset) + 0x7c > 0xff)
                        + u64::from(u16::from(offset) + 0x7e > 0xff);
                    crate::cycle_ledger::charge(1100 + crossings * 6);
                }
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
            } else if self.lag_free {
                crate::cycle_ledger::charge(6); // $A7B0: taken sentinel BEQ.
            }
            frame += 1;
            idx += 1;
            // $08:A802..A80A: INX twice, INC, LDA, CMP and actual BNE.
            if self.lag_free { crate::cycle_ledger::charge(130 + if idx != idx_end { 6 } else { 0 }); }
            if idx == idx_end {
                break;
            }
        }
        if self.lag_free { crate::cycle_ledger::charge(42); } // $08:A80C RTS.
        oam
    }

    fn bomb_draw(&mut self, k: usize) {
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x089e9e));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        // $08:9E9E..9FB5: Bomb_Draw self work; called bodies own their work.
        b.block(0x089e9e);
        let (x, mut y) = self.ancilla_prep_adjusted_oam_coord(k);
        b.block(0x089ea1);
        let z = self.ancilla_slot_view(k).z() as i8;
        if z >= 0 { b.taken(0x089eac); } else { b.block(0x089eae); }
        b.block(0x089eb1);
        // STA $04 preserves CMP #$0080 / ORA #$FF00 flags: the BEQ at
        // $9EB3 is not a test of the stored zero altitude and is never taken.
        {
            b.block(0x089eb5);
            if z == -1 { b.taken(0x089eb8); } else {
                b.block(0x089eba);
                if self.ancilla_slot_view(k).k() == 3 { b.taken(0x089ec3); } else {
                    b.block(0x089ec5);
                    if self.ancilla_slot_view(k).object_priority() == 0 {
                        b.taken(0x089ecb);
                    } else { b.block(0x089ecd); }
                }
            }
        }
        if z != 0
            && z != -1
            && self.ancilla_slot_view(k).k() != 3
            && self.ancilla_slot_view(k).object_priority() != 0
        {
            self.oam_state_mut().set_priority_word(0x3000);
        }
        y = y.wrapping_sub(z as i16 as u16);
        b.block(0x089ed2);
        let j =
            BOMB_DRAW_FRAME_STARTS[self.ancilla_slot_view(k).item_to_link() as usize] as usize * 6;

        let mut r11 = 2;
        if self.ancilla_slot_view(k).item_to_link() == 0 {
            b.block(0x089eff);
            if self.ancilla_slot_view(k).work_byte_3() >= 0x20 {
                b.taken(0x089f08);
            } else { b.block(0x089f0a); }
            r11 = if self.ancilla_slot_view(k).work_byte_3() < 0x20 {
                self.ancilla_slot_view(k).work_byte_3() & 0x0e
            } else {
                4
            };
        } else { b.taken(0x089efd); }

        b.block(0x089f0e);
        if self.ancilla_slot_view(k).item_to_link() == 0 {
            // $08:9F17..9F72: source short-circuit allocation branches.
            b.block(0x089f17);
            let mut region_b = false;
            if self.ancilla_slot_view(k).l() != 0 { b.taken(0x089f1a); } else {
                b.block(0x089f1c);
                let eligible = if self.sprite_slot_view(0).sprite_type() == 0x92 {
                    b.taken(0x089f21); true
                } else {
                    b.block(0x089f23);
                    if k + 1 != self.game_state.player.follower_link.ancilla_pickup_flag() as usize {
                        b.taken(0x089f28); false
                    } else { true }
                };
                if eligible {
                    b.block(0x089f2a);
                    if !self.game_state.player.follower_link.is_lifting_or_carrying() {
                        b.taken(0x089f2f); region_b = true;
                    } else {
                        b.block(0x089f31);
                        if self.ancilla_slot_view(k).k() == 3 { b.taken(0x089f36); } else {
                            b.block(0x089f38);
                            if self.game_state.player.follower_link.facing() != 0 {
                                b.taken(0x089f3a);
                            } else { region_b = true; }
                        }
                    }
                }
            }
            if region_b { b.block(0x089f3c); } else {
                b.block(0x089f43);
                if !self.game_state.oam.has_sprite_sorting() { b.taken(0x089f46); } else {
                    b.block(0x089f48);
                    if self.ancilla_slot_view(k).floor() == 0 { b.taken(0x089f4b); } else {
                        b.block(0x089f4d);
                        if self.ancilla_slot_view(k).l() != 0 {
                            b.taken(0x089f50); b.block(0x089f5e);
                        } else {
                            b.block(0x089f52);
                            if k + 1 != self.game_state.player.follower_link.ancilla_pickup_flag() as usize {
                                b.taken(0x089f57);
                            } else {
                                b.block(0x089f59);
                                if !self.game_state.player.follower_link.is_lifting_or_carrying() {
                                    b.taken(0x089f5c);
                                } else { b.block(0x089f5e); }
                            }
                        }
                    }
                }
            }
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
                b.block(0x089f41);
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
        } else { b.taken(0x089f15); }

        b.block(0x089f74);
        let oam_org = self.game_state.oam.current_pointer_usize();
        let numframes =
            BOMB_DRAW_FRAME_COUNTS[self.ancilla_slot_view(k).item_to_link() as usize] as usize;
        let mut oam = oam_org;
        let mut offset = false;
        if self.ancilla_slot_view(k).item_to_link() != 0 { b.taken(0x089f7d); } else {
            b.block(0x089f7f);
            if self.ancilla_slot_view(k).tile_attribute() == 9 {
                b.taken(0x089f84); offset = true;
            } else {
                b.block(0x089f86);
                if self.ancilla_slot_view(k).tile_attribute() != 0x40 {
                    b.taken(0x089f88);
                } else { offset = true; }
            }
        }
        b.block(if offset { 0x089f8a } else { 0x089f8e });
        if self.ancilla_slot_view(k).item_to_link() == 0
            && (self.ancilla_slot_view(k).tile_attribute() == 9
                || self.ancilla_slot_view(k).tile_attribute() == 0x40)
        {
            oam += 8;
        }

        b.block(0x089f90);
        self.ancilla_draw_explosion(oam, j, 0, numframes, r11, x, y);
        b.block(0x089fa6);
        oam += numframes * 4;

        let mut pt = Point16U { x, y };
        let underside = self.bomb_check_underside_sprite_status(k, &mut pt);
        b.block(0x089fab);
        if let Some(r10) = underside {
            b.block(0x089fad);
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
            b.block(0x089fb2);
        } else { b.taken(0x089fab); }
        b.block(0x089fb5);
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
        // $08:AA8F JSR F6E1: complete XY body, once per executed draw.
        self.ancilla_set_oam_marked(
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

#[cfg(test)]
mod price_tests {
    use super::*;

    #[test]
    fn bomb_damage_markers_match_rom_paths() {
        let rom = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc")).unwrap();
        let mut cases = 0;
        for slot in [0, 5, 6, 9] {
            for sprite in 0..16 {
                // Each rejection, both special-sprite branches, and damage.
                for path in 0..10 {
                    for z in [0u8, 16, 255] {
                        let mut native = ZeldaState::new();
                        native.set_lag_free(true);
                        native.write_wram_byte(0x1a, if path == 0 { (sprite ^ 1) as u8 } else { sprite as u8 });
                        native.write_wram_byte(0xef0 + sprite, if path == 1 { 1 } else { 0 });
                        native.write_wram_byte(0xba0 + sprite, if path == 2 { 1 } else { 0 });
                        native.write_wram_byte(0xf20 + sprite, if path == 3 { 1 } else { 0 });
                        native.write_wram_byte(0xdd0 + sprite, if path == 4 { 8 } else { 9 });
                        native.write_wram_byte(0xe20 + sprite, if path == 7 || path == 8 { 0x92 } else { 0x18 });
                        native.write_wram_byte(0xdb0 + sprite, if path == 8 { 3 } else { 2 });
                        native.write_wram_byte(0xe50 + sprite, 16);
                        native.write_wram_byte(0xc4a + slot, 7);
                        native.write_wram_byte(0x29e + slot, z);
                        native.ancilla_set_xy(slot, 0x140, 0x140);
                        let sx = if path == 5 { 0x240u16 } else { 0x140 };
                        let sy = if path == 6 { 0x240u16 } else { 0x140u16.wrapping_sub(z as u16) };
                        for (a, v) in [(0xd10 + sprite, sx as u8), (0xd30 + sprite, (sx >> 8) as u8),
                            (0xd00 + sprite, sy as u8), (0xd20 + sprite, (sy >> 8) as u8)] {
                            native.write_wram_byte(a, v);
                        }
                        native.sync_native_game_state_from_ram();
                        let mut machine = snes::Snes::new();
                        machine.cart.load(snes::CartType::LoRom, &rom, 0x2000);
                        machine.ram.copy_from_slice(&native.wram_image());
                        machine.ram[0x1fd..0x1ff].copy_from_slice(&0x7fffu16.to_le_bytes());
                        machine.cpu.e = false; machine.cpu.unpack_flags(0x30);
                        machine.cpu.k = 8; machine.cpu.db = 8; machine.cpu.pc = 0x8287;
                        machine.cpu.x = slot as u16; machine.cpu.sp = 0x1fc;
                        let source = crate::cycle_ledger::assert_rom_marker_work(0x088287, machine,
                            |s| s.cpu.k == 8 && (0x8287..0x832b).contains(&s.cpu.pc),
                            || native.bomb_check_sprite_damage(slot));
                        for a in [0xf30 + sprite, 0xf40 + sprite, 0xef0 + sprite] {
                            assert_eq!(native.read_wram_byte(a), source.snes().ram[a],
                                "slot={slot} sprite={sprite} path={path} z={z} address={a:04x}");
                        }
                        native.set_lag_free(false);
                        assert_eq!(crate::cycle_ledger::measure_routine_self_work(0x088287,
                            || native.bomb_check_sprite_damage(slot)), 0);
                        cases += 1;
                    }
                }
            }
        }
        eprintln!("bomb damage source cases={cases}");
    }

    #[test]
    fn bomb_draw_markers_match_rom_branches_and_child_exclusion() {
        for slot in 0..10 {
            for variant in 0..192usize {
                let mut n = ZeldaState::new();
                n.set_lag_free(true);
                let mut ram = n.wram_image().into_owned();
                ram[0xfa0] = slot as u8;
                ram[0xc4a + slot] = 7;
                ram[0xc5e + slot] = (variant % 11) as u8;
                ram[0x29e + slot] = [0, 1, 127, 128, 255][variant % 5];
                ram[0x380 + slot] = ((variant / 3) % 4) as u8;
                ram[0x385 + slot] = ((variant / 5) % 2) as u8;
                ram[0x280 + slot] = ((variant / 7) % 2) as u8;
                ram[0x39f + slot] = [0, 14, 31, 32, 255][variant % 5];
                ram[0x3e4 + slot] = [0, 9, 0x40][variant % 3];
                ram[0xc7c + slot] = ((variant / 2) % 2) as u8;
                ram[0xfb3] = ((variant / 4) % 2) as u8;
                ram[0x2ec] = if variant & 1 != 0 { slot as u8 + 1 } else { 0 };
                ram[0x308] = if variant & 2 != 0 { 0x80 } else { 0 };
                ram[0x2f] = ((variant / 8) % 4 * 2) as u8;
                ram[0xe20] = if variant & 4 != 0 { 0x92 } else { 0 };
                ram[0xc04 + slot] = 80;
                ram[0xbfa + slot] = 96;
                ram[0x90..0x92].copy_from_slice(&0x820u16.to_le_bytes());
                ram[0x92..0x94].copy_from_slice(&0xa28u16.to_le_bytes());
                n.ram.copy_from_slice(&ram);
                n.sync_native_game_state_from_ram();
                let machine = super::super::tests::marker29_machine(&n, 0x089e9e, slot).expect("ROM");
                crate::cycle_ledger::assert_rom_marker_work(0x089e9e, machine,
                    |s| s.cpu.k == 8 && (0x9e9e..0x9fb6).contains(&s.cpu.pc),
                    || n.bomb_draw(slot));
                n.set_lag_free(false);
                assert_eq!(crate::cycle_ledger::measure_routine_self_work(0x089e9e,
                    || n.bomb_draw(slot)), 0);
            }
        }
    }

    #[test]
    fn explosion_price_matches_executed_rom_paths() {
        use snes::{CartType, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline,
            CpuRasterPosition, RomCpuTimingProbe, Snes, SourcePpuReadState};
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let rom = std::fs::read(root.join("saves/zelda3.sfc")).unwrap();
        for frame in [0usize, 6, 12, 18, 24, 30, 36, 42, 48] {
            for count in [1usize, 4, 6] {
                for (x, y) in [(0u16, 0u16), (127, 239), (128, 240), (255, 255),
                    (256, 256), (65535, 65535)] {
                    let mut native = ZeldaState::new();
                    native.set_lag_free(true);
                    let mut machine = Snes::new();
                    machine.cart.load(CartType::LoRom, &rom, 0x2000);
                    machine.ram[0x1fd..0x1ff].copy_from_slice(&0x7fffu16.to_le_bytes());
                    for (address, value) in [(0x90, 0x800u16), (0x92, 0xa20),
                        (4, (frame * 4) as u16), (0xc, y), (0xe, x)] {
                        machine.ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
                    }
                    machine.ram[8] = count as u8;
                    machine.ram[0xb] = 0x32;
                    machine.ram[0x65] = 0x30;
                    machine.cpu.e = false;
                    machine.cpu.unpack_flags(0x30);
                    machine.cpu.k = 8;
                    machine.cpu.db = 8;
                    machine.cpu.pc = 0xa7ab;
                    machine.cpu.sp = 0x1fc;
                    machine.cpu.x = (frame * 2) as u16;
                    let clock = CpuMasterTimeline::at_raster(0, CpuRasterPosition::new(0, 0),
                        CpuBusWorkload::default(), CpuFieldTiming::non_interlace(false));
                    let mut source = RomCpuTimingProbe::new(machine, clock,
                        SourcePpuReadState::snes9x_reset()).unwrap();
                    let mut measured = 0;
                    for _ in 0..1000 {
                        let pc = source.snes().cpu.pc;
                        let step = source.step().unwrap();
                        if (0xa7ab..=0xa80c).contains(&pc) {
                            measured += step.transactions.iter()
                                .map(|t| u64::from(t.duration_master_cycles)).sum::<u64>();
                        }
                        if source.snes().cpu.pc == 0x8000 { break; }
                    }
                    assert_eq!(source.snes().cpu.pc, 0x8000);
                    let actual = crate::cycle_ledger::measure_routine_self_work(0x08_a7ab, || {
                        native.ancilla_draw_explosion(0x800, frame, 0, count, 0x32, x, y);
                    });
                    assert_eq!(actual, measured, "frame={frame} count={count} x={x} y={y}");
                    native.set_lag_free(false);
                    assert_eq!(crate::cycle_ledger::measure_routine_self_work(0x08_a7ab, || {
                        native.ancilla_draw_explosion(0x800, frame, 0, count, 0x32, x, y);
                    }), 0);
                }
            }
        }
    }
}
