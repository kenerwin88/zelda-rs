//! Boomerang, hookshot, rods, arrows, sword beams, and attack sparkles.

use super::*;
const BOOMERANG_TAGALONG_LAYER_PRIORITY_BITS: [u8; 4] = [0x20, 0x10, 0x30, 0x20];
const FIRE_SHOT_DRAW_CHARS: [u8; 3] = [0xa2, 0xa0, 0x8e];
const SPIN_SPARK_TRAIL_CHARS: [u8; 3] = [0xb7, 0x80, 0x83];
use crate::tile_definition::{NativeTile};
use crate::types::{abs16, sign16, sign8, ProjectSpeedRet};

impl ZeldaState {
    fn boomerang_cheat_when_no_ones_looking(&self, k: usize, pt: &mut ProjectSpeedRet) {
        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_sub(self.ancilla_get_x(k))
            .wrapping_add(0xf0);
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_sub(self.ancilla_get_y(k))
            .wrapping_add(0xf0);
        if x >= 0x1e0 {
            pt.x = if sign16(x.wrapping_sub(0x1e0)) {
                0x90
            } else {
                0x70
            };
        } else if y >= 0x1e0 {
            pt.y = if sign16(y.wrapping_sub(0x1e0)) {
                0x90
            } else {
                0x70
            };
        }
    }

    fn boomerang_screen_edge(&self, k: usize) -> bool {
        let x = self.ancilla_get_x(k);
        let y = self.ancilla_get_y(k);
        if self.game_state.messaging.runtime.effect_index() & 3 != 0 {
            let t = x
                .wrapping_add(
                    if self.game_state.messaging.runtime.effect_index() & 1 != 0 {
                        16
                    } else {
                        0
                    },
                )
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
            if t >= 0x100 {
                return true;
            }
        }
        if self.game_state.messaging.runtime.effect_index() & 12 != 0 {
            let t = y
                .wrapping_add(
                    if self.game_state.messaging.runtime.effect_index() & 4 != 0 {
                        16
                    } else {
                        0
                    },
                )
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
            if t >= 0xe2 {
                return true;
            }
        }
        false
    }

    fn boomerang_stop_off_screen(&mut self, k: usize) {
        let x = self.ancilla_get_x(k).wrapping_add(8);
        let y = self.ancilla_get_y(k).wrapping_add(8);
        let link_x = self.game_state.player.follower_link.x();
        let link_y = self.game_state.player.follower_link.y();
        if x >= link_x && x < link_x.wrapping_add(16) && y >= link_y && y < link_y.wrapping_add(24)
        {
            self.boomerang_terminate(k);
        }
    }

    fn boomerang_terminate(&mut self, k: usize) {
        self.ancilla_slot_view_mut(k).clear();
        self.minigame_state_mut().clear_flag_boomerang_in_place();
        if self.game_state.player.follower_link.item_in_hand_has(0x80) {
            self.follower_link_state_mut().clear_item_in_hand();
            self.follower_link_state_mut()
                .clear_button_mask_b_y_bits(0x40);
            if self.game_state.player.follower_link.button_mask_b_y() & 0x80 == 0 {
                self.follower_link_state_mut().clear_direction_lock_bits(1);
            }
        }
    }

    pub(super) fn ancilla05_boomerang(&mut self, k: usize) {
        for j in (0..=4).rev() {
            if self.ancilla_slot_view(j).ancilla_type() == 0x22 {
                self.boomerang_draw(k);
                return;
            }
        }
        if self.game_state.frame.submodule != 0 {
            self.boomerang_draw(k);
            return;
        }

        if self.game_state.frame.frame_counter & 7 == 0 {
            self.ancilla_sfx2_pan(k, 9);
        }

        if self.ancilla_slot_view(k).aux_timer() == 0 {
            if self.game_state.player.follower_link.button_b_frames() < 9
                && self.game_state.player.follower_link.action_handler_timer() == 0
            {
                if self.game_state.player.follower_link.is_bunny_mirror()
                    || self.game_state.player.follower_link.has_auxiliary_state()
                    || !self.game_state.player.follower_link.has_item_in_hand()
                        && self
                            .game_state
                            .enhanced_features
                            .has(ANCILLA05_BOOMERANG_FEATURES0_MISC_BUG_FIXES)
                {
                    self.boomerang_terminate(k);
                    return;
                }
                self.boomerang_draw(k);
                return;
            }
            let j = (self.ancilla_slot_view(k).work_byte_23() >> 1) as usize;
            self.ancilla_set_xy(
                k,
                self.game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_add(ANCILLA05_BOOMERANG_BOOMERANG_X0[j] as i16 as u16),
                self.game_state
                    .player
                    .follower_link
                    .y()
                    .wrapping_add(8)
                    .wrapping_add(ANCILLA05_BOOMERANG_BOOMERANG_Y0[j] as i16 as u16),
            );
            self.ancilla_slot_view_mut(k).add_aux_timer(1);
        }

        if self.ancilla_slot_view(k).g() != 0 && self.game_state.frame.frame_counter & 1 == 0 {
            self.ancilla_add_sword_charge_sparkle(k);
        }

        if self.ancilla_slot_view(k).item_to_link() != 0 {
            if self.ancilla_slot_view(k).k() != 0 {
                self.ancilla_slot_view_mut(k).advance_k();
            }
            let link_y = self.game_state.player.follower_link.y();
            self.ancilla_slot_view_mut(k).set_a_word(link_y);
            self.follower_link_state_mut().set_y(link_y.wrapping_add(8));
            let speed = self.ancilla_slot_view(k).h();
            let mut pt = self.ancilla_project_speed_towards_player(k, speed);
            self.boomerang_cheat_when_no_ones_looking(k, &mut pt);
            {
                let mut boomerang = self.ancilla_slot_view_mut(k);
                boomerang.set_x_velocity(pt.x);
                boomerang.set_y_velocity(pt.y);
            }
            let y = self.ancilla_slot_view(k).a_word();
            self.follower_link_state_mut().set_y(y);
        }

        if self.ancilla_slot_view(k).y_velocity() != 0 {
            let acceleration = self.ancilla_slot_view(k).k();
            self.ancilla_slot_view_mut(k).add_y_velocity(acceleration);
        }
        self.ancilla_move_y(k);

        if self.ancilla_slot_view(k).x_velocity() != 0 {
            let acceleration = self.ancilla_slot_view(k).k();
            self.ancilla_slot_view_mut(k).add_x_velocity(acceleration);
        }
        self.ancilla_move_x(k);
        // ROM $08:91D2
        let hit_spr = self.ancilla_check_sprite_collision_marked(k);

        if self.ancilla_slot_view(k).item_to_link() == 0 {
            if hit_spr.is_some() {
                let item_to_link = self.ancilla_slot_view(k).item_to_link() ^ 1;
                self.ancilla_slot_view_mut(k).set_item_to_link(item_to_link);
            } else if self.ancilla_check_tile_collision(k) != 0 {
                self.ancilla_add_boomerang_wall_clink(k);
                self.ancilla_sfx2_pan(
                    k,
                    if self.ancilla_slot_view(k).tile_attribute() == 0xf0 {
                        6
                    } else {
                        5
                    },
                );
                let item_to_link = self.ancilla_slot_view(k).item_to_link() ^ 1;
                self.ancilla_slot_view_mut(k).set_item_to_link(item_to_link);
            } else {
                let reached_edge = self.boomerang_screen_edge(k);
                if !reached_edge {
                    self.ancilla_slot_view_mut(k).retreat_step();
                }
                if reached_edge || self.ancilla_slot_view(k).step() == 0 {
                    let item_to_link = self.ancilla_slot_view(k).item_to_link() ^ 1;
                    self.ancilla_slot_view_mut(k).set_item_to_link(item_to_link);
                } else if self.ancilla_slot_view(k).step() < 5 {
                    self.ancilla_slot_view_mut(k).retreat_k();
                }
            }
        } else {
            let bak0 = self.ancilla_slot_view(k).object_priority();
            let bak1 = self.ancilla_slot_view(k).floor();
            self.ancilla_slot_view_mut(k).set_floor(0);
            self.ancilla_check_tile_collision(k);
            {
                let mut ancilla = self.ancilla_slot_view_mut(k);
                ancilla.set_floor(bak1);
                ancilla.set_object_priority(bak0);
            }
            self.boomerang_stop_off_screen(k);
        }

        self.boomerang_draw(k);
    }

