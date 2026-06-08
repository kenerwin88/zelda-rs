// Methods ported from zelda3/src/overlord.c and included inside ZeldaState.

use super::*;
use crate::types::sign8;

const OVERLORD_X_LO: usize = 0x0b08;
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
const OVERWORLD_SPRITE_WAS_LOADED: usize = 0x1ef80;
const GARNISH_ACTIVE: usize = 0x0fb4;
const TMP_COUNTER_OVERLORD: usize = 0x0fb5;
const SPRITE_TILETYPE: usize = 0x0fa5;
const SPRCOLL_Y_BASE: usize = 0x0fbe;
const ACTIVE_OVERLORD_INDEX: usize = 0x0fde;
const DUNG_FLOOR_MOVE_FLAGS: usize = 0x041a;
const GARNISH_Y_LO: usize = 0x1f81e;
const GARNISH_X_LO: usize = 0x1f83c;
const GARNISH_Y_HI: usize = 0x1f85a;
const GARNISH_X_HI: usize = 0x1f878;
const GARNISH_COUNTDOWN: usize = 0x1f90e;
const DUNG_BG2_ATTR_TABLE_OVERLORD: usize = 0x12000;

const SINUS_LOOKUP_TABLE: [u16; 256] = [
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
        self.ram[OVERLORD_X_LO + k] as u16 | ((self.ram[OVERLORD_X_HI + k] as u16) << 8)
    }

    pub(super) fn overlord_get_y(&self, k: usize) -> u16 {
        self.ram[OVERLORD_Y_LO + k] as u16 | ((self.ram[OVERLORD_Y_HI + k] as u16) << 8)
    }

    pub(super) fn overlord_stalfos_factory(&self, _k: usize) {
        // Overlord_StalfosFactory is an unused assert-only slot in C.
        panic!("overlord_stalfos_factory assert");
    }

    pub(super) fn overlord_set_x(&mut self, k: usize, v: u16) {
        self.ram[OVERLORD_X_LO + k] = v as u8;
        self.ram[OVERLORD_X_HI + k] = (v >> 8) as u8;
    }

    pub(super) fn overlord_set_y(&mut self, k: usize, v: u16) {
        self.ram[OVERLORD_Y_LO + k] = v as u8;
        self.ram[OVERLORD_Y_HI + k] = (v >> 8) as u8;
    }

    pub(super) fn overlord_spawn_boulder(&mut self) {
        if self.ram[PLAYER_IS_INDOORS] != 0
            || self.ram[OVERWORLD_BOULDER_TRAP_COUNT] == 0
            || (self.frame_control_view().submodule() | self.ram[FLAG_UNK1]) != 0
        {
            return;
        }
        self.ram[OVERWORLD_BOULDER_TRAP_TIMER] =
            self.ram[OVERWORLD_BOULDER_TRAP_TIMER].wrapping_add(1);
        if self.ram[OVERWORLD_BOULDER_TRAP_TIMER] & 63 != 0 {
            return;
        }
        let camera_y_hi = (read_le_u16(&self.ram, BG2VOFS_COPY2) >> 8) as u8;
        let coll_y_hi = (read_le_u16(&self.ram, SPRCOLL_Y_BASE) >> 8) as u8;
        if sign8(camera_y_hi.wrapping_sub(coll_y_hi).wrapping_sub(2)) {
            return;
        }
        if let Some((j, _info)) = self.Sprite_SpawnDynamically(0, 0xc2) {
            let x = read_le_u16(&self.ram, BG2HOFS_COPY2)
                .wrapping_add((self.get_random_number() & 127) as u16)
                .wrapping_add(64);
            let y = read_le_u16(&self.ram, BG2VOFS_COPY2).wrapping_sub(0x30);
            self.Sprite_SetX(j, x);
            self.Sprite_SetY(j, y);
            self.ram[SPRITE_FLOOR + j] = 0;
            self.ram[SPRITE_D + j] = 0;
            self.ram[SPRITE_Z + j] = 0;
        }
    }

    pub(super) fn overlord_main(&mut self) {
        self.overlord_execute_all();
        self.overlord_spawn_boulder();
    }

    pub(super) fn overlord_execute_all(&mut self) {
        if (self.frame_control_view().submodule() | self.ram[FLAG_UNK1]) != 0 {
            return;
        }
        for i in (0..=7).rev() {
            if self.ram[OVERLORD_TYPE + i] != 0 {
                self.overlord_execute_single(i);
            }
        }
    }

    pub(super) fn overlord_execute_single(&mut self, k: usize) {
        let j = self.ram[OVERLORD_TYPE + k];
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
        if self.ram[OVERLORD_GEN2 + k] != 0 {
            self.ram[OVERLORD_GEN2 + k] = self.ram[OVERLORD_GEN2 + k].wrapping_sub(1);
        }
        match self.ram[OVERLORD_GEN1 + k] {
            0 => {
                if self.ram[SPRITE_A] != 0 {
                    self.ram[OVERLORD_X_LO + k] = 120;
                    self.ram[OVERLORD_FLOOR + k] = 255;
                    self.ram[OVERLORD_X_LO + 2] = 64;
                    self.ram[OVERLORD_X_LO] = 192;
                    self.ram[OVERLORD_X_LO + 1] = 1;
                    self.armos_coordinator_rotate_knights(k);
                }
            }
            1 => {
                if self.armos_coordinator_check_knights() {
                    self.ram[OVERLORD_GEN1 + k] = self.ram[OVERLORD_GEN1 + k].wrapping_add(1);
                    self.ram[OVERLORD_GEN2 + k] = 0xff;
                }
            }
            2 | 4 => self.armos_coordinator_rotate_knights(k),
            3 => {
                self.ram[OVERLORD_X_LO + 2] = self.ram[OVERLORD_X_LO + 2].wrapping_sub(1);
                if self.ram[OVERLORD_X_LO + 2] == 32 {
                    self.ram[OVERLORD_GEN1 + k] = self.ram[OVERLORD_GEN1 + k].wrapping_add(1);
                    self.ram[OVERLORD_GEN2 + k] = 64;
                }
                self.armos_coordinator_rotate(k);
            }
            5 => {
                self.ram[OVERLORD_X_LO + 2] = self.ram[OVERLORD_X_LO + 2].wrapping_add(1);
                if self.ram[OVERLORD_X_LO + 2] == 64 {
                    self.ram[OVERLORD_GEN1 + k] = self.ram[OVERLORD_GEN1 + k].wrapping_add(1);
                    self.ram[OVERLORD_GEN2 + k] = 64;
                }
                self.armos_coordinator_rotate(k);
            }
            6 => {
                if self.ram[OVERLORD_GEN2 + k] != 0 {
                    return;
                }
                self.armos_coordinator_disable_coercion(k);
                for j in (0..=5).rev() {
                    self.ram[OVERLORD_X_HI + j] = ARMOS_COORDINATOR_BACK_WALL_X[j];
                    self.ram[OVERLORD_GEN2 + j] = 48;
                }
                self.ram[OVERLORD_GEN1 + k] = self.ram[OVERLORD_GEN1 + k].wrapping_add(1);
                self.ram[OVERLORD_GEN2 + k] = 255;
            }
            7 => {
                if self.ram[OVERLORD_GEN2 + k] != 0 {
                    return;
                }
                for j in (0..=5).rev() {
                    self.ram[OVERLORD_GEN2 + j] = self.ram[OVERLORD_GEN2 + j].wrapping_add(1);
                    if self.ram[OVERLORD_GEN2 + j] == 192 {
                        self.ram[OVERLORD_GEN1 + k] = 1;
                        self.ram[OVERLORD_FLOOR + k] =
                            0u8.wrapping_sub(self.ram[OVERLORD_FLOOR + k]);
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
        self.ram[OVERLORD_TYPE + k] = 0;
        self.ram[TMP_COUNTER_OVERLORD] = 3;
        loop {
            let i = self.ram[TMP_COUNTER_OVERLORD] as usize;
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
            self.ram[SPRITE_DELAY_MAIN + j] = RED_STALFOS_TRAP_DELAY[i];
            self.ram[SPRITE_FLOOR + j] = self.ram[OVERLORD_FLOOR + k];
            self.ram[SPRITE_E + j] = 1;
            self.ram[SPRITE_FLAGS2 + j] = 3;
            self.ram[SPRITE_D + j] = 2;
            self.ram[TMP_COUNTER_OVERLORD] = self.ram[TMP_COUNTER_OVERLORD].wrapping_sub(1);
            if sign8(self.ram[TMP_COUNTER_OVERLORD]) {
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
            self.ram[OVERLORD_TYPE + k] = 0;
            self.ram[DUNGEON_TRAP_TRIGGER_LATCH] =
                self.ram[DUNGEON_TRAP_TRIGGER_LATCH].wrapping_add(1);
        }
    }

    pub(super) fn overlord16_zoro_spawner(&mut self, k: usize) {
        const OVERLORD_ZORO_FACTORY_X: [i8; 8] = [-4, -2, 0, 2, 4, 6, 8, 12];
        self.ram[OVERLORD_GEN2 + k] = self.ram[OVERLORD_GEN2 + k].wrapping_sub(1);
        let x = self.overlord_get_x(k).wrapping_add(8);
        let y = self.overlord_get_y(k).wrapping_add(8);
        if self.get_tile_attribute_for_overlord(self.ram[OVERLORD_FLOOR + k], x, y) != 0x82 {
            return;
        }
        if self.ram[OVERLORD_GEN2 + k] >= 0x18 || self.ram[OVERLORD_GEN2 + k] & 3 != 0 {
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
            self.ram[SPRITE_Y_LO + j] = info.r7_overlord_y.wrapping_add(8) as u8;
            self.ram[SPRITE_Y_HI + j] = (info.r7_overlord_y >> 8) as u8;
            self.ram[SPRITE_FLOOR + j] = self.ram[OVERLORD_FLOOR + k];
            self.ram[SPRITE_FLAGS4 + j] = 1;
            self.ram[SPRITE_E + j] = 1;
            self.ram[SPRITE_IGNORE_PROJECTILE + j] = 1;
            self.ram[SPRITE_Y_VEL + j] = 16;
            self.ram[SPRITE_FLAGS2 + j] = 32;
            self.ram[SPRITE_OAM_FLAGS + j] = 13;
            self.ram[SPRITE_SUBTYPE2 + j] = self.get_random_number();
            self.ram[SPRITE_DELAY_MAIN + j] = 48;
            self.ram[SPRITE_BUMP_DAMAGE + j] = 3;
        }
    }

    pub(super) fn overlord15_wizzrobe_spawner(&mut self, k: usize) {
        const OVERLORD_WIZZROBE_X: [i8; 4] = [48, -48, 0, 0];
        const OVERLORD_WIZZROBE_Y: [i8; 4] = [16, 16, 64, -32];
        const OVERLORD_WIZZROBE_DELAY: [u8; 4] = [0, 16, 32, 48];
        if self.ram[OVERLORD_GEN2 + k] != 128 {
            if self.ram[FRAME_COUNTER] & 1 != 0 {
                self.ram[OVERLORD_GEN2 + k] = self.ram[OVERLORD_GEN2 + k].wrapping_sub(1);
            }
            return;
        }
        self.ram[OVERLORD_GEN2 + k] = 127;
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
                self.ram[SPRITE_DELAY_MAIN + j] = OVERLORD_WIZZROBE_DELAY[i];
                self.ram[SPRITE_FLOOR + j] = self.ram[OVERLORD_FLOOR + k];
                self.ram[SPRITE_B + j] = 1;
            }
        }
        self.ram[TMP_COUNTER_OVERLORD] = 0xff;
    }

    pub(super) fn overlord14_tile_room(&mut self, k: usize) {
        let x = self
            .overlord_get_x(k)
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        let y = self
            .overlord_get_y(k)
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        if x & 0xff00 != 0 || y & 0xff00 != 0 {
            return;
        }
        self.ram[OVERLORD_GEN2 + k] = self.ram[OVERLORD_GEN2 + k].wrapping_sub(1);
        if self.ram[OVERLORD_GEN2 + k] != 0x80 {
            return;
        }
        if self.tile_room_spawn_tile(k) < 0 {
            self.ram[OVERLORD_GEN2 + k] = 0x81;
            return;
        }
        self.ram[OVERLORD_GEN1 + k] = self.ram[OVERLORD_GEN1 + k].wrapping_add(1);
        if self.ram[OVERLORD_GEN1 + k] != 22 {
            self.ram[OVERLORD_GEN2 + k] = 0xe0;
        } else {
            self.ram[OVERLORD_TYPE + k] = 0;
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
        self.ram[SPRITE_E + j] = 1;
        let i = self.ram[OVERLORD_GEN1 + k] as usize;
        self.ram[SPRITE_X_LO + j] = SPAWN_FLYING_TILE_X[i];
        self.ram[SPRITE_Y_LO + j] = SPAWN_FLYING_TILE_Y[i].wrapping_sub(8);
        self.ram[SPRITE_Y_HI + j] = self.ram[OVERLORD_Y_HI + k];
        self.ram[SPRITE_X_HI + j] = self.ram[OVERLORD_X_HI + k];
        self.ram[SPRITE_FLOOR + j] = self.ram[OVERLORD_FLOOR + k];
        self.ram[SPRITE_HEALTH + j] = 4;
        self.ram[SPRITE_FLAGS5 + j] = 0;
        self.ram[SPRITE_HEALTH + j] = 0;
        self.ram[SPRITE_DEFL_BITS + j] = 8;
        self.ram[SPRITE_FLAGS2 + j] = 4;
        self.ram[SPRITE_OAM_FLAGS + j] = 1;
        self.ram[SPRITE_BUMP_DAMAGE + j] = 4;
        j as i32
    }

    pub(super) fn overlord10_pirogusu_spawner_left(&mut self, k: usize) {
        const OVERLORD_PIROGUSU_A: [u8; 4] = [2, 3, 0, 1];
        self.ram[TMP_COUNTER_OVERLORD] = self.ram[OVERLORD_TYPE + k].wrapping_sub(16);
        if self.ram[OVERLORD_GEN2 + k] != 128 {
            self.ram[OVERLORD_GEN2 + k] = self.ram[OVERLORD_GEN2 + k].wrapping_sub(1);
            return;
        }
        self.ram[OVERLORD_GEN2 + k] = (self.get_random_number() & 31).wrapping_add(96);
        let mut n = 0;
        for i in 0..16 {
            if self.ram[SPRITE_STATE + i] != 0 && self.ram[SPRITE_TYPE + i] == 0x10 {
                n += 1;
            }
        }
        if n >= 5 {
            return;
        }
        if let Some((j, info)) = self.Sprite_SpawnDynamicallyEx(k, 0x94, 12) {
            self.Sprite_SetX(j, info.r5_overlord_x);
            self.Sprite_SetY(j, info.r7_overlord_y);
            self.ram[SPRITE_FLOOR + j] = self.ram[OVERLORD_FLOOR + k];
            self.ram[SPRITE_DELAY_MAIN + j] = 32;
            self.ram[SPRITE_D + j] = self.ram[TMP_COUNTER_OVERLORD];
            self.ram[SPRITE_A + j] = OVERLORD_PIROGUSU_A[self.ram[TMP_COUNTER_OVERLORD] as usize];
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
        if self.ram[OVERLORD_GEN2 + k] != 0 {
            if self.ram[OVERLORD_GEN3 + k] != 0 {
                self.ram[OVERLORD_GEN2 + k] = self.ram[OVERLORD_GEN2 + k].wrapping_sub(1);
                return;
            }
            let x = self
                .overlord_get_x(k)
                .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
            let y = self
                .overlord_get_y(k)
                .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
            if !(x & 0xff00 != 0 || y & 0xff00 != 0) {
                self.ram[OVERLORD_GEN3 + k] = self.ram[OVERLORD_GEN3 + k].wrapping_add(1);
            }
            return;
        }
        self.ram[OVERLORD_GEN2 + k] = 16;
        self.spawn_falling_tile(k);
        let j = self.ram[OVERLORD_TYPE + k].wrapping_sub(10) as usize;
        let i = self.ram[OVERLORD_GEN1 + k] as usize;
        self.ram[OVERLORD_GEN1 + k] = self.ram[OVERLORD_GEN1 + k].wrapping_add(1);
        if i == (CRUMBLE_TILE_PATH_OFFS[j + 1] - CRUMBLE_TILE_PATH_OFFS[j]) as usize {
            self.ram[OVERLORD_TYPE + k] = 0;
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
            self.ram[GARNISH_TYPE + j] = 3;
            self.ram[GARNISH_X_HI + j] = self.ram[OVERLORD_X_HI + k];
            self.ram[GARNISH_X_LO + j] = self.ram[OVERLORD_X_LO + k];
            self.ram[SOUND_EFFECT_1] =
                self.calculate_sfx_pan_arbitrary(self.ram[GARNISH_X_LO + j]) | 0x1f;
            let y = self.overlord_get_y(k).wrapping_add(16);
            self.ram[GARNISH_Y_LO + j] = y as u8;
            self.ram[GARNISH_Y_HI + j] = (y >> 8) as u8;
            self.ram[GARNISH_COUNTDOWN + j] = 31;
            self.ram[GARNISH_ACTIVE] = 31;
        }
    }

    pub(super) fn overlord09_wallmaster_spawner(&mut self, k: usize) {
        if self.ram[OVERLORD_GEN2 + k] != 128 {
            if self.ram[FRAME_COUNTER] & 1 == 0 {
                self.ram[OVERLORD_GEN2 + k] = self.ram[OVERLORD_GEN2 + k].wrapping_sub(1);
            }
            return;
        }
        self.ram[OVERLORD_GEN2 + k] = 127;
        let Some((j, _info)) = self.Sprite_SpawnDynamicallyEx(k, 0x90, 12) else {
            return;
        };
        self.Sprite_SetX(j, self.player_state_view().x());
        self.Sprite_SetY(j, self.player_state_view().y());
        self.ram[SPRITE_Z + j] = 208;
        self.sprite_sfx_queue_sfx2_with_pan(j, 0x20);
        self.ram[SPRITE_FLOOR + j] = self.ram[LINK_IS_ON_LOWER_LEVEL];
    }

    pub(super) fn overlord08_blob_spawner(&mut self, k: usize) {
        if self.ram[OVERLORD_GEN2 + k] != 0 {
            self.ram[OVERLORD_GEN2 + k] = self.ram[OVERLORD_GEN2 + k].wrapping_sub(1);
            return;
        }
        self.ram[OVERLORD_GEN2 + k] = 0xa0;
        let mut n = 0;
        for i in 0..16 {
            if self.ram[SPRITE_STATE + i] != 0 && self.ram[SPRITE_TYPE + i] == 0x8f {
                n += 1;
            }
        }
        if n >= 5 {
            return;
        }
        if let Some((j, _info)) = self.Sprite_SpawnDynamicallyEx(k, 0x8f, 12) {
            const OVERLORD_ZOL_X: [i8; 4] = [0, 0, -48, 48];
            const OVERLORD_ZOL_Y: [i8; 4] = [-40, 56, 8, 8];
            let i = (self.ram[LINK_DIRECTION_FACING] >> 1) as usize;
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
            self.ram[SPRITE_Z + j] = 192;
            self.ram[SPRITE_FLOOR + j] = self.ram[LINK_IS_ON_LOWER_LEVEL];
            self.ram[SPRITE_AI_STATE + j] = 2;
            self.ram[SPRITE_E + j] = 2;
            self.ram[SPRITE_C + j] = 2;
            self.ram[SPRITE_HEAD_DIR + j] = self.get_random_number() & 31 | 16;
        }
    }

    pub(super) fn overlord07_moving_floor(&mut self, k: usize) {
        if self.ram[SPRITE_STATE] == 4 {
            self.ram[OVERLORD_TYPE + k] = 0;
            self.ram[DUNG_FLOOR_MOVE_FLAGS] = 1;
            return;
        }

        if self.ram[OVERLORD_GEN1 + k] == 0 {
            self.ram[OVERLORD_GEN2 + k] = self.ram[OVERLORD_GEN2 + k].wrapping_add(1);
            if self.ram[OVERLORD_GEN2 + k] == 32 {
                self.ram[OVERLORD_GEN2 + k] = 0;
                let mask = if self.ram[OVERLORD_X_LO + k] != 0 {
                    3
                } else {
                    1
                };
                self.ram[DUNG_FLOOR_MOVE_FLAGS] = (self.get_random_number() & mask) * 2;
                self.ram[OVERLORD_GEN2 + k] = (self.get_random_number() & 127).wrapping_add(128);
                self.ram[OVERLORD_GEN1 + k] = self.ram[OVERLORD_GEN1 + k].wrapping_add(1);
            } else {
                self.ram[DUNG_FLOOR_MOVE_FLAGS] = 1;
            }
        } else {
            self.ram[OVERLORD_GEN2 + k] = self.ram[OVERLORD_GEN2 + k].wrapping_sub(1);
            if self.ram[OVERLORD_GEN2 + k] == 0 {
                self.ram[OVERLORD_GEN1 + k] = 0;
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
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        let y = self
            .overlord_get_y(k)
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        if x & 0xff00 != 0 || y & 0xff00 != 0 {
            return;
        }
        if self.ram[OVERLORD_GEN1 + k] == 0 {
            if self.ram[DUNGEON_TRAP_TRIGGER_LATCH] != 0 {
                self.ram[OVERLORD_GEN1 + k] = self.ram[OVERLORD_GEN1 + k].wrapping_add(1);
            }
            return;
        }
        let old = self.ram[OVERLORD_GEN1 + k];
        self.ram[OVERLORD_GEN1 + k] = self.ram[OVERLORD_GEN1 + k].wrapping_add(1);
        if old == STALFOS_TRAP_TRIGGER[k] {
            self.ram[OVERLORD_TYPE + k] = 0;
            let Some((j, info)) = self.Sprite_SpawnDynamicallyEx(k, 0x85, 12) else {
                return;
            };
            self.Sprite_SetX(j, info.r5_overlord_x);
            self.Sprite_SetY(j, info.r7_overlord_y);
            self.ram[SPRITE_Z + j] = 224;
            self.ram[SPRITE_FLOOR + j] = self.ram[OVERLORD_FLOOR + k];
            self.ram[SPRITE_D + j] = 0;
            self.sprite_overlord_play_falling_sfx(j);
        }
    }

    pub(super) fn overlord06_bad_switch_snake(&mut self, k: usize) {
        const SNAKE_TRAP_OVERLORD_TAB1: [u8; 8] = [0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90];
        let a = self.ram[OVERLORD_GEN1 + k];
        if a == 0 {
            if self.ram[ACTIVATE_BOMB_TRAP_OVERLORD] != 0 {
                self.ram[OVERLORD_GEN1 + k] = 1;
            }
            return;
        }
        self.ram[OVERLORD_GEN1 + k] = a.wrapping_add(1);
        if a != SNAKE_TRAP_OVERLORD_TAB1[k] {
            return;
        }
        let Some((j, info)) = self.Sprite_SpawnDynamically(k, 0x6e) else {
            return;
        };
        self.Sprite_SetX(j, info.r5_overlord_x);
        self.Sprite_SetY(j, info.r7_overlord_y);
        self.ram[SPRITE_Z + j] = 192;
        self.ram[SPRITE_E + j] = 192;
        self.ram[SPRITE_FLAGS3 + j] |= 0x10;
        self.ram[SPRITE_FLOOR + j] = self.ram[OVERLORD_FLOOR + k];
        self.sprite_sfx_queue_sfx2_with_pan(j, 0x20);
        let sprite_type = self.ram[OVERLORD_TYPE + k];
        self.ram[OVERLORD_TYPE + k] = 0;
        if sprite_type == 26 {
            self.ram[SPRITE_TYPE + j] = 74;
            self.Sprite_TransmuteToBomb(j);
            self.ram[SPRITE_DELAY_AUX1 + j] = 112;
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
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        let y = self
            .overlord_get_y(k)
            .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2));
        if (x | y) & 0xff00 != 0 || self.ram[FRAME_COUNTER] & 0x0f != 0 {
            return;
        }
        self.ram[SPRITE_SHARED_SCRATCH_A] = 0;
        let j = (self.get_random_number() & 15) as usize;
        self.ram[TMP_COUNTER_OVERLORD] = ALL_DIRECTION_METAL_BALL_FACTORY_IDX[j];
        self.ram[OVERLORD_X_LO + k] = ALL_DIRECTION_METAL_BALL_FACTORY_X[j];
        self.ram[OVERLORD_X_HI + k] = self.ram[SPRITE_ROOM_ORIGIN_X_HI];
        self.ram[OVERLORD_Y_LO + k] = ALL_DIRECTION_METAL_BALL_FACTORY_Y[j];
        self.ram[OVERLORD_Y_HI + k] = self.ram[SPRITE_ROOM_ORIGIN_Y_HI].wrapping_add(1);
        self.overlord_spawn_cannon_ball(k, 0);
    }

    pub(super) fn overlord03_vertical_cannon(&mut self, k: usize) {
        let x = self
            .overlord_get_x(k)
            .wrapping_sub(read_le_u16(&self.ram, BG2HOFS_COPY2));
        if x & 0xff00 != 0 {
            self.ram[OVERLORD_GEN2 + k] = 255;
            return;
        }
        if self.ram[FRAME_COUNTER] & 1 == 0 && self.ram[OVERLORD_GEN2 + k] != 0 {
            self.ram[OVERLORD_GEN2 + k] = self.ram[OVERLORD_GEN2 + k].wrapping_sub(1);
        }
        self.ram[TMP_COUNTER_OVERLORD] = 2;
        self.ram[SPRITE_SHARED_SCRATCH_A] = 0;
        self.ram[OVERLORD_GEN1 + k] = self.ram[OVERLORD_GEN1 + k].wrapping_sub(1);
        if !sign8(self.ram[OVERLORD_GEN1 + k]) {
            return;
        }
        self.ram[OVERLORD_GEN1 + k] = 56;
        let xd = if self.ram[OVERLORD_GEN2 + k] == 0 {
            self.ram[OVERLORD_GEN2 + k] = 160;
            self.ram[SPRITE_SHARED_SCRATCH_A] = 160;
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
        let counter = self.ram[TMP_COUNTER_OVERLORD] as usize;
        self.ram[SPRITE_X_VEL + j] = OVERLORD_SPAWN_BALL_XVEL[counter] as u8;
        self.ram[SPRITE_Y_VEL + j] = OVERLORD_SPAWN_BALL_YVEL[counter] as u8;
        self.ram[SPRITE_FLOOR + j] = self.ram[OVERLORD_FLOOR + k];
        if self.ram[SPRITE_SHARED_SCRATCH_A] != 0 {
            self.ram[SPRITE_AI_STATE + j] = self.ram[SPRITE_SHARED_SCRATCH_A];
            self.ram[SPRITE_Y_LO + j] = self.ram[SPRITE_Y_LO + j].wrapping_add(8);
            self.ram[SPRITE_FLAGS2 + j] = 3;
            self.ram[SPRITE_FLAGS4 + j] = 9;
        }
        self.ram[SPRITE_DELAY_AUX2 + j] = 64;
        self.sprite_sfx_queue_sfx3_with_pan(j, 0x07);
    }

    pub(super) fn overlord01_position_target(&mut self, k: usize) {
        self.ram[ACTIVE_OVERLORD_INDEX] = k as u8;
    }

    pub(super) fn overlord_check_if_active(&mut self, k: usize) {
        const OVERLORD_IN_RANGE_OFFS: [u16; 2] = [0x0130, (-0x40i16) as u16];
        if self.ram[PLAYER_IS_INDOORS] != 0 {
            return;
        }
        let j = (self.ram[FRAME_COUNTER] & 1) as usize;
        let x = read_le_u16(&self.ram, BG2HOFS_COPY2)
            .wrapping_add(OVERLORD_IN_RANGE_OFFS[j])
            .wrapping_sub(self.overlord_get_x(k));
        let y = read_le_u16(&self.ram, BG2VOFS_COPY2)
            .wrapping_add(OVERLORD_IN_RANGE_OFFS[j])
            .wrapping_sub(self.overlord_get_y(k));
        if ((x >> 15) as usize) != j || ((y >> 15) as usize) != j {
            self.ram[OVERLORD_TYPE + k] = 0;
            let blk = read_le_u16(&self.ram, OVERLORD_OFFSET_SPRITE_POS + k * 2);
            if blk != 0xffff {
                let loadedmask = 0x80 >> (blk & 7);
                self.ram[OVERWORLD_SPRITE_WAS_LOADED + (blk >> 3) as usize] &= !loadedmask as u8;
            }
        }
    }

    pub(super) fn armos_coordinator_rotate_knights(&mut self, k: usize) {
        if self.ram[OVERLORD_GEN2 + k] == 0 {
            self.ram[OVERLORD_GEN1 + k] = self.ram[OVERLORD_GEN1 + k].wrapping_add(1);
        }
        self.armos_coordinator_rotate(k);
    }

    pub(super) fn armos_coordinator_rotate(&mut self, k: usize) {
        const ARMOS_COORDINATOR_TAB0: [u16; 6] = [0, 425, 340, 255, 170, 85];
        let angle = read_le_u16(&self.ram, OVERLORD_X_LO)
            .wrapping_add(self.ram[OVERLORD_FLOOR + k] as i8 as i16 as u16);
        write_le_u16(&mut self.ram, OVERLORD_X_LO, angle);

        for (i, offset) in ARMOS_COORDINATOR_TAB0.iter().enumerate() {
            let t0 = angle.wrapping_add(*offset);
            let size = self.ram[OVERLORD_X_LO + 2];
            let tx = self
                .overlord_get_x(k)
                .wrapping_add(armos_sin(t0, size) as i16 as u16);
            self.ram[OVERLORD_X_HI + i] = tx as u8;
            self.ram[OVERLORD_Y_HI + i] = (tx >> 8) as u8;
            let ty = self
                .overlord_get_y(k)
                .wrapping_add(armos_sin(t0.wrapping_add(0x80), size) as i16 as u16);
            self.ram[OVERLORD_GEN2 + i] = ty as u8;
            self.ram[OVERLORD_FLOOR + i] = (ty >> 8) as u8;
        }
        self.ram[TMP_COUNTER_OVERLORD] = 6;
    }

    pub(super) fn armos_coordinator_check_knights(&self) -> bool {
        for j in (0..=5).rev() {
            if self.ram[SPRITE_STATE + j] != 0 && self.ram[SPRITE_AI_STATE + j] == 0 {
                return false;
            }
        }
        true
    }

    pub(super) fn armos_coordinator_disable_coercion(&mut self, _k: usize) {
        for j in (0..=5).rev() {
            self.ram[SPRITE_AI_STATE + j] = 0;
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
        self.ram[SPRITE_X_LO + k] = x as u8;
        self.ram[SPRITE_X_HI + k] = (x >> 8) as u8;
    }

    fn Sprite_SetY(&mut self, k: usize, y: u16) {
        self.ram[SPRITE_Y_LO + k] = y as u8;
        self.ram[SPRITE_Y_HI + k] = (y >> 8) as u8;
    }

    fn Sprite_GetX(&self, k: usize) -> u16 {
        self.ram[SPRITE_X_LO + k] as u16 | ((self.ram[SPRITE_X_HI + k] as u16) << 8)
    }

    fn GarnishAlloc(&self) -> Option<usize> {
        (0..30).rev().find(|&j| self.ram[GARNISH_TYPE + j] == 0)
    }

    fn Sprite_TransmuteToBomb(&mut self, k: usize) {
        self.ram[SPRITE_TYPE + k] = 0x4a;
        self.ram[SPRITE_C + k] = 1;
        self.ram[SPRITE_DELAY_AUX1 + k] = 255;
        self.ram[SPRITE_FLAGS3 + k] = 0x18;
        self.ram[SPRITE_OAM_FLAGS + k] = 8;
        self.ram[SPRITE_HEALTH + k] = 0;
    }

    fn get_tile_attribute_for_overlord(&mut self, floor: u8, x: u16, y: u16) -> u8 {
        let tiletype = if self.ram[PLAYER_IS_INDOORS] != 0 {
            let mut t = if floor >= 1 { 0x1000 } else { 0 };
            t += ((x & 0x01f8) >> 3) as usize;
            t += ((y & 0x01f8) << 3) as usize;
            self.ram[DUNG_BG2_ATTR_TABLE_OVERLORD + t]
        } else {
            self.overworld_get_tile_attribute_at_location(x >> 3, y)
        };
        self.ram[SPRITE_TILETYPE] = tiletype;
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
    let t = armos_mult(SINUS_LOOKUP_TABLE[(a & 0xff) as usize], b);
    if a & 0x100 != 0 {
        (0u8.wrapping_sub(t)) as i8
    } else {
        t as i8
    }
}
