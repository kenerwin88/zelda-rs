//! Ported Mothula-boss handlers from sprite_main.c.
//!
//! Each method preserves a 1:1 mapping to the C source (sprite_main.c
//! lines 13775, 22508, 22591, 22599, 22620). The original C body is
//! reproduced as a comment block immediately above each port so a
//! reviewer can verify behavior line-by-line.
//!
//! Helpers whose graph reaches outside the currently-ported subset
//! (`Sprite_MoveZ`, `Sprite_MoveXY`, `Sprite_CheckDamageToAndFromLink`,
//! `Sprite_ApplySpeedTowardsLink`, `Sprite_Get16BitCoords`,
//! `Sprite_SpawnDynamically`/`Sprite_SetSpawnedCoordinates`) keep
//! `_for_mothula` adapter names so the data-state side of these handlers
//! stays exercisable while the remaining canonical ports land.

use super::sprite::{DrawMultipleData, PrepOamCoordsRet as SpritePrepOamCoordsRet};
use super::*;
use crate::types::{sign8, PointU8, SpriteHitBox};
use crate::zelda_rtl::sprite::SpriteSpawnInfo;

// SPRITE_DELAY_AUX3 = 0x0ee0 (sprite_delay_aux3 in variables.h).
const SPRITE_DELAY_AUX3_MOTHULA: usize = 0x0ee0;
const SPRITE_Y_RECOIL_MOTHULA: usize = 0x0f30;
const SPRITE_TILETYPE_MOTHULA: usize = 0x0fa5;
// tmp_counter is g_ram+0xFB5 (variables.h:767). Shared with other modules.
const TMP_COUNTER_MOTHULA: usize = 0x0fb5;
const ARMOS_KNIGHT_REMAINING_COUNT: usize = 0x0ff8;
const GARNISH_ACTIVE_MOTHULA: usize = 0x0fb4;
const GARNISH_Y_LO_MOTHULA: usize = 0x1f81e;
const GARNISH_X_LO_MOTHULA: usize = 0x1f83c;
const GARNISH_Y_HI_MOTHULA: usize = 0x1f85a;
const GARNISH_X_HI_MOTHULA: usize = 0x1f878;
const GARNISH_COUNTDOWN_MOTHULA: usize = 0x1f90e;
const GARNISH_SPRITE_MOTHULA: usize = 0x1f92c;
const GARNISH_FLOOR_MOTHULA: usize = 0x1f968;
const OVERLORD_X_LO_MOTHULA: usize = 0x0b08;
const OVERLORD_X_HI_MOTHULA: usize = 0x0b10;
const OVERLORD_Y_LO_MOTHULA: usize = 0x0b18;
const OVERLORD_Y_HI_MOTHULA: usize = 0x0b20;
const OVERLORD_GEN1_MOTHULA: usize = 0x0b28;
const OVERLORD_GEN2_MOTHULA: usize = 0x0b30;
const OVERLORD_GEN3_MOTHULA: usize = 0x0b38;
const OVERLORD_FLOOR_MOTHULA: usize = 0x0b40;
const K_FEATURES0_MISC_BUG_FIXES_MOTHULA: u32 = 4096;

// kMothula_Dmd from sprite_main.c:13776 — packed as (x:i8, y:i8, char:u16, big:u8).
type MothulaDmd = (i8, i8, u16, u8);
const K_MOTHULA_DMD: [MothulaDmd; 24] = [
    (-24, -8, 0x0080, 2),
    (-8, -8, 0x0082, 2),
    (8, -8, 0x4082, 2),
    (24, -8, 0x4080, 2),
    (-24, 8, 0x00a0, 2),
    (-8, 8, 0x00a2, 2),
    (8, 8, 0x40a2, 2),
    (24, 8, 0x40a0, 2),
    (-24, -8, 0x0084, 2),
    (-8, -8, 0x0086, 2),
    (8, -8, 0x4086, 2),
    (24, -8, 0x4084, 2),
    (-24, 8, 0x00a4, 2),
    (-8, 8, 0x00a6, 2),
    (8, 8, 0x40a6, 2),
    (24, 8, 0x40a4, 2),
    (-8, -8, 0x0088, 2),
    (-8, -8, 0x0088, 2),
    (8, -8, 0x4088, 2),
    (8, -8, 0x4088, 2),
    (-8, 8, 0x00a8, 2),
    (-8, 8, 0x00a8, 2),
    (8, 8, 0x40a8, 2),
    (8, 8, 0x40a8, 2),
];

// kMothula_Draw_X from sprite_main.c:13809.
const K_MOTHULA_DRAW_X: [i8; 27] = [
    0, 3, 6, 9, 12, -3, -6, -9, -12, 0, 2, 4, 6, 8, -2, -4, -6, -8, 0, 1, 2, 3, 4, -1, -2, -3, -4,
];

// kMothula_FlapWingsGfx from sprite_main.c:22592.
const K_MOTHULA_FLAP_WINGS_GFX: [u8; 4] = [0, 1, 2, 1];

// kMothula_XYvel from sprite_main.c:22562.
const K_MOTHULA_XYVEL: [i8; 10] = [-16, -12, 0, 12, 16, 12, 0, -12, -16, -12];

// kMothula_Beam_Xvel / Yvel from sprite_main.c:22600-22601.
const K_MOTHULA_BEAM_XVEL: [i8; 3] = [-16, 0, 16];
const K_MOTHULA_BEAM_YVEL: [i8; 3] = [24, 32, 24];

// kMothula_Spike_XLo / YLo / Dir from sprite_main.c:22621-22632.
const K_MOTHULA_SPIKE_XLO: [u8; 30] = [
    0x38, 0x48, 0x58, 0x68, 0x88, 0x98, 0xa8, 0xb8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xb8,
    0xa8, 0x98, 0x78, 0x68, 0x58, 0x48, 0x38, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28,
];
const K_MOTHULA_SPIKE_YLO: [u8; 30] = [
    0x38, 0x38, 0x38, 0x38, 0x38, 0x38, 0x38, 0x38, 0x48, 0x58, 0x68, 0x78, 0x98, 0xa8, 0xb8, 0xc8,
    0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xb8, 0xa8, 0x98, 0x78, 0x68, 0x58, 0x48,
];
const K_MOTHULA_SPIKE_DIR: [u8; 30] = [
    2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0,
];
const K_GIBDO_DIR_TARGET: [u8; 4] = [2, 6, 4, 0];
const K_GIBDO_GFX: [u8; 8] = [4, 8, 11, 10, 0, 6, 3, 7];
const K_GIBDO_XY_VEL: [i8; 10] = [-16, 0, 0, 0, 16, 0, 0, 0, -16, 0];
const K_GIBDO_GFX2: [u8; 8] = [9, 2, 0, 4, 11, 3, 1, 5];
const K_PIROGUSU_A0: [u8; 4] = [2, 3, 0, 1];
const K_PIROGUSU_A1: [u8; 8] = [9, 11, 5, 7, 5, 11, 7, 9];
const K_PIROGUSU_A2: [u8; 8] = [16, 17, 18, 19, 12, 13, 14, 15];
const K_PIROGUSU_XY_VEL: [i8; 6] = [0, 0, 4, -4, 0, 0];
const K_PIROGUSU_XY_VEL2: [i8; 6] = [2, -2, 0, 0, 2, -2];
const K_PIROGUSU_XY_VEL3: [i8; 6] = [24, -24, 0, 0, 24, -24];
const K_PIROGUSU_DIR: [u8; 8] = [2, 3, 2, 3, 0, 1, 0, 1];
const K_LASER_EYE_DIRS: [u8; 4] = [2, 3, 0, 1];
const K_STALFOS_KNIGHT_CASE2_GFX: [u8; 2] = [0, 1];
const K_STALFOS_KNIGHT_CASE2_DIR: [u8; 16] = [0, 0, 0, 2, 1, 1, 1, 2, 0, 0, 0, 2, 1, 1, 1, 2];
const K_STALFOS_KNIGHT_CASE6_C: [u8; 32] = [
    0, 4, 8, 12, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14,
    14, 14, 15, 14, 12, 8, 4, 0,
];
const K_STALFOS_KNIGHT_CASE7_GFX: [u8; 2] = [1, 4];
const K_TERRORPIN_XVEL: [i8; 8] = [8, -8, 0, 0, 12, -12, 0, 0];
const K_TERRORPIN_YVEL: [i8; 8] = [0, 0, 8, -8, 0, 0, 12, -12];
const K_TERRORPIN_OAMFLAGS: [u8; 2] = [0, 0x40];
const K_TERRORPIN_OVERTURNED_XVEL: [i8; 2] = [8, -8];
const K_ARRGHUS_GFX: [u8; 9] = [1, 1, 1, 2, 2, 1, 1, 0, 0];
const K_ARRGI_GFX: [u8; 8] = [0, 1, 2, 2, 2, 2, 2, 1];
const K_BABUSU_GFX: [u8; 6] = [5, 4, 3, 2, 1, 0];
const K_BABUSU_DIR_GFX: [u8; 4] = [6, 6, 0, 0];
const K_BABUSU_XY_VEL: [i8; 6] = [32, -32, 0, 0, 32, -32];
const K_BABUSU_SCURRY_GFX: [u8; 4] = [18, 14, 12, 16];
const K_WIZZROBE_CLOAK_GFX: [u8; 4] = [4, 2, 0, 6];
const K_WIZZROBE_ATTACK_GFX: [u8; 8] = [0, 1, 1, 1, 1, 1, 1, 0];
const K_WIZZROBE_ATTACK_DIR_GFX: [u8; 4] = [4, 2, 0, 6];
const K_KYAMERON_COAGULATE_GFX: [u8; 8] = [4, 7, 14, 13, 12, 6, 6, 5];
const K_KYAMERON_XVEL: [i8; 4] = [32, -32, 32, -32];
const K_KYAMERON_YVEL: [i8; 4] = [32, 32, -32, -32];
const K_KYAMERON_MOVING_GFX: [u8; 4] = [3, 2, 1, 0];
const K_PENGATOR_GFX: [u8; 4] = [5, 0, 10, 15];
const K_PENGATOR_XY_VEL: [i8; 6] = [1, -1, 0, 0, 1, -1];
const K_PENGATOR_JUMP: [u8; 4] = [4, 4, 3, 2];
const K_PENGATOR_GARNISH_Y: [i8; 8] = [8, 10, 12, 14, 12, 12, 12, 12];
const K_PENGATOR_GARNISH_X: [i8; 8] = [4, 4, 4, 4, 0, 4, 8, 12];
const K_FLUTE_BOY_ANIMAL_XVEL_MOTHULA: [i8; 4] = [16, -16, 0, 0];
const K_ZAZAK_YVEL_MOTHULA: [i8; 4] = [0, 0, 16, -16];
const K_GORIYA_XVEL: [i8; 32] = [
    0, 16, -16, 0, 0, 13, -13, 0, 0, 13, -13, 0, 0, 0, 0, 0, 0, -24, 24, 0, 0, -16, 16, 0, 0, -16,
    16, 0, 0, 0, 0, 0,
];
const K_GORIYA_YVEL: [i8; 32] = [
    0, 0, 0, 0, -16, -5, -5, 0, 16, 13, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, -24, -16, -16, 0, 24, 16,
    16, 0, 0, 0, 0, 0,
];
const K_GORIYA_DIR: [u8; 32] = [
    0, 0, 1, 0, 3, 3, 3, 0, 2, 2, 2, 0, 0, 0, 0, 0, 0, 1, 0, 0, 3, 3, 3, 0, 2, 2, 2, 0, 0, 0, 0, 0,
];
const K_GORIYA_GFX: [u8; 16] = [8, 6, 0, 3, 9, 7, 1, 4, 8, 6, 0, 3, 9, 7, 2, 5];
const K_EYEGORE_CLOSING_GFX: [u8; 8] = [0, 0, 1, 1, 2, 2, 2, 2];
const K_EYEGORE_OPENING_GFX: [u8; 8] = [2, 2, 2, 2, 1, 1, 0, 0];
const K_EYEGORE_CHASING_GFX: [u8; 16] = [7, 5, 2, 9, 8, 6, 3, 10, 7, 5, 2, 9, 8, 6, 4, 11];
const K_EYEGORE_OPENING_DELAY: [u8; 4] = [0x60, 0x80, 0xa0, 0x80];
const K_ARMOS_KNIGHT_GFX1: [u8; 5] = [5, 4, 3, 2, 1];
const K_ARMOS_KNIGHT_XV: [i8; 2] = [16, -16];
const K_FLUTE_BOY_ANIMAL_OAM_FLAGS: [u8; 2] = [0x40, 0];
const K_FLUTE_BOY_ANIMAL_GFX: [u8; 3] = [0, 1, 2];
const K_FLUTE_BOY_OSTRICH_GFX: [u8; 4] = [0, 1, 0, 2];
const K_FLUTE_BOY_BIRD_X: [i8; 2] = [8, 0];
const K_FREEZOR_XVEL: [i8; 4] = [8, -8, 0, 0];
const K_FREEZOR_YVEL: [i8; 4] = [0, 0, 18, -18];
const K_FREEZOR_MOVING_GFX: [u8; 4] = [1, 2, 1, 3];
const K_FREEZOR_SPARKLE_X: [i8; 8] = [-4, -2, 0, 2, 4, 6, 8, 10];
const K_FREEZOR_MELTING_GFX: [u8; 4] = [6, 5, 4, 7];
const K_KODONDO_XVEL: [i8; 4] = [1, -1, 0, 0];
const K_KODONDO_YVEL: [i8; 4] = [0, 0, 1, -1];
const K_KODONDO_GFX: [u8; 8] = [2, 2, 0, 5, 3, 3, 0, 5];
const K_KODONDO_OAM_FLAGS: [u8; 8] = [0x40, 0, 0, 0, 0x40, 0, 0x40, 0x40];
const K_KODONDO_FLAME_GFX: [u8; 8] = [2, 2, 0, 5, 4, 4, 1, 6];
const K_KHOLDSTARE_TARGET_XVEL: [i8; 4] = [16, 16, -16, -16];
const K_KHOLDSTARE_TARGET_YVEL: [i8; 4] = [-16, 16, 16, -16];
const K_KHOLDSTARE_TRIPLICATE_TAB0: [i8; 3] = [32, -32, 0];
const K_KHOLDSTARE_TRIPLICATE_TAB1: [i8; 3] = [-32, -32, 48];
const K_BOMBER_GFX: [u8; 4] = [9, 10, 8, 7];
const K_BOMBER_XVEL: [i8; 8] = [16, 12, 0, -12, -16, -12, 0, 12];
const K_BOMBER_YVEL: [i8; 8] = [0, 12, 16, 12, 0, -12, -16, -12];
const K_BOMBER_TAB0: [u8; 4] = [0, 4, 2, 6];
const K_BOMBER_SPAWN_PELLET_X: [i8; 4] = [14, -6, 4, 4];
const K_BOMBER_SPAWN_PELLET_Y: [i8; 4] = [7, 7, 12, -4];
const K_PIKIT_GFX: [u8; 24] = [
    2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2,
];
const K_PIKIT_XY_OFFS: [i8; 72] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    12, 16, 24, 32, 32, 24, 16, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -12, -16, -24,
    -32, -32, -24, -16, -12, 0, 0, 0, 0, 0, 0, 0, 0,
];
const K_PIKIT_TAB0: [u8; 8] = [24, 24, 0, 48, 48, 48, 0, 24];
const K_PIKIT_TAB1: [u8; 8] = [0, 24, 24, 24, 0, 48, 48, 48];
const K_STALFOS_ANIM_STATE2: [u8; 4] = [8, 9, 10, 11];
const K_STALFOS_CHECK_DIR: [u8; 4] = [3, 2, 1, 0];
const K_STALFOS_ANIM_STATE1: [u8; 8] = [6, 4, 0, 2, 7, 5, 1, 3];
const K_STALFOS_DELAY: [u8; 4] = [16, 32, 64, 32];
const K_ZAZAK_DIR2_MOTHULA: [u8; 8] = [2, 3, 2, 3, 0, 1, 0, 1];
const K_FIREBALL_JUNCTION_X: [i8; 4] = [12, -12, 0, 0];
const K_FIREBALL_JUNCTION_Y: [i8; 4] = [0, 0, 12, -12];
const K_FIREBALL_JUNCTION_XYVEL: [i8; 6] = [0, 0, 40, -40, 0, 0];
const K_GIBO_OAM_FLAGS_MOTHULA: [u8; 4] = [0, 0x40, 0xc0, 0x80];
const K_GIBO_XVEL: [i8; 8] = [16, 16, 0, -16, -16, -16, 0, 16];
const K_GIBO_YVEL: [i8; 8] = [0, 0, 16, -16, 16, 16, -16, -16];
const K_TEKITE_DIR: [u8; 4] = [3, 2, 1, 0];
const K_TEKITE_XVEL: [i8; 4] = [16, -16, 16, -16];
const K_TEKITE_YVEL: [i8; 4] = [16, 16, -16, -16];
const K_HOVER_OAM_FLAGS: [u8; 4] = [0x40, 0, 0x40, 0];
const K_HOVER_ACCEL_X0: [i8; 4] = [1, -1, 1, -1];
const K_HOVER_ACCEL_Y0: [i8; 4] = [1, 1, -1, -1];
const K_HOVER_ACCEL_X1: [i8; 4] = [-1, 1, -1, 1];
const K_HOVER_ACCEL_Y1: [i8; 4] = [-1, -1, 1, 1];
const K_CHAIN_CHOMP_XVEL: [i8; 16] = [
    0, 8, 11, 14, 16, 14, 11, 8, 0, -8, -11, -14, -16, -14, -11, -8,
];
const K_CHAIN_CHOMP_YVEL: [i8; 16] = [
    -16, -14, -11, -8, 0, 8, 11, 14, 16, 14, 11, 8, 0, -9, -11, -14,
];
const K_HOKBOK_B: [u8; 8] = [8, 7, 6, 5, 4, 5, 6, 7];
const K_BOULDER_ZVEL: [i8; 2] = [32, 48];
const K_BOULDER_YVEL: [i8; 2] = [8, 32];
const K_BOULDER_XVEL: [i8; 4] = [24, 16, -24, -16];

impl ZeldaState {
    // void Sprite_Wizzbeam(int k) {
    pub(super) fn sprite_wizzbeam(&mut self, k: usize) {
        self.wizzbeam_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.ram[SPRITE_OAM_FLAGS + k] ^= 6;
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        if self.ram[SPRITE_AI_STATE + k] == 0 {
            self.sprite_check_damage_to_link(k);
        }
        self.sprite_move_xy(k);
        if self.sprite_check_tile_collision(k) != 0 {
            self.ram[SPRITE_STATE + k] = 0;
        }
    }

