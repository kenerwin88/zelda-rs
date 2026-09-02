// Methods ported from zelda3/src/overlord.c and included inside ZeldaState.

use super::*;

mod overlord_shared;
use crate::types::sign8;
use overlord_shared::*;

impl ZeldaState {
    pub(super) fn overlord_get_x(&self, k: usize) -> u16 {
        self.overlord_slot_view(k).x_low() as u16
            | ((self.overlord_slot_view(k).x_high() as u16) << 8)
    }

    pub(super) fn overlord_get_y(&self, k: usize) -> u16 {
        self.overlord_slot_view(k).y_low() as u16
            | ((self.overlord_slot_view(k).y_high() as u16) << 8)
    }

    pub(super) fn overlord_stalfos_factory(&self, _k: usize) {
        // Overlord_StalfosFactory is an unused assert-only slot in C.
        panic!("overlord_stalfos_factory assert");
    }

    pub(super) fn overlord_set_x(&mut self, k: usize, v: u16) {
        let value = v as u8;
        self.overlord_slot_view_mut(k).set_x_low(value);
        let value = (v >> 8) as u8;
        self.overlord_slot_view_mut(k).set_x_high(value);
    }

    pub(super) fn overlord_set_y(&mut self, k: usize, v: u16) {
        let value = v as u8;
        self.overlord_slot_view_mut(k).set_y_low(value);
        let value = (v >> 8) as u8;
        self.overlord_slot_view_mut(k).set_y_high(value);
    }

    pub(super) fn overlord_spawn_boulder(&mut self) {
        if self.game_state.world.location.is_indoors()
            || self.game_state.sprites.garnish_runtime.boulder_trap_count() == 0
            || (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) != 0
        {
            return;
        }
        let timer = self.garnish_state_mut().increment_boulder_trap_timer();
        if timer & 63 != 0 {
            return;
        }
        let camera_y_hi = (self.game_state.display.ppu_scroll_copy.bg2_v_copy2() >> 8) as u8;
        let coll_y_hi = self.game_state.sprites.garnish_runtime.sprcoll_y_hi();
        if sign8(camera_y_hi.wrapping_sub(coll_y_hi).wrapping_sub(2)) {
            return;
        }
        if let Some((j, _info)) = self.Sprite_SpawnDynamically(0, 0xc2) {
            let x = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_h_copy2()
                .wrapping_add((self.get_random_number() & 127) as u16)
                .wrapping_add(64);
            let y = self
                .game_state
                .display
                .ppu_scroll_copy
                .bg2_v_copy2()
                .wrapping_sub(0x30);
            self.Sprite_SetX(j, x);
            self.Sprite_SetY(j, y);
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_floor(0);
            sprite.set_direction(0);
            sprite.set_z(0);
        }
    }

    pub(super) fn overlord_main(&mut self) {
        self.overlord_execute_all();
        self.overlord_spawn_boulder();
    }

    pub(super) fn overlord_execute_all(&mut self) {
        if (self.game_state.frame.submodule | self.game_state.frame.modal_pause_flag) != 0 {
            return;
        }
        for i in (0..=7).rev() {
            if self
                .game_state
                .sprites
                .overlord_slots
                .slot(i)
                .overlord_type()
                != 0
            {
                self.overlord_execute_single(i);
            }
        }
    }

    pub(super) fn overlord_execute_single(&mut self, k: usize) {
        let j = self
            .game_state
            .sprites
            .overlord_slots
            .slot(k)
            .overlord_type();
        self.overlord_check_if_active(k);
        match j {
            1 => self.overlord01_position_target(k),
            2 => self.overlord02_full_room_cannons(k),
            3 => self.overlord03_vertical_cannon(k),
            4 => self.overlord_stalfos_factory(k),
            5 => self.overlord05_falling_stalfos(k),
            6 => self.overlord06_bad_switch_snake(k),
            7 => self.overlord07_moving_floor(k),
            8 => self.overlord08_blob_spawner(k),
            9 => self.overlord09_wallmaster_spawner(k),
            10..=15 => self.overlord0_a_falling_square(k),
            16..=19 => self.overlord10_pirogusu_spawner_left(k),
            20 => self.overlord14_tile_room(k),
            21 => self.overlord15_wizzrobe_spawner(k),
            22 => self.overlord16_zoro_spawner(k),
            23 => self.overlord17_pot_trap(k),
            24 => self.overlord18_invisible_stalfos(k),
            25 => self.overlord19_armos_coordinator_bounce(k),
            26 => self.overlord06_bad_switch_snake(k),
            _ => panic!("overlord_execute_single invalid type {j}"),
        }
    }

