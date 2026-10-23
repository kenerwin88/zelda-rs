//! Boss draw tables for Agahnim, Helmasaur, Moldorm, Lanmolas, and Trinexx.

use crate::zelda_rtl::sprite::DrawMultipleData;

pub(in crate::zelda_rtl::sprite_main_draw) const TRINEXX_DRAW1_DRAW_FRAMES: [DrawMultipleData; 36] = [
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x40c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x00c0,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x40e0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x00e0,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0000,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x0002,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x0020,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x0022,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x00c2,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x00c4,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x80c2,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x80c4,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x8020,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x8022,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x8000,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x8002,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0xc0e0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x80e0,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0xc0c0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x80c0,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0xc022,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0xc020,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0xc002,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0xc000,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x40c4,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x40c2,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0xc0c4,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0xc0c2,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x4002,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4000,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x4022,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x4020,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0026,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4026,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x8026,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0xc026,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const TRINEXX_DRAW_X_OFFSETS: [i8; 35] = [
    0, 3, 9, 16, 24, 0, 2, 7, 13, 20, 0, 1, 4, 9, 15, 0, 0, 0, 0, 0, 0, -1, -4, -9, -15, 0, -2, -7,
    -13, -20, 0, -3, -9, -16, -24,
];

pub(in crate::zelda_rtl::sprite_main_draw) const TRINEXX_DRAW_Y_OFFSETS: [u8; 35] = [
    0x18, 0x20, 0x25, 0x25, 0x21, 0x18, 0x20, 0x27, 0x2a, 0x2c, 0x18, 0x20, 0x28, 0x2f, 0x34, 0x18,
    0x21, 0x2a, 0x34, 0x3d, 0x18, 0x20, 0x28, 0x2f, 0x34, 0x18, 0x20, 0x27, 0x2a, 0x2c, 0x18, 0x20,
    0x25, 0x25, 0x21,
];

pub(in crate::zelda_rtl::sprite_main_draw) const TRINEXX_DRAW_CHARS: [u8; 5] = [6, 0x28, 0x28, 0x2c, 0x2c];

pub(in crate::zelda_rtl::sprite_main_draw) const TRINEXX_SCALE_MULTIPLIERS: [u8; 8] =
    [0xfc, 0xe0, 0xc0, 0xa0, 0x80, 0x60, 0x40, 0x20];

pub(in crate::zelda_rtl::sprite_main_draw) const TRINEXX_WIGGLE_X_OFFSETS: [i8; 16] =
    [0, 2, 3, 4, 4, 4, 3, 2, 0, -2, -3, -4, -4, -4, -3, -2];

pub(in crate::zelda_rtl::sprite_main_draw) const TRINEXX_WIGGLE_Y_OFFSETS: [i8; 16] =
    [-4, -4, -3, -2, 0, 2, 3, 4, 4, 4, 3, 2, 0, -2, -3, -4];

pub(in crate::zelda_rtl::sprite_main_draw) const CHATTY_AGAHNIM_TELEWARP_DRAW_DATA: [(i8, i8, u8, u8); 28] = [
    (-10, -16, 0xce, 0x06),
    (18, -16, 0xce, 0x06),
    (20, -13, 0x26, 0x06),
    (20, -5, 0x36, 0x06),
    (-12, -13, 0x26, 0x46),
    (-12, -5, 0x36, 0x46),
    (18, 0, 0x26, 0x06),
    (18, 8, 0x36, 0x06),
    (-10, 0, 0x26, 0x46),
    (-10, 8, 0x36, 0x46),
    (-8, 0, 0x22, 0x06),
    (8, 0, 0x22, 0x46),
    (-8, 16, 0x22, 0x86),
    (8, 16, 0x22, 0xc6),
    (-10, -16, 0xce, 0x04),
    (18, -16, 0xce, 0x04),
    (20, -13, 0x26, 0x44),
    (20, -5, 0x36, 0x44),
    (-12, -13, 0x26, 0x04),
    (-12, -5, 0x36, 0x04),
    (18, 0, 0x26, 0x44),
    (18, 8, 0x36, 0x44),
    (-10, 0, 0x26, 0x04),
    (-10, 8, 0x36, 0x04),
    (-8, 0, 0x20, 0x04),
    (8, 0, 0x20, 0x44),
    (-8, 16, 0x20, 0x84),
    (8, 16, 0x20, 0xc4),
];

pub(in crate::zelda_rtl::sprite_main_draw) const TELEWARP_DRAW_SIZES: [u8; 14] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2];

