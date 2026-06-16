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
const ARMOS_KNIGHT_REMAINING_COUNT: usize = 0x0ff8;
const GARNISH_ACTIVE_MOTHULA: usize = 0x0fb4;
const GARNISH_Y_LO_MOTHULA: usize = 0x1f81e;
const GARNISH_X_LO_MOTHULA: usize = 0x1f83c;
const GARNISH_Y_HI_MOTHULA: usize = 0x1f85a;
const GARNISH_X_HI_MOTHULA: usize = 0x1f878;
const GARNISH_COUNTDOWN_MOTHULA: usize = 0x1f90e;
const GARNISH_SPRITE_MOTHULA: usize = 0x1f92c;
const GARNISH_FLOOR_MOTHULA: usize = 0x1f968;
const FEATURE_MISC_BUG_FIXES_MOTHULA: u32 = 4096;

// kMothula_Dmd from sprite_main.c:13776 — packed as (x:i8, y:i8, char:u16, big:u8).
type MothulaDrawFrame = (i8, i8, u16, u8);
const MOTHULA_DRAW_FRAMES: [MothulaDrawFrame; 24] = [
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
const MOTHULA_DRAW_X_OFFSETS: [i8; 27] = [
    0, 3, 6, 9, 12, -3, -6, -9, -12, 0, 2, 4, 6, 8, -2, -4, -6, -8, 0, 1, 2, 3, 4, -1, -2, -3, -4,
];

// kMothula_FlapWingsGfx from sprite_main.c:22592.
const MOTHULA_WING_FLAP_GRAPHICS: [u8; 4] = [0, 1, 2, 1];

// kMothula_XYvel from sprite_main.c:22562.
const MOTHULA_AXIS_VELOCITIES: [i8; 10] = [-16, -12, 0, 12, 16, 12, 0, -12, -16, -12];

// kMothula_Beam_Xvel / Yvel from sprite_main.c:22600-22601.
const MOTHULA_BEAM_X_VELOCITIES: [i8; 3] = [-16, 0, 16];
const MOTHULA_BEAM_Y_VELOCITIES: [i8; 3] = [24, 32, 24];

// kMothula_Spike_XLo / YLo / Dir from sprite_main.c:22621-22632.
const MOTHULA_SPIKE_X_LOW: [u8; 30] = [
    0x38, 0x48, 0x58, 0x68, 0x88, 0x98, 0xa8, 0xb8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xb8,
    0xa8, 0x98, 0x78, 0x68, 0x58, 0x48, 0x38, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28,
];
const MOTHULA_SPIKE_Y_LOW: [u8; 30] = [
    0x38, 0x38, 0x38, 0x38, 0x38, 0x38, 0x38, 0x38, 0x48, 0x58, 0x68, 0x78, 0x98, 0xa8, 0xb8, 0xc8,
    0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xc8, 0xb8, 0xa8, 0x98, 0x78, 0x68, 0x58, 0x48,
];
const MOTHULA_SPIKE_DIRECTIONS: [u8; 30] = [
    2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0,
];
const GIBDO_DIRECTION_TARGETS: [u8; 4] = [2, 6, 4, 0];
const GIBDO_GRAPHICS: [u8; 8] = [4, 8, 11, 10, 0, 6, 3, 7];
const GIBDO_AXIS_VELOCITIES: [i8; 10] = [-16, 0, 0, 0, 16, 0, 0, 0, -16, 0];
const GIBDO_ALT_GRAPHICS: [u8; 8] = [9, 2, 0, 4, 11, 3, 1, 5];
const PIROGUSU_DIRECTION_LOOKUP: [u8; 4] = [2, 3, 0, 1];
const PIROGUSU_GRAPHICS_LOOKUP: [u8; 8] = [9, 11, 5, 7, 5, 11, 7, 9];
const PIROGUSU_ANIMATION_STATE_LOOKUP: [u8; 8] = [16, 17, 18, 19, 12, 13, 14, 15];
const PIROGUSU_AXIS_VELOCITIES: [i8; 6] = [0, 0, 4, -4, 0, 0];
const PIROGUSU_COLLISION_AXIS_VELOCITIES: [i8; 6] = [2, -2, 0, 0, 2, -2];
const PIROGUSU_FAST_AXIS_VELOCITIES: [i8; 6] = [24, -24, 0, 0, 24, -24];
const PIROGUSU_DIRECTIONS: [u8; 8] = [2, 3, 2, 3, 0, 1, 0, 1];
const LASER_EYE_DIRECTIONS: [u8; 4] = [2, 3, 0, 1];
const STALFOS_KNIGHT_CASE2_GRAPHICS: [u8; 2] = [0, 1];
const STALFOS_KNIGHT_CASE2_DIRECTIONS: [u8; 16] = [0, 0, 0, 2, 1, 1, 1, 2, 0, 0, 0, 2, 1, 1, 1, 2];
const STALFOS_KNIGHT_CASE6_STATES: [u8; 32] = [
    0, 4, 8, 12, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14,
    14, 14, 15, 14, 12, 8, 4, 0,
];
const STALFOS_KNIGHT_CASE7_GRAPHICS: [u8; 2] = [1, 4];
const TERRORPIN_X_VELOCITIES: [i8; 8] = [8, -8, 0, 0, 12, -12, 0, 0];
const TERRORPIN_Y_VELOCITIES: [i8; 8] = [0, 0, 8, -8, 0, 0, 12, -12];
const TERRORPIN_OAM_FLAGS: [u8; 2] = [0, 0x40];
const TERRORPIN_OVERTURNED_X_VELOCITIES: [i8; 2] = [8, -8];
const ARRGHUS_GRAPHICS_BY_STEP: [u8; 9] = [1, 1, 1, 2, 2, 1, 1, 0, 0];
const ARRGHI_GRAPHICS_BY_STEP: [u8; 8] = [0, 1, 2, 2, 2, 2, 2, 1];
const BABUSU_GRAPHICS: [u8; 6] = [5, 4, 3, 2, 1, 0];
const BABUSU_DIRECTION_GRAPHICS: [u8; 4] = [6, 6, 0, 0];
const BABUSU_AXIS_VELOCITIES: [i8; 6] = [32, -32, 0, 0, 32, -32];
const BABUSU_SCURRY_GRAPHICS: [u8; 4] = [18, 14, 12, 16];
const WIZZROBE_CLOAK_GRAPHICS: [u8; 4] = [4, 2, 0, 6];
const WIZZROBE_ATTACK_GRAPHICS: [u8; 8] = [0, 1, 1, 1, 1, 1, 1, 0];
const WIZZROBE_ATTACK_DIRECTION_GRAPHICS: [u8; 4] = [4, 2, 0, 6];
const KYAMERON_COAGULATE_GRAPHICS: [u8; 8] = [4, 7, 14, 13, 12, 6, 6, 5];
const KYAMERON_X_VELOCITIES: [i8; 4] = [32, -32, 32, -32];
const KYAMERON_Y_VELOCITIES: [i8; 4] = [32, 32, -32, -32];
const KYAMERON_MOVING_GRAPHICS: [u8; 4] = [3, 2, 1, 0];
const PENGATOR_GRAPHICS_BY_DIRECTION: [u8; 4] = [5, 0, 10, 15];
const PENGATOR_AXIS_VELOCITIES: [i8; 6] = [1, -1, 0, 0, 1, -1];
const PENGATOR_JUMP_GRAPHICS: [u8; 4] = [4, 4, 3, 2];
const PENGATOR_GARNISH_Y_OFFSETS: [i8; 8] = [8, 10, 12, 14, 12, 12, 12, 12];
const PENGATOR_GARNISH_X_OFFSETS: [i8; 8] = [4, 4, 4, 4, 0, 4, 8, 12];
const FLUTE_BOY_ANIMAL_X_VELOCITIES: [i8; 4] = [16, -16, 0, 0];
const ZAZAK_Y_VELOCITIES: [i8; 4] = [0, 0, 16, -16];
const GORIYA_X_VELOCITIES: [i8; 32] = [
    0, 16, -16, 0, 0, 13, -13, 0, 0, 13, -13, 0, 0, 0, 0, 0, 0, -24, 24, 0, 0, -16, 16, 0, 0, -16,
    16, 0, 0, 0, 0, 0,
];
const GORIYA_Y_VELOCITIES: [i8; 32] = [
    0, 0, 0, 0, -16, -5, -5, 0, 16, 13, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, -24, -16, -16, 0, 24, 16,
    16, 0, 0, 0, 0, 0,
];
const GORIYA_DIRECTIONS: [u8; 32] = [
    0, 0, 1, 0, 3, 3, 3, 0, 2, 2, 2, 0, 0, 0, 0, 0, 0, 1, 0, 0, 3, 3, 3, 0, 2, 2, 2, 0, 0, 0, 0, 0,
];
const GORIYA_GRAPHICS: [u8; 16] = [8, 6, 0, 3, 9, 7, 1, 4, 8, 6, 0, 3, 9, 7, 2, 5];
const EYEGORE_CLOSING_GRAPHICS: [u8; 8] = [0, 0, 1, 1, 2, 2, 2, 2];
const EYEGORE_OPENING_GRAPHICS: [u8; 8] = [2, 2, 2, 2, 1, 1, 0, 0];
const EYEGORE_CHASING_GRAPHICS: [u8; 16] = [7, 5, 2, 9, 8, 6, 3, 10, 7, 5, 2, 9, 8, 6, 4, 11];
const EYEGORE_OPENING_DELAYS: [u8; 4] = [0x60, 0x80, 0xa0, 0x80];
const ARMOS_KNIGHT_WAKE_GRAPHICS: [u8; 5] = [5, 4, 3, 2, 1];
const ARMOS_KNIGHT_CHARGE_X_VELOCITIES: [i8; 2] = [16, -16];
const FLUTE_BOY_ANIMAL_OAM_FLAGS: [u8; 2] = [0x40, 0];
const FLUTE_BOY_ANIMAL_GRAPHICS: [u8; 3] = [0, 1, 2];
const FLUTE_BOY_OSTRICH_GRAPHICS: [u8; 4] = [0, 1, 0, 2];
const FLUTE_BOY_BIRD_X_OFFSETS: [i8; 2] = [8, 0];
const FREEZOR_X_VELOCITIES: [i8; 4] = [8, -8, 0, 0];
const FREEZOR_Y_VELOCITIES: [i8; 4] = [0, 0, 18, -18];
const FREEZOR_MOVING_GRAPHICS: [u8; 4] = [1, 2, 1, 3];
const FREEZOR_SPARKLE_X_OFFSETS: [i8; 8] = [-4, -2, 0, 2, 4, 6, 8, 10];
const FREEZOR_MELTING_GRAPHICS: [u8; 4] = [6, 5, 4, 7];
const KODONDO_X_VELOCITIES: [i8; 4] = [1, -1, 0, 0];
const KODONDO_Y_VELOCITIES: [i8; 4] = [0, 0, 1, -1];
const KODONDO_GRAPHICS: [u8; 8] = [2, 2, 0, 5, 3, 3, 0, 5];
const KODONDO_OAM_FLAGS: [u8; 8] = [0x40, 0, 0, 0, 0x40, 0, 0x40, 0x40];
const KODONDO_FLAME_GRAPHICS: [u8; 8] = [2, 2, 0, 5, 4, 4, 1, 6];
const KHOLDSTARE_TARGET_X_VELOCITIES: [i8; 4] = [16, 16, -16, -16];
const KHOLDSTARE_TARGET_Y_VELOCITIES: [i8; 4] = [-16, 16, 16, -16];
const KHOLDSTARE_SHELL_FRAGMENT_Z_VELOCITIES: [i8; 3] = [32, -32, 0];
const KHOLDSTARE_SHELL_FRAGMENT_Z_SUBPIXELS: [i8; 3] = [-32, -32, 48];
const BOMBER_GRAPHICS_BY_DIRECTION: [u8; 4] = [9, 10, 8, 7];
const BOMBER_X_VELOCITIES: [i8; 8] = [16, 12, 0, -12, -16, -12, 0, 12];
const BOMBER_Y_VELOCITIES: [i8; 8] = [0, 12, 16, 12, 0, -12, -16, -12];
const BOMBER_SHOT_GRAPHICS_BY_DIRECTION: [u8; 4] = [0, 4, 2, 6];
const BOMBER_PELLET_X_OFFSETS: [i8; 4] = [14, -6, 4, 4];
const BOMBER_PELLET_Y_OFFSETS: [i8; 4] = [7, 7, 12, -4];
const PIKIT_GRAPHICS: [u8; 24] = [
    2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2,
];
const PIKIT_TONGUE_XY_OFFSETS: [i8; 72] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    12, 16, 24, 32, 32, 24, 16, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -12, -16, -24,
    -32, -32, -24, -16, -12, 0, 0, 0, 0, 0, 0, 0, 0,
];
const PIKIT_TONGUE_X_OFFSET_BASES: [u8; 8] = [24, 24, 0, 48, 48, 48, 0, 24];
const PIKIT_TONGUE_Y_OFFSET_BASES: [u8; 8] = [0, 24, 24, 24, 0, 48, 48, 48];
const STALFOS_DIRECTION_ANIMATION_STATES: [u8; 4] = [8, 9, 10, 11];
const STALFOS_CHECK_DIRECTIONS: [u8; 4] = [3, 2, 1, 0];
const STALFOS_PRIMARY_ANIMATION_STATES: [u8; 8] = [6, 4, 0, 2, 7, 5, 1, 3];
const STALFOS_HOP_DELAYS: [u8; 4] = [16, 32, 64, 32];
const ZAZAK_ALT_DIRECTIONS: [u8; 8] = [2, 3, 2, 3, 0, 1, 0, 1];
const FIREBALL_JUNCTION_X_OFFSETS: [i8; 4] = [12, -12, 0, 0];
const FIREBALL_JUNCTION_Y_OFFSETS: [i8; 4] = [0, 0, 12, -12];
const FIREBALL_JUNCTION_AXIS_VELOCITIES: [i8; 6] = [0, 0, 40, -40, 0, 0];
const GIBO_OAM_FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];
const GIBO_X_VELOCITIES: [i8; 8] = [16, 16, 0, -16, -16, -16, 0, 16];
const GIBO_Y_VELOCITIES: [i8; 8] = [0, 0, 16, -16, 16, 16, -16, -16];
const TEKITE_DIRECTIONS: [u8; 4] = [3, 2, 1, 0];
const TEKITE_X_VELOCITIES: [i8; 4] = [16, -16, 16, -16];
const TEKITE_Y_VELOCITIES: [i8; 4] = [16, 16, -16, -16];
const HOVER_OAM_FLAGS: [u8; 4] = [0x40, 0, 0x40, 0];
const HOVER_PRIMARY_X_ACCELERATIONS: [i8; 4] = [1, -1, 1, -1];
const HOVER_PRIMARY_Y_ACCELERATIONS: [i8; 4] = [1, 1, -1, -1];
const HOVER_SECONDARY_X_ACCELERATIONS: [i8; 4] = [-1, 1, -1, 1];
const HOVER_SECONDARY_Y_ACCELERATIONS: [i8; 4] = [-1, -1, 1, 1];
const CHAIN_CHOMP_X_VELOCITIES: [i8; 16] = [
    0, 8, 11, 14, 16, 14, 11, 8, 0, -8, -11, -14, -16, -14, -11, -8,
];
const CHAIN_CHOMP_Y_VELOCITIES: [i8; 16] = [
    -16, -14, -11, -8, 0, 8, 11, 14, 16, 14, 11, 8, 0, -9, -11, -14,
];
const HOKBOK_SEGMENT_STATES: [u8; 8] = [8, 7, 6, 5, 4, 5, 6, 7];
const BOULDER_Z_VELOCITIES: [i8; 2] = [32, 48];
const BOULDER_Y_VELOCITIES: [i8; 2] = [8, 32];
const BOULDER_X_VELOCITIES: [i8; 4] = [24, 16, -24, -16];

impl ZeldaState {
    // void Sprite_Wizzbeam(int k) {
    pub(super) fn sprite_wizzbeam(&mut self, k: usize) {
        self.wizzbeam_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_slot_mut(k).xor_oam_flags(6);
        self.sprite_slot_mut(k).increment_subtype2();
        if self.sprite_slot(k).ai_state() == 0 {
            self.sprite_check_damage_to_link(k);
        }
        self.sprite_move_xy(k);
        if self.sprite_check_tile_collision(k) != 0 {
            self.sprite_slot_mut(k).set_state(0);
        }
    }