    pub(super) fn overlord19_armos_coordinator_bounce(&mut self, k: usize) {
        if self.overlord_slot_view(k).gen2() != 0 {
            let value = self
                .game_state
                .sprites
                .overlord_slots
                .slot(k)
                .gen2()
                .wrapping_sub(1);
            self.overlord_slot_view_mut(k).set_gen2(value);
        }
        match self.overlord_slot_view(k).gen1() {
            0 => {
                if self.sprite_slot_view(0).a() != 0 {
                    let value = 120;
                    self.overlord_slot_view_mut(k).set_x_low(value);
                    let value = 255;
                    self.overlord_slot_view_mut(k).set_floor(value);
                    let value = 64;
                    self.overlord_slot_view_mut(2).set_x_low(value);
                    self.overlord_slot_view_mut(0).set_x_low(192);
                    let value = 1;
                    self.overlord_slot_view_mut(1).set_x_low(value);
                    self.armos_coordinator_rotate_knights(k);
                }
            }
            1 => {
                if self.armos_coordinator_check_knights() {
                    let value = self
                        .game_state
                        .sprites
                        .overlord_slots
                        .slot(k)
                        .gen1()
                        .wrapping_add(1);
                    self.overlord_slot_view_mut(k).set_gen1(value);
                    let value = 0xff;
                    self.overlord_slot_view_mut(k).set_gen2(value);
                }
            }
            2 | 4 => self.armos_coordinator_rotate_knights(k),
            3 => {
                let value = self
                    .game_state
                    .sprites
                    .overlord_slots
                    .slot(2)
                    .x_low()
                    .wrapping_sub(1);
                self.overlord_slot_view_mut(2).set_x_low(value);
                if self.overlord_slot_view(2).x_low() == 32 {
                    let value = self
                        .game_state
                        .sprites
                        .overlord_slots
                        .slot(k)
                        .gen1()
                        .wrapping_add(1);
                    self.overlord_slot_view_mut(k).set_gen1(value);
                    let value = 64;
                    self.overlord_slot_view_mut(k).set_gen2(value);
                }
                self.armos_coordinator_rotate(k);
            }
            5 => {
                let value = self
                    .game_state
                    .sprites
                    .overlord_slots
                    .slot(2)
                    .x_low()
                    .wrapping_add(1);
                self.overlord_slot_view_mut(2).set_x_low(value);
                if self.overlord_slot_view(2).x_low() == 64 {
                    let value = self
                        .game_state
                        .sprites
                        .overlord_slots
                        .slot(k)
                        .gen1()
                        .wrapping_add(1);
                    self.overlord_slot_view_mut(k).set_gen1(value);
                    let value = 64;
                    self.overlord_slot_view_mut(k).set_gen2(value);
                }
                self.armos_coordinator_rotate(k);
            }
            6 => {
                if self.overlord_slot_view(k).gen2() != 0 {
                    return;
                }
                self.armos_coordinator_disable_coercion(k);
                for j in (0..=5).rev() {
                    let value = ARMOS_COORDINATOR_BACK_WALL_X_OFFSETS[j];
                    self.overlord_slot_view_mut(j).set_x_high(value);
                    let value = 48;
                    self.overlord_slot_view_mut(j).set_gen2(value);
                }
                let value = self
                    .game_state
                    .sprites
                    .overlord_slots
                    .slot(k)
                    .gen1()
                    .wrapping_add(1);
                self.overlord_slot_view_mut(k).set_gen1(value);
                let value = 255;
                self.overlord_slot_view_mut(k).set_gen2(value);
            }
            7 => {
                if self.overlord_slot_view(k).gen2() != 0 {
                    return;
                }
                for j in (0..=5).rev() {
                    let value = self
                        .game_state
                        .sprites
                        .overlord_slots
                        .slot(j)
                        .gen2()
                        .wrapping_add(1);
                    self.overlord_slot_view_mut(j).set_gen2(value);
                    if self.overlord_slot_view(j).gen2() == 192 {
                        let value = 1;
                        self.overlord_slot_view_mut(k).set_gen1(value);
                        let value = 0u8.wrapping_sub(self.overlord_slot_view(k).floor());
                        self.overlord_slot_view_mut(k).set_floor(value);
                        self.armos_coordinator_disable_coercion(k);
                        self.armos_coordinator_rotate(k);
                        return;
                    }
                }
            }
            _ => {}
        }
    }

    pub(super) fn overlord18_invisible_stalfos(&mut self, k: usize) {
        let x = self.overlord_get_x(k);
        let y = self.overlord_get_y(k);
        if x.wrapping_sub(self.game_state.player.follower_link.x())
            .wrapping_add(24)
            >= 48
            || y.wrapping_sub(self.game_state.player.follower_link.y())
                .wrapping_add(24)
                >= 48
        {
            return;
        }
        let value = 0;
        self.overlord_slot_view_mut(k).set_overlord_type(value);
        self.temp_counter_mut().set(3);
        loop {
            let i = self.game_state.scratch_counter.value() as usize;
            let Some((j, _info)) = self.Sprite_SpawnDynamicallyEx(k, 0xa7, 12) else {
                return;
            };
            self.Sprite_SetX(
                j,
                self.game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_add(INVISIBLE_STALFOS_TRAP_X_OFFSETS[i] as i16 as u16),
            );
            self.Sprite_SetY(
                j,
                self.game_state
                    .player
                    .follower_link
                    .y()
                    .wrapping_add(INVISIBLE_STALFOS_TRAP_Y_OFFSETS[i] as i16 as u16),
            );
            let floor = self.overlord_slot_view(k).floor();
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_delay_main(INVISIBLE_STALFOS_TRAP_DELAYS[i]);
            sprite.set_floor(floor);
            sprite.set_e(1);
            sprite.set_flags2(3);
            sprite.set_direction(2);
            self.temp_counter_mut().decrement();
            if sign8(self.game_state.scratch_counter.value()) {
                break;
            }
        }
    }

