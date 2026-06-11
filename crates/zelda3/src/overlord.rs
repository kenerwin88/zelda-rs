// Methods ported from zelda3/src/overlord.c and included inside ZeldaState.

use super::*;
use crate::types::sign8;

const OVERLORD_X_HI: usize = 0x0b10;
const OVERLORD_Y_LO: usize = 0x0b18;
const OVERLORD_Y_HI: usize = 0x0b20;
const OVERLORD_GEN1: usize = 0x0b28;
const OVERLORD_GEN2: usize = 0x0b30;
const OVERLORD_GEN3: usize = 0x0b38;
const OVERLORD_FLOOR: usize = 0x0b40;
const OVERLORD_OFFSET_SPRITE_POS: usize = 0x0b48;
const SPRITE_BUMP_DAMAGE: usize = 0x0cd2;
const ACTIVATE_BOMB_TRAP_OVERLORD: usize = 0x0cf4;
const SPRITE_AI_STATE: usize = 0x0d80;
const SPRITE_B: usize = 0x0da0;
const SPRITE_C: usize = 0x0db0;
const SPRITE_STATE: usize = 0x0dd0;
const SPRITE_DELAY_AUX1: usize = 0x0e00;
const SPRITE_DELAY_AUX2: usize = 0x0e10;
const SPRITE_HEALTH: usize = 0x0e50;
const GARNISH_ACTIVE: usize = 0x0fb4;
const SPRITE_TILETYPE: usize = 0x0fa5;
const SPRCOLL_Y_BASE: usize = 0x0fbe;
const ACTIVE_OVERLORD_INDEX: usize = 0x0fde;
const DUNG_FLOOR_MOVE_FLAGS: usize = 0x041a;
const GARNISH_Y_LO: usize = 0x1f81e;
const GARNISH_X_LO: usize = 0x1f83c;
const GARNISH_Y_HI: usize = 0x1f85a;
const GARNISH_X_HI: usize = 0x1f878;
const GARNISH_COUNTDOWN: usize = 0x1f90e;