    fn hookshot_check_proximity_to_link(&self, x: i32, y: i32) -> bool {
        abs16(
            self.game_state
                .player
                .follower_link
                .y()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
                .wrapping_add(12)
                .wrapping_sub(y as u16)
                .wrapping_sub(4),
        ) < 12
            && abs16(
                self.game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2())
                    .wrapping_add(8)
                    .wrapping_sub(x as u16)
                    .wrapping_sub(4),
            ) < 12
    }

    fn hookshot_should_i_even_bother_with_tiles(&self, k: usize) -> bool {
        let x = self.ancilla_get_x(k);
        let y = self.ancilla_get_y(k);
        if self.game_state.world.location.is_outdoors() {
            let area = (self.game_state.world.region.current_area_of_player() >> 1) as usize;
            let bound = self.overworld_right_bottom_scroll_bound();
            if self.ancilla_slot_view(k).direction() & 2 == 0 {
                let t = y.wrapping_sub(ANCILLA_OVERWORLD_AREA_BASE_Y[area]);
                return t < 4 || t >= bound;
            } else {
                let t = x.wrapping_sub(ANCILLA_OVERWORLD_AREA_BASE_X[area]);
                return t < 6 || t >= bound;
            }
        }
        if self.ancilla_slot_view(k).direction() & 2 == 0 {
            (y & 0x1ff) < 4
                || (y & 0x1ff) >= 0x1e8
                || (y & 0x200) != (self.game_state.player.follower_link.y() & 0x200)
        } else {
            (x & 0x1ff) < 4
                || (x & 0x1ff) >= 0x1f0
                || (x & 0x200) != (self.game_state.player.follower_link.x() & 0x200)
        }
    }

    fn boomerang_draw(&mut self, k: usize) {
        let (info_x, info_y) = self.ancilla_prep_oam_coord(k);

        if self.ancilla_slot_view(k).item_to_link() != 0 {
            let floor = self.game_state.player.follower_link.lower_level_state();
            self.ancilla_slot_view_mut(k).set_floor(floor);
            let priority = (BOOMERANG_TAGALONG_LAYER_PRIORITY_BITS
                [self.game_state.player.follower_link.lower_level_state() as usize]
                as u16)
                << 8;
            self.oam_state_mut().set_priority_word(priority);
        }

        if self.ancilla_slot_view(k).object_priority() != 0 {
            self.oam_state_mut().set_priority_word(0x3000);
        }

        if self.game_state.frame.submodule == 0
            && self.ancilla_slot_view(k).aux_timer() != 0
            && sign8(self.ancilla_slot_view_mut(k).tick_work_byte_3())
        {
            let frame_reset = BOOMERANG_DRAW_BOOMERANG_FRAME_RESET_BY_TYPE
                [self.ancilla_slot_view(k).g() as usize];
            self.ancilla_slot_view_mut(k).set_work_byte_3(frame_reset);
            let delta = if self.ancilla_slot_view(k).s_player() != 0 {
                0xff
            } else {
                1
            };
            self.ancilla_slot_view_mut(k).add_work_byte_1_mod4(delta);
        }

        let j = self.ancilla_slot_view(k).work_byte_1() as usize;
        let offset = BOOMERANG_DRAW_BOOMERANG_DRAW_OFFSET[j];
        let x = info_x.wrapping_add(offset.x as i16 as u16);
        let y = info_y.wrapping_add(offset.y as i16 as u16);
        if self.ancilla_slot_view(k).aux_timer() == 0 {
            let i = BOOMERANG_DRAW_BOOMERANG_DRAW_OAM_IDX
                [self.game_state.oam.sprite_sorting_offset_index()];
            self.oam_state_mut()
                .set_current_extended_pointer((i >> 2) + 0xa20);
            self.oam_state_mut().set_current_pointer(i + 0x800);
        }
        self.ancilla_set_oam_safe(
            self.game_state.oam.current_pointer_usize(),
            x,
            y,
            0x26,
            (BOOMERANG_DRAW_BOOMERANG_FLAGS[self.ancilla_slot_view(k).g() as usize * 4 + j]
                & !0x30)
                | self.game_state.oam.priority_high(),
            2,
        );
    }

    pub(super) fn ancilla02_fire_rod_shot(&mut self, k: usize) {
        if self.ancilla_slot_view(k).step() == 0 {
            if self.game_state.frame.submodule == 0 {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_l(value);
                self.ancilla_move_x(k);
                self.ancilla_move_y(k);
                // ROM $08:86E7
                let mut coll = self.ancilla_check_sprite_collision_marked(k).is_some();
                if !coll {
                    self.ancilla_slot_view_mut(k).or_direction(8);
                    coll = self.ancilla_check_tile_collision(k) != 0;
                    let value = self.ancilla_slot_view(k).tile_attribute();
                    self.ancilla_slot_view_mut(k).set_l(value);
                    if !coll {
                        self.ancilla_slot_view_mut(k).or_direction(12);
                        let bak = self.ancilla_slot_view(k).u();
                        coll = self.ancilla_check_tile_collision(k) != 0;
                        let value = bak;
                        self.ancilla_slot_view_mut(k).set_u(value);
                    }
                }
                if coll {
                    self.ancilla_slot_view_mut(k).add_step(1);
                    let value = 31;
                    self.ancilla_slot_view_mut(k).set_timer(value);
                    let value = 8;
                    self.ancilla_slot_view_mut(k).set_num_sprites(value);
                    self.ancilla_sfx2_pan(k, 0x2a);
                }
                let value = self.ancilla_slot_view(k).item_to_link().wrapping_add(1);
                self.ancilla_slot_view_mut(k).set_item_to_link(value);
                self.ancilla_slot_view_mut(k).and_direction(!0x0c);
                let target = NativeTile::from_cartridge(self.ancilla_slot_view(k).l());
                self.dungeon_torch_mut().set_target(target);
                if self.game_state.dungeon.torch.targets_torch() {
                    self.dungeon_light_torch();
                } else {
                    let target =
                        NativeTile::from_cartridge(self.ancilla_slot_view(k).tile_attribute());
                    self.dungeon_torch_mut().set_target(target);
                    if self.game_state.dungeon.torch.targets_torch() {
                        self.dungeon_light_torch();
                    }
                }
            }
            self.fire_shot_draw(k);
        } else {
            self.ancilla_check_basic_sprite_collision(k);
            let Some(info) = self.ancilla_return_if_outside_bounds(k) else {
                return;
            };
            let oam = self.game_state.oam.current_pointer_usize();
            if self.ancilla_slot_view(k).timer() == 0 {
                let old_type = self.ancilla_slot_view(k).ancilla_type();
                self.ancilla_slot_view_mut(k).clear();
                if old_type != 0x2f
                    && self.game_state.world.location.overworld_screen_index() == 64
                    && self.ancilla_slot_view(k).tile_attribute() == 0x43
                {
                    self.fire_rod_shot_become_skull_woods_fire(k);
                }
                return;
            }
            let j = self.ancilla_slot_view(k).timer() >> 3;
            if j != 0 {
                self.ancilla_set_oam_plain(
                    oam,
                    info.x as u16,
                    info.y as u16,
                    FIRE_SHOT_DRAW_CHARS[j as usize - 1],
                    info.flags | 2,
                    2,
                );
            } else {
                self.ancilla_set_oam_plain(
                    oam,
                    info.x as u16,
                    info.y.wrapping_sub(3) as u16,
                    0xa4,
                    info.flags | 2,
                    0,
                );
                self.ancilla_set_oam_plain(
                    oam + 4,
                    info.x.wrapping_add(8) as u16,
                    info.y.wrapping_sub(3) as u16,
                    0xa5,
                    info.flags | 2,
                    0,
                );
            }
        }
    }

    fn fire_rod_shot_become_skull_woods_fire(&mut self, _k: usize) {
        if self.game_state.world.location.is_indoors()
            || self.game_state.world.location.overworld_screen_index() & 0x40 == 0
        {
            return;
        }

        self.ancilla_slot_view_mut(0).set_ancilla_type(0x34);
        for i in 1..=5 {
            self.ancilla_slot_view_mut(i).clear();
        }
        self.minigame_state_mut().clear_flag_boomerang_in_place();
        self.ancilla_slot_view_mut(0)
            .set_num_sprites(ANCILLA_DRAW_SPRITE_COUNTS[0x34]);
        self.skull_woods_fire_mut(0).set_phase(253);
        self.skull_woods_fire_mut(1).set_phase(254);
        self.skull_woods_fire_mut(2).set_phase(255);
        self.skull_woods_fire_mut(3).set_phase(0);
        self.skull_woods_fire_scratch_mut()
            .clear_entrance_opening_started();
        for i in 0..4 {
            self.skull_woods_fire_mut(i).set_timer(5);
        }
        self.ancilla_slot_view_mut(0).set_aux_timer(5);
        self.skull_woods_fire_scratch_mut()
            .set_inner_position(0x0098, 0x0100);
        self.skull_woods_fire_scratch_mut()
            .set_outer_position(0x0098, 0x0100);
        self.set_special_entrance_trigger(2);
        self.set_subsubmodule(0);
        self.scratch_word_mut().clear_module_transition_counter();
        let value = self.game_state.player.follower_link.lower_level_state();
        self.ancilla_slot_view_mut(0).set_floor(value);
        let value = self
            .game_state
            .player
            .follower_link
            .lower_level_mirror_state();
        self.ancilla_slot_view_mut(0).set_floor2(value);
        self.ancilla_slot_view_mut(0).set_item_to_link(0);
        self.ancilla_slot_view_mut(0).set_step(0);
    }

    pub(super) fn ancilla0_b_ice_rod_shot(&mut self, k: usize) {
        if self.game_state.frame.submodule == 0 {
            let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
            if sign8(aux_timer) {
                let item_to_link = self.ancilla_slot_view_mut(k).advance_item_to_link();
                if item_to_link & !1 != 0 {
                    let mut ice_shot = self.ancilla_slot_view_mut(k);
                    ice_shot.set_step(1);
                    ice_shot.set_item_to_link(item_to_link & 7 | 4);
                }
                self.ancilla_slot_view_mut(k).set_aux_timer(3);
            }
            if self.ancilla_slot_view(k).step() != 0 {
                if self.ancilla_return_if_outside_bounds(k).is_none() {
                    return;
                }
                self.ancilla_move_y(k);
                self.ancilla_move_x(k);
                // ROM $08:A515
                if self.ancilla_check_sprite_collision_marked(k).is_some()
                    || self.ancilla_check_tile_collision(k) != 0
                {
                    self.ancilla_slot_view_mut(k).set_ancilla_type(0x11);
                    let value = ANCILLA_DRAW_SPRITE_COUNTS[0x11];
                    self.ancilla_slot_view_mut(k).set_num_sprites(value);
                    let mut ancilla = self.ancilla_slot_view_mut(k);
                    ancilla.set_item_to_link(0);
                    ancilla.set_aux_timer(4);
                }
            }
        }
        self.ancilla_add_ice_rod_sparkle(k);
    }

    pub(super) fn ancilla09_arrow(&mut self, k: usize) {
        if self.game_state.frame.submodule != 0 {
            self.arrow_draw(k);
            return;
        }

        let item_to_link = self.ancilla_slot_view_mut(k).retreat_item_to_link();
        if !sign8(item_to_link) {
            if item_to_link >= 4 {
                return;
            }
        } else {
            self.ancilla_slot_view_mut(k).set_item_to_link(0xff);
        }
        self.ancilla_move_y(k);
        self.ancilla_move_x(k);
        if self.game_state.inventory.items.has_silver_arrows()
            && self.game_state.frame.frame_counter & 1 == 0
        {
            self.ancilla_add_silver_arrow_sparkle(k);
        }
        let value = 255;
        self.ancilla_slot_view_mut(k).set_s_player(value);
        let j;
        // ROM $08:A16B
        if let Some(sprite) = self.ancilla_check_sprite_collision_marked(k) {
            j = sprite;
            let value = self
                .ancilla_slot_view(k)
                .x_low()
                .wrapping_sub(self.sprite_slot_view(sprite).x_low());
            self.ancilla_slot_view_mut(k).set_x_velocity(value);
            let value = self
                .ancilla_slot_view(k)
                .y_low()
                .wrapping_sub(self.sprite_slot_view(sprite).y_low())
                .wrapping_add(self.sprite_slot_view(sprite).z());
            self.ancilla_slot_view_mut(k).set_y_velocity(value);
            let value = sprite as u8;
            self.ancilla_slot_view_mut(k).set_s_player(value);
            if self.sprite_slot_view(sprite).sprite_type() == 0x65 {
                if self.sprite_slot_view(sprite).a() == 1 {
                    self.set_sound_effect_2(0x2d);
                    let value = 0x80;
                    self.sprite_slot_view_mut(sprite).set_delay_aux2(value);
                    self.sprite_slot_view_mut(0).set_delay_aux4(128);
                    if self.game_state.archery_game.hit_counter() < 9 {
                        self.archery_game_mut().increment_hit_counter();
                    }
                    let value = self.game_state.archery_game.hit_counter();
                    self.sprite_slot_view_mut(sprite).set_b(value);
                    let value = self.sprite_slot_view(sprite).g().wrapping_add(1);
                    self.sprite_slot_view_mut(sprite).set_g(value);
                } else {
                    let value = 4;
                    self.sprite_slot_view_mut(sprite).set_delay_aux3(value);
                    self.archery_game_mut().clear_hit_counter();
                }
            } else {
                self.archery_game_mut().clear_hit_counter();
            }
        } else {
            let coll = self.ancilla_check_tile_collision(k);
            if coll != 0 {
                let value = coll >> 1;
                self.ancilla_slot_view_mut(k).set_h(value);
                let dir = (self.ancilla_slot_view(k).direction() & 3) as usize;
                self.ancilla_set_xy(
                    k,
                    self.ancilla_get_x(k)
                        .wrapping_add(ANCILLA09_ARROW_ARROW_X[dir] as i16 as u16),
                    self.ancilla_get_y(k)
                        .wrapping_add(ANCILLA09_ARROW_ARROW_Y[dir] as i16 as u16),
                );
                self.archery_game_mut().clear_hit_counter();
                j = dir;
            } else {
                self.arrow_draw(k);
                return;
            }
        }
        if self.sprite_slot_view(j).sprite_type() != 0x1b {
            self.ancilla_sfx2_pan(k, 8);
        }
        {
            let mut arrow = self.ancilla_slot_view_mut(k);
            arrow.set_item_to_link(0);
            arrow.set_ancilla_type(10);
            arrow.set_aux_timer(1);
        }
        if self.ancilla_slot_view(k).h() != 0 {
            let value = self
                .ancilla_slot_view(k)
                .x_low()
                .wrapping_add(self.game_state.display.ppu_scroll_copy.bg1_h_copy2_low())
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2_low());
            self.ancilla_slot_view_mut(k).set_x_low(value);
            let value = self
                .ancilla_slot_view(k)
                .y_low()
                .wrapping_add(self.game_state.display.ppu_scroll_copy.bg1_v_copy2_low())
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low());
            self.ancilla_slot_view_mut(k).set_y_low(value);
        }
        self.arrow_draw(k);
    }

    pub(super) fn ancilla_sword_beam(&mut self, k: usize) {
        let mut flags = 2;

        if self.game_state.frame.submodule == 0 {
            self.ancilla_set_xy(
                k,
                self.game_state.sprites.ether_orbit.swordbeam_temp_x(),
                self.game_state.sprites.ether_orbit.swordbeam_temp_y(),
            );
            self.ancilla_move_x(k);
            self.ancilla_move_y(k);
            let x = self.ancilla_get_x(k);
            let y = self.ancilla_get_y(k);
            self.ether_orbit_mut().set_swordbeam_temp(x, y);

            let g = self.ancilla_slot_view(k).g();
            let value = g.wrapping_add(1);
            self.ancilla_slot_view_mut(k).set_g(value);
            if g & 0x0f == 0 {
                self.set_sound_effect_2_with_ancilla_pan(k, 1);
            }

            if self.ancilla_check_sprite_collision(k).is_some()
                || self.ancilla_check_tile_collision(k) != 0
            {
                let j = self.ancilla_slot_view(k).direction() as usize;
                self.ancilla_set_xy(
                    k,
                    self.ancilla_get_x(k)
                        .wrapping_add(ANCILLA_SWORD_BEAM_SWORD_BEAM_XVEL2[j] as i16 as u16),
                    self.ancilla_get_y(k)
                        .wrapping_add(ANCILLA_SWORD_BEAM_SWORD_BEAM_YVEL2[j] as i16 as u16),
                );
                {
                    let mut beam = self.ancilla_slot_view_mut(k);
                    beam.set_ancilla_type(4);
                    beam.set_timer(7);
                }
                let value = 0x10;
                self.ancilla_slot_view_mut(k).set_num_sprites(value);
                return;
            }
            let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
            if sign8(aux_timer) {
                flags = 4;
                self.ancilla_slot_view_mut(k).set_aux_timer(2);
            }
        }

        let oam_org = self.game_state.oam.current_pointer_usize();
        let mut oam = oam_org;
        let s = self.ancilla_slot_view(k).s_player();
        for i in (0..=3).rev() {
            let angle = if self.game_state.frame.submodule == 0 {
                self.effect_angle_scratch_mut().add_angle_mod64(i, s)
            } else {
                self.game_state.effects.angle_scratch.angle(i)
            };
            let pt = self.sparkle_prep_oam_from_radial(self.ancilla_get_radial_projection(
                angle,
                self.game_state.effects.angle_scratch.radial_radius(),
            ));
            self.ancilla_set_oam(
                oam,
                pt.x,
                pt.y,
                ANCILLA_SWORD_BEAM_SWORD_BEAM_CHAR[i],
                flags | self.game_state.oam.priority_high(),
                0,
            );
            oam += 4;
        }

        if self.game_state.frame.submodule == 0 {
            let work_byte_3 = self.ancilla_slot_view_mut(k).tick_work_byte_3();
            if !sign8(work_byte_3) {
                self.ancilla_sword_beam_check_offscreen(k, oam_org);
                return;
            }

            let work_byte_1 = {
                let mut beam = self.ancilla_slot_view_mut(k);
                beam.set_work_byte_3(0);
                beam.advance_work_byte_1_mod4()
            };
            if work_byte_1 == 3 {
                self.effect_angle_scratch_mut().add_trailing_angle_mod64(s);
            }
        }

        let t = self.ancilla_slot_view(k).work_byte_1();
        if t != 3 {
            let pt = self.sparkle_prep_oam_from_radial(self.ancilla_get_radial_projection(
                self.game_state.effects.angle_scratch.trailing_angle(),
                self.game_state.effects.angle_scratch.radial_radius(),
            ));
            self.ancilla_set_oam(
                oam,
                pt.x,
                pt.y,
                ANCILLA_SWORD_BEAM_SWORD_BEAM_CHAR2[t as usize],
                4 | self.game_state.oam.priority_high(),
                0,
            );
        }

        self.ancilla_sword_beam_check_offscreen(k, oam_org);
    }

    fn ancilla_sword_beam_check_offscreen(&mut self, k: usize, oam_org: usize) {
        for i in 0..4 {
            if self.game_state.oam.entry_y(oam_org + i * 4) != 0xf0 {
                return;
            }
        }
        self.ancilla_slot_view_mut(k).clear();
    }

    pub(super) fn ancilla0_d_spin_attack_full_charge_spark(&mut self, k: usize) {
        let value = self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, 4) as u8;

        self.ancilla_slot_view_mut(k).set_oam_index(value);

        if self.ancilla_slot_view(k).timer() == 0 {
            self.ancilla_slot_view_mut(k).clear();
            return;
        }

        let j = self.game_state.player.follower_link.facing_index();

        let x = self
            .game_state
            .player
            .follower_link
            .x()
            .wrapping_add(
                ANCILLA0_D_SPIN_ATTACK_FULL_CHARGE_SPARK_SWORD_FULL_CHARGE_SPARK_X[j] as i16 as u16,
            )
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let y = self
            .game_state
            .player
            .follower_link
            .y()
            .wrapping_add(
                ANCILLA0_D_SPIN_ATTACK_FULL_CHARGE_SPARK_SWORD_FULL_CHARGE_SPARK_Y[j] as i16 as u16,
            )
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());

        let flags = ANCILLA0_D_SPIN_ATTACK_FULL_CHARGE_SPARK_SWORD_FULL_CHARGE_SPARK_FLAGS
            [self.ancilla_slot_view(k).floor() as usize];
        self.oam_state_mut().set_priority_word((flags as u16) << 8);
        let oam = self.game_state.oam.current_pointer_usize();
        self.ancilla_set_oam(oam, x, y, 0xd7, flags | 2, 0);
    }

    pub(super) fn ancilla26_sword_swing_sparkle(&mut self, k: usize) {
        // $08:D65A..D703: timer, three-tile writer and signed offsets.
        // F671 coordinate preparation and F6E1 XY writes own their bodies.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x08d65a));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x08d65a);
        let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
        if sign8(aux_timer) {
            b.block(0x08d65f);
            let item_to_link = {
                let mut sparkle = self.ancilla_slot_view_mut(k);
                sparkle.set_aux_timer(0);
                sparkle.advance_item_to_link()
            };
            if item_to_link == 4 {
                b.block(0x08d66e);
                self.ancilla_slot_view_mut(k).clear();
                return;
            } else { b.taken(0x08d66c); }
        } else { b.taken(0x08d65d); }
        b.block(0x08d672);
        self.ancilla_set_xy(
            k,
            self.game_state.player.follower_link.x(),
            self.game_state.player.follower_link.y(),
        );

        let (x, y) = self.ancilla_prep_oam_coord(k);

        b.block(0x08d68a);

        let mut j = self.ancilla_slot_view(k).item_to_link() as usize * 3
            + self.ancilla_slot_view(k).direction() as usize * 12;

        let mut oam = self.game_state.oam.current_pointer_usize();
        for i in (0..=2).rev() {
            b.block(0x08d6ac);
            let chr = ANCILLA26_SWORD_SWING_SPARKLE_SWORD_SWING_SPARKLE_CHAR[j];
            if chr != 0xff {
                b.block(0x08d6b3);
                if 0xf6 + j > 0xff { b.crossing(0x08d6b5); }
                // $D5F6 is the Y-offset table ($00/$04 in the ROM);
                // $D626 is X ($02/$06). Native drawing names X first.
                if ANCILLA26_SWORD_SWING_SPARKLE_SWORD_SWING_SPARKLE_Y[j] < 0 {
                    b.block(0x08d6c0);
                } else { b.taken(0x08d6be); }
                b.block(0x08d6c3);
                if 0x26 + j > 0xff { b.crossing(0x08d6c8); }
                if ANCILLA26_SWORD_SWING_SPARKLE_SWORD_SWING_SPARKLE_X[j] < 0 {
                    b.block(0x08d6d3);
                } else { b.taken(0x08d6d1); }
                b.block(0x08d6d6);
                // $08:D6DD JSR F6E1: complete XY body, once per executed draw.
                self.ancilla_set_oam_marked(
                    oam,
                    x.wrapping_add(
                        ANCILLA26_SWORD_SWING_SPARKLE_SWORD_SWING_SPARKLE_X[j] as i16 as u16,
                    ),
                    y.wrapping_add(
                        ANCILLA26_SWORD_SWING_SPARKLE_SWORD_SWING_SPARKLE_Y[j] as i16 as u16,
                    ),
                    chr,
                    ANCILLA26_SWORD_SWING_SPARKLE_SWORD_SWING_SPARKLE_FLAGS[j]
                        | 0x4
                        | self.game_state.oam.priority_high(),
                    0,
                );
                b.block(0x08d6e0);
            } else { b.taken(0x08d6b1); }
            b.block(0x08d6fd);
            if i != 0 { b.taken(0x08d700); }
            j += 1;
            oam += 4;
        }
        b.block(0x08d702);
    }

    pub(super) fn ancilla3_b_sword_up_sparkle(&mut self, k: usize) {
        if self.ancilla_slot_view(k).aux_timer() != 0 {
            self.ancilla_slot_view_mut(k).tick_aux_timer();
            return;
        }

        if sign8(self.ancilla_slot_view_mut(k).tick_work_byte_3()) {
            self.ancilla_slot_view_mut(k).set_work_byte_3(1);
            let sparkle_phase = self.ancilla_slot_view_mut(k).advance_item_to_link();
            if sparkle_phase == 4 {
                let mut sparkle = self.ancilla_slot_view_mut(k);
                sparkle.clear();
                sparkle.tick_aux_timer();
                return;
            }
        }
        self.ancilla_prep_oam_coord(k);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let mut j = self.ancilla_slot_view(k).item_to_link() as usize * 4;
        for _ in 0..4 {
            if ANCILLA3_B_SWORD_UP_SPARKLE_ANCILLA_VICTORY_SPARKLE_CHAR[j] != 0xff {
                // $08:C1BD JSR F6E1: complete XY body, once per executed draw.
                self.ancilla_set_oam_marked(
                    oam,
                    self.game_state
                        .player
                        .follower_link
                        .x()
                        .wrapping_add(
                            ANCILLA3_B_SWORD_UP_SPARKLE_ANCILLA_VICTORY_SPARKLE_X[j] as i16 as u16,
                        )
                        .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()),
                    self.game_state
                        .player
                        .follower_link
                        .y()
                        .wrapping_add(
                            ANCILLA3_B_SWORD_UP_SPARKLE_ANCILLA_VICTORY_SPARKLE_Y[j] as i16 as u16,
                        )
                        .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()),
                    ANCILLA3_B_SWORD_UP_SPARKLE_ANCILLA_VICTORY_SPARKLE_CHAR[j],
                    ANCILLA3_B_SWORD_UP_SPARKLE_ANCILLA_VICTORY_SPARKLE_FLAGS[j]
                        | 4
                        | self.game_state.oam.priority_high(),
                    0,
                );
                oam += 4;
            }
            j += 1;
        }
    }

    fn spin_spark_draw(&mut self, k: usize, offs: i32) {
        let (x, y) = self.ancilla_prep_oam_coord(k);
        let mut oam = self.game_state.oam.current_pointer_usize();
        let mut t = (i32::from(self.ancilla_slot_view(k).item_to_link()) + offs) * 4;
        assert!(t < 32);
        for _ in 0..4 {
            let idx = t as usize;
            if SPIN_SPARK_DRAW_INITIAL_SPIN_SPARK_CHAR[idx] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x.wrapping_add(SPIN_SPARK_DRAW_INITIAL_SPIN_SPARK_X[idx] as u16),
                    y.wrapping_add(SPIN_SPARK_DRAW_INITIAL_SPIN_SPARK_Y[idx] as i16 as u16),
                    SPIN_SPARK_DRAW_INITIAL_SPIN_SPARK_CHAR[idx],
                    SPIN_SPARK_DRAW_INITIAL_SPIN_SPARK_FLAGS[idx] & !0x30
                        | self.game_state.oam.priority_high(),
                    0,
                );
                oam += 4;
            }
            t += 1;
        }
    }

    pub(super) fn ancilla2_a_spin_attack_sparkle_a(&mut self, k: usize) {
        if self.game_state.frame.submodule == 0 {
            let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
            if sign8(aux_timer) {
                self.ancilla_slot_view_mut(k).set_aux_timer(0);
                if self.ancilla_slot_view(k).timer() == 0 {
                    let j = self.ancilla_slot_view(k).item_to_link().wrapping_add(1);
                    {
                        let mut sparkle = self.ancilla_slot_view_mut(k);
                        sparkle.set_item_to_link(j);
                        sparkle.set_timer(
                            ANCILLA2_A_SPIN_ATTACK_SPARKLE_A_INITIAL_SPIN_SPARK_TIMER[j as usize],
                        );
                    }
                    if j == 5 {
                        if self.ancilla_slot_view(k).step() != 0 {
                            self.add_sword_beam(j);
                        } else {
                            self.spin_attack_sparkle_a_transmute_to_next_spark(k);
                        }
                        return;
                    }
                }
            }
        }
        if self.ancilla_slot_view(k).item_to_link() == 0 {
            return;
        }
        self.spin_spark_draw(k, -1);
    }

    fn spin_attack_sparkle_a_transmute_to_next_spark(&mut self, k: usize) {
        let mut j = self.game_state.player.follower_link.facing() as usize * 2;
        self.effect_angle_scratch_mut().set_angles4(
            &SPIN_ATTACK_SPARKLE_A_TRANSMUTE_TO_NEXT_SPARK_TRANSMUTE_SPIN_SPARK_ARR,
            j,
        );
        self.effect_angle_scratch_mut().set_trailing_angle(
            SPIN_ATTACK_SPARKLE_A_TRANSMUTE_TO_NEXT_SPARK_TRANSMUTE_SPIN_SPARK_ARR[j + 3],
        );
        {
            let mut sparkle = self.ancilla_slot_view_mut(k);
            sparkle.set_ancilla_type(0x2b);
            sparkle.set_aux_timer(2);
            sparkle.set_item_to_link(0x4c);
            sparkle.set_work_byte_3(8);
            sparkle.set_step(0);
            sparkle.set_l(0);
            sparkle.set_work_byte_1(255);
        }
        self.effect_angle_scratch_mut().set_radial_radius(20);

        let swordbeam_temp_x = self.game_state.player.follower_link.x().wrapping_add(8);
        let swordbeam_temp_y = self.game_state.player.follower_link.y().wrapping_add(12);
        self.ether_orbit_mut()
            .set_swordbeam_temp(swordbeam_temp_x, swordbeam_temp_y);

        j = self.game_state.player.follower_link.facing_index();
        self.ancilla_set_xy(
            k,
            self.game_state.player.follower_link.x().wrapping_add(
                SPIN_ATTACK_SPARKLE_A_TRANSMUTE_TO_NEXT_SPARK_TRANSMUTE_SPIN_SPARK_X[j] as i16
                    as u16,
            ),
            self.game_state.player.follower_link.y().wrapping_add(
                SPIN_ATTACK_SPARKLE_A_TRANSMUTE_TO_NEXT_SPARK_TRANSMUTE_SPIN_SPARK_Y[j] as i16
                    as u16,
            ),
        );
        self.ancilla2_b_spin_attack_sparkle_b(k);
    }

    pub(super) fn ancilla2_b_spin_attack_sparkle_b(&mut self, k: usize) {
        if self.ancilla_slot_view(k).l() != 0 {
            self.spin_attack_sparkle_b_closer(k);
            return;
        }
        let mut flags = 2;
        if self.game_state.frame.submodule == 0 {
            let t = self.ancilla_slot_view(k).item_to_link().wrapping_sub(3);
            self.ancilla_slot_view_mut(k).set_item_to_link(t);
            if t < 13 {
                let mut sparkle = self.ancilla_slot_view_mut(k);
                sparkle.set_aux_timer(1);
                sparkle.set_l(1);
                sparkle.set_item_to_link(0);
                self.spin_attack_sparkle_b_closer(k);
                return;
            }
            let step = if t < 0x42 {
                3
            } else if t == 0x46 {
                1
            } else if t == 0x43 {
                2
            } else {
                0
            };
            let aux_timer = {
                let mut sparkle = self.ancilla_slot_view_mut(k);
                sparkle.set_step(step);
                sparkle.tick_aux_timer()
            };
            if sign8(aux_timer) {
                flags = 4;
                self.ancilla_slot_view_mut(k).set_aux_timer(2);
            }
        }

        let oam_org = self.game_state.oam.current_pointer_usize();
        let mut oam = oam_org;
        let mut i = self.ancilla_slot_view(k).step() as usize;
        loop {
            let angle = if self.game_state.frame.submodule == 0 {
                self.effect_angle_scratch_mut().add_angle_mod64(i, 4)
            } else {
                self.game_state.effects.angle_scratch.angle(i)
            };
            let pt = self.sparkle_prep_oam_from_radial(self.ancilla_get_radial_projection(
                angle,
                self.game_state.effects.angle_scratch.radial_radius(),
            ));
            self.ancilla_set_oam(
                oam,
                pt.x,
                pt.y,
                ANCILLA2_B_SPIN_ATTACK_SPARKLE_B_SPIN_SPARK_CHAR[i],
                flags | self.game_state.oam.priority_high(),
                0,
            );
            oam += 4;
            if i == 0 {
                break;
            }
            i -= 1;
        }

        if self.game_state.frame.submodule == 0 {
            let work_byte_3 = self.ancilla_slot_view_mut(k).tick_work_byte_3();
            if !sign8(work_byte_3) {
                if self.ancilla_slot_view(k).item_to_link() == 7 {
                    let value = 1;
                    self.oam_state_mut()
                        .set_extended_byte((oam_org - OAM_BUF) / 4 + 3, value);
                }
                return;
            }

            let work_byte_1 = {
                let mut sparkle = self.ancilla_slot_view_mut(k);
                sparkle.set_work_byte_3(0);
                sparkle.advance_work_byte_1_mod4()
            };
            if work_byte_1 == 3 {
                self.effect_angle_scratch_mut().add_trailing_angle_mod64(9);
            }
        }

        let t = self.ancilla_slot_view(k).work_byte_1();
        if t != 3 {
            let pt = self.sparkle_prep_oam_from_radial(self.ancilla_get_radial_projection(
                self.game_state.effects.angle_scratch.trailing_angle(),
                self.game_state.effects.angle_scratch.radial_radius(),
            ));
            self.ancilla_set_oam(
                oam,
                pt.x,
                pt.y,
                SPIN_SPARK_TRAIL_CHARS[t as usize],
                4 | self.game_state.oam.priority_high(),
                0,
            );
        }
        if self.ancilla_slot_view(k).item_to_link() == 7 {
            let value = 1;
            self.oam_state_mut()
                .set_extended_byte((oam_org - OAM_BUF) / 4 + 3, value);
        }
    }

    fn spin_attack_sparkle_b_closer(&mut self, k: usize) {
        let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
        if sign8(aux_timer) {
            let item_to_link = {
                let mut sparkle = self.ancilla_slot_view_mut(k);
                sparkle.set_aux_timer(1);
                sparkle.advance_item_to_link()
            };
            if item_to_link == 3 {
                self.ancilla_slot_view_mut(k).clear();
            }
        }
        self.spin_spark_draw(k, 4);
    }

    pub(super) fn ancilla1_f_hookshot(&mut self, k: usize) {
        if self.game_state.frame.submodule == 0 {
            if self.ancilla_slot_view(k).timer() == 0 {
                self.ancilla_slot_view_mut(k).set_timer(7);
                self.ancilla_sfx2_pan(k, 0x0a);
            }

            if !self
                .game_state
                .player
                .follower_link
                .has_hookshot_interlock()
            {
                self.ancilla_move_y(k);
                self.ancilla_move_x(k);
                if self.ancilla_slot_view(k).step() != 0 {
                    let hookshot_length = self.ancilla_slot_view_mut(k).retreat_item_to_link();
                    if sign8(hookshot_length) {
                        self.ancilla_slot_view_mut(k).clear();
                        return;
                    }
                } else {
                    let hookshot_length = self.ancilla_slot_view_mut(k).advance_item_to_link();
                    if hookshot_length == 32 {
                        let mut hookshot = self.ancilla_slot_view_mut(k);
                        hookshot.set_step(1);
                        hookshot.negate_x_velocity();
                        hookshot.negate_y_velocity();
                    }

                    if !self.hookshot_should_i_even_bother_with_tiles(k) {
                        if self.ancilla_slot_view(k).l() == 0
                            && self.ancilla_slot_view(k).step() == 0
                            // ROM $08:BDD7
                            && self.ancilla_check_sprite_collision_marked(k).is_some()
                            && self.ancilla_slot_view(k).step() == 0
                        {
                            let mut hookshot = self.ancilla_slot_view_mut(k);
                            hookshot.set_step(1);
                            hookshot.negate_y_velocity();
                            hookshot.negate_x_velocity();
                        }

                        self.hookshot_check_tile_collision(k as i32);

                        let mut r0 = 0u8;
                        let contact = if self.game_state.world.location.is_indoors() {
                            if self.ancilla_slot_view(k).direction() & 2 == 0 {
                                r0 = (self.game_state.player.tile_detection.vertical_ledge()
                                    | (self.game_state.player.tile_detection.vertical_ledge()
                                        >> 4))
                                    & 3;
                            } else {
                                r0 = self.game_state.player.tile_detection.horizontal_ledge() & 3;
                            }
                            r0 != 0
                        } else {
                            (self.game_state.player.tile_detection.horizontal_ledge() & 3
                                | self.game_state.player.tile_detection.vertical_ledge()
                                | self.game_state.player.tile_detection.diagonal_ledge_tiles())
                                & 0x33
                                != 0
                        };

                        if contact {
                            self.ancilla_slot_view_mut(k).tick_g();
                        }
                        if contact && sign8(self.ancilla_slot_view(k).g()) {
                            if self.ancilla_slot_view(k).k() != 0
                                && ((r0 & 3) != 0
                                    || self.ancilla_slot_view(k).k()
                                        != self
                                            .game_state
                                            .player
                                            .tile_detection
                                            .interacting_tile_low())
                            {
                                self.ancilla_slot_view_mut(k).set_g(2);
                                let l = self.ancilla_slot_view_mut(k).retreat_l();
                                if sign8(l) {
                                    self.ancilla_slot_view_mut(k).set_l(0);
                                }
                            } else {
                                let interacting_tile =
                                    self.game_state.player.tile_detection.interacting_tile_low();
                                let mut hookshot = self.ancilla_slot_view_mut(k);
                                hookshot.advance_l();
                                hookshot.set_k(interacting_tile);
                                hookshot.set_g(1);
                            }
                        }

                        if self.ancilla_slot_view(k).l() == 0 {
                            if !sign8(self.ancilla_slot_view(k).g()) {
                                self.ancilla_slot_view_mut(k).tick_g();
                            } else {
                                let collision_bits =
                                    self.game_state.player.tile_detection.collision_bits();
                                let blocked = (((collision_bits >> 4)
                                    | collision_bits
                                    | self.game_state.player.tile_detection.stair_tile() as u16
                                    | self
                                        .game_state
                                        .player
                                        .tile_detection
                                        .slope_collision_bits())
                                    & 3)
                                    != 0;
                                if blocked && self.ancilla_slot_view(k).step() == 0 {
                                    let mut hookshot = self.ancilla_slot_view_mut(k);
                                    hookshot.set_step(1);
                                    hookshot.negate_y_velocity();
                                    hookshot.negate_x_velocity();
                                    if self.game_state.player.tile_detection.misc_tiles() & 3 == 0 {
                                        self.ancilla_add_hookshot_wall_clink(k, 6, 1);
                                        self.ancilla_sfx2_pan(
                                            k,
                                            if self.game_state.player.tile_detection.misc_tiles()
                                                & 0x30
                                                != 0
                                            {
                                                6
                                            } else {
                                                5
                                            },
                                        );
                                    }
                                }

                                if self.game_state.player.tile_detection.misc_tiles() & 3 != 0 {
                                    if self.ancilla_slot_view(k).item_to_link() < 4 {
                                        self.ancilla_slot_view_mut(k).clear();
                                        return;
                                    }
                                    self.follower_link_state_mut().set_hookshot_interlock(1);
                                    self.messaging_state_mut().set_effect_index(k as u8);
                                }
                            }
                        }
                    }
                }
            }
        }

        let (info_x, info_y) = self.ancilla_prep_oam_coord(k);
        if self.ancilla_slot_view(k).l() != 0 {
            self.oam_state_mut().set_priority_word(0x3000);
        }
        let mut oam = self.game_state.oam.current_pointer_usize();

        let mut j = self.ancilla_slot_view(k).direction() as usize * 3;
        let mut x = info_x;
        let mut y = info_y;
        for i in (0..=2).rev() {
            if ANCILLA1_F_HOOKSHOT_HOOKSHOT_DRAW_CHAR[j] != 0xff {
                self.ancilla_set_oam(
                    oam,
                    x,
                    y,
                    ANCILLA1_F_HOOKSHOT_HOOKSHOT_DRAW_CHAR[j],
                    ANCILLA1_F_HOOKSHOT_HOOKSHOT_DRAW_FLAGS[j]
                        | 2
                        | self.game_state.oam.priority_high(),
                    0,
                );
                oam += 4;
            }
            if i == 1 {
                x = x.wrapping_sub(8);
                y = y.wrapping_add(8);
            } else {
                x = x.wrapping_add(8);
            }
            j += 1;
        }

        let mut r10 = 0i32;
        let mut n = (self.ancilla_slot_view(k).item_to_link() >> 1) as i32;
        if n >= 7 {
            r10 = n - 7;
            n = 6;
        }
        if n == 0 {
            return;
        }
        if self.ancilla_slot_view(k).direction() & 1 != 0 {
            r10 = -r10;
        }
        let mut x = info_x;
        let mut y = info_y;
        let j = self.ancilla_slot_view(k).direction() as usize;
        if ANCILLA1_F_HOOKSHOT_HOOKSHOT_MOVE_Y[j] == 0 {
            y = y.wrapping_add(4);
        }
        if ANCILLA1_F_HOOKSHOT_HOOKSHOT_MOVE_X[j] == 0 {
            x = x.wrapping_add(4);
        }
        loop {
            if ANCILLA1_F_HOOKSHOT_HOOKSHOT_MOVE_Y[j] != 0 {
                y = y.wrapping_add(
                    (ANCILLA1_F_HOOKSHOT_HOOKSHOT_MOVE_Y[j] as i32 + r10) as i16 as u16,
                );
            }
            if ANCILLA1_F_HOOKSHOT_HOOKSHOT_MOVE_X[j] != 0 {
                x = x.wrapping_add(
                    (ANCILLA1_F_HOOKSHOT_HOOKSHOT_MOVE_X[j] as i32 + r10) as i16 as u16,
                );
            }
            if !self.hookshot_check_proximity_to_link(x as i32, y as i32) {
                self.ancilla_set_oam(
                    oam,
                    x,
                    y,
                    0x19,
                    (self.game_state.frame.frame_counter & 2) << 6
                        | 2
                        | self.game_state.oam.priority_high(),
                    0,
                );
                oam += 4;
            }
            n -= 1;
            if n < 0 {
                break;
            }
        }
    }

    pub(super) fn ancilla3_c_spin_attack_charge_sparkle(&mut self, k: usize) {
        // $08:C1EA..C22E: complete caller, excluding allocation/prep/XY children.
        let _scope = self.lag_free.then(|| crate::cycle_ledger::routine(0x08c1ea));
        let b = crate::cycle_ledger::blocks(self.lag_free);
        b.block(0x08c1ea);
        if self.game_state.frame.submodule != 0 { b.taken(0x08c1ec); }
        else {
            b.block(0x08c1ee);
            if self.ancilla_slot_view(k).timer() != 0 { b.taken(0x08c1f1); }
        }
        if self.game_state.frame.submodule == 0 && self.ancilla_slot_view(k).timer() == 0 {
            b.block(0x08c1f3);
            let value = 4;
            self.ancilla_slot_view_mut(k).set_timer(value);
            self.ancilla_slot_view_mut(k).add_item_to_link(1);
            if self.ancilla_slot_view(k).item_to_link() == 3 {
                b.block(0x08c202);
                let value = 0;
                self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                return;
            } else { b.taken(0x08c200); }
        }
        b.block(0x08c206);
        let value = self.ancilla_allocate_oam_from_region_a_or_d_or_f(k, 4) as u8;
        b.block(0x08c20c);
        self.ancilla_slot_view_mut(k).set_oam_index(value);
        let (x, y) = self.ancilla_prep_oam_coord(k);
        b.block(0x08c213);
        let j = self.ancilla_slot_view(k).item_to_link() as usize;
        // $08:C219 JSR F6E1: complete XY body, once per executed draw.
        self.ancilla_set_oam_marked(
            self.game_state.oam.current_pointer_usize(),
            x,
            y,
            ANCILLA3_C_SPIN_ATTACK_CHARGE_SPARKLE_SWORD_CHARGE_SPARK_CHAR[j],
            ANCILLA3_C_SPIN_ATTACK_CHARGE_SPARKLE_SWORD_CHARGE_SPARK_FLAGS[j]
                | self.game_state.oam.priority_high(),
            0,
        );
        b.block(0x08c21c);
    }

    pub(super) fn ancilla04_beam_hit(&mut self, k: usize) {
        let Some(info) = self.ancilla_return_if_outside_bounds(k) else {
            return;
        };
        if self.ancilla_slot_view(k).timer() == 0 {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
            return;
        }

        let mut oam = self.game_state.oam.current_pointer_usize();
        let j = (self.ancilla_slot_view(k).timer() >> 1) as usize;
        let ancilla_x = self.ancilla_get_x(k);
        let ancilla_y = self.ancilla_get_y(k);
        let r7 =
            ancilla_x.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2()) as u8;
        let r6 =
            ancilla_y.wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2()) as u8;
        for i in (0..=3).rev() {
            let m = j * 4 + i;
            let x = info.x.wrapping_add(ANCILLA04_BEAM_HIT_BEAM_HIT_X[m] as u8);
            let y = info.y.wrapping_add(ANCILLA04_BEAM_HIT_BEAM_HIT_Y[m] as u8);
            let x_adj = ancilla_x
                .wrapping_add(x.wrapping_sub(r7) as i8 as i16 as u16)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
            let y_adj = ancilla_y
                .wrapping_add(y.wrapping_sub(r6) as i8 as i16 as u16)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
                .wrapping_add(0x10);
            self.oam_state_mut().write_entry(
                oam,
                x,
                if y_adj >= 0x100 { 0xf0 } else { y },
                ANCILLA04_BEAM_HIT_BEAM_HIT_CHAR[m].wrapping_add(0x82),
                ANCILLA04_BEAM_HIT_BEAM_HIT_FLAGS[m] | 2 | info.flags,
            );
            let value = if x_adj >= 0x100 { 1 } else { 0 };
            self.oam_state_mut()
                .set_extended_byte((oam - OAM_BUF) / 4, value);
            oam += 4;
        }
    }

    pub(super) fn ancilla13_ice_rod_sparkle(&mut self, k: usize) {
        if self.ancilla_slot_view(k).timer() == 0 {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
        }
        if self.game_state.frame.submodule == 0 {
            self.ancilla_move_x(k);
            self.ancilla_move_y(k);
        }
        let Some(mut info) = self.ancilla_return_if_outside_bounds(k) else {
            return;
        };

        let mut j = 4i32;
        while j >= 0 && self.ancilla_slot_view(j as usize).ancilla_type() != 0x0b {
            j -= 1;
        }
        if j >= 0 && self.ancilla_slot_view(j as usize).object_priority() != 0 {
            info.flags = 0x30;
        }

        if self.game_state.oam.has_sprite_sorting() {
            if self.ancilla_slot_view(k).floor() != 0 {
                self.oam_allocate_from_region_e(0x10);
            } else {
                self.oam_allocate_from_region_d(0x10);
            }
        } else {
            self.oam_allocate_from_region_a(0x10);
        }

        let mut oam = self.game_state.oam.current_pointer_usize();
        j = (self.ancilla_slot_view(k).timer() & 0x1c) as i32;
        for i in (0..=3).rev() {
            let n = i + j as usize;
            self.oam_state_mut().write_entry(
                oam,
                info.x
                    .wrapping_add(ANCILLA13_ICE_ROD_SPARKLE_ICE_SHOT_SPARKLE_X[n]),
                info.y
                    .wrapping_add(ANCILLA13_ICE_ROD_SPARKLE_ICE_SHOT_SPARKLE_Y[n]),
                ANCILLA13_ICE_ROD_SPARKLE_ICE_SHOT_SPARKLE_CHAR[n],
                info.flags | 4,
            );
            let value = 0;
            self.oam_state_mut()
                .set_extended_byte((oam - OAM_BUF) / 4, value);
            oam += 4;
        }
    }

    fn fire_shot_draw(&mut self, k: usize) {
        let Some(mut info) = self.ancilla_return_if_outside_bounds(k) else {
            return;
        };
        if self.ancilla_slot_view(k).object_priority() != 0 {
            info.flags |= 0x30;
        }

        let mut oam = self.game_state.oam.current_pointer_usize();
        let j = (self.ancilla_slot_view(k).item_to_link() & 0x0c) as usize;
        for i in (0..=2).rev() {
            self.ancilla_set_oam_plain(
                oam,
                u16::from(info.x.wrapping_add(FIRE_SHOT_DRAW_FIRE_SHOT_DRAW_X2[j + i])),
                u16::from(info.y.wrapping_add(FIRE_SHOT_DRAW_FIRE_SHOT_DRAW_Y2[j + i])),
                FIRE_SHOT_DRAW_FIRE_SHOT_DRAW_CHAR2[i],
                info.flags | 2,
                0,
            );
            oam += 4;
        }
    }

    fn ice_shot_spread_draw(&mut self, k: usize) {
        let (info_x, info_y) = self.ancilla_prep_oam_coord(k);
        self.ancilla_allocate_oam_from_region_a_or_d_or_f(
            k,
            self.ancilla_slot_view(k).num_sprites(),
        );
        let mut oam = self.game_state.oam.current_pointer_usize();
        let oam_org = oam;
        let mut j = self.ancilla_slot_view(k).item_to_link() as usize * 4;
        for _ in 0..4 {
            let offset = ICE_SHOT_SPREAD_DRAW_ICE_SHOT_SPREAD_OFFSET[j];
            let tile = ICE_SHOT_SPREAD_DRAW_ICE_SHOT_SPREAD_TILE[j];
            let y = info_y.wrapping_add(offset.y as i16 as u16);
            let x = info_x.wrapping_add(offset.x as i16 as u16);
            let mut yv = 0xf0;
            if x < 256 && y < 256 {
                self.oam_state_mut().set_entry_x(oam, x as u8);
                if y < 224 {
                    yv = y as u8;
                }
            }
            self.oam_state_mut().set_entry_y(oam, yv);
            self.oam_state_mut().set_entry_char(oam, tile.char);
            let flags = tile.flags & !0x30 | self.game_state.oam.priority_high();
            self.oam_state_mut().set_entry_flags(oam, flags);
            let value = 0;
            self.oam_state_mut()
                .set_extended_byte((oam - OAM_BUF) / 4, value);
            oam = self.ancilla_allocate_oam_from_custom_region(oam + 4);
            j += 1;
        }
        if self.game_state.oam.entry_y(oam_org) == 0xf0
            && self.game_state.oam.entry_y(oam_org + 4) == 0xf0
        {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
        }
    }

    pub(super) fn ancilla11_ice_rod_wall_hit(&mut self, k: usize) {
        let aux_timer = self.ancilla_slot_view_mut(k).tick_aux_timer();
        if sign8(aux_timer) {
            let item_to_link = {
                let mut wall_hit = self.ancilla_slot_view_mut(k);
                wall_hit.set_aux_timer(7);
                wall_hit.advance_item_to_link()
            };
            if item_to_link == 2 {
                self.ancilla_slot_view_mut(k).clear();
                return;
            }
        }
        self.ice_shot_spread_draw(k);
    }

    pub(super) fn ancilla0_a_arrow_in_the_wall(&mut self, k: usize) {
        let j = self.ancilla_slot_view(k).s_player();
        if !sign8(j) {
            let j = j as usize;
            if self.sprite_slot_view(j).state() < 9
                || sign8(self.sprite_slot_view(j).z())
                || self.sprite_slot_view(j).ignore_projectile() != 0
                || self.sprite_slot_view(j).deflection_bits() & 2 != 0
            {
                let value = 0;
                self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                return;
            }
            self.ancilla_set_x(
                k,
                self.sprite_get_x(j)
                    .wrapping_add(self.ancilla_slot_view(k).x_velocity() as i8 as i16 as u16),
            );
            self.ancilla_set_y(
                k,
                self.sprite_get_y(j)
                    .wrapping_add(self.ancilla_slot_view(k).y_velocity() as i8 as i16 as u16)
                    .wrapping_sub(u16::from(self.sprite_slot_view(j).z())),
            );
        }
        if self.game_state.frame.submodule == 0 {
            self.ancilla_slot_view_mut(k).tick_aux_timer();
            if self.ancilla_slot_view(k).aux_timer() == 0 {
                let value = 2;
                self.ancilla_slot_view_mut(k).set_aux_timer(value);
                let value = self.ancilla_slot_view(k).item_to_link().wrapping_add(1);
                self.ancilla_slot_view_mut(k).set_item_to_link(value);
                if self.ancilla_slot_view(k).item_to_link() == 9 {
                    let value = 0;
                    self.ancilla_slot_view_mut(k).set_ancilla_type(value);
                    return;
                } else if self.ancilla_slot_view(k).item_to_link() & 8 != 0 {
                    let value = 0x80;
                    self.ancilla_slot_view_mut(k).set_aux_timer(value);
                }
            }
        }
        self.arrow_draw(k);
    }

    fn arrow_draw(&mut self, k: usize) {
        let (mut x, mut y) = self.ancilla_prep_adjusted_oam_coord(k);
        if self.ancilla_slot_view(k).object_priority() != 0 {
            self.oam_state_mut().set_priority_high(0x30);
        }
        if self.ancilla_slot_view(k).h() != 0 {
            x = x.wrapping_add(
                self.game_state
                    .display
                    .ppu_scroll_copy
                    .bg2_v_copy2()
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg1_v_copy2()),
            );
            y = y.wrapping_add(
                self.game_state
                    .display
                    .ppu_scroll_copy
                    .bg2_h_copy2()
                    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg1_h_copy2()),
            );
        }

        let r7 = self.ancilla_slot_view(k).item_to_link();
        let mut j = self.ancilla_slot_view(k).direction() & !4;
        if self.ancilla_slot_view(k).ancilla_type() == 0x0a {
            j = j
                .wrapping_mul(4)
                .wrapping_add(8)
                .wrapping_add(if r7 & 8 != 0 { 1 } else { r7 & 3 });
        } else if !sign8(r7) {
            j |= 4;
        }
        let mut j = j as usize * 2;

        let mut oam = self.game_state.oam.current_pointer_usize();
        let oam_org = oam;
        let flags = if self.game_state.inventory.items.has_silver_arrows() {
            2
        } else {
            4
        };
        for _ in 0..2 {
            if ARROW_DRAW_ARROW_DRAW_CHAR[j] != 0xff {
                // $08:A417 JSR F6E1: complete XY body, once per executed draw.
                self.ancilla_set_oam_marked(
                    oam,
                    x.wrapping_add(ARROW_DRAW_ARROW_DRAW_X[j] as i16 as u16),
                    y.wrapping_add(ARROW_DRAW_ARROW_DRAW_Y[j] as i16 as u16),
                    ARROW_DRAW_ARROW_DRAW_CHAR[j],
                    ARROW_DRAW_ARROW_DRAW_FLAGS[j] & !0x3e
                        | flags
                        | self.game_state.oam.priority_high(),
                    0,
                );
                oam += 4;
            }
            j += 1;
        }

        if self.game_state.oam.entry_y(oam_org) == 0xf0
            && self.game_state.oam.entry_y(oam_org + 4) == 0xf0
        {
            let value = 0;
            self.ancilla_slot_view_mut(k).set_ancilla_type(value);
        }
    }

    pub(super) fn sprite_create_deflected_arrow(&mut self, k: usize) {
        let value = 0;
        self.ancilla_slot_view_mut(k).set_ancilla_type(value);
        if let Some((j, _)) = self.spawn_sprite_dynamically(k, 0x1b) {
            let value = self.ancilla_slot_view(k).x_low();
            self.sprite_slot_view_mut(j).set_x_low(value);
            let value = self.ancilla_slot_view(k).x_high();
            self.sprite_slot_view_mut(j).set_x_high(value);
            let value = self.ancilla_slot_view(k).y_low();
            self.sprite_slot_view_mut(j).set_y_low(value);
            let value = self.ancilla_slot_view(k).y_high();
            self.sprite_slot_view_mut(j).set_y_high(value);
            let value = 6;
            self.sprite_slot_view_mut(j).set_state(value);
            let value = 31;
            self.sprite_slot_view_mut(j).set_delay_main(value);
            let value = self.ancilla_slot_view(k).x_velocity();
            self.sprite_slot_view_mut(j).set_x_velocity(value);
            let value = self.ancilla_slot_view(k).y_velocity();
            self.sprite_slot_view_mut(j).set_y_velocity(value);
            let value = self.game_state.player.follower_link.lower_level_state();
            self.sprite_slot_view_mut(j).set_floor(value);
            self.sprite_place_weapon_tink(j);
        }
    }

}