    pub(super) fn overlord17_pot_trap(&mut self, k: usize) {
        let x = self.overlord_get_x(k);
        let y = self.overlord_get_y(k);
        if x.wrapping_sub(self.game_state.player.follower_link.x())
            .wrapping_add(32)
            < 64
            && y.wrapping_sub(self.game_state.player.follower_link.y())
                .wrapping_add(32)
                < 64
        {
            let value = 0;
            self.overlord_slot_view_mut(k).set_overlord_type(value);
            self.dungeon_room_effects_mut()
                .increment_trap_trigger_latch();
        }
    }

    pub(super) fn overlord16_zoro_spawner(&mut self, k: usize) {
        let value = self
            .game_state
            .sprites
            .overlord_slots
            .slot(k)
            .gen2()
            .wrapping_sub(1);
        self.overlord_slot_view_mut(k).set_gen2(value);
        let x = self.overlord_get_x(k).wrapping_add(8);
        let y = self.overlord_get_y(k).wrapping_add(8);
        if self.get_tile_attribute_for_overlord(self.overlord_slot_view(k).floor(), x, y) != 0x82 {
            return;
        }
        if self.overlord_slot_view(k).gen2() >= 0x18 || self.overlord_slot_view(k).gen2() & 3 != 0 {
            return;
        }
        if let Some((j, info)) = self.Sprite_SpawnDynamicallyEx(k, 0x9c, 12) {
            let random_x = ZORO_SPAWNER_X_OFFSETS[(self.get_random_number() & 7) as usize];
            self.Sprite_SetX(
                j,
                info.r5_overlord_x
                    .wrapping_add(random_x as i16 as u16)
                    .wrapping_add(8),
            );
            let floor = self.overlord_slot_view(k).floor();
            let subtype2 = self.get_random_number();
            let y = (info.r7_overlord_y & 0xff00) | info.r7_overlord_y.wrapping_add(8) & 0x00ff;
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_y(y);
            sprite.set_floor(floor);
            sprite.set_flags4(1);
            sprite.set_e(1);
            sprite.set_ignore_projectile(1);
            sprite.set_y_velocity(16);
            sprite.set_flags2(32);
            sprite.set_oam_flags(13);
            sprite.set_subtype2(subtype2);
            sprite.set_delay_main(48);
            sprite.set_bump_damage(3);
        }
    }

    pub(super) fn overlord15_wizzrobe_spawner(&mut self, k: usize) {
        if self.overlord_slot_view(k).gen2() != 128 {
            if self.game_state.frame.frame_counter & 1 != 0 {
                let value = self
                    .game_state
                    .sprites
                    .overlord_slots
                    .slot(k)
                    .gen2()
                    .wrapping_sub(1);
                self.overlord_slot_view_mut(k).set_gen2(value);
            }
            return;
        }
        let value = 127;
        self.overlord_slot_view_mut(k).set_gen2(value);
        for i in (0..=3).rev() {
            if let Some((j, _info)) = self.Sprite_SpawnDynamicallyEx(k, 0x9b, 12) {
                self.Sprite_SetX(
                    j,
                    self.game_state
                        .player
                        .follower_link
                        .x()
                        .wrapping_add(WIZZROBE_SPAWNER_X_OFFSETS[i] as i16 as u16),
                );
                self.Sprite_SetY(
                    j,
                    self.game_state
                        .player
                        .follower_link
                        .y()
                        .wrapping_add(WIZZROBE_SPAWNER_Y_OFFSETS[i] as i16 as u16),
                );
                let floor = self.overlord_slot_view(k).floor();
                let mut sprite = self.sprite_slot_view_mut(j);
                sprite.set_delay_main(WIZZROBE_SPAWNER_DELAYS[i]);
                sprite.set_floor(floor);
                sprite.set_b(1);
            }
        }
        self.temp_counter_mut().set(0xff);
    }