const ARMOS_SINE_LOOKUP_TABLE: [u16; 256] = [
    0, 3, 6, 9, 12, 15, 18, 21, 25, 28, 31, 34, 37, 40, 40, 46, 49, 53, 56, 59, 62, 65, 68, 71, 74,
    77, 80, 83, 86, 89, 92, 95, 97, 100, 103, 106, 109, 112, 115, 117, 120, 123, 126, 128, 131,
    134, 136, 139, 142, 144, 147, 149, 152, 155, 157, 159, 162, 164, 167, 169, 171, 174, 176, 178,
    181, 183, 185, 187, 189, 191, 193, 195, 197, 199, 201, 203, 205, 207, 209, 211, 212, 214, 216,
    217, 219, 221, 222, 224, 225, 227, 228, 230, 231, 232, 234, 235, 236, 237, 238, 239, 241, 242,
    243, 244, 244, 245, 246, 247, 248, 249, 249, 250, 251, 251, 252, 252, 253, 253, 254, 254, 254,
    255, 255, 255, 255, 255, 255, 255, 256, 255, 255, 255, 255, 255, 255, 255, 254, 254, 254, 253,
    253, 252, 252, 251, 251, 250, 249, 249, 248, 247, 246, 245, 244, 244, 243, 242, 241, 239, 238,
    237, 236, 235, 234, 232, 231, 230, 228, 227, 225, 224, 222, 221, 219, 217, 216, 214, 212, 211,
    209, 207, 205, 203, 201, 199, 197, 195, 193, 191, 189, 187, 185, 183, 181, 178, 176, 174, 171,
    169, 167, 164, 162, 159, 157, 155, 152, 149, 147, 144, 142, 139, 136, 134, 131, 128, 126, 123,
    120, 117, 115, 112, 109, 106, 103, 100, 97, 95, 92, 89, 86, 83, 80, 77, 74, 71, 68, 65, 62, 59,
    56, 53, 49, 46, 43, 40, 37, 34, 31, 28, 25, 21, 18, 15, 12, 9, 6, 3,
];

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
        if self.world_state_view().is_indoors()
            || self.garnish_state_view().boulder_trap_count() == 0
            || (self.frame_control_view().submodule()
                | self.frame_control_view().modal_pause_flag())
                != 0
        {
            return;
        }
        let timer = self.garnish_state_view_mut().increment_boulder_trap_timer();
        if timer & 63 != 0 {
            return;
        }
        let camera_y_hi = (self.world_state_view().bg2_y() >> 8) as u8;
        let coll_y_hi = self.garnish_state_view().sprcoll_y_hi();
        if sign8(camera_y_hi.wrapping_sub(coll_y_hi).wrapping_sub(2)) {
            return;
        }
        if let Some((j, _info)) = self.Sprite_SpawnDynamically(0, 0xc2) {
            let x = self
                .world_state_view()
                .bg2_x()
                .wrapping_add((self.get_random_number() & 127) as u16)
                .wrapping_add(64);
            let y = self.world_state_view().bg2_y().wrapping_sub(0x30);
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
        if (self.frame_control_view().submodule() | self.frame_control_view().modal_pause_flag())
            != 0
        {
            return;
        }
        for i in (0..=7).rev() {
            if self.overlord_slot_view(i).overlord_type() != 0 {
                self.overlord_execute_single(i);
            }
        }
    }

    pub(super) fn overlord_execute_single(&mut self, k: usize) {
        let j = self.overlord_slot_view(k).overlord_type();
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
        const ARMOS_COORDINATOR_BACK_WALL_X: [u8; 6] = [49, 77, 105, 131, 159, 187];
        if self.overlord_slot_view(k).gen2() != 0 {
            let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
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
                    let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
                    self.overlord_slot_view_mut(k).set_gen1(value);
                    let value = 0xff;
                    self.overlord_slot_view_mut(k).set_gen2(value);
                }
            }
            2 | 4 => self.armos_coordinator_rotate_knights(k),
            3 => {
                let value = self.overlord_slot_view(2).x_low().wrapping_sub(1);
                self.overlord_slot_view_mut(2).set_x_low(value);
                if self.overlord_slot_view(2).x_low() == 32 {
                    let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
                    self.overlord_slot_view_mut(k).set_gen1(value);
                    let value = 64;
                    self.overlord_slot_view_mut(k).set_gen2(value);
                }
                self.armos_coordinator_rotate(k);
            }
            5 => {
                let value = self.overlord_slot_view(2).x_low().wrapping_add(1);
                self.overlord_slot_view_mut(2).set_x_low(value);
                if self.overlord_slot_view(2).x_low() == 64 {
                    let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
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
                    let value = ARMOS_COORDINATOR_BACK_WALL_X[j];
                    self.overlord_slot_view_mut(j).set_x_high(value);
                    let value = 48;
                    self.overlord_slot_view_mut(j).set_gen2(value);
                }
                let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
                self.overlord_slot_view_mut(k).set_gen1(value);
                let value = 255;
                self.overlord_slot_view_mut(k).set_gen2(value);
            }
            7 => {
                if self.overlord_slot_view(k).gen2() != 0 {
                    return;
                }
                for j in (0..=5).rev() {
                    let value = self.overlord_slot_view(j).gen2().wrapping_add(1);
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
        const RED_STALFOS_TRAP_X: [i8; 4] = [0, 0, -48, 48];
        const RED_STALFOS_TRAP_Y: [i8; 4] = [-40, 56, 8, 8];
        const RED_STALFOS_TRAP_DELAY: [u8; 4] = [0x30, 0x50, 0x70, 0x90];
        let x = self.overlord_get_x(k);
        let y = self.overlord_get_y(k);
        if x.wrapping_sub(self.player_state_view().x())
            .wrapping_add(24)
            >= 48
            || y.wrapping_sub(self.player_state_view().y())
                .wrapping_add(24)
                >= 48
        {
            return;
        }
        let value = 0;
        self.overlord_slot_view_mut(k).set_overlord_type(value);
        self.temp_counter_view_mut().set(3);
        loop {
            let i = self.temp_counter_view().value() as usize;
            let Some((j, _info)) = self.Sprite_SpawnDynamicallyEx(k, 0xa7, 12) else {
                return;
            };
            self.Sprite_SetX(
                j,
                self.player_state_view()
                    .x()
                    .wrapping_add(RED_STALFOS_TRAP_X[i] as i16 as u16),
            );
            self.Sprite_SetY(
                j,
                self.player_state_view()
                    .y()
                    .wrapping_add(RED_STALFOS_TRAP_Y[i] as i16 as u16),
            );
            let floor = self.overlord_slot_view(k).floor();
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_delay_main(RED_STALFOS_TRAP_DELAY[i]);
            sprite.set_floor(floor);
            sprite.set_e(1);
            sprite.set_flags2(3);
            sprite.set_direction(2);
            self.temp_counter_view_mut().decrement();
            if sign8(self.temp_counter_view().value()) {
                break;
            }
        }
    }

    pub(super) fn overlord17_pot_trap(&mut self, k: usize) {
        let x = self.overlord_get_x(k);
        let y = self.overlord_get_y(k);
        if x.wrapping_sub(self.player_state_view().x())
            .wrapping_add(32)
            < 64
            && y.wrapping_sub(self.player_state_view().y())
                .wrapping_add(32)
                < 64
        {
            let value = 0;
            self.overlord_slot_view_mut(k).set_overlord_type(value);
            self.dungeon_state_view_mut().increment_trap_trigger_latch();
        }
    }

    pub(super) fn overlord16_zoro_spawner(&mut self, k: usize) {
        const OVERLORD_ZORO_FACTORY_X: [i8; 8] = [-4, -2, 0, 2, 4, 6, 8, 12];
        let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
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
            let random_x = OVERLORD_ZORO_FACTORY_X[(self.get_random_number() & 7) as usize];
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
        const OVERLORD_WIZZROBE_X: [i8; 4] = [48, -48, 0, 0];
        const OVERLORD_WIZZROBE_Y: [i8; 4] = [16, 16, 64, -32];
        const OVERLORD_WIZZROBE_DELAY: [u8; 4] = [0, 16, 32, 48];
        if self.overlord_slot_view(k).gen2() != 128 {
            if self.frame_control_view().frame_counter() & 1 != 0 {
                let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
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
                    self.player_state_view()
                        .x()
                        .wrapping_add(OVERLORD_WIZZROBE_X[i] as i16 as u16),
                );
                self.Sprite_SetY(
                    j,
                    self.player_state_view()
                        .y()
                        .wrapping_add(OVERLORD_WIZZROBE_Y[i] as i16 as u16),
                );
                let floor = self.overlord_slot_view(k).floor();
                let mut sprite = self.sprite_slot_view_mut(j);
                sprite.set_delay_main(OVERLORD_WIZZROBE_DELAY[i]);
                sprite.set_floor(floor);
                sprite.set_b(1);
            }
        }
        self.temp_counter_view_mut().set(0xff);
    }

    pub(super) fn overlord14_tile_room(&mut self, k: usize) {
        let x = self
            .overlord_get_x(k)
            .wrapping_sub(self.world_state_view().bg2_x());
        let y = self
            .overlord_get_y(k)
            .wrapping_sub(self.world_state_view().bg2_y());
        if x & 0xff00 != 0 || y & 0xff00 != 0 {
            return;
        }
        let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
        self.overlord_slot_view_mut(k).set_gen2(value);
        if self.overlord_slot_view(k).gen2() != 0x80 {
            return;
        }
        if self.tile_room_spawn_tile(k) < 0 {
            let value = 0x81;
            self.overlord_slot_view_mut(k).set_gen2(value);
            return;
        }
        let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
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
        const SPAWN_FLYING_TILE_X: [u8; 22] = [
            0x70, 0x80, 0x60, 0x90, 0x90, 0x60, 0x70, 0x80, 0x80, 0x70, 0x50, 0xa0, 0xa0, 0x50,
            0x50, 0xa0, 0xa0, 0x50, 0x70, 0x80, 0x80, 0x70,
        ];
        const SPAWN_FLYING_TILE_Y: [u8; 22] = [
            0x80, 0x80, 0x70, 0x90, 0x70, 0x90, 0x60, 0xa0, 0x60, 0xa0, 0x60, 0xb0, 0x60, 0xb0,
            0x80, 0x90, 0x80, 0x90, 0x70, 0x90, 0x70, 0x90,
        ];
        let Some((j, _info)) = self.Sprite_SpawnDynamically(k, 0x94) else {
            return -1;
        };
        let i = self.overlord_slot_view(k).gen1() as usize;
        let x = SPAWN_FLYING_TILE_X[i] as u16 | ((self.overlord_slot_view(k).x_high() as u16) << 8);
        let y = SPAWN_FLYING_TILE_Y[i].wrapping_sub(8) as u16
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
        const OVERLORD_PIROGUSU_A: [u8; 4] = [2, 3, 0, 1];
        let counter = self.overlord_slot_view(k).overlord_type().wrapping_sub(16);
        self.temp_counter_view_mut().set(counter);
        if self.overlord_slot_view(k).gen2() != 128 {
            let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
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
            let direction = self.temp_counter_view().value();
            let a = OVERLORD_PIROGUSU_A[direction as usize];
            let mut sprite = self.sprite_slot_view_mut(j);
            sprite.set_floor(floor);
            sprite.set_delay_main(32);
            sprite.set_direction(direction);
            sprite.set_a(a);
        }
    }

    pub(super) fn overlord0_a_falling_square(&mut self, k: usize) {
        const CRUMBLE_TILE_PATH_DATA: [u8; 109] = [
            2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 3, 1, 3,
            0, 3, 1, 3, 0, 3, 1, 3, 0, 3, 1, 3, 0, 3, 1, 3, 0, 3, 1, 3, 0, 3, 1, 3, 0, 3, 1, 3, 0,
            3, 1, 3, 0, 3, 1, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0xff,
        ];
        const CRUMBLE_TILE_PATH_OFFS: [u8; 7] = [0, 25, 66, 77, 87, 98, 108];
        const CRUMBLE_TILE_PATH_X: [i8; 4] = [16, -16, 0, 0];
        const CRUMBLE_TILE_PATH_Y: [i8; 4] = [0, 0, 16, -16];
        if self.overlord_slot_view(k).gen2() != 0 {
            if self.overlord_slot_view(k).gen3() != 0 {
                let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
                self.overlord_slot_view_mut(k).set_gen2(value);
                return;
            }
            let x = self
                .overlord_get_x(k)
                .wrapping_sub(self.world_state_view().bg2_x());
            let y = self
                .overlord_get_y(k)
                .wrapping_sub(self.world_state_view().bg2_y());
            if !(x & 0xff00 != 0 || y & 0xff00 != 0) {
                let value = self.overlord_slot_view(k).gen3().wrapping_add(1);
                self.overlord_slot_view_mut(k).set_gen3(value);
            }
            return;
        }
        let value = 16;
        self.overlord_slot_view_mut(k).set_gen2(value);
        self.spawn_falling_tile(k);
        let j = self.overlord_slot_view(k).overlord_type().wrapping_sub(10) as usize;
        let i = self.overlord_slot_view(k).gen1() as usize;
        let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
        self.overlord_slot_view_mut(k).set_gen1(value);
        if i == (CRUMBLE_TILE_PATH_OFFS[j + 1] - CRUMBLE_TILE_PATH_OFFS[j]) as usize {
            let value = 0;
            self.overlord_slot_view_mut(k).set_overlord_type(value);
        }
        let t = CRUMBLE_TILE_PATH_DATA[CRUMBLE_TILE_PATH_OFFS[j] as usize + i];
        if t == 0xff {
            self.overlord_set_x(k, self.overlord_get_x(k).wrapping_add(0x0c1a));
            self.overlord_set_y(k, self.overlord_get_y(k).wrapping_add(0xbb66));
        } else {
            self.overlord_set_x(
                k,
                self.overlord_get_x(k)
                    .wrapping_add(CRUMBLE_TILE_PATH_X[t as usize] as i16 as u16),
            );
            self.overlord_set_y(
                k,
                self.overlord_get_y(k)
                    .wrapping_add(CRUMBLE_TILE_PATH_Y[t as usize] as i16 as u16),
            );
        }
    }

    pub(super) fn spawn_falling_tile(&mut self, k: usize) {
        if let Some(j) = self.GarnishAlloc() {
            let value = 3;
            self.garnish_slot_view_mut(j).set_garnish_type(value);
            let value = self.overlord_slot_view(k).x_high();
            self.garnish_slot_view_mut(j).set_x_high(value);
            let value = self.overlord_slot_view(k).x_low();
            self.garnish_slot_view_mut(j).set_x_low(value);
            let sfx = self.calculate_sfx_pan_arbitrary(self.garnish_slot_view(j).x_low()) | 0x1f;
            self.system_signals_view_mut().set_sound_effect_1(sfx);
            let y = self.overlord_get_y(k).wrapping_add(16);
            let value = y as u8;
            self.garnish_slot_view_mut(j).set_y_low(value);
            let value = (y >> 8) as u8;
            self.garnish_slot_view_mut(j).set_y_high(value);
            let value = 31;
            self.garnish_slot_view_mut(j).set_countdown(value);
            self.garnish_state_view_mut().set_active_type(31);
        }
    }

    pub(super) fn overlord09_wallmaster_spawner(&mut self, k: usize) {
        if self.overlord_slot_view(k).gen2() != 128 {
            if self.frame_control_view().frame_counter() & 1 == 0 {
                let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
                self.overlord_slot_view_mut(k).set_gen2(value);
            }
            return;
        }
        let value = 127;
        self.overlord_slot_view_mut(k).set_gen2(value);
        let Some((j, _info)) = self.Sprite_SpawnDynamicallyEx(k, 0x90, 12) else {
            return;
        };
        self.Sprite_SetX(j, self.player_state_view().x());
        self.Sprite_SetY(j, self.player_state_view().y());
        self.sprite_slot_view_mut(j).set_z(208);
        self.sprite_sfx_queue_sfx2_with_pan(j, 0x20);
        let floor = self.player_state_view().lower_level_state();
        self.sprite_slot_view_mut(j).set_floor(floor);
    }

    pub(super) fn overlord08_blob_spawner(&mut self, k: usize) {
        if self.overlord_slot_view(k).gen2() != 0 {
            let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
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
            const OVERLORD_ZOL_X: [i8; 4] = [0, 0, -48, 48];
            const OVERLORD_ZOL_Y: [i8; 4] = [-40, 56, 8, 8];
            let i = self.player_state_view().facing_index();
            self.Sprite_SetX(
                j,
                self.player_state_view()
                    .x()
                    .wrapping_add(OVERLORD_ZOL_X[i] as i16 as u16),
            );
            self.Sprite_SetY(
                j,
                self.player_state_view()
                    .y()
                    .wrapping_add(OVERLORD_ZOL_Y[i] as i16 as u16),
            );
            let floor = self.player_state_view().lower_level_state();
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
            self.dungeon_state_view_mut().set_floor_move_flags(1);
            return;
        }

        if self.overlord_slot_view(k).gen1() == 0 {
            let value = self.overlord_slot_view(k).gen2().wrapping_add(1);
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
                self.dungeon_state_view_mut().set_floor_move_flags(flags);
                let value = (self.get_random_number() & 127).wrapping_add(128);
                self.overlord_slot_view_mut(k).set_gen2(value);
                let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
                self.overlord_slot_view_mut(k).set_gen1(value);
            } else {
                self.dungeon_state_view_mut().set_floor_move_flags(1);
            }
        } else {
            let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
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
        const STALFOS_TRAP_TRIGGER: [u8; 8] = [255, 224, 192, 160, 128, 96, 64, 32];
        let x = self
            .overlord_get_x(k)
            .wrapping_sub(self.world_state_view().bg2_x());
        let y = self
            .overlord_get_y(k)
            .wrapping_sub(self.world_state_view().bg2_y());
        if x & 0xff00 != 0 || y & 0xff00 != 0 {
            return;
        }
        if self.overlord_slot_view(k).gen1() == 0 {
            if self.dungeon_state_view().trap_trigger_latch() != 0 {
                let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
                self.overlord_slot_view_mut(k).set_gen1(value);
            }
            return;
        }
        let old = self.overlord_slot_view(k).gen1();
        let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
        self.overlord_slot_view_mut(k).set_gen1(value);
        if old == STALFOS_TRAP_TRIGGER[k] {
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
            sprite.set_direction(0);
            self.sprite_overlord_play_falling_sfx(j);
        }
    }

    pub(super) fn overlord06_bad_switch_snake(&mut self, k: usize) {
        const SNAKE_TRAP_SPAWN_TIMERS_BY_SLOT: [u8; 8] =
            [0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90];
        let a = self.overlord_slot_view(k).gen1();
        if a == 0 {
            if self.dungeon_state_view().has_bomb_trap_activation() {
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
        let sprite_type = self.overlord_slot_view(k).overlord_type();
        let value = 0;
        self.overlord_slot_view_mut(k).set_overlord_type(value);
        if sprite_type == 26 {
            self.sprite_slot_view_mut(j).set_sprite_type(74);
            self.Sprite_TransmuteToBomb(j);
            self.sprite_slot_view_mut(j).set_delay_aux1(112);
        }
    }

    pub(super) fn overlord02_full_room_cannons(&mut self, k: usize) {
        const ALL_DIRECTION_METAL_BALL_FACTORY_IDX: [u8; 16] =
            [2, 2, 2, 2, 1, 1, 1, 1, 3, 3, 3, 3, 0, 0, 0, 0];
        const ALL_DIRECTION_METAL_BALL_FACTORY_X: [u8; 16] = [
            64, 96, 144, 176, 240, 240, 240, 240, 176, 144, 96, 64, 0, 0, 0, 0,
        ];
        const ALL_DIRECTION_METAL_BALL_FACTORY_Y: [u8; 16] = [
            16, 16, 16, 16, 64, 96, 160, 192, 240, 240, 240, 240, 192, 160, 96, 64,
        ];
        let x = self
            .overlord_get_x(k)
            .wrapping_sub(self.world_state_view().bg2_x());
        let y = self
            .overlord_get_y(k)
            .wrapping_sub(self.world_state_view().bg2_y());
        if (x | y) & 0xff00 != 0 || self.frame_control_view().frame_counter() & 0x0f != 0 {
            return;
        }
        self.sprite_workspace_view_mut().set_shared_scratch_a(0);
        let j = (self.get_random_number() & 15) as usize;
        self.temp_counter_view_mut()
            .set(ALL_DIRECTION_METAL_BALL_FACTORY_IDX[j]);
        let value = ALL_DIRECTION_METAL_BALL_FACTORY_X[j];
        self.overlord_slot_view_mut(k).set_x_low(value);
        let value = self.sprite_workspace_view().room_origin_x_high();
        self.overlord_slot_view_mut(k).set_x_high(value);
        let value = ALL_DIRECTION_METAL_BALL_FACTORY_Y[j];
        self.overlord_slot_view_mut(k).set_y_low(value);
        let value = self
            .sprite_workspace_view()
            .room_origin_y_high()
            .wrapping_add(1);
        self.overlord_slot_view_mut(k).set_y_high(value);
        self.overlord_spawn_cannon_ball(k, 0);
    }

    pub(super) fn overlord03_vertical_cannon(&mut self, k: usize) {
        let x = self
            .overlord_get_x(k)
            .wrapping_sub(self.world_state_view().bg2_x());
        if x & 0xff00 != 0 {
            let value = 255;
            self.overlord_slot_view_mut(k).set_gen2(value);
            return;
        }
        if self.frame_control_view().frame_counter() & 1 == 0
            && self.overlord_slot_view(k).gen2() != 0
        {
            let value = self.overlord_slot_view(k).gen2().wrapping_sub(1);
            self.overlord_slot_view_mut(k).set_gen2(value);
        }
        self.temp_counter_view_mut().set(2);
        self.sprite_workspace_view_mut().set_shared_scratch_a(0);
        let value = self.overlord_slot_view(k).gen1().wrapping_sub(1);
        self.overlord_slot_view_mut(k).set_gen1(value);
        if !sign8(self.overlord_slot_view(k).gen1()) {
            return;
        }
        let value = 56;
        self.overlord_slot_view_mut(k).set_gen1(value);
        let xd = if self.overlord_slot_view(k).gen2() == 0 {
            let value = 160;
            self.overlord_slot_view_mut(k).set_gen2(value);
            self.sprite_workspace_view_mut().set_shared_scratch_a(160);
            8
        } else {
            ((self.get_random_number() & 2) * 8) as i32
        };
        self.overlord_spawn_cannon_ball(k, xd);
    }

    pub(super) fn overlord_spawn_cannon_ball(&mut self, k: usize, xd: i32) {
        const OVERLORD_SPAWN_BALL_XVEL: [i8; 4] = [24, -24, 0, 0];
        const OVERLORD_SPAWN_BALL_YVEL: [i8; 4] = [0, 0, 24, -24];
        let Some((j, info)) = self.Sprite_SpawnDynamically(k, 0x50) else {
            return;
        };
        self.Sprite_SetX(j, info.r5_overlord_x.wrapping_add(xd as i16 as u16));
        self.Sprite_SetY(j, info.r7_overlord_y.wrapping_sub(1));
        let counter = self.temp_counter_view().value() as usize;
        let floor = self.overlord_slot_view(k).floor();
        let mut sprite = self.sprite_slot_view_mut(j);
        sprite.set_x_velocity(OVERLORD_SPAWN_BALL_XVEL[counter] as u8);
        sprite.set_y_velocity(OVERLORD_SPAWN_BALL_YVEL[counter] as u8);
        sprite.set_floor(floor);
        if self.sprite_workspace_view().shared_scratch_a() != 0 {
            let scratch = self.sprite_workspace_view().shared_scratch_a();
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
        self.garnish_state_view_mut().set_active_overlord_index(k as u8);
    }

    pub(super) fn overlord_check_if_active(&mut self, k: usize) {
        const OVERLORD_IN_RANGE_OFFS: [u16; 2] = [0x0130, (-0x40i16) as u16];
        if self.world_state_view().is_indoors() {
            return;
        }
        let j = (self.frame_control_view().frame_counter() & 1) as usize;
        let x = self
            .world_state_view()
            .bg2_x()
            .wrapping_add(OVERLORD_IN_RANGE_OFFS[j])
            .wrapping_sub(self.overlord_get_x(k));
        let y = self
            .world_state_view()
            .bg2_y()
            .wrapping_add(OVERLORD_IN_RANGE_OFFS[j])
            .wrapping_sub(self.overlord_get_y(k));
        if ((x >> 15) as usize) != j || ((y >> 15) as usize) != j {
            let value = 0;
            self.overlord_slot_view_mut(k).set_overlord_type(value);
            let blk = self.overlord_slot_view(k).sprite_block_pos();
            if blk != 0xffff {
                let loadedmask = 0x80 >> (blk & 7);
                self.overworld_sprite_loaded_view_mut()
                    .clear_loaded_mask(blk, loadedmask as u8);
            }
        }
    }

    pub(super) fn armos_coordinator_rotate_knights(&mut self, k: usize) {
        if self.overlord_slot_view(k).gen2() == 0 {
            let value = self.overlord_slot_view(k).gen1().wrapping_add(1);
            self.overlord_slot_view_mut(k).set_gen1(value);
        }
        self.armos_coordinator_rotate(k);
    }

    pub(super) fn armos_coordinator_rotate(&mut self, k: usize) {
        const ARMOS_KNIGHT_RING_ANGLE_OFFSETS: [u16; 6] = [0, 425, 340, 255, 170, 85];
        let angle = self
            .overlord_slot_view(0)
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
        self.temp_counter_view_mut().set(6);
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
        let tiletype = if self.world_state_view().is_indoors() {
            let mut t = if floor >= 1 { 0x1000 } else { 0 };
            t += ((x & 0x01f8) >> 3) as usize;
            t += ((y & 0x01f8) << 3) as usize;
            self.dungeon_state_view().bg2_attr(t)
        } else {
            self.overworld_get_tile_attribute_at_location(x >> 3, y)
        };
        self.sprite_workspace_view_mut().set_tile_type(tiletype);
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