    // void Sprite_9B_Wizzrobe(int k) {  // 9e9d1b
    pub(super) fn sprite_9_b_wizzrobe(&mut self, k: usize) {
        if self.ram[SPRITE_C + k] != 0 {
            self.sprite_wizzbeam(k);
            return;
        }

        if self.ram[SPRITE_AI_STATE + k] == 0
            || ((self.ram[SPRITE_AI_STATE + k] & 1) != 0
                && (self.ram[SPRITE_DELAY_MAIN + k] & 1) != 0)
        {
            let mut info = SpritePrepOamCoordsRet {
                x: 0,
                y: 0,
                r4: 0,
                flags: 0,
            };
            self.sprite_prep_oam_coord(k, &mut info);
        } else {
            self.wizzrobe_draw(k);
        }

        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.ram[SPRITE_IGNORE_PROJECTILE + k] = 1;

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_X_VEL + k] = 1;
                    self.ram[SPRITE_Y_VEL + k] = 1;
                    if self.sprite_check_tile_collision(k) == 0 {
                        self.ram[SPRITE_AI_STATE + k] = 1;
                        self.ram[SPRITE_DELAY_MAIN + k] = 63;
                        let j = self.sprite_direction_to_face_link(k, None);
                        self.ram[SPRITE_D + k] = j;
                        self.ram[SPRITE_GRAPHICS + k] = K_WIZZROBE_CLOAK_GFX[usize::from(j)];
                    } else {
                        self.ram[SPRITE_STATE + k] = 0;
                    }
                }
            }
            1 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[SPRITE_DELAY_MAIN + k] = 63;
                }
            }
            2 => {
                self.ram[SPRITE_IGNORE_PROJECTILE + k] = 0;
                self.sprite_check_damage_to_and_from_link(k);
                let j = self.ram[SPRITE_DELAY_MAIN + k];
                if j == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 3;
                    self.ram[SPRITE_DELAY_MAIN + k] = 63;
                    return;
                }
                if j == 32 {
                    self.wizzrobe_fire_beam(k);
                }
                self.ram[SPRITE_GRAPHICS + k] = K_WIZZROBE_ATTACK_GFX[usize::from(j >> 3)]
                    + K_WIZZROBE_ATTACK_DIR_GFX[usize::from(self.ram[SPRITE_D + k])];
            }
            3 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    if self.ram[SPRITE_B + k] != 0 {
                        self.ram[SPRITE_STATE + k] = 0;
                    }
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    self.ram[SPRITE_DELAY_MAIN + k] =
                        (self.get_random_number() & 31).wrapping_add(32);
                }
            }
            _ => {}
        }
    }

    // void Sprite_9A_Kyameron(int k) {  // 9e9e7b
    pub(super) fn sprite_9_a_kyameron(&mut self, k: usize) {
        if self.ram[SPRITE_AI_STATE + k] == 0 {
            let mut info = SpritePrepOamCoordsRet {
                x: 0,
                y: 0,
                r4: 0,
                flags: 0,
            };
            self.sprite_prep_oam_coord(k, &mut info);
        } else {
            self.kyameron_draw(k);
        }

        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.ram[SPRITE_IGNORE_PROJECTILE + k] = 1;

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.ram[SPRITE_DELAY_MAIN + k] =
                        (self.get_random_number() & 63).wrapping_add(96);
                    self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_A + k];
                    self.ram[SPRITE_X_HI + k] = self.ram[SPRITE_B + k];
                    self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_C + k];
                    self.ram[SPRITE_Y_HI + k] = self.ram[SPRITE_HEAD_DIR + k];
                    self.ram[SPRITE_SUBTYPE2 + k] = 5;
                    self.ram[SPRITE_GRAPHICS + k] = 8;
                }
            }
            1 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_DELAY_MAIN + k] = 31;
                    self.ram[SPRITE_AI_STATE + k] = 2;
                }
                self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_sub(1);
                if sign8(self.ram[SPRITE_SUBTYPE2 + k]) {
                    self.ram[SPRITE_SUBTYPE2 + k] = 5;
                    self.ram[SPRITE_GRAPHICS + k] =
                        (self.ram[SPRITE_GRAPHICS + k].wrapping_add(1) & 3).wrapping_add(8);
                }
            }
            2 => {
                let j = self.ram[SPRITE_DELAY_MAIN + k];
                if j == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 3;
                    let idx = usize::from(
                        self.sprite_is_below_link(k).a * 2 + self.sprite_is_right_of_link(k).a,
                    );
                    self.ram[SPRITE_X_VEL + k] = K_KYAMERON_XVEL[idx] as u8;
                    self.ram[SPRITE_Y_VEL + k] = K_KYAMERON_YVEL[idx] as u8;
                } else {
                    if j == 7 {
                        self.sprite_set_y(k, self.sprite_get_y(k).wrapping_sub(29));
                    }
                    self.ram[SPRITE_GRAPHICS + k] = K_KYAMERON_COAGULATE_GFX[usize::from(j >> 2)];
                }
            }
            3 => {
                self.ram[SPRITE_IGNORE_PROJECTILE + k] = 0;
                let mut should_disperse = false;
                if !self.sprite_check_damage_to_and_from_link(k) {
                    self.sprite_move_xy(k);
                    let j = self.sprite_check_tile_collision(k);
                    if (j & 3) != 0 {
                        self.ram[SPRITE_X_VEL + k] = (0u8).wrapping_sub(self.ram[SPRITE_X_VEL + k]);
                        self.ram[SPRITE_ANIM_CLOCK + k] =
                            self.ram[SPRITE_ANIM_CLOCK + k].wrapping_add(1);
                    }
                    if (j & 12) != 0 {
                        self.ram[SPRITE_Y_VEL + k] = (0u8).wrapping_sub(self.ram[SPRITE_Y_VEL + k]);
                        self.ram[SPRITE_ANIM_CLOCK + k] =
                            self.ram[SPRITE_ANIM_CLOCK + k].wrapping_add(1);
                    }
                    if self.ram[SPRITE_ANIM_CLOCK + k] >= 3 {
                        should_disperse = true;
                    }
                } else {
                    should_disperse = true;
                }
                if should_disperse {
                    self.ram[SPRITE_AI_STATE + k] = 4;
                    self.ram[SPRITE_DELAY_MAIN + k] = 15;
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x28);
                }
                self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                self.ram[SPRITE_GRAPHICS + k] =
                    K_KYAMERON_MOVING_GFX[usize::from((self.ram[SPRITE_SUBTYPE2 + k] >> 3) & 3)];
                if (((k as u8) ^ self.ram[FRAME_COUNTER]) & 7) == 0 {
                    let x = u16::from(self.get_random_number() & 0x0f).wrapping_sub(4);
                    let y = u16::from(self.get_random_number() & 0x0f).wrapping_sub(4);
                    self.sprite_garnish_spawn_sparkle(k, x, y);
                }
            }
            4 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_ANIM_CLOCK + k] = 0;
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    self.ram[SPRITE_Z + k] = 0;
                    self.ram[SPRITE_DELAY_MAIN + k] = 64;
                } else {
                    self.ram[SPRITE_GRAPHICS + k] = (self.ram[SPRITE_DELAY_MAIN + k] >> 2) + 15;
                }
            }
            _ => {}
        }
    }

    // void Sprite_99_Pengator(int k) {  // 9ea196
    pub(super) fn sprite_99_pengator(&mut self, k: usize) {
        self.ram[SPRITE_GRAPHICS + k] = self.ram[SPRITE_A + k]
            .wrapping_add(K_PENGATOR_GFX[usize::from(self.ram[SPRITE_D + k])]);
        self.pengator_draw(k);
        if self.ram[SPRITE_F + k] != 0 || (self.ram[SPRITE_WALLCOLL + k] & 15) != 0 {
            self.ram[SPRITE_AI_STATE + k] = 0;
            self.ram[SPRITE_X_VEL + k] = 0;
            self.ram[SPRITE_Y_VEL + k] = 0;
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xyz(k);
        self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
        if sign8(self.ram[SPRITE_Z + k]) {
            self.ram[SPRITE_Z_VEL + k] = 0;
            self.ram[SPRITE_Z + k] = 0;
        }
        self.sprite_check_tile_collision(k);

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.ram[SPRITE_D + k] = self.sprite_direction_to_face_link(k, None);
                self.ram[SPRITE_AI_STATE + k] = 1;
            }
            1 => {
                if (((k as u8) ^ self.ram[FRAME_COUNTER]) & 3) == 0 {
                    let mut flag = false;
                    let j = usize::from(self.ram[SPRITE_D + k]);
                    if self.ram[SPRITE_X_VEL + k] != K_FLUTE_BOY_ANIMAL_XVEL_MOTHULA[j] as u8 {
                        self.ram[SPRITE_X_VEL + k] =
                            self.ram[SPRITE_X_VEL + k].wrapping_add(K_PENGATOR_XY_VEL[j] as u8);
                        flag = true;
                    }
                    if self.ram[SPRITE_Y_VEL + k] != K_ZAZAK_YVEL_MOTHULA[j] as u8 {
                        self.ram[SPRITE_Y_VEL + k] =
                            self.ram[SPRITE_Y_VEL + k].wrapping_add(K_PENGATOR_XY_VEL[j + 2] as u8);
                        flag = true;
                    }
                    if !flag {
                        self.ram[SPRITE_DELAY_MAIN + k] = 15;
                        self.ram[SPRITE_AI_STATE + k] = 2;
                    }
                }
                self.ram[SPRITE_A + k] = (self.ram[FRAME_COUNTER] & 4) >> 2;
            }
            2 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                } else if self.ram[SPRITE_DELAY_MAIN + k] == 5 {
                    self.ram[SPRITE_Z_VEL + k] = 24;
                }
                self.ram[SPRITE_A + k] =
                    K_PENGATOR_JUMP[usize::from(self.ram[SPRITE_DELAY_MAIN + k] >> 2)];
            }
            3 => {
                if ((((k as u8) ^ self.ram[FRAME_COUNTER]) & 7) | self.ram[SPRITE_Z + k]) == 0 {
                    let i = usize::from(self.ram[SPRITE_D + k]);
                    let base = usize::from(i >= 2) * 4;
                    let x = K_PENGATOR_GARNISH_X[usize::from(self.get_random_number() & 3) + base];
                    let y = K_PENGATOR_GARNISH_Y[usize::from(self.get_random_number() & 3) + base];
                    self.sprite_garnish_spawn_sparkle_limited(k, x as i16 as u16, y as i16 as u16);
                }
            }
            _ => {}
        }
    }

    // void Sprite_9E_HauntedGroveOstritch(int k) {  // 9e995b
    pub(super) fn sprite_9_e_haunted_grove_ostritch(&mut self, k: usize) {
        self.flute_boy_ostrich_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.ram[SPRITE_GRAPHICS + k] = if (self.ram[FRAME_COUNTER] & 0x18) != 0 {
                    3
                } else {
                    0
                };
                if self.ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH] != 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.ram[SPRITE_Y_VEL + k] = (-8i8) as u8;
                    self.ram[SPRITE_X_VEL + k] = (-16i8) as u8;
                }
            }
            1 => {
                self.sprite_move_xyz(k);
                self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
                if sign8(self.ram[SPRITE_Z + k]) {
                    self.ram[SPRITE_Z_VEL + k] = 32;
                    self.ram[SPRITE_Z + k] = 0;
                    self.ram[SPRITE_SUBTYPE2 + k] = 0;
                    self.ram[SPRITE_A + k] = 0;
                }
                self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                if (self.ram[SPRITE_SUBTYPE2 + k] & 7) == 0 && self.ram[SPRITE_A + k] != 3 {
                    self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_add(1);
                }
                self.ram[SPRITE_GRAPHICS + k] =
                    K_FLUTE_BOY_OSTRICH_GFX[usize::from(self.ram[SPRITE_A + k])];
            }
            _ => {}
        }
    }

    // void Sprite_9F_HauntedGroveRabbit(int k) {  // 9e9a6d
    pub(super) fn sprite_9_f_haunted_grove_rabbit(&mut self, k: usize) {
        self.ram[SPRITE_OAM_FLAGS + k] = (self.ram[SPRITE_OAM_FLAGS + k] & !0x40)
            | K_FLUTE_BOY_ANIMAL_OAM_FLAGS[usize::from(self.ram[SPRITE_D + k])];
        self.sprite_draw_single_large(k);

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.ram[SPRITE_GRAPHICS + k] = 3;
                if self.ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH] != 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.ram[SPRITE_D + k] ^= 1;
                    self.ram[SPRITE_X_VEL + k] =
                        K_FLUTE_BOY_ANIMAL_XVEL_MOTHULA[usize::from(self.ram[SPRITE_D + k])] as u8;
                    self.ram[SPRITE_Y_VEL + k] = (-8i8) as u8;
                }
            }
            1 => {
                self.sprite_move_xyz(k);
                self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(3);
                if sign8(self.ram[SPRITE_Z + k]) {
                    self.ram[SPRITE_Z_VEL + k] = 24;
                    self.ram[SPRITE_Z + k] = 0;
                    self.ram[SPRITE_SUBTYPE2 + k] = 0;
                    self.ram[SPRITE_A + k] = 0;
                }
                self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                if (self.ram[SPRITE_SUBTYPE2 + k] & 3) == 0 && self.ram[SPRITE_A + k] != 2 {
                    self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_add(1);
                }
                self.ram[SPRITE_GRAPHICS + k] =
                    K_FLUTE_BOY_ANIMAL_GFX[usize::from(self.ram[SPRITE_A + k])];
            }
            _ => {}
        }
    }

    // void Sprite_A0_HauntedGroveBird(int k) {  // 9e9aec
    pub(super) fn sprite_a0_haunted_grove_bird(&mut self, k: usize) {
        if self.ram[SPRITE_GRAPHICS + k] == 3 {
            self.haunted_grove_bird_blink(k);
        }
        self.ram[SPRITE_OAM_FLAGS + k] = (self.ram[SPRITE_OAM_FLAGS + k] & !0x40)
            | K_FLUTE_BOY_ANIMAL_OAM_FLAGS[usize::from(self.ram[SPRITE_D + k])];
        let cur = read_le_u16(&self.ram, OAM_CUR_PTR);
        write_le_u16(&mut self.ram, OAM_CUR_PTR, cur.wrapping_add(4));
        let ext = read_le_u16(&self.ram, OAM_EXT_CUR_PTR);
        write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, ext.wrapping_add(1));
        self.ram[SPRITE_FLAGS2 + k] = self.ram[SPRITE_FLAGS2 + k].wrapping_sub(1);
        self.sprite_draw_single_large(k);
        self.ram[SPRITE_FLAGS2 + k] = self.ram[SPRITE_FLAGS2 + k].wrapping_add(1);
        self.sprite_move_xyz(k);

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.ram[SPRITE_GRAPHICS + k] = if (self.ram[FRAME_COUNTER] & 0x18) != 0 {
                    0
                } else {
                    3
                };
                if self.ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH] != 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.ram[SPRITE_D + k] ^= 1;
                    self.ram[SPRITE_X_VEL + k] =
                        K_FLUTE_BOY_ANIMAL_XVEL_MOTHULA[usize::from(self.ram[SPRITE_D + k])] as u8;
                    self.ram[SPRITE_DELAY_MAIN + k] = 32;
                    self.ram[SPRITE_Z_VEL + k] = 16;
                    self.ram[SPRITE_Y_VEL + k] = (-8i8) as u8;
                }
            }
            1 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_add(2);
                    if !sign8(self.ram[SPRITE_Z_VEL + k].wrapping_sub(0x10)) {
                        self.ram[SPRITE_AI_STATE + k] = 2;
                    }
                }
                self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                self.ram[SPRITE_GRAPHICS + k] = ((self.ram[SPRITE_SUBTYPE2 + k] >> 1) & 1) + 1;
            }
            2 => {
                self.ram[SPRITE_GRAPHICS + k] = 1;
                self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(1);
                if sign8(self.ram[SPRITE_Z_VEL + k].wrapping_add(15)) {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                }
            }
            _ => {}
        }
    }

    // void HauntedGroveBird_Blink(int k) {  // 9e9b9c
    pub(super) fn haunted_grove_bird_blink(&mut self, k: usize) {
        let Some((x, y, flags)) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return;
        };
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let j = usize::from(self.ram[SPRITE_D + k]);
        self.ram[oam] = x.wrapping_add(K_FLUTE_BOY_BIRD_X[j] as i16 as u16) as u8;
        self.ram[oam + 1] = y as u8;
        self.ram[oam + 2] = 0xae;
        self.ram[oam + 3] = flags | K_FLUTE_BOY_ANIMAL_OAM_FLAGS[j];
        self.sprite_correct_oam_entries(k, 0, 0);
    }

    // void Sprite_A4_FallingIce(int k) {  // 9e9710
    pub(super) fn sprite_a4_falling_ice(&mut self, k: usize) {
        if self.ram[SPRITE_C + k] == 0 {
            if self.sprite_return_if_inactive(k) {
                return;
            }
            if self.ram[SPRITE_STATE + 2] < 9
                && self.ram[SPRITE_STATE + 3] < 9
                && self.ram[SPRITE_STATE + 4] < 9
            {
                self.ram[SPRITE_STATE + k] = 0;
            }
            self.generate_iceball(k);
            return;
        }

        self.ram[SPRITE_IGNORE_PROJECTILE + k] = 1;
        self.ram[SPRITE_OBJ_PRIO + k] = 0x30;
        self.sprite_draw_single_large(k);
        if self.ram[SPRITE_AI_STATE + k] == 0 {
            self.ram[SPRITE_FLAGS3 + k] ^= 16;
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
            if self.ram[SPRITE_DELAY_MAIN + k] == 1 {
                self.ram[SPRITE_STATE + k] = 0;
            }
            self.ram[SPRITE_GRAPHICS + k] = (self.ram[SPRITE_DELAY_MAIN + k] >> 3) + 2;
            return;
        }

        self.sprite_move_xy(k);
        let mut hit_solid = false;
        if self.ram[SPRITE_AI_STATE + k] != 0 {
            self.sprite_check_damage_to_link(k);
            hit_solid = self.sprite_check_tile_collision(k) != 0;
        }
        if self.ram[SPRITE_AI_STATE + k] == 0 || !hit_solid {
            let old_z = self.ram[SPRITE_Z + k];
            self.sprite_move_z(k);
            if !sign8(self.ram[SPRITE_Z_VEL + k].wrapping_add(64)) {
                self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(3);
            }
            if !(sign8(old_z ^ self.ram[SPRITE_Z + k]) && sign8(self.ram[SPRITE_Z + k])) {
                return;
            }
            self.ram[SPRITE_Z + k] = 0;
            if self.ram[SPRITE_AI_STATE + k] == 0 {
                self.ram[SPRITE_STATE + k] = 0;
                self.ice_ball_split(k);
                return;
            }
        }
        self.ram[SPRITE_DELAY_MAIN + k] = 15;
        self.ram[SPRITE_OAM_FLAGS + k] = 4;
        if self.ram[SOUND_EFFECT_1] == 0 {
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x1e);
            self.ram[SPRITE_GRAPHICS + k] = 3;
        }
    }

    // void Sprite_A1_Freezor(int k) {  // 9e981d
    pub(super) fn sprite_a1_freezor(&mut self, k: usize) {
        self.freezor_draw(k);
        if self.ram[SPRITE_STATE + k] != 9 {
            self.ram[SPRITE_AI_STATE + k] = 3;
            self.ram[SPRITE_DELAY_MAIN + k] = 31;
            self.ram[SPRITE_IGNORE_PROJECTILE + k] = 31;
            self.ram[SPRITE_STATE + k] = 9;
            self.ram[SPRITE_HIT_TIMER + k] = 0;
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.ram[SPRITE_AI_STATE + k] != 3 && self.sprite_return_if_recoiling(k) {
            return;
        }

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.ram[SPRITE_IGNORE_PROJECTILE + k] =
                    self.ram[SPRITE_IGNORE_PROJECTILE + k].wrapping_add(1);
                if self.sprite_is_right_of_link(k).b.wrapping_add(16) < 32 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.ram[SPRITE_DELAY_MAIN + k] = 32;
                }
            }
            1 => {
                self.ram[SPRITE_IGNORE_PROJECTILE + k] = self.ram[SPRITE_DELAY_MAIN + k];
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    let x = self.sprite_get_x(k).wrapping_sub(5);
                    let y = self.sprite_get_y(k);
                    self.dungeon_update_tile_map_with_common_tile_for_mothula(x, y, 8);
                    self.ram[SPRITE_DELAY_AUX1 + k] = 96;
                    self.ram[SPRITE_D + k] = 2;
                    self.ram[SPRITE_DELAY_MAIN + k] = 80;
                } else {
                    self.ram[SPRITE_X_VEL + k] = if (self.ram[SPRITE_DELAY_MAIN + k] & 1) != 0 {
                        (-16i8) as u8
                    } else {
                        16
                    };
                    self.sprite_move_x(k);
                }
            }
            2 => {
                self.sprite_check_damage_to_link(k);
                if self.sprite_check_damage_from_link(k) != 0 {
                    self.ram[SPRITE_HIT_TIMER + k] = 0;
                }
                if self.ram[SPRITE_DELAY_AUX1 + k] != 0
                    && (((k as u8) ^ self.ram[FRAME_COUNTER]) & 7) == 0
                {
                    let x = K_FREEZOR_SPARKLE_X[usize::from(self.get_random_number() & 7)] as i16
                        as u16;
                    self.sprite_garnish_spawn_sparkle(k, x, (-4i16) as u16);
                }
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_D + k] = self.sprite_direction_to_face_link(k, None);
                }
                let j = usize::from(self.ram[SPRITE_D + k]);
                self.ram[SPRITE_X_VEL + k] = K_FREEZOR_XVEL[j] as u8;
                self.ram[SPRITE_Y_VEL + k] = K_FREEZOR_YVEL[j] as u8;
                if (self.ram[SPRITE_WALLCOLL + k] & 15) == 0 {
                    self.sprite_move_xy(k);
                }
                self.sprite_check_tile_collision(k);
                self.ram[SPRITE_GRAPHICS + k] = K_FREEZOR_MOVING_GFX
                    [usize::from(((k as u8) ^ self.ram[FRAME_COUNTER]) >> 2 & 3)];
            }
            3 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.sprite_manually_set_death_flag_uw(k);
                    self.ram[SPRITE_STATE + k] = 0;
                }
                self.ram[SPRITE_GRAPHICS + k] =
                    K_FREEZOR_MELTING_GFX[usize::from(self.ram[SPRITE_DELAY_MAIN + k] >> 3)];
            }
            _ => {}
        }
    }

    // void Sprite_A3_KholdstareShell(int k) {  // 9e9460
    pub(super) fn sprite_a3_kholdstare_shell(&mut self, k: usize) {
        if self.sprite_return_if_paused(k) {
            return;
        }
        let mut pt = PointU8 { x: 0, y: 0 };
        self.sprite_direction_to_face_link(k, Some(&mut pt));
        if pt.x.wrapping_add(32) < 64 && pt.y.wrapping_add(32) < 64 {
            self.sprite_nullify_hookshot_drag();
            self.sprite_repel_dash();
        }
        self.sprite_check_damage_from_link(k);
        if self.ram[SPRITE_AI_STATE + k] == 0 {
            if self.ram[SPRITE_STATE + k] == 6 {
                self.ram[SPRITE_FLAGS3 + k] = 0xc0;
                self.ram[SPRITE_AI_STATE + k] = 1;
                self.ram[SPRITE_STATE + k] = 9;
            } else if self.ram[SPRITE_HIT_TIMER + k] != 0 {
                let x_offs = if (self.ram[SPRITE_HIT_TIMER + k] & 2) != 0 {
                    0xffff
                } else {
                    1
                };
                write_le_u16(&mut self.ram, DUNG_FLOOR_X_OFFS, x_offs);
                self.ram[DUNG_HDR_COLLISION_2_MIRROR] = 1;
            } else {
                self.ram[DUNG_HDR_COLLISION_2_MIRROR] = 0;
            }
        } else {
            let state = self.ram[SPRITE_AI_STATE + k];
            self.ram[SPRITE_AI_STATE + k] = state.wrapping_add(1);
            if state != 18 {
                self.KholdstareShell_PaletteFiltering();
            } else {
                self.ram[SPRITE_STATE + k] = 0;
                self.ram[SPRITE_AI_STATE + 2] = 2;
                self.ram[SPRITE_DELAY_MAIN + 2] = 128;
            }
        }
    }

    // void Sprite_A2_Kholdstare(int k) {  // 9e9518
    pub(super) fn sprite_a2_kholdstare(&mut self, k: usize) {
        self.kholdstare_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.ram[SPRITE_AI_STATE + k] < 2 {
            self.kholdstare_spawn_puff_cloud_garnish(k);
            if (self.ram[FRAME_COUNTER] & 7) == 0 {
                self.ram[SOUND_EFFECT_1] = 2;
            }
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }

        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_sub(1);
        if sign8(self.ram[SPRITE_SUBTYPE2 + k]) {
            self.ram[SPRITE_SUBTYPE2 + k] = 10;
            self.ram[SPRITE_GRAPHICS + k] = self.ram[SPRITE_GRAPHICS + k].wrapping_add(1) & 3;
        }

        if (self.ram[FRAME_COUNTER] & 3) == 0 {
            let pt = self.sprite_project_speed_towards_link(k, 31);
            self.ram[SPRITE_A + k] = ZeldaState::sprite_convert_velocity_to_angle(pt.x, pt.y);
        }

        self.sprite_move_xy(k);
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.sprite_check_damage_to_and_from_link(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.ram[SPRITE_DELAY_MAIN + k] = (self.get_random_number() & 63) + 32;
                    return;
                }
                let x_delta = self.ram[SPRITE_X_VEL + k].wrapping_sub(self.ram[SPRITE_Z_VEL + k]);
                if x_delta != 0 {
                    self.ram[SPRITE_X_VEL + k] = self.ram[SPRITE_X_VEL + k]
                        .wrapping_add(if sign8(x_delta) { 1 } else { 0xff });
                }
                let y_delta =
                    self.ram[SPRITE_Y_VEL + k].wrapping_sub(self.ram[SPRITE_Z_SUBPOS + k]);
                if y_delta != 0 {
                    self.ram[SPRITE_Y_VEL + k] = self.ram[SPRITE_Y_VEL + k]
                        .wrapping_add(if sign8(y_delta) { 1 } else { 0xff });
                }
                self.kholdstare_check_collision_for_mothula(k);
            }
            1 => {
                self.sprite_check_damage_to_and_from_link(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    self.ram[SPRITE_DELAY_MAIN + k] = (self.get_random_number() & 63) + 96;
                    let j = self.get_random_number();
                    if (j & 0x1c) == 0 {
                        let pt = self.sprite_project_speed_towards_link(k, 24);
                        self.ram[SPRITE_Z_VEL + k] = pt.x;
                        self.ram[SPRITE_Z_SUBPOS + k] = pt.y;
                    } else {
                        let i = usize::from(j & 3);
                        self.ram[SPRITE_Z_VEL + k] = K_KHOLDSTARE_TARGET_XVEL[i] as u8;
                        self.ram[SPRITE_Z_SUBPOS + k] = K_KHOLDSTARE_TARGET_YVEL[i] as u8;
                    }
                } else {
                    if self.ram[SPRITE_X_VEL + k] != 0 {
                        self.ram[SPRITE_X_VEL + k] = self.ram[SPRITE_X_VEL + k].wrapping_add(
                            if sign8(self.ram[SPRITE_X_VEL + k]) {
                                1
                            } else {
                                0xff
                            },
                        );
                    }
                    if self.ram[SPRITE_Y_VEL + k] != 0 {
                        self.ram[SPRITE_Y_VEL + k] = self.ram[SPRITE_Y_VEL + k].wrapping_add(
                            if sign8(self.ram[SPRITE_Y_VEL + k]) {
                                1
                            } else {
                                0xff
                            },
                        );
                    }
                    self.kholdstare_check_collision_for_mothula(k);
                }
            }
            2 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 1 {
                    self.ram[SPRITE_STATE + k] = 0;
                    self.ram[SPRITE_STATE + k + 1] = 0;
                    self.ram[SPRITE_STATE + k + 2] = 0;
                    for i in (0..=2usize).rev() {
                        let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
                        let j = self.sprite_spawn_dynamically_ex(k, 0xa2, &mut info, 4);
                        if j >= 0 {
                            let j = j as usize;
                            self.sprite_set_spawned_coordinates(j, &info);
                            self.ram[SPRITE_Z_VEL + j] = K_KHOLDSTARE_TRIPLICATE_TAB0[i] as u8;
                            self.ram[SPRITE_Z_SUBPOS + j] = K_KHOLDSTARE_TRIPLICATE_TAB1[i] as u8;
                            self.ram[SPRITE_DELAY_MAIN + j] = 32;
                        }
                    }
                    self.ram[TMP_COUNTER] = 0xff;
                } else {
                    self.ram[SPRITE_HIT_TIMER + k] |= 0xe0;
                }
            }
            _ => {}
        }
    }

    // void Sprite_86_Kodongo(int k) {  // 9ec103
    pub(super) fn sprite_86_kodongo(&mut self, k: usize) {
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.ram[SPRITE_FLAGS + k] = 0;

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                self.ram[SPRITE_D + k] = self.get_random_number() & 3;
                self.ram[SPRITE_FLAGS + k] = 176;
                loop {
                    let j = usize::from(self.ram[SPRITE_D + k]);
                    self.ram[SPRITE_X_VEL + k] = K_KODONDO_XVEL[j] as u8;
                    self.ram[SPRITE_Y_VEL + k] = K_KODONDO_YVEL[j] as u8;
                    if self.sprite_check_tile_collision(k) == 0 {
                        break;
                    }
                    self.ram[SPRITE_D + k] = self.ram[SPRITE_D + k].wrapping_add(1) & 3;
                }
                self.kodongo_set_direction(k);
            }
            1 => {
                self.sprite_move_xy(k);
                if self.sprite_check_tile_collision(k) != 0 {
                    self.ram[SPRITE_D + k] ^= 1;
                    self.kodongo_set_direction(k);
                }
                if (self.ram[SPRITE_X_LO + k] & 0x1f) == 4
                    && (self.ram[SPRITE_Y_LO + k] & 0x1f) == 0x1b
                    && (self.get_random_number() & 3) == 0
                {
                    self.ram[SPRITE_DELAY_MAIN + k] = 111;
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[SPRITE_A + k] = 0;
                }
                self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                let j = usize::from((self.ram[SPRITE_SUBTYPE2 + k] & 4) | self.ram[SPRITE_D + k]);
                self.ram[SPRITE_GRAPHICS + k] = K_KODONDO_GFX[j];
                self.ram[SPRITE_OAM_FLAGS + k] =
                    (self.ram[SPRITE_OAM_FLAGS + k] & !0x40) | K_KODONDO_OAM_FLAGS[j];
            }
            2 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 0;
                }
                let j = u8::from(self.ram[SPRITE_DELAY_MAIN + k].wrapping_sub(0x20) < 0x30);
                if j != 0 && (self.ram[SPRITE_DELAY_MAIN + k] & 0x0f) == 0 {
                    self.kodongo_spawn_fire(k);
                }
                self.ram[SPRITE_GRAPHICS + k] =
                    K_KODONDO_FLAME_GFX[usize::from(j * 4 + self.ram[SPRITE_D + k])];
            }
            _ => {}
        }
    }

    fn kholdstare_check_collision_for_mothula(&mut self, k: usize) {
        let j = self.sprite_check_tile_collision(k);
        if (j & 3) != 0 {
            self.ram[SPRITE_X_VEL + k] = self.ram[SPRITE_X_VEL + k].wrapping_neg();
            self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_neg();
        }
        if (j & 12) != 0 {
            self.ram[SPRITE_Y_VEL + k] = self.ram[SPRITE_Y_VEL + k].wrapping_neg();
            self.ram[SPRITE_Z_SUBPOS + k] = self.ram[SPRITE_Z_SUBPOS + k].wrapping_neg();
        }
    }

    // void Sprite_MadBatterBolt(int k) {  // 9e8a96
    pub(super) fn sprite_mad_batter_bolt(&mut self, k: usize) {
        const X: [u16; 8] = [0, 4, 8, 12, 12, 4, 8, 0];
        const Y: [u16; 8] = [0, 4, 8, 12, 12, 4, 8, 0];

        if (self.ram[SPRITE_SUBTYPE2 + k] & 16) != 0 {
            self.oam_allocate_from_region_b(4);
        }
        self.sprite_draw_single_small(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.ram[SPRITE_AI_STATE + k] == 0 {
            self.sprite_move_xy(k);
            if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                self.ram[SPRITE_AI_STATE + k] = 1;
            }
        } else {
            self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
            if self.ram[SPRITE_AI_STATE + k] == 0 {
                self.ram[SPRITE_STATE + k] = 0;
            }
            self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
            let j = self.ram[SPRITE_SUBTYPE2 + k];
            if (j & 7) == 0 {
                self.ram[SOUND_EFFECT_2] = 48;
            }
            self.sprite_set_x(
                k,
                self.player_state_view()
                    .x()
                    .wrapping_add(X[usize::from((j >> 2) & 7)]),
            );
            self.sprite_set_y(
                k,
                self.player_state_view()
                    .y()
                    .wrapping_add(Y[usize::from((j >> 4) & 7)]),
            );
        }
    }

    // void Sprite_AA_Pikit(int k) {  // 9e8bbf
    pub(super) fn sprite_aa_pikit(&mut self, k: usize) {
        self.pikit_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    self.ram[SPRITE_C + k] = self.ram[SPRITE_C + k].wrapping_add(1);
                    let j = if self.ram[SPRITE_C + k] == 4 {
                        self.ram[SPRITE_C + k] = 0;
                        self.sprite_direction_to_face_link(k, None)
                    } else {
                        self.get_random_number() & 3
                    };
                    let j = usize::from(j);
                    self.ram[SPRITE_X_VEL + k] = K_FLUTE_BOY_ANIMAL_XVEL_MOTHULA[j] as u8;
                    self.ram[SPRITE_Y_VEL + k] = K_ZAZAK_YVEL_MOTHULA[j] as u8;
                    self.ram[SPRITE_Z_VEL + k] = (self.get_random_number() & 7) + 19;
                }
                self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                self.ram[SPRITE_GRAPHICS + k] = (self.ram[SPRITE_SUBTYPE2 + k] >> 3) & 1;
            }
            1 => {
                self.sprite_move_xyz(k);
                self.sprite_check_tile_collision(k);
                self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
                if sign8(self.ram[SPRITE_Z + k]) {
                    self.ram[SPRITE_Z + k] = 0;
                    self.ram[SPRITE_Z_VEL + k] = 0;
                    let mut pt = PointU8 { x: 0, y: 0 };
                    self.sprite_direction_to_face_link(k, Some(&mut pt));
                    if pt.x.wrapping_add(48) < 96 && pt.y.wrapping_add(48) < 96 {
                        self.ram[SPRITE_AI_STATE + k] =
                            self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                        let pp = self.sprite_project_speed_towards_link(k, 31);
                        self.ram[SPRITE_D + k] =
                            ZeldaState::sprite_convert_velocity_to_angle(pp.x, pp.y) >> 1;
                        self.ram[SPRITE_DELAY_MAIN + k] = 95;
                        return;
                    }
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    self.ram[SPRITE_DELAY_MAIN + k] = 16;
                }
                self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                self.ram[SPRITE_GRAPHICS + k] = (self.ram[SPRITE_SUBTYPE2 + k] >> 3) & 1;
            }
            2 => {
                let mut j = self.ram[SPRITE_DELAY_MAIN + k];
                if j == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    self.ram[SPRITE_DELAY_MAIN + k] = 16;
                    self.ram[SPRITE_A + k] = 0;
                    self.ram[SPRITE_B + k] = 0;
                    self.ram[SPRITE_G + k] = 0;
                    return;
                }
                j >>= 2;
                self.ram[SPRITE_GRAPHICS + k] = K_PIKIT_GFX[usize::from(j)];
                let dir = usize::from(self.ram[SPRITE_D + k]);
                let xo = K_PIKIT_XY_OFFS[usize::from(j + K_PIKIT_TAB0[dir])];
                let yo = K_PIKIT_XY_OFFS[usize::from(j + K_PIKIT_TAB1[dir])];
                self.ram[SPRITE_A + k] = xo as u8;
                self.ram[SPRITE_B + k] = yo as u8;
                let x_delta = read_le_u16(&self.ram, CUR_SPRITE_X)
                    .wrapping_add(xo as i16 as u16)
                    .wrapping_sub(self.player_state_view().x())
                    .wrapping_add(12);
                let y_delta = read_le_u16(&self.ram, CUR_SPRITE_Y)
                    .wrapping_add(yo as i16 as u16)
                    .wrapping_sub(self.player_state_view().y())
                    .wrapping_add(12);
                if self.ram[SPRITE_G + k] == 0
                    && x_delta < 24
                    && y_delta < 32
                    && self.ram[SPRITE_DELAY_MAIN + k] < 46
                {
                    self.ram[SOUND_EFFECT_1] = self.link_calculate_sfx_pan() | 0x26;
                    let loot = (self.get_random_number() & 3) + 1;
                    self.ram[SPRITE_G + k] = loot;
                    self.ram[SPRITE_E + k] = loot;
                    match loot {
                        1 => {
                            if self.ram[LINK_ITEM_BOMBS] != 0 {
                                self.ram[LINK_ITEM_BOMBS] =
                                    self.ram[LINK_ITEM_BOMBS].wrapping_sub(1);
                            } else {
                                self.ram[SPRITE_G + k] = 0;
                            }
                        }
                        2 => {
                            if self.ram[LINK_NUM_ARROWS] != 0 {
                                self.ram[LINK_NUM_ARROWS] =
                                    self.ram[LINK_NUM_ARROWS].wrapping_sub(1);
                            } else {
                                self.ram[SPRITE_G + k] = 0;
                            }
                        }
                        3 => {
                            let rupees = read_le_u16(&self.ram, LINK_RUPEES_GOAL);
                            if rupees != 0 {
                                write_le_u16(
                                    &mut self.ram,
                                    LINK_RUPEES_GOAL,
                                    rupees.wrapping_sub(1),
                                );
                            } else {
                                self.ram[SPRITE_G + k] = 0;
                            }
                        }
                        _ => {
                            self.ram[SPRITE_SUBTYPE + k] = self.ram[LINK_SHIELD_TYPE];
                            if self.ram[LINK_SHIELD_TYPE] != 0 && self.ram[LINK_SHIELD_TYPE] != 3 {
                                self.ram[LINK_SHIELD_TYPE] = 0;
                            } else {
                                self.ram[SPRITE_G + k] = 0;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // void Sprite_A7_Stalfos(int k) {  // 9e906c
    pub(super) fn sprite_a7_stalfos(&mut self, k: usize) {
        if self.ram[SPRITE_A + k] != 0 {
            self.sprite_stalfos_bone(k);
            return;
        }
        if self.ram[SPRITE_E + k] == 0 {
            self.stalfos_skellington(k);
            return;
        }
        if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
            self.ram[SPRITE_X_VEL + k] = 1;
            self.ram[SPRITE_Y_VEL + k] = 1;
            if self.sprite_check_tile_collision(k) != 0 {
                self.ram[SPRITE_STATE + k] = 0;
                return;
            }
            self.ram[SPRITE_E + k] = 0;
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x15);
            self.sprite_spawn_poof_garnish(k);
            self.ram[SPRITE_DELAY_AUX2 + k] = 8;
            self.ram[SPRITE_DELAY_MAIN + k] = 64;
            self.ram[SPRITE_Y_VEL + k] = 0;
            self.ram[SPRITE_X_VEL + k] = 0;
        }
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_prep_oam_coord(k, &mut info);
    }

    // void Stalfos_Skellington(int k) {  // 9e90b5
    pub(super) fn stalfos_skellington(&mut self, k: usize) {
        if self.ram[SPRITE_STATE + k] == 9
            && self
                .player_state_view()
                .x()
                .wrapping_sub(read_le_u16(&self.ram, CUR_SPRITE_X))
                .wrapping_add(40)
                < 80
            && self
                .player_state_view()
                .y()
                .wrapping_sub(read_le_u16(&self.ram, CUR_SPRITE_Y))
                .wrapping_add(48)
                < 80
            && self.ram[PLAYER_OAM_Y_OFFSET] != 0x80
            && (self.ram[SPRITE_Z + k] | self.ram[SPRITE_PAUSE + k]) == 0
            && self.ram[SPRITE_FLOOR + k] == self.ram[LINK_IS_ON_LOWER_LEVEL]
        {
            let dir = self.sprite_direction_to_face_link(k, None);
            let mut should_jump = false;
            let mut may_check_dir = true;
            if self.ram[LINK_IS_RUNNING] == 0 {
                if self.ram[BUTTON_B_FRAMES] >= 0x90 {
                    should_jump = true;
                } else if !sign8(self.ram[BUTTON_B_FRAMES].wrapping_sub(9)) {
                    may_check_dir = false;
                }
            }
            let facing = usize::from((self.ram[LINK_DIRECTION_FACING] >> 1) & 3);
            if may_check_dir && (should_jump || dir != K_STALFOS_CHECK_DIR[facing]) {
                self.ram[SPRITE_D + k] = dir;
                let pt = self.sprite_project_speed_towards_link(k, 32);
                self.ram[SPRITE_X_VEL + k] = pt.x.wrapping_neg();
                self.ram[SPRITE_Y_VEL + k] = pt.y.wrapping_neg();
                self.ram[SPRITE_Z_VEL + k] = 32;
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x13);
                self.ram[SPRITE_Z + k] = self.ram[SPRITE_Z + k].wrapping_add(1);
            }
        }

        if self.ram[SPRITE_Z + k] == 0 {
            self.sprite_zazak_main(k);
            return;
        }
        self.ram[SPRITE_GRAPHICS + k] = K_STALFOS_ANIM_STATE2[usize::from(self.ram[SPRITE_D + k])];
        self.stalfos_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.ram[SPRITE_F + k] != 0 {
            self.ram[SPRITE_Z_VEL + k] = 0;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        let t = self.sprite_check_tile_collision(k);
        if (t & 3) != 0 {
            self.ram[SPRITE_X_VEL + k] = 0;
        }
        if (t & 12) != 0 {
            self.ram[SPRITE_Y_VEL + k] = 0;
        }
        self.sprite_move_xyz(k);
        self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
        if sign8(self.ram[SPRITE_Z + k].wrapping_sub(1)) {
            self.ram[SPRITE_Z + k] = 0;
            self.sprite_zero_velocity_xy(k);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x21);
            if self.ram[SPRITE_SUBTYPE + k] != 0 {
                self.ram[SPRITE_DELAY_AUX3_MOTHULA + k] = 16;
                self.ram[SPRITE_SUBTYPE2 + k] = 0;
            }
        }
    }

    // void Sprite_Zazak_Main(int k) {  // 9e919f
    pub(super) fn sprite_zazak_main(&mut self, k: usize) {
        if self.ram[SPRITE_B + k] != 0 {
            self.fire_phlegm_draw(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            self.ram[SPRITE_GRAPHICS + k] = (self.ram[FRAME_COUNTER] >> 1) & 1;
            self.sprite_check_damage_to_link(k);
            self.sprite_move_xy(k);
            if self.sprite_check_tile_collision(k) != 0 {
                self.ram[SPRITE_STATE + k] = 0;
                self.sprite_place_rupulse_spark_2(k);
            }
            return;
        }

        let t = self.ram[SPRITE_DELAY_AUX3_MOTHULA + k];
        let trace_stalfos_head = std::env::var_os("ZELDA3_TRACE_STALFOS_HEAD").is_some()
            && self.ram[SPRITE_TYPE + k] == 0xa7
            && k == 0
            && self.world_state_view().dungeon_room() == 0x00a8;
        if t != 0 {
            let old_head = self.ram[SPRITE_HEAD_DIR + k];
            self.ram[SPRITE_AI_STATE + k] = 0;
            self.ram[SPRITE_DELAY_MAIN + k] = 32;
            self.sprite_zero_velocity_xy(k);
            let face = self.sprite_direction_to_face_link(k, None);
            self.ram[SPRITE_HEAD_DIR + k] = face;
            if trace_stalfos_head {
                eprintln!(
                    "R stalfos head aux3 fc={} t=0x{:02x} x=0x{:04x} y=0x{:04x} old=0x{:02x} face=0x{:02x} d=0x{:02x} c=0x{:02x} delay=0x{:02x} rng=0x{:02x}",
                    self.ram[FRAME_COUNTER],
                    t,
                    self.sprite_get_x(k),
                    self.sprite_get_y(k),
                    old_head,
                    face,
                    self.ram[SPRITE_D + k],
                    self.ram[SPRITE_C + k],
                    self.ram[SPRITE_DELAY_MAIN + k],
                    self.ram[RNG_SEED],
                );
            }
        }
        if t == 1 {
            self.stalfos_throw_bone(k);
            self.ram[SPRITE_SUBTYPE2 + k] = 1;
        }
        self.ram[SPRITE_GRAPHICS + k] = K_STALFOS_ANIM_STATE1
            [usize::from((self.ram[SPRITE_SUBTYPE2 + k] & 1) * 4 + self.ram[SPRITE_D + k])];
        if self.ram[SPRITE_TYPE + k] == 0xa7 {
            self.stalfos_draw(k);
        } else {
            self.zazak_draw(k);
        }
        if std::env::var_os("ZELDA3_TRACE_STALFOS_INACTIVE").is_some()
            && self.ram[SPRITE_TYPE + k] == 0xa7
            && self.world_state_view().dungeon_room() == 0x00a8
        {
            eprintln!(
                "R stalfos inactive-check fc={} k={} x=0x{:04x} y=0x{:04x} state=0x{:02x} flag=0x{:02x} sub=0x{:02x} defl=0x{:02x} pause=0x{:02x} delay=0x{:02x} ai=0x{:02x} z=0x{:02x} f=0x{:02x} xr=0x{:02x} yr=0x{:02x} bump=0x{:02x}",
                self.ram[FRAME_COUNTER],
                k,
                self.sprite_get_x(k),
                self.sprite_get_y(k),
                self.ram[SPRITE_STATE + k],
                self.ram[FLAG_UNK1],
                self.frame_control_view().submodule(),
                self.ram[SPRITE_DEFL_BITS + k],
                self.ram[SPRITE_PAUSE + k],
                self.ram[SPRITE_DELAY_MAIN + k],
                self.ram[SPRITE_AI_STATE + k],
                self.ram[SPRITE_Z + k],
                self.ram[SPRITE_F + k],
                self.ram[SPRITE_X_RECOIL + k],
                self.ram[SPRITE_Y_RECOIL_MOTHULA + k],
                self.ram[SPRITE_BUMP_DAMAGE + k],
            );
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        let trace_stalfos = std::env::var_os("ZELDA3_TRACE_STALFOS").is_some()
            && self.ram[SPRITE_TYPE + k] == 0xa7
            && self.world_state_view().dungeon_room() == 0x00a8;
        if trace_stalfos {
            eprintln!(
                "R stalfos pre-move fc={} k={} x=0x{:04x} y=0x{:04x} d=0x{:02x} head=0x{:02x} ai=0x{:02x} delay=0x{:02x} g=0x{:02x} wall=0x{:02x} xv=0x{:02x} yv=0x{:02x} z=0x{:02x} zv=0x{:02x}",
                self.ram[FRAME_COUNTER],
                k,
                self.sprite_get_x(k),
                self.sprite_get_y(k),
                self.ram[SPRITE_D + k],
                self.ram[SPRITE_HEAD_DIR + k],
                self.ram[SPRITE_AI_STATE + k],
                self.ram[SPRITE_DELAY_MAIN + k],
                self.ram[SPRITE_G + k],
                self.ram[SPRITE_WALLCOLL + k],
                self.ram[SPRITE_X_VEL + k],
                self.ram[SPRITE_Y_VEL + k],
                self.ram[SPRITE_Z + k],
                self.ram[SPRITE_Z_VEL + k],
            );
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xy(k);
        let trace_tile = self.sprite_check_tile_collision(k);
        if trace_stalfos {
            eprintln!(
                "R stalfos post-move fc={} k={} x=0x{:04x} y=0x{:04x} d=0x{:02x} head=0x{:02x} ai=0x{:02x} delay=0x{:02x} g=0x{:02x} wall=0x{:02x} tile=0x{:02x} xv=0x{:02x} yv=0x{:02x}",
                self.ram[FRAME_COUNTER],
                k,
                self.sprite_get_x(k),
                self.sprite_get_y(k),
                self.ram[SPRITE_D + k],
                self.ram[SPRITE_HEAD_DIR + k],
                self.ram[SPRITE_AI_STATE + k],
                self.ram[SPRITE_DELAY_MAIN + k],
                self.ram[SPRITE_G + k],
                self.ram[SPRITE_WALLCOLL + k],
                trace_tile,
                self.ram[SPRITE_X_VEL + k],
                self.ram[SPRITE_Y_VEL + k],
            );
        }

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    let old_delay = self.ram[SPRITE_DELAY_MAIN + k];
                    let rng_before = self.ram[RNG_SEED];
                    let rng = self.get_random_number();
                    self.ram[SPRITE_DELAY_MAIN + k] = K_STALFOS_DELAY[usize::from(rng & 3)];
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    let j = usize::from(self.ram[SPRITE_HEAD_DIR + k]);
                    self.ram[SPRITE_D + k] = j as u8;
                    self.ram[SPRITE_X_VEL + k] = K_FLUTE_BOY_ANIMAL_XVEL_MOTHULA[j] as u8;
                    self.ram[SPRITE_Y_VEL + k] = K_ZAZAK_YVEL_MOTHULA[j] as u8;
                    if std::env::var_os("ZELDA3_TRACE_STALFOS_DELAY").is_some()
                        && self.ram[SPRITE_TYPE + k] == 0xa7
                        && self.world_state_view().dungeon_room() == 0x00a8
                    {
                        eprintln!(
                            "R stalfos delay fc={} k={} x=0x{:04x} y=0x{:04x} old_delay=0x{:02x} new_delay=0x{:02x} head=0x{:02x} d=0x{:02x} ai=0x{:02x} rng_before=0x{:02x} rng=0x{:02x}",
                            self.ram[FRAME_COUNTER],
                            k,
                            self.sprite_get_x(k),
                            self.sprite_get_y(k),
                            old_delay,
                            self.ram[SPRITE_DELAY_MAIN + k],
                            self.ram[SPRITE_HEAD_DIR + k],
                            self.ram[SPRITE_D + k],
                            self.ram[SPRITE_AI_STATE + k],
                            rng_before,
                            rng,
                        );
                    }
                }
            }
            1 => {
                if self.ram[SPRITE_WALLCOLL + k] != 0 {
                    self.ram[SPRITE_DELAY_MAIN + k] = 16;
                } else if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
                    self.ram[SPRITE_G + k] = self.ram[SPRITE_G + k].wrapping_sub(1);
                    if sign8(self.ram[SPRITE_G + k]) {
                        self.ram[SPRITE_G + k] = 11;
                        self.ram[SPRITE_SUBTYPE2 + k] =
                            self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                    }
                    return;
                } else if self.ram[SPRITE_TYPE + k] == 0xa6
                    && self.ram[SPRITE_D + k] == self.sprite_direction_to_face_link(k, None)
                    && self.ram[SPRITE_FLOOR + k] == self.ram[LINK_IS_ON_LOWER_LEVEL]
                {
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[SPRITE_DELAY_MAIN + k] = 48;
                    self.ram[SPRITE_DELAY_AUX1 + k] = 48;
                    self.ram[SPRITE_Y_VEL + k] = 0;
                    self.ram[SPRITE_X_VEL + k] = 0;
                    return;
                } else {
                    self.ram[SPRITE_DELAY_MAIN + k] = 32;
                }
                let old_head = self.ram[SPRITE_HEAD_DIR + k];
                let rng_before = self.ram[RNG_SEED];
                let rng = self.get_random_number();
                let rng_bit = rng & 1;
                self.ram[SPRITE_HEAD_DIR + k] =
                    K_ZAZAK_DIR2_MOTHULA[usize::from(self.ram[SPRITE_D + k] * 2 + rng_bit)];
                if trace_stalfos_head {
                    eprintln!(
                        "R stalfos head random fc={} x=0x{:04x} y=0x{:04x} old=0x{:02x} new=0x{:02x} d=0x{:02x} c_before=0x{:02x} rng_before=0x{:02x} rng=0x{:02x} bit=0x{:02x} delay=0x{:02x} wall=0x{:02x}",
                        self.ram[FRAME_COUNTER],
                        self.sprite_get_x(k),
                        self.sprite_get_y(k),
                        old_head,
                        self.ram[SPRITE_HEAD_DIR + k],
                        self.ram[SPRITE_D + k],
                        self.ram[SPRITE_C + k],
                        rng_before,
                        rng,
                        rng_bit,
                        self.ram[SPRITE_DELAY_MAIN + k],
                        self.ram[SPRITE_WALLCOLL + k],
                    );
                }
                self.ram[SPRITE_AI_STATE + k] = 0;
                self.ram[SPRITE_C + k] = self.ram[SPRITE_C + k].wrapping_add(1);
                if self.ram[SPRITE_C + k] == 4 {
                    self.ram[SPRITE_C + k] = 0;
                    let old_head = self.ram[SPRITE_HEAD_DIR + k];
                    let face = self.sprite_direction_to_face_link(k, None);
                    self.ram[SPRITE_HEAD_DIR + k] = face;
                    if trace_stalfos_head {
                        eprintln!(
                            "R stalfos head face4 fc={} x=0x{:04x} y=0x{:04x} old=0x{:02x} face=0x{:02x} d=0x{:02x} c=0x{:02x} delay=0x{:02x} rng=0x{:02x}",
                            self.ram[FRAME_COUNTER],
                            self.sprite_get_x(k),
                            self.sprite_get_y(k),
                            old_head,
                            face,
                            self.ram[SPRITE_D + k],
                            self.ram[SPRITE_C + k],
                            self.ram[SPRITE_DELAY_MAIN + k],
                            self.ram[RNG_SEED],
                        );
                    }
                    self.ram[SPRITE_DELAY_MAIN + k] = 24;
                }
                self.ram[SPRITE_Y_VEL + k] = 0;
                self.ram[SPRITE_X_VEL + k] = 0;
            }
            2 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 0;
                } else if self.ram[SPRITE_DELAY_MAIN + k] == 24 {
                    self.sprite_spawn_fire_phlegm(k);
                }
            }
            _ => {}
        }
    }

    pub(super) fn sprite_83_green_eyegore(&mut self, k: usize) {
        if self.ram[SPRITE_B + k] == 0 {
            self.eyegore_main(k);
            return;
        }
        self.goriya_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        if self.ram[SPRITE_DELAY_AUX1 + k] == 8 {
            self.sprite_spawn_fire_phlegm(k);
        }
        if self.ram[PLAYER_DEFENSE_FLAGS] != 0 || (self.ram[JOYPAD1H_LAST] & 0x0f) == 0 {
            self.ram[SPRITE_A + k] = 0;
            self.sprite_check_damage_to_and_from_link(k);
            self.sprite_check_tile_collision(k);
            return;
        }

        let j = usize::from(
            (self.ram[JOYPAD1H_LAST] & 0x0f)
                | if self.ram[SPRITE_TYPE + k] == 0x84 {
                    16
                } else {
                    0
                },
        );
        self.ram[SPRITE_D + k] = K_GORIYA_DIR[j];
        self.ram[SPRITE_X_VEL + k] = K_GORIYA_XVEL[j] as u8;
        self.ram[SPRITE_Y_VEL + k] = K_GORIYA_YVEL[j] as u8;
        if self.ram[SPRITE_WALLCOLL + k] == 0 {
            self.sprite_move_xy(k);
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_check_tile_collision(k);
        let gfx_idx = usize::from(
            self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1) & 12 | self.ram[SPRITE_D + k],
        );
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        self.ram[SPRITE_GRAPHICS + k] = K_GORIYA_GFX[gfx_idx];

        if self.ram[SPRITE_TYPE + k] == 0x84 {
            let mut pt = PointU8 { x: 0, y: 0 };
            let dir = self.sprite_direction_to_face_link(k, Some(&mut pt));
            if (pt.x.wrapping_add(8) < 16 || pt.y.wrapping_add(8) < 16)
                && self.ram[SPRITE_D + k] == dir
            {
                if self.ram[SPRITE_A + k] & 0x1f == 0 {
                    self.ram[SPRITE_DELAY_AUX1 + k] = 16;
                }
                self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_add(1);
                return;
            }
        }
        self.ram[SPRITE_A + k] = 0;
    }

    pub(super) fn eyegore_main(&mut self, k: usize) {
        self.eyegore_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.ram[SPRITE_FLAGS3 + k] |= 64;
        self.ram[SPRITE_DEFL_BITS + k] |= 4;

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    let mut pt = PointU8 { x: 0, y: 0 };
                    self.sprite_direction_to_face_link(k, Some(&mut pt));
                    if pt.x.wrapping_add(48) < 96 && pt.y.wrapping_add(48) < 96 {
                        self.ram[SPRITE_AI_STATE + k] =
                            self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                        self.ram[SPRITE_DELAY_MAIN + k] = 63;
                    }
                }
            }
            1 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_D + k] = self.sprite_direction_to_face_link(k, None);
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[SPRITE_DELAY_MAIN + k] =
                        K_EYEGORE_OPENING_DELAY[usize::from(self.get_random_number() & 3)];
                } else {
                    self.ram[SPRITE_GRAPHICS + k] =
                        K_EYEGORE_OPENING_GFX[usize::from(self.ram[SPRITE_DELAY_MAIN + k] >> 3)];
                }
            }
            2 => {
                self.ram[SPRITE_FLAGS3 + k] &= !0x40;
                if self.ram[SPRITE_TYPE + k] != 0x84 {
                    self.ram[SPRITE_DEFL_BITS + k] &= !4;
                }
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_DELAY_MAIN + k] = 63;
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    self.ram[SPRITE_GRAPHICS + k] = 0;
                } else {
                    if ((k as u8) ^ self.ram[FRAME_COUNTER]) & 31 == 0 {
                        self.ram[SPRITE_D + k] = self.sprite_direction_to_face_link(k, None);
                    }
                    let j = usize::from(self.ram[SPRITE_D + k]);
                    self.ram[SPRITE_X_VEL + k] = K_FLUTE_BOY_ANIMAL_XVEL_MOTHULA[j] as u8;
                    self.ram[SPRITE_Y_VEL + k] = K_ZAZAK_YVEL_MOTHULA[j] as u8;
                    if self.ram[SPRITE_WALLCOLL + k] == 0 {
                        self.sprite_move_xy(k);
                    }
                    self.sprite_check_tile_collision(k);
                    let gfx_idx = usize::from(
                        self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1) & 12 | self.ram[SPRITE_D + k],
                    );
                    self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                    self.ram[SPRITE_GRAPHICS + k] = K_EYEGORE_CHASING_GFX[gfx_idx];
                }
            }
            3 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    self.ram[SPRITE_DELAY_MAIN + k] = 96;
                } else {
                    self.ram[SPRITE_GRAPHICS + k] =
                        K_EYEGORE_CLOSING_GFX[usize::from(self.ram[SPRITE_DELAY_MAIN + k] >> 3)];
                }
            }
            _ => {}
        }
    }

    // void Sprite_A8_GreenZirro(int k) {  // 9e8dd2
    pub(super) fn sprite_a8_green_zirro(&mut self, k: usize) {
        self.ram[SPRITE_OBJ_PRIO + k] = 0x30;
        if self.ram[SPRITE_A + k] != 0 {
            match self.ram[SPRITE_AI_STATE + k] {
                0 => {
                    self.sprite_draw_single_small(k);
                    if self.sprite_return_if_inactive(k) {
                        return;
                    }
                    self.sprite_move_xy(k);
                    self.sprite_move_z(k);
                    self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
                    if sign8(self.ram[SPRITE_Z + k]) {
                        self.ram[SPRITE_Z + k] = 0;
                        self.ram[SPRITE_AI_STATE + k] =
                            self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                        self.ram[SPRITE_DELAY_MAIN + k] = 19;
                        self.ram[SPRITE_FLAGS2 + k] = self.ram[SPRITE_FLAGS2 + k].wrapping_add(1);
                        self.sprite_sfx_queue_sfx2_with_pan(k, 0x0c);
                    }
                }
                1 => {
                    self.sprite_draw_zirro_bomb(k);
                    if self.sprite_return_if_inactive(k) {
                        return;
                    }
                    if (self.ram[FRAME_COUNTER] & 3) == 0 {
                        self.ram[SPRITE_DELAY_MAIN + k] =
                            self.ram[SPRITE_DELAY_MAIN + k].wrapping_add(1);
                    }
                    self.sprite_check_damage_to_link(k);
                }
                _ => {}
            }
            return;
        }

        if self.ram[SPRITE_DELAY_AUX1 + k] != 0 {
            self.ram[SPRITE_GRAPHICS + k] = K_BOMBER_GFX[usize::from(self.ram[SPRITE_D + k])];
        }
        self.ram[SPRITE_OBJ_PRIO + k] |= 0x30;
        self.bomber_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        if self.ram[SPRITE_DELAY_AUX1 + k] == 8 {
            self.zirro_drop_bomb(k);
        }
        self.sprite_check_damage_to_and_from_link(k);
        if (self.ram[FRAME_COUNTER] & 1) == 0 {
            let j = self.ram[SPRITE_G + k] & 1;
            self.ram[SPRITE_Z_VEL + k] =
                self.ram[SPRITE_Z_VEL + k].wrapping_add(if j != 0 { 0xff } else { 1 });
            if self.ram[SPRITE_Z_VEL + k] == if j != 0 { (-8i8) as u8 } else { 8 } {
                self.ram[SPRITE_G + k] = self.ram[SPRITE_G + k].wrapping_add(1);
            }
        }
        self.sprite_move_z(k);
        let mut pt = PointU8 { x: 0, y: 0 };
        self.sprite_direction_to_face_link(k, Some(&mut pt));
        if pt.x.wrapping_add(40) < 80
            && pt.y.wrapping_add(40) < 80
            && self.ram[PLAYER_OAM_Y_OFFSET] != 0x80
            && (self.ram[LINK_IS_RUNNING] != 0 || sign8(self.ram[BUTTON_B_FRAMES].wrapping_sub(9)))
        {
            let pp = self.sprite_project_speed_towards_link(k, 0x30);
            self.ram[SPRITE_X_VEL + k] = pp.x.wrapping_neg();
            self.ram[SPRITE_Y_VEL + k] = pp.y.wrapping_neg();
            self.ram[SPRITE_DELAY_MAIN + k] = 8;
            self.ram[SPRITE_AI_STATE + k] = 2;
        }

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    self.ram[SPRITE_B + k] = self.ram[SPRITE_B + k].wrapping_add(1);
                    let j = if self.ram[SPRITE_B + k] == 3 {
                        self.ram[SPRITE_B + k] = 0;
                        self.ram[SPRITE_DELAY_MAIN + k] = 48;
                        K_BOMBER_TAB0[usize::from(self.sprite_direction_to_face_link(k, None))]
                    } else {
                        let r = self.get_random_number();
                        self.ram[SPRITE_DELAY_MAIN + k] = (r & 0x1f) | 0x20;
                        r & 7
                    };
                    self.ram[SPRITE_X_VEL + k] = K_BOMBER_XVEL[usize::from(j)] as u8;
                    self.ram[SPRITE_Y_VEL + k] = K_BOMBER_YVEL[usize::from(j)] as u8;
                }
                self.green_zirro_set_dir_for_mothula(k);
            }
            1 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    self.ram[SPRITE_DELAY_MAIN + k] = 10;
                    if self.ram[SPRITE_TYPE + k] == 0xa8 {
                        self.ram[SPRITE_DELAY_AUX1 + k] = 16;
                    }
                } else {
                    self.sprite_move_xy(k);
                    self.green_zirro_set_dir_for_mothula(k);
                }
            }
            2 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 0;
                }
                self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(2);
                self.sprite_move_xy(k);
                self.green_zirro_set_dir_for_mothula(k);
            }
            _ => {}
        }
    }

    fn green_zirro_set_dir_for_mothula(&mut self, k: usize) {
        self.ram[SPRITE_D + k] = self.sprite_direction_to_face_link(k, None);
        self.ram[SPRITE_GRAPHICS + k] = (self.ram[SPRITE_D + k] << 1)
            | ((self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1) >> 3) & 1);
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
    }

    // void Zirro_DropBomb(int k) {  // 9e8f81
    pub(super) fn zirro_drop_bomb(&mut self, k: usize) {
        let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xa8, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
            self.ram[SPRITE_Z + j] = info.r4_z;
            let i = usize::from(self.ram[SPRITE_D + j]);
            self.sprite_set_x(
                j,
                info.r0_x
                    .wrapping_add(K_BOMBER_SPAWN_PELLET_X[i] as i16 as u16),
            );
            self.sprite_set_y(
                j,
                info.r2_y
                    .wrapping_add(K_BOMBER_SPAWN_PELLET_Y[i] as i16 as u16),
            );
            self.ram[SPRITE_X_VEL + j] = K_FLUTE_BOY_ANIMAL_XVEL_MOTHULA[i] as u8;
            self.ram[SPRITE_Y_VEL + j] = K_ZAZAK_YVEL_MOTHULA[i] as u8;
            self.ram[SPRITE_A + j] = 1;
            self.ram[SPRITE_IGNORE_PROJECTILE + j] = 1;
            self.ram[SPRITE_FLAGS4 + j] = 9;
            self.ram[SPRITE_FLAGS3 + j] = 0x33;
            self.ram[SPRITE_OAM_FLAGS + j] = 0x33 & 15;
        }
    }

    // void Sprite_C5_Medusa(int k) {  // 9dc7eb
    pub(super) fn sprite_c5_medusa(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_prep_oam_coord(k, &mut info);
        if self.ram[PLAYER_IS_INDOORS] == 0 {
            self.ram[SPRITE_X_VEL + k] = 255;
            self.ram[SPRITE_SUBTYPE + k] = 255;
            if self.sprite_check_tile_collision(k) == 0 {
                return;
            }
            if self.sprite_return_if_inactive(k) {
                return;
            }
            self.ram[SPRITE_TYPE + k] = 0x19;
            self.sprite_prep_load_properties(k);
            self.ram[SPRITE_E + k] = self.ram[SPRITE_E + k].wrapping_add(1);
            self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_X_LO + k].wrapping_add(8);
            self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_Y_LO + k].wrapping_sub(8);
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x19);
            self.ram[SPRITE_DEFL_BITS + k] = 0x80;
        } else {
            if self.sprite_return_if_inactive(k) {
                return;
            }
            self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
            if (self.ram[SPRITE_SUBTYPE2 + k] & 0x7f) == 0
                && self.ram[SPRITE_FLOOR + k] == self.ram[LINK_IS_ON_LOWER_LEVEL]
            {
                let j = self.sprite_spawn_fireball(k);
                if j >= 0 {
                    let j = j as usize;
                    self.ram[SPRITE_DEFL_BITS + j] |= 8;
                    self.ram[SPRITE_BUMP_DAMAGE + j] = 4;
                }
            }
        }
    }

    // void Sprite_C6_4WayShooter(int k) {  // 9dc869
    pub(super) fn sprite_c6_4_way_shooter(&mut self, k: usize) {
        let mut info = SpritePrepOamCoordsRet {
            x: 0,
            y: 0,
            r4: 0,
            flags: 0,
        };
        self.sprite_prep_oam_coord(k, &mut info);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.ram[SPRITE_DELAY_MAIN + k] == 24 {
            let j = self.sprite_spawn_fireball(k);
            if j >= 0 {
                let j = j as usize;
                self.ram[SPRITE_DEFL_BITS + j] |= 8;
                self.ram[SPRITE_BUMP_DAMAGE + j] = 4;
                let i = usize::from(self.sprite_direction_to_face_link(j, None));
                self.ram[SPRITE_X_VEL + j] = K_FIREBALL_JUNCTION_XYVEL[i + 2] as u8;
                self.ram[SPRITE_Y_VEL + j] = K_FIREBALL_JUNCTION_XYVEL[i] as u8;
                self.sprite_set_x(
                    j,
                    self.sprite_get_x(j)
                        .wrapping_add(K_FIREBALL_JUNCTION_X[i] as i16 as u16),
                );
                self.sprite_set_y(
                    j,
                    self.sprite_get_y(j)
                        .wrapping_add(K_FIREBALL_JUNCTION_Y[i] as i16 as u16),
                );
            }
        } else if self.ram[SPRITE_DELAY_MAIN + k] == 0
            && self.ram[BUTTON_B_FRAMES] != 0
            && self.ram[SPRITE_FLOOR + k] == self.ram[LINK_IS_ON_LOWER_LEVEL]
        {
            self.ram[SPRITE_DELAY_MAIN + k] = 32;
        }
    }

    // void Sprite_C3_Gibo(int k) {  // 9dcce1
    pub(super) fn sprite_c3_gibo(&mut self, k: usize) {
        if self.ram[SPRITE_B + k] != 0 {
            self.sprite_draw_single_large(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            self.sprite_check_damage_to_and_from_link(k);
            self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
            self.ram[SPRITE_OAM_FLAGS + k] = (self.ram[SPRITE_OAM_FLAGS + k] & 0x3f)
                | K_GIBO_OAM_FLAGS_MOTHULA[usize::from((self.ram[SPRITE_SUBTYPE2 + k] >> 2) & 3)];
            if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
                self.sprite_move_xy(k);
                self.sprite_bounce_from_tile_collision(k);
            }
            return;
        }

        self.gibo_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.ram[SPRITE_ANIM_CLOCK + k] = self.ram[SPRITE_ANIM_CLOCK + k].wrapping_add(1);
        let mut j = usize::from(self.ram[SPRITE_HEAD_DIR + k]);
        if self.ram[SPRITE_STATE + j] == 6 {
            self.ram[SPRITE_STATE + k] = self.ram[SPRITE_STATE + j];
            self.ram[SPRITE_DELAY_MAIN + k] = self.ram[SPRITE_DELAY_MAIN + j];
            self.ram[SPRITE_FLAGS2 + k] = self.ram[SPRITE_FLAGS2 + k].wrapping_add(4);
            return;
        }
        self.ram[SPRITE_SUBTYPE2 + k] = (self.ram[FRAME_COUNTER] >> 3) & 3;
        if (self.ram[FRAME_COUNTER] & 63) == 0 {
            self.ram[SPRITE_D + k] = self.sprite_is_right_of_link(k).a << 2;
        }
        self.sprite_check_damage_to_link(k);

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    self.ram[SPRITE_DELAY_MAIN + k] = 48;
                    self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_add(1);
                    let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
                    let spawned = self.sprite_spawn_dynamically(k, 0xc3, &mut info);
                    if spawned >= 0 {
                        j = spawned as usize;
                        self.sprite_set_spawned_coordinates(j, &info);
                        self.ram[SPRITE_HEAD_DIR + k] = j as u8;
                        self.ram[SPRITE_FLAGS2 + j] = 1;
                        self.ram[SPRITE_B + j] = 1;
                        self.ram[SPRITE_FLAGS3 + j] = 16;
                        self.ram[SPRITE_HEALTH + j] = self.ram[SPRITE_G + k];
                        self.ram[SPRITE_OAM_FLAGS + j] = 7;
                        self.ram[SPRITE_DELAY_MAIN + j] = 48;
                        self.ram[SPRITE_C + k] = self.ram[SPRITE_C + k].wrapping_add(1);
                        let i = if self.ram[SPRITE_C + k] == 3 {
                            self.ram[SPRITE_C + k] = 0;
                            self.sprite_direction_to_face_link(k, None)
                        } else {
                            self.get_random_number() & 7
                        };
                        let i = usize::from(i);
                        self.ram[SPRITE_X_VEL + j] = K_GIBO_XVEL[i] as u8;
                        self.ram[SPRITE_Y_VEL + j] = K_GIBO_YVEL[i] as u8;
                    }
                } else if self.ram[SPRITE_DELAY_MAIN + k] == 32 {
                    self.ram[SPRITE_DELAY_AUX1 + k] = 32;
                }
            }
            1 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                }
            }
            2 => {
                if (((k as u8) ^ self.ram[FRAME_COUNTER]) & 3) == 0 {
                    let x = self.sprite_get_x(j);
                    let y = self.sprite_get_y(j);
                    if read_le_u16(&self.ram, CUR_SPRITE_X)
                        .wrapping_sub(x)
                        .wrapping_add(2)
                        < 4
                        && read_le_u16(&self.ram, CUR_SPRITE_Y)
                            .wrapping_sub(y)
                            .wrapping_add(2)
                            < 4
                    {
                        j = usize::from(self.ram[SPRITE_HEAD_DIR + k]);
                        self.ram[SPRITE_STATE + j] = 0;
                        self.ram[SPRITE_A + k] = 0;
                        self.ram[SPRITE_AI_STATE + k] = 0;
                        self.ram[SPRITE_G + k] = self.ram[SPRITE_HEALTH + j];
                        self.ram[SPRITE_DELAY_MAIN + k] = (self.get_random_number() & 31) + 32;
                        return;
                    }
                    let pt = self.sprite_project_speed_towards_location(k, x, y, 16);
                    self.ram[SPRITE_X_VEL + k] = pt.x;
                    self.ram[SPRITE_Y_VEL + k] = pt.y;
                }
                self.sprite_move_xy(k);
            }
            _ => {}
        }
    }

    // void Sprite_Tektite(int k) {  // 9dc293
    pub(super) fn sprite_tektite(&mut self, k: usize) {
        if self.ram[SPRITE_DELAY_AUX1 + k] != 0 {
            self.ram[SPRITE_GRAPHICS + k] = 0;
        }
        self.tektite_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xyz(k);
        self.sprite_bounce_from_tile_collision(k);
        self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(1);
        if sign8(self.ram[SPRITE_Z + k]) {
            self.ram[SPRITE_Z + k] = 0;
            self.ram[SPRITE_Z_VEL + k] = 0;
        }

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                let mut pt = PointU8 { x: 0, y: 0 };
                let mut j = self.sprite_direction_to_face_link(k, Some(&mut pt));
                if pt.x.wrapping_add(40) < 80
                    && pt.y.wrapping_add(40) < 80
                    && self.ram[PLAYER_OAM_Y_OFFSET] != 0x80
                    && (self.ram[SPRITE_Z + k] | self.ram[SPRITE_PAUSE + k]) == 0
                    && self.ram[LINK_IS_ON_LOWER_LEVEL] == self.ram[SPRITE_FLOOR + k]
                    && j != K_TEKITE_DIR[usize::from(self.ram[LINK_DIRECTION_FACING] >> 1)]
                {
                    let pt = self.sprite_project_speed_towards_link(k, 32);
                    self.ram[SPRITE_X_VEL + k] = pt.x.wrapping_neg();
                    self.ram[SPRITE_Y_VEL + k] = pt.y.wrapping_neg();
                    self.ram[SPRITE_Z_VEL + k] = 16;
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    return;
                }
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    self.ram[SPRITE_B + k] = self.ram[SPRITE_B + k].wrapping_add(1);
                    if self.ram[SPRITE_B + k] == 4 {
                        self.ram[SPRITE_B + k] = 0;
                        self.ram[SPRITE_AI_STATE + k] =
                            self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                        self.ram[SPRITE_DELAY_MAIN + k] =
                            (self.get_random_number() & 63).wrapping_add(48);
                        self.ram[SPRITE_Z_VEL + k] = 12;
                        j = self.sprite_is_below_link(k).a * 2 + self.sprite_is_right_of_link(k).a;
                    } else {
                        self.ram[SPRITE_Z_VEL + k] =
                            (self.get_random_number() & 7).wrapping_add(24);
                        j = self.get_random_number() & 3;
                    }
                    self.ram[SPRITE_X_VEL + k] = K_TEKITE_XVEL[usize::from(j)] as u8;
                    self.ram[SPRITE_Y_VEL + k] = K_TEKITE_YVEL[usize::from(j)] as u8;
                } else {
                    self.ram[SPRITE_GRAPHICS + k] = (self.ram[SPRITE_DELAY_MAIN + k] >> 4) & 1;
                }
            }
            1 => {
                if self.ram[SPRITE_Z + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    self.ram[SPRITE_DELAY_MAIN + k] =
                        (self.get_random_number() & 63).wrapping_add(72);
                    self.ram[SPRITE_Y_VEL + k] = 0;
                    self.ram[SPRITE_X_VEL + k] = 0;
                } else {
                    self.ram[SPRITE_GRAPHICS + k] = 2;
                }
            }
            2 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    self.ram[SPRITE_DELAY_MAIN + k] =
                        (self.get_random_number() & 63).wrapping_add(72);
                    self.ram[SPRITE_Y_VEL + k] = 0;
                    self.ram[SPRITE_X_VEL + k] = 0;
                    return;
                }
                if self.ram[SPRITE_Z + k] == 0 {
                    self.ram[SPRITE_Z_VEL + k] = 12;
                    self.ram[SPRITE_Z + k] = self.ram[SPRITE_Z + k].wrapping_add(1);
                    self.ram[SPRITE_DELAY_AUX1 + k] = 8;
                }
                self.ram[SPRITE_GRAPHICS + k] = 2;
            }
            _ => {}
        }
    }

    // void Sprite_C9_Tektite(int k) {  // 9dc275
    pub(super) fn sprite_c9_tektite(&mut self, k: usize) {
        let j = self.ram[SPRITE_ANIM_CLOCK + k];
        if j != 0 {
            self.ram[SPRITE_IGNORE_PROJECTILE + k] = j;
            self.ram[SPRITE_OBJ_PRIO + k] = 0x30;
        }
        match j {
            0 => self.sprite_tektite(k),
            1 => self.sprite_phantom_ganon(k),
            2 => self.sprite_ganon_trident(k),
            3 => self.sprite_spiral_fire_bat(k),
            4 => self.sprite_fire_bat_launched(k),
            5 => self.sprite_fire_bat_trailer(k),
            _ => {}
        }
    }

    // void Sprite_D2_FloppingFish(int k) {  // 9d8235
    pub(super) fn sprite_d2_flopping_fish(&mut self, k: usize) {
        const XVEL: [i8; 8] = [0, 12, 16, 12, 0, -12, -16, -12];
        const YVEL: [i8; 8] = [-16, -12, 0, 12, 16, 12, 0, -12];
        const TAB1: [u8; 2] = [2, 0];
        const GFX: [u8; 3] = [1, 5, 3];
        const GFX2: [u8; 17] = [5, 5, 6, 6, 5, 5, 4, 4, 3, 7, 7, 8, 8, 7, 7, 8, 8];

        if self.ram[SPRITE_CHR_HALFSLOT_STATE] < 3 {
            self.fish_draw(k);
        }
        if self.ram[SPRITE_STATE + k] == 10 {
            self.ram[SPRITE_AI_STATE + k] = 4;
            self.ram[SPRITE_GRAPHICS + k] = ((self.ram[FRAME_COUNTER] >> 4) & 1) + 3;
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.sprite_check_tile_collision(k);
                if self.ram[SPRITE_TILETYPE_MOTHULA] == 8 {
                    self.ram[SPRITE_STATE + k] = 0;
                } else {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                }
            }
            1 => {
                self.sprite_check_if_lifted_permissive(k);
                self.sprite_bounce_from_tile_collision(k);
                self.sprite_move_xyz(k);
                self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
                if sign8(self.ram[SPRITE_Z + k]) {
                    self.ram[SPRITE_Z + k] = 0;
                    if self.ram[SPRITE_TILETYPE_MOTHULA] == 9 {
                        self.sprite_spawn_small_splash(k);
                    } else if self.ram[SPRITE_TILETYPE_MOTHULA] == 8 {
                        self.ram[SPRITE_STATE + k] = 0;
                        self.sprite_spawn_small_splash(k);
                    }
                    self.ram[SPRITE_Z_VEL + k] = (self.get_random_number() & 15).wrapping_add(16);
                    let j = usize::from(self.get_random_number() & 7);
                    self.ram[SPRITE_X_VEL + k] = XVEL[j] as u8;
                    self.ram[SPRITE_Y_VEL + k] = YVEL[j] as u8;
                    self.ram[SPRITE_D + k] = self.ram[SPRITE_D + k].wrapping_add(1);
                    self.ram[SPRITE_SUBTYPE2 + k] = 3;
                }
                self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                if (self.ram[SPRITE_SUBTYPE2 + k] & 7) == 0 {
                    let j = usize::from(self.ram[SPRITE_D + k] & 1);
                    if self.ram[SPRITE_A + k] != TAB1[j] {
                        self.ram[SPRITE_A + k] =
                            self.ram[SPRITE_A + k].wrapping_add(if j != 0 { 0xff } else { 1 });
                    }
                }
                let a = usize::from(self.ram[SPRITE_A + k]);
                self.ram[SPRITE_GRAPHICS + k] = GFX[a] + ((self.ram[FRAME_COUNTER] >> 3) & 1);
            }
            2 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 3;
                    self.ram[SPRITE_Z_VEL + k] = 48;
                    self.sprite_spawn_small_splash(k);
                }
            }
            3 => {
                self.sprite_move_z(k);
                self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
                if self.ram[SPRITE_Z_VEL + k] == 0 && self.ram[SPRITE_A + k] != 0 {
                    write_le_u16(&mut self.ram, DIALOGUE_MESSAGE_INDEX, 0x176);
                    self.sprite_show_message_minimal_c();
                }
                if sign8(self.ram[SPRITE_Z + k]) {
                    self.ram[SPRITE_Z + k] = 0;
                    self.sprite_spawn_small_splash(k);
                    if self.ram[SPRITE_A + k] != 0 {
                        let mut info = SpriteSpawnInfo::default();
                        let j = self.sprite_spawn_dynamically(k, 0xdb, &mut info);
                        if j >= 0 {
                            let j = j as usize;
                            self.sprite_set_spawned_coordinates(j, &info);
                            self.sprite_set_x(j, info.r0_x.wrapping_add(4));
                            self.ram[SPRITE_STUNNED + j] = 255;
                            self.ram[SPRITE_Z_VEL + j] = 48;
                            self.ram[SPRITE_DELAY_AUX3_MOTHULA + j] = 48;
                            self.sprite_apply_speed_towards_link(j, 16);
                        }
                    }
                    self.ram[SPRITE_STATE + k] = 0;
                }
                self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                let idx = usize::from(self.ram[SPRITE_SUBTYPE2 + k] >> 2);
                self.ram[SPRITE_GRAPHICS + k] = GFX2[idx];
            }
            4 => {
                if self.ram[SPRITE_Z + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                }
                self.sprite_move_xy(k);
                self.thrown_sprite_tile_and_sprite_interaction(k);
            }
            _ => {}
        }
    }

    // void Sprite_81_Hover(int k) {  // 9ecc02
    pub(super) fn sprite_81_hover(&mut self, k: usize) {
        self.ram[SPRITE_OBJ_PRIO + k] |= 48;
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.ram[SPRITE_F + k] != 0 {
            self.ram[SPRITE_AI_STATE + k] = 0;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        if self.ram[SPRITE_WALLCOLL + k] == 0 {
            self.sprite_move_xy(k);
        }
        self.sprite_check_tile_collision(k);
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        self.ram[SPRITE_GRAPHICS + k] = (self.ram[SPRITE_SUBTYPE2 + k] >> 3) & 2;

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    let j = self.sprite_is_right_of_link(k).a + self.sprite_is_below_link(k).a * 2;
                    self.ram[SPRITE_D + k] = j;
                    self.ram[SPRITE_OAM_FLAGS + k] = (self.ram[SPRITE_OAM_FLAGS + k] & !0x40)
                        | K_HOVER_OAM_FLAGS[usize::from(j)];
                    self.ram[SPRITE_DELAY_MAIN + k] =
                        (self.get_random_number() & 15).wrapping_add(12);
                    self.sprite_zero_velocity_xy(k);
                }
            }
            1 => {
                let j = usize::from(self.ram[SPRITE_D + k]);
                if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
                    self.ram[SPRITE_X_VEL + k] =
                        self.ram[SPRITE_X_VEL + k].wrapping_add(K_HOVER_ACCEL_X0[j] as u8);
                    self.ram[SPRITE_Y_VEL + k] =
                        self.ram[SPRITE_Y_VEL + k].wrapping_add(K_HOVER_ACCEL_Y0[j] as u8);
                    self.ram[SPRITE_GRAPHICS + k] = (self.ram[SPRITE_SUBTYPE2 + k] >> 3) & 1;
                } else {
                    self.ram[SPRITE_X_VEL + k] =
                        self.ram[SPRITE_X_VEL + k].wrapping_add(K_HOVER_ACCEL_X1[j] as u8);
                    self.ram[SPRITE_Y_VEL + k] =
                        self.ram[SPRITE_Y_VEL + k].wrapping_add(K_HOVER_ACCEL_Y1[j] as u8);
                    if self.ram[SPRITE_Y_VEL + k] == 0 {
                        self.ram[SPRITE_AI_STATE + k] = 0;
                        self.ram[SPRITE_DELAY_MAIN + k] = 64;
                    }
                }
            }
            _ => {}
        }
    }

    // void Sprite_CA_ChainChomp(int k) {  // 9dbe7d
    pub(super) fn sprite_ca_chain_chomp(&mut self, k: usize) {
        self.chain_chomp_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.chain_chomp_handle_leash(k);
        if (((k as u8) ^ self.ram[FRAME_COUNTER]) & 3) == 0
            && (self.ram[SPRITE_X_VEL + k] | self.ram[SPRITE_Y_VEL + k]) != 0
        {
            self.ram[SPRITE_D + k] = ZeldaState::sprite_convert_velocity_to_angle(
                self.ram[SPRITE_X_VEL + k],
                self.ram[SPRITE_Y_VEL + k],
            ) & 0x0f;
        }
        self.sprite_move_xyz(k);
        self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
        if sign8(self.ram[SPRITE_Z + k]) {
            self.ram[SPRITE_Z + k] = 0;
            self.ram[SPRITE_Z_VEL + k] = 0;
        }
        let cur_x = self.sprite_get_x(k);
        let cur_y = self.sprite_get_y(k);
        write_le_u16(&mut self.ram, CUR_SPRITE_X, cur_x);
        write_le_u16(&mut self.ram, CUR_SPRITE_Y, cur_y);
        let x = u16::from(self.ram[SPRITE_A + k]) | (u16::from(self.ram[SPRITE_B + k]) << 8);
        let y = u16::from(self.ram[SPRITE_C + k]) | (u16::from(self.ram[SPRITE_G + k]) << 8);
        self.ram[SPRITE_ANIM_CLOCK + k] = u8::from(
            cur_x.wrapping_sub(x).wrapping_add(48) < 96
                && cur_y.wrapping_sub(y).wrapping_add(48) < 96,
        );

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                    if self.ram[SPRITE_SUBTYPE2 + k] == 4 {
                        self.ram[SPRITE_SUBTYPE2 + k] = 0;
                        self.ram[SPRITE_AI_STATE + k] = 2;
                        let j = usize::from(self.get_random_number() & 15);
                        self.ram[SPRITE_X_VEL + k] = (K_CHAIN_CHOMP_XVEL[j] << 2) as u8;
                        self.ram[SPRITE_Y_VEL + k] = (K_CHAIN_CHOMP_YVEL[j] << 2) as u8;
                        self.get_random_number();
                        self.sprite_apply_speed_towards_link(k, 64);
                        self.sprite_sfx_queue_sfx3_with_pan(k, 0x4);
                    } else {
                        self.ram[SPRITE_DELAY_MAIN + k] =
                            (self.get_random_number() & 31).wrapping_add(16);
                        let j = usize::from(self.get_random_number() & 15);
                        self.ram[SPRITE_X_VEL + k] = K_CHAIN_CHOMP_XVEL[j] as u8;
                        self.ram[SPRITE_Y_VEL + k] = K_CHAIN_CHOMP_YVEL[j] as u8;
                        self.ram[SPRITE_AI_STATE + k] = 1;
                    }
                } else {
                    self.ram[SPRITE_X_VEL + k] = 0;
                    self.ram[SPRITE_Y_VEL + k] = 0;
                }
            }
            1 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_DELAY_MAIN + k] = 32;
                    self.ram[SPRITE_AI_STATE + k] = 0;
                }
                if (self.ram[SPRITE_DELAY_MAIN + k] & 15) == 0 {
                    self.chain_chomp_move_chain(k);
                }
                if self.ram[SPRITE_Z + k] == 0 {
                    self.ram[SPRITE_Z_VEL + k] = 16;
                }
                if self.ram[SPRITE_ANIM_CLOCK + k] == 0 {
                    let x = u16::from(self.ram[SPRITE_A + k])
                        | (u16::from(self.ram[SPRITE_B + k]) << 8);
                    let y = u16::from(self.ram[SPRITE_C + k])
                        | (u16::from(self.ram[SPRITE_G + k]) << 8);
                    let pt = self.sprite_project_speed_towards_location(k, x, y, 16);
                    self.ram[SPRITE_X_VEL + k] = pt.x;
                    self.ram[SPRITE_Y_VEL + k] = pt.y;
                    self.sprite_move_xy(k);
                    self.ram[SPRITE_DELAY_MAIN + k] = 12;
                }
            }
            2 => {
                if self.ram[SPRITE_ANIM_CLOCK + k] == 0 {
                    self.ram[SPRITE_X_VEL + k] = self.ram[SPRITE_X_VEL + k].wrapping_neg();
                    self.ram[SPRITE_Y_VEL + k] = self.ram[SPRITE_Y_VEL + k].wrapping_neg();
                    self.sprite_move_xy(k);
                    self.ram[SPRITE_X_VEL + k] = 0;
                    self.ram[SPRITE_Y_VEL + k] = 0;
                    self.ram[SPRITE_AI_STATE + k] = 3;
                    self.ram[SPRITE_DELAY_AUX1 + k] = 48;
                }
                self.chain_chomp_move_chain(k);
                self.chain_chomp_move_chain(k);
            }
            3 => {
                if self.ram[SPRITE_DELAY_AUX1 + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    self.ram[SPRITE_DELAY_MAIN + k] = 48;
                }
                self.chain_chomp_move_chain(k);
                self.chain_chomp_move_chain(k);
            }
            _ => {}
        }
    }

    // void Sprite_C7_Pokey(int k) {  // 9dc64f
    pub(super) fn sprite_c7_pokey(&mut self, k: usize) {
        if self.ram[SPRITE_C + k] != 0 {
            self.sprite_draw_single_large(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            self.sprite_check_damage_to_and_from_link(k);
            self.sprite_move_xyz(k);
            self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
            if sign8(self.ram[SPRITE_Z + k]) {
                self.ram[SPRITE_Z_VEL + k] = 16;
                self.ram[SPRITE_Z + k] = 0;
            }
            if self.sprite_bounce_from_tile_collision(k) != 0 {
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x21);
            }
            if self.ram[SPRITE_G + k] >= 3 {
                self.ram[SPRITE_STATE + k] = 6;
                self.ram[SPRITE_DELAY_MAIN + k] = 10;
                self.ram[SPRITE_FLAGS5 + k] = 0;
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x1e);
            }
            return;
        }

        self.hokbok_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.ram[SPRITE_A + k] != 0 && self.ram[SPRITE_F + k] == 15 {
            self.ram[SPRITE_F + k] = 6;
            self.ram[SPRITE_Z + k] = self.ram[SPRITE_Z + k].wrapping_add(self.ram[SPRITE_B + k]);
            self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_sub(1);
            if self.ram[SPRITE_A + k] == 0 {
                self.ram[SPRITE_HEALTH + k] = 17;
            }
            self.ram[SPRITE_X_VEL + k] = if sign8(self.ram[SPRITE_X_VEL + k]) {
                self.ram[SPRITE_X_VEL + k].wrapping_sub(4)
            } else {
                self.ram[SPRITE_X_VEL + k].wrapping_add(4)
            };
            self.ram[SPRITE_Y_VEL + k] = if sign8(self.ram[SPRITE_Y_VEL + k]) {
                self.ram[SPRITE_Y_VEL + k].wrapping_sub(4)
            } else {
                self.ram[SPRITE_Y_VEL + k].wrapping_add(4)
            };

            let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0xc7, &mut info);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                self.ram[SPRITE_C + j] = 1;
                self.ram[SPRITE_HEALTH + j] = 1;
                self.ram[SPRITE_X_VEL + j] = self.ram[SPRITE_X_RECOIL + k];
                self.ram[SPRITE_Y_VEL + j] = self.ram[SPRITE_Y_RECOIL_MOTHULA + k];
                self.ram[SPRITE_DEFL_BITS + j] = 64;
            }
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.ram[SPRITE_Z_VEL + k] = 16;
                } else {
                    self.ram[SPRITE_B + k] =
                        K_HOKBOK_B[usize::from(self.ram[SPRITE_DELAY_MAIN + k] >> 1)];
                }
            }
            1 => {
                self.sprite_move_xyz(k);
                self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
                if sign8(self.ram[SPRITE_Z + k]) {
                    self.ram[SPRITE_Z + k] = 0;
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    self.ram[SPRITE_DELAY_MAIN + k] = 15;
                }
                self.sprite_bounce_from_tile_collision(k);
            }
            _ => {}
        }
    }

    // void Sprite_C2_Boulder(int k) {  // 9dcfcb
    pub(super) fn sprite_c2_boulder(&mut self, k: usize) {
        if self.ram[PLAYER_IS_INDOORS] == 0 {
            self.boulder_outdoors_main(k);
            return;
        }
        if self.ram[SPRITE_CHR_HALFSLOT_STATE] < 3 {
            self.sprite_draw_single_small(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.ram[SPRITE_OAM_FLAGS + k] = (self.ram[FRAME_COUNTER] << 2) & 0xc0;
        self.sprite_move_xyz(k);
        if (((k as u8) ^ self.ram[FRAME_COUNTER]) & 3) != 0 {
            return;
        }
        let cur_x = read_le_u16(&self.ram, CUR_SPRITE_X);
        let cur_y = read_le_u16(&self.ram, CUR_SPRITE_Y);
        let link_x = self.player_state_view().x();
        let link_y = self.player_state_view().y();
        if cur_x.wrapping_sub(link_x).wrapping_add(4) < 16
            && cur_y.wrapping_sub(link_y).wrapping_sub(4) < 12
        {
            self.sprite_attempt_damage_to_link_plus_recoil(k);
        }
        if self.sprite_check_tile_collision(k) != 0 {
            self.ram[SPRITE_STATE + k] = 0;
        }
    }

    // void Boulder_OutdoorsMain(int k) {  // 9dd02a
    pub(super) fn boulder_outdoors_main(&mut self, k: usize) {
        self.ram[SPRITE_OBJ_PRIO + k] = 0x30;
        self.boulder_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.ram[SPRITE_SUBTYPE2 + k] =
            self.ram[SPRITE_SUBTYPE2 + k].wrapping_sub(self.ram[SPRITE_D + k]);
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xyz(k);
        self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
        if sign8(self.ram[SPRITE_Z + k]) {
            self.ram[SPRITE_Z + k] = 0;
            let mut j = usize::from(self.sprite_check_tile_collision(k) != 0);
            self.ram[SPRITE_Z_VEL + k] = K_BOULDER_ZVEL[j] as u8;
            self.ram[SPRITE_Y_VEL + k] = K_BOULDER_YVEL[j] as u8;
            j += usize::from(self.get_random_number() & 1) * 2;
            self.ram[SPRITE_X_VEL + k] = K_BOULDER_XVEL[j] as u8;
            self.ram[SPRITE_D + k] = (((j & 2) as u8).wrapping_sub(1)) as u8;
            self.sprite_sfx_queue_sfx2_with_pan(k, 0xb);
        }
    }

    // void Sprite_9C_Zoro(int k) {  // 9e9bc8
    pub(super) fn sprite_9_c_zoro(&mut self, k: usize) {
        if self.ram[SPRITE_E + k] != 0 {
            self.zoro(k);
        } else {
            self.babasu(k);
        }
    }

    // void Babasu(int k) {  // 9e9c6b
    pub(super) fn babasu(&mut self, k: usize) {
        self.babusu_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                self.ram[SPRITE_DELAY_MAIN + k] = 128;
                self.ram[SPRITE_GRAPHICS + k] = 255;
            }
            1 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    self.ram[SPRITE_DELAY_MAIN + k] = 55;
                }
            }
            2 => {
                let j = self.ram[SPRITE_DELAY_MAIN + k];
                let i = usize::from(self.ram[SPRITE_D + k]);
                if j == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 3;
                    self.ram[SPRITE_X_VEL + k] = K_BABUSU_XY_VEL[i] as u8;
                    self.ram[SPRITE_Y_VEL + k] = K_BABUSU_XY_VEL[i + 2] as u8;
                    self.ram[SPRITE_DELAY_MAIN + k] = 32;
                }
                if j >= 32 {
                    self.ram[SPRITE_GRAPHICS + k] =
                        K_BABUSU_GFX[usize::from((j - 32) >> 2)] + K_BABUSU_DIR_GFX[i];
                } else {
                    self.ram[SPRITE_GRAPHICS + k] = 0xff;
                }
            }
            3 => {
                self.sprite_check_damage_to_and_from_link(k);
                self.sprite_move_xy(k);
                self.ram[SPRITE_GRAPHICS + k] = ((self.ram[FRAME_COUNTER] >> 1) & 1)
                    + K_BABUSU_SCURRY_GFX[usize::from(self.ram[SPRITE_D + k])];
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 && self.sprite_check_tile_collision(k) != 0
                {
                    self.ram[SPRITE_D + k] ^= 1;
                    self.ram[SPRITE_AI_STATE + k] = 0;
                }
            }
            _ => {}
        }
    }

    // void Sprite_DrawLargeWaterTurbulence(int k) {  // 84ebe5
    pub(super) fn sprite_draw_large_water_turbulence(&mut self, k: usize) {
        const D: [DrawMultipleData; 6] = [
            DrawMultipleData {
                x: -10,
                y: 14,
                char_flags: 0x00c0,
                ext: 2,
            },
            DrawMultipleData {
                x: -5,
                y: 16,
                char_flags: 0x40c0,
                ext: 2,
            },
            DrawMultipleData {
                x: -2,
                y: 18,
                char_flags: 0x00c0,
                ext: 2,
            },
            DrawMultipleData {
                x: 2,
                y: 18,
                char_flags: 0x40c0,
                ext: 2,
            },
            DrawMultipleData {
                x: 5,
                y: 16,
                char_flags: 0x00c0,
                ext: 2,
            },
            DrawMultipleData {
                x: 10,
                y: 14,
                char_flags: 0x40c0,
                ext: 2,
            },
        ];
        let bak = self.ram[SPRITE_OAM_FLAGS + k];
        self.ram[SPRITE_OAM_FLAGS + k] = if ((self.ram[SPRITE_SUBTYPE2 + k] >> 1) & 1) != 0 {
            0x44
        } else {
            4
        };
        self.ram[SPRITE_OBJ_PRIO + k] &= !0x0f;
        self.oam_allocate_from_region_c(self.ram[SPRITE_OBJ_PRIO + k]);
        self.sprite_draw_multiple(k, &D, None);
        self.ram[SPRITE_OAM_FLAGS + k] = bak;
    }

    // void Sprite_8C_Arrghus(int k) {  // 9eb433
    pub(super) fn sprite_8_c_arrghus(&mut self, k: usize) {
        self.ram[SPRITE_OBJ_PRIO + k] |= 0x30;
        self.arrghus_draw(k);
        if self.ram[SPRITE_STATE + k] != 9 || self.ram[SPRITE_Z + k] < 96 {
            if self.sprite_return_if_inactive(k) {
                return;
            }
        }

        self.arrghus_handle_puffs(k);
        self.ram[OVERLORD_X_LO_MOTHULA + 4] = 1;
        if (self.ram[SPRITE_HIT_TIMER + k] & 127) == 2 {
            self.ram[SPRITE_AI_STATE + k] = 3;
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x32);
            self.ram[SPRITE_SUBTYPE2 + k] = 0;
            self.ram[SPRITE_FLAGS3 + k] = 64;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }

        self.sprite_check_damage_to_link(k);
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        if ((self.ram[SPRITE_SUBTYPE2 + k].wrapping_sub(1)) & 3) == 0 {
            self.ram[SPRITE_G + k] = self.ram[SPRITE_G + k].wrapping_add(1);
            if self.ram[SPRITE_G + k] == 9 {
                self.ram[SPRITE_G + k] = 0;
            }
            self.ram[SPRITE_GRAPHICS + k] = K_ARRGHUS_GFX[usize::from(self.ram[SPRITE_G + k])];
        }

        let collision = self.sprite_check_tile_collision(k);
        if collision != 0 {
            if self.ram[SPRITE_AI_STATE + k] == 5 {
                if (collision & 3) != 0 {
                    self.ram[SPRITE_X_VEL + k] = (0u8).wrapping_sub(self.ram[SPRITE_X_VEL + k]);
                } else {
                    self.ram[SPRITE_Y_VEL + k] = (0u8).wrapping_sub(self.ram[SPRITE_Y_VEL + k]);
                }
            } else {
                self.sprite_zero_velocity_xy(k);
            }
        }

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.ram[SPRITE_DELAY_MAIN + k] = 48;
                }
                self.sprite_move_xy(k);
                self.sprite_approach_target_speed(
                    k,
                    self.ram[SPRITE_HEAD_DIR + k],
                    self.ram[SPRITE_D + k],
                );
            }
            1 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    if !self.sprite_check_if_screen_is_clear() {
                        self.ram[OVERLORD_X_LO_MOTHULA + 3] =
                            self.ram[OVERLORD_X_LO_MOTHULA + 3].wrapping_add(1);
                        if self.ram[OVERLORD_X_LO_MOTHULA + 3] == 4 {
                            self.ram[OVERLORD_X_LO_MOTHULA + 3] = 0;
                            self.ram[SPRITE_AI_STATE + k] = 2;
                            self.ram[SPRITE_DELAY_MAIN + k] = 176;
                        } else {
                            self.ram[SPRITE_DELAY_MAIN + k] =
                                (self.get_random_number() & 63).wrapping_add(48);
                            let speed = (self.ram[SPRITE_DELAY_MAIN + k] & 3).wrapping_add(8);
                            let pt = self.sprite_project_speed_towards_link(k, speed);
                            self.ram[SPRITE_HEAD_DIR + k] = pt.x;
                            self.ram[SPRITE_D + k] = pt.y;
                        }
                    } else {
                        self.ram[SPRITE_AI_STATE + k] = 3;
                        self.sprite_sfx_queue_sfx3_with_pan(k, 0x32);
                        self.ram[SPRITE_SUBTYPE2 + k] = 0;
                    }
                } else {
                    self.sprite_move_xy(k);
                    self.sprite_approach_target_speed(k, 0, 0);
                }
            }
            2 => {
                self.ram[OVERLORD_X_LO_MOTHULA + 4] = 8;
                if self.ram[SPRITE_DELAY_MAIN + k] < 32 {
                    self.ram[OVERLORD_X_LO_MOTHULA + 2] =
                        self.ram[OVERLORD_X_LO_MOTHULA + 2].wrapping_sub(1);
                    if sign8(self.ram[OVERLORD_X_LO_MOTHULA + 2]) {
                        self.ram[OVERLORD_X_LO_MOTHULA + 2] = 0;
                        self.ram[SPRITE_AI_STATE + k] = 1;
                        self.ram[SPRITE_DELAY_MAIN + k] = 112;
                    }
                } else if self.ram[SPRITE_DELAY_MAIN + k] < 96 {
                    self.ram[OVERLORD_X_LO_MOTHULA + 2] =
                        self.ram[OVERLORD_X_LO_MOTHULA + 2].wrapping_add(1);
                } else if self.ram[SPRITE_DELAY_MAIN + k] == 96 {
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x26);
                } else if (self.ram[SPRITE_DELAY_MAIN + k] & 0x0f) == 0 {
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x06);
                }
            }
            3 => {
                self.ram[SPRITE_Z_VEL + k] = 120;
                self.sprite_move_z(k);
                if self.ram[SPRITE_Z + k] >= 224 {
                    self.ram[SPRITE_DELAY_MAIN + k] = 64;
                    self.ram[SPRITE_AI_STATE + k] = 4;
                    self.ram[SPRITE_Z_VEL + k] = 0;
                    self.ram[SPRITE_X_LO + k] = self.ram[LINK_X_COORD];
                    self.ram[SPRITE_Y_LO + k] = self.ram[LINK_Y_COORD];
                }
            }
            4 => {
                let mut a = self.ram[SPRITE_DELAY_MAIN + k];
                if a == 0 {
                    self.ram[SPRITE_Z_VEL + k] = 144;
                    let old_z = self.ram[SPRITE_Z + k];
                    self.sprite_move_z(k);
                    a = old_z ^ self.ram[SPRITE_Z + k];
                    if sign8(a) {
                        a = self.ram[SPRITE_Z + k];
                        if sign8(a) {
                            self.ram[SPRITE_Z + k] = 0;
                            self.sprite_spawn_big_splash(k);
                            self.ram[SPRITE_AI_STATE + k] = 5;
                            self.ram[SPRITE_DELAY_MAIN + k] = 32;
                            self.sprite_sfx_queue_sfx3_with_pan(k, 0x03);
                            self.ram[SPRITE_X_VEL + k] = 32;
                            self.ram[SPRITE_Y_VEL + k] = 32;
                        }
                    }
                }
                if a == 1 {
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
                }
            }
            5 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_FLAGS3 + k] = 0;
                    self.sprite_move_xy(k);
                    self.sprite_check_damage_from_link(k);
                    if (self.ram[FRAME_COUNTER] & 7) == 0 {
                        self.sprite_sfx_queue_sfx2_with_pan(k, 0x28);
                        let j = self.garnish_alloc_limit(if sign8(self.ram[SPRITE_Y_VEL + k]) {
                            29
                        } else {
                            14
                        });
                        if j >= 0 {
                            let j = j as usize;
                            self.ram[GARNISH_TYPE + j] = 21;
                            self.ram[GARNISH_ACTIVE_MOTHULA] = 21;
                            self.ram[GARNISH_X_LO_MOTHULA + j] = self.ram[SPRITE_X_LO + k];
                            self.ram[GARNISH_X_HI_MOTHULA + j] = self.ram[SPRITE_X_HI + k];
                            self.ram[GARNISH_Y_LO_MOTHULA + j] =
                                self.ram[SPRITE_Y_LO + k].wrapping_add(24);
                            self.ram[GARNISH_Y_HI_MOTHULA + j] = self.ram[SPRITE_Y_HI + k];
                            self.ram[GARNISH_COUNTDOWN_MOTHULA + j] = 15;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // void Arrghus_Draw(int k) {  // 9eb840
    pub(super) fn arrghus_draw(&mut self, k: usize) {
        const D: [DrawMultipleData; 5] = [
            DrawMultipleData {
                x: -8,
                y: -4,
                char_flags: 0x0080,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: -4,
                char_flags: 0x4080,
                ext: 2,
            },
            DrawMultipleData {
                x: -8,
                y: 12,
                char_flags: 0x00a0,
                ext: 2,
            },
            DrawMultipleData {
                x: 8,
                y: 12,
                char_flags: 0x40a0,
                ext: 2,
            },
            DrawMultipleData {
                x: 0,
                y: 24,
                char_flags: 0x00a8,
                ext: 2,
            },
        ];
        self.sprite_draw_multiple(k, &D, None);
        let oam = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        let chr = self.ram[SPRITE_GRAPHICS + k].wrapping_mul(2);
        for i in 0..4 {
            self.ram[oam + i * 4 + 2] = self.ram[oam + i * 4 + 2].wrapping_add(chr);
        }
        if self.ram[SPRITE_AI_STATE + k] == 5 {
            self.ram[oam + 4 * 4 + 1] = 0xf0;
        }
        if (self.ram[SPRITE_SUBTYPE2 + k] & 8) != 0 {
            self.ram[oam + 4 * 4 + 3] |= 0x40;
        }

        if self.ram[SPRITE_AI_STATE + k] != 5 {
            let cur = read_le_u16(&self.ram, OAM_CUR_PTR);
            write_le_u16(&mut self.ram, OAM_CUR_PTR, cur.wrapping_add(4));
            let ext = read_le_u16(&self.ram, OAM_EXT_CUR_PTR);
            write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, ext.wrapping_add(1));
            if self.ram[SPRITE_Z + k] < 0xa0 {
                let bak = self.ram[SPRITE_OAM_FLAGS + k];
                self.ram[SPRITE_OAM_FLAGS + k] &= !1;
                self.sprite_draw_big_shadow(k, 0);
                self.ram[SPRITE_OAM_FLAGS + k] = bak;
            }
        } else {
            self.sprite_draw_large_water_turbulence(k);
        }
    }

    // void Sprite_8D_Arrghi(int k) {  // 9eb8c4
    pub(super) fn sprite_8_d_arrghi(&mut self, k: usize) {
        self.ram[SPRITE_OBJ_PRIO + k] |= 0x30;
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        self.ram[SPRITE_GRAPHICS + k] =
            K_ARRGI_GFX[usize::from((self.ram[SPRITE_SUBTYPE2 + k] >> 3) & 7)];

        if self.ram[SPRITE_B + k] != 0 {
            let j = usize::from(self.ram[SPRITE_B + k] - 1);
            if self.ram[ANCILLA_TYPE + j] != 0 {
                self.ram[SPRITE_X_LO + k] = self.ram[ANCILLA_X_LO + j];
                self.ram[SPRITE_X_HI + k] = self.ram[ANCILLA_X_HI + j];
                self.ram[SPRITE_Y_LO + k] = self.ram[ANCILLA_Y_LO + j];
                self.ram[SPRITE_Y_HI + k] = self.ram[ANCILLA_Y_HI + j];
                self.ram[SPRITE_OAM_FLAGS + k] = 5;
                self.ram[SPRITE_FLAGS3 + k] &= !0x40;
                return;
            }
            self.ram[SPRITE_AI_STATE + k] = 1;
            self.ram[SPRITE_B + k] = 0;
            self.ram[SPRITE_DELAY_MAIN + k] = 32;
        }

        if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
            self.sprite_check_damage_to_link(k);
        }

        if self.ram[SPRITE_AI_STATE + k] == 0 {
            self.ram[SPRITE_X_LO + k] = self.ram[OVERLORD_X_LO_MOTHULA + k + 7];
            self.ram[SPRITE_X_HI + k] = self.ram[OVERLORD_Y_LO_MOTHULA + k + 7];
            self.ram[SPRITE_Y_LO + k] = self.ram[OVERLORD_GEN1_MOTHULA + k + 7];
            self.ram[SPRITE_Y_HI + k] = self.ram[OVERLORD_GEN3_MOTHULA + k + 7];
            return;
        }

        self.sprite_check_damage_from_link(k);
        if (((k as u8) ^ self.ram[FRAME_COUNTER]) & 3) == 0 {
            let x = u16::from(self.ram[OVERLORD_X_LO_MOTHULA + k + 7])
                | (u16::from(self.ram[OVERLORD_Y_LO_MOTHULA + k + 7]) << 8);
            let y = u16::from(self.ram[OVERLORD_GEN1_MOTHULA + k + 7])
                | (u16::from(self.ram[OVERLORD_GEN3_MOTHULA + k + 7]) << 8);
            let pt = self.sprite_project_speed_towards_location(k, x, y, 4);
            self.ram[SPRITE_Y_VEL + k] = pt.y;
            self.ram[SPRITE_X_VEL + k] = pt.x;
            if self.ram[SPRITE_X_LO + k]
                .wrapping_sub(self.ram[OVERLORD_X_LO_MOTHULA + k + 7])
                .wrapping_add(8)
                < 16
                && self.ram[SPRITE_Y_LO + k]
                    .wrapping_sub(self.ram[OVERLORD_GEN1_MOTHULA + k + 7])
                    .wrapping_add(8)
                    < 16
            {
                self.ram[SPRITE_AI_STATE + k] = 0;
                self.ram[SPRITE_OAM_FLAGS + k] = 0x0d;
                self.ram[SPRITE_FLAGS3 + k] |= 0x40;
            }
        }
        self.sprite_move_xy(k);
    }

    // void Sprite_8F_Blob(int k) {  // 9eb002
    pub(super) fn sprite_8_f_blob(&mut self, k: usize) {
        if self.ram[SPRITE_STATE + k] == 9 && self.ram[SPRITE_E + k] != 0 {
            self.ram[SPRITE_E + k] = 0;
            self.ram[SPRITE_X_VEL + k] = 1;
            let collided = self.sprite_check_tile_collision(k);
            self.ram[SPRITE_X_VEL + k] = 0;
            if collided != 0 {
                self.ram[SPRITE_STATE + k] = 0;
                return;
            }
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
        }

        if self.ram[SPRITE_C + k] != 0 {
            self.ram[SPRITE_OBJ_PRIO + k] = 0x30;
        }
        self.zol_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }

        if self.ram[SPRITE_AI_STATE + k] >= 2 {
            self.sprite_check_damage_from_link(k);
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }

        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                let bak = self.ram[SPRITE_FLAGS4 + k];
                self.ram[SPRITE_FLAGS4 + k] |= 9;
                self.ram[SPRITE_FLAGS2 + k] |= 0x80;
                let hit_link = self.sprite_check_damage_to_link(k);
                self.ram[SPRITE_FLAGS4 + k] = bak;
                if hit_link {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    self.ram[SPRITE_DELAY_MAIN + k] = 127;
                    self.ram[SPRITE_FLAGS2 + k] &= !0x80;
                    self.sprite_set_x(k, self.player_state_view().x());
                    self.sprite_set_y(k, self.player_state_view().y().wrapping_add(8));
                    self.ram[SPRITE_DELAY_AUX4 + k] = 48;
                    self.ram[SPRITE_IGNORE_PROJECTILE + k] = 0;
                }
            }
            1 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    self.ram[SPRITE_Z_VEL + k] = 32;
                    self.sprite_apply_speed_towards_link(k, 16);
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x30);
                } else {
                    const POPPING_OUT_GFX: [u8; 16] =
                        [0, 1, 7, 7, 6, 6, 5, 5, 6, 6, 5, 5, 4, 4, 4, 4];
                    self.ram[SPRITE_GRAPHICS + k] =
                        POPPING_OUT_GFX[usize::from(self.ram[SPRITE_DELAY_MAIN + k] >> 3)];
                }
            }
            2 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.sprite_check_damage_from_link(k);
                    self.sprite_move_xy(k);
                    self.sprite_check_tile_collision(k);
                    let old_z = self.ram[SPRITE_Z + k];
                    self.sprite_move_z(k);
                    if !sign8(self.ram[SPRITE_Z_VEL + k].wrapping_add(64)) {
                        self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
                    }
                    if sign8(self.ram[SPRITE_Z + k] ^ old_z) && sign8(self.ram[SPRITE_Z + k]) {
                        self.ram[SPRITE_Z_VEL + k] = 0;
                        self.ram[SPRITE_Z + k] = 0;
                        self.ram[SPRITE_C + k] = 0;
                        self.ram[SPRITE_DELAY_MAIN + k] = 31;
                        self.ram[SPRITE_HEAD_DIR + k] = 8;
                    }
                } else if self.ram[SPRITE_DELAY_MAIN + k] == 1 {
                    self.ram[SPRITE_DELAY_MAIN + k] = 32;
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_add(1);
                    self.ram[SPRITE_GRAPHICS + k] = 0;
                } else {
                    const FALLING_XVEL: [i8; 2] = [-8, 8];
                    const FALLING_GFX: [u8; 2] = [0, 1];
                    self.ram[SPRITE_GRAPHICS + k] =
                        FALLING_GFX[usize::from((self.ram[SPRITE_DELAY_MAIN + k] - 1) >> 4)];
                    self.ram[SPRITE_X_VEL + k] =
                        FALLING_XVEL[usize::from((self.ram[FRAME_COUNTER] >> 1) & 1)] as u8;
                    self.sprite_move_x(k);
                }
            }
            3 => {
                self.sprite_check_damage_to_link(k);
                if self.ram[SPRITE_DELAY_AUX1 + k] == 0 {
                    self.sprite_apply_speed_towards_link(k, 48);
                    self.ram[SPRITE_DELAY_AUX1 + k] = (self.get_random_number() & 63) | 96;
                    self.ram[SPRITE_OAM_FLAGS + k] = (self.ram[SPRITE_OAM_FLAGS + k] & 0x3f)
                        | if sign8(self.ram[SPRITE_X_VEL + k]) {
                            0x40
                        } else {
                            0
                        };
                }
                if self.ram[SPRITE_DELAY_AUX2 + k] == 0 {
                    self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                    if ((self.ram[SPRITE_SUBTYPE2 + k] & 14) | self.ram[SPRITE_WALLCOLL + k]) == 0 {
                        self.sprite_move_xy(k);
                        self.ram[SPRITE_G + k] = self.ram[SPRITE_G + k].wrapping_add(1);
                        if self.ram[SPRITE_G + k] == self.ram[SPRITE_HEAD_DIR + k] {
                            self.ram[SPRITE_G + k] = 0;
                            self.ram[SPRITE_DELAY_AUX2 + k] =
                                (self.get_random_number() & 31).wrapping_add(64);
                            self.ram[SPRITE_HEAD_DIR + k] = (self.get_random_number() & 31) | 16;
                        }
                    }
                    self.sprite_check_tile_collision(k);
                    self.ram[SPRITE_GRAPHICS + k] = (self.ram[SPRITE_SUBTYPE2 + k] & 8) >> 3;
                } else {
                    self.ram[SPRITE_GRAPHICS + k] = if (self.ram[SPRITE_DELAY_AUX2 + k] & 0x10) != 0
                    {
                        1
                    } else {
                        0
                    };
                }
            }
            _ => {}
        }
    }

    // void Sprite_8E_Terrorpin(int k) {  // 9eb26f
    pub(super) fn sprite_8_e_terrorpin(&mut self, k: usize) {
        self.sprite_draw_single_large(k);
        self.sprite_check_tile_collision(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        if self.ram[SPRITE_DELAY_AUX2 + k] == 0 {
            self.sprite_check_damage_from_link(k);
        }
        self.terrorpin_check_for_hammer(k);
        self.sprite_move_xyz(k);

        match self.ram[SPRITE_B + k] {
            0 => {
                if self.ram[SPRITE_DELAY_AUX4 + k] == 0 {
                    self.ram[SPRITE_DELAY_AUX4 + k] =
                        (self.get_random_number() & 31).wrapping_add(32);
                    self.ram[SPRITE_D + k] = self.sprite_direction_to_face_link(k, None);
                }
                let j = usize::from(self.ram[SPRITE_D + k].wrapping_add(self.ram[SPRITE_G + k]));
                self.ram[SPRITE_X_VEL + k] = K_TERRORPIN_XVEL[j] as u8;
                self.ram[SPRITE_Y_VEL + k] = K_TERRORPIN_YVEL[j] as u8;
                self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
                if sign8(self.ram[SPRITE_Z + k]) {
                    self.ram[SPRITE_Z + k] = 0;
                    self.ram[SPRITE_Z_VEL + k] = 0;
                }
                self.ram[SPRITE_GRAPHICS + k] = (self.ram[FRAME_COUNTER]
                    >> if self.ram[SPRITE_G + k] != 0 { 2 } else { 3 })
                    & 1;
                self.ram[SPRITE_FLAGS3 + k] |= 64;
                self.ram[SPRITE_DEFL_BITS + k] = 4;
                self.sprite_check_damage_to_link(k);
            }
            1 => {
                self.ram[SPRITE_FLAGS3 + k] &= 191;
                self.ram[SPRITE_DEFL_BITS + k] = 0;
                if self.ram[SPRITE_DELAY_AUX4 + k] == 0 {
                    self.ram[SPRITE_B + k] = 0;
                    self.ram[SPRITE_Z_VEL + k] = 32;
                    self.ram[SPRITE_DELAY_AUX4 + k] = 64;
                    return;
                }
                self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
                if sign8(self.ram[SPRITE_Z + k]) {
                    self.ram[SPRITE_Z + k] = 0;
                    let t = (0u8).wrapping_sub(self.ram[SPRITE_Z_VEL + k]) >> 1;
                    self.ram[SPRITE_Z_VEL + k] = if t < 9 { 0 } else { t };
                    self.ram[SPRITE_X_VEL + k] = (self.ram[SPRITE_X_VEL + k] as i8 >> 1) as u8;
                    if self.ram[SPRITE_X_VEL + k] == 0xff {
                        self.ram[SPRITE_X_VEL + k] = 0;
                    }
                    self.ram[SPRITE_Y_VEL + k] = (self.ram[SPRITE_Y_VEL + k] as i8 >> 1) as u8;
                    if self.ram[SPRITE_Y_VEL + k] == 0xff {
                        self.ram[SPRITE_Y_VEL + k] = 0;
                    }
                }
                if self.ram[SPRITE_DELAY_AUX4 + k] < 64 {
                    self.ram[SPRITE_X_VEL + k] = K_TERRORPIN_OVERTURNED_XVEL
                        [usize::from((self.ram[SPRITE_DELAY_AUX4 + k] >> 1) & 1)]
                        as u8;
                    self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                }
                self.ram[SPRITE_GRAPHICS + k] = 2;
                self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                self.ram[SPRITE_OAM_FLAGS + k] = (self.ram[SPRITE_OAM_FLAGS + k] & !0x40)
                    | K_TERRORPIN_OAMFLAGS[usize::from((self.ram[SPRITE_SUBTYPE2 + k] >> 3) & 1)];
            }
            _ => {}
        }
    }

    // void Terrorpin_CheckForHammer(int k) {  // 9eb3a3
    pub(super) fn terrorpin_check_for_hammer(&mut self, k: usize) {
        if (self.ram[SPRITE_Z + k] | self.ram[SPRITE_DELAY_AUX2 + k]) == 0
            && self.ram[SPRITE_FLOOR + k] == self.ram[LINK_IS_ON_LOWER_LEVEL]
            && self.ram[PLAYER_OAM_Y_OFFSET] != 0x80
            && (self.ram[LINK_ITEM_IN_HAND] & 0x0a) != 0
        {
            let mut hb = SpriteHitBox {
                r0_xlo: 0,
                r8_xhi: 0,
                r1_ylo: 0,
                r9_yhi: 0,
                r2: 0,
                r3: 0,
                r4_spr_xlo: 0,
                r10_spr_xhi: 0,
                r5_spr_ylo: 0,
                r11_spr_yhi: 0,
                r6_spr_xsize: 0,
                r7_spr_ysize: 0,
            };
            self.player_setup_action_hit_box(&mut hb);
            self.terrorpin_set_up_hammer_hit_box(k, &mut hb);
            if self.check_if_hit_boxes_overlap(&hb) {
                self.ram[SPRITE_X_VEL + k] = (0u8).wrapping_sub(self.ram[SPRITE_X_VEL + k]);
                self.ram[SPRITE_Y_VEL + k] = (0u8).wrapping_sub(self.ram[SPRITE_Y_VEL + k]);
                self.ram[SPRITE_DELAY_AUX2 + k] = 32;
                self.ram[SPRITE_Z_VEL + k] = 32;
                self.ram[SPRITE_G + k] = 4;
                self.ram[SPRITE_B + k] ^= 1;
                self.ram[SPRITE_DELAY_AUX4 + k] = if self.ram[SPRITE_B + k] != 0 {
                    0xff
                } else {
                    0x40
                };
            }
        }
        self.ram[SPRITE_HEAD_DIR + k] = 0;
    }

    // void Terrorpin_SetUpHammerHitBox(int k, SpriteHitBox *hb) {  // 9eb405
    pub(super) fn terrorpin_set_up_hammer_hit_box(&self, k: usize, hb: &mut SpriteHitBox) {
        let x = self.sprite_get_x(k).wrapping_sub(16);
        let y = self.sprite_get_y(k).wrapping_sub(16);
        hb.r4_spr_xlo = x as u8;
        hb.r10_spr_xhi = (x >> 8) as u8;
        hb.r5_spr_ylo = y as u8;
        hb.r11_spr_yhi = (y >> 8) as u8;
        hb.r6_spr_xsize = 48;
        hb.r7_spr_ysize = 48;
    }

    // void Sprite_8B_Gibdo(int k) {  // 9eb9a9
    pub(super) fn sprite_8_b_gibdo(&mut self, k: usize) {
        self.gibdo_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.ram[SPRITE_GRAPHICS + k] = K_GIBDO_GFX[self.ram[SPRITE_D + k] as usize];
                if (self.ram[FRAME_COUNTER] & 7) == 0 {
                    let j = self.ram[SPRITE_A + k] as usize;
                    let delta = self.ram[SPRITE_D + k].wrapping_sub(K_GIBDO_DIR_TARGET[j]);
                    if delta != 0 {
                        self.ram[SPRITE_D + k] = self.ram[SPRITE_D + k]
                            .wrapping_add(if sign8(delta) { 1 } else { 0xff });
                    } else {
                        self.ram[SPRITE_DELAY_MAIN + k] =
                            (self.get_random_number() & 31).wrapping_add(48);
                        self.ram[SPRITE_AI_STATE + k] = 1;
                    }
                }
            }
            1 => {
                let j = self.ram[SPRITE_D + k] as usize;
                self.ram[SPRITE_X_VEL + k] = K_GIBDO_XY_VEL[j + 2] as u8;
                self.ram[SPRITE_Y_VEL + k] = K_GIBDO_XY_VEL[j] as u8;
                self.sprite_move_xy(k);
                self.sprite_check_tile_collision(k);
                let mut turned = false;
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 || self.ram[SPRITE_WALLCOLL + k] != 0 {
                    let face = self.sprite_direction_to_face_link(k, None);
                    if face != self.ram[SPRITE_A + k] {
                        self.ram[SPRITE_A + k] = face;
                        self.ram[SPRITE_AI_STATE + k] = 0;
                        turned = true;
                    }
                }
                if !turned {
                    self.ram[SPRITE_B + k] = self.ram[SPRITE_B + k].wrapping_sub(1);
                    if sign8(self.ram[SPRITE_B + k]) {
                        self.ram[SPRITE_B + k] = 14;
                        self.ram[SPRITE_SUBTYPE2 + k] =
                            self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
                    }
                    let idx = (((self.ram[SPRITE_SUBTYPE2 + k] & 1) << 2) | self.ram[SPRITE_A + k])
                        as usize;
                    self.ram[SPRITE_GRAPHICS + k] = K_GIBDO_GFX2[idx];
                }
            }
            _ => {}
        }
    }

    // void Sprite_89_MothulaBeam(int k) {  // 9ebb42
    pub(super) fn sprite_89_mothula_beam(&mut self, k: usize) {
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_check_damage_to_link(k);
        if (self.ram[FRAME_COUNTER] & 1) == 0 {
            self.ram[SPRITE_OAM_FLAGS + k] ^= 0x80;
        }
        self.sprite_move_xy(k);
        if self.ram[SPRITE_DELAY_MAIN + k] == 0 && self.sprite_check_tile_collision(k) != 0 {
            self.ram[SPRITE_STATE + k] = 0;
        }
        if (((k as u8) ^ self.ram[FRAME_COUNTER]) & 3) != 0 {
            return;
        }
        for i in (0..=14usize).rev() {
            if self.ram[GARNISH_TYPE + i] == 0 {
                self.ram[GARNISH_TYPE + i] = 2;
                self.ram[GARNISH_ACTIVE_MOTHULA] = 2;
                self.ram[GARNISH_X_LO_MOTHULA + i] = self.ram[SPRITE_X_LO + k];
                self.ram[GARNISH_X_HI_MOTHULA + i] = self.ram[SPRITE_X_HI + k];
                self.ram[GARNISH_Y_LO_MOTHULA + i] = self.ram[SPRITE_Y_LO + k];
                self.ram[GARNISH_Y_HI_MOTHULA + i] = self.ram[SPRITE_Y_HI + k];
                self.ram[GARNISH_COUNTDOWN_MOTHULA + i] = 16;
                self.ram[GARNISH_SPRITE_MOTHULA + i] = k as u8;
                self.ram[GARNISH_FLOOR_MOTHULA + i] = self.ram[SPRITE_FLOOR + k];
                break;
            }
        }
    }

    // void Sprite_94_Tile(int k) {  // 9ebbb9
    pub(super) fn sprite_94_tile(&mut self, k: usize) {
        self.ram[SPRITE_OBJ_PRIO + k] = 0x30;
        self.flying_tile_draw(k);
        if self.sprite_return_if_paused(k) {
            return;
        }
        if self.ram[SPRITE_HIT_TIMER + k] != 0 {
            self.sprite_94_tile_break(k);
            return;
        }
        self.ram[SPRITE_IGNORE_PROJECTILE + k] = 1;
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                let y = u16::from(self.ram[SPRITE_Y_LO + k].wrapping_add(8))
                    | (u16::from(self.ram[SPRITE_Y_HI + k]) << 8);
                self.dungeon_update_tile_map_with_common_tile_for_mothula(
                    self.sprite_get_x(k),
                    y,
                    6,
                );
                self.ram[SPRITE_AI_STATE + k] = 1;
                self.ram[SPRITE_DELAY_MAIN + k] = 128;
            }
            1 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[SPRITE_DELAY_MAIN + k] = 16;
                    self.sprite_apply_speed_towards_link(k, 32);
                } else {
                    if self.ram[SPRITE_DELAY_MAIN + k] >= 0x40 {
                        self.ram[SPRITE_Z_VEL + k] = 4;
                        self.sprite_move_z(k);
                    }
                    self.sprite_94_tile_animate(k);
                }
            }
            2 => {
                self.ram[SPRITE_IGNORE_PROJECTILE + k] = 0;
                if self.ram[SPRITE_DELAY_MAIN + k] != 0
                    && (self.ram[SPRITE_DELAY_MAIN + k] & 3) == 0
                {
                    self.sprite_apply_speed_towards_link(k, 32);
                }
                if !self.sprite_check_damage_to_and_from_link(k) {
                    self.sprite_move_xy(k);
                    let cy = read_le_u16(&self.ram, CUR_SPRITE_Y)
                        .wrapping_sub(u16::from(self.ram[SPRITE_Z + k]));
                    write_le_u16(&mut self.ram, CUR_SPRITE_Y, cy);
                    if self.sprite_check_tile_collision(k) == 0 {
                        self.sprite_94_tile_animate(k);
                        return;
                    }
                }
                self.sprite_94_tile_break(k);
            }
            _ => {}
        }
    }

    // void Sprite_94_Pirogusu(int k) {  // 9ea742
    pub(super) fn sprite_94_pirogusu(&mut self, k: usize) {
        if self.ram[SPRITE_E + k] != 0 {
            self.sprite_94_tile(k);
            return;
        }
        self.ram[SPRITE_OBJ_PRIO + k] |= 0x30;
        self.pirogusu_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.ram[SPRITE_DELAY_MAIN + k] = 31;
                }
                self.ram[SPRITE_IGNORE_PROJECTILE + k] = self.ram[SPRITE_DELAY_MAIN + k];
                self.ram[SPRITE_A + k] = K_PIROGUSU_A0[self.ram[SPRITE_D + k] as usize];
            }
            1 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[SPRITE_DELAY_MAIN + k] = 32;
                    self.ram[SPRITE_IGNORE_PROJECTILE + k] = 0;
                    self.sprite_zero_velocity_xy(k);
                } else {
                    let j = self.ram[SPRITE_D + k] as usize;
                    let idx = ((self.ram[SPRITE_DELAY_MAIN + k] >> 3) & 1) as usize | (j << 1);
                    self.ram[SPRITE_A + k] = K_PIROGUSU_A1[idx];
                    self.ram[SPRITE_X_VEL + k] = K_PIROGUSU_XY_VEL[j + 2] as u8;
                    self.ram[SPRITE_Y_VEL + k] = K_PIROGUSU_XY_VEL[j] as u8;
                    self.sprite_move_xy(k);
                }
            }
            2 => {
                self.sprite_check_damage_to_and_from_link(k);
                self.sprite_move_xy(k);
                let j = self.ram[SPRITE_D + k] as usize;
                self.ram[SPRITE_X_VEL + k] =
                    self.ram[SPRITE_X_VEL + k].wrapping_add(K_PIROGUSU_XY_VEL2[j] as u8);
                self.ram[SPRITE_Y_VEL + k] =
                    self.ram[SPRITE_Y_VEL + k].wrapping_add(K_PIROGUSU_XY_VEL2[j + 2] as u8);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.sprite_spawn_small_splash(k);
                    self.ram[SPRITE_DELAY_AUX1 + k] = 16;
                    self.ram[SPRITE_AI_STATE + k] = 3;
                }
                let idx = ((self.ram[FRAME_COUNTER] >> 2) & 1) as usize | (j << 1);
                self.ram[SPRITE_A + k] = K_PIROGUSU_A2[idx];
            }
            3 => {
                if self.sprite_return_if_recoiling(k) {
                    return;
                }
                self.sprite_check_damage_to_and_from_link(k);
                let j = self.ram[SPRITE_D + k] as usize;
                let idx = ((self.ram[FRAME_COUNTER] >> 2) & 1) as usize | (j << 1);
                self.ram[SPRITE_A + k] = K_PIROGUSU_A2[idx].wrapping_add(8);
                if self.ram[SPRITE_DELAY_AUX1 + k] == 0 {
                    self.pirogusu_spawn_splash(k);
                    self.sprite_move_xy(k);
                    if (self.sprite_check_tile_collision(k) & 15) != 0 {
                        let rnd = self.get_random_number() & 1;
                        self.ram[SPRITE_D + k] = K_PIROGUSU_DIR[(j << 1) | rnd as usize];
                    }
                    let j = self.ram[SPRITE_D + k] as usize;
                    self.ram[SPRITE_X_VEL + k] = K_PIROGUSU_XY_VEL3[j] as u8;
                    self.ram[SPRITE_Y_VEL + k] = K_PIROGUSU_XY_VEL3[j + 2] as u8;
                }
            }
            _ => {}
        }
    }

    // void Sprite_LaserBeam(int k) {  // 9ea462
    pub(super) fn sprite_laser_beam(&mut self, k: usize) {
        self.sprite_draw_single_small(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.laser_beam_build_up_garnish(k);
        self.sprite_move_xy(k);
        self.sprite_check_damage_to_link_same_layer(k);
        if self.ram[SPRITE_DELAY_MAIN + k] == 0 && self.sprite_check_tile_collision(k) != 0 {
            self.ram[SPRITE_STATE + k] = 0;
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x26);
        }
    }

    // void Sprite_95_LaserEyeLeft(int k) {  // 9ea541
    pub(super) fn sprite_95_laser_eye_left(&mut self, k: usize) {
        if self.ram[SPRITE_A + k] != 0 {
            self.sprite_laser_beam(k);
            return;
        }
        self.laser_eye_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                let facing = ((self.ram[LINK_DIRECTION_FACING] >> 1) & 3) as usize;
                if self.ram[SPRITE_HEAD_DIR + k] == 0
                    && self.ram[SPRITE_D + k] != K_LASER_EYE_DIRS[facing]
                {
                    self.ram[SPRITE_GRAPHICS + k] = 0;
                } else {
                    let j = if self.ram[SPRITE_D + k] < 2 {
                        self.player_state_view()
                            .y()
                            .wrapping_sub(read_le_u16(&self.ram, CUR_SPRITE_Y))
                    } else {
                        self.player_state_view()
                            .x()
                            .wrapping_sub(read_le_u16(&self.ram, CUR_SPRITE_X))
                    };
                    if j.wrapping_add(16) < 32 {
                        self.ram[SPRITE_DELAY_MAIN + k] = 32;
                        self.ram[SPRITE_AI_STATE + k] = 1;
                    } else {
                        self.ram[SPRITE_GRAPHICS + k] = 0;
                    }
                }
            }
            1 => {
                self.ram[SPRITE_GRAPHICS + k] = 1;
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 0;
                    self.laser_eye_fire_beam(k);
                    self.ram[SPRITE_DELAY_AUX4 + k] = 12;
                }
            }
            _ => {}
        }
    }

    // void Sprite_91_StalfosKnight(int k) {  // 9eaaa7
    pub(super) fn sprite_91_stalfos_knight(&mut self, k: usize) {
        if self.ram[SPRITE_AI_STATE + k] == 0 {
            let _ = self.sprite_prep_oam_coord_or_double_ret(k);
        } else {
            self.stalfos_knight_draw(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if (self.ram[SPRITE_HIT_TIMER + k] & 127) == 1 {
            self.ram[SPRITE_HIT_TIMER + k] = 0;
            self.ram[SPRITE_AI_STATE + k] = 6;
            self.ram[SPRITE_DELAY_MAIN + k] = 255;
            self.ram[SPRITE_X_VEL + k] = 0;
            self.ram[SPRITE_Y_VEL + k] = 0;
            self.ram[ENEMY_DAMAGE_DATA + 0x918] = 2;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                self.ram[SPRITE_FLAGS4 + k] = 9;
                self.ram[SPRITE_IGNORE_PROJECTILE + k] = 9;
                let bak0 = self.ram[SPRITE_FLAGS2 + k];
                self.ram[SPRITE_FLAGS2 + k] |= 128;
                let flag = self.sprite_check_damage_to_link(k);
                self.ram[SPRITE_FLAGS2 + k] = bak0;
                if flag {
                    self.ram[SPRITE_Z + k] = 144;
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.ram[SPRITE_HEAD_DIR + k] = 2;
                    self.ram[SPRITE_GRAPHICS + k] = 2;
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
                }
            }
            1 => {
                let old_z = self.ram[SPRITE_Z + k];
                self.sprite_move_z(k);
                if !sign8(self.ram[SPRITE_Z_VEL + k].wrapping_add(64)) {
                    self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(3);
                }
                if sign8(old_z ^ self.ram[SPRITE_Z + k]) && sign8(self.ram[SPRITE_Z + k]) {
                    self.stalfos_knight_set_to_ground(k);
                }
            }
            2 => {
                self.ram[ENEMY_DAMAGE_DATA + 0x918] = 0;
                self.sprite_check_damage_to_and_from_link(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 3;
                    self.ram[SPRITE_B + k] = self.get_random_number() & 63;
                    self.ram[SPRITE_DELAY_MAIN + k] = 127;
                } else {
                    let gfx =
                        K_STALFOS_KNIGHT_CASE2_GFX[(self.ram[SPRITE_DELAY_MAIN + k] >> 5) as usize];
                    self.ram[SPRITE_C + k] = gfx;
                    self.ram[SPRITE_GRAPHICS + k] = gfx;
                    self.ram[SPRITE_HEAD_DIR + k] = 2;
                }
            }
            3 => {
                self.sprite_check_damage_to_and_from_link(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == self.ram[SPRITE_B + k] {
                    self.ram[SPRITE_HEAD_DIR + k] = self.sprite_is_right_of_link(k).a;
                    self.ram[SPRITE_AI_STATE + k] = 4;
                    self.ram[SPRITE_DELAY_MAIN + k] = 32;
                } else {
                    self.ram[SPRITE_HEAD_DIR + k] =
                        K_STALFOS_KNIGHT_CASE2_DIR[(self.ram[SPRITE_DELAY_MAIN + k] >> 3) as usize];
                    self.ram[SPRITE_C + k] = 0;
                    self.ram[SPRITE_GRAPHICS + k] = 0;
                }
            }
            4 => {
                self.sprite_check_damage_to_and_from_link(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 5;
                    self.ram[SPRITE_DELAY_MAIN + k] = 255;
                    self.ram[SPRITE_DELAY_AUX1 + k] = 32;
                }
                self.ram[SPRITE_C + k] = 1;
                self.ram[SPRITE_GRAPHICS + k] = 1;
            }
            5 => {
                self.sprite_check_damage_to_and_from_link(k);
                if self.ram[SPRITE_DELAY_AUX1 + k] == 0 {
                    self.sprite_move_xyz(k);
                    self.sprite_check_tile_collision(k);
                    if !sign8(self.ram[SPRITE_Z_VEL + k].wrapping_add(64)) {
                        self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
                    }
                    if sign8(self.ram[SPRITE_Z + k].wrapping_sub(1)) {
                        self.ram[SPRITE_Z + k] = 0;
                        self.ram[SPRITE_Z_VEL + k] = 0;
                        if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                            self.stalfos_knight_set_to_ground(k);
                            return;
                        }
                        self.ram[SPRITE_DELAY_AUX1 + k] = 16;
                    }
                    self.ram[SPRITE_GRAPHICS + k] =
                        if sign8(self.ram[SPRITE_Z_VEL + k].wrapping_sub(24)) {
                            2
                        } else {
                            0
                        };
                } else {
                    if self.ram[SPRITE_DELAY_AUX1 + k] == 1 {
                        self.ram[SPRITE_Z_VEL + k] = 48;
                        self.sprite_apply_speed_towards_link(k, 16);
                        self.ram[SPRITE_HEAD_DIR + k] = self.sprite_is_right_of_link(k).a;
                        self.sprite_sfx_queue_sfx3_with_pan(k, 0x13);
                    }
                    self.ram[SPRITE_C + k] = 1;
                    self.ram[SPRITE_GRAPHICS + k] = 1;
                }
            }
            6 => {
                self.sprite_move_xyz(k);
                self.sprite_check_tile_collision(k);
                if !sign8(self.ram[SPRITE_Z_VEL + k].wrapping_add(64)) {
                    self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(2);
                }
                if sign8(self.ram[SPRITE_Z + k].wrapping_sub(1)) {
                    self.ram[SPRITE_Z + k] = 0;
                    self.ram[SPRITE_Z_VEL + k] = 0;
                }
                let j = self.ram[SPRITE_DELAY_MAIN + k];
                if j == 0 {
                    if (self.get_random_number() & 1) != 0 {
                        self.stalfos_knight_set_to_ground(k);
                    } else {
                        self.ram[SPRITE_AI_STATE + k] = 7;
                        self.ram[SPRITE_DELAY_MAIN + k] = 80;
                    }
                } else {
                    if j >= 224 && (j & 3) == 0 {
                        self.sprite_sfx_queue_sfx3_with_pan(k, 0x14);
                    }
                    self.ram[SPRITE_C + k] = K_STALFOS_KNIGHT_CASE6_C[(j >> 3) as usize];
                    self.ram[SPRITE_GRAPHICS + k] = 3;
                    self.ram[SPRITE_HEAD_DIR + k] = 2;
                }
            }
            7 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.stalfos_knight_set_to_ground(k);
                } else {
                    self.ram[SPRITE_GRAPHICS + k] = K_STALFOS_KNIGHT_CASE7_GFX
                        [((self.ram[SPRITE_DELAY_MAIN + k] >> 2) & 1) as usize];
                }
            }
            _ => {}
        }
    }

    // void Sprite_90_Wallmaster(int k) {  // 9eaea4
    pub(super) fn sprite_90_wallmaster(&mut self, k: usize) {
        self.ram[SPRITE_OBJ_PRIO + k] |= 0x30;
        self.wall_master_draw(k);
        if self.ram[SPRITE_STATE + k] != 9 {
            self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.ram[SPRITE_A + k] != 0 {
            let link_x = self.sprite_get_x(k);
            let link_y = self
                .sprite_get_y(k)
                .wrapping_sub(u16::from(self.ram[SPRITE_Z + k]))
                .wrapping_add(3);
            self.player_state_view_mut().set_x(link_x);
            self.player_state_view_mut().set_y(link_y);
            self.ram[FLAG_IS_LINK_IMMOBILIZED] = 1;
            self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 1;
            self.ram[LINK_INCAPACITATED_TIMER] = 0;
            self.ram[LINK_ACTUAL_VEL_X] = 0;
            self.ram[LINK_ACTUAL_VEL_Y] = 0;
            self.ram[LINK_Y_VEL] = 0;
            self.ram[LINK_X_VEL] = 0;
            if link_y
                .wrapping_sub(read_le_u16(&self.ram, BG2VOFS_COPY2))
                .wrapping_sub(16)
                >= 0x100
            {
                self.ram[FLAG_IS_LINK_IMMOBILIZED] = 0;
                self.ram[LINK_DISABLE_SPRITE_DAMAGE] = 0;
                self.wall_master_send_player_to_last_entrance();
                self.link_initialize();
                return;
            }
        } else {
            self.sprite_check_damage_from_link(k);
        }
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                let old_z = self.ram[SPRITE_Z + k];
                self.sprite_move_z(k);
                if !sign8(self.ram[SPRITE_Z_VEL + k].wrapping_add(64)) {
                    self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(3);
                }
                if sign8(old_z ^ self.ram[SPRITE_Z + k]) && sign8(self.ram[SPRITE_Z + k]) {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                    self.ram[SPRITE_Z + k] = 0;
                    self.ram[SPRITE_Z_VEL + k] = 0;
                    self.ram[SPRITE_DELAY_MAIN + k] = 63;
                }
            }
            1 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 2;
                }
                self.ram[SPRITE_GRAPHICS + k] = if (self.ram[SPRITE_DELAY_MAIN + k] & 0x20) != 0 {
                    0
                } else {
                    1
                };
                if self.sprite_check_damage_to_link(k) {
                    self.ram[SPRITE_A + k] = 1;
                    self.ram[SPRITE_FLAGS3 + k] = 64;
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x2a);
                }
            }
            2 => {
                let old_z = self.ram[SPRITE_Z + k];
                self.sprite_move_z(k);
                if sign8(self.ram[SPRITE_Z_VEL + k].wrapping_sub(64)) {
                    self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_add(2);
                }
                if sign8(old_z ^ self.ram[SPRITE_Z + k]) && !sign8(self.ram[SPRITE_Z + k]) {
                    self.ram[SPRITE_STATE + k] = 0;
                }
            }
            _ => {}
        }
    }

    // void Sprite_8A_SpikeBlock(int k) {  // 9ebce8
    pub(super) fn sprite_8_a_spike_block(&mut self, k: usize) {
        if self.ram[SPRITE_E + k] == 0 {
            self.sprite_draw_single_large(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            self.sprite_check_damage_to_and_from_link(k);
            self.sprite_move_xy(k);
            self.sprite_check_tile_collision(k);
            if self.ram[SPRITE_DELAY_MAIN + k] == 0
                && (!self.spike_block_check_statue_collision(k)
                    || (self.ram[SPRITE_WALLCOLL + k] & 0x0f) != 0)
            {
                self.ram[SPRITE_DELAY_MAIN + k] = 4;
                self.ram[SPRITE_X_VEL + k] = (0u8).wrapping_sub(self.ram[SPRITE_X_VEL + k]);
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x05);
            }
            return;
        }

        self.oam_allocate_from_region_b(4);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        if self.ram[SPRITE_AI_STATE + k] == 0 {
            self.dungeon_update_tile_map_with_common_tile_for_mothula(
                self.sprite_get_x(k),
                self.sprite_get_y(k),
                0,
            );
            self.ram[SPRITE_AI_STATE + k] = 1;
            self.ram[SPRITE_DELAY_MAIN + k] = 64;
            self.ram[SPRITE_DELAY_AUX1 + k] = 105;
        } else if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
            if self.ram[SPRITE_DELAY_MAIN + k] == 1 {
                self.ram[SPRITE_X_LO + k] = self.ram[SPRITE_A + k];
                self.ram[SPRITE_Y_LO + k] = self.ram[SPRITE_B + k];
            } else {
                self.ram[SPRITE_X_VEL + k] = if ((self.ram[SPRITE_DELAY_MAIN + k] >> 1) & 1) != 0 {
                    (-8i8) as u8
                } else {
                    8
                };
                self.sprite_move_x(k);
                self.ram[SPRITE_X_VEL + k] = 0;
            }
        } else if self.ram[SPRITE_AI_STATE + k] == 1 {
            const X_TARGET: [i8; 4] = [32, -32, 0, 0];
            const Y_TARGET: [i8; 4] = [0, 0, 32, -32];
            const X_DELTA: [i8; 4] = [1, -1, 0, 0];
            const Y_DELTA: [i8; 4] = [0, 0, 1, -1];
            let j = self.ram[SPRITE_D + k] as usize;
            if self.ram[SPRITE_X_VEL + k] != X_TARGET[j] as u8 {
                self.ram[SPRITE_X_VEL + k] =
                    self.ram[SPRITE_X_VEL + k].wrapping_add(X_DELTA[j] as u8);
            }
            if self.ram[SPRITE_Y_VEL + k] != Y_TARGET[j] as u8 {
                self.ram[SPRITE_Y_VEL + k] =
                    self.ram[SPRITE_Y_VEL + k].wrapping_add(Y_DELTA[j] as u8);
            }
            self.sprite_move_xy(k);
            if self.ram[SPRITE_DELAY_AUX1 + k] == 0 {
                self.sprite_get_16bit_coords_for_mothula(k);
                if self.sprite_check_tile_collision(k) != 0 {
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[SPRITE_DELAY_AUX1 + k] = 64;
                }
            }
        } else if self.ram[SPRITE_DELAY_AUX1 + k] == 0 {
            const X_VEL: [i8; 4] = [-16, 16, 0, 0];
            const Y_VEL: [i8; 4] = [0, 0, -16, 16];
            let j = self.ram[SPRITE_D + k] as usize;
            self.ram[SPRITE_X_VEL + k] = X_VEL[j] as u8;
            self.ram[SPRITE_Y_VEL + k] = Y_VEL[j] as u8;
            self.sprite_move_xy(k);
            if self.ram[SPRITE_X_LO + k] == self.ram[SPRITE_A + k]
                && self.ram[SPRITE_Y_LO + k] == self.ram[SPRITE_B + k]
            {
                self.ram[SPRITE_STATE + k] = 0;
                self.dungeon_update_tile_map_with_common_tile_for_mothula(
                    self.sprite_get_x(k),
                    self.sprite_get_y(k),
                    2,
                );
            }
        }
    }

    // void Sprite_7D_BigSpike(int k) {  // 9ecf47
    pub(super) fn sprite_7_d_big_spike(&mut self, k: usize) {
        const XVEL: [i8; 4] = [32, -32, 0, 0];
        const XVEL2: [i8; 4] = [-16, 16, 0, 0];
        const YVEL: [i8; 4] = [0, 0, 32, -32];
        const YVEL2: [i8; 4] = [0, 0, -16, 16];
        const DELAY: [u8; 4] = [0x40, 0x40, 0x38, 0x38];

        self.spike_trap_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        if self.ram[SPRITE_AI_STATE + k] == 0 {
            let mut pt = PointU8 { x: 0, y: 0 };
            let j = usize::from(self.sprite_direction_to_face_link(k, Some(&mut pt)));
            self.ram[SPRITE_D + k] = j as u8;
            if pt.x.wrapping_add(16) < 32 || pt.y.wrapping_add(16) < 32 {
                self.ram[SPRITE_DELAY_MAIN + k] = DELAY[j];
                self.ram[SPRITE_AI_STATE + k] = 1;
                self.ram[SPRITE_X_VEL + k] = XVEL[j] as u8;
                self.ram[SPRITE_Y_VEL + k] = YVEL[j] as u8;
            }
        } else if self.ram[SPRITE_AI_STATE + k] == 1 {
            if self.sprite_check_tile_collision(k) != 0 || self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                self.ram[SPRITE_AI_STATE + k] = 2;
                self.ram[SPRITE_DELAY_MAIN + k] = 96;
            }
            self.sprite_move_xy(k);
        } else if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
            let j = usize::from(self.ram[SPRITE_D + k]);
            self.ram[SPRITE_X_VEL + k] = XVEL2[j] as u8;
            self.ram[SPRITE_Y_VEL + k] = YVEL2[j] as u8;
            self.sprite_move_xy(k);
            if self.ram[SPRITE_X_LO + k] == self.ram[SPRITE_A + k]
                && self.ram[SPRITE_Y_LO + k] == self.ram[SPRITE_C + k]
            {
                self.ram[SPRITE_AI_STATE + k] = 0;
            }
        }
    }

    // void Sprite_88_Mothula(int k) {  // 9ebe7e
    pub(super) fn sprite_88_mothula(&mut self, k: usize) {
        if self.read_u32_ram(ENHANCED_FEATURES0) & K_FEATURES0_MISC_BUG_FIXES_MOTHULA != 0 {
            self.ram[ENEMY_DAMAGE_DATA + 0x884] = 1;
            self.ram[ENEMY_DAMAGE_DATA + 0x885] = 1;
        }
        self.mothula_main(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.mothula_handle_spikes(k);
    }

    // void Mothula_Draw(int k) {  // 9afdb5
    //   static const DrawMultipleData kMothula_Dmd[24] = { ... };
    //   oam_cur_ptr = 0x920;
    //   oam_ext_cur_ptr = 0xa68;
    //   PrepOamCoordsRet info;
    //   Sprite_DrawMultiple(k, &kMothula_Dmd[sprite_graphics[k] * 8], 8, &info);
    //   if (sprite_pause[k])
    //     return;
    //   info.y += sprite_z[k];
    //   static const int8 kMothula_Draw_X[27] = { ... };
    //   OamEnt *oam = GetOamCurPtr() + 10;
    //   int g = sprite_graphics[k];
    //   for (int i = 8; i >= 0; i--, oam++) {
    //     SetOamHelper0(oam, info.x + kMothula_Draw_X[g * 9 + i], info.y + 16,
    //                   0x6c, 0x24, 2);
    //   }
    // }
    pub(super) fn mothula_draw(&mut self, k: usize) {
        write_le_u16(&mut self.ram, OAM_CUR_PTR, 0x920);
        write_le_u16(&mut self.ram, OAM_EXT_CUR_PTR, 0xa68);
        let g = self.ram[SPRITE_GRAPHICS + k] as usize;
        let (info_x, info_y) = self.sprite_draw_multiple_for_mothula(k, g * 8, 8);
        if self.ram[SPRITE_PAUSE + k] != 0 {
            return;
        }
        let info_y = info_y.wrapping_add(self.ram[SPRITE_Z + k] as u16);
        // oam = current oam ptr + 10 entries (each OamEnt is 4 bytes
        // in the small region pointed to by oam_cur_ptr in the C port).
        let oam_base = read_le_u16(&self.ram, OAM_CUR_PTR) as usize;
        // Iterate i = 8..=0 (inclusive). The C `oam++` advances one
        // OamEnt per iteration; the small region uses 4 bytes/entry.
        for step in 0..=8 {
            let i = 8 - step;
            let oam = oam_base + (10 + step) * 4;
            let x = info_x.wrapping_add(K_MOTHULA_DRAW_X[g * 9 + i] as i16 as u16);
            let y = info_y.wrapping_add(16);
            self.set_oam_helper0_for_mothula(oam, x, y, 0x6c, 0x24, 2);
        }
    }

    // void Mothula_Main(int k) {  // 9ebe88
    //   int j;
    //   Mothula_Draw(k);
    //   if (sprite_state[k] == 11)
    //     sprite_ai_state[k] = 0;
    //   if (Sprite_ReturnIfInactive(k))
    //     return;
    //   sprite_flags3[k] = 0;
    //   if (sprite_delay_aux3[k])
    //     sprite_flags3[k] = 64;
    //   if ((sprite_F[k] & 127) == 6) {
    //     sprite_F[k] = 0;
    //     sprite_delay_aux3[k] = 32;
    //     sprite_ai_state[k] = 2;
    //     sprite_delay_main[k] = 0;
    //     sprite_G[k] = 64;
    //   }
    //   if (Sprite_ReturnIfRecoiling(k))
    //     return;
    //   switch(sprite_ai_state[k]) {
    //   case 0: // Delay
    //     if (!sprite_delay_main[k])
    //       sprite_ai_state[k] = 1;
    //     break;
    //   case 1: // Ascend
    //     sprite_z_vel[k] = 8;
    //     Sprite_MoveZ(k);
    //     sprite_z_vel[k] = 0;
    //     if (sprite_z[k] >= 24) {
    //       sprite_G[k] = 128;
    //       sprite_ai_state[k] = 2;
    //       sprite_ignore_projectile[k] = 0;
    //       sprite_delay_main[k] = 64;
    //     }
    //     Mothula_FlapWings(k);
    //     break;
    //   case 2: // FlyAbout
    //     if (!sprite_G[k]) {
    //       sprite_delay_main[k] = 63;
    //       sprite_ai_state[k] = 3;
    //       return;
    //     }
    //     sprite_G[k]--;
    //     Mothula_FlapWings(k);
    //     j = sprite_A[k] & 1;
    //     sprite_z_vel[k] += j ? -1 : 1;
    //     if (sprite_z_vel[k] == (uint8)(j ? -16 : 16))
    //       sprite_A[k]++;
    //     if (!sprite_delay_main[k]) {
    //       if (++sprite_C[k] == 7) {
    //         sprite_C[k] = 0;
    //         Sprite_ApplySpeedTowardsLink(k, 32);
    //         sprite_delay_main[k] = 128;
    //       } else {
    //         static const int8 kMothula_XYvel[10] = {-16, -12, 0, 12, 16, 12, 0, -12, -16, -12};
    //         j = GetRandomNumber() & 7;
    //         sprite_x_vel[k] = kMothula_XYvel[j + 2];
    //         sprite_y_vel[k] = kMothula_XYvel[j];
    //         sprite_delay_main[k] = (GetRandomNumber() & 31) + 64;
    //       }
    //     }
    //     if (!sprite_wallcoll[k])
    //       Sprite_MoveXY(k);
    //     Sprite_MoveZ(k);
    //     if (Sprite_CheckTileCollision(k))
    //       sprite_delay_main[k] = 0;
    //     Sprite_CheckDamageToAndFromLink(k);
    //     sprite_subtype2[k] += 2;
    //     break;
    //   case 3: // FireBeams
    //     Sprite_CheckDamageToAndFromLink(k);
    //     if (!sprite_delay_main[k]) {
    //       sprite_ai_state[k]--;
    //       sprite_G[k] = GetRandomNumber() & 31 | 64;
    //     } else {
    //       if (sprite_delay_main[k] == 0x20)
    //         Mothula_SpawnBeams(k);
    //       Mothula_FlapWings(k);
    //     }
    //     break;
    //   }
    // }
    pub(super) fn mothula_main(&mut self, k: usize) {
        self.mothula_draw(k);
        if self.ram[SPRITE_STATE + k] == 11 {
            self.ram[SPRITE_AI_STATE + k] = 0;
        }
        if self.sprite_return_if_inactive_for_mothula(k) {
            return;
        }
        self.ram[SPRITE_FLAGS3 + k] = 0;
        if self.ram[SPRITE_DELAY_AUX3_MOTHULA + k] != 0 {
            self.ram[SPRITE_FLAGS3 + k] = 64;
        }
        if (self.ram[SPRITE_F + k] & 127) == 6 {
            self.ram[SPRITE_F + k] = 0;
            self.ram[SPRITE_DELAY_AUX3_MOTHULA + k] = 32;
            self.ram[SPRITE_AI_STATE + k] = 2;
            self.ram[SPRITE_DELAY_MAIN + k] = 0;
            self.ram[SPRITE_G + k] = 64;
        }
        if self.sprite_return_if_recoiling_for_mothula(k) {
            return;
        }
        match self.ram[SPRITE_AI_STATE + k] {
            0 => {
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                }
            }
            1 => {
                self.ram[SPRITE_Z_VEL + k] = 8;
                self.sprite_move_z_for_mothula(k);
                self.ram[SPRITE_Z_VEL + k] = 0;
                if self.ram[SPRITE_Z + k] >= 24 {
                    self.ram[SPRITE_G + k] = 128;
                    self.ram[SPRITE_AI_STATE + k] = 2;
                    self.ram[SPRITE_IGNORE_PROJECTILE + k] = 0;
                    self.ram[SPRITE_DELAY_MAIN + k] = 64;
                }
                self.mothula_flap_wings(k);
            }
            2 => {
                if self.ram[SPRITE_G + k] == 0 {
                    self.ram[SPRITE_DELAY_MAIN + k] = 63;
                    self.ram[SPRITE_AI_STATE + k] = 3;
                    return;
                }
                self.ram[SPRITE_G + k] = self.ram[SPRITE_G + k].wrapping_sub(1);
                self.mothula_flap_wings(k);
                let j = (self.ram[SPRITE_A + k] & 1) as usize;
                // sprite_z_vel[k] += j ? -1 : 1
                let delta: i8 = if j != 0 { -1 } else { 1 };
                self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_add(delta as u8);
                // limit = (uint8)(j ? -16 : 16)
                let limit: u8 = if j != 0 { (-16i8) as u8 } else { 16 };
                if self.ram[SPRITE_Z_VEL + k] == limit {
                    self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_add(1);
                }
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_C + k] = self.ram[SPRITE_C + k].wrapping_add(1);
                    if self.ram[SPRITE_C + k] == 7 {
                        self.ram[SPRITE_C + k] = 0;
                        self.sprite_apply_speed_towards_link_for_mothula(k, 32);
                        self.ram[SPRITE_DELAY_MAIN + k] = 128;
                    } else {
                        let j2 = (self.get_random_number() & 7) as usize;
                        self.ram[SPRITE_X_VEL + k] = K_MOTHULA_XYVEL[j2 + 2] as u8;
                        self.ram[SPRITE_Y_VEL + k] = K_MOTHULA_XYVEL[j2] as u8;
                        self.ram[SPRITE_DELAY_MAIN + k] =
                            (self.get_random_number() & 31).wrapping_add(64);
                    }
                }
                if self.ram[SPRITE_WALLCOLL + k] == 0 {
                    self.sprite_move_xy_for_mothula(k);
                }
                self.sprite_move_z_for_mothula(k);
                if self.sprite_check_tile_collision_for_mothula(k) {
                    self.ram[SPRITE_DELAY_MAIN + k] = 0;
                }
                self.sprite_check_damage_to_and_from_link_for_mothula(k);
                self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(2);
            }
            3 => {
                self.sprite_check_damage_to_and_from_link_for_mothula(k);
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_AI_STATE + k] = self.ram[SPRITE_AI_STATE + k].wrapping_sub(1);
                    self.ram[SPRITE_G + k] = (self.get_random_number() & 31) | 64;
                } else {
                    if self.ram[SPRITE_DELAY_MAIN + k] == 0x20 {
                        self.mothula_spawn_beams(k);
                    }
                    self.mothula_flap_wings(k);
                }
            }
            _ => {}
        }
    }

    // void Mothula_FlapWings(int k) {  // 9ebf9f
    //   static const uint8 kMothula_FlapWingsGfx[4] = {0, 1, 2, 1};
    //   int j = ++sprite_subtype2[k] >> 2 & 3;
    //   if (j == 0)
    //     SpriteSfx_QueueSfx3WithPan(k, 0x2);
    //   sprite_graphics[k] = kMothula_FlapWingsGfx[j];
    // }
    pub(super) fn mothula_flap_wings(&mut self, k: usize) {
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        let j = ((self.ram[SPRITE_SUBTYPE2 + k] >> 2) & 3) as usize;
        if j == 0 {
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x2);
        }
        self.ram[SPRITE_GRAPHICS + k] = K_MOTHULA_FLAP_WINGS_GFX[j];
    }

    // void Mothula_SpawnBeams(int k) {  // 9ebfdf
    //   static const int8 kMothula_Beam_Xvel[3] = {-16, 0, 16};
    //   static const int8 kMothula_Beam_Yvel[3] = {24, 32, 24};
    //   SpriteSfx_QueueSfx3WithPan(k, 0x36);
    //   for (int i = 2; i >= 0; i--) {
    //     SpriteSpawnInfo info;
    //     int j = Sprite_SpawnDynamically(k, 0x89, &info);
    //     if (j >= 0) {
    //       Sprite_SetSpawnedCoordinates(j, &info);
    //       sprite_y_lo[j] = info.r2_y - info.r4_z + 3;
    //       sprite_delay_main[j] = 16;
    //       sprite_ignore_projectile[j] = 16;
    //       sprite_x_lo[j] = info.r0_x + kMothula_Beam_Xvel[i];
    //       sprite_x_vel[j] = kMothula_Beam_Xvel[i];
    //       sprite_y_vel[j] = kMothula_Beam_Yvel[i];
    //       sprite_z[j] = 0;
    //     }
    //   }
    //   tmp_counter = 0xff;
    // }
    pub(super) fn mothula_spawn_beams(&mut self, k: usize) {
        self.sprite_sfx_queue_sfx3_with_pan(k, 0x36);
        for i in (0..=2usize).rev() {
            if let Some((j, r0_x, r2_y, r4_z)) = self.sprite_spawn_dynamically_for_mothula(k, 0x89)
            {
                self.sprite_set_spawned_coordinates_for_mothula(j, r0_x, r2_y);
                self.ram[SPRITE_Y_LO + j] = (r2_y as u8).wrapping_sub(r4_z as u8).wrapping_add(3);
                self.ram[SPRITE_DELAY_MAIN + j] = 16;
                self.ram[SPRITE_IGNORE_PROJECTILE + j] = 16;
                self.ram[SPRITE_X_LO + j] = (r0_x as u8).wrapping_add(K_MOTHULA_BEAM_XVEL[i] as u8);
                self.ram[SPRITE_X_VEL + j] = K_MOTHULA_BEAM_XVEL[i] as u8;
                self.ram[SPRITE_Y_VEL + j] = K_MOTHULA_BEAM_YVEL[i] as u8;
                self.ram[SPRITE_Z + j] = 0;
            }
        }
        self.ram[TMP_COUNTER_MOTHULA] = 0xff;
    }

    // void Mothula_HandleSpikes(int k) {  // 9ec088
    //   static const uint8 kMothula_Spike_XLo[30] = { ... };
    //   static const uint8 kMothula_Spike_YLo[30] = { ... };
    //   static const uint8 kMothula_Spike_Dir[30] = { ... };
    //
    //   if (--sprite_head_dir[k])
    //     return;
    //   sprite_head_dir[k] = 0x40;
    //   SpriteSpawnInfo info;
    //   int j = Sprite_SpawnDynamically(k, 0x8a, &info);
    //   if (j < 0)
    //     return;
    //   int i = GetRandomNumber() & 0x1f;
    //   if (i >= 30) i -= 30;
    //   sprite_A[j] = sprite_x_lo[j] = kMothula_Spike_XLo[i];
    //   sprite_B[j] = sprite_y_lo[j] = kMothula_Spike_YLo[i] - 1;
    //   sprite_D[j] = kMothula_Spike_Dir[i];
    //   sprite_E[j] = 1;
    //   sprite_x_hi[j] = SPRITE_ROOM_ORIGIN_X_HI + 1;
    //   sprite_y_hi[j] = SPRITE_ROOM_ORIGIN_Y_HI + 1;
    //   sprite_x_vel[j] = 1;
    //   Sprite_Get16BitCoords(j);
    //   Sprite_CheckTileCollision(j);
    //   sprite_x_vel[j] = 0;
    //   sprite_x_lo[j] = sprite_A[j];
    //   sprite_y_lo[j] = sprite_B[j];
    //   if (!sprite_wallcoll[j]) {
    //     sprite_state[j] = 0;
    //     sprite_head_dir[k] = 1;
    //   }
    // }
    pub(super) fn mothula_handle_spikes(&mut self, k: usize) {
        self.ram[SPRITE_HEAD_DIR + k] = self.ram[SPRITE_HEAD_DIR + k].wrapping_sub(1);
        if self.ram[SPRITE_HEAD_DIR + k] != 0 {
            return;
        }
        self.ram[SPRITE_HEAD_DIR + k] = 0x40;
        let Some((j, _r0_x, _r2_y, _r4_z)) = self.sprite_spawn_dynamically_for_mothula(k, 0x8a)
        else {
            return;
        };
        let mut i = (self.get_random_number() & 0x1f) as usize;
        if i >= 30 {
            i -= 30;
        }
        self.ram[SPRITE_A + j] = K_MOTHULA_SPIKE_XLO[i];
        self.ram[SPRITE_X_LO + j] = K_MOTHULA_SPIKE_XLO[i];
        self.ram[SPRITE_B + j] = K_MOTHULA_SPIKE_YLO[i].wrapping_sub(1);
        self.ram[SPRITE_Y_LO + j] = K_MOTHULA_SPIKE_YLO[i].wrapping_sub(1);
        self.ram[SPRITE_D + j] = K_MOTHULA_SPIKE_DIR[i];
        self.ram[SPRITE_E + j] = 1;
        self.ram[SPRITE_X_HI + j] = self.ram[SPRITE_ROOM_ORIGIN_X_HI].wrapping_add(1);
        self.ram[SPRITE_Y_HI + j] = self.ram[SPRITE_ROOM_ORIGIN_Y_HI].wrapping_add(1);
        self.ram[SPRITE_X_VEL + j] = 1;
        self.sprite_get_16bit_coords_for_mothula(j);
        self.sprite_check_tile_collision_for_mothula(j);
        self.ram[SPRITE_X_VEL + j] = 0;
        self.ram[SPRITE_X_LO + j] = self.ram[SPRITE_A + j];
        self.ram[SPRITE_Y_LO + j] = self.ram[SPRITE_B + j];
        if self.ram[SPRITE_WALLCOLL + j] == 0 {
            self.ram[SPRITE_STATE + j] = 0;
            self.ram[SPRITE_HEAD_DIR + k] = 1;
        }
    }

    // void Sprite_53_ArmosKnight(int k) {  // 85a036
    //   static const uint8 kArmosKnight_Gfx1[5] = {5, 4, 3, 2, 1};
    //   static const int8 kArmosKnight_Xv[2] = {16, -16};
    //
    //   sprite_obj_prio[k] |= 0x30;
    //   ArmosKnight_Draw(k);
    //   if (Sprite_ReturnIfPaused(k))
    //     return;
    //   if (sprite_state[k] != 9) {
    //     if (sprite_delay_main[k]) {
    //       sprite_graphics[k] = kArmosKnight_Gfx1[sprite_delay_main[k] >> 3];
    //       return;
    //     }
    //     if (--SPRITE_PREP_SHARED_COUNTER == 1) {
    //       for (int j = 5; j >= 0; j--) {
    //         sprite_health[j] = 48;
    //         sprite_x_vel[j] = sprite_y_vel[j] = sprite_z_vel[j] = 0;
    //       }
    //     }
    //     sprite_state[k] = 0;
    //     if (Sprite_CheckIfScreenIsClear()) {
    //       SpriteSpawnInfo info;
    //       int j = Sprite_SpawnDynamically(k, 0xea, &info);
    //       assert(j >= 0);
    //       Sprite_SetSpawnedCoordinates(j, &info);
    //       sprite_z_vel[j] = 32;
    //       sprite_A[j] = 1;
    //     }
    //     return;
    //   }
    //   Sprite_MoveXY(k);
    //   Sprite_MoveZ(k);
    //   sprite_z_vel[k] -= 4;
    //   if (sign8(sprite_z[k])) {
    //     sprite_z_vel[k] = 0;
    //     sprite_z[k] = 0;
    //     if (SPRITE_PREP_SHARED_COUNTER != 1 && sprite_A[k]) {
    //       sprite_z_vel[k] = 48;
    //       SpriteSfx_QueueSfx3WithPan(k, 0x16);
    //     }
    //   }
    //   if (sprite_F[k]) {
    //     Sprite_ZeroVelocity_XY(k);
    //     sprite_ai_state[k] = 0;
    //     sprite_G[k] = 0;
    //   }
    //   if (Sprite_ReturnIfRecoiling(k))
    //     return;
    //   if (!sprite_A[k]) {
    //     if (!sprite_delay_main[k]) {
    //       sprite_A[k]++;
    //       sprite_flags2[k] = (sprite_flags2[k] & 0x7f) - 2;
    //       sprite_defl_bits[k] &= ~4;
    //       sprite_flags3[k] &= ~0x40;
    //     } else {
    //       if (sprite_delay_main[k] == 64) {
    //         sound_effect_1 = 0x35;
    //       } else if (sprite_delay_main[k] < 64) {
    //         int j = ((sprite_delay_main[k] >> 1) ^ k) & 1;
    //         sprite_x_vel[k] = kArmosKnight_Xv[j];
    //         Sprite_MoveX(k);
    //         sprite_x_vel[k] = 0;
    //       }
    //       Sprite_CheckDamageFromLink(k);
    //       if (Sprite_CheckDamageToLink_same_layer(k)) {
    //         Sprite_NullifyHookshotDrag();
    //         Sprite_RepelDash();
    //       }
    //     }
    //   } else if (SPRITE_PREP_SHARED_COUNTER == 1) {
    //     Sprite_ArmosCrusher(k);
    //   } else {
    //     Sprite_CheckDamageToAndFromLink(k);
    //     if (!sprite_ai_state[k]) {
    //       uint16 x = overlord_y_hi[k] << 8 | overlord_x_hi[k];
    //       uint16 y = overlord_floor[k] << 8 | overlord_gen2[k];
    //       ProjectSpeedRet pt = Sprite_ProjectSpeedTowardsLocation(k, x, y, 16);
    //       sprite_x_vel[k] = pt.x;
    //       sprite_y_vel[k] = pt.y;
    //       Sprite_Get16BitCoords(k);
    //       if ((uint16)(x - cur_sprite_x + 2) < 4 && (uint16)(y - cur_sprite_y + 2) < 4)
    //         sprite_ai_state[k] = 1;
    //     } else {
    //       sprite_x_lo[k] = overlord_x_hi[k];
    //       sprite_x_hi[k] = overlord_y_hi[k];
    //       sprite_y_lo[k] = overlord_gen2[k];
    //       sprite_y_hi[k] = overlord_floor[k];
    //     }
    //   }
    // }
    pub(super) fn sprite_53_armos_knight(&mut self, k: usize) {
        self.ram[SPRITE_OBJ_PRIO + k] |= 0x30;
        self.armos_knight_draw(k);
        if self.sprite_return_if_paused(k) {
            return;
        }
        if self.ram[SPRITE_STATE + k] != 9 {
            if self.ram[SPRITE_DELAY_MAIN + k] != 0 {
                self.ram[SPRITE_GRAPHICS + k] =
                    K_ARMOS_KNIGHT_GFX1[(self.ram[SPRITE_DELAY_MAIN + k] >> 3) as usize];
                return;
            }
            self.ram[ARMOS_KNIGHT_REMAINING_COUNT] =
                self.ram[ARMOS_KNIGHT_REMAINING_COUNT].wrapping_sub(1);
            if self.ram[ARMOS_KNIGHT_REMAINING_COUNT] == 1 {
                for j in (0..=5usize).rev() {
                    self.ram[SPRITE_HEALTH + j] = 48;
                    self.ram[SPRITE_X_VEL + j] = 0;
                    self.ram[SPRITE_Y_VEL + j] = 0;
                    self.ram[SPRITE_Z_VEL + j] = 0;
                }
            }
            self.ram[SPRITE_STATE + k] = 0;
            if self.sprite_check_if_screen_is_clear() {
                let mut info = SpriteSpawnInfo::default();
                let j = self.sprite_spawn_dynamically(k, 0xea, &mut info);
                if j >= 0 {
                    let ju = j as usize;
                    self.sprite_set_spawned_coordinates(ju, &info);
                    self.ram[SPRITE_Z_VEL + ju] = 32;
                    self.ram[SPRITE_A + ju] = 1;
                }
            }
            return;
        }

        self.sprite_move_xy(k);
        self.sprite_move_z(k);
        self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_sub(4);
        if sign8(self.ram[SPRITE_Z + k]) {
            self.ram[SPRITE_Z_VEL + k] = 0;
            self.ram[SPRITE_Z + k] = 0;
            if self.ram[ARMOS_KNIGHT_REMAINING_COUNT] != 1 && self.ram[SPRITE_A + k] != 0 {
                self.ram[SPRITE_Z_VEL + k] = 48;
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x16);
            }
        }
        if self.ram[SPRITE_F + k] != 0 {
            self.sprite_zero_velocity_xy(k);
            self.ram[SPRITE_AI_STATE + k] = 0;
            self.ram[SPRITE_G + k] = 0;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }

        if self.ram[SPRITE_A + k] == 0 {
            if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                self.ram[SPRITE_A + k] = self.ram[SPRITE_A + k].wrapping_add(1);
                self.ram[SPRITE_FLAGS2 + k] = (self.ram[SPRITE_FLAGS2 + k] & 0x7f).wrapping_sub(2);
                self.ram[SPRITE_DEFL_BITS + k] &= !4;
                self.ram[SPRITE_FLAGS3 + k] &= !0x40;
            } else {
                if self.ram[SPRITE_DELAY_MAIN + k] == 64 {
                    self.ram[SOUND_EFFECT_1] = 0x35;
                } else if self.ram[SPRITE_DELAY_MAIN + k] < 64 {
                    let j = (((self.ram[SPRITE_DELAY_MAIN + k] >> 1) ^ k as u8) & 1) as usize;
                    self.ram[SPRITE_X_VEL + k] = K_ARMOS_KNIGHT_XV[j] as u8;
                    self.sprite_move_x(k);
                    self.ram[SPRITE_X_VEL + k] = 0;
                }
                self.sprite_check_damage_from_link(k);
                if self.sprite_check_damage_to_link_same_layer(k) {
                    self.sprite_nullify_hookshot_drag();
                    self.sprite_repel_dash();
                }
            }
        } else if self.ram[ARMOS_KNIGHT_REMAINING_COUNT] == 1 {
            self.sprite_armos_crusher(k);
        } else {
            self.sprite_check_damage_to_and_from_link(k);
            if self.ram[SPRITE_AI_STATE + k] == 0 {
                let x = u16::from(self.ram[OVERLORD_X_HI_MOTHULA + k])
                    | (u16::from(self.ram[OVERLORD_Y_HI_MOTHULA + k]) << 8);
                let y = u16::from(self.ram[OVERLORD_GEN2_MOTHULA + k])
                    | (u16::from(self.ram[OVERLORD_FLOOR_MOTHULA + k]) << 8);
                let pt = self.sprite_project_speed_towards_location(k, x, y, 16);
                self.ram[SPRITE_X_VEL + k] = pt.x;
                self.ram[SPRITE_Y_VEL + k] = pt.y;
                let cur_x = u16::from(self.ram[SPRITE_X_LO + k])
                    | (u16::from(self.ram[SPRITE_X_HI + k]) << 8);
                let cur_y = u16::from(self.ram[SPRITE_Y_LO + k])
                    | (u16::from(self.ram[SPRITE_Y_HI + k]) << 8);
                write_le_u16(&mut self.ram, CUR_SPRITE_X, cur_x);
                write_le_u16(&mut self.ram, CUR_SPRITE_Y, cur_y);
                if x.wrapping_sub(cur_x).wrapping_add(2) < 4
                    && y.wrapping_sub(cur_y).wrapping_add(2) < 4
                {
                    self.ram[SPRITE_AI_STATE + k] = 1;
                }
            } else {
                self.ram[SPRITE_X_LO + k] = self.ram[OVERLORD_X_HI_MOTHULA + k];
                self.ram[SPRITE_X_HI + k] = self.ram[OVERLORD_Y_HI_MOTHULA + k];
                self.ram[SPRITE_Y_LO + k] = self.ram[OVERLORD_GEN2_MOTHULA + k];
                self.ram[SPRITE_Y_HI + k] = self.ram[OVERLORD_FLOOR_MOTHULA + k];
            }
        }
    }

    // void GiantMoldorm_IncrementalSegmentExplosion(int k) {  // 9dd8f2
    //   if (sprite_state[k] == 9 && sprite_delay_aux4[k] && sprite_delay_aux4[k] < 80 &&
    //       !(sprite_delay_aux4[k] & 15 | submodule_index | flag_unk1)) {
    //     sprite_B[k]++;
    //     Sprite_MakeBossExplosion(k);
    //   }
    // }
    pub(super) fn giant_moldorm_incremental_segment_explosion(&mut self, k: usize) {
        let aux4 = self.ram[SPRITE_DELAY_AUX4 + k];
        if self.ram[SPRITE_STATE + k] == 9
            && aux4 != 0
            && aux4 < 80
            && ((aux4 & 15) | self.frame_control_view().submodule() | self.ram[FLAG_UNK1]) == 0
        {
            self.ram[SPRITE_B + k] = self.ram[SPRITE_B + k].wrapping_add(1);
            self.sprite_make_boss_explosion(k);
        }
    }

    // void Sprite_ArmosCrusher(int k) {  // 9def7e
    //   sprite_oam_flags[k] = 7;
    //   bg1_y_offset = sprite_delay_aux4[k] ? (sprite_delay_aux4[k] & 1 ? -1 : 1) : 0;
    //   switch (sprite_G[k]) {
    //   case 0:
    //     Sprite_CheckDamageToAndFromLink(k);
    //     if (!(sprite_delay_main[k] | sprite_z[k])) {
    //       Sprite_ApplySpeedTowardsLink(k, 32);
    //       sprite_z_vel[k] = 32;
    //       sprite_G[k]++;
    //       sprite_B[k] = link_x_coord;
    //       sprite_C[k] = link_x_coord >> 8;
    //       sprite_E[k] = link_y_coord;
    //       sprite_head_dir[k] = link_y_coord >> 8;
    //       SpriteSfx_QueueSfx2WithPan(k, 0x20);
    //     }
    //     break;
    //   case 1:
    //     sprite_z_vel[k] += 3;
    //     if (Sprite_CheckTileCollision(k))
    //       goto advance;
    //     Sprite_Get16BitCoords(k);
    //     uint16 x, y;
    //     x = sprite_B[k] | sprite_C[k] << 8;
    //     y = sprite_E[k] | sprite_head_dir[k] << 8;
    //     if ((uint16)(x - cur_sprite_x + 16) < 32 && (uint16)(y - cur_sprite_y + 16) < 32) {
    // advance:
    //       sprite_G[k]++;
    //       sprite_delay_main[k] = 16;
    //       sprite_x_vel[k] = 0;
    //       sprite_y_vel[k] = 0;
    //     }
    //     break;
    //   case 2:
    //     sprite_z_vel[k] = 0;
    //     if (!sprite_delay_main[k])
    //       sprite_G[k]++;
    //     break;
    //   case 3:
    //     sprite_z_vel[k] = -104;
    //     if (!sign8(sprite_z[k])) {
    //       sprite_delay_main[k] = 32;
    //       sprite_delay_aux4[k] = 32;
    //       sprite_G[k] = 0;
    //       SpriteSfx_QueueSfx2WithPan(k, 0xc);
    //     }
    //     break;
    //   }
    // }
    pub(super) fn sprite_armos_crusher(&mut self, k: usize) {
        self.ram[SPRITE_OAM_FLAGS + k] = 7;
        let aux4 = self.ram[SPRITE_DELAY_AUX4 + k];
        let bg1_y = if aux4 == 0 {
            0
        } else if aux4 & 1 != 0 {
            0xffff
        } else {
            1
        };
        write_le_u16(&mut self.ram, BG1_Y_OFFSET, bg1_y);

        match self.ram[SPRITE_G + k] {
            0 => {
                self.sprite_check_damage_to_and_from_link(k);
                if (self.ram[SPRITE_DELAY_MAIN + k] | self.ram[SPRITE_Z + k]) == 0 {
                    self.sprite_apply_speed_towards_link(k, 32);
                    self.ram[SPRITE_Z_VEL + k] = 32;
                    self.ram[SPRITE_G + k] = self.ram[SPRITE_G + k].wrapping_add(1);
                    self.ram[SPRITE_B + k] = self.ram[LINK_X_COORD];
                    self.ram[SPRITE_C + k] = self.ram[LINK_X_COORD + 1];
                    self.ram[SPRITE_E + k] = self.ram[LINK_Y_COORD];
                    self.ram[SPRITE_HEAD_DIR + k] = self.ram[LINK_Y_COORD + 1];
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
                }
            }
            1 => {
                self.ram[SPRITE_Z_VEL + k] = self.ram[SPRITE_Z_VEL + k].wrapping_add(3);
                let mut advance = self.sprite_check_tile_collision(k) != 0;
                if !advance {
                    let cur_x = u16::from(self.ram[SPRITE_X_LO + k])
                        | (u16::from(self.ram[SPRITE_X_HI + k]) << 8);
                    let cur_y = u16::from(self.ram[SPRITE_Y_LO + k])
                        | (u16::from(self.ram[SPRITE_Y_HI + k]) << 8);
                    write_le_u16(&mut self.ram, CUR_SPRITE_X, cur_x);
                    write_le_u16(&mut self.ram, CUR_SPRITE_Y, cur_y);
                    let x = u16::from(self.ram[SPRITE_B + k])
                        | (u16::from(self.ram[SPRITE_C + k]) << 8);
                    let y = u16::from(self.ram[SPRITE_E + k])
                        | (u16::from(self.ram[SPRITE_HEAD_DIR + k]) << 8);
                    advance = x.wrapping_sub(cur_x).wrapping_add(16) < 32
                        && y.wrapping_sub(cur_y).wrapping_add(16) < 32;
                }
                if advance {
                    self.ram[SPRITE_G + k] = self.ram[SPRITE_G + k].wrapping_add(1);
                    self.ram[SPRITE_DELAY_MAIN + k] = 16;
                    self.ram[SPRITE_X_VEL + k] = 0;
                    self.ram[SPRITE_Y_VEL + k] = 0;
                }
            }
            2 => {
                self.ram[SPRITE_Z_VEL + k] = 0;
                if self.ram[SPRITE_DELAY_MAIN + k] == 0 {
                    self.ram[SPRITE_G + k] = self.ram[SPRITE_G + k].wrapping_add(1);
                }
            }
            3 => {
                self.ram[SPRITE_Z_VEL + k] = (-104i8) as u8;
                if !sign8(self.ram[SPRITE_Z + k]) {
                    self.ram[SPRITE_DELAY_MAIN + k] = 32;
                    self.ram[SPRITE_DELAY_AUX4 + k] = 32;
                    self.ram[SPRITE_G + k] = 0;
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x0c);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------
    // Minimal local helpers (named with `_for_mothula` suffix to avoid
    // colliding with future canonical Sprite_* helpers). These mirror
    // the data-mutating effects of the C helpers that Mothula reaches
    // for, but defer the heavy OAM/collision pipelines until those
    // canonical ports land.
    // -----------------------------------------------------------------

    fn sprite_return_if_inactive_for_mothula(&mut self, k: usize) -> bool {
        // Rewired to canonical Sprite_ReturnIfInactive port.
        self.sprite_return_if_inactive(k)
    }

    fn sprite_return_if_recoiling_for_mothula(&mut self, k: usize) -> bool {
        // Rewired to canonical Sprite_ReturnIfRecoiling port.
        self.sprite_return_if_recoiling(k)
    }

    fn sprite_draw_multiple_for_mothula(
        &mut self,
        k: usize,
        start: usize,
        count: usize,
    ) -> (u16, u16) {
        let Some(prepped) = self.sprite_prep_oam_coord_or_double_ret(k) else {
            return (0, 0);
        };
        let entries: Vec<DrawMultipleData> = K_MOTHULA_DMD[start..start + count]
            .iter()
            .map(|&(x, y, char_flags, ext)| DrawMultipleData {
                x,
                y,
                char_flags,
                ext,
            })
            .collect();
        self.sprite_draw_multiple_with_info(k, &entries, prepped);
        (prepped.0, prepped.1)
    }

    fn set_oam_helper0_for_mothula(
        &mut self,
        oam: usize,
        x: u16,
        y: u16,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.set_oam_helper0_at(oam, x, y, charnum, flags, big);
    }

    fn sprite_move_z_for_mothula(&mut self, k: usize) {
        // Rewired to canonical Sprite_MoveZ port.
        self.sprite_move_z(k);
    }

    fn sprite_move_xy_for_mothula(&mut self, k: usize) {
        // Rewired to canonical Sprite_MoveXY port.
        self.sprite_move_xy(k);
    }

    fn sprite_check_tile_collision_for_mothula(&mut self, k: usize) -> bool {
        // Rewired to canonical Sprite_CheckTileCollision port. The C helper
        // returns the wallcoll byte; Mothula keys off "any collision" via
        // a non-zero check.
        self.sprite_check_tile_collision(k) != 0
    }

    fn sprite_check_damage_to_and_from_link_for_mothula(&mut self, k: usize) {
        // Rewired to canonical Sprite_CheckDamageToAndFromLink port.
        self.sprite_check_damage_to_and_from_link(k);
    }

    fn sprite_apply_speed_towards_link_for_mothula(&mut self, k: usize, speed: u8) {
        // Rewired to canonical Sprite_ApplySpeedTowardsLink port.
        self.sprite_apply_speed_towards_link(k, speed);
    }

    fn sprite_spawn_dynamically_for_mothula(
        &mut self,
        k: usize,
        what: u8,
    ) -> Option<(usize, u16, u16, u16)> {
        // Rewired to canonical Sprite_SpawnDynamically port. The local
        // 4-tuple keeps mothula's existing call sites' destructuring shape;
        // the canonical helper populates r4_z directly from `sprite_z[k]`.
        let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, what, &mut info);
        if j < 0 {
            None
        } else {
            Some((j as usize, info.r0_x, info.r2_y, info.r4_z as u16))
        }
    }

    fn sprite_set_spawned_coordinates_for_mothula(&mut self, j: usize, r0_x: u16, r2_y: u16) {
        // Rewired to canonical Sprite_SetSpawnedCoordinates port.
        let info = crate::zelda_rtl::sprite::SpriteSpawnInfo {
            r0_x,
            r2_y,
            ..Default::default()
        };
        self.sprite_set_spawned_coordinates(j, &info);
    }

    fn sprite_get_16bit_coords_for_mothula(&mut self, j: usize) {
        // Rewired to canonical Sprite_Get16BitCoords port.
        self.sprite_get16_bit_coords(j);
    }

    fn dungeon_update_tile_map_with_common_tile_for_mothula(&mut self, x: u16, y: u16, v: u8) {
        // Rewired to canonical Dungeon_UpdateTileMapWithCommonTile port.
        self.Dungeon_UpdateTileMapWithCommonTile(x as i32, y as i32, v);
    }

    fn sprite_94_tile_animate(&mut self, k: usize) {
        self.ram[SPRITE_SUBTYPE2 + k] = self.ram[SPRITE_SUBTYPE2 + k].wrapping_add(1);
        self.ram[SPRITE_GRAPHICS + k] = (self.ram[SPRITE_SUBTYPE2 + k] >> 2) & 1;
        if (((k as u8) ^ self.ram[FRAME_COUNTER]) & 7) == 0 {
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x07);
        }
    }

    fn sprite_94_tile_break(&mut self, k: usize) {
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x1f);
        self.ram[SPRITE_STATE + k] = 6;
        self.ram[SPRITE_DELAY_MAIN + k] = 31;
        self.ram[SPRITE_TYPE + k] = 0xec;
        self.ram[SPRITE_HIT_TIMER + k] = 0;
        self.ram[SPRITE_C + k] = 0x80;
    }

    fn stalfos_knight_set_to_ground(&mut self, k: usize) {
        self.ram[SPRITE_AI_STATE + k] = 2;
        self.ram[SPRITE_IGNORE_PROJECTILE + k] = 0;
        self.ram[SPRITE_Z + k] = 0;
        self.ram[SPRITE_Z_VEL + k] = 0;
        self.ram[SPRITE_DELAY_MAIN + k] = 63;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_state() -> ZeldaState {
        ZeldaState::new()
    }

    fn make_active(state: &mut ZeldaState, k: usize) {
        // Configure the slot so `sprite_return_if_inactive_for_mothula`
        // returns false: state=9, no flags, no pause via defl bit 0x80.
        state.ram[SPRITE_STATE + k] = 9;
        state.ram[FLAG_UNK1] = 0;
        state.ram[SUBMODULE_INDEX] = 0;
        state.ram[SPRITE_DEFL_BITS + k] = 0x80;
        state.ram[SPRITE_PAUSE + k] = 0;
        state.ram[SPRITE_HIT_TIMER + k] = 0;
    }

    #[test]
    fn flap_wings_advances_subtype2_and_picks_gfx() {
        // Mothula_FlapWings: pre-increments sprite_subtype2[k], picks
        // gfx from kMothula_FlapWingsGfx[(subtype2 >> 2) & 3].
        let mut s = fresh_state();
        let k = 3;

        // subtype2 starts at 0 -> after ++ = 1, j = (1 >> 2) & 3 = 0
        // -> sfx queued, gfx = kMothula_FlapWingsGfx[0] = 0.
        s.ram[SPRITE_SUBTYPE2 + k] = 0;
        s.mothula_flap_wings(k);
        assert_eq!(s.ram[SPRITE_SUBTYPE2 + k], 1);
        assert_eq!(s.ram[SPRITE_GRAPHICS + k], 0);

        // subtype2 = 3 -> ++ = 4 -> j = 1 -> gfx = 1.
        s.ram[SPRITE_SUBTYPE2 + k] = 3;
        s.mothula_flap_wings(k);
        assert_eq!(s.ram[SPRITE_SUBTYPE2 + k], 4);
        assert_eq!(s.ram[SPRITE_GRAPHICS + k], 1);

        // subtype2 = 7 -> ++ = 8 -> j = 2 -> gfx = 2.
        s.ram[SPRITE_SUBTYPE2 + k] = 7;
        s.mothula_flap_wings(k);
        assert_eq!(s.ram[SPRITE_GRAPHICS + k], 2);

        // subtype2 = 11 -> ++ = 12 -> j = 3 -> gfx = 1.
        s.ram[SPRITE_SUBTYPE2 + k] = 11;
        s.mothula_flap_wings(k);
        assert_eq!(s.ram[SPRITE_GRAPHICS + k], 1);
    }

    #[test]
    fn spawn_beams_sets_tmp_counter_and_beam_state() {
        // Mothula_SpawnBeams writes 0xff into tmp_counter and, for each
        // spawned slot, populates the per-beam velocity/x_lo/z/delay
        // fields. With an empty pool of sprite slots, three slots are
        // taken from the back of the array.
        let mut s = fresh_state();
        let k = 0;
        // Canonical Sprite_SpawnDynamically reads info.r0_x from
        // sprite_x_lo[k] | sprite_x_hi[k] << 8 (via Sprite_GetX). Seed the
        // sprite's per-slot position so r0_x / r2_y land at 0x80 / 0x50.
        s.ram[SPRITE_X_LO + k] = 0x80;
        s.ram[SPRITE_X_HI + k] = 0x00;
        s.ram[SPRITE_Y_LO + k] = 0x50;
        s.ram[SPRITE_Y_HI + k] = 0x00;
        s.ram[SPRITE_Z + k] = 0;

        s.mothula_spawn_beams(k);
        assert_eq!(s.ram[TMP_COUNTER_MOTHULA], 0xff);

        // The reverse-allocator hands out 15, 14, 13 across the loop.
        // First iter (i = 2): x_vel = 16, y_vel = 24, x_lo = 0x80 + 16.
        assert_eq!(s.ram[SPRITE_X_VEL + 15] as i8, 16);
        assert_eq!(s.ram[SPRITE_Y_VEL + 15], 24);
        assert_eq!(s.ram[SPRITE_X_LO + 15], 0x80u8.wrapping_add(16));
        assert_eq!(s.ram[SPRITE_DELAY_MAIN + 15], 16);
        assert_eq!(s.ram[SPRITE_IGNORE_PROJECTILE + 15], 16);
        assert_eq!(s.ram[SPRITE_Z + 15], 0);
        // Second iter (i = 1): x_vel = 0, y_vel = 32.
        assert_eq!(s.ram[SPRITE_X_VEL + 14], 0);
        assert_eq!(s.ram[SPRITE_Y_VEL + 14], 32);
        // Third iter (i = 0): x_vel = -16 (=> 0xf0), y_vel = 24.
        assert_eq!(s.ram[SPRITE_X_VEL + 13] as i8, -16);
        assert_eq!(s.ram[SPRITE_Y_VEL + 13], 24);
    }

    #[test]
    fn main_transitions_delay_to_ascend() {
        // ai_state = 0, sprite_delay_main = 0 -> ai_state becomes 1.
        let mut s = fresh_state();
        let k = 4;
        make_active(&mut s, k);
        s.ram[SPRITE_AI_STATE + k] = 0;
        s.ram[SPRITE_DELAY_MAIN + k] = 0;
        s.ram[SPRITE_F + k] = 0;
        s.mothula_main(k);
        assert_eq!(s.ram[SPRITE_AI_STATE + k], 1);
        assert_eq!(s.ram[SPRITE_FLAGS3 + k], 0);
    }

    #[test]
    fn main_flag_f6_arms_phase2() {
        // sprite_F & 127 == 6 forces F=0, delay_aux3=32, ai_state=2,
        // delay_main=0, G=64. After that the case-2 branch fires
        // (because we are mid-call): G is decremented to 63, the
        // flap-wings + z-vel maths run, and since delay_main==0 the
        // "++C == 7" else-branch overwrites x_vel/y_vel/delay_main.
        let mut s = fresh_state();
        let k = 2;
        make_active(&mut s, k);
        s.ram[SPRITE_F + k] = 6;
        s.ram[SPRITE_AI_STATE + k] = 3;
        s.ram[SPRITE_DELAY_MAIN + k] = 5;
        s.ram[SPRITE_G + k] = 0;
        s.mothula_main(k);
        assert_eq!(s.ram[SPRITE_F + k], 0);
        assert_eq!(s.ram[SPRITE_DELAY_AUX3_MOTHULA + k], 32);
        assert_eq!(s.ram[SPRITE_AI_STATE + k], 2);
        // case 2 ran: G decremented from 64 to 63.
        assert_eq!(s.ram[SPRITE_G + k], 63);
        // sprite_flags3 is evaluated BEFORE the F=6 branch sets
        // delay_aux3, so it stays 0 here.
        assert_eq!(s.ram[SPRITE_FLAGS3 + k], 0);
    }

    #[test]
    fn main_state11_resets_ai_state() {
        // sprite_state[k] == 11 forces ai_state -> 0, then the active
        // check trips on state != 9 and exits early.
        let mut s = fresh_state();
        let k = 5;
        s.ram[SPRITE_STATE + k] = 11;
        s.ram[SPRITE_AI_STATE + k] = 3;
        s.mothula_main(k);
        assert_eq!(s.ram[SPRITE_AI_STATE + k], 0);
        // Inactive exit means flags3 wasn't reset.
        assert_eq!(s.ram[SPRITE_FLAGS3 + k], 0);
    }

    #[test]
    fn handle_spikes_decrements_and_returns_early() {
        // First call: sprite_head_dir is decremented; non-zero -> early
        // return (no allocation occurs).
        let mut s = fresh_state();
        let k = 1;
        s.ram[SPRITE_HEAD_DIR + k] = 3;
        // Pre-fill a slot so we can prove no allocation happened.
        for j in 0..16 {
            s.ram[SPRITE_STATE + j] = 9;
        }
        s.mothula_handle_spikes(k);
        assert_eq!(s.ram[SPRITE_HEAD_DIR + k], 2);
        // No slot freed -> SPAWN cannot succeed even if it tried.
        for j in 0..16 {
            assert_eq!(s.ram[SPRITE_STATE + j], 9);
        }
    }

    #[test]
    fn handle_spikes_arms_when_decrement_hits_zero() {
        // sprite_head_dir = 1 -> after decrement = 0 -> reload to 0x40
        // and try to spawn. Spawn succeeds (one slot free); spike
        // tables populate the target slot.
        let mut s = fresh_state();
        let k = 0;
        s.ram[SPRITE_HEAD_DIR + k] = 1;
        // Mark all slots active except 15, so allocator picks 15.
        for j in 0..15 {
            s.ram[SPRITE_STATE + j] = 9;
        }
        s.ram[SPRITE_STATE + 15] = 0;
        // Force the random number deterministically by seeding RNG via
        // calling get_random_number isn't an option here, but the
        // table lookup with whatever index is produced just needs to be
        // valid; we assert side-effects independent of index.
        s.ram[SPRITE_ROOM_ORIGIN_X_HI] = 0x10;
        s.ram[SPRITE_ROOM_ORIGIN_Y_HI] = 0x20;
        write_le_u16(&mut s.ram, SPRCOLL_X_SIZE, 0xffff);
        write_le_u16(&mut s.ram, SPRCOLL_Y_SIZE, 0xffff);

        s.mothula_handle_spikes(k);
        // head_dir was set to 0x40 before the spawn path. If the spawn
        // succeeded and tile-collision found no wall, head_dir is reset
        // to 1; this fixture leaves the spawned spike away from walls, so
        // wallcoll stays 0 and the final branch fires:
        assert_eq!(s.ram[SPRITE_HEAD_DIR + k], 1);
        // Slot 15 was claimed and then re-zeroed by the wallcoll==0
        // branch. So sprite_state[15] == 0 again.
        assert_eq!(s.ram[SPRITE_STATE + 15], 0);
        // x_hi / y_hi reflect the current sprite-room origin plus 1.
        assert_eq!(s.ram[SPRITE_X_HI + 15], 0x11);
        assert_eq!(s.ram[SPRITE_Y_HI + 15], 0x21);
        // x_vel was zeroed after the collision check.
        assert_eq!(s.ram[SPRITE_X_VEL + 15], 0);
    }
}