    pub(super) fn overlord14_tile_room(&mut self, k: usize) {
        let x = self
            .overlord_get_x(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let y = self
            .overlord_get_y(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        if x & 0xff00 != 0 || y & 0xff00 != 0 {
            return;
        }
        let value = self
            .game_state
            .sprites
            .overlord_slots
            .slot(k)
            .gen2()
            .wrapping_sub(1);
        self.overlord_slot_view_mut(k).set_gen2(value);
        if self.overlord_slot_view(k).gen2() != 0x80 {
            return;
        }
        if self.tile_room_spawn_tile(k) < 0 {
            let value = 0x81;
            self.overlord_slot_view_mut(k).set_gen2(value);
            return;
        }
        let value = self
            .game_state
            .sprites
            .overlord_slots
            .slot(k)
            .gen1()
            .wrapping_add(1);
        self.overlord_slot_view_mut(k).set_gen1(value);
        if self.overlord_slot_view(k).gen1() != 22 {
            let value = 0xe0;
            self.overlord_slot_view_mut(k).set_gen2(value);
        } else {
            let value = 0;
            self.overlord_slot_view_mut(k).set_overlord_type(value);
        }
    }

    pub(super) fn tile_room_spawn_tile(&mut self, k: usize) -> i32 {
        let Some((j, _info)) = self.Sprite_SpawnDynamically(k, 0x94) else {
            return -1;
        };
        let i = self.overlord_slot_view(k).gen1() as usize;
        let x = TILE_ROOM_FLYING_TILE_X_POSITIONS[i] as u16
            | ((self.overlord_slot_view(k).x_high() as u16) << 8);
        let y = TILE_ROOM_FLYING_TILE_Y_POSITIONS[i].wrapping_sub(8) as u16
            | ((self.overlord_slot_view(k).y_high() as u16) << 8);
        let floor = self.overlord_slot_view(k).floor();
        let mut sprite = self.sprite_slot_view_mut(j);
        sprite.set_e(1);
        sprite.set_x(x);
        sprite.set_y(y);
        sprite.set_floor(floor);
        sprite.set_health(4);
        sprite.set_flags5(0);
        sprite.set_health(0);
        sprite.set_deflection_bits(8);
        sprite.set_flags2(4);
        sprite.set_oam_flags(1);
        sprite.set_bump_damage(4);
        j as i32
    }

    pub(super) fn overlord10_pirogusu_spawner_left(&mut self, k: usize) {
        let counter = self
            .game_state
            .sprites
            .overlord_slots
            .slot(k)
            .overlord_type()
            .wrapping_sub(16);
        self.temp_counter_mut().set(counter);
        if self.overlord_slot_view(k).gen2() != 128 {
            let value = self
                .game_state
                .sprites
                .overlord_slots
                .slot(k)
                .gen2()
                .wrapping_sub(1);
            self.overlord_slot_view_mut(k).set_gen2(value);
            return;
        }
        let value = (self.get_random_number() & 31).wrapping_add(96);
        self.overlord_slot_view_mut(k).set_gen2(value);
        let mut n = 0;
        for i in 0..16 {
            let sprite = self.sprite_slot_view(i);
            if sprite.state() != 0 && sprite.sprite_type() == 0x10 {
                n += 1;
            }
        }
        if n >= 5 {
            return;
        }
        if let Some((j, info)) = self.Sprite_SpawnDynamicallyEx(k, 0x94, 12) {
            self.Sprite_SetX(j, info.r5_overlord_x);
            self.Sprite_SetY(j, info.r7_overlord_y);
            let floor = self.overlord_slot_view(k).floor();
            let direction = self.game_state.scratch_counter.value();
            let a = PIROGUSU_SPAWNER_DIRECTIONS[direction as usize];
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_floor(floor);
            sprite.set_delay_main(32);
            sprite.set_direction(direction);
            sprite.set_a(a);
        }
    }

    pub(super) fn overlord0_a_falling_square(&mut self, k: usize) {
        if self.overlord_slot_view(k).gen2() != 0 {
            if self.overlord_slot_view(k).gen3() != 0 {
                let value = self
                    .game_state
                    .sprites
                    .overlord_slots
                    .slot(k)
                    .gen2()
                    .wrapping_sub(1);
                self.overlord_slot_view_mut(k).set_gen2(value);
                return;
            }
            let x = self
                .overlord_get_x(k)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
            let y = self
                .overlord_get_y(k)
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
            if !(x & 0xff00 != 0 || y & 0xff00 != 0) {
                let value = self
                    .game_state
                    .sprites
                    .overlord_slots
                    .slot(k)
                    .gen3()
                    .wrapping_add(1);
                self.overlord_slot_view_mut(k).set_gen3(value);
            }
            return;
        }
        let value = 16;
        self.overlord_slot_view_mut(k).set_gen2(value);
        self.spawn_falling_tile(k);
        let j = self
            .game_state
            .sprites
            .overlord_slots
            .slot(k)
            .overlord_type()
            .wrapping_sub(10) as usize;
        let i = self.overlord_slot_view(k).gen1() as usize;
        let value = self
            .game_state
            .sprites
            .overlord_slots
            .slot(k)
            .gen1()
            .wrapping_add(1);
        self.overlord_slot_view_mut(k).set_gen1(value);
        if i == (FALLING_SQUARE_CRUMBLE_PATH_OFFSETS[j + 1]
            - FALLING_SQUARE_CRUMBLE_PATH_OFFSETS[j]) as usize
        {
            let value = 0;
            self.overlord_slot_view_mut(k).set_overlord_type(value);
        }
        let t = FALLING_SQUARE_CRUMBLE_PATH_DIRECTIONS
            [FALLING_SQUARE_CRUMBLE_PATH_OFFSETS[j] as usize + i];
        if t == 0xff {
            self.overlord_set_x(k, self.overlord_get_x(k).wrapping_add(0x0c1a));
            self.overlord_set_y(k, self.overlord_get_y(k).wrapping_add(0xbb66));
        } else {
            self.overlord_set_x(
                k,
                self.overlord_get_x(k)
                    .wrapping_add(FALLING_SQUARE_CRUMBLE_X_DELTAS[t as usize] as i16 as u16),
            );
            self.overlord_set_y(
                k,
                self.overlord_get_y(k)
                    .wrapping_add(FALLING_SQUARE_CRUMBLE_Y_DELTAS[t as usize] as i16 as u16),
            );
        }
    }

    pub(super) fn spawn_falling_tile(&mut self, k: usize) {
        if let Some(j) = self.GarnishAlloc() {
            let x_high = self.overlord_slot_view(k).x_high();
            let x_low = self.overlord_slot_view(k).x_low();
            {
                let mut garnish = self.garnish_slot_view_mut(j);
                garnish.set_garnish_type(3);
                garnish.set_x_high(x_high);
                garnish.set_x_low(x_low);
            }
            let sfx = self.calculate_sfx_pan_arbitrary(x_low) | 0x1f;
            self.set_sound_effect_1(sfx);
            let y = self.overlord_get_y(k).wrapping_add(16);
            let mut garnish = self.garnish_slot_view_mut(j);
            garnish.set_y_low(y as u8);
            garnish.set_y_high((y >> 8) as u8);
            garnish.set_countdown(31);
            self.garnish_state_mut().set_active_type(31);
        }
    }

    pub(super) fn overlord09_wallmaster_spawner(&mut self, k: usize) {
        if self.overlord_slot_view(k).gen2() != 128 {
            if self.game_state.frame.frame_counter & 1 == 0 {
                let value = self
                    .game_state
                    .sprites
                    .overlord_slots
                    .slot(k)
                    .gen2()
                    .wrapping_sub(1);
                self.overlord_slot_view_mut(k).set_gen2(value);
            }
            return;
        }
        let value = 127;
        self.overlord_slot_view_mut(k).set_gen2(value);
        let Some((j, _info)) = self.Sprite_SpawnDynamicallyEx(k, 0x90, 12) else {
            return;
        };
        self.Sprite_SetX(j, self.game_state.player.follower_link.x());
        self.Sprite_SetY(j, self.game_state.player.follower_link.y());
        self.sprite_slot_view_mut(j).set_z(208);
        self.sprite_sfx_queue_sfx2_with_pan(j, 0x20);
        let floor = self.game_state.player.follower_link.lower_level_state();
        self.sprite_slot_view_mut(j).set_floor(floor);
    }

    pub(super) fn overlord08_blob_spawner(&mut self, k: usize) {
        if self.overlord_slot_view(k).gen2() != 0 {
            let value = self
                .game_state
                .sprites
                .overlord_slots
                .slot(k)
                .gen2()
                .wrapping_sub(1);
            self.overlord_slot_view_mut(k).set_gen2(value);
            return;
        }
        let value = 0xa0;
        self.overlord_slot_view_mut(k).set_gen2(value);
        let mut n = 0;
        for i in 0..16 {
            let sprite = self.sprite_slot_view(i);
            if sprite.state() != 0 && sprite.sprite_type() == 0x8f {
                n += 1;
            }
        }
        if n >= 5 {
            return;
        }
        if let Some((j, _info)) = self.Sprite_SpawnDynamicallyEx(k, 0x8f, 12) {
            let i = self.game_state.player.follower_link.facing_index();
            self.Sprite_SetX(
                j,
                self.game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_add(BLOB_SPAWNER_ZOL_X_OFFSETS[i] as i16 as u16),
            );
            self.Sprite_SetY(
                j,
                self.game_state
                    .player
                    .follower_link
                    .y()
                    .wrapping_add(BLOB_SPAWNER_ZOL_Y_OFFSETS[i] as i16 as u16),
            );
            let floor = self.game_state.player.follower_link.lower_level_state();
            let head_direction = self.get_random_number() & 31 | 16;
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_z(192);
            sprite.set_floor(floor);
            sprite.set_ai_state(2);
            sprite.set_e(2);
            sprite.set_c(2);
            sprite.set_head_direction(head_direction);
        }
    }

    pub(super) fn overlord07_moving_floor(&mut self, k: usize) {
        if self.sprite_slot_view(0).state() == 4 {
            let value = 0;
            self.overlord_slot_view_mut(k).set_overlord_type(value);
            self.dungeon_moving_floor_mut().set_floor_move_flags(1);
            return;
        }

        if self.overlord_slot_view(k).gen1() == 0 {
            let value = self
                .game_state
                .sprites
                .overlord_slots
                .slot(k)
                .gen2()
                .wrapping_add(1);
            self.overlord_slot_view_mut(k).set_gen2(value);
            if self.overlord_slot_view(k).gen2() == 32 {
                let value = 0;
                self.overlord_slot_view_mut(k).set_gen2(value);
                let mask = if self.overlord_slot_view(k).x_low() != 0 {
                    3
                } else {
                    1
                };
                let flags = (self.get_random_number() & mask) * 2;
                self.dungeon_moving_floor_mut()
                    .set_floor_move_flags(u16::from(flags));
                // ROM Garnish03_FallingTile/overlord gen2 $09:BD6E: `JSL GetRandomNumber : AND : ADC` consumes the RNG carry-out (the C port drops it).
                let value = self.get_random_number_with_carry().masked_adc(127, 128);
                self.overlord_slot_view_mut(k).set_gen2(value);
                let value = self
                    .game_state
                    .sprites
                    .overlord_slots
                    .slot(k)
                    .gen1()
                    .wrapping_add(1);
                self.overlord_slot_view_mut(k).set_gen1(value);
            } else {
                self.dungeon_moving_floor_mut().set_floor_move_flags(1);
            }
        } else {
            let value = self
                .game_state
                .sprites
                .overlord_slots
                .slot(k)
                .gen2()
                .wrapping_sub(1);
            self.overlord_slot_view_mut(k).set_gen2(value);
            if self.overlord_slot_view(k).gen2() == 0 {
                let value = 0;
                self.overlord_slot_view_mut(k).set_gen1(value);
            }
        }
    }

    pub(super) fn sprite_overlord_play_falling_sfx(&mut self, k: usize) {
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
    }

    pub(super) fn overlord05_falling_stalfos(&mut self, k: usize) {
        let x = self
            .overlord_get_x(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let y = self
            .overlord_get_y(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        if x & 0xff00 != 0 || y & 0xff00 != 0 {
            return;
        }
        if self.overlord_slot_view(k).gen1() == 0 {
            if self.game_state.dungeon.room_effects.trap_trigger_latch() != 0 {
                let value = self
                    .game_state
                    .sprites
                    .overlord_slots
                    .slot(k)
                    .gen1()
                    .wrapping_add(1);
                self.overlord_slot_view_mut(k).set_gen1(value);
            }
            return;
        }
        let old = self.overlord_slot_view(k).gen1();
        let value = self
            .game_state
            .sprites
            .overlord_slots
            .slot(k)
            .gen1()
            .wrapping_add(1);
        self.overlord_slot_view_mut(k).set_gen1(value);
        if old == FALLING_STALFOS_TRAP_TRIGGER_TIMERS[k] {
            let value = 0;
            self.overlord_slot_view_mut(k).set_overlord_type(value);
            let Some((j, info)) = self.Sprite_SpawnDynamicallyEx(k, 0x85, 12) else {
                return;
            };
            self.Sprite_SetX(j, info.r5_overlord_x);
            self.Sprite_SetY(j, info.r7_overlord_y);
            let floor = self.overlord_slot_view(k).floor();
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_z(224);
            sprite.set_floor(floor);
            // The vanilla ROM leaves sprite_D uninitialized here: the spawn
            // already copied sprite_D[k] with the OVERLORD index aliased into
            // the sprite table (oracle D=0x24 at route frame 90920). The C
            // port's `sprite_D[j] = 0` bug-fix is not ROM behavior.
            self.sprite_overlord_play_falling_sfx(j);
        }
    }

    pub(super) fn overlord06_bad_switch_snake(&mut self, k: usize) {
        let a = self.overlord_slot_view(k).gen1();
        if a == 0 {
            if self
                .game_state
                .dungeon
                .room_effects
                .has_bomb_trap_activation()
            {
                let value = 1;
                self.overlord_slot_view_mut(k).set_gen1(value);
            }
            return;
        }
        let value = a.wrapping_add(1);
        self.overlord_slot_view_mut(k).set_gen1(value);
        if a != SNAKE_TRAP_SPAWN_TIMERS_BY_SLOT[k] {
            return;
        }
        let Some((j, info)) = self.Sprite_SpawnDynamically(k, 0x6e) else {
            return;
        };
        self.Sprite_SetX(j, info.r5_overlord_x);
        self.Sprite_SetY(j, info.r7_overlord_y);
        let floor = self.overlord_slot_view(k).floor();
        {
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_z(192);
            sprite.set_e(192);
            sprite.or_flags3(0x10);
            sprite.set_floor(floor);
        }
        self.sprite_sfx_queue_sfx2_with_pan(j, 0x20);
        let sprite_type = self
            .game_state
            .sprites
            .overlord_slots
            .slot(k)
            .overlord_type();
        let value = 0;
        self.overlord_slot_view_mut(k).set_overlord_type(value);
        if sprite_type == 26 {
            self.sprite_slot_view_mut(j).set_sprite_type(74);
            self.Sprite_TransmuteToBomb(j);
            self.sprite_slot_view_mut(j).set_delay_aux1(112);
        }
    }

    pub(super) fn overlord02_full_room_cannons(&mut self, k: usize) {
        let x = self
            .overlord_get_x(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        let y = self
            .overlord_get_y(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2());
        if (x | y) & 0xff00 != 0 || self.game_state.frame.frame_counter & 0x0f != 0 {
            return;
        }
        self.sprite_workspace_mut().set_shared_scratch_a(0);
        let j = (self.get_random_number() & 15) as usize;
        self.temp_counter_mut()
            .set(FULL_ROOM_CANNON_BALL_DIRECTIONS[j]);
        let value = FULL_ROOM_CANNON_BALL_X_POSITIONS[j];
        self.overlord_slot_view_mut(k).set_x_low(value);
        let value = self.game_state.sprites.workspace.room_origin_x_high();
        self.overlord_slot_view_mut(k).set_x_high(value);
        let value = FULL_ROOM_CANNON_BALL_Y_POSITIONS[j];
        self.overlord_slot_view_mut(k).set_y_low(value);
        let value = self
            .game_state
            .sprites
            .workspace
            .room_origin_y_high()
            .wrapping_add(1);
        self.overlord_slot_view_mut(k).set_y_high(value);
        self.overlord_spawn_cannon_ball(k, 0);
    }

    pub(super) fn overlord03_vertical_cannon(&mut self, k: usize) {
        let x = self
            .overlord_get_x(k)
            .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2());
        if x & 0xff00 != 0 {
            let value = 255;
            self.overlord_slot_view_mut(k).set_gen2(value);
            return;
        }
        if self.game_state.frame.frame_counter & 1 == 0 && self.overlord_slot_view(k).gen2() != 0 {
            let value = self
                .game_state
                .sprites
                .overlord_slots
                .slot(k)
                .gen2()
                .wrapping_sub(1);
            self.overlord_slot_view_mut(k).set_gen2(value);
        }
        self.temp_counter_mut().set(2);
        self.sprite_workspace_mut().set_shared_scratch_a(0);
        let value = self
            .game_state
            .sprites
            .overlord_slots
            .slot(k)
            .gen1()
            .wrapping_sub(1);
        self.overlord_slot_view_mut(k).set_gen1(value);
        if !sign8(self.overlord_slot_view(k).gen1()) {
            return;
        }
        let value = 56;
        self.overlord_slot_view_mut(k).set_gen1(value);
        let xd = if self.overlord_slot_view(k).gen2() == 0 {
            let value = 160;
            self.overlord_slot_view_mut(k).set_gen2(value);
            self.sprite_workspace_mut().set_shared_scratch_a(160);
            8
        } else {
            ((self.get_random_number() & 2) * 8) as i32
        };
        self.overlord_spawn_cannon_ball(k, xd);
    }

    pub(super) fn overlord_spawn_cannon_ball(&mut self, k: usize, xd: i32) {
        let Some((j, info)) = self.Sprite_SpawnDynamically(k, 0x50) else {
            return;
        };
        self.Sprite_SetX(j, info.r5_overlord_x.wrapping_add(xd as i16 as u16));
        self.Sprite_SetY(j, info.r7_overlord_y.wrapping_sub(1));
        let counter = self.game_state.scratch_counter.value() as usize;
        let floor = self.overlord_slot_view(k).floor();
        let mut sprite = self.sprite_slot_view_mut(j);
        sprite.set_x_velocity(CANNON_BALL_X_VELOCITIES[counter] as u8);
        sprite.set_y_velocity(CANNON_BALL_Y_VELOCITIES[counter] as u8);
        sprite.set_floor(floor);
        if self.game_state.sprites.workspace.shared_scratch_a() != 0 {
            let scratch = self.game_state.sprites.workspace.shared_scratch_a();
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_ai_state(scratch);
            sprite.add_y_low(8);
            sprite.set_flags2(3);
            sprite.set_flags4(9);
        }
        self.sprite_slot_view_mut(j).set_delay_aux2(64);
        self.sprite_sfx_queue_sfx3_with_pan(j, 0x07);
    }

    pub(super) fn overlord01_position_target(&mut self, k: usize) {
        self.garnish_state_mut().set_active_overlord_index(k as u8);
    }

    pub(super) fn overlord_check_if_active(&mut self, k: usize) {
        if self.game_state.world.location.is_indoors() {
            return;
        }
        let j = (self.game_state.frame.frame_counter & 1) as usize;
        let x = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_h_copy2()
            .wrapping_add(OVERLORD_ACTIVE_RANGE_OFFSETS[j])
            .wrapping_sub(self.overlord_get_x(k));
        let y = self
            .game_state
            .display
            .ppu_scroll_copy
            .bg2_v_copy2()
            .wrapping_add(OVERLORD_ACTIVE_RANGE_OFFSETS[j])
            .wrapping_sub(self.overlord_get_y(k));
        if ((x >> 15) as usize) != j || ((y >> 15) as usize) != j {
            let value = 0;
            self.overlord_slot_view_mut(k).set_overlord_type(value);
            let blk = self
                .game_state
                .sprites
                .overlord_slots
                .slot(k)
                .sprite_block_pos();
            if blk != 0xffff {
                let loadedmask = 0x80 >> (blk & 7);
                self.clear_overworld_sprite_loaded_mask(blk, loadedmask as u8);
            }
        }
    }

    pub(super) fn armos_coordinator_rotate_knights(&mut self, k: usize) {
        if self.overlord_slot_view(k).gen2() == 0 {
            let value = self
                .game_state
                .sprites
                .overlord_slots
                .slot(k)
                .gen1()
                .wrapping_add(1);
            self.overlord_slot_view_mut(k).set_gen1(value);
        }
        self.armos_coordinator_rotate(k);
    }

    pub(super) fn armos_coordinator_rotate(&mut self, k: usize) {
        let angle = self
            .game_state
            .sprites
            .overlord_slots
            .slot(0)
            .adjacent_x_low_word()
            .wrapping_add(self.overlord_slot_view(k).floor() as i8 as i16 as u16);
        self.overlord_slot_view_mut(0)
            .set_adjacent_x_low_word(angle);

        for (i, offset) in ARMOS_KNIGHT_RING_ANGLE_OFFSETS.iter().enumerate() {
            let t0 = angle.wrapping_add(*offset);
            let size = self.overlord_slot_view(2).x_low();
            let tx = self
                .overlord_get_x(k)
                .wrapping_add(armos_sin(t0, size) as i16 as u16);
            let value = tx as u8;
            self.overlord_slot_view_mut(i).set_x_high(value);
            let value = (tx >> 8) as u8;
            self.overlord_slot_view_mut(i).set_y_high(value);
            let ty = self
                .overlord_get_y(k)
                .wrapping_add(armos_sin(t0.wrapping_add(0x80), size) as i16 as u16);
            let value = ty as u8;
            self.overlord_slot_view_mut(i).set_gen2(value);
            let value = (ty >> 8) as u8;
            self.overlord_slot_view_mut(i).set_floor(value);
        }
        self.temp_counter_mut().set(6);
    }

    pub(super) fn armos_coordinator_check_knights(&self) -> bool {
        for j in (0..=5).rev() {
            let sprite = self.sprite_slot_view(j);
            if sprite.state() != 0 && sprite.ai_state() == 0 {
                return false;
            }
        }
        true
    }

    pub(super) fn armos_coordinator_disable_coercion(&mut self, _k: usize) {
        for j in (0..=5).rev() {
            self.sprite_slot_view_mut(j).set_ai_state(0);
        }
    }

    fn Sprite_SpawnDynamically(
        &mut self,
        k: usize,
        sprite: u8,
    ) -> Option<(usize, super::sprite::SpriteSpawnInfo)> {
        self.Sprite_SpawnDynamicallyEx(k, sprite, 15)
    }

    fn Sprite_SpawnDynamicallyEx(
        &mut self,
        k: usize,
        sprite: u8,
        j_in: i32,
    ) -> Option<(usize, super::sprite::SpriteSpawnInfo)> {
        let mut info = super::sprite::SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically_ex(k, sprite, &mut info, j_in);
        if j >= 0 {
            Some((j as usize, info))
        } else {
            None
        }
    }

    fn Sprite_SetX(&mut self, k: usize, x: u16) {
        self.sprite_slot_view_mut(k).set_x(x);
    }

    fn Sprite_SetY(&mut self, k: usize, y: u16) {
        self.sprite_slot_view_mut(k).set_y(y);
    }

    fn Sprite_GetX(&self, k: usize) -> u16 {
        self.sprite_slot_view(k).x()
    }

    fn GarnishAlloc(&self) -> Option<usize> {
        (0..30)
            .rev()
            .find(|&j| self.garnish_slot_view(j).is_empty())
    }

    fn Sprite_TransmuteToBomb(&mut self, k: usize) {
        let mut sprite = self.sprite_slot_view_mut(k);
        sprite.set_sprite_type(0x4a);
        sprite.set_c(1);
        sprite.set_delay_aux1(255);
        sprite.set_flags3(0x18);
        sprite.set_oam_flags(8);
        sprite.set_health(0);
    }

    fn get_tile_attribute_for_overlord(&mut self, floor: u8, x: u16, y: u16) -> u8 {
        let tiletype = if self.game_state.world.location.is_indoors() {
            let mut t = if floor >= 1 { 0x1000 } else { 0 };
            t += ((x & 0x01f8) >> 3) as usize;
            t += ((y & 0x01f8) << 3) as usize;
            self.game_state.dungeon.bg2_attributes.bg2_attr(t)
        } else {
            self.overworld_get_tile_attribute_at_location(x >> 3, y)
        };
        self.sprite_workspace_mut().set_tile_type(tiletype);
        tiletype
    }
}

fn armos_mult(a: u16, b: u8) -> u8 {
    if a >= 256 {
        return b;
    }
    let p = a * b as u16;
    ((p >> 8) + ((p >> 7) & 1)) as u8
}

fn armos_sin(a: u16, b: u8) -> i8 {
    let t = armos_mult(ARMOS_SINE_LOOKUP_TABLE[(a & 0xff) as usize], b);
    if a & 0x100 != 0 {
        (0u8.wrapping_sub(t)) as i8
    } else {
        t as i8
    }
}