    // void Sprite_9B_Wizzrobe(int k) {  // 9e9d1b
    pub(super) fn sprite_9_b_wizzrobe(&mut self, k: usize) {
        if self.sprite_slot(k).c() != 0 {
            self.sprite_wizzbeam(k);
            return;
        }

        if self.sprite_slot(k).ai_state() == 0
            || ((self.sprite_slot(k).ai_state() & 1) != 0
                && (self.sprite_slot(k).delay_main() & 1) != 0)
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
        self.sprite_slot_mut(k).set_ignore_projectile(1);

        match self.sprite_slot(k).ai_state() {
            0 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_x_velocity(1);
                    self.sprite_slot_mut(k).set_y_velocity(1);
                    if self.sprite_check_tile_collision(k) == 0 {
                        self.sprite_slot_mut(k).set_ai_state(1);
                        self.sprite_slot_mut(k).set_delay_main(63);
                        let j = self.sprite_direction_to_face_link(k, None);
                        self.sprite_slot_mut(k).set_direction(j);
                        self.sprite_slot_mut(k)
                            .set_graphics(WIZZROBE_CLOAK_GRAPHICS[usize::from(j)]);
                    } else {
                        self.sprite_slot_mut(k).set_state(0);
                    }
                }
            }
            1 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(2);
                    self.sprite_slot_mut(k).set_delay_main(63);
                }
            }
            2 => {
                self.sprite_slot_mut(k).set_ignore_projectile(0);
                self.sprite_check_damage_to_and_from_link(k);
                let j = self.sprite_slot(k).delay_main();
                if j == 0 {
                    self.sprite_slot_mut(k).set_ai_state(3);
                    self.sprite_slot_mut(k).set_delay_main(63);
                    return;
                }
                if j == 32 {
                    self.wizzrobe_fire_beam(k);
                }
                let value = WIZZROBE_ATTACK_GRAPHICS[usize::from(j >> 3)]
                    + WIZZROBE_ATTACK_DIRECTION_GRAPHICS
                        [usize::from(self.sprite_slot(k).direction())];
                self.sprite_slot_mut(k).set_graphics(value);
            }
            3 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    if self.sprite_slot(k).b() != 0 {
                        self.sprite_slot_mut(k).set_state(0);
                    }
                    self.sprite_slot_mut(k).set_ai_state(0);
                    let value = (self.get_random_number() & 31).wrapping_add(32);
                    self.sprite_slot_mut(k).set_delay_main(value);
                }
            }
            _ => {}
        }
    }

    // void Sprite_9A_Kyameron(int k) {  // 9e9e7b
    pub(super) fn sprite_9_a_kyameron(&mut self, k: usize) {
        if self.sprite_slot(k).ai_state() == 0 {
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
        self.sprite_slot_mut(k).set_ignore_projectile(1);

        match self.sprite_slot(k).ai_state() {
            0 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(1);
                    let value = (self.get_random_number() & 63).wrapping_add(96);
                    self.sprite_slot_mut(k).set_delay_main(value);
                    let value = self.sprite_slot(k).a();
                    self.sprite_slot_mut(k).set_x_low(value);
                    let value = self.sprite_slot(k).b();
                    self.sprite_slot_mut(k).set_x_high(value);
                    let value = self.sprite_slot(k).c();
                    self.sprite_slot_mut(k).set_y_low(value);
                    let value = self.sprite_slot(k).head_direction();
                    self.sprite_slot_mut(k).set_y_high(value);
                    self.sprite_slot_mut(k).set_subtype2(5);
                    self.sprite_slot_mut(k).set_graphics(8);
                }
            }
            1 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_delay_main(31);
                    self.sprite_slot_mut(k).set_ai_state(2);
                }
                self.sprite_slot_mut(k).decrement_subtype2();
                if sign8(self.sprite_slot(k).subtype2()) {
                    self.sprite_slot_mut(k).set_subtype2(5);
                    let value =
                        (self.sprite_slot(k).graphics().wrapping_add(1) & 3).wrapping_add(8);
                    self.sprite_slot_mut(k).set_graphics(value);
                }
            }
            2 => {
                let j = self.sprite_slot(k).delay_main();
                if j == 0 {
                    self.sprite_slot_mut(k).set_ai_state(3);
                    let idx = usize::from(
                        self.sprite_is_below_link(k).a * 2 + self.sprite_is_right_of_link(k).a,
                    );
                    self.sprite_slot_mut(k)
                        .set_x_velocity(KYAMERON_X_VELOCITIES[idx] as u8);
                    self.sprite_slot_mut(k)
                        .set_y_velocity(KYAMERON_Y_VELOCITIES[idx] as u8);
                } else {
                    if j == 7 {
                        self.sprite_set_y(k, self.sprite_get_y(k).wrapping_sub(29));
                    }
                    self.sprite_slot_mut(k)
                        .set_graphics(KYAMERON_COAGULATE_GRAPHICS[usize::from(j >> 2)]);
                }
            }
            3 => {
                self.sprite_slot_mut(k).set_ignore_projectile(0);
                let mut should_disperse = false;
                if !self.sprite_check_damage_to_and_from_link(k) {
                    self.sprite_move_xy(k);
                    let j = self.sprite_check_tile_collision(k);
                    if (j & 3) != 0 {
                        let value = (0u8).wrapping_sub(self.sprite_slot(k).x_velocity());
                        self.sprite_slot_mut(k).set_x_velocity(value);
                        self.sprite_slot_mut(k).increment_anim_clock();
                    }
                    if (j & 12) != 0 {
                        let value = (0u8).wrapping_sub(self.sprite_slot(k).y_velocity());
                        self.sprite_slot_mut(k).set_y_velocity(value);
                        self.sprite_slot_mut(k).increment_anim_clock();
                    }
                    if self.sprite_slot(k).anim_clock() >= 3 {
                        should_disperse = true;
                    }
                } else {
                    should_disperse = true;
                }
                if should_disperse {
                    self.sprite_slot_mut(k).set_ai_state(4);
                    self.sprite_slot_mut(k).set_delay_main(15);
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x28);
                }
                self.sprite_slot_mut(k).increment_subtype2();
                let value = KYAMERON_MOVING_GRAPHICS
                    [usize::from((self.sprite_slot(k).subtype2() >> 3) & 3)];
                self.sprite_slot_mut(k).set_graphics(value);
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 7) == 0 {
                    let x = u16::from(self.get_random_number() & 0x0f).wrapping_sub(4);
                    let y = u16::from(self.get_random_number() & 0x0f).wrapping_sub(4);
                    self.sprite_garnish_spawn_sparkle(k, x, y);
                }
            }
            4 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_anim_clock(0);
                    self.sprite_slot_mut(k).set_ai_state(0);
                    self.sprite_slot_mut(k).set_z(0);
                    self.sprite_slot_mut(k).set_delay_main(64);
                } else {
                    let value = (self.sprite_slot(k).delay_main() >> 2) + 15;
                    self.sprite_slot_mut(k).set_graphics(value);
                }
            }
            _ => {}
        }
    }

    // void Sprite_99_Pengator(int k) {  // 9ea196
    pub(super) fn sprite_99_pengator(&mut self, k: usize) {
        let value = self.sprite_slot(k).a().wrapping_add(
            PENGATOR_GRAPHICS_BY_DIRECTION[usize::from(self.sprite_slot(k).direction())],
        );
        self.sprite_slot_mut(k).set_graphics(value);
        self.pengator_draw(k);
        if self.sprite_slot(k).f() != 0 || (self.sprite_slot(k).wall_collision() & 15) != 0 {
            self.sprite_slot_mut(k).set_ai_state(0);
            self.sprite_slot_mut(k).set_x_velocity(0);
            self.sprite_slot_mut(k).set_y_velocity(0);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xyz(k);
        self.sprite_slot_mut(k).subtract_z_velocity(2);
        if sign8(self.sprite_slot(k).z()) {
            self.sprite_slot_mut(k).set_z_velocity(0);
            self.sprite_slot_mut(k).set_z(0);
        }
        self.sprite_check_tile_collision(k);

        match self.sprite_slot(k).ai_state() {
            0 => {
                let value = self.sprite_direction_to_face_link(k, None);
                self.sprite_slot_mut(k).set_direction(value);
                self.sprite_slot_mut(k).set_ai_state(1);
            }
            1 => {
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0 {
                    let mut flag = false;
                    let j = usize::from(self.sprite_slot(k).direction());
                    if self.sprite_slot(k).x_velocity() != FLUTE_BOY_ANIMAL_X_VELOCITIES[j] as u8 {
                        let value = self
                            .sprite_slot(k)
                            .x_velocity()
                            .wrapping_add(PENGATOR_AXIS_VELOCITIES[j] as u8);
                        self.sprite_slot_mut(k).set_x_velocity(value);
                        flag = true;
                    }
                    if self.sprite_slot(k).y_velocity() != ZAZAK_Y_VELOCITIES[j] as u8 {
                        let value = self
                            .sprite_slot(k)
                            .y_velocity()
                            .wrapping_add(PENGATOR_AXIS_VELOCITIES[j + 2] as u8);
                        self.sprite_slot_mut(k).set_y_velocity(value);
                        flag = true;
                    }
                    if !flag {
                        self.sprite_slot_mut(k).set_delay_main(15);
                        self.sprite_slot_mut(k).set_ai_state(2);
                    }
                }
                let value = (self.game_state.frame.frame_counter & 4) >> 2;
                self.sprite_slot_mut(k).set_a(value);
            }
            2 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).increment_ai_state();
                } else if self.sprite_slot(k).delay_main() == 5 {
                    self.sprite_slot_mut(k).set_z_velocity(24);
                }
                let value =
                    PENGATOR_JUMP_GRAPHICS[usize::from(self.sprite_slot(k).delay_main() >> 2)];
                self.sprite_slot_mut(k).set_a(value);
            }
            3 => {
                if ((((k as u8) ^ self.game_state.frame.frame_counter) & 7)
                    | self.sprite_slot(k).z())
                    == 0
                {
                    let i = usize::from(self.sprite_slot(k).direction());
                    let base = usize::from(i >= 2) * 4;
                    let x = PENGATOR_GARNISH_X_OFFSETS
                        [usize::from(self.get_random_number() & 3) + base];
                    let y = PENGATOR_GARNISH_Y_OFFSETS
                        [usize::from(self.get_random_number() & 3) + base];
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

        match self.sprite_slot(k).ai_state() {
            0 => {
                let value = if (self.game_state.frame.frame_counter & 0x18) != 0 {
                    3
                } else {
                    0
                };
                self.sprite_slot_mut(k).set_graphics(value);
                if self
                    .game_state
                    .sprites
                    .garnish_runtime
                    .haunted_grove_flute_event_latch()
                    != 0
                {
                    self.sprite_slot_mut(k).set_ai_state(1);
                    self.sprite_slot_mut(k).set_y_velocity((-8i8) as u8);
                    self.sprite_slot_mut(k).set_x_velocity((-16i8) as u8);
                }
            }
            1 => {
                self.sprite_move_xyz(k);
                self.sprite_slot_mut(k).subtract_z_velocity(2);
                if sign8(self.sprite_slot(k).z()) {
                    self.sprite_slot_mut(k).set_z_velocity(32);
                    self.sprite_slot_mut(k).set_z(0);
                    self.sprite_slot_mut(k).set_subtype2(0);
                    self.sprite_slot_mut(k).set_a(0);
                }
                self.sprite_slot_mut(k).increment_subtype2();
                if (self.sprite_slot(k).subtype2() & 7) == 0 && self.sprite_slot(k).a() != 3 {
                    self.sprite_slot_mut(k).add_a(1);
                }
                let value = FLUTE_BOY_OSTRICH_GRAPHICS[usize::from(self.sprite_slot(k).a())];
                self.sprite_slot_mut(k).set_graphics(value);
            }
            _ => {}
        }
    }

    // void Sprite_9F_HauntedGroveRabbit(int k) {  // 9e9a6d
    pub(super) fn sprite_9_f_haunted_grove_rabbit(&mut self, k: usize) {
        let value = (self.sprite_slot(k).oam_flags() & !0x40)
            | FLUTE_BOY_ANIMAL_OAM_FLAGS[usize::from(self.sprite_slot(k).direction())];
        self.sprite_slot_mut(k).set_oam_flags(value);
        self.sprite_draw_single_large(k);

        match self.sprite_slot(k).ai_state() {
            0 => {
                self.sprite_slot_mut(k).set_graphics(3);
                if self
                    .game_state
                    .sprites
                    .garnish_runtime
                    .haunted_grove_flute_event_latch()
                    != 0
                {
                    self.sprite_slot_mut(k).set_ai_state(1);
                    self.sprite_slot_mut(k).xor_direction(1);
                    let value = FLUTE_BOY_ANIMAL_X_VELOCITIES
                        [usize::from(self.sprite_slot(k).direction())]
                        as u8;
                    self.sprite_slot_mut(k).set_x_velocity(value);
                    self.sprite_slot_mut(k).set_y_velocity((-8i8) as u8);
                }
            }
            1 => {
                self.sprite_move_xyz(k);
                self.sprite_slot_mut(k).subtract_z_velocity(3);
                if sign8(self.sprite_slot(k).z()) {
                    self.sprite_slot_mut(k).set_z_velocity(24);
                    self.sprite_slot_mut(k).set_z(0);
                    self.sprite_slot_mut(k).set_subtype2(0);
                    self.sprite_slot_mut(k).set_a(0);
                }
                self.sprite_slot_mut(k).increment_subtype2();
                if (self.sprite_slot(k).subtype2() & 3) == 0 && self.sprite_slot(k).a() != 2 {
                    self.sprite_slot_mut(k).add_a(1);
                }
                let value = FLUTE_BOY_ANIMAL_GRAPHICS[usize::from(self.sprite_slot(k).a())];
                self.sprite_slot_mut(k).set_graphics(value);
            }
            _ => {}
        }
    }

    // void Sprite_A0_HauntedGroveBird(int k) {  // 9e9aec
    pub(super) fn sprite_a0_haunted_grove_bird(&mut self, k: usize) {
        if self.sprite_slot(k).graphics() == 3 {
            self.haunted_grove_bird_blink(k);
        }
        let value = (self.sprite_slot(k).oam_flags() & !0x40)
            | FLUTE_BOY_ANIMAL_OAM_FLAGS[usize::from(self.sprite_slot(k).direction())];
        self.sprite_slot_mut(k).set_oam_flags(value);
        let cur = self.game_state.oam.current_pointer();
        self.oam_state_mut()
            .set_current_pointer(cur.wrapping_add(4));
        let ext = self.game_state.oam.current_extended_pointer();
        self.oam_state_mut()
            .set_current_extended_pointer(ext.wrapping_add(1));
        self.sprite_slot_mut(k).subtract_flags2(1);
        self.sprite_draw_single_large(k);
        self.sprite_slot_mut(k).add_flags2(1);
        self.sprite_move_xyz(k);

        match self.sprite_slot(k).ai_state() {
            0 => {
                let value = if (self.game_state.frame.frame_counter & 0x18) != 0 {
                    0
                } else {
                    3
                };
                self.sprite_slot_mut(k).set_graphics(value);
                if self
                    .game_state
                    .sprites
                    .garnish_runtime
                    .haunted_grove_flute_event_latch()
                    != 0
                {
                    self.sprite_slot_mut(k).set_ai_state(1);
                    self.sprite_slot_mut(k).xor_direction(1);
                    let value = FLUTE_BOY_ANIMAL_X_VELOCITIES
                        [usize::from(self.sprite_slot(k).direction())]
                        as u8;
                    self.sprite_slot_mut(k).set_x_velocity(value);
                    self.sprite_slot_mut(k).set_delay_main(32);
                    self.sprite_slot_mut(k).set_z_velocity(16);
                    self.sprite_slot_mut(k).set_y_velocity((-8i8) as u8);
                }
            }
            1 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).add_z_velocity(2);
                    if !sign8(self.sprite_slot(k).z_velocity().wrapping_sub(0x10)) {
                        self.sprite_slot_mut(k).set_ai_state(2);
                    }
                }
                self.sprite_slot_mut(k).increment_subtype2();
                let value = ((self.sprite_slot(k).subtype2() >> 1) & 1) + 1;
                self.sprite_slot_mut(k).set_graphics(value);
            }
            2 => {
                self.sprite_slot_mut(k).set_graphics(1);
                self.sprite_slot_mut(k).decrement_z_velocity();
                if sign8(self.sprite_slot(k).z_velocity().wrapping_add(15)) {
                    self.sprite_slot_mut(k).set_ai_state(1);
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
        let oam = self.game_state.oam.current_pointer_usize();
        let j = usize::from(self.sprite_slot(k).direction());
        self.oam_state_mut().write_entry(
            oam,
            x.wrapping_add(FLUTE_BOY_BIRD_X_OFFSETS[j] as i16 as u16) as u8,
            y as u8,
            0xae,
            flags | FLUTE_BOY_ANIMAL_OAM_FLAGS[j],
        );
        self.sprite_correct_oam_entries(k, 0, 0);
    }

    // void Sprite_A4_FallingIce(int k) {  // 9e9710
    pub(super) fn sprite_a4_falling_ice(&mut self, k: usize) {
        if self.sprite_slot(k).c() == 0 {
            if self.sprite_return_if_inactive(k) {
                return;
            }
            if self.sprite_slot(2).state() < 9
                && self.sprite_slot(3).state() < 9
                && self.sprite_slot(4).state() < 9
            {
                self.sprite_slot_mut(k).set_state(0);
            }
            self.generate_iceball(k);
            return;
        }

        self.sprite_slot_mut(k).set_ignore_projectile(1);
        self.sprite_slot_mut(k).set_object_priority(0x30);
        self.sprite_draw_single_large(k);
        if self.sprite_slot(k).ai_state() == 0 {
            self.sprite_slot_mut(k).xor_flags3(16);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot(k).delay_main() != 0 {
            if self.sprite_slot(k).delay_main() == 1 {
                self.sprite_slot_mut(k).set_state(0);
            }
            let value = (self.sprite_slot(k).delay_main() >> 3) + 2;
            self.sprite_slot_mut(k).set_graphics(value);
            return;
        }

        self.sprite_move_xy(k);
        let mut hit_solid = false;
        if self.sprite_slot(k).ai_state() != 0 {
            self.sprite_check_damage_to_link(k);
            hit_solid = self.sprite_check_tile_collision(k) != 0;
        }
        if self.sprite_slot(k).ai_state() == 0 || !hit_solid {
            let old_z = self.sprite_slot(k).z();
            self.sprite_move_z(k);
            if !sign8(self.sprite_slot(k).z_velocity().wrapping_add(64)) {
                self.sprite_slot_mut(k).subtract_z_velocity(3);
            }
            if !(sign8(old_z ^ self.sprite_slot(k).z()) && sign8(self.sprite_slot(k).z())) {
                return;
            }
            self.sprite_slot_mut(k).set_z(0);
            if self.sprite_slot(k).ai_state() == 0 {
                self.sprite_slot_mut(k).set_state(0);
                self.ice_ball_split(k);
                return;
            }
        }
        self.sprite_slot_mut(k).set_delay_main(15);
        self.sprite_slot_mut(k).set_oam_flags(4);
        if !self.game_state.system_signals.has_sound_effect_1() {
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x1e);
            self.sprite_slot_mut(k).set_graphics(3);
        }
    }

    // void Sprite_A1_Freezor(int k) {  // 9e981d
    pub(super) fn sprite_a1_freezor(&mut self, k: usize) {
        self.freezor_draw(k);
        if self.sprite_slot(k).state() != 9 {
            self.sprite_slot_mut(k).set_ai_state(3);
            self.sprite_slot_mut(k).set_delay_main(31);
            self.sprite_slot_mut(k).set_ignore_projectile(31);
            self.sprite_slot_mut(k).set_state(9);
            self.sprite_slot_mut(k).set_hit_timer(0);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot(k).ai_state() != 3 && self.sprite_return_if_recoiling(k) {
            return;
        }

        match self.sprite_slot(k).ai_state() {
            0 => {
                let value = self.sprite_slot(k).ignore_projectile().wrapping_add(1);
                self.sprite_slot_mut(k).set_ignore_projectile(value);
                if self.sprite_is_right_of_link(k).b.wrapping_add(16) < 32 {
                    self.sprite_slot_mut(k).set_ai_state(1);
                    self.sprite_slot_mut(k).set_delay_main(32);
                }
            }
            1 => {
                let value = self.sprite_slot(k).delay_main();
                self.sprite_slot_mut(k).set_ignore_projectile(value);
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(2);
                    let x = self.sprite_get_x(k).wrapping_sub(5);
                    let y = self.sprite_get_y(k);
                    self.dungeon_update_tile_map_with_common_tile_for_mothula(x, y, 8);
                    self.sprite_slot_mut(k).set_delay_aux1(96);
                    self.sprite_slot_mut(k).set_direction(2);
                    self.sprite_slot_mut(k).set_delay_main(80);
                } else {
                    let value = if (self.sprite_slot(k).delay_main() & 1) != 0 {
                        (-16i8) as u8
                    } else {
                        16
                    };
                    self.sprite_slot_mut(k).set_x_velocity(value);
                    self.sprite_move_x(k);
                }
            }
            2 => {
                self.sprite_check_damage_to_link(k);
                if self.sprite_check_damage_from_link(k) != 0 {
                    self.sprite_slot_mut(k).set_hit_timer(0);
                }
                if self.sprite_slot(k).delay_aux1() != 0
                    && (((k as u8) ^ self.game_state.frame.frame_counter) & 7) == 0
                {
                    let x = FREEZOR_SPARKLE_X_OFFSETS[usize::from(self.get_random_number() & 7)]
                        as i16 as u16;
                    self.sprite_garnish_spawn_sparkle(k, x, (-4i16) as u16);
                }
                if self.sprite_slot(k).delay_main() == 0 {
                    let value = self.sprite_direction_to_face_link(k, None);
                    self.sprite_slot_mut(k).set_direction(value);
                }
                let j = usize::from(self.sprite_slot(k).direction());
                self.sprite_slot_mut(k)
                    .set_x_velocity(FREEZOR_X_VELOCITIES[j] as u8);
                self.sprite_slot_mut(k)
                    .set_y_velocity(FREEZOR_Y_VELOCITIES[j] as u8);
                if (self.sprite_slot(k).wall_collision() & 15) == 0 {
                    self.sprite_move_xy(k);
                }
                self.sprite_check_tile_collision(k);
                let value = FREEZOR_MOVING_GRAPHICS
                    [usize::from(((k as u8) ^ self.game_state.frame.frame_counter) >> 2 & 3)];
                self.sprite_slot_mut(k).set_graphics(value);
            }
            3 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_manually_set_death_flag_uw(k);
                    self.sprite_slot_mut(k).set_state(0);
                }
                let value =
                    FREEZOR_MELTING_GRAPHICS[usize::from(self.sprite_slot(k).delay_main() >> 3)];
                self.sprite_slot_mut(k).set_graphics(value);
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
        if self.sprite_slot(k).ai_state() == 0 {
            if self.sprite_slot(k).state() == 6 {
                self.sprite_slot_mut(k).set_flags3(0xc0);
                self.sprite_slot_mut(k).set_ai_state(1);
                self.sprite_slot_mut(k).set_state(9);
            } else if self.sprite_slot(k).hit_timer() != 0 {
                let x_offs = if (self.sprite_slot(k).hit_timer() & 2) != 0 {
                    0xffff
                } else {
                    1
                };
                self.dungeon_moving_floor_mut().set_floor_x_offset(x_offs);
                self.dungeon_room_load_mut()
                    .set_header_collision_2_mirror(1);
            } else {
                self.dungeon_room_load_mut()
                    .set_header_collision_2_mirror(0);
            }
        } else {
            let state = self.sprite_slot(k).ai_state();
            self.sprite_slot_mut(k).set_ai_state(state.wrapping_add(1));
            if state != 18 {
                self.KholdstareShell_PaletteFiltering();
            } else {
                self.sprite_slot_mut(k).set_state(0);
                self.sprite_slot_mut(2).set_ai_state(2);
                self.sprite_slot_mut(2).set_delay_main(128);
            }
        }
    }

    // void Sprite_A2_Kholdstare(int k) {  // 9e9518
    pub(super) fn sprite_a2_kholdstare(&mut self, k: usize) {
        self.kholdstare_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot(k).ai_state() < 2 {
            self.kholdstare_spawn_puff_cloud_garnish(k);
            if (self.game_state.frame.frame_counter & 7) == 0 {
                self.set_sound_effect_1(2);
            }
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }

        self.sprite_slot_mut(k).decrement_subtype2();
        if sign8(self.sprite_slot(k).subtype2()) {
            self.sprite_slot_mut(k).set_subtype2(10);
            let value = self.sprite_slot(k).graphics().wrapping_add(1) & 3;
            self.sprite_slot_mut(k).set_graphics(value);
        }

        if (self.game_state.frame.frame_counter & 3) == 0 {
            let pt = self.sprite_project_speed_towards_link(k, 31);
            self.sprite_slot_mut(k)
                .set_a(ZeldaState::sprite_convert_velocity_to_angle(pt.x, pt.y));
        }

        self.sprite_move_xy(k);
        match self.sprite_slot(k).ai_state() {
            0 => {
                self.sprite_check_damage_to_and_from_link(k);
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(1);
                    let value = (self.get_random_number() & 63) + 32;
                    self.sprite_slot_mut(k).set_delay_main(value);
                    return;
                }
                let x_delta = self
                    .sprite_slot(k)
                    .x_velocity()
                    .wrapping_sub(self.sprite_slot(k).z_velocity());
                if x_delta != 0 {
                    let value = self
                        .sprite_slot(k)
                        .x_velocity()
                        .wrapping_add(if sign8(x_delta) { 1 } else { 0xff });
                    self.sprite_slot_mut(k).set_x_velocity(value);
                }
                let y_delta = self
                    .sprite_slot(k)
                    .y_velocity()
                    .wrapping_sub(self.sprite_slot(k).z_subpixel());
                if y_delta != 0 {
                    let value = self
                        .sprite_slot(k)
                        .y_velocity()
                        .wrapping_add(if sign8(y_delta) { 1 } else { 0xff });
                    self.sprite_slot_mut(k).set_y_velocity(value);
                }
                self.kholdstare_check_collision_for_mothula(k);
            }
            1 => {
                self.sprite_check_damage_to_and_from_link(k);
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(0);
                    let value = (self.get_random_number() & 63) + 96;
                    self.sprite_slot_mut(k).set_delay_main(value);
                    let j = self.get_random_number();
                    if (j & 0x1c) == 0 {
                        let pt = self.sprite_project_speed_towards_link(k, 24);
                        self.sprite_slot_mut(k).set_z_velocity(pt.x);
                        self.sprite_slot_mut(k).set_z_subpixel(pt.y);
                    } else {
                        let i = usize::from(j & 3);
                        self.sprite_slot_mut(k)
                            .set_z_velocity(KHOLDSTARE_TARGET_X_VELOCITIES[i] as u8);
                        self.sprite_slot_mut(k)
                            .set_z_subpixel(KHOLDSTARE_TARGET_Y_VELOCITIES[i] as u8);
                    }
                } else {
                    if self.sprite_slot(k).x_velocity() != 0 {
                        let value = self.sprite_slot(k).x_velocity().wrapping_add(
                            if sign8(self.sprite_slot(k).x_velocity()) {
                                1
                            } else {
                                0xff
                            },
                        );
                        self.sprite_slot_mut(k).set_x_velocity(value);
                    }
                    if self.sprite_slot(k).y_velocity() != 0 {
                        let value = self.sprite_slot(k).y_velocity().wrapping_add(
                            if sign8(self.sprite_slot(k).y_velocity()) {
                                1
                            } else {
                                0xff
                            },
                        );
                        self.sprite_slot_mut(k).set_y_velocity(value);
                    }
                    self.kholdstare_check_collision_for_mothula(k);
                }
            }
            2 => {
                if self.sprite_slot(k).delay_main() == 1 {
                    self.sprite_slot_mut(k).set_state(0);
                    self.sprite_slot_mut(k + 1).set_state(0);
                    self.sprite_slot_mut(k + 2).set_state(0);
                    for i in (0..=2usize).rev() {
                        let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
                        let j = self.sprite_spawn_dynamically_ex(k, 0xa2, &mut info, 4);
                        if j >= 0 {
                            let j = j as usize;
                            self.sprite_set_spawned_coordinates(j, &info);
                            self.sprite_slot_mut(j)
                                .set_z_velocity(KHOLDSTARE_SHELL_FRAGMENT_Z_VELOCITIES[i] as u8);
                            self.sprite_slot_mut(j)
                                .set_z_subpixel(KHOLDSTARE_SHELL_FRAGMENT_Z_SUBPIXELS[i] as u8);
                            self.sprite_slot_mut(j).set_delay_main(32);
                        }
                    }
                    self.temp_counter_mut().set(0xff);
                } else {
                    self.sprite_slot_mut(k).or_hit_timer(0xe0);
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
        self.sprite_slot_mut(k).set_flags(0);

        match self.sprite_slot(k).ai_state() {
            0 => {
                self.sprite_slot_mut(k).increment_ai_state();
                let value = self.get_random_number() & 3;
                self.sprite_slot_mut(k).set_direction(value);
                self.sprite_slot_mut(k).set_flags(176);
                loop {
                    let j = usize::from(self.sprite_slot(k).direction());
                    self.sprite_slot_mut(k)
                        .set_x_velocity(KODONDO_X_VELOCITIES[j] as u8);
                    self.sprite_slot_mut(k)
                        .set_y_velocity(KODONDO_Y_VELOCITIES[j] as u8);
                    if self.sprite_check_tile_collision(k) == 0 {
                        break;
                    }
                    let value = self.sprite_slot(k).direction().wrapping_add(1) & 3;
                    self.sprite_slot_mut(k).set_direction(value);
                }
                self.kodongo_set_direction(k);
            }
            1 => {
                self.sprite_move_xy(k);
                if self.sprite_check_tile_collision(k) != 0 {
                    self.sprite_slot_mut(k).xor_direction(1);
                    self.kodongo_set_direction(k);
                }
                if (self.sprite_slot(k).x_low() & 0x1f) == 4
                    && (self.sprite_slot(k).y_low() & 0x1f) == 0x1b
                    && (self.get_random_number() & 3) == 0
                {
                    self.sprite_slot_mut(k).set_delay_main(111);
                    self.sprite_slot_mut(k).set_ai_state(2);
                    self.sprite_slot_mut(k).set_a(0);
                }
                self.sprite_slot_mut(k).increment_subtype2();
                let j = usize::from(
                    (self.sprite_slot(k).subtype2() & 4) | self.sprite_slot(k).direction(),
                );
                self.sprite_slot_mut(k).set_graphics(KODONDO_GRAPHICS[j]);
                let value = (self.sprite_slot(k).oam_flags() & !0x40) | KODONDO_OAM_FLAGS[j];
                self.sprite_slot_mut(k).set_oam_flags(value);
            }
            2 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(0);
                }
                let j = u8::from(self.sprite_slot(k).delay_main().wrapping_sub(0x20) < 0x30);
                if j != 0 && (self.sprite_slot(k).delay_main() & 0x0f) == 0 {
                    self.kodongo_spawn_fire(k);
                }
                let value =
                    KODONDO_FLAME_GRAPHICS[usize::from(j * 4 + self.sprite_slot(k).direction())];
                self.sprite_slot_mut(k).set_graphics(value);
            }
            _ => {}
        }
    }

    fn kholdstare_check_collision_for_mothula(&mut self, k: usize) {
        let j = self.sprite_check_tile_collision(k);
        if (j & 3) != 0 {
            self.sprite_slot_mut(k).negate_x_velocity();
            self.sprite_slot_mut(k).negate_z_velocity();
        }
        if (j & 12) != 0 {
            self.sprite_slot_mut(k).negate_y_velocity();
            self.sprite_slot_mut(k).negate_z_subpixel();
        }
    }

    // void Sprite_MadBatterBolt(int k) {  // 9e8a96
    pub(super) fn sprite_mad_batter_bolt(&mut self, k: usize) {
        const X: [u16; 8] = [0, 4, 8, 12, 12, 4, 8, 0];
        const Y: [u16; 8] = [0, 4, 8, 12, 12, 4, 8, 0];

        if (self.sprite_slot(k).subtype2() & 16) != 0 {
            self.oam_allocate_from_region_b(4);
        }
        self.sprite_draw_single_small(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot(k).ai_state() == 0 {
            self.sprite_move_xy(k);
            if self.sprite_slot(k).delay_main() == 0 {
                self.sprite_slot_mut(k).set_ai_state(1);
            }
        } else {
            self.sprite_slot_mut(k).increment_ai_state();
            if self.sprite_slot(k).ai_state() == 0 {
                self.sprite_slot_mut(k).set_state(0);
            }
            self.sprite_slot_mut(k).increment_subtype2();
            let j = self.sprite_slot(k).subtype2();
            if (j & 7) == 0 {
                self.set_sound_effect_2(48);
            }
            self.sprite_set_x(
                k,
                self.game_state
                    .player
                    .follower_link
                    .x()
                    .wrapping_add(X[usize::from((j >> 2) & 7)]),
            );
            self.sprite_set_y(
                k,
                self.game_state
                    .player
                    .follower_link
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

        match self.sprite_slot(k).ai_state() {
            0 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).increment_ai_state();
                    self.sprite_slot_mut(k).add_c(1);
                    let j = if self.sprite_slot(k).c() == 4 {
                        self.sprite_slot_mut(k).set_c(0);
                        self.sprite_direction_to_face_link(k, None)
                    } else {
                        self.get_random_number() & 3
                    };
                    let j = usize::from(j);
                    self.sprite_slot_mut(k)
                        .set_x_velocity(FLUTE_BOY_ANIMAL_X_VELOCITIES[j] as u8);
                    self.sprite_slot_mut(k)
                        .set_y_velocity(ZAZAK_Y_VELOCITIES[j] as u8);
                    let value = (self.get_random_number() & 7) + 19;
                    self.sprite_slot_mut(k).set_z_velocity(value);
                }
                self.sprite_slot_mut(k).increment_subtype2();
                let value = (self.sprite_slot(k).subtype2() >> 3) & 1;
                self.sprite_slot_mut(k).set_graphics(value);
            }
            1 => {
                self.sprite_move_xyz(k);
                self.sprite_check_tile_collision(k);
                self.sprite_slot_mut(k).subtract_z_velocity(2);
                if sign8(self.sprite_slot(k).z()) {
                    self.sprite_slot_mut(k).set_z(0);
                    self.sprite_slot_mut(k).set_z_velocity(0);
                    let mut pt = PointU8 { x: 0, y: 0 };
                    self.sprite_direction_to_face_link(k, Some(&mut pt));
                    if pt.x.wrapping_add(48) < 96 && pt.y.wrapping_add(48) < 96 {
                        self.sprite_slot_mut(k).increment_ai_state();
                        let pp = self.sprite_project_speed_towards_link(k, 31);
                        self.sprite_slot_mut(k).set_direction(
                            ZeldaState::sprite_convert_velocity_to_angle(pp.x, pp.y) >> 1,
                        );
                        self.sprite_slot_mut(k).set_delay_main(95);
                        return;
                    }
                    self.sprite_slot_mut(k).set_ai_state(0);
                    self.sprite_slot_mut(k).set_delay_main(16);
                }
                self.sprite_slot_mut(k).increment_subtype2();
                let value = (self.sprite_slot(k).subtype2() >> 3) & 1;
                self.sprite_slot_mut(k).set_graphics(value);
            }
            2 => {
                let mut j = self.sprite_slot(k).delay_main();
                if j == 0 {
                    self.sprite_slot_mut(k).set_ai_state(0);
                    self.sprite_slot_mut(k).set_delay_main(16);
                    self.sprite_slot_mut(k).set_a(0);
                    self.sprite_slot_mut(k).set_b(0);
                    self.sprite_slot_mut(k).set_g(0);
                    return;
                }
                j >>= 2;
                self.sprite_slot_mut(k)
                    .set_graphics(PIKIT_GRAPHICS[usize::from(j)]);
                let dir = usize::from(self.sprite_slot(k).direction());
                let xo = PIKIT_TONGUE_XY_OFFSETS[usize::from(j + PIKIT_TONGUE_X_OFFSET_BASES[dir])];
                let yo = PIKIT_TONGUE_XY_OFFSETS[usize::from(j + PIKIT_TONGUE_Y_OFFSET_BASES[dir])];
                self.sprite_slot_mut(k).set_a(xo as u8);
                self.sprite_slot_mut(k).set_b(yo as u8);
                let x_delta = self
                    .game_state
                    .sprites
                    .workspace
                    .current_sprite_x()
                    .wrapping_add(xo as i16 as u16)
                    .wrapping_sub(self.game_state.player.follower_link.x())
                    .wrapping_add(12);
                let y_delta = self
                    .game_state
                    .sprites
                    .workspace
                    .current_sprite_y()
                    .wrapping_add(yo as i16 as u16)
                    .wrapping_sub(self.game_state.player.follower_link.y())
                    .wrapping_add(12);
                if self.sprite_slot(k).g() == 0
                    && x_delta < 24
                    && y_delta < 32
                    && self.sprite_slot(k).delay_main() < 46
                {
                    let sfx = self.link_calculate_sfx_pan() | 0x26;
                    self.set_sound_effect_1(sfx);
                    let loot = (self.get_random_number() & 3) + 1;
                    self.sprite_slot_mut(k).set_g(loot);
                    self.sprite_slot_mut(k).set_e(loot);
                    match loot {
                        1 => {
                            if self.game_state.inventory.player_resources.bombs() != 0 {
                                self.player_resources_mut().decrement_bombs();
                            } else {
                                self.sprite_slot_mut(k).set_g(0);
                            }
                        }
                        2 => {
                            if self.game_state.inventory.player_resources.arrows() != 0 {
                                self.player_resources_mut().decrement_arrows();
                            } else {
                                self.sprite_slot_mut(k).set_g(0);
                            }
                        }
                        3 => {
                            let rupees = self.game_state.inventory.player_resources.rupees_goal();
                            if rupees != 0 {
                                self.player_resources_mut().subtract_rupees_goal(1);
                            } else {
                                self.sprite_slot_mut(k).set_g(0);
                            }
                        }
                        _ => {
                            let value = self.game_state.inventory.items.shield_type();
                            self.sprite_slot_mut(k).set_subtype(value);
                            if self.game_state.inventory.items.shield_type() != 0
                                && self.game_state.inventory.items.shield_type() != 3
                            {
                                self.inventory_items_mut().set_shield_type(0);
                            } else {
                                self.sprite_slot_mut(k).set_g(0);
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
        if self.sprite_slot(k).a() != 0 {
            self.sprite_stalfos_bone(k);
            return;
        }
        if self.sprite_slot(k).e() == 0 {
            self.stalfos_skellington(k);
            return;
        }
        if self.sprite_slot(k).delay_main() == 0 {
            self.sprite_slot_mut(k).set_x_velocity(1);
            self.sprite_slot_mut(k).set_y_velocity(1);
            if self.sprite_check_tile_collision(k) != 0 {
                self.sprite_slot_mut(k).set_state(0);
                return;
            }
            self.sprite_slot_mut(k).set_e(0);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x15);
            self.sprite_spawn_poof_garnish(k);
            self.sprite_slot_mut(k).set_delay_aux2(8);
            self.sprite_slot_mut(k).set_delay_main(64);
            self.sprite_slot_mut(k).set_y_velocity(0);
            self.sprite_slot_mut(k).set_x_velocity(0);
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
        if self.sprite_slot(k).state() == 9
            && self
                .game_state
                .player
                .follower_link
                .x()
                .wrapping_sub(self.game_state.sprites.workspace.current_sprite_x())
                .wrapping_add(40)
                < 80
            && self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_sub(self.game_state.sprites.workspace.current_sprite_y())
                .wrapping_add(48)
                < 80
            && !self
                .game_state
                .player
                .follower_link
                .has_disabled_oam_offsets()
            && (self.sprite_slot(k).z() | self.sprite_slot(k).pause()) == 0
            && self.sprite_slot(k).floor()
                == self.game_state.player.follower_link.lower_level_state()
        {
            let dir = self.sprite_direction_to_face_link(k, None);
            let mut should_jump = false;
            let mut may_check_dir = true;
            if !self.game_state.player.follower_link.is_running() {
                if self.game_state.player.follower_link.button_b_frames() >= 0x90 {
                    should_jump = true;
                } else if !sign8(
                    self.game_state
                        .player
                        .follower_link
                        .button_b_frames()
                        .wrapping_sub(9),
                ) {
                    may_check_dir = false;
                }
            }
            let facing = self.game_state.player.follower_link.facing_index() & 3;
            if may_check_dir && (should_jump || dir != STALFOS_CHECK_DIRECTIONS[facing]) {
                self.sprite_slot_mut(k).set_direction(dir);
                let pt = self.sprite_project_speed_towards_link(k, 32);
                self.sprite_slot_mut(k).set_x_velocity(pt.x.wrapping_neg());
                self.sprite_slot_mut(k).set_y_velocity(pt.y.wrapping_neg());
                self.sprite_slot_mut(k).set_z_velocity(32);
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x13);
                self.sprite_slot_mut(k).add_z(1);
            }
        }

        if self.sprite_slot(k).z() == 0 {
            self.sprite_zazak_main(k);
            return;
        }
        let value =
            STALFOS_DIRECTION_ANIMATION_STATES[usize::from(self.sprite_slot(k).direction())];
        self.sprite_slot_mut(k).set_graphics(value);
        self.stalfos_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot(k).f() != 0 {
            self.sprite_slot_mut(k).set_z_velocity(0);
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        let t = self.sprite_check_tile_collision(k);
        if (t & 3) != 0 {
            self.sprite_slot_mut(k).set_x_velocity(0);
        }
        if (t & 12) != 0 {
            self.sprite_slot_mut(k).set_y_velocity(0);
        }
        self.sprite_move_xyz(k);
        self.sprite_slot_mut(k).subtract_z_velocity(2);
        if sign8(self.sprite_slot(k).z().wrapping_sub(1)) {
            self.sprite_slot_mut(k).set_z(0);
            self.sprite_zero_velocity_xy(k);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x21);
            if self.sprite_slot(k).subtype() != 0 {
                self.sprite_slot_mut(k).set_delay_aux3(16);
                self.sprite_slot_mut(k).set_subtype2(0);
            }
        }
    }

    // void Sprite_Zazak_Main(int k) {  // 9e919f
    pub(super) fn sprite_zazak_main(&mut self, k: usize) {
        if self.sprite_slot(k).b() != 0 {
            self.fire_phlegm_draw(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            let value = (self.game_state.frame.frame_counter >> 1) & 1;
            self.sprite_slot_mut(k).set_graphics(value);
            self.sprite_check_damage_to_link(k);
            self.sprite_move_xy(k);
            if self.sprite_check_tile_collision(k) != 0 {
                self.sprite_slot_mut(k).set_state(0);
                self.sprite_place_rupulse_spark_2(k);
            }
            return;
        }

        let t = self.sprite_slot(k).delay_aux3();
        let trace_stalfos_head = std::env::var_os("ZELDA3_TRACE_STALFOS_HEAD").is_some()
            && self.sprite_slot(k).sprite_type() == 0xa7
            && k == 0
            && self.game_state.world.location.dungeon_room() == 0x00a8;
        if t != 0 {
            let old_head = self.sprite_slot(k).head_direction();
            self.sprite_slot_mut(k).set_ai_state(0);
            self.sprite_slot_mut(k).set_delay_main(32);
            self.sprite_zero_velocity_xy(k);
            let face = self.sprite_direction_to_face_link(k, None);
            let value = face;
            self.sprite_slot_mut(k).set_head_direction(value);
            if trace_stalfos_head {
                eprintln!(
                    "R stalfos head aux3 fc={} t=0x{:02x} x=0x{:04x} y=0x{:04x} old=0x{:02x} face=0x{:02x} d=0x{:02x} c=0x{:02x} delay=0x{:02x} rng=0x{:02x}",
                    self.game_state.frame.frame_counter,
                    t,
                    self.sprite_get_x(k),
                    self.sprite_get_y(k),
                    old_head,
                    face,
                    self.sprite_slot(k).direction(),
                    self.sprite_slot(k).c(),
                    self.sprite_slot(k).delay_main(),
                    self.game_state.world.region.rng_seed(),
                );
            }
        }
        if t == 1 {
            self.stalfos_throw_bone(k);
            self.sprite_slot_mut(k).set_subtype2(1);
        }
        let value = STALFOS_PRIMARY_ANIMATION_STATES[usize::from(
            (self.sprite_slot(k).subtype2() & 1) * 4 + self.sprite_slot(k).direction(),
        )];
        self.sprite_slot_mut(k).set_graphics(value);
        if self.sprite_slot(k).sprite_type() == 0xa7 {
            self.stalfos_draw(k);
        } else {
            self.zazak_draw(k);
        }
        if std::env::var_os("ZELDA3_TRACE_STALFOS_INACTIVE").is_some()
            && self.sprite_slot(k).sprite_type() == 0xa7
            && self.game_state.world.location.dungeon_room() == 0x00a8
        {
            eprintln!(
                "R stalfos inactive-check fc={} k={} x=0x{:04x} y=0x{:04x} state=0x{:02x} flag=0x{:02x} sub=0x{:02x} defl=0x{:02x} pause=0x{:02x} delay=0x{:02x} ai=0x{:02x} z=0x{:02x} f=0x{:02x} xr=0x{:02x} yr=0x{:02x} bump=0x{:02x}",
                self.game_state.frame.frame_counter,
                k,
                self.sprite_get_x(k),
                self.sprite_get_y(k),
                self.sprite_slot(k).state(),
                self.game_state.frame.modal_pause_flag,
                self.game_state.frame.submodule,
                self.sprite_slot(k).deflection_bits(),
                self.sprite_slot(k).pause(),
                self.sprite_slot(k).delay_main(),
                self.sprite_slot(k).ai_state(),
                self.sprite_slot(k).z(),
                self.sprite_slot(k).f(),
                self.sprite_slot(k).x_recoil(),
                self.sprite_slot(k).y_recoil(),
                self.sprite_slot(k).bump_damage(),
            );
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        let trace_stalfos = std::env::var_os("ZELDA3_TRACE_STALFOS").is_some()
            && self.sprite_slot(k).sprite_type() == 0xa7
            && self.game_state.world.location.dungeon_room() == 0x00a8;
        if trace_stalfos {
            eprintln!(
                "R stalfos pre-move fc={} k={} x=0x{:04x} y=0x{:04x} d=0x{:02x} head=0x{:02x} ai=0x{:02x} delay=0x{:02x} g=0x{:02x} wall=0x{:02x} xv=0x{:02x} yv=0x{:02x} z=0x{:02x} zv=0x{:02x}",
                self.game_state.frame.frame_counter,
                k,
                self.sprite_get_x(k),
                self.sprite_get_y(k),
                self.sprite_slot(k).direction(),
                self.sprite_slot(k).head_direction(),
                self.sprite_slot(k).ai_state(),
                self.sprite_slot(k).delay_main(),
                self.sprite_slot(k).g(),
                self.sprite_slot(k).wall_collision(),
                self.sprite_slot(k).x_velocity(),
                self.sprite_slot(k).y_velocity(),
                self.sprite_slot(k).z(),
                self.sprite_slot(k).z_velocity(),
            );
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xy(k);
        let trace_tile = self.sprite_check_tile_collision(k);
        if trace_stalfos {
            eprintln!(
                "R stalfos post-move fc={} k={} x=0x{:04x} y=0x{:04x} d=0x{:02x} head=0x{:02x} ai=0x{:02x} delay=0x{:02x} g=0x{:02x} wall=0x{:02x} tile=0x{:02x} xv=0x{:02x} yv=0x{:02x}",
                self.game_state.frame.frame_counter,
                k,
                self.sprite_get_x(k),
                self.sprite_get_y(k),
                self.sprite_slot(k).direction(),
                self.sprite_slot(k).head_direction(),
                self.sprite_slot(k).ai_state(),
                self.sprite_slot(k).delay_main(),
                self.sprite_slot(k).g(),
                self.sprite_slot(k).wall_collision(),
                trace_tile,
                self.sprite_slot(k).x_velocity(),
                self.sprite_slot(k).y_velocity(),
            );
        }

        match self.sprite_slot(k).ai_state() {
            0 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    let old_delay = self.sprite_slot(k).delay_main();
                    let rng_before = self.game_state.world.region.rng_seed();
                    let rng = self.get_random_number();
                    self.sprite_slot_mut(k)
                        .set_delay_main(STALFOS_HOP_DELAYS[usize::from(rng & 3)]);
                    self.sprite_slot_mut(k).set_ai_state(1);
                    let j = usize::from(self.sprite_slot(k).head_direction());
                    self.sprite_slot_mut(k).set_direction(j as u8);
                    self.sprite_slot_mut(k)
                        .set_x_velocity(FLUTE_BOY_ANIMAL_X_VELOCITIES[j] as u8);
                    self.sprite_slot_mut(k)
                        .set_y_velocity(ZAZAK_Y_VELOCITIES[j] as u8);
                    if std::env::var_os("ZELDA3_TRACE_STALFOS_DELAY").is_some()
                        && self.sprite_slot(k).sprite_type() == 0xa7
                        && self.game_state.world.location.dungeon_room() == 0x00a8
                    {
                        eprintln!(
                            "R stalfos delay fc={} k={} x=0x{:04x} y=0x{:04x} old_delay=0x{:02x} new_delay=0x{:02x} head=0x{:02x} d=0x{:02x} ai=0x{:02x} rng_before=0x{:02x} rng=0x{:02x}",
                            self.game_state.frame.frame_counter,
                            k,
                            self.sprite_get_x(k),
                            self.sprite_get_y(k),
                            old_delay,
                            self.sprite_slot(k).delay_main(),
                            self.sprite_slot(k).head_direction(),
                            self.sprite_slot(k).direction(),
                            self.sprite_slot(k).ai_state(),
                            rng_before,
                            rng,
                        );
                    }
                }
            }
            1 => {
                if self.sprite_slot(k).wall_collision() != 0 {
                    self.sprite_slot_mut(k).set_delay_main(16);
                } else if self.sprite_slot(k).delay_main() != 0 {
                    self.sprite_slot_mut(k).decrement_g();
                    if sign8(self.sprite_slot(k).g()) {
                        self.sprite_slot_mut(k).set_g(11);
                        self.sprite_slot_mut(k).increment_subtype2();
                    }
                    return;
                } else if self.sprite_slot(k).sprite_type() == 0xa6
                    && self.sprite_slot(k).direction()
                        == self.sprite_direction_to_face_link(k, None)
                    && self.sprite_slot(k).floor()
                        == self.game_state.player.follower_link.lower_level_state()
                {
                    self.sprite_slot_mut(k).set_ai_state(2);
                    self.sprite_slot_mut(k).set_delay_main(48);
                    self.sprite_slot_mut(k).set_delay_aux1(48);
                    self.sprite_slot_mut(k).set_y_velocity(0);
                    self.sprite_slot_mut(k).set_x_velocity(0);
                    return;
                } else {
                    self.sprite_slot_mut(k).set_delay_main(32);
                }
                let old_head = self.sprite_slot(k).head_direction();
                let rng_before = self.game_state.world.region.rng_seed();
                let rng = self.get_random_number();
                let rng_bit = rng & 1;
                let value = ZAZAK_ALT_DIRECTIONS
                    [usize::from(self.sprite_slot(k).direction() * 2 + rng_bit)];
                self.sprite_slot_mut(k).set_head_direction(value);
                if trace_stalfos_head {
                    eprintln!(
                        "R stalfos head random fc={} x=0x{:04x} y=0x{:04x} old=0x{:02x} new=0x{:02x} d=0x{:02x} c_before=0x{:02x} rng_before=0x{:02x} rng=0x{:02x} bit=0x{:02x} delay=0x{:02x} wall=0x{:02x}",
                        self.game_state.frame.frame_counter,
                        self.sprite_get_x(k),
                        self.sprite_get_y(k),
                        old_head,
                        self.sprite_slot(k).head_direction(),
                        self.sprite_slot(k).direction(),
                        self.sprite_slot(k).c(),
                        rng_before,
                        rng,
                        rng_bit,
                        self.sprite_slot(k).delay_main(),
                        self.sprite_slot(k).wall_collision(),
                    );
                }
                self.sprite_slot_mut(k).set_ai_state(0);
                self.sprite_slot_mut(k).add_c(1);
                if self.sprite_slot(k).c() == 4 {
                    self.sprite_slot_mut(k).set_c(0);
                    let old_head = self.sprite_slot(k).head_direction();
                    let face = self.sprite_direction_to_face_link(k, None);
                    let value = face;
                    self.sprite_slot_mut(k).set_head_direction(value);
                    if trace_stalfos_head {
                        eprintln!(
                            "R stalfos head face4 fc={} x=0x{:04x} y=0x{:04x} old=0x{:02x} face=0x{:02x} d=0x{:02x} c=0x{:02x} delay=0x{:02x} rng=0x{:02x}",
                            self.game_state.frame.frame_counter,
                            self.sprite_get_x(k),
                            self.sprite_get_y(k),
                            old_head,
                            face,
                            self.sprite_slot(k).direction(),
                            self.sprite_slot(k).c(),
                            self.sprite_slot(k).delay_main(),
                            self.game_state.world.region.rng_seed(),
                        );
                    }
                    self.sprite_slot_mut(k).set_delay_main(24);
                }
                self.sprite_slot_mut(k).set_y_velocity(0);
                self.sprite_slot_mut(k).set_x_velocity(0);
            }
            2 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(0);
                } else if self.sprite_slot(k).delay_main() == 24 {
                    self.sprite_spawn_fire_phlegm(k);
                }
            }
            _ => {}
        }
    }

    pub(super) fn sprite_83_green_eyegore(&mut self, k: usize) {
        if self.sprite_slot(k).b() == 0 {
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
        if self.sprite_slot(k).delay_aux1() == 8 {
            self.sprite_spawn_fire_phlegm(k);
        }
        if self.game_state.player.follower_link.defense_flags() != 0
            || (self.game_state.player.follower_link.joypad1h_last() & 0x0f) == 0
        {
            self.sprite_slot_mut(k).set_a(0);
            self.sprite_check_damage_to_and_from_link(k);
            self.sprite_check_tile_collision(k);
            return;
        }

        let j = usize::from(
            (self.game_state.player.follower_link.joypad1h_last() & 0x0f)
                | if self.sprite_slot(k).sprite_type() == 0x84 {
                    16
                } else {
                    0
                },
        );
        self.sprite_slot_mut(k).set_direction(GORIYA_DIRECTIONS[j]);
        self.sprite_slot_mut(k)
            .set_x_velocity(GORIYA_X_VELOCITIES[j] as u8);
        self.sprite_slot_mut(k)
            .set_y_velocity(GORIYA_Y_VELOCITIES[j] as u8);
        if self.sprite_slot(k).wall_collision() == 0 {
            self.sprite_move_xy(k);
        }
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_check_tile_collision(k);
        let gfx_idx = usize::from(
            self.sprite_slot(k).subtype2().wrapping_add(1) & 12 | self.sprite_slot(k).direction(),
        );
        self.sprite_slot_mut(k).increment_subtype2();
        self.sprite_slot_mut(k)
            .set_graphics(GORIYA_GRAPHICS[gfx_idx]);

        if self.sprite_slot(k).sprite_type() == 0x84 {
            let mut pt = PointU8 { x: 0, y: 0 };
            let dir = self.sprite_direction_to_face_link(k, Some(&mut pt));
            if (pt.x.wrapping_add(8) < 16 || pt.y.wrapping_add(8) < 16)
                && self.sprite_slot(k).direction() == dir
            {
                if self.sprite_slot(k).a() & 0x1f == 0 {
                    self.sprite_slot_mut(k).set_delay_aux1(16);
                }
                self.sprite_slot_mut(k).add_a(1);
                return;
            }
        }
        self.sprite_slot_mut(k).set_a(0);
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
        self.sprite_slot_mut(k).or_flags3(64);
        self.sprite_slot_mut(k).or_deflection_bits(4);

        match self.sprite_slot(k).ai_state() {
            0 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    let mut pt = PointU8 { x: 0, y: 0 };
                    self.sprite_direction_to_face_link(k, Some(&mut pt));
                    if pt.x.wrapping_add(48) < 96 && pt.y.wrapping_add(48) < 96 {
                        self.sprite_slot_mut(k).increment_ai_state();
                        self.sprite_slot_mut(k).set_delay_main(63);
                    }
                }
            }
            1 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    let value = self.sprite_direction_to_face_link(k, None);
                    self.sprite_slot_mut(k).set_direction(value);
                    self.sprite_slot_mut(k).set_ai_state(2);
                    let value = EYEGORE_OPENING_DELAYS[usize::from(self.get_random_number() & 3)];
                    self.sprite_slot_mut(k).set_delay_main(value);
                } else {
                    let value = EYEGORE_OPENING_GRAPHICS
                        [usize::from(self.sprite_slot(k).delay_main() >> 3)];
                    self.sprite_slot_mut(k).set_graphics(value);
                }
            }
            2 => {
                self.sprite_slot_mut(k).and_flags3(!0x40);
                if self.sprite_slot(k).sprite_type() != 0x84 {
                    self.sprite_slot_mut(k).and_deflection_bits(!4);
                }
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_delay_main(63);
                    self.sprite_slot_mut(k).increment_ai_state();
                    self.sprite_slot_mut(k).set_graphics(0);
                } else {
                    if ((k as u8) ^ self.game_state.frame.frame_counter) & 31 == 0 {
                        let value = self.sprite_direction_to_face_link(k, None);
                        self.sprite_slot_mut(k).set_direction(value);
                    }
                    let j = usize::from(self.sprite_slot(k).direction());
                    self.sprite_slot_mut(k)
                        .set_x_velocity(FLUTE_BOY_ANIMAL_X_VELOCITIES[j] as u8);
                    self.sprite_slot_mut(k)
                        .set_y_velocity(ZAZAK_Y_VELOCITIES[j] as u8);
                    if self.sprite_slot(k).wall_collision() == 0 {
                        self.sprite_move_xy(k);
                    }
                    self.sprite_check_tile_collision(k);
                    let gfx_idx = usize::from(
                        self.sprite_slot(k).subtype2().wrapping_add(1) & 12
                            | self.sprite_slot(k).direction(),
                    );
                    self.sprite_slot_mut(k).increment_subtype2();
                    self.sprite_slot_mut(k)
                        .set_graphics(EYEGORE_CHASING_GRAPHICS[gfx_idx]);
                }
            }
            3 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(0);
                    self.sprite_slot_mut(k).set_delay_main(96);
                } else {
                    let value = EYEGORE_CLOSING_GRAPHICS
                        [usize::from(self.sprite_slot(k).delay_main() >> 3)];
                    self.sprite_slot_mut(k).set_graphics(value);
                }
            }
            _ => {}
        }
    }

    // void Sprite_A8_GreenZirro(int k) {  // 9e8dd2
    pub(super) fn sprite_a8_green_zirro(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_object_priority(0x30);
        if self.sprite_slot(k).a() != 0 {
            match self.sprite_slot(k).ai_state() {
                0 => {
                    self.sprite_draw_single_small(k);
                    if self.sprite_return_if_inactive(k) {
                        return;
                    }
                    self.sprite_move_xy(k);
                    self.sprite_move_z(k);
                    self.sprite_slot_mut(k).subtract_z_velocity(2);
                    if sign8(self.sprite_slot(k).z()) {
                        self.sprite_slot_mut(k).set_z(0);
                        self.sprite_slot_mut(k).increment_ai_state();
                        self.sprite_slot_mut(k).set_delay_main(19);
                        self.sprite_slot_mut(k).add_flags2(1);
                        self.sprite_sfx_queue_sfx2_with_pan(k, 0x0c);
                    }
                }
                1 => {
                    self.sprite_draw_zirro_bomb(k);
                    if self.sprite_return_if_inactive(k) {
                        return;
                    }
                    if (self.game_state.frame.frame_counter & 3) == 0 {
                        self.sprite_slot_mut(k).add_delay_main(1);
                    }
                    self.sprite_check_damage_to_link(k);
                }
                _ => {}
            }
            return;
        }

        if self.sprite_slot(k).delay_aux1() != 0 {
            let value = BOMBER_GRAPHICS_BY_DIRECTION[usize::from(self.sprite_slot(k).direction())];
            self.sprite_slot_mut(k).set_graphics(value);
        }
        self.sprite_slot_mut(k).or_object_priority(0x30);
        self.bomber_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        if self.sprite_slot(k).delay_aux1() == 8 {
            self.zirro_drop_bomb(k);
        }
        self.sprite_check_damage_to_and_from_link(k);
        if (self.game_state.frame.frame_counter & 1) == 0 {
            let j = self.sprite_slot(k).g() & 1;
            let value =
                self.sprite_slot(k)
                    .z_velocity()
                    .wrapping_add(if j != 0 { 0xff } else { 1 });
            self.sprite_slot_mut(k).set_z_velocity(value);
            if self.sprite_slot(k).z_velocity() == if j != 0 { (-8i8) as u8 } else { 8 } {
                self.sprite_slot_mut(k).add_g(1);
            }
        }
        self.sprite_move_z(k);
        let mut pt = PointU8 { x: 0, y: 0 };
        self.sprite_direction_to_face_link(k, Some(&mut pt));
        if pt.x.wrapping_add(40) < 80
            && pt.y.wrapping_add(40) < 80
            && !self
                .game_state
                .player
                .follower_link
                .has_disabled_oam_offsets()
            && (self.game_state.player.follower_link.is_running()
                || sign8(
                    self.game_state
                        .player
                        .follower_link
                        .button_b_frames()
                        .wrapping_sub(9),
                ))
        {
            let pp = self.sprite_project_speed_towards_link(k, 0x30);
            self.sprite_slot_mut(k).set_x_velocity(pp.x.wrapping_neg());
            self.sprite_slot_mut(k).set_y_velocity(pp.y.wrapping_neg());
            self.sprite_slot_mut(k).set_delay_main(8);
            self.sprite_slot_mut(k).set_ai_state(2);
        }

        match self.sprite_slot(k).ai_state() {
            0 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).increment_ai_state();
                    self.sprite_slot_mut(k).add_b(1);
                    let j = if self.sprite_slot(k).b() == 3 {
                        self.sprite_slot_mut(k).set_b(0);
                        self.sprite_slot_mut(k).set_delay_main(48);
                        BOMBER_SHOT_GRAPHICS_BY_DIRECTION
                            [usize::from(self.sprite_direction_to_face_link(k, None))]
                    } else {
                        let r = self.get_random_number();
                        self.sprite_slot_mut(k).set_delay_main((r & 0x1f) | 0x20);
                        r & 7
                    };
                    self.sprite_slot_mut(k)
                        .set_x_velocity(BOMBER_X_VELOCITIES[usize::from(j)] as u8);
                    self.sprite_slot_mut(k)
                        .set_y_velocity(BOMBER_Y_VELOCITIES[usize::from(j)] as u8);
                }
                self.green_zirro_set_dir_for_mothula(k);
            }
            1 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(0);
                    self.sprite_slot_mut(k).set_delay_main(10);
                    if self.sprite_slot(k).sprite_type() == 0xa8 {
                        self.sprite_slot_mut(k).set_delay_aux1(16);
                    }
                } else {
                    self.sprite_move_xy(k);
                    self.green_zirro_set_dir_for_mothula(k);
                }
            }
            2 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(0);
                }
                self.sprite_slot_mut(k).add_subtype2(2);
                self.sprite_move_xy(k);
                self.green_zirro_set_dir_for_mothula(k);
            }
            _ => {}
        }
    }

    fn green_zirro_set_dir_for_mothula(&mut self, k: usize) {
        let value = self.sprite_direction_to_face_link(k, None);
        self.sprite_slot_mut(k).set_direction(value);
        let value = (self.sprite_slot(k).direction() << 1)
            | ((self.sprite_slot(k).subtype2().wrapping_add(1) >> 3) & 1);
        self.sprite_slot_mut(k).set_graphics(value);
        self.sprite_slot_mut(k).increment_subtype2();
    }

    // void Zirro_DropBomb(int k) {  // 9e8f81
    pub(super) fn zirro_drop_bomb(&mut self, k: usize) {
        let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
        let j = self.sprite_spawn_dynamically(k, 0xa8, &mut info);
        if j >= 0 {
            let j = j as usize;
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
            self.sprite_slot_mut(j).set_z(info.r4_z);
            let i = usize::from(self.sprite_slot(j).direction());
            self.sprite_set_x(
                j,
                info.r0_x
                    .wrapping_add(BOMBER_PELLET_X_OFFSETS[i] as i16 as u16),
            );
            self.sprite_set_y(
                j,
                info.r2_y
                    .wrapping_add(BOMBER_PELLET_Y_OFFSETS[i] as i16 as u16),
            );
            self.sprite_slot_mut(j)
                .set_x_velocity(FLUTE_BOY_ANIMAL_X_VELOCITIES[i] as u8);
            self.sprite_slot_mut(j)
                .set_y_velocity(ZAZAK_Y_VELOCITIES[i] as u8);
            self.sprite_slot_mut(j).set_a(1);
            self.sprite_slot_mut(j).set_ignore_projectile(1);
            self.sprite_slot_mut(j).set_flags4(9);
            self.sprite_slot_mut(j).set_flags3(0x33);
            self.sprite_slot_mut(j).set_oam_flags(0x33 & 15);
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
        if self.game_state.world.location.is_outdoors() {
            self.sprite_slot_mut(k).set_x_velocity(255);
            self.sprite_slot_mut(k).set_subtype(255);
            if self.sprite_check_tile_collision(k) == 0 {
                return;
            }
            if self.sprite_return_if_inactive(k) {
                return;
            }
            self.sprite_slot_mut(k).set_sprite_type(0x19);
            self.sprite_prep_load_properties(k);
            self.sprite_slot_mut(k).add_e(1);
            let value = self.sprite_slot(k).x_low().wrapping_add(8);
            self.sprite_slot_mut(k).set_x_low(value);
            self.sprite_slot_mut(k).subtract_y_low(8);
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x19);
            self.sprite_slot_mut(k).set_deflection_bits(0x80);
        } else {
            if self.sprite_return_if_inactive(k) {
                return;
            }
            self.sprite_slot_mut(k).increment_subtype2();
            if (self.sprite_slot(k).subtype2() & 0x7f) == 0
                && self.sprite_slot(k).floor()
                    == self.game_state.player.follower_link.lower_level_state()
            {
                let j = self.sprite_spawn_fireball(k);
                if j >= 0 {
                    let j = j as usize;
                    self.sprite_slot_mut(j).or_deflection_bits(8);
                    self.sprite_slot_mut(j).set_bump_damage(4);
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
        if self.sprite_slot(k).delay_main() == 24 {
            let j = self.sprite_spawn_fireball(k);
            if j >= 0 {
                let j = j as usize;
                self.sprite_slot_mut(j).or_deflection_bits(8);
                self.sprite_slot_mut(j).set_bump_damage(4);
                let i = usize::from(self.sprite_direction_to_face_link(j, None));
                self.sprite_slot_mut(j)
                    .set_x_velocity(FIREBALL_JUNCTION_AXIS_VELOCITIES[i + 2] as u8);
                self.sprite_slot_mut(j)
                    .set_y_velocity(FIREBALL_JUNCTION_AXIS_VELOCITIES[i] as u8);
                self.sprite_set_x(
                    j,
                    self.sprite_get_x(j)
                        .wrapping_add(FIREBALL_JUNCTION_X_OFFSETS[i] as i16 as u16),
                );
                self.sprite_set_y(
                    j,
                    self.sprite_get_y(j)
                        .wrapping_add(FIREBALL_JUNCTION_Y_OFFSETS[i] as i16 as u16),
                );
            }
        } else if self.sprite_slot(k).delay_main() == 0
            && self.game_state.player.follower_link.button_b_frames() != 0
            && self.sprite_slot(k).floor()
                == self.game_state.player.follower_link.lower_level_state()
        {
            self.sprite_slot_mut(k).set_delay_main(32);
        }
    }

    // void Sprite_C3_Gibo(int k) {  // 9dcce1
    pub(super) fn sprite_c3_gibo(&mut self, k: usize) {
        if self.sprite_slot(k).b() != 0 {
            self.sprite_draw_single_large(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            self.sprite_check_damage_to_and_from_link(k);
            self.sprite_slot_mut(k).increment_subtype2();
            let value = (self.sprite_slot(k).oam_flags() & 0x3f)
                | GIBO_OAM_FLAGS[usize::from((self.sprite_slot(k).subtype2() >> 2) & 3)];
            self.sprite_slot_mut(k).set_oam_flags(value);
            if self.sprite_slot(k).delay_main() != 0 {
                self.sprite_move_xy(k);
                self.sprite_bounce_from_tile_collision(k);
            }
            return;
        }

        self.gibo_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_slot_mut(k).increment_anim_clock();
        let mut j = usize::from(self.sprite_slot(k).head_direction());
        if self.sprite_slot(j).state() == 6 {
            let value = self.sprite_slot(j).state();
            self.sprite_slot_mut(k).set_state(value);
            let value = self.sprite_slot(j).delay_main();
            self.sprite_slot_mut(k).set_delay_main(value);
            self.sprite_slot_mut(k).add_flags2(4);
            return;
        }
        let value = (self.game_state.frame.frame_counter >> 3) & 3;
        self.sprite_slot_mut(k).set_subtype2(value);
        if (self.game_state.frame.frame_counter & 63) == 0 {
            let value = self.sprite_is_right_of_link(k).a << 2;
            self.sprite_slot_mut(k).set_direction(value);
        }
        self.sprite_check_damage_to_link(k);

        match self.sprite_slot(k).ai_state() {
            0 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).increment_ai_state();
                    self.sprite_slot_mut(k).set_delay_main(48);
                    self.sprite_slot_mut(k).add_a(1);
                    let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
                    let spawned = self.sprite_spawn_dynamically(k, 0xc3, &mut info);
                    if spawned >= 0 {
                        j = spawned as usize;
                        self.sprite_set_spawned_coordinates(j, &info);
                        let value = j as u8;
                        self.sprite_slot_mut(k).set_head_direction(value);
                        self.sprite_slot_mut(j).set_flags2(1);
                        self.sprite_slot_mut(j).set_b(1);
                        self.sprite_slot_mut(j).set_flags3(16);
                        let value = self.sprite_slot(k).g();
                        self.sprite_slot_mut(j).set_health(value);
                        self.sprite_slot_mut(j).set_oam_flags(7);
                        self.sprite_slot_mut(j).set_delay_main(48);
                        self.sprite_slot_mut(k).add_c(1);
                        let i = if self.sprite_slot(k).c() == 3 {
                            self.sprite_slot_mut(k).set_c(0);
                            self.sprite_direction_to_face_link(k, None)
                        } else {
                            self.get_random_number() & 7
                        };
                        let i = usize::from(i);
                        self.sprite_slot_mut(j)
                            .set_x_velocity(GIBO_X_VELOCITIES[i] as u8);
                        self.sprite_slot_mut(j)
                            .set_y_velocity(GIBO_Y_VELOCITIES[i] as u8);
                    }
                } else if self.sprite_slot(k).delay_main() == 32 {
                    self.sprite_slot_mut(k).set_delay_aux1(32);
                }
            }
            1 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).increment_ai_state();
                }
            }
            2 => {
                if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0 {
                    let x = self.sprite_get_x(j);
                    let y = self.sprite_get_y(j);
                    if self
                        .game_state
                        .sprites
                        .workspace
                        .current_sprite_x()
                        .wrapping_sub(x)
                        .wrapping_add(2)
                        < 4
                        && self
                            .game_state
                            .sprites
                            .workspace
                            .current_sprite_y()
                            .wrapping_sub(y)
                            .wrapping_add(2)
                            < 4
                    {
                        j = usize::from(self.sprite_slot(k).head_direction());
                        self.sprite_slot_mut(j).set_state(0);
                        self.sprite_slot_mut(k).set_a(0);
                        self.sprite_slot_mut(k).set_ai_state(0);
                        let value = self.sprite_slot(j).health();
                        self.sprite_slot_mut(k).set_g(value);
                        let value = (self.get_random_number() & 31) + 32;
                        self.sprite_slot_mut(k).set_delay_main(value);
                        return;
                    }
                    let pt = self.sprite_project_speed_towards_location(k, x, y, 16);
                    self.sprite_slot_mut(k).set_x_velocity(pt.x);
                    self.sprite_slot_mut(k).set_y_velocity(pt.y);
                }
                self.sprite_move_xy(k);
            }
            _ => {}
        }
    }

    // void Sprite_Tektite(int k) {  // 9dc293
    pub(super) fn sprite_tektite(&mut self, k: usize) {
        if self.sprite_slot(k).delay_aux1() != 0 {
            self.sprite_slot_mut(k).set_graphics(0);
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
        self.sprite_slot_mut(k).decrement_z_velocity();
        if sign8(self.sprite_slot(k).z()) {
            self.sprite_slot_mut(k).set_z(0);
            self.sprite_slot_mut(k).set_z_velocity(0);
        }

        match self.sprite_slot(k).ai_state() {
            0 => {
                let mut pt = PointU8 { x: 0, y: 0 };
                let mut j = self.sprite_direction_to_face_link(k, Some(&mut pt));
                if pt.x.wrapping_add(40) < 80
                    && pt.y.wrapping_add(40) < 80
                    && !self
                        .game_state
                        .player
                        .follower_link
                        .has_disabled_oam_offsets()
                    && (self.sprite_slot(k).z() | self.sprite_slot(k).pause()) == 0
                    && self.game_state.player.follower_link.lower_level_state()
                        == self.sprite_slot(k).floor()
                    && j != TEKITE_DIRECTIONS[self.game_state.player.follower_link.facing_index()]
                {
                    let pt = self.sprite_project_speed_towards_link(k, 32);
                    self.sprite_slot_mut(k).set_x_velocity(pt.x.wrapping_neg());
                    self.sprite_slot_mut(k).set_y_velocity(pt.y.wrapping_neg());
                    self.sprite_slot_mut(k).set_z_velocity(16);
                    self.sprite_slot_mut(k).set_ai_state(1);
                    return;
                }
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).increment_ai_state();
                    self.sprite_slot_mut(k).add_b(1);
                    if self.sprite_slot(k).b() == 4 {
                        self.sprite_slot_mut(k).set_b(0);
                        self.sprite_slot_mut(k).increment_ai_state();
                        let value = (self.get_random_number() & 63).wrapping_add(48);
                        self.sprite_slot_mut(k).set_delay_main(value);
                        self.sprite_slot_mut(k).set_z_velocity(12);
                        j = self.sprite_is_below_link(k).a * 2 + self.sprite_is_right_of_link(k).a;
                    } else {
                        let value = (self.get_random_number() & 7).wrapping_add(24);
                        self.sprite_slot_mut(k).set_z_velocity(value);
                        j = self.get_random_number() & 3;
                    }
                    self.sprite_slot_mut(k)
                        .set_x_velocity(TEKITE_X_VELOCITIES[usize::from(j)] as u8);
                    self.sprite_slot_mut(k)
                        .set_y_velocity(TEKITE_Y_VELOCITIES[usize::from(j)] as u8);
                } else {
                    let value = (self.sprite_slot(k).delay_main() >> 4) & 1;
                    self.sprite_slot_mut(k).set_graphics(value);
                }
            }
            1 => {
                if self.sprite_slot(k).z() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(0);
                    let value = (self.get_random_number() & 63).wrapping_add(72);
                    self.sprite_slot_mut(k).set_delay_main(value);
                    self.sprite_slot_mut(k).set_y_velocity(0);
                    self.sprite_slot_mut(k).set_x_velocity(0);
                } else {
                    self.sprite_slot_mut(k).set_graphics(2);
                }
            }
            2 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(0);
                    let value = (self.get_random_number() & 63).wrapping_add(72);
                    self.sprite_slot_mut(k).set_delay_main(value);
                    self.sprite_slot_mut(k).set_y_velocity(0);
                    self.sprite_slot_mut(k).set_x_velocity(0);
                    return;
                }
                if self.sprite_slot(k).z() == 0 {
                    self.sprite_slot_mut(k).set_z_velocity(12);
                    self.sprite_slot_mut(k).add_z(1);
                    self.sprite_slot_mut(k).set_delay_aux1(8);
                }
                self.sprite_slot_mut(k).set_graphics(2);
            }
            _ => {}
        }
    }

    // void Sprite_C9_Tektite(int k) {  // 9dc275
    pub(super) fn sprite_c9_tektite(&mut self, k: usize) {
        let j = self.sprite_slot(k).anim_clock();
        if j != 0 {
            self.sprite_slot_mut(k).set_ignore_projectile(j);
            self.sprite_slot_mut(k).set_object_priority(0x30);
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
        const FLOPPING_FISH_X_VELOCITIES: [i8; 8] = [0, 12, 16, 12, 0, -12, -16, -12];
        const FLOPPING_FISH_Y_VELOCITIES: [i8; 8] = [-16, -12, 0, 12, 16, 12, 0, -12];
        const FLOPPING_FISH_A_TARGET_BY_DIRECTION: [u8; 2] = [2, 0];
        const FLOPPING_FISH_AIR_GRAPHICS: [u8; 3] = [1, 5, 3];
        const FLOPPING_FISH_GROUND_GRAPHICS: [u8; 17] =
            [5, 5, 6, 6, 5, 5, 4, 4, 3, 7, 7, 8, 8, 7, 7, 8, 8];

        if self.game_state.sprites.system.chr_halfslot_state() < 3 {
            self.fish_draw(k);
        }
        if self.sprite_slot(k).state() == 10 {
            self.sprite_slot_mut(k).set_ai_state(4);
            let value = ((self.game_state.frame.frame_counter >> 4) & 1) + 3;
            self.sprite_slot_mut(k).set_graphics(value);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }

        match self.sprite_slot(k).ai_state() {
            0 => {
                self.sprite_check_tile_collision(k);
                if self.game_state.sprites.workspace.tile_type() == 8 {
                    self.sprite_slot_mut(k).set_state(0);
                } else {
                    self.sprite_slot_mut(k).set_ai_state(1);
                }
            }
            1 => {
                self.sprite_check_if_lifted_permissive(k);
                self.sprite_bounce_from_tile_collision(k);
                self.sprite_move_xyz(k);
                self.sprite_slot_mut(k).subtract_z_velocity(2);
                if sign8(self.sprite_slot(k).z()) {
                    self.sprite_slot_mut(k).set_z(0);
                    if self.game_state.sprites.workspace.tile_type() == 9 {
                        self.sprite_spawn_small_splash(k);
                    } else if self.game_state.sprites.workspace.tile_type() == 8 {
                        self.sprite_slot_mut(k).set_state(0);
                        self.sprite_spawn_small_splash(k);
                    }
                    let value = (self.get_random_number() & 15).wrapping_add(16);
                    self.sprite_slot_mut(k).set_z_velocity(value);
                    let j = usize::from(self.get_random_number() & 7);
                    self.sprite_slot_mut(k)
                        .set_x_velocity(FLOPPING_FISH_X_VELOCITIES[j] as u8);
                    self.sprite_slot_mut(k)
                        .set_y_velocity(FLOPPING_FISH_Y_VELOCITIES[j] as u8);
                    let value = self.sprite_slot(k).direction().wrapping_add(1);
                    self.sprite_slot_mut(k).set_direction(value);
                    self.sprite_slot_mut(k).set_subtype2(3);
                }
                self.sprite_slot_mut(k).increment_subtype2();
                if (self.sprite_slot(k).subtype2() & 7) == 0 {
                    let j = usize::from(self.sprite_slot(k).direction() & 1);
                    if self.sprite_slot(k).a() != FLOPPING_FISH_A_TARGET_BY_DIRECTION[j] {
                        let value =
                            self.sprite_slot(k)
                                .a()
                                .wrapping_add(if j != 0 { 0xff } else { 1 });
                        self.sprite_slot_mut(k).set_a(value);
                    }
                }
                let a = usize::from(self.sprite_slot(k).a());
                let value = FLOPPING_FISH_AIR_GRAPHICS[a]
                    + ((self.game_state.frame.frame_counter >> 3) & 1);
                self.sprite_slot_mut(k).set_graphics(value);
            }
            2 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(3);
                    self.sprite_slot_mut(k).set_z_velocity(48);
                    self.sprite_spawn_small_splash(k);
                }
            }
            3 => {
                self.sprite_move_z(k);
                self.sprite_slot_mut(k).subtract_z_velocity(2);
                if self.sprite_slot(k).z_velocity() == 0 && self.sprite_slot(k).a() != 0 {
                    self.dialogue_message_index_mut().set_value(0x176);
                    self.sprite_show_message_minimal_c();
                }
                if sign8(self.sprite_slot(k).z()) {
                    self.sprite_slot_mut(k).set_z(0);
                    self.sprite_spawn_small_splash(k);
                    if self.sprite_slot(k).a() != 0 {
                        let mut info = SpriteSpawnInfo::default();
                        let j = self.sprite_spawn_dynamically(k, 0xdb, &mut info);
                        if j >= 0 {
                            let j = j as usize;
                            self.sprite_set_spawned_coordinates(j, &info);
                            self.sprite_set_x(j, info.r0_x.wrapping_add(4));
                            self.sprite_slot_mut(j).set_stunned(255);
                            self.sprite_slot_mut(j).set_z_velocity(48);
                            self.sprite_slot_mut(j).set_delay_aux3(48);
                            self.sprite_apply_speed_towards_link(j, 16);
                        }
                    }
                    self.sprite_slot_mut(k).set_state(0);
                }
                self.sprite_slot_mut(k).increment_subtype2();
                let idx = usize::from(self.sprite_slot(k).subtype2() >> 2);
                self.sprite_slot_mut(k)
                    .set_graphics(FLOPPING_FISH_GROUND_GRAPHICS[idx]);
            }
            4 => {
                if self.sprite_slot(k).z() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(1);
                }
                self.sprite_move_xy(k);
                self.thrown_sprite_tile_and_sprite_interaction(k);
            }
            _ => {}
        }
    }

    // void Sprite_81_Hover(int k) {  // 9ecc02
    pub(super) fn sprite_81_hover(&mut self, k: usize) {
        self.sprite_slot_mut(k).or_object_priority(48);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot(k).f() != 0 {
            self.sprite_slot_mut(k).set_ai_state(0);
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        if self.sprite_slot(k).wall_collision() == 0 {
            self.sprite_move_xy(k);
        }
        self.sprite_check_tile_collision(k);
        self.sprite_slot_mut(k).increment_subtype2();
        let value = (self.sprite_slot(k).subtype2() >> 3) & 2;
        self.sprite_slot_mut(k).set_graphics(value);

        match self.sprite_slot(k).ai_state() {
            0 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(1);
                    let j = self.sprite_is_right_of_link(k).a + self.sprite_is_below_link(k).a * 2;
                    self.sprite_slot_mut(k).set_direction(j);
                    let value =
                        (self.sprite_slot(k).oam_flags() & !0x40) | HOVER_OAM_FLAGS[usize::from(j)];
                    self.sprite_slot_mut(k).set_oam_flags(value);
                    let value = (self.get_random_number() & 15).wrapping_add(12);
                    self.sprite_slot_mut(k).set_delay_main(value);
                    self.sprite_zero_velocity_xy(k);
                }
            }
            1 => {
                let j = usize::from(self.sprite_slot(k).direction());
                if self.sprite_slot(k).delay_main() != 0 {
                    let value = self
                        .sprite_slot(k)
                        .x_velocity()
                        .wrapping_add(HOVER_PRIMARY_X_ACCELERATIONS[j] as u8);
                    self.sprite_slot_mut(k).set_x_velocity(value);
                    let value = self
                        .sprite_slot(k)
                        .y_velocity()
                        .wrapping_add(HOVER_PRIMARY_Y_ACCELERATIONS[j] as u8);
                    self.sprite_slot_mut(k).set_y_velocity(value);
                    let value = (self.sprite_slot(k).subtype2() >> 3) & 1;
                    self.sprite_slot_mut(k).set_graphics(value);
                } else {
                    let value = self
                        .sprite_slot(k)
                        .x_velocity()
                        .wrapping_add(HOVER_SECONDARY_X_ACCELERATIONS[j] as u8);
                    self.sprite_slot_mut(k).set_x_velocity(value);
                    let value = self
                        .sprite_slot(k)
                        .y_velocity()
                        .wrapping_add(HOVER_SECONDARY_Y_ACCELERATIONS[j] as u8);
                    self.sprite_slot_mut(k).set_y_velocity(value);
                    if self.sprite_slot(k).y_velocity() == 0 {
                        self.sprite_slot_mut(k).set_ai_state(0);
                        self.sprite_slot_mut(k).set_delay_main(64);
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
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0
            && (self.sprite_slot(k).x_velocity() | self.sprite_slot(k).y_velocity()) != 0
        {
            let value = ZeldaState::sprite_convert_velocity_to_angle(
                self.sprite_slot(k).x_velocity(),
                self.sprite_slot(k).y_velocity(),
            ) & 0x0f;
            self.sprite_slot_mut(k).set_direction(value);
        }
        self.sprite_move_xyz(k);
        self.sprite_slot_mut(k).subtract_z_velocity(2);
        if sign8(self.sprite_slot(k).z()) {
            self.sprite_slot_mut(k).set_z(0);
            self.sprite_slot_mut(k).set_z_velocity(0);
        }
        let cur_x = self.sprite_get_x(k);
        let cur_y = self.sprite_get_y(k);
        self.sprite_workspace_mut().set_current_sprite_x(cur_x);
        self.sprite_workspace_mut().set_current_sprite_y(cur_y);
        let x = u16::from(self.sprite_slot(k).a()) | (u16::from(self.sprite_slot(k).b()) << 8);
        let y = u16::from(self.sprite_slot(k).c()) | (u16::from(self.sprite_slot(k).g()) << 8);
        self.sprite_slot_mut(k).set_anim_clock(u8::from(
            cur_x.wrapping_sub(x).wrapping_add(48) < 96
                && cur_y.wrapping_sub(y).wrapping_add(48) < 96,
        ));

        match self.sprite_slot(k).ai_state() {
            0 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).increment_subtype2();
                    if self.sprite_slot(k).subtype2() == 4 {
                        self.sprite_slot_mut(k).set_subtype2(0);
                        self.sprite_slot_mut(k).set_ai_state(2);
                        let j = usize::from(self.get_random_number() & 15);
                        self.sprite_slot_mut(k)
                            .set_x_velocity((CHAIN_CHOMP_X_VELOCITIES[j] << 2) as u8);
                        self.sprite_slot_mut(k)
                            .set_y_velocity((CHAIN_CHOMP_Y_VELOCITIES[j] << 2) as u8);
                        self.get_random_number();
                        self.sprite_apply_speed_towards_link(k, 64);
                        self.sprite_sfx_queue_sfx3_with_pan(k, 0x4);
                    } else {
                        let value = (self.get_random_number() & 31).wrapping_add(16);
                        self.sprite_slot_mut(k).set_delay_main(value);
                        let j = usize::from(self.get_random_number() & 15);
                        self.sprite_slot_mut(k)
                            .set_x_velocity(CHAIN_CHOMP_X_VELOCITIES[j] as u8);
                        self.sprite_slot_mut(k)
                            .set_y_velocity(CHAIN_CHOMP_Y_VELOCITIES[j] as u8);
                        self.sprite_slot_mut(k).set_ai_state(1);
                    }
                } else {
                    self.sprite_slot_mut(k).set_x_velocity(0);
                    self.sprite_slot_mut(k).set_y_velocity(0);
                }
            }
            1 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_delay_main(32);
                    self.sprite_slot_mut(k).set_ai_state(0);
                }
                if (self.sprite_slot(k).delay_main() & 15) == 0 {
                    self.chain_chomp_move_chain(k);
                }
                if self.sprite_slot(k).z() == 0 {
                    self.sprite_slot_mut(k).set_z_velocity(16);
                }
                if self.sprite_slot(k).anim_clock() == 0 {
                    let x = u16::from(self.sprite_slot(k).a())
                        | (u16::from(self.sprite_slot(k).b()) << 8);
                    let y = u16::from(self.sprite_slot(k).c())
                        | (u16::from(self.sprite_slot(k).g()) << 8);
                    let pt = self.sprite_project_speed_towards_location(k, x, y, 16);
                    self.sprite_slot_mut(k).set_x_velocity(pt.x);
                    self.sprite_slot_mut(k).set_y_velocity(pt.y);
                    self.sprite_move_xy(k);
                    self.sprite_slot_mut(k).set_delay_main(12);
                }
            }
            2 => {
                if self.sprite_slot(k).anim_clock() == 0 {
                    self.sprite_slot_mut(k).negate_x_velocity();
                    self.sprite_slot_mut(k).negate_y_velocity();
                    self.sprite_move_xy(k);
                    self.sprite_slot_mut(k).set_x_velocity(0);
                    self.sprite_slot_mut(k).set_y_velocity(0);
                    self.sprite_slot_mut(k).set_ai_state(3);
                    self.sprite_slot_mut(k).set_delay_aux1(48);
                }
                self.chain_chomp_move_chain(k);
                self.chain_chomp_move_chain(k);
            }
            3 => {
                if self.sprite_slot(k).delay_aux1() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(0);
                    self.sprite_slot_mut(k).set_delay_main(48);
                }
                self.chain_chomp_move_chain(k);
                self.chain_chomp_move_chain(k);
            }
            _ => {}
        }
    }

    // void Sprite_C7_Pokey(int k) {  // 9dc64f
    pub(super) fn sprite_c7_pokey(&mut self, k: usize) {
        if self.sprite_slot(k).c() != 0 {
            self.sprite_draw_single_large(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            self.sprite_check_damage_to_and_from_link(k);
            self.sprite_move_xyz(k);
            self.sprite_slot_mut(k).subtract_z_velocity(2);
            if sign8(self.sprite_slot(k).z()) {
                self.sprite_slot_mut(k).set_z_velocity(16);
                self.sprite_slot_mut(k).set_z(0);
            }
            if self.sprite_bounce_from_tile_collision(k) != 0 {
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x21);
            }
            if self.sprite_slot(k).g() >= 3 {
                self.sprite_slot_mut(k).set_state(6);
                self.sprite_slot_mut(k).set_delay_main(10);
                self.sprite_slot_mut(k).set_flags5(0);
                self.sprite_sfx_queue_sfx2_with_pan(k, 0x1e);
            }
            return;
        }

        self.hokbok_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot(k).a() != 0 && self.sprite_slot(k).f() == 15 {
            self.sprite_slot_mut(k).set_f(6);
            let value = self
                .sprite_slot(k)
                .z()
                .wrapping_add(self.sprite_slot(k).b());
            self.sprite_slot_mut(k).set_z(value);
            self.sprite_slot_mut(k).subtract_a(1);
            if self.sprite_slot(k).a() == 0 {
                self.sprite_slot_mut(k).set_health(17);
            }
            let value = if sign8(self.sprite_slot(k).x_velocity()) {
                self.sprite_slot(k).x_velocity().wrapping_sub(4)
            } else {
                self.sprite_slot(k).x_velocity().wrapping_add(4)
            };
            self.sprite_slot_mut(k).set_x_velocity(value);
            let value = if sign8(self.sprite_slot(k).y_velocity()) {
                self.sprite_slot(k).y_velocity().wrapping_sub(4)
            } else {
                self.sprite_slot(k).y_velocity().wrapping_add(4)
            };
            self.sprite_slot_mut(k).set_y_velocity(value);

            let mut info = crate::zelda_rtl::sprite::SpriteSpawnInfo::default();
            let j = self.sprite_spawn_dynamically(k, 0xc7, &mut info);
            if j >= 0 {
                let j = j as usize;
                self.sprite_set_spawned_coordinates(j, &info);
                self.sprite_slot_mut(j).set_c(1);
                self.sprite_slot_mut(j).set_health(1);
                let value = self.sprite_slot(k).x_recoil();
                self.sprite_slot_mut(j).set_x_velocity(value);
                let value = self.sprite_slot(k).y_recoil();
                self.sprite_slot_mut(j).set_y_velocity(value);
                self.sprite_slot_mut(j).set_deflection_bits(64);
            }
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        match self.sprite_slot(k).ai_state() {
            0 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(1);
                    self.sprite_slot_mut(k).set_z_velocity(16);
                } else {
                    let value =
                        HOKBOK_SEGMENT_STATES[usize::from(self.sprite_slot(k).delay_main() >> 1)];
                    self.sprite_slot_mut(k).set_b(value);
                }
            }
            1 => {
                self.sprite_move_xyz(k);
                self.sprite_slot_mut(k).subtract_z_velocity(2);
                if sign8(self.sprite_slot(k).z()) {
                    self.sprite_slot_mut(k).set_z(0);
                    self.sprite_slot_mut(k).set_ai_state(0);
                    self.sprite_slot_mut(k).set_delay_main(15);
                }
                self.sprite_bounce_from_tile_collision(k);
            }
            _ => {}
        }
    }

    // void Sprite_C2_Boulder(int k) {  // 9dcfcb
    pub(super) fn sprite_c2_boulder(&mut self, k: usize) {
        if self.game_state.world.location.is_outdoors() {
            self.boulder_outdoors_main(k);
            return;
        }
        if self.game_state.sprites.system.chr_halfslot_state() < 3 {
            self.sprite_draw_single_small(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let value = (self.game_state.frame.frame_counter << 2) & 0xc0;
        self.sprite_slot_mut(k).set_oam_flags(value);
        self.sprite_move_xyz(k);
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) != 0 {
            return;
        }
        let cur_x = self.game_state.sprites.workspace.current_sprite_x();
        let cur_y = self.game_state.sprites.workspace.current_sprite_y();
        let link_x = self.game_state.player.follower_link.x();
        let link_y = self.game_state.player.follower_link.y();
        if cur_x.wrapping_sub(link_x).wrapping_add(4) < 16
            && cur_y.wrapping_sub(link_y).wrapping_sub(4) < 12
        {
            self.sprite_attempt_damage_to_link_plus_recoil(k);
        }
        if self.sprite_check_tile_collision(k) != 0 {
            self.sprite_slot_mut(k).set_state(0);
        }
    }

    // void Boulder_OutdoorsMain(int k) {  // 9dd02a
    pub(super) fn boulder_outdoors_main(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_object_priority(0x30);
        self.boulder_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        let value = self
            .sprite_slot(k)
            .subtype2()
            .wrapping_sub(self.sprite_slot(k).direction());
        self.sprite_slot_mut(k).set_subtype2(value);
        self.sprite_check_damage_to_and_from_link(k);
        self.sprite_move_xyz(k);
        self.sprite_slot_mut(k).subtract_z_velocity(2);
        if sign8(self.sprite_slot(k).z()) {
            self.sprite_slot_mut(k).set_z(0);
            let mut j = usize::from(self.sprite_check_tile_collision(k) != 0);
            self.sprite_slot_mut(k)
                .set_z_velocity(BOULDER_Z_VELOCITIES[j] as u8);
            self.sprite_slot_mut(k)
                .set_y_velocity(BOULDER_Y_VELOCITIES[j] as u8);
            j += usize::from(self.get_random_number() & 1) * 2;
            self.sprite_slot_mut(k)
                .set_x_velocity(BOULDER_X_VELOCITIES[j] as u8);
            self.sprite_slot_mut(k)
                .set_direction((((j & 2) as u8).wrapping_sub(1)) as u8);
            self.sprite_sfx_queue_sfx2_with_pan(k, 0xb);
        }
    }

    // void Sprite_9C_Zoro(int k) {  // 9e9bc8
    pub(super) fn sprite_9_c_zoro(&mut self, k: usize) {
        if self.sprite_slot(k).e() != 0 {
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

        match self.sprite_slot(k).ai_state() {
            0 => {
                self.sprite_slot_mut(k).increment_ai_state();
                self.sprite_slot_mut(k).set_delay_main(128);
                self.sprite_slot_mut(k).set_graphics(255);
            }
            1 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).increment_ai_state();
                    self.sprite_slot_mut(k).set_delay_main(55);
                }
            }
            2 => {
                let j = self.sprite_slot(k).delay_main();
                let i = usize::from(self.sprite_slot(k).direction());
                if j == 0 {
                    self.sprite_slot_mut(k).set_ai_state(3);
                    self.sprite_slot_mut(k)
                        .set_x_velocity(BABUSU_AXIS_VELOCITIES[i] as u8);
                    self.sprite_slot_mut(k)
                        .set_y_velocity(BABUSU_AXIS_VELOCITIES[i + 2] as u8);
                    self.sprite_slot_mut(k).set_delay_main(32);
                }
                if j >= 32 {
                    self.sprite_slot_mut(k).set_graphics(
                        BABUSU_GRAPHICS[usize::from((j - 32) >> 2)] + BABUSU_DIRECTION_GRAPHICS[i],
                    );
                } else {
                    self.sprite_slot_mut(k).set_graphics(0xff);
                }
            }
            3 => {
                self.sprite_check_damage_to_and_from_link(k);
                self.sprite_move_xy(k);
                let value = ((self.game_state.frame.frame_counter >> 1) & 1)
                    + BABUSU_SCURRY_GRAPHICS[usize::from(self.sprite_slot(k).direction())];
                self.sprite_slot_mut(k).set_graphics(value);
                if self.sprite_slot(k).delay_main() == 0 && self.sprite_check_tile_collision(k) != 0
                {
                    self.sprite_slot_mut(k).xor_direction(1);
                    self.sprite_slot_mut(k).set_ai_state(0);
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
        let bak = self.sprite_slot(k).oam_flags();
        let value = if ((self.sprite_slot(k).subtype2() >> 1) & 1) != 0 {
            0x44
        } else {
            4
        };
        self.sprite_slot_mut(k).set_oam_flags(value);
        self.sprite_slot_mut(k).and_object_priority(!0x0f);
        self.oam_allocate_from_region_c(self.sprite_slot(k).object_priority());
        self.sprite_draw_multiple(k, &D, None);
        self.sprite_slot_mut(k).set_oam_flags(bak);
    }

    // void Sprite_8C_Arrghus(int k) {  // 9eb433
    pub(super) fn sprite_8_c_arrghus(&mut self, k: usize) {
        self.sprite_slot_mut(k).or_object_priority(0x30);
        self.arrghus_draw(k);
        if self.sprite_slot(k).state() != 9 || self.sprite_slot(k).z() < 96 {
            if self.sprite_return_if_inactive(k) {
                return;
            }
        }

        self.arrghus_handle_puffs(k);
        self.game_state
            .sprites
            .overlord_slots
            .slot_mut(&mut self.ram, 4)
            .set_x_low(1);
        if (self.sprite_slot(k).hit_timer() & 127) == 2 {
            self.sprite_slot_mut(k).set_ai_state(3);
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x32);
            self.sprite_slot_mut(k).set_subtype2(0);
            self.sprite_slot_mut(k).set_flags3(64);
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }

        self.sprite_check_damage_to_link(k);
        self.sprite_slot_mut(k).increment_subtype2();
        if ((self.sprite_slot(k).subtype2().wrapping_sub(1)) & 3) == 0 {
            self.sprite_slot_mut(k).add_g(1);
            if self.sprite_slot(k).g() == 9 {
                self.sprite_slot_mut(k).set_g(0);
            }
            let value = ARRGHUS_GRAPHICS_BY_STEP[usize::from(self.sprite_slot(k).g())];
            self.sprite_slot_mut(k).set_graphics(value);
        }

        let collision = self.sprite_check_tile_collision(k);
        if collision != 0 {
            if self.sprite_slot(k).ai_state() == 5 {
                if (collision & 3) != 0 {
                    let value = (0u8).wrapping_sub(self.sprite_slot(k).x_velocity());
                    self.sprite_slot_mut(k).set_x_velocity(value);
                } else {
                    let value = (0u8).wrapping_sub(self.sprite_slot(k).y_velocity());
                    self.sprite_slot_mut(k).set_y_velocity(value);
                }
            } else {
                self.sprite_zero_velocity_xy(k);
            }
        }

        match self.sprite_slot(k).ai_state() {
            0 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(1);
                    self.sprite_slot_mut(k).set_delay_main(48);
                }
                self.sprite_move_xy(k);
                self.sprite_approach_target_speed(
                    k,
                    self.sprite_slot(k).head_direction(),
                    self.sprite_slot(k).direction(),
                );
            }
            1 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(0);
                    if !self.sprite_check_if_screen_is_clear() {
                        self.game_state
                            .sprites
                            .overlord_slots
                            .slot_mut(&mut self.ram, 3)
                            .add_x_low(1);
                        if self.game_state.sprites.overlord_slots.slot(3).x_low() == 4 {
                            self.game_state
                                .sprites
                                .overlord_slots
                                .slot_mut(&mut self.ram, 3)
                                .set_x_low(0);
                            self.sprite_slot_mut(k).set_ai_state(2);
                            self.sprite_slot_mut(k).set_delay_main(176);
                        } else {
                            let value = (self.get_random_number() & 63).wrapping_add(48);
                            self.sprite_slot_mut(k).set_delay_main(value);
                            let speed = (self.sprite_slot(k).delay_main() & 3).wrapping_add(8);
                            let pt = self.sprite_project_speed_towards_link(k, speed);
                            let value = pt.x;
                            self.sprite_slot_mut(k).set_head_direction(value);
                            self.sprite_slot_mut(k).set_direction(pt.y);
                        }
                    } else {
                        self.sprite_slot_mut(k).set_ai_state(3);
                        self.sprite_sfx_queue_sfx3_with_pan(k, 0x32);
                        self.sprite_slot_mut(k).set_subtype2(0);
                    }
                } else {
                    self.sprite_move_xy(k);
                    self.sprite_approach_target_speed(k, 0, 0);
                }
            }
            2 => {
                self.game_state
                    .sprites
                    .overlord_slots
                    .slot_mut(&mut self.ram, 4)
                    .set_x_low(8);
                if self.sprite_slot(k).delay_main() < 32 {
                    self.game_state
                        .sprites
                        .overlord_slots
                        .slot_mut(&mut self.ram, 2)
                        .subtract_x_low(1);
                    if sign8(self.game_state.sprites.overlord_slots.slot(2).x_low()) {
                        self.game_state
                            .sprites
                            .overlord_slots
                            .slot_mut(&mut self.ram, 2)
                            .set_x_low(0);
                        self.sprite_slot_mut(k).set_ai_state(1);
                        self.sprite_slot_mut(k).set_delay_main(112);
                    }
                } else if self.sprite_slot(k).delay_main() < 96 {
                    self.game_state
                        .sprites
                        .overlord_slots
                        .slot_mut(&mut self.ram, 2)
                        .add_x_low(1);
                } else if self.sprite_slot(k).delay_main() == 96 {
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x26);
                } else if (self.sprite_slot(k).delay_main() & 0x0f) == 0 {
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x06);
                }
            }
            3 => {
                self.sprite_slot_mut(k).set_z_velocity(120);
                self.sprite_move_z(k);
                if self.sprite_slot(k).z() >= 224 {
                    self.sprite_slot_mut(k).set_delay_main(64);
                    self.sprite_slot_mut(k).set_ai_state(4);
                    self.sprite_slot_mut(k).set_z_velocity(0);
                    let value = self.game_state.player.follower_link.x() as u8;
                    self.sprite_slot_mut(k).set_x_low(value);
                    let value = self.game_state.player.follower_link.y() as u8;
                    self.sprite_slot_mut(k).set_y_low(value);
                }
            }
            4 => {
                let mut a = self.sprite_slot(k).delay_main();
                if a == 0 {
                    self.sprite_slot_mut(k).set_z_velocity(144);
                    let old_z = self.sprite_slot(k).z();
                    self.sprite_move_z(k);
                    a = old_z ^ self.sprite_slot(k).z();
                    if sign8(a) {
                        a = self.sprite_slot(k).z();
                        if sign8(a) {
                            self.sprite_slot_mut(k).set_z(0);
                            self.sprite_spawn_big_splash(k);
                            self.sprite_slot_mut(k).set_ai_state(5);
                            self.sprite_slot_mut(k).set_delay_main(32);
                            self.sprite_sfx_queue_sfx3_with_pan(k, 0x03);
                            self.sprite_slot_mut(k).set_x_velocity(32);
                            self.sprite_slot_mut(k).set_y_velocity(32);
                        }
                    }
                }
                if a == 1 {
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
                }
            }
            5 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_flags3(0);
                    self.sprite_move_xy(k);
                    self.sprite_check_damage_from_link(k);
                    if (self.game_state.frame.frame_counter & 7) == 0 {
                        self.sprite_sfx_queue_sfx2_with_pan(k, 0x28);
                        let j =
                            self.garnish_alloc_limit(if sign8(self.sprite_slot(k).y_velocity()) {
                                29
                            } else {
                                14
                            });
                        if j >= 0 {
                            let j = j as usize;
                            let value = 21;
                            self.game_state
                                .sprites
                                .garnish_slots
                                .slot_mut(&mut self.ram, j)
                                .set_garnish_type(value);
                            self.garnish_state_mut().set_active_type(21);
                            let value = self.sprite_slot(k).x_low();
                            self.game_state
                                .sprites
                                .garnish_slots
                                .slot_mut(&mut self.ram, j)
                                .set_x_low(value);
                            let value = self.sprite_slot(k).x_high();
                            self.game_state
                                .sprites
                                .garnish_slots
                                .slot_mut(&mut self.ram, j)
                                .set_x_high(value);
                            let value = self.sprite_slot(k).y_low().wrapping_add(24);
                            self.game_state
                                .sprites
                                .garnish_slots
                                .slot_mut(&mut self.ram, j)
                                .set_y_low(value);
                            let value = self.sprite_slot(k).y_high();
                            self.game_state
                                .sprites
                                .garnish_slots
                                .slot_mut(&mut self.ram, j)
                                .set_y_high(value);
                            let value = 15;
                            self.game_state
                                .sprites
                                .garnish_slots
                                .slot_mut(&mut self.ram, j)
                                .set_countdown(value);
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
        let oam = self.game_state.oam.current_pointer_usize();
        let chr = self.sprite_slot(k).graphics().wrapping_mul(2);
        for i in 0..4 {
            let addr = oam + i * 4;
            let charnum = self.game_state.oam.entry_char(addr).wrapping_add(chr);
            self.oam_state_mut().set_entry_char(addr, charnum);
        }
        if self.sprite_slot(k).ai_state() == 5 {
            self.oam_state_mut().hide_entry(oam + 4 * 4);
        }
        if (self.sprite_slot(k).subtype2() & 8) != 0 {
            self.oam_state_mut().or_entry_flags(oam + 4 * 4, 0x40);
        }

        if self.sprite_slot(k).ai_state() != 5 {
            let cur = self.game_state.oam.current_pointer();
            self.oam_state_mut()
                .set_current_pointer(cur.wrapping_add(4));
            let ext = self.game_state.oam.current_extended_pointer();
            self.oam_state_mut()
                .set_current_extended_pointer(ext.wrapping_add(1));
            if self.sprite_slot(k).z() < 0xa0 {
                let bak = self.sprite_slot(k).oam_flags();
                self.sprite_slot_mut(k).and_oam_flags(!1);
                self.sprite_draw_big_shadow(k, 0);
                self.sprite_slot_mut(k).set_oam_flags(bak);
            }
        } else {
            self.sprite_draw_large_water_turbulence(k);
        }
    }

    // void Sprite_8D_Arrghi(int k) {  // 9eb8c4
    pub(super) fn sprite_8_d_arrghi(&mut self, k: usize) {
        self.sprite_slot_mut(k).or_object_priority(0x30);
        self.sprite_draw_single_large(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_slot_mut(k).increment_subtype2();
        let value = ARRGHI_GRAPHICS_BY_STEP[usize::from((self.sprite_slot(k).subtype2() >> 3) & 7)];
        self.sprite_slot_mut(k).set_graphics(value);

        if self.sprite_slot(k).b() != 0 {
            let j = usize::from(self.sprite_slot(k).b() - 1);
            if self.game_state.sprites.ancilla_slots.slot(j).ancilla_type() != 0 {
                let value = self.game_state.sprites.ancilla_slots.slot(j).x_low();
                self.sprite_slot_mut(k).set_x_low(value);
                let value = self.game_state.sprites.ancilla_slots.slot(j).x_high();
                self.sprite_slot_mut(k).set_x_high(value);
                let value = self.game_state.sprites.ancilla_slots.slot(j).y_low();
                self.sprite_slot_mut(k).set_y_low(value);
                let value = self.game_state.sprites.ancilla_slots.slot(j).y_high();
                self.sprite_slot_mut(k).set_y_high(value);
                self.sprite_slot_mut(k).set_oam_flags(5);
                self.sprite_slot_mut(k).and_flags3(!0x40);
                return;
            }
            self.sprite_slot_mut(k).set_ai_state(1);
            self.sprite_slot_mut(k).set_b(0);
            self.sprite_slot_mut(k).set_delay_main(32);
        }

        if self.sprite_slot(k).delay_main() == 0 {
            self.sprite_check_damage_to_link(k);
        }

        if self.sprite_slot(k).ai_state() == 0 {
            let home = self.arrghus_puff_home_position(k);
            let x_low = home.x_low();
            let x_high = home.x_high();
            let y_low = home.y_low();
            let y_high = home.y_high();
            self.sprite_slot_mut(k).set_x_low(x_low);
            self.sprite_slot_mut(k).set_x_high(x_high);
            self.sprite_slot_mut(k).set_y_low(y_low);
            self.sprite_slot_mut(k).set_y_high(y_high);
            return;
        }

        self.sprite_check_damage_from_link(k);
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) == 0 {
            let home = self.arrghus_puff_home_position(k);
            let x = home.x();
            let y = home.y();
            let x_low = home.x_low();
            let y_low = home.y_low();
            let pt = self.sprite_project_speed_towards_location(k, x, y, 4);
            self.sprite_slot_mut(k).set_y_velocity(pt.y);
            self.sprite_slot_mut(k).set_x_velocity(pt.x);
            if self
                .sprite_slot(k)
                .x_low()
                .wrapping_sub(x_low)
                .wrapping_add(8)
                < 16
                && self
                    .sprite_slot(k)
                    .y_low()
                    .wrapping_sub(y_low)
                    .wrapping_add(8)
                    < 16
            {
                self.sprite_slot_mut(k).set_ai_state(0);
                self.sprite_slot_mut(k).set_oam_flags(0x0d);
                self.sprite_slot_mut(k).or_flags3(0x40);
            }
        }
        self.sprite_move_xy(k);
    }

    // void Sprite_8F_Blob(int k) {  // 9eb002
    pub(super) fn sprite_8_f_blob(&mut self, k: usize) {
        if self.sprite_slot(k).state() == 9 && self.sprite_slot(k).e() != 0 {
            self.sprite_slot_mut(k).set_e(0);
            self.sprite_slot_mut(k).set_x_velocity(1);
            let collided = self.sprite_check_tile_collision(k);
            self.sprite_slot_mut(k).set_x_velocity(0);
            if collided != 0 {
                self.sprite_slot_mut(k).set_state(0);
                return;
            }
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
        }

        if self.sprite_slot(k).c() != 0 {
            self.sprite_slot_mut(k).set_object_priority(0x30);
        }
        self.zol_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }

        if self.sprite_slot(k).ai_state() >= 2 {
            self.sprite_check_damage_from_link(k);
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }

        match self.sprite_slot(k).ai_state() {
            0 => {
                let bak = self.sprite_slot(k).flags4();
                self.sprite_slot_mut(k).or_flags4(9);
                self.sprite_slot_mut(k).or_flags2(0x80);
                let hit_link = self.sprite_check_damage_to_link(k);
                self.sprite_slot_mut(k).set_flags4(bak);
                if hit_link {
                    self.sprite_slot_mut(k).increment_ai_state();
                    self.sprite_slot_mut(k).set_delay_main(127);
                    self.sprite_slot_mut(k).and_flags2(!0x80);
                    self.sprite_set_x(k, self.game_state.player.follower_link.x());
                    self.sprite_set_y(k, self.game_state.player.follower_link.y().wrapping_add(8));
                    self.sprite_slot_mut(k).set_delay_aux4(48);
                    self.sprite_slot_mut(k).set_ignore_projectile(0);
                }
            }
            1 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).increment_ai_state();
                    self.sprite_slot_mut(k).set_z_velocity(32);
                    self.sprite_apply_speed_towards_link(k, 16);
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x30);
                } else {
                    const POPPING_OUT_GFX: [u8; 16] =
                        [0, 1, 7, 7, 6, 6, 5, 5, 6, 6, 5, 5, 4, 4, 4, 4];
                    let value = POPPING_OUT_GFX[usize::from(self.sprite_slot(k).delay_main() >> 3)];
                    self.sprite_slot_mut(k).set_graphics(value);
                }
            }
            2 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_check_damage_from_link(k);
                    self.sprite_move_xy(k);
                    self.sprite_check_tile_collision(k);
                    let old_z = self.sprite_slot(k).z();
                    self.sprite_move_z(k);
                    if !sign8(self.sprite_slot(k).z_velocity().wrapping_add(64)) {
                        self.sprite_slot_mut(k).subtract_z_velocity(2);
                    }
                    if sign8(self.sprite_slot(k).z() ^ old_z) && sign8(self.sprite_slot(k).z()) {
                        self.sprite_slot_mut(k).set_z_velocity(0);
                        self.sprite_slot_mut(k).set_z(0);
                        self.sprite_slot_mut(k).set_c(0);
                        self.sprite_slot_mut(k).set_delay_main(31);
                        let value = 8;
                        self.sprite_slot_mut(k).set_head_direction(value);
                    }
                } else if self.sprite_slot(k).delay_main() == 1 {
                    self.sprite_slot_mut(k).set_delay_main(32);
                    self.sprite_slot_mut(k).increment_ai_state();
                    self.sprite_slot_mut(k).set_graphics(0);
                } else {
                    const FALLING_XVEL: [i8; 2] = [-8, 8];
                    const FALLING_GFX: [u8; 2] = [0, 1];
                    let value =
                        FALLING_GFX[usize::from((self.sprite_slot(k).delay_main() - 1) >> 4)];
                    self.sprite_slot_mut(k).set_graphics(value);
                    let value = FALLING_XVEL
                        [usize::from((self.game_state.frame.frame_counter >> 1) & 1)]
                        as u8;
                    self.sprite_slot_mut(k).set_x_velocity(value);
                    self.sprite_move_x(k);
                }
            }
            3 => {
                self.sprite_check_damage_to_link(k);
                if self.sprite_slot(k).delay_aux1() == 0 {
                    self.sprite_apply_speed_towards_link(k, 48);
                    let value = (self.get_random_number() & 63) | 96;
                    self.sprite_slot_mut(k).set_delay_aux1(value);
                    let value = (self.sprite_slot(k).oam_flags() & 0x3f)
                        | if sign8(self.sprite_slot(k).x_velocity()) {
                            0x40
                        } else {
                            0
                        };
                    self.sprite_slot_mut(k).set_oam_flags(value);
                }
                if self.sprite_slot(k).delay_aux2() == 0 {
                    self.sprite_slot_mut(k).increment_subtype2();
                    if ((self.sprite_slot(k).subtype2() & 14)
                        | self.sprite_slot(k).wall_collision())
                        == 0
                    {
                        self.sprite_move_xy(k);
                        self.sprite_slot_mut(k).add_g(1);
                        if self.sprite_slot(k).g() == self.sprite_slot(k).head_direction() {
                            self.sprite_slot_mut(k).set_g(0);
                            let value = (self.get_random_number() & 31).wrapping_add(64);
                            self.sprite_slot_mut(k).set_delay_aux2(value);
                            let value = (self.get_random_number() & 31) | 16;
                            self.sprite_slot_mut(k).set_head_direction(value);
                        }
                    }
                    self.sprite_check_tile_collision(k);
                    let value = (self.sprite_slot(k).subtype2() & 8) >> 3;
                    self.sprite_slot_mut(k).set_graphics(value);
                } else {
                    let value = if (self.sprite_slot(k).delay_aux2() & 0x10) != 0 {
                        1
                    } else {
                        0
                    };
                    self.sprite_slot_mut(k).set_graphics(value);
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
        if self.sprite_slot(k).delay_aux2() == 0 {
            self.sprite_check_damage_from_link(k);
        }
        self.terrorpin_check_for_hammer(k);
        self.sprite_move_xyz(k);

        match self.sprite_slot(k).b() {
            0 => {
                if self.sprite_slot(k).delay_aux4() == 0 {
                    let value = (self.get_random_number() & 31).wrapping_add(32);
                    self.sprite_slot_mut(k).set_delay_aux4(value);
                    let value = self.sprite_direction_to_face_link(k, None);
                    self.sprite_slot_mut(k).set_direction(value);
                }
                let j = usize::from(
                    self.sprite_slot(k)
                        .direction()
                        .wrapping_add(self.sprite_slot(k).g()),
                );
                self.sprite_slot_mut(k)
                    .set_x_velocity(TERRORPIN_X_VELOCITIES[j] as u8);
                self.sprite_slot_mut(k)
                    .set_y_velocity(TERRORPIN_Y_VELOCITIES[j] as u8);
                self.sprite_slot_mut(k).subtract_z_velocity(2);
                if sign8(self.sprite_slot(k).z()) {
                    self.sprite_slot_mut(k).set_z(0);
                    self.sprite_slot_mut(k).set_z_velocity(0);
                }
                let value = (self.game_state.frame.frame_counter
                    >> if self.sprite_slot(k).g() != 0 { 2 } else { 3 })
                    & 1;
                self.sprite_slot_mut(k).set_graphics(value);
                self.sprite_slot_mut(k).or_flags3(64);
                self.sprite_slot_mut(k).set_deflection_bits(4);
                self.sprite_check_damage_to_link(k);
            }
            1 => {
                self.sprite_slot_mut(k).and_flags3(191);
                self.sprite_slot_mut(k).set_deflection_bits(0);
                if self.sprite_slot(k).delay_aux4() == 0 {
                    self.sprite_slot_mut(k).set_b(0);
                    self.sprite_slot_mut(k).set_z_velocity(32);
                    self.sprite_slot_mut(k).set_delay_aux4(64);
                    return;
                }
                self.sprite_slot_mut(k).subtract_z_velocity(2);
                if sign8(self.sprite_slot(k).z()) {
                    self.sprite_slot_mut(k).set_z(0);
                    let t = (0u8).wrapping_sub(self.sprite_slot(k).z_velocity()) >> 1;
                    self.sprite_slot_mut(k)
                        .set_z_velocity(if t < 9 { 0 } else { t });
                    let value = (self.sprite_slot(k).x_velocity() as i8 >> 1) as u8;
                    self.sprite_slot_mut(k).set_x_velocity(value);
                    if self.sprite_slot(k).x_velocity() == 0xff {
                        self.sprite_slot_mut(k).set_x_velocity(0);
                    }
                    let value = (self.sprite_slot(k).y_velocity() as i8 >> 1) as u8;
                    self.sprite_slot_mut(k).set_y_velocity(value);
                    if self.sprite_slot(k).y_velocity() == 0xff {
                        self.sprite_slot_mut(k).set_y_velocity(0);
                    }
                }
                if self.sprite_slot(k).delay_aux4() < 64 {
                    let value = TERRORPIN_OVERTURNED_X_VELOCITIES
                        [usize::from((self.sprite_slot(k).delay_aux4() >> 1) & 1)]
                        as u8;
                    self.sprite_slot_mut(k).set_x_velocity(value);
                    self.sprite_slot_mut(k).increment_subtype2();
                }
                self.sprite_slot_mut(k).set_graphics(2);
                self.sprite_slot_mut(k).increment_subtype2();
                let value = (self.sprite_slot(k).oam_flags() & !0x40)
                    | TERRORPIN_OAM_FLAGS[usize::from((self.sprite_slot(k).subtype2() >> 3) & 1)];
                self.sprite_slot_mut(k).set_oam_flags(value);
            }
            _ => {}
        }
    }

    // void Terrorpin_CheckForHammer(int k) {  // 9eb3a3
    pub(super) fn terrorpin_check_for_hammer(&mut self, k: usize) {
        if (self.sprite_slot(k).z() | self.sprite_slot(k).delay_aux2()) == 0
            && self.sprite_slot(k).floor()
                == self.game_state.player.follower_link.lower_level_state()
            && !self
                .game_state
                .player
                .follower_link
                .has_disabled_oam_offsets()
            && self.game_state.player.follower_link.item_in_hand_has(0x0a)
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
                let value = (0u8).wrapping_sub(self.sprite_slot(k).x_velocity());
                self.sprite_slot_mut(k).set_x_velocity(value);
                let value = (0u8).wrapping_sub(self.sprite_slot(k).y_velocity());
                self.sprite_slot_mut(k).set_y_velocity(value);
                self.sprite_slot_mut(k).set_delay_aux2(32);
                self.sprite_slot_mut(k).set_z_velocity(32);
                self.sprite_slot_mut(k).set_g(4);
                self.sprite_slot_mut(k).xor_b(1);
                let value = if self.sprite_slot(k).b() != 0 {
                    0xff
                } else {
                    0x40
                };
                self.sprite_slot_mut(k).set_delay_aux4(value);
            }
        }
        let value = 0;
        self.sprite_slot_mut(k).set_head_direction(value);
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
        match self.sprite_slot(k).ai_state() {
            0 => {
                let value = GIBDO_GRAPHICS[self.sprite_slot(k).direction() as usize];
                self.sprite_slot_mut(k).set_graphics(value);
                if (self.game_state.frame.frame_counter & 7) == 0 {
                    let j = self.sprite_slot(k).a() as usize;
                    let delta = self
                        .sprite_slot(k)
                        .direction()
                        .wrapping_sub(GIBDO_DIRECTION_TARGETS[j]);
                    if delta != 0 {
                        let value = self
                            .sprite_slot(k)
                            .direction()
                            .wrapping_add(if sign8(delta) { 1 } else { 0xff });
                        self.sprite_slot_mut(k).set_direction(value);
                    } else {
                        let value = (self.get_random_number() & 31).wrapping_add(48);
                        self.sprite_slot_mut(k).set_delay_main(value);
                        self.sprite_slot_mut(k).set_ai_state(1);
                    }
                }
            }
            1 => {
                let j = self.sprite_slot(k).direction() as usize;
                self.sprite_slot_mut(k)
                    .set_x_velocity(GIBDO_AXIS_VELOCITIES[j + 2] as u8);
                self.sprite_slot_mut(k)
                    .set_y_velocity(GIBDO_AXIS_VELOCITIES[j] as u8);
                self.sprite_move_xy(k);
                self.sprite_check_tile_collision(k);
                let mut turned = false;
                if self.sprite_slot(k).delay_main() == 0
                    || self.sprite_slot(k).wall_collision() != 0
                {
                    let face = self.sprite_direction_to_face_link(k, None);
                    if face != self.sprite_slot(k).a() {
                        self.sprite_slot_mut(k).set_a(face);
                        self.sprite_slot_mut(k).set_ai_state(0);
                        turned = true;
                    }
                }
                if !turned {
                    self.sprite_slot_mut(k).subtract_b(1);
                    if sign8(self.sprite_slot(k).b()) {
                        self.sprite_slot_mut(k).set_b(14);
                        self.sprite_slot_mut(k).increment_subtype2();
                    }
                    let idx = (((self.sprite_slot(k).subtype2() & 1) << 2)
                        | self.sprite_slot(k).a()) as usize;
                    self.sprite_slot_mut(k)
                        .set_graphics(GIBDO_ALT_GRAPHICS[idx]);
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
        if (self.game_state.frame.frame_counter & 1) == 0 {
            self.sprite_slot_mut(k).xor_oam_flags(0x80);
        }
        self.sprite_move_xy(k);
        if self.sprite_slot(k).delay_main() == 0 && self.sprite_check_tile_collision(k) != 0 {
            self.sprite_slot_mut(k).set_state(0);
        }
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 3) != 0 {
            return;
        }
        for i in (0..=14usize).rev() {
            if self.game_state.sprites.garnish_slots.slot(i).is_empty() {
                let value = 2;
                self.game_state
                    .sprites
                    .garnish_slots
                    .slot_mut(&mut self.ram, i)
                    .set_garnish_type(value);
                self.garnish_state_mut().set_active_type(2);
                let value = self.sprite_slot(k).x_low();
                self.game_state
                    .sprites
                    .garnish_slots
                    .slot_mut(&mut self.ram, i)
                    .set_x_low(value);
                let value = self.sprite_slot(k).x_high();
                self.game_state
                    .sprites
                    .garnish_slots
                    .slot_mut(&mut self.ram, i)
                    .set_x_high(value);
                let value = self.sprite_slot(k).y_low();
                self.game_state
                    .sprites
                    .garnish_slots
                    .slot_mut(&mut self.ram, i)
                    .set_y_low(value);
                let value = self.sprite_slot(k).y_high();
                self.game_state
                    .sprites
                    .garnish_slots
                    .slot_mut(&mut self.ram, i)
                    .set_y_high(value);
                let value = 16;
                self.game_state
                    .sprites
                    .garnish_slots
                    .slot_mut(&mut self.ram, i)
                    .set_countdown(value);
                let value = k as u8;
                self.game_state
                    .sprites
                    .garnish_slots
                    .slot_mut(&mut self.ram, i)
                    .set_sprite(value);
                let value = self.sprite_slot(k).floor();
                self.game_state
                    .sprites
                    .garnish_slots
                    .slot_mut(&mut self.ram, i)
                    .set_floor(value);
                break;
            }
        }
    }

    // void Sprite_94_Tile(int k) {  // 9ebbb9
    pub(super) fn sprite_94_tile(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_object_priority(0x30);
        self.flying_tile_draw(k);
        if self.sprite_return_if_paused(k) {
            return;
        }
        if self.sprite_slot(k).hit_timer() != 0 {
            self.sprite_94_tile_break(k);
            return;
        }
        self.sprite_slot_mut(k).set_ignore_projectile(1);
        match self.sprite_slot(k).ai_state() {
            0 => {
                let y = u16::from(self.sprite_slot(k).y_low().wrapping_add(8))
                    | (u16::from(self.sprite_slot(k).y_high()) << 8);
                self.dungeon_update_tile_map_with_common_tile_for_mothula(
                    self.sprite_get_x(k),
                    y,
                    6,
                );
                self.sprite_slot_mut(k).set_ai_state(1);
                self.sprite_slot_mut(k).set_delay_main(128);
            }
            1 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(2);
                    self.sprite_slot_mut(k).set_delay_main(16);
                    self.sprite_apply_speed_towards_link(k, 32);
                } else {
                    if self.sprite_slot(k).delay_main() >= 0x40 {
                        self.sprite_slot_mut(k).set_z_velocity(4);
                        self.sprite_move_z(k);
                    }
                    self.sprite_94_tile_animate(k);
                }
            }
            2 => {
                self.sprite_slot_mut(k).set_ignore_projectile(0);
                if self.sprite_slot(k).delay_main() != 0
                    && (self.sprite_slot(k).delay_main() & 3) == 0
                {
                    self.sprite_apply_speed_towards_link(k, 32);
                }
                if !self.sprite_check_damage_to_and_from_link(k) {
                    self.sprite_move_xy(k);
                    let cy = self
                        .game_state
                        .sprites
                        .workspace
                        .current_sprite_y()
                        .wrapping_sub(u16::from(self.sprite_slot(k).z()));
                    self.sprite_workspace_mut().set_current_sprite_y(cy);
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
        if self.sprite_slot(k).e() != 0 {
            self.sprite_94_tile(k);
            return;
        }
        self.sprite_slot_mut(k).or_object_priority(0x30);
        self.pirogusu_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        match self.sprite_slot(k).ai_state() {
            0 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(1);
                    self.sprite_slot_mut(k).set_delay_main(31);
                }
                let value = self.sprite_slot(k).delay_main();
                self.sprite_slot_mut(k).set_ignore_projectile(value);
                let value = PIROGUSU_DIRECTION_LOOKUP[self.sprite_slot(k).direction() as usize];
                self.sprite_slot_mut(k).set_a(value);
            }
            1 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(2);
                    self.sprite_slot_mut(k).set_delay_main(32);
                    self.sprite_slot_mut(k).set_ignore_projectile(0);
                    self.sprite_zero_velocity_xy(k);
                } else {
                    let j = self.sprite_slot(k).direction() as usize;
                    let idx = ((self.sprite_slot(k).delay_main() >> 3) & 1) as usize | (j << 1);
                    self.sprite_slot_mut(k).set_a(PIROGUSU_GRAPHICS_LOOKUP[idx]);
                    self.sprite_slot_mut(k)
                        .set_x_velocity(PIROGUSU_AXIS_VELOCITIES[j + 2] as u8);
                    self.sprite_slot_mut(k)
                        .set_y_velocity(PIROGUSU_AXIS_VELOCITIES[j] as u8);
                    self.sprite_move_xy(k);
                }
            }
            2 => {
                self.sprite_check_damage_to_and_from_link(k);
                self.sprite_move_xy(k);
                let j = self.sprite_slot(k).direction() as usize;
                let value = self
                    .sprite_slot(k)
                    .x_velocity()
                    .wrapping_add(PIROGUSU_COLLISION_AXIS_VELOCITIES[j] as u8);
                self.sprite_slot_mut(k).set_x_velocity(value);
                let value = self
                    .sprite_slot(k)
                    .y_velocity()
                    .wrapping_add(PIROGUSU_COLLISION_AXIS_VELOCITIES[j + 2] as u8);
                self.sprite_slot_mut(k).set_y_velocity(value);
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_spawn_small_splash(k);
                    self.sprite_slot_mut(k).set_delay_aux1(16);
                    self.sprite_slot_mut(k).set_ai_state(3);
                }
                let idx = ((self.game_state.frame.frame_counter >> 2) & 1) as usize | (j << 1);
                self.sprite_slot_mut(k)
                    .set_a(PIROGUSU_ANIMATION_STATE_LOOKUP[idx]);
            }
            3 => {
                if self.sprite_return_if_recoiling(k) {
                    return;
                }
                self.sprite_check_damage_to_and_from_link(k);
                let j = self.sprite_slot(k).direction() as usize;
                let idx = ((self.game_state.frame.frame_counter >> 2) & 1) as usize | (j << 1);
                self.sprite_slot_mut(k)
                    .set_a(PIROGUSU_ANIMATION_STATE_LOOKUP[idx].wrapping_add(8));
                if self.sprite_slot(k).delay_aux1() == 0 {
                    self.pirogusu_spawn_splash(k);
                    self.sprite_move_xy(k);
                    if (self.sprite_check_tile_collision(k) & 15) != 0 {
                        let rnd = self.get_random_number() & 1;
                        self.sprite_slot_mut(k)
                            .set_direction(PIROGUSU_DIRECTIONS[(j << 1) | rnd as usize]);
                    }
                    let j = self.sprite_slot(k).direction() as usize;
                    self.sprite_slot_mut(k)
                        .set_x_velocity(PIROGUSU_FAST_AXIS_VELOCITIES[j] as u8);
                    self.sprite_slot_mut(k)
                        .set_y_velocity(PIROGUSU_FAST_AXIS_VELOCITIES[j + 2] as u8);
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
        if self.sprite_slot(k).delay_main() == 0 && self.sprite_check_tile_collision(k) != 0 {
            self.sprite_slot_mut(k).set_state(0);
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x26);
        }
    }

    // void Sprite_95_LaserEyeLeft(int k) {  // 9ea541
    pub(super) fn sprite_95_laser_eye_left(&mut self, k: usize) {
        if self.sprite_slot(k).a() != 0 {
            self.sprite_laser_beam(k);
            return;
        }
        self.laser_eye_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        match self.sprite_slot(k).ai_state() {
            0 => {
                let facing = self.game_state.player.follower_link.facing_index() & 3;
                if self.sprite_slot(k).head_direction() == 0
                    && self.sprite_slot(k).direction() != LASER_EYE_DIRECTIONS[facing]
                {
                    self.sprite_slot_mut(k).set_graphics(0);
                } else {
                    let j = if self.sprite_slot(k).direction() < 2 {
                        self.game_state
                            .player
                            .follower_link
                            .y()
                            .wrapping_sub(self.game_state.sprites.workspace.current_sprite_y())
                    } else {
                        self.game_state
                            .player
                            .follower_link
                            .x()
                            .wrapping_sub(self.game_state.sprites.workspace.current_sprite_x())
                    };
                    if j.wrapping_add(16) < 32 {
                        self.sprite_slot_mut(k).set_delay_main(32);
                        self.sprite_slot_mut(k).set_ai_state(1);
                    } else {
                        self.sprite_slot_mut(k).set_graphics(0);
                    }
                }
            }
            1 => {
                self.sprite_slot_mut(k).set_graphics(1);
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(0);
                    self.laser_eye_fire_beam(k);
                    self.sprite_slot_mut(k).set_delay_aux4(12);
                }
            }
            _ => {}
        }
    }

    // void Sprite_91_StalfosKnight(int k) {  // 9eaaa7
    pub(super) fn sprite_91_stalfos_knight(&mut self, k: usize) {
        if self.sprite_slot(k).ai_state() == 0 {
            let _ = self.sprite_prep_oam_coord_or_double_ret(k);
        } else {
            self.stalfos_knight_draw(k);
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if (self.sprite_slot(k).hit_timer() & 127) == 1 {
            self.sprite_slot_mut(k).set_hit_timer(0);
            self.sprite_slot_mut(k).set_ai_state(6);
            self.sprite_slot_mut(k).set_delay_main(255);
            self.sprite_slot_mut(k).set_x_velocity(0);
            self.sprite_slot_mut(k).set_y_velocity(0);
            self.enemy_damage_subclass_table_mut().set_entry(0x918, 2);
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }
        match self.sprite_slot(k).ai_state() {
            0 => {
                self.sprite_slot_mut(k).set_flags4(9);
                self.sprite_slot_mut(k).set_ignore_projectile(9);
                let bak0 = self.sprite_slot(k).flags2();
                self.sprite_slot_mut(k).or_flags2(128);
                let flag = self.sprite_check_damage_to_link(k);
                self.sprite_slot_mut(k).set_flags2(bak0);
                if flag {
                    self.sprite_slot_mut(k).set_z(144);
                    self.sprite_slot_mut(k).set_ai_state(1);
                    let value = 2;
                    self.sprite_slot_mut(k).set_head_direction(value);
                    self.sprite_slot_mut(k).set_graphics(2);
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
                }
            }
            1 => {
                let old_z = self.sprite_slot(k).z();
                self.sprite_move_z(k);
                if !sign8(self.sprite_slot(k).z_velocity().wrapping_add(64)) {
                    self.sprite_slot_mut(k).subtract_z_velocity(3);
                }
                if sign8(old_z ^ self.sprite_slot(k).z()) && sign8(self.sprite_slot(k).z()) {
                    self.stalfos_knight_set_to_ground(k);
                }
            }
            2 => {
                self.enemy_damage_subclass_table_mut().set_entry(0x918, 0);
                self.sprite_check_damage_to_and_from_link(k);
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(3);
                    let value = self.get_random_number() & 63;
                    self.sprite_slot_mut(k).set_b(value);
                    self.sprite_slot_mut(k).set_delay_main(127);
                } else {
                    let gfx = STALFOS_KNIGHT_CASE2_GRAPHICS
                        [(self.sprite_slot(k).delay_main() >> 5) as usize];
                    self.sprite_slot_mut(k).set_c(gfx);
                    self.sprite_slot_mut(k).set_graphics(gfx);
                    let value = 2;
                    self.sprite_slot_mut(k).set_head_direction(value);
                }
            }
            3 => {
                self.sprite_check_damage_to_and_from_link(k);
                if self.sprite_slot(k).delay_main() == self.sprite_slot(k).b() {
                    let value = self.sprite_is_right_of_link(k).a;
                    self.sprite_slot_mut(k).set_head_direction(value);
                    self.sprite_slot_mut(k).set_ai_state(4);
                    self.sprite_slot_mut(k).set_delay_main(32);
                } else {
                    let value = STALFOS_KNIGHT_CASE2_DIRECTIONS
                        [(self.sprite_slot(k).delay_main() >> 3) as usize];
                    self.sprite_slot_mut(k).set_head_direction(value);
                    self.sprite_slot_mut(k).set_c(0);
                    self.sprite_slot_mut(k).set_graphics(0);
                }
            }
            4 => {
                self.sprite_check_damage_to_and_from_link(k);
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(5);
                    self.sprite_slot_mut(k).set_delay_main(255);
                    self.sprite_slot_mut(k).set_delay_aux1(32);
                }
                self.sprite_slot_mut(k).set_c(1);
                self.sprite_slot_mut(k).set_graphics(1);
            }
            5 => {
                self.sprite_check_damage_to_and_from_link(k);
                if self.sprite_slot(k).delay_aux1() == 0 {
                    self.sprite_move_xyz(k);
                    self.sprite_check_tile_collision(k);
                    if !sign8(self.sprite_slot(k).z_velocity().wrapping_add(64)) {
                        self.sprite_slot_mut(k).subtract_z_velocity(2);
                    }
                    if sign8(self.sprite_slot(k).z().wrapping_sub(1)) {
                        self.sprite_slot_mut(k).set_z(0);
                        self.sprite_slot_mut(k).set_z_velocity(0);
                        if self.sprite_slot(k).delay_main() == 0 {
                            self.stalfos_knight_set_to_ground(k);
                            return;
                        }
                        self.sprite_slot_mut(k).set_delay_aux1(16);
                    }
                    let value = if sign8(self.sprite_slot(k).z_velocity().wrapping_sub(24)) {
                        2
                    } else {
                        0
                    };
                    self.sprite_slot_mut(k).set_graphics(value);
                } else {
                    if self.sprite_slot(k).delay_aux1() == 1 {
                        self.sprite_slot_mut(k).set_z_velocity(48);
                        self.sprite_apply_speed_towards_link(k, 16);
                        let value = self.sprite_is_right_of_link(k).a;
                        self.sprite_slot_mut(k).set_head_direction(value);
                        self.sprite_sfx_queue_sfx3_with_pan(k, 0x13);
                    }
                    self.sprite_slot_mut(k).set_c(1);
                    self.sprite_slot_mut(k).set_graphics(1);
                }
            }
            6 => {
                self.sprite_move_xyz(k);
                self.sprite_check_tile_collision(k);
                if !sign8(self.sprite_slot(k).z_velocity().wrapping_add(64)) {
                    self.sprite_slot_mut(k).subtract_z_velocity(2);
                }
                if sign8(self.sprite_slot(k).z().wrapping_sub(1)) {
                    self.sprite_slot_mut(k).set_z(0);
                    self.sprite_slot_mut(k).set_z_velocity(0);
                }
                let j = self.sprite_slot(k).delay_main();
                if j == 0 {
                    if (self.get_random_number() & 1) != 0 {
                        self.stalfos_knight_set_to_ground(k);
                    } else {
                        self.sprite_slot_mut(k).set_ai_state(7);
                        self.sprite_slot_mut(k).set_delay_main(80);
                    }
                } else {
                    if j >= 224 && (j & 3) == 0 {
                        self.sprite_sfx_queue_sfx3_with_pan(k, 0x14);
                    }
                    self.sprite_slot_mut(k)
                        .set_c(STALFOS_KNIGHT_CASE6_STATES[(j >> 3) as usize]);
                    self.sprite_slot_mut(k).set_graphics(3);
                    let value = 2;
                    self.sprite_slot_mut(k).set_head_direction(value);
                }
            }
            7 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.stalfos_knight_set_to_ground(k);
                } else {
                    let value = STALFOS_KNIGHT_CASE7_GRAPHICS
                        [((self.sprite_slot(k).delay_main() >> 2) & 1) as usize];
                    self.sprite_slot_mut(k).set_graphics(value);
                }
            }
            _ => {}
        }
    }

    // void Sprite_90_Wallmaster(int k) {  // 9eaea4
    pub(super) fn sprite_90_wallmaster(&mut self, k: usize) {
        self.sprite_slot_mut(k).or_object_priority(0x30);
        self.wall_master_draw(k);
        if self.sprite_slot(k).state() != 9 {
            self.follower_link_state_mut().clear_immobilized();
            self.follower_link_state_mut()
                .clear_sprite_damage_disable_timer();
        }
        if self.sprite_return_if_inactive(k) {
            return;
        }
        if self.sprite_slot(k).a() != 0 {
            let link_x = self.sprite_get_x(k);
            let link_y = self
                .sprite_get_y(k)
                .wrapping_sub(u16::from(self.sprite_slot(k).z()))
                .wrapping_add(3);
            self.follower_link_state_mut().set_x(link_x);
            self.follower_link_state_mut().set_y(link_y);
            self.follower_link_state_mut().immobilize();
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.follower_link_state_mut().set_incapacitated_timer(0);
            self.follower_link_state_mut().set_actual_velocity_xy(0, 0);
            self.follower_link_state_mut().clear_movement_velocity();
            if link_y
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
                .wrapping_sub(16)
                >= 0x100
            {
                self.follower_link_state_mut().clear_immobilized();
                self.follower_link_state_mut()
                    .clear_sprite_damage_disable_timer();
                self.wall_master_send_player_to_last_entrance();
                self.link_initialize();
                return;
            }
        } else {
            self.sprite_check_damage_from_link(k);
        }
        match self.sprite_slot(k).ai_state() {
            0 => {
                let old_z = self.sprite_slot(k).z();
                self.sprite_move_z(k);
                if !sign8(self.sprite_slot(k).z_velocity().wrapping_add(64)) {
                    self.sprite_slot_mut(k).subtract_z_velocity(3);
                }
                if sign8(old_z ^ self.sprite_slot(k).z()) && sign8(self.sprite_slot(k).z()) {
                    self.sprite_slot_mut(k).set_ai_state(1);
                    self.sprite_slot_mut(k).set_z(0);
                    self.sprite_slot_mut(k).set_z_velocity(0);
                    self.sprite_slot_mut(k).set_delay_main(63);
                }
            }
            1 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(2);
                }
                let value = if (self.sprite_slot(k).delay_main() & 0x20) != 0 {
                    0
                } else {
                    1
                };
                self.sprite_slot_mut(k).set_graphics(value);
                if self.sprite_check_damage_to_link(k) {
                    self.sprite_slot_mut(k).set_a(1);
                    self.sprite_slot_mut(k).set_flags3(64);
                    self.sprite_sfx_queue_sfx3_with_pan(k, 0x2a);
                }
            }
            2 => {
                let old_z = self.sprite_slot(k).z();
                self.sprite_move_z(k);
                if sign8(self.sprite_slot(k).z_velocity().wrapping_sub(64)) {
                    self.sprite_slot_mut(k).add_z_velocity(2);
                }
                if sign8(old_z ^ self.sprite_slot(k).z()) && !sign8(self.sprite_slot(k).z()) {
                    self.sprite_slot_mut(k).set_state(0);
                }
            }
            _ => {}
        }
    }

    // void Sprite_8A_SpikeBlock(int k) {  // 9ebce8
    pub(super) fn sprite_8_a_spike_block(&mut self, k: usize) {
        if self.sprite_slot(k).e() == 0 {
            self.sprite_draw_single_large(k);
            if self.sprite_return_if_inactive(k) {
                return;
            }
            self.sprite_check_damage_to_and_from_link(k);
            self.sprite_move_xy(k);
            self.sprite_check_tile_collision(k);
            if self.sprite_slot(k).delay_main() == 0
                && (!self.spike_block_check_statue_collision(k)
                    || (self.sprite_slot(k).wall_collision() & 0x0f) != 0)
            {
                self.sprite_slot_mut(k).set_delay_main(4);
                let value = (0u8).wrapping_sub(self.sprite_slot(k).x_velocity());
                self.sprite_slot_mut(k).set_x_velocity(value);
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
        if self.sprite_slot(k).ai_state() == 0 {
            self.dungeon_update_tile_map_with_common_tile_for_mothula(
                self.sprite_get_x(k),
                self.sprite_get_y(k),
                0,
            );
            self.sprite_slot_mut(k).set_ai_state(1);
            self.sprite_slot_mut(k).set_delay_main(64);
            self.sprite_slot_mut(k).set_delay_aux1(105);
        } else if self.sprite_slot(k).delay_main() != 0 {
            if self.sprite_slot(k).delay_main() == 1 {
                let value = self.sprite_slot(k).a();
                self.sprite_slot_mut(k).set_x_low(value);
                let value = self.sprite_slot(k).b();
                self.sprite_slot_mut(k).set_y_low(value);
            } else {
                let value = if ((self.sprite_slot(k).delay_main() >> 1) & 1) != 0 {
                    (-8i8) as u8
                } else {
                    8
                };
                self.sprite_slot_mut(k).set_x_velocity(value);
                self.sprite_move_x(k);
                self.sprite_slot_mut(k).set_x_velocity(0);
            }
        } else if self.sprite_slot(k).ai_state() == 1 {
            const X_TARGET: [i8; 4] = [32, -32, 0, 0];
            const Y_TARGET: [i8; 4] = [0, 0, 32, -32];
            const X_DELTA: [i8; 4] = [1, -1, 0, 0];
            const Y_DELTA: [i8; 4] = [0, 0, 1, -1];
            let j = self.sprite_slot(k).direction() as usize;
            if self.sprite_slot(k).x_velocity() != X_TARGET[j] as u8 {
                let value = self
                    .sprite_slot(k)
                    .x_velocity()
                    .wrapping_add(X_DELTA[j] as u8);
                self.sprite_slot_mut(k).set_x_velocity(value);
            }
            if self.sprite_slot(k).y_velocity() != Y_TARGET[j] as u8 {
                let value = self
                    .sprite_slot(k)
                    .y_velocity()
                    .wrapping_add(Y_DELTA[j] as u8);
                self.sprite_slot_mut(k).set_y_velocity(value);
            }
            self.sprite_move_xy(k);
            if self.sprite_slot(k).delay_aux1() == 0 {
                self.sprite_get_16bit_coords_for_mothula(k);
                if self.sprite_check_tile_collision(k) != 0 {
                    self.sprite_slot_mut(k).set_ai_state(2);
                    self.sprite_slot_mut(k).set_delay_aux1(64);
                }
            }
        } else if self.sprite_slot(k).delay_aux1() == 0 {
            const SPIKE_BLOCK_RETURN_X_VELOCITIES: [i8; 4] = [-16, 16, 0, 0];
            const SPIKE_BLOCK_RETURN_Y_VELOCITIES: [i8; 4] = [0, 0, -16, 16];
            let j = self.sprite_slot(k).direction() as usize;
            self.sprite_slot_mut(k)
                .set_x_velocity(SPIKE_BLOCK_RETURN_X_VELOCITIES[j] as u8);
            self.sprite_slot_mut(k)
                .set_y_velocity(SPIKE_BLOCK_RETURN_Y_VELOCITIES[j] as u8);
            self.sprite_move_xy(k);
            if self.sprite_slot(k).x_low() == self.sprite_slot(k).a()
                && self.sprite_slot(k).y_low() == self.sprite_slot(k).b()
            {
                self.sprite_slot_mut(k).set_state(0);
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
        const BIG_SPIKE_ATTACK_X_VELOCITIES: [i8; 4] = [32, -32, 0, 0];
        const BIG_SPIKE_RETURN_X_VELOCITIES: [i8; 4] = [-16, 16, 0, 0];
        const BIG_SPIKE_ATTACK_Y_VELOCITIES: [i8; 4] = [0, 0, 32, -32];
        const BIG_SPIKE_RETURN_Y_VELOCITIES: [i8; 4] = [0, 0, -16, 16];
        const BIG_SPIKE_ATTACK_DELAYS: [u8; 4] = [0x40, 0x40, 0x38, 0x38];

        self.spike_trap_draw(k);
        if self.sprite_return_if_inactive(k) {
            return;
        }
        self.sprite_check_damage_to_and_from_link(k);
        if self.sprite_slot(k).ai_state() == 0 {
            let mut pt = PointU8 { x: 0, y: 0 };
            let j = usize::from(self.sprite_direction_to_face_link(k, Some(&mut pt)));
            self.sprite_slot_mut(k).set_direction(j as u8);
            if pt.x.wrapping_add(16) < 32 || pt.y.wrapping_add(16) < 32 {
                self.sprite_slot_mut(k)
                    .set_delay_main(BIG_SPIKE_ATTACK_DELAYS[j]);
                self.sprite_slot_mut(k).set_ai_state(1);
                self.sprite_slot_mut(k)
                    .set_x_velocity(BIG_SPIKE_ATTACK_X_VELOCITIES[j] as u8);
                self.sprite_slot_mut(k)
                    .set_y_velocity(BIG_SPIKE_ATTACK_Y_VELOCITIES[j] as u8);
            }
        } else if self.sprite_slot(k).ai_state() == 1 {
            if self.sprite_check_tile_collision(k) != 0 || self.sprite_slot(k).delay_main() == 0 {
                self.sprite_slot_mut(k).set_ai_state(2);
                self.sprite_slot_mut(k).set_delay_main(96);
            }
            self.sprite_move_xy(k);
        } else if self.sprite_slot(k).delay_main() == 0 {
            let j = usize::from(self.sprite_slot(k).direction());
            self.sprite_slot_mut(k)
                .set_x_velocity(BIG_SPIKE_RETURN_X_VELOCITIES[j] as u8);
            self.sprite_slot_mut(k)
                .set_y_velocity(BIG_SPIKE_RETURN_Y_VELOCITIES[j] as u8);
            self.sprite_move_xy(k);
            if self.sprite_slot(k).x_low() == self.sprite_slot(k).a()
                && self.sprite_slot(k).y_low() == self.sprite_slot(k).c()
            {
                self.sprite_slot_mut(k).set_ai_state(0);
            }
        }
    }

    // void Sprite_88_Mothula(int k) {  // 9ebe7e
    pub(super) fn sprite_88_mothula(&mut self, k: usize) {
        if self
            .game_state
            .enhanced_features
            .has(FEATURE_MISC_BUG_FIXES_MOTHULA)
        {
            let mut damage_data = self.enemy_damage_subclass_table_mut();
            damage_data.set_entry(0x884, 1);
            damage_data.set_entry(0x885, 1);
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
        self.oam_state_mut().set_current_pointer(0x920);
        self.oam_state_mut().set_current_extended_pointer(0xa68);
        let g = self.sprite_slot(k).graphics() as usize;
        let (info_x, info_y) = self.sprite_draw_multiple_for_mothula(k, g * 8, 8);
        if self.sprite_slot(k).pause() != 0 {
            return;
        }
        let info_y = info_y.wrapping_add(self.sprite_slot(k).z() as u16);
        // oam = current oam ptr + 10 entries (each OamEnt is 4 bytes
        // in the small region pointed to by oam_cur_ptr in the C port).
        let oam_base = self.game_state.oam.current_pointer_usize();
        // Iterate i = 8..=0 (inclusive). The C `oam++` advances one
        // OamEnt per iteration; the small region uses 4 bytes/entry.
        for step in 0..=8 {
            let i = 8 - step;
            let oam = oam_base + (10 + step) * 4;
            let x = info_x.wrapping_add(MOTHULA_DRAW_X_OFFSETS[g * 9 + i] as i16 as u16);
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
        if self.sprite_slot(k).state() == 11 {
            self.sprite_slot_mut(k).set_ai_state(0);
        }
        if self.sprite_return_if_inactive_for_mothula(k) {
            return;
        }
        self.sprite_slot_mut(k).set_flags3(0);
        if self.sprite_slot(k).delay_aux3() != 0 {
            self.sprite_slot_mut(k).set_flags3(64);
        }
        if (self.sprite_slot(k).f() & 127) == 6 {
            self.sprite_slot_mut(k).set_f(0);
            self.sprite_slot_mut(k).set_delay_aux3(32);
            self.sprite_slot_mut(k).set_ai_state(2);
            self.sprite_slot_mut(k).set_delay_main(0);
            self.sprite_slot_mut(k).set_g(64);
        }
        if self.sprite_return_if_recoiling_for_mothula(k) {
            return;
        }
        match self.sprite_slot(k).ai_state() {
            0 => {
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).set_ai_state(1);
                }
            }
            1 => {
                self.sprite_slot_mut(k).set_z_velocity(8);
                self.sprite_move_z_for_mothula(k);
                self.sprite_slot_mut(k).set_z_velocity(0);
                if self.sprite_slot(k).z() >= 24 {
                    self.sprite_slot_mut(k).set_g(128);
                    self.sprite_slot_mut(k).set_ai_state(2);
                    self.sprite_slot_mut(k).set_ignore_projectile(0);
                    self.sprite_slot_mut(k).set_delay_main(64);
                }
                self.mothula_flap_wings(k);
            }
            2 => {
                if self.sprite_slot(k).g() == 0 {
                    self.sprite_slot_mut(k).set_delay_main(63);
                    self.sprite_slot_mut(k).set_ai_state(3);
                    return;
                }
                self.sprite_slot_mut(k).decrement_g();
                self.mothula_flap_wings(k);
                let j = (self.sprite_slot(k).a() & 1) as usize;
                // sprite_z_vel[k] += j ? -1 : 1
                let delta: i8 = if j != 0 { -1 } else { 1 };
                let value = self.sprite_slot(k).z_velocity().wrapping_add(delta as u8);
                self.sprite_slot_mut(k).set_z_velocity(value);
                // limit = (uint8)(j ? -16 : 16)
                let limit: u8 = if j != 0 { (-16i8) as u8 } else { 16 };
                if self.sprite_slot(k).z_velocity() == limit {
                    self.sprite_slot_mut(k).add_a(1);
                }
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).add_c(1);
                    if self.sprite_slot(k).c() == 7 {
                        self.sprite_slot_mut(k).set_c(0);
                        self.sprite_apply_speed_towards_link_for_mothula(k, 32);
                        self.sprite_slot_mut(k).set_delay_main(128);
                    } else {
                        let j2 = (self.get_random_number() & 7) as usize;
                        self.sprite_slot_mut(k)
                            .set_x_velocity(MOTHULA_AXIS_VELOCITIES[j2 + 2] as u8);
                        self.sprite_slot_mut(k)
                            .set_y_velocity(MOTHULA_AXIS_VELOCITIES[j2] as u8);
                        let value = (self.get_random_number() & 31).wrapping_add(64);
                        self.sprite_slot_mut(k).set_delay_main(value);
                    }
                }
                if self.sprite_slot(k).wall_collision() == 0 {
                    self.sprite_move_xy_for_mothula(k);
                }
                self.sprite_move_z_for_mothula(k);
                if self.sprite_check_tile_collision_for_mothula(k) {
                    self.sprite_slot_mut(k).set_delay_main(0);
                }
                self.sprite_check_damage_to_and_from_link_for_mothula(k);
                self.sprite_slot_mut(k).add_subtype2(2);
            }
            3 => {
                self.sprite_check_damage_to_and_from_link_for_mothula(k);
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).decrement_ai_state();
                    let value = (self.get_random_number() & 31) | 64;
                    self.sprite_slot_mut(k).set_g(value);
                } else {
                    if self.sprite_slot(k).delay_main() == 0x20 {
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
        self.sprite_slot_mut(k).increment_subtype2();
        let j = ((self.sprite_slot(k).subtype2() >> 2) & 3) as usize;
        if j == 0 {
            self.sprite_sfx_queue_sfx3_with_pan(k, 0x2);
        }
        self.sprite_slot_mut(k)
            .set_graphics(MOTHULA_WING_FLAP_GRAPHICS[j]);
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
                self.sprite_slot_mut(j)
                    .set_y_low((r2_y as u8).wrapping_sub(r4_z as u8).wrapping_add(3));
                self.sprite_slot_mut(j).set_delay_main(16);
                self.sprite_slot_mut(j).set_ignore_projectile(16);
                self.sprite_slot_mut(j)
                    .set_x_low((r0_x as u8).wrapping_add(MOTHULA_BEAM_X_VELOCITIES[i] as u8));
                self.sprite_slot_mut(j)
                    .set_x_velocity(MOTHULA_BEAM_X_VELOCITIES[i] as u8);
                self.sprite_slot_mut(j)
                    .set_y_velocity(MOTHULA_BEAM_Y_VELOCITIES[i] as u8);
                self.sprite_slot_mut(j).set_z(0);
            }
        }
        self.temp_counter_mut().set(0xff);
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
        self.sprite_slot_mut(k).decrement_head_direction();
        if self.sprite_slot(k).head_direction() != 0 {
            return;
        }
        let value = 0x40;
        self.sprite_slot_mut(k).set_head_direction(value);
        let Some((j, _r0_x, _r2_y, _r4_z)) = self.sprite_spawn_dynamically_for_mothula(k, 0x8a)
        else {
            return;
        };
        let mut i = (self.get_random_number() & 0x1f) as usize;
        if i >= 30 {
            i -= 30;
        }
        self.sprite_slot_mut(j).set_a(MOTHULA_SPIKE_X_LOW[i]);
        self.sprite_slot_mut(j).set_x_low(MOTHULA_SPIKE_X_LOW[i]);
        self.sprite_slot_mut(j)
            .set_b(MOTHULA_SPIKE_Y_LOW[i].wrapping_sub(1));
        self.sprite_slot_mut(j)
            .set_y_low(MOTHULA_SPIKE_Y_LOW[i].wrapping_sub(1));
        self.sprite_slot_mut(j)
            .set_direction(MOTHULA_SPIKE_DIRECTIONS[i]);
        self.sprite_slot_mut(j).set_e(1);
        let value = self
            .game_state
            .sprites
            .workspace
            .room_origin_x_high()
            .wrapping_add(1);
        self.sprite_slot_mut(j).set_x_high(value);
        let value = self
            .game_state
            .sprites
            .workspace
            .room_origin_y_high()
            .wrapping_add(1);
        self.sprite_slot_mut(j).set_y_high(value);
        self.sprite_slot_mut(j).set_x_velocity(1);
        self.sprite_get_16bit_coords_for_mothula(j);
        self.sprite_check_tile_collision_for_mothula(j);
        self.sprite_slot_mut(j).set_x_velocity(0);
        let value = self.sprite_slot(j).a();
        self.sprite_slot_mut(j).set_x_low(value);
        let value = self.sprite_slot(j).b();
        self.sprite_slot_mut(j).set_y_low(value);
        if self.sprite_slot(j).wall_collision() == 0 {
            self.sprite_slot_mut(j).set_state(0);
            let value = 1;
            self.sprite_slot_mut(k).set_head_direction(value);
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
        self.sprite_slot_mut(k).or_object_priority(0x30);
        self.armos_knight_draw(k);
        if self.sprite_return_if_paused(k) {
            return;
        }
        if self.sprite_slot(k).state() != 9 {
            if self.sprite_slot(k).delay_main() != 0 {
                let value =
                    ARMOS_KNIGHT_WAKE_GRAPHICS[(self.sprite_slot(k).delay_main() >> 3) as usize];
                self.sprite_slot_mut(k).set_graphics(value);
                return;
            }
            if self
                .sprite_workspace_mut()
                .decrement_armos_knight_remaining_count()
                == 1
            {
                for j in (0..=5usize).rev() {
                    self.sprite_slot_mut(j).set_health(48);
                    self.sprite_slot_mut(j).set_x_velocity(0);
                    self.sprite_slot_mut(j).set_y_velocity(0);
                    self.sprite_slot_mut(j).set_z_velocity(0);
                }
            }
            self.sprite_slot_mut(k).set_state(0);
            if self.sprite_check_if_screen_is_clear() {
                let mut info = SpriteSpawnInfo::default();
                let j = self.sprite_spawn_dynamically(k, 0xea, &mut info);
                if j >= 0 {
                    let ju = j as usize;
                    self.sprite_set_spawned_coordinates(ju, &info);
                    self.sprite_slot_mut(ju).set_z_velocity(32);
                    self.sprite_slot_mut(ju).set_a(1);
                }
            }
            return;
        }

        self.sprite_move_xy(k);
        self.sprite_move_z(k);
        self.sprite_slot_mut(k).subtract_z_velocity(4);
        if sign8(self.sprite_slot(k).z()) {
            self.sprite_slot_mut(k).set_z_velocity(0);
            self.sprite_slot_mut(k).set_z(0);
            if self
                .game_state
                .sprites
                .workspace
                .armos_knight_remaining_count()
                != 1
                && self.sprite_slot(k).a() != 0
            {
                self.sprite_slot_mut(k).set_z_velocity(48);
                self.sprite_sfx_queue_sfx3_with_pan(k, 0x16);
            }
        }
        if self.sprite_slot(k).f() != 0 {
            self.sprite_zero_velocity_xy(k);
            self.sprite_slot_mut(k).set_ai_state(0);
            self.sprite_slot_mut(k).set_g(0);
        }
        if self.sprite_return_if_recoiling(k) {
            return;
        }

        if self.sprite_slot(k).a() == 0 {
            if self.sprite_slot(k).delay_main() == 0 {
                self.sprite_slot_mut(k).add_a(1);
                let value = (self.sprite_slot(k).flags2() & 0x7f).wrapping_sub(2);
                self.sprite_slot_mut(k).set_flags2(value);
                self.sprite_slot_mut(k).and_deflection_bits(!4);
                self.sprite_slot_mut(k).and_flags3(!0x40);
            } else {
                if self.sprite_slot(k).delay_main() == 64 {
                    self.set_sound_effect_1(0x35);
                } else if self.sprite_slot(k).delay_main() < 64 {
                    let j = (((self.sprite_slot(k).delay_main() >> 1) ^ k as u8) & 1) as usize;
                    self.sprite_slot_mut(k)
                        .set_x_velocity(ARMOS_KNIGHT_CHARGE_X_VELOCITIES[j] as u8);
                    self.sprite_move_x(k);
                    self.sprite_slot_mut(k).set_x_velocity(0);
                }
                self.sprite_check_damage_from_link(k);
                if self.sprite_check_damage_to_link_same_layer(k) {
                    self.sprite_nullify_hookshot_drag();
                    self.sprite_repel_dash();
                }
            }
        } else if self
            .game_state
            .sprites
            .workspace
            .armos_knight_remaining_count()
            == 1
        {
            self.sprite_armos_crusher(k);
        } else {
            self.sprite_check_damage_to_and_from_link(k);
            if self.sprite_slot(k).ai_state() == 0 {
                let home = self.armos_knight_home_position(k);
                let x = home.x();
                let y = home.y();
                let pt = self.sprite_project_speed_towards_location(k, x, y, 16);
                self.sprite_slot_mut(k).set_x_velocity(pt.x);
                self.sprite_slot_mut(k).set_y_velocity(pt.y);
                let cur_x = u16::from(self.sprite_slot(k).x_low())
                    | (u16::from(self.sprite_slot(k).x_high()) << 8);
                let cur_y = u16::from(self.sprite_slot(k).y_low())
                    | (u16::from(self.sprite_slot(k).y_high()) << 8);
                self.sprite_workspace_mut().set_current_sprite_x(cur_x);
                self.sprite_workspace_mut().set_current_sprite_y(cur_y);
                if x.wrapping_sub(cur_x).wrapping_add(2) < 4
                    && y.wrapping_sub(cur_y).wrapping_add(2) < 4
                {
                    self.sprite_slot_mut(k).set_ai_state(1);
                }
            } else {
                let home = self.armos_knight_home_position(k);
                let x_low = home.x_low();
                let x_high = home.x_high();
                let y_low = home.y_low();
                let y_high = home.y_high();
                self.sprite_slot_mut(k).set_x_low(x_low);
                self.sprite_slot_mut(k).set_x_high(x_high);
                self.sprite_slot_mut(k).set_y_low(y_low);
                self.sprite_slot_mut(k).set_y_high(y_high);
            }
        }
    }

    // void GiantMoldorm_IncrementalSegmentExplosion(int k) {  // 9dd8f2
    //   if (sprite_state[k] == 9 && sprite_delay_aux4[k] && sprite_delay_aux4[k] < 80 &&
    //       !(sprite_delay_aux4[k] & 15 | submodule_index | modal_pause_flag)) {
    //     sprite_B[k]++;
    //     Sprite_MakeBossExplosion(k);
    //   }
    // }
    pub(super) fn giant_moldorm_incremental_segment_explosion(&mut self, k: usize) {
        let aux4 = self.sprite_slot(k).delay_aux4();
        if self.sprite_slot(k).state() == 9
            && aux4 != 0
            && aux4 < 80
            && ((aux4 & 15)
                | self.game_state.frame.submodule
                | self.game_state.frame.modal_pause_flag)
                == 0
        {
            self.sprite_slot_mut(k).add_b(1);
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
        self.sprite_slot_mut(k).set_oam_flags(7);
        let aux4 = self.sprite_slot(k).delay_aux4();
        let bg1_y = if aux4 == 0 {
            0
        } else if aux4 & 1 != 0 {
            0xffff
        } else {
            1
        };
        self.set_bg1_y_offset(bg1_y);

        match self.sprite_slot(k).g() {
            0 => {
                self.sprite_check_damage_to_and_from_link(k);
                if (self.sprite_slot(k).delay_main() | self.sprite_slot(k).z()) == 0 {
                    self.sprite_apply_speed_towards_link(k, 32);
                    self.sprite_slot_mut(k).set_z_velocity(32);
                    self.sprite_slot_mut(k).add_g(1);
                    let link_x = self.game_state.player.follower_link.x();
                    let link_y = self.game_state.player.follower_link.y();
                    self.sprite_slot_mut(k).set_b(link_x as u8);
                    self.sprite_slot_mut(k).set_c((link_x >> 8) as u8);
                    self.sprite_slot_mut(k).set_e(link_y as u8);
                    self.sprite_slot_mut(k)
                        .set_head_direction((link_y >> 8) as u8);
                    self.sprite_sfx_queue_sfx2_with_pan(k, 0x20);
                }
            }
            1 => {
                self.sprite_slot_mut(k).add_z_velocity(3);
                let mut advance = self.sprite_check_tile_collision(k) != 0;
                if !advance {
                    let cur_x = u16::from(self.sprite_slot(k).x_low())
                        | (u16::from(self.sprite_slot(k).x_high()) << 8);
                    let cur_y = u16::from(self.sprite_slot(k).y_low())
                        | (u16::from(self.sprite_slot(k).y_high()) << 8);
                    self.sprite_workspace_mut().set_current_sprite_x(cur_x);
                    self.sprite_workspace_mut().set_current_sprite_y(cur_y);
                    let x = u16::from(self.sprite_slot(k).b())
                        | (u16::from(self.sprite_slot(k).c()) << 8);
                    let y = u16::from(self.sprite_slot(k).e())
                        | (u16::from(self.sprite_slot(k).head_direction()) << 8);
                    advance = x.wrapping_sub(cur_x).wrapping_add(16) < 32
                        && y.wrapping_sub(cur_y).wrapping_add(16) < 32;
                }
                if advance {
                    self.sprite_slot_mut(k).add_g(1);
                    self.sprite_slot_mut(k).set_delay_main(16);
                    self.sprite_slot_mut(k).set_x_velocity(0);
                    self.sprite_slot_mut(k).set_y_velocity(0);
                }
            }
            2 => {
                self.sprite_slot_mut(k).set_z_velocity(0);
                if self.sprite_slot(k).delay_main() == 0 {
                    self.sprite_slot_mut(k).add_g(1);
                }
            }
            3 => {
                self.sprite_slot_mut(k).set_z_velocity((-104i8) as u8);
                if !sign8(self.sprite_slot(k).z()) {
                    self.sprite_slot_mut(k).set_delay_main(32);
                    self.sprite_slot_mut(k).set_delay_aux4(32);
                    self.sprite_slot_mut(k).set_g(0);
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
        let entries: Vec<DrawMultipleData> = MOTHULA_DRAW_FRAMES[start..start + count]
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
        self.sprite_slot_mut(k).increment_subtype2();
        let value = (self.sprite_slot(k).subtype2() >> 2) & 1;
        self.sprite_slot_mut(k).set_graphics(value);
        if (((k as u8) ^ self.game_state.frame.frame_counter) & 7) == 0 {
            self.sprite_sfx_queue_sfx2_with_pan(k, 0x07);
        }
    }

    fn sprite_94_tile_break(&mut self, k: usize) {
        self.sprite_sfx_queue_sfx2_with_pan(k, 0x1f);
        self.sprite_slot_mut(k).set_state(6);
        self.sprite_slot_mut(k).set_delay_main(31);
        self.sprite_slot_mut(k).set_sprite_type(0xec);
        self.sprite_slot_mut(k).set_hit_timer(0);
        self.sprite_slot_mut(k).set_c(0x80);
    }

    fn stalfos_knight_set_to_ground(&mut self, k: usize) {
        self.sprite_slot_mut(k).set_ai_state(2);
        self.sprite_slot_mut(k).set_ignore_projectile(0);
        self.sprite_slot_mut(k).set_z(0);
        self.sprite_slot_mut(k).set_z_velocity(0);
        self.sprite_slot_mut(k).set_delay_main(63);
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
        state.sprite_slot_mut(k).set_state(9);
        state.clear_modal_pause_flag();
        state.set_submodule(0);
        state.sprite_slot_mut(k).set_deflection_bits(0x80);
        state.sprite_slot_mut(k).set_pause(0);
        state.sprite_slot_mut(k).set_hit_timer(0);
    }

    #[test]
    fn flap_wings_advances_subtype2_and_picks_gfx() {
        // Mothula_FlapWings: pre-increments sprite_subtype2[k], picks
        // gfx from kMothula_FlapWingsGfx[(subtype2 >> 2) & 3].
        let mut s = fresh_state();
        let k = 3;

        // subtype2 starts at 0 -> after ++ = 1, j = (1 >> 2) & 3 = 0
        // -> sfx queued, gfx = kMothula_FlapWingsGfx[0] = 0.
        s.sprite_slot_mut(k).set_subtype2(0);
        s.mothula_flap_wings(k);
        assert_eq!(s.sprite_slot(k).subtype2(), 1);
        assert_eq!(s.sprite_slot(k).graphics(), 0);

        // subtype2 = 3 -> ++ = 4 -> j = 1 -> gfx = 1.
        s.sprite_slot_mut(k).set_subtype2(3);
        s.mothula_flap_wings(k);
        assert_eq!(s.sprite_slot(k).subtype2(), 4);
        assert_eq!(s.sprite_slot(k).graphics(), 1);

        // subtype2 = 7 -> ++ = 8 -> j = 2 -> gfx = 2.
        s.sprite_slot_mut(k).set_subtype2(7);
        s.mothula_flap_wings(k);
        assert_eq!(s.sprite_slot(k).graphics(), 2);

        // subtype2 = 11 -> ++ = 12 -> j = 3 -> gfx = 1.
        s.sprite_slot_mut(k).set_subtype2(11);
        s.mothula_flap_wings(k);
        assert_eq!(s.sprite_slot(k).graphics(), 1);
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
        s.sprite_slot_mut(k).set_x_low(0x80);
        s.sprite_slot_mut(k).set_x_high(0x00);
        s.sprite_slot_mut(k).set_y_low(0x50);
        s.sprite_slot_mut(k).set_y_high(0x00);
        s.sprite_slot_mut(k).set_z(0);

        s.mothula_spawn_beams(k);
        assert_eq!(s.game_state.scratch_counter.value(), 0xff);

        // The reverse-allocator hands out 15, 14, 13 across the loop.
        // First iter (i = 2): x_vel = 16, y_vel = 24, x_lo = 0x80 + 16.
        assert_eq!(s.sprite_slot(15).x_velocity() as i8, 16);
        assert_eq!(s.sprite_slot(15).y_velocity(), 24);
        assert_eq!(s.sprite_slot(15).x_low(), 0x80u8.wrapping_add(16));
        assert_eq!(s.sprite_slot(15).delay_main(), 16);
        assert_eq!(s.sprite_slot(15).ignore_projectile(), 16);
        assert_eq!(s.sprite_slot(15).z(), 0);
        // Second iter (i = 1): x_vel = 0, y_vel = 32.
        assert_eq!(s.sprite_slot(14).x_velocity(), 0);
        assert_eq!(s.sprite_slot(14).y_velocity(), 32);
        // Third iter (i = 0): x_vel = -16 (=> 0xf0), y_vel = 24.
        assert_eq!(s.sprite_slot(13).x_velocity() as i8, -16);
        assert_eq!(s.sprite_slot(13).y_velocity(), 24);
    }

    #[test]
    fn main_transitions_delay_to_ascend() {
        // ai_state = 0, sprite_delay_main = 0 -> ai_state becomes 1.
        let mut s = fresh_state();
        let k = 4;
        make_active(&mut s, k);
        s.sprite_slot_mut(k).set_ai_state(0);
        s.sprite_slot_mut(k).set_delay_main(0);
        s.sprite_slot_mut(k).set_f(0);
        s.mothula_main(k);
        assert_eq!(s.sprite_slot(k).ai_state(), 1);
        assert_eq!(s.sprite_slot(k).flags3(), 0);
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
        s.sprite_slot_mut(k).set_f(6);
        s.sprite_slot_mut(k).set_ai_state(3);
        s.sprite_slot_mut(k).set_delay_main(5);
        s.sprite_slot_mut(k).set_g(0);
        s.mothula_main(k);
        assert_eq!(s.sprite_slot(k).f(), 0);
        assert_eq!(s.sprite_slot(k).delay_aux3(), 32);
        assert_eq!(s.sprite_slot(k).ai_state(), 2);
        // case 2 ran: G decremented from 64 to 63.
        assert_eq!(s.sprite_slot(k).g(), 63);
        // sprite_flags3 is evaluated BEFORE the F=6 branch sets
        // delay_aux3, so it stays 0 here.
        assert_eq!(s.sprite_slot(k).flags3(), 0);
    }

    #[test]
    fn main_state11_resets_ai_state() {
        // sprite_state[k] == 11 forces ai_state -> 0, then the active
        // check trips on state != 9 and exits early.
        let mut s = fresh_state();
        let k = 5;
        s.sprite_slot_mut(k).set_state(11);
        s.sprite_slot_mut(k).set_ai_state(3);
        s.mothula_main(k);
        assert_eq!(s.sprite_slot(k).ai_state(), 0);
        // Inactive exit means flags3 wasn't reset.
        assert_eq!(s.sprite_slot(k).flags3(), 0);
    }

    #[test]
    fn handle_spikes_decrements_and_returns_early() {
        // First call: sprite_head_dir is decremented; non-zero -> early
        // return (no allocation occurs).
        let mut s = fresh_state();
        let k = 1;
        s.sprite_slot_mut(k).set_head_direction(3);
        // Pre-fill a slot so we can prove no allocation happened.
        for j in 0..16 {
            s.sprite_slot_mut(j).set_state(9);
        }
        s.mothula_handle_spikes(k);
        assert_eq!(s.sprite_slot(k).head_direction(), 2);
        // No slot freed -> SPAWN cannot succeed even if it tried.
        for j in 0..16 {
            assert_eq!(s.sprite_slot(j).state(), 9);
        }
    }

    #[test]
    fn handle_spikes_arms_when_decrement_hits_zero() {
        // sprite_head_dir = 1 -> after decrement = 0 -> reload to 0x40
        // and try to spawn. Spawn succeeds (one slot free); spike
        // tables populate the target slot.
        let mut s = fresh_state();
        let k = 0;
        s.sprite_slot_mut(k).set_head_direction(1);
        // Mark all slots active except 15, so allocator picks 15.
        for j in 0..15 {
            s.sprite_slot_mut(j).set_state(9);
        }
        s.sprite_slot_mut(15).set_state(0);
        // Force the random number deterministically by seeding RNG via
        // calling get_random_number isn't an option here, but the
        // table lookup with whatever index is produced just needs to be
        // valid; we assert side-effects independent of index.
        s.sprite_workspace_mut().set_room_origin_x_high(0x10);
        s.sprite_workspace_mut().set_room_origin_y_high(0x20);
        s.garnish_state_mut().set_sprcoll_x_size(0xffff);
        s.garnish_state_mut().set_sprcoll_y_size(0xffff);

        s.mothula_handle_spikes(k);
        // head_dir was set to 0x40 before the spawn path. If the spawn
        // succeeded and tile-collision found no wall, head_dir is reset
        // to 1; this fixture leaves the spawned spike away from walls, so
        // wallcoll stays 0 and the final branch fires:
        assert_eq!(s.sprite_slot(k).head_direction(), 1);
        // Slot 15 was claimed and then re-zeroed by the wallcoll==0
        // branch. So sprite_state[15] == 0 again.
        assert_eq!(s.sprite_slot(15).state(), 0);
        // x_hi / y_hi reflect the current sprite-room origin plus 1.
        assert_eq!(s.sprite_slot(15).x_high(), 0x11);
        assert_eq!(s.sprite_slot(15).y_high(), 0x21);
        // x_vel was zeroed after the collision check.
        assert_eq!(s.sprite_slot(15).x_velocity(), 0);
    }
}