// Shared boss and enemy draw tables promoted from sprite_main_draw.rs method bodies.
pub(in crate::zelda_rtl::sprite_main_draw) const ALTAR_ZELDA_WARP_DRAW_FRAMES: [DrawMultipleData; 10] = [
    DrawMultipleData {
        x: 4,
        y: 4,
        char_flags: 0x0480,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 4,
        char_flags: 0x0480,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 4,
        char_flags: 0x04b7,
        ext: 0,
    },
    DrawMultipleData {
        x: 4,
        y: 4,
        char_flags: 0x04b7,
        ext: 0,
    },
    DrawMultipleData {
        x: -6,
        y: 0,
        char_flags: 0x0524,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: 0,
        char_flags: 0x4524,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x0524,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4524,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x05c6,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 0,
        char_flags: 0x05c6,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const ALTAR_ZELDA_DRAW_FRAMES: [DrawMultipleData; 4] = [
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x0103,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x0104,
        ext: 2,
    },
    DrawMultipleData {
        x: -4,
        y: 0,
        char_flags: 0x0100,
        ext: 2,
    },
    DrawMultipleData {
        x: 4,
        y: 0,
        char_flags: 0x0101,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const LEVITATE_GFX: [u8; 4] = [2, 0, 3, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const AGAHNIM_SHADOW_X_OFFSETS: [i8; 6] = [0, 10, 8, 0, -10, -10];

pub(in crate::zelda_rtl::sprite_main_draw) const AGAHNIM_SHADOW_Y_OFFSETS: [i8; 6] = [-9, -2, -2, -9, -2, -2];

pub(in crate::zelda_rtl::sprite_main_draw) const CLONE_DASH_X_VELOCITIES: [i8; 2] = [32, -32];

pub(in crate::zelda_rtl::sprite_main_draw) const CLONE_FLAGS3_BY_SLOT: [u8; 2] = [9, 11];

pub(in crate::zelda_rtl::sprite_main_draw) const ATTACK_POSE_OFFSETS_BY_DELAY: [u8; 16] =
    [0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const HEAD_TURN_STEPS_BY_DELAY: [u8; 16] =
    [0, 0, 0, 0, 0, 0, 0, 6, 5, 4, 3, 2, 1, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const HEAD_TURN_DIRECTION_BASES: [u8; 6] = [30, 24, 12, 0, 6, 18];

pub(in crate::zelda_rtl::sprite_main_draw) const TELEPORT_TARGET_X_LOW: [u8; 16] = [
    0x38, 0x38, 0x38, 0x58, 0x78, 0x98, 0xb8, 0xb8, 0xb8, 0x98, 0x58, 0x58, 0x60, 0x90, 0x98, 0x78,
];

pub(in crate::zelda_rtl::sprite_main_draw) const TELEPORT_TARGET_Y_LOW: [u8; 16] = [
    0xb8, 0x78, 0x58, 0x48, 0x48, 0x48, 0x58, 0x78, 0xb8, 0xb8, 0xb8, 0x90, 0x70, 0x70, 0x90, 0xa0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ENERGY_BALL_GRAPHICS: [u8; 16] = [2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 0, 0, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const GIANT_MOLDORM_X_VELOCITIES: [i8; 32] = [
    24, 22, 17, 9, 0, -9, -17, -22, -24, -22, -17, -9, 0, 9, 17, 22, 36, 33, 25, 13, 0, -13, -25,
    -33, -36, -33, -25, -13, 0, 13, 25, 33,
];

pub(in crate::zelda_rtl::sprite_main_draw) const GIANT_MOLDORM_Y_VELOCITIES: [i8; 32] = [
    0, 9, 17, 22, 24, 22, 17, 9, 0, -9, -17, -22, -24, -22, -17, -9, 0, 13, 25, 33, 36, 33, 25, 13,
    0, -13, -25, -33, -36, -33, -25, -13,
];

pub(in crate::zelda_rtl::sprite_main_draw) const GIANT_MOLDORM_NEXT_DIRECTIONS: [u8; 16] =
    [8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7];

pub(in crate::zelda_rtl::sprite_main_draw) const GIANT_MOLDORM_SEG_A_DRAW_FRAMES: [DrawMultipleData; 8] = [
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0084,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x00a4,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x4086,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4084,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x40a6,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x40a4,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const GIANT_MOLDORM_OAM_FLAGS: [u8; 4] = [0, 0x40, 0xc0, 0x80];

pub(in crate::zelda_rtl::sprite_main_draw) const GIANT_MOLDORM_HEAD_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x0082,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x00a2,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x4082,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x4080,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x40a2,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x40a0,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: -6,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: -6,
        char_flags: 0x0082,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 6,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: 6,
        char_flags: 0x00a2,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: -6,
        char_flags: 0x4082,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: -6,
        char_flags: 0x4080,
        ext: 2,
    },
    DrawMultipleData {
        x: -6,
        y: 6,
        char_flags: 0x40a2,
        ext: 2,
    },
    DrawMultipleData {
        x: 6,
        y: 6,
        char_flags: 0x40a0,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const GIANT_MOLDORM_EYE_X_OFFSETS: [i16; 16] = [
    16, 15, 12, 6, 0, -6, -12, -13, -16, -13, -12, -6, 0, 6, 12, 15,
];

pub(in crate::zelda_rtl::sprite_main_draw) const GIANT_MOLDORM_EYE_Y_OFFSETS: [i16; 16] = [
    0, 6, 12, 15, 16, 15, 12, 6, 0, -6, -12, -13, -16, -13, -12, -6,
];

pub(in crate::zelda_rtl::sprite_main_draw) const GIANT_MOLDORM_EYE_CHARS: [u8; 16] = [
    0xaa, 0xaa, 0xa8, 0xa8, 0x8a, 0x8a, 0xa8, 0xa8, 0xaa, 0xaa, 0xa8, 0xa8, 0x8a, 0x8a, 0xa8, 0xa8,
];

pub(in crate::zelda_rtl::sprite_main_draw) const GIANT_MOLDORM_EYE_FLAGS: [u8; 16] = [
    0, 0, 0, 0, 0x80, 0x80, 0x40, 0x40, 0x40, 0x40, 0xc0, 0xc0, 0, 0, 0x80, 0x80,
];

pub(in crate::zelda_rtl::sprite_main_draw) const HELMASAUR_KING_MASK_X_OFFSETS: [i8; 2] = [-3, 11];

pub(in crate::zelda_rtl::sprite_main_draw) const HELMASAUR_KING_MASK_CHARS: [u8; 8] =
    [0xce, 0xcf, 0xde, 0xde, 0xde, 0xde, 0xcf, 0xce];

pub(in crate::zelda_rtl::sprite_main_draw) const HELMASAUR_KING_MASK_FLAGS: [u8; 2] = [0x3b, 0x7b];

pub(in crate::zelda_rtl::sprite_main_draw) const HELMASAUR_KING_DRAW_D_DRAW_FRAMES: [DrawMultipleData; 19] = [
    DrawMultipleData {
        x: -24,
        y: -32,
        char_flags: 0x0b80,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -32,
        char_flags: 0x0b82,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -32,
        char_flags: 0x4b82,
        ext: 2,
    },
    DrawMultipleData {
        x: 24,
        y: -32,
        char_flags: 0x4b80,
        ext: 2,
    },
    DrawMultipleData {
        x: -24,
        y: -16,
        char_flags: 0x0b84,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -16,
        char_flags: 0x0b86,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -16,
        char_flags: 0x4b86,
        ext: 2,
    },
    DrawMultipleData {
        x: 24,
        y: -16,
        char_flags: 0x4b84,
        ext: 2,
    },
    DrawMultipleData {
        x: -24,
        y: 0,
        char_flags: 0x0b88,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 0,
        char_flags: 0x0b8a,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 0,
        char_flags: 0x4b8a,
        ext: 2,
    },
    DrawMultipleData {
        x: 24,
        y: 0,
        char_flags: 0x4b88,
        ext: 2,
    },
    DrawMultipleData {
        x: -24,
        y: 16,
        char_flags: 0x0b8c,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 16,
        char_flags: 0x0b8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 16,
        char_flags: 0x4b8e,
        ext: 2,
    },
    DrawMultipleData {
        x: 24,
        y: 16,
        char_flags: 0x4b8c,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 32,
        char_flags: 0x0ba0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 32,
        char_flags: 0x4ba0,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -40,
        char_flags: 0x0bac,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const HELMASAUR_KING_EXPLOSION_X_OFFSETS: [i8; 4] = [-28, -28, 28, 28];

pub(in crate::zelda_rtl::sprite_main_draw) const HELMASAUR_KING_EXPLOSION_Y_OFFSETS: [i8; 4] = [-28, 4, -28, 4];

pub(in crate::zelda_rtl::sprite_main_draw) const HELMASAUR_KING_EXPLOSION_CHARS: [u8; 4] = [0xa2, 0xa6, 0xa2, 0xa6];

pub(in crate::zelda_rtl::sprite_main_draw) const HELMASAUR_KING_EXPLOSION_FLAGS: [u8; 4] = [0xb, 0xb, 0x4b, 0x4b];

pub(in crate::zelda_rtl::sprite_main_draw) const HELMASAUR_KING_FIREBALL_Y_OFFSETS: [u8; 32] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 9, 8, 7, 6,
    5, 4, 3, 2, 1,
];

pub(in crate::zelda_rtl::sprite_main_draw) const HELMASAUR_MASK_DRAW_FRAMES: [DrawMultipleData; 24] = [
    DrawMultipleData {
        x: -16,
        y: -5,
        char_flags: 0x0dae,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x0dc0,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: -5,
        char_flags: 0x4dae,
        ext: 2,
    },
    DrawMultipleData {
        x: -16,
        y: 11,
        char_flags: 0x0dc2,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 11,
        char_flags: 0x0dc4,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 11,
        char_flags: 0x4dc2,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 27,
        char_flags: 0x0dc6,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 27,
        char_flags: 0x4dc6,
        ext: 2,
    },
    DrawMultipleData {
        x: -16,
        y: -5,
        char_flags: 0x0dae,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x0dc0,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: -5,
        char_flags: 0x4dae,
        ext: 2,
    },
    DrawMultipleData {
        x: -16,
        y: 11,
        char_flags: 0x0dc8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 11,
        char_flags: 0x0dc4,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 11,
        char_flags: 0x4dc2,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 27,
        char_flags: 0x0dc6,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 27,
        char_flags: 0x4dc6,
        ext: 2,
    },
    DrawMultipleData {
        x: -16,
        y: -5,
        char_flags: 0x0dae,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: -5,
        char_flags: 0x0dc0,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: -5,
        char_flags: 0x4dae,
        ext: 2,
    },
    DrawMultipleData {
        x: -16,
        y: 11,
        char_flags: 0x0dc8,
        ext: 2,
    },
    DrawMultipleData {
        x: 0,
        y: 11,
        char_flags: 0x0dc4,
        ext: 2,
    },
    DrawMultipleData {
        x: 16,
        y: 11,
        char_flags: 0x4dc8,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 27,
        char_flags: 0x0dc6,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 27,
        char_flags: 0x4dc6,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const ARMOS_KNIGHT_DRAW_X_OFFSETS: [i8; 24] = [
    -8, 8, -8, 8, -10, 10, -10, 10, -10, 10, -10, 10, -12, 12, -12, 12, -14, 14, -14, 14, -16, 24,
    -16, 24,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ARMOS_KNIGHT_DRAW_Y_OFFSETS: [i8; 24] = [
    -8, -8, 8, 8, -10, -10, 10, 10, -10, -10, 10, 10, -12, -12, 12, 12, -14, -14, 14, 14, -16, -16,
    24, 24,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ARMOS_KNIGHT_DRAW_CHARS: [u8; 24] = [
    0xc0, 0xc2, 0xe0, 0xe2, 0xc0, 0xc2, 0xe0, 0xe2, 0xc4, 0xc4, 0xc4, 0xc4, 0xc6, 0xc6, 0xc6, 0xc6,
    0xc8, 0xc8, 0xc8, 0xc8, 0xd8, 0xd8, 0xd8, 0xd8,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ARMOS_KNIGHT_DRAW_FLAGS: [u8; 24] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0, 0xc0, 0x80, 0x40, 0, 0xc0, 0x80, 0x40, 0, 0xc0, 0x80, 0x40, 0,
    0xc0, 0x80,
];

pub(in crate::zelda_rtl::sprite_main_draw) const ARMOS_KNIGHT_DRAW_SIZES: [u8; 24] = [
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const DESERT_BARRIER_X_VELOCITY_TARGETS: [u8; 2] = [16, (-16i8) as u8];

pub(in crate::zelda_rtl::sprite_main_draw) const KHOLDSTARE_DRAW_FRAMES: [DrawMultipleData; 16] = [
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x0082,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x00a2,
        ext: 2,
    },
    DrawMultipleData {
        x: -7,
        y: -7,
        char_flags: 0x0080,
        ext: 2,
    },
    DrawMultipleData {
        x: 7,
        y: -7,
        char_flags: 0x0082,
        ext: 2,
    },
    DrawMultipleData {
        x: -7,
        y: 7,
        char_flags: 0x00a0,
        ext: 2,
    },
    DrawMultipleData {
        x: 7,
        y: 7,
        char_flags: 0x00a2,
        ext: 2,
    },
    DrawMultipleData {
        x: -7,
        y: -7,
        char_flags: 0x0084,
        ext: 2,
    },
    DrawMultipleData {
        x: 7,
        y: -7,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: -7,
        y: 7,
        char_flags: 0x00a4,
        ext: 2,
    },
    DrawMultipleData {
        x: 7,
        y: 7,
        char_flags: 0x00a6,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: -8,
        char_flags: 0x0084,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: -8,
        char_flags: 0x0086,
        ext: 2,
    },
    DrawMultipleData {
        x: -8,
        y: 8,
        char_flags: 0x00a4,
        ext: 2,
    },
    DrawMultipleData {
        x: 8,
        y: 8,
        char_flags: 0x00a6,
        ext: 2,
    },
];

pub(in crate::zelda_rtl::sprite_main_draw) const AGAHNIM_DRAW_X_OFFSETS_0: [i8; 72] = [
    -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8,
    8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -8, 8, -6, 6,
    -6, 6, -8, 8, -8, 8, -6, 6, -6, 6, 0, 8, 0, 8, -8, 8, -8, 8,
];

pub(in crate::zelda_rtl::sprite_main_draw) const AGAHNIM_DRAW_Y_OFFSETS_0: [i8; 72] = [
    -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8,
    8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -8, -8, 8, 8, -6, -6,
    6, 6, -8, -8, 8, 8, -6, -6, 6, 6, 0, 0, 8, 8, 8, 8, 8, 8,
];

pub(in crate::zelda_rtl::sprite_main_draw) const AGAHNIM_DRAW_CHARS_0: [u8; 72] = [
    0x82, 0x82, 0xa2, 0xa2, 0x80, 0x80, 0xa0, 0xa0, 0x84, 0x84, 0xa4, 0xa4, 0x86, 0x86, 0xa6, 0xa6,
    0x88, 0x8a, 0xa8, 0xaa, 0x8c, 0x8e, 0xac, 0xae, 0xc4, 0xc2, 0xe4, 0xe6, 0xc0, 0xc2, 0xe0, 0xe2,
    0x8a, 0x88, 0xaa, 0xa8, 0x8e, 0x8c, 0xae, 0xac, 0xc2, 0xc4, 0xe6, 0xe4, 0xc2, 0xc0, 0xe2, 0xe0,
    0xec, 0xec, 0xec, 0xec, 0xec, 0xec, 0xec, 0xec, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee,
    0xdf, 0xdf, 0xdf, 0xdf, 0x40, 0x42, 0x40, 0x42,
];

pub(in crate::zelda_rtl::sprite_main_draw) const AGAHNIM_DRAW_FLAGS_0: [u8; 72] = [
    0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0x40,
    0x80, 0xc0, 0, 0x40, 0x80, 0xc0, 0, 0, 0, 0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const AGAHNIM_DRAW_X_OFFSETS_1: [i8; 72] = [
    -7, 15, -11, 11, -11, 11, -8, 8, -4, 4, 0, 0, -10, -1, -14, -5, -14, -5, -12, -7, -10, -7, -10,
    -10, 16, 8, 12, 4, 12, 4, 10, 6, 9, 7, 8, 8, -6, -6, -10, -10, -10, -10, -10, -10, -10, -10,
    -10, -10, 14, 14, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, -7, 15, -11, 11, -11, 11, -8, 8, -4,
    4, 0, 0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const AGAHNIM_DRAW_Y_OFFSETS_1: [i8; 72] = [
    -5, -5, -9, -9, -9, -9, -9, -9, -9, -9, -9, -9, -3, 9, -7, 5, -7, 5, -5, 3, -3, 3, -2, -2, -3,
    9, -7, 5, -7, 5, -5, 3, -3, 3, -2, -2, -3, 9, -7, 5, -7, 5, -5, 3, -3, 3, -2, -2, -3, 9, -7, 5,
    -7, 5, -5, 3, -3, 3, -2, -2, -5, -5, -9, -9, -9, -9, -9, -9, -9, -9, -9, -9,
];

pub(in crate::zelda_rtl::sprite_main_draw) const AGAHNIM_DRAW_CHARS_1: [u8; 36] = [
    0xce, 0xcc, 0xc6, 0xc6, 0xc6, 0xc6, 0xce, 0xcc, 0xc6, 0xc6, 0xc6, 0xc6, 0xce, 0xcc, 0xc6, 0xc6,
    0xc6, 0xc6, 0xce, 0xcc, 0xc6, 0xc6, 0xc6, 0xc6, 0xce, 0xcc, 0xc6, 0xc6, 0xc6, 0xc6, 0xce, 0xcc,
    0xc6, 0xc6, 0xc6, 0xc6,
];

pub(in crate::zelda_rtl::sprite_main_draw) const AGAHNIM_DRAW_BIG_1: [u8; 36] = [
    0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2,
    2, 2, 2, 2,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_7_A_AGAHNIM_START_STATE: [u8; 2] = [1, 6];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_7_A_AGAHNIM_GRAPHICS_0: [u8; 5] = [12, 13, 14, 15, 16];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_7_A_AGAHNIM_DIRECTIONS: [u8; 25] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 0, 1, 1, 4, 4, 0, 2, 2, 4, 4, 3, 2, 2,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_7_A_AGAHNIM_GRAPHICS_1: [u8; 6] = [2, 10, 8, 0, 4, 6];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_7_A_AGAHNIM_SECONDARY_GRAPHICS: [u8; 5] = [16, 15, 14, 13, 12];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_7_A_AGAHNIM_GRAPHICS_3: [u8; 7] = [0, 8, 10, 2, 2, 6, 4];

pub(in crate::zelda_rtl::sprite_main_draw) const KING_HELMASAUR_OPERATE_TAIL_MULTIPLIERS: [u8; 32] = [
    0xff, 0xf0, 0xe0, 0xd0, 0xc0, 0xb0, 0xa0, 0x90, 0x80, 0x70, 0x60, 0x50, 0x40, 0x30, 0x20, 0x10,
    0xff, 0xf8, 0xf0, 0xe8, 0xe0, 0xd8, 0xd0, 0xc8, 0xbc, 0xb0, 0xa0, 0x90, 0x70, 0x40, 0x20, 0x10,
];

pub(in crate::zelda_rtl::sprite_main_draw) const KING_HELMASAUR_OPERATE_TAIL_MULT_B: [u8; 16] = [
    0xff, 0xf0, 0xe0, 0xd0, 0xc0, 0xb0, 0xa0, 0x90, 0x80, 0x70, 0x60, 0x50, 0x40, 0x30, 0x20, 0x10,
];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_70_KING_HELMASAUR_FIREBALL_CHARS: [u8; 3] = [0xcc, 0xcc, 0xca];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_70_KING_HELMASAUR_FIREBALL_FLAGS: [u8; 2] = [0x33, 0x73];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_70_KING_HELMASAUR_FIREBALL_LOCAL_GRAPHICS: [u8; 4] = [2, 2, 1, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const KHOLDSTARE_DRAW_X_OFFSETS: [i8; 16] =
    [8, 7, 4, 2, 0, -2, -4, -7, -8, -7, -4, -2, 0, 2, 4, 7];

pub(in crate::zelda_rtl::sprite_main_draw) const KHOLDSTARE_DRAW_Y_OFFSETS: [i8; 16] =
    [0, 2, 4, 7, 8, 7, 4, 2, 0, -2, -4, -7, -8, -7, -4, -2];

pub(in crate::zelda_rtl::sprite_main_draw) const KHOLDSTARE_DRAW_CHARS: [u8; 16] = [
    0xac, 0xac, 0xaa, 0x8c, 0x8c, 0x8c, 0xaa, 0xac, 0xac, 0xaa, 0xaa, 0x8c, 0x8c, 0x8c, 0xaa, 0xac,
];

pub(in crate::zelda_rtl::sprite_main_draw) const KHOLDSTARE_DRAW_FL: [u8; 16] = [
    0x40, 0x40, 0x40, 0, 0, 0, 0, 0, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0xc0, 0xc0,
];

pub(in crate::zelda_rtl::sprite_main_draw) const MOLDORM_DRAW_X_OFFSETS: [i8; 16] =
    [11, 10, 9, 6, 3, 0, -2, -3, -4, -3, -2, 1, 4, 7, 9, 10];

pub(in crate::zelda_rtl::sprite_main_draw) const MOLDORM_DRAW_Y_OFFSETS: [i8; 16] =
    [4, 6, 9, 10, 11, 10, 9, 6, 3, 0, -2, -3, -4, -3, -2, 1];

pub(in crate::zelda_rtl::sprite_main_draw) const MOLDORM_DRAW_CHARS: [u8; 3] = [0x5d, 0x62, 0x60];

pub(in crate::zelda_rtl::sprite_main_draw) const MOLDORM_DRAW_XY: [i8; 3] = [4, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const MOLDORM_DRAW_BIG: [u8; 3] = [0, 2, 2];

pub(in crate::zelda_rtl::sprite_main_draw) const MOLDORM_DRAW_GET_OFFS: [u8; 3] = [21, 26, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_54_LANMOLAS_RAND_B: [u8; 8] =
    [0x58, 0x50, 0x60, 0x70, 0x80, 0x90, 0xa0, 0x98];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_54_LANMOLAS_RAND_C: [u8; 8] =
    [0x68, 0x60, 0x70, 0x80, 0x90, 0xa0, 0xa8, 0x80];

pub(in crate::zelda_rtl::sprite_main_draw) const SPRITE_54_LANMOLAS_ZVEL: [i8; 2] = [2, -2];

pub(in crate::zelda_rtl::sprite_main_draw) const LANMOLA_DRAW_SPR_OFFS: [u8; 4] = [76, 60, 44, 28];

pub(in crate::zelda_rtl::sprite_main_draw) const LANMOLA_DRAW_CHARS_1: [u8; 16] = [
    0xc4, 0xe2, 0xc2, 0xe0, 0xc0, 0xe0, 0xc2, 0xe2, 0xc4, 0xe2, 0xc2, 0xe0, 0xc0, 0xe0, 0xc2, 0xe2,
];

pub(in crate::zelda_rtl::sprite_main_draw) const LANMOLA_DRAW_CHARS_0: [u8; 16] = [
    0xcc, 0xe4, 0xca, 0xe6, 0xc8, 0xe6, 0xca, 0xe4, 0xcc, 0xe4, 0xca, 0xe6, 0xc8, 0xe6, 0xca, 0xe4,
];

pub(in crate::zelda_rtl::sprite_main_draw) const LANMOLA_DRAW_FLAGS: [u8; 16] = [
    0xc0, 0xc0, 0xc0, 0xc0, 0x80, 0x80, 0x80, 0x80, 0, 0, 0, 0, 0x40, 0x40, 0x40, 0x40,
];

pub(in crate::zelda_rtl::sprite_main_draw) const LANMOLA_DRAW_BODY_FRAME_INDICES: [u8; 16] =
    [4, 5, 4, 5, 4, 5, 4, 5, 4, 3, 2, 2, 1, 1, 0, 0];

pub(in crate::zelda_rtl::sprite_main_draw) const LANMOLA_DRAW_BODY_CHARS: [u8; 6] = [0xee, 0xee, 0xec, 0xec, 0xce, 0xce];

pub(in crate::zelda_rtl::sprite_main_draw) const LANMOLA_DRAW_BODY_FLAGS: [u8; 6] = [0, 0x40, 0, 0x40, 0, 0x40];

pub(in crate::zelda_rtl::sprite_main_draw) const LANMOLA_DRAW_SPLASH_X_OFFSETS: [i8; 8] = [-8, 8, -10, 10, -16, 16, -24, 32];

pub(in crate::zelda_rtl::sprite_main_draw) const LANMOLA_DRAW_SPLASH_Y_OFFSETS: [i8; 8] = [0, 0, -1, -1, -1, -1, 3, 3];

pub(in crate::zelda_rtl::sprite_main_draw) const LANMOLA_DRAW_SPLASH_CHARS: [u8; 8] =
    [0xe8, 0xe8, 0xe8, 0xe8, 0xea, 0xea, 0xea, 0xea];

pub(in crate::zelda_rtl::sprite_main_draw) const LANMOLA_DRAW_SPLASH_FLAGS: [u8; 8] = [0, 0x40, 0, 0x40, 0, 0x40, 0, 0x40];

pub(in crate::zelda_rtl::sprite_main_draw) const LANMOLA_DRAW_SPLASH_SIZES: [u8; 8] = [2, 2, 2, 2, 2, 2, 0, 0];
